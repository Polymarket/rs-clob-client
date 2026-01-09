use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::primitives::U256;
use chrono::{DateTime, Utc};
use rand::Rng as _;
use rust_decimal::prelude::ToPrimitive as _;

use crate::Result;
use crate::auth::Kind as AuthKind;
use crate::auth::state::Authenticated;
use crate::clob::Client;
use crate::clob::types::request::OrderBookSummaryRequest;
use crate::clob::types::{
    Amount, AmountInner, Order, OrderType, Side, SignableOrder, SignatureType,
};
use crate::error::Error;
use crate::types::{Address, Decimal};

pub(crate) const USDC_DECIMALS: u32 = 6;

/// Maximum number of decimal places for `size`
pub(crate) const LOT_SIZE_SCALE: u32 = 2;

/// Placeholder type for compile-time checks on limit order builders
#[non_exhaustive]
#[derive(Debug)]
pub struct Limit;

/// Placeholder type for compile-time checks on market order builders
#[non_exhaustive]
#[derive(Debug)]
pub struct Market;

/// Used to create an order iteratively and ensure validity with respect to its order kind.
#[derive(Debug)]
pub struct OrderBuilder<OrderKind, K: AuthKind> {
    pub(crate) client: Client<Authenticated<K>>,
    pub(crate) signer: Address,
    pub(crate) signature_type: SignatureType,
    pub(crate) salt_generator: fn() -> u64,
    pub(crate) token_id: Option<U256>,
    pub(crate) price: Option<Decimal>,
    pub(crate) size: Option<Decimal>,
    pub(crate) amount: Option<Amount>,
    pub(crate) side: Option<Side>,
    pub(crate) nonce: Option<u64>,
    pub(crate) expiration: Option<DateTime<Utc>>,
    pub(crate) taker: Option<Address>,
    pub(crate) order_type: Option<OrderType>,
    pub(crate) post_only: Option<bool>,
    pub(crate) funder: Option<Address>,
    pub(crate) _kind: PhantomData<OrderKind>,
}

impl<OrderKind, K: AuthKind> OrderBuilder<OrderKind, K> {
    /// Sets the `token_id` for this builder. This is a required field.
    #[must_use]
    pub fn token_id(mut self, token_id: U256) -> Self {
        self.token_id = Some(token_id);
        self
    }

    /// Sets the [`Side`] for this builder. This is a required field.
    #[must_use]
    pub fn side(mut self, side: Side) -> Self {
        self.side = Some(side);
        self
    }

    /// Sets the nonce for this builder.
    #[must_use]
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    #[must_use]
    pub fn expiration(mut self, expiration: DateTime<Utc>) -> Self {
        self.expiration = Some(expiration);
        self
    }

    #[must_use]
    pub fn taker(mut self, taker: Address) -> Self {
        self.taker = Some(taker);
        self
    }

    #[must_use]
    pub fn order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    /// Sets the `postOnly` flag for this builder.
    #[must_use]
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }
}

impl<K: AuthKind> OrderBuilder<Limit, K> {
    /// Sets the price for this limit builder. This is a required field.
    #[must_use]
    pub fn price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    /// Sets the size for this limit builder. This is a required field.
    #[must_use]
    pub fn size(mut self, size: Decimal) -> Self {
        self.size = Some(size);
        self
    }

    /// Validates and transforms this limit builder into a [`SignableOrder`]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(self), err(level = "warn"))
    )]
    pub async fn build(self) -> Result<SignableOrder> {
        let Some(token_id) = self.token_id else {
            return Err(Error::validation(
                "Unable to build Order due to missing token ID",
            ));
        };

        let Some(side) = self.side else {
            return Err(Error::validation(
                "Unable to build Order due to missing token side",
            ));
        };

        let Some(price) = self.price else {
            return Err(Error::validation(
                "Unable to build Order due to missing price",
            ));
        };

        if price.is_sign_negative() {
            return Err(Error::validation(format!(
                "Unable to build Order due to negative price {price}"
            )));
        }

        let fee_rate = self.client.fee_rate_bps(token_id).await?;
        let minimum_tick_size = self
            .client
            .tick_size(token_id)
            .await?
            .minimum_tick_size
            .as_decimal();

        let decimals = minimum_tick_size.scale();

        if price.scale() > minimum_tick_size.scale() {
            return Err(Error::validation(format!(
                "Unable to build Order: Price {price} has {} decimal places. Minimum tick size \
                {minimum_tick_size} has {} decimal places. Price decimal places <= minimum tick size decimal places",
                price.scale(),
                minimum_tick_size.scale()
            )));
        }

        if price < minimum_tick_size || price > Decimal::ONE - minimum_tick_size {
            return Err(Error::validation(format!(
                "Price {price} is too small or too large for the minimum tick size {minimum_tick_size}"
            )));
        }

        let Some(size) = self.size else {
            return Err(Error::validation(
                "Unable to build Order due to missing size",
            ));
        };

        if size.scale() > LOT_SIZE_SCALE {
            return Err(Error::validation(format!(
                "Unable to build Order: Size {size} has {} decimal places. Maximum lot size is {LOT_SIZE_SCALE}",
                size.scale()
            )));
        }

        if size.is_zero() || size.is_sign_negative() {
            return Err(Error::validation(format!(
                "Unable to build Order due to negative size {size}"
            )));
        }

        let nonce = self.nonce.unwrap_or(0);
        let expiration = self.expiration.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
        let taker = self.taker.unwrap_or(Address::ZERO);
        let order_type = self.order_type.unwrap_or(OrderType::GTC);
        let post_only = Some(self.post_only.unwrap_or(false));

        if !matches!(order_type, OrderType::GTD) && expiration > DateTime::<Utc>::UNIX_EPOCH {
            return Err(Error::validation(
                "Only GTD orders may have a non-zero expiration",
            ));
        }

        if post_only == Some(true) && !matches!(order_type, OrderType::GTC | OrderType::GTD) {
            return Err(Error::validation(
                "postOnly is only supported for GTC and GTD orders",
            ));
        }

        // When buying `YES` tokens, the user will "make" `size` * `price` USDC and "take"
        // `size` `YES` tokens, and vice versa for sells. We have to truncate the notional values
        // to the combined precision of the tick size _and_ the lot size. This is to ensure that
        // this order will "snap" to the precision of resting orders on the book. The returned
        // values are quantized to `USDC_DECIMALS`.
        //
        // e.g. User submits a limit order to buy 100 `YES` tokens at $0.34.
        // This means they will take/receive 100 `YES` tokens, make/give up 34 USDC. This means that
        // the `taker_amount` is `100000000` and the `maker_amount` of `34000000`.
        let (taker_amount, maker_amount) = match side {
            Side::Buy => (
                size,
                (size * price).trunc_with_scale(decimals + LOT_SIZE_SCALE),
            ),
            Side::Sell => (
                (size * price).trunc_with_scale(decimals + LOT_SIZE_SCALE),
                size,
            ),
            side => return Err(Error::validation(format!("Invalid side: {side}"))),
        };

        let salt = to_ieee_754_int((self.salt_generator)());

        let order = Order {
            salt: U256::from(salt),
            maker: self.funder.unwrap_or(self.signer),
            taker,
            tokenId: token_id,
            makerAmount: U256::from(to_fixed_u128(maker_amount)),
            takerAmount: U256::from(to_fixed_u128(taker_amount)),
            side: side as u8,
            feeRateBps: U256::from(fee_rate.base_fee),
            nonce: U256::from(nonce),
            signer: self.signer,
            expiration: U256::from(expiration.timestamp().to_u64().ok_or(Error::validation(
                format!("Unable to represent expiration {expiration} as a u64"),
            ))?),
            signatureType: self.signature_type as u8,
        };

        #[cfg(feature = "tracing")]
        tracing::debug!(token_id = %token_id, side = ?side, price = %price, size = %size, "limit order built");

        Ok(SignableOrder {
            order,
            order_type,
            post_only,
        })
    }
}

impl<K: AuthKind> OrderBuilder<Market, K> {
    /// Sets the price for this market builder. This is an optional field.
    #[must_use]
    pub fn price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    /// Sets the [`Amount`] for this market order. This is a required field.
    #[must_use]
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    // Attempts to calculate the market price from the top of the book for the particular token.
    // - Uses an orderbook depth search to find the cutoff price:
    //   - BUY + USDC: walk asks until notional >= USDC
    //   - BUY + Shares: walk asks until shares >= N
    //   - SELL + Shares: walk bids until shares >= N
    async fn calculate_price(&self, order_type: OrderType) -> Result<Decimal> {
        let token_id = self
            .token_id
            .expect("Token ID was already validated in `build`");
        let side = self.side.expect("Side was already validated in `build`");
        let amount = self
            .amount
            .as_ref()
            .expect("Amount was already validated in `build`");

        let book = self
            .client
            .order_book(&OrderBookSummaryRequest {
                token_id,
                side: None,
            })
            .await?;

        if !matches!(order_type, OrderType::FAK | OrderType::FOK) {
            return Err(Error::validation(
                "Cannot set an order type other than FAK/FOK for a market order",
            ));
        }

        let (levels, amount) = match side {
            Side::Buy => (book.asks, amount.0),
            Side::Sell => match amount.0 {
                a @ AmountInner::Shares(_) => (book.bids, a),
                AmountInner::Usdc(_) => {
                    return Err(Error::validation(
                        "Sell Orders must specify their `amount`s in shares",
                    ));
                }
            },

            side => return Err(Error::validation(format!("Invalid side: {side}"))),
        };

        let first = levels.first().ok_or(Error::validation(format!(
            "No opposing orders for {token_id} which means there is no market price"
        )))?;

        let mut sum = Decimal::ZERO;
        let cutoff_price = levels.iter().rev().find_map(|level| {
            match amount {
                AmountInner::Usdc(_) => sum += level.size * level.price,
                AmountInner::Shares(_) => sum += level.size,
            }
            (sum >= amount.as_inner()).then_some(level.price)
        });

        match cutoff_price {
            Some(price) => Ok(price),
            None if matches!(order_type, OrderType::FOK) => Err(Error::validation(format!(
                "Insufficient liquidity to fill order for {token_id} at {}",
                amount.as_inner()
            ))),
            None => Ok(first.price),
        }
    }

    /// Validates and transforms this market builder into a [`SignableOrder`]
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip(self), err(level = "warn"))
    )]
    pub async fn build(self) -> Result<SignableOrder> {
        let Some(token_id) = self.token_id else {
            return Err(Error::validation(
                "Unable to build Order due to missing token ID",
            ));
        };

        let Some(side) = self.side else {
            return Err(Error::validation(
                "Unable to build Order due to missing token side",
            ));
        };

        let amount = self
            .amount
            .ok_or_else(|| Error::validation("Unable to build Order due to missing amount"))?;

        let nonce = self.nonce.unwrap_or(0);
        let taker = self.taker.unwrap_or(Address::ZERO);

        let order_type = self.order_type.clone().unwrap_or(OrderType::FAK);
        let post_only = self.post_only;
        if post_only == Some(true) {
            return Err(Error::validation(
                "postOnly is only supported for limit orders",
            ));
        }
        let price = match self.price {
            Some(price) => price,
            None => self.calculate_price(order_type.clone()).await?,
        };

        let minimum_tick_size = self
            .client
            .tick_size(token_id)
            .await?
            .minimum_tick_size
            .as_decimal();
        let fee_rate = self.client.fee_rate_bps(token_id).await?;

        let decimals = minimum_tick_size.scale();

        // Ensure that the market price returned internally is truncated to our tick size
        let price = price.trunc_with_scale(decimals);
        if price < minimum_tick_size || price > Decimal::ONE - minimum_tick_size {
            return Err(Error::validation(format!(
                "Price {price} is too small or too large for the minimum tick size {minimum_tick_size}"
            )));
        }

        // When buying `YES` tokens, the user will "make" `USDC` dollars and "take"
        // `USDC` / `price` `YES` tokens. When selling `YES` tokens, the user will "make" `YES`
        // token shares, and "take" `YES` shares * `price`. We have to truncate the notional values
        // to the combined precision of the tick size _and_ the lot size. This is to ensure that
        // this order will "snap" to the precision of resting orders on the book. The returned
        // values are quantized to `USDC_DECIMALS`.
        //
        // e.g. User submits a market order to buy $100 worth of `YES` tokens at
        // the current `market_price` of $0.34. This means they will take/receive (100/0.34)
        // 294.1176(47) `YES` tokens, make/give up $100. This means that the `taker_amount` is
        // `294117600` and the `maker_amount` of `100000000`.
        //
        // e.g. User submits a market order to sell 100 `YES` tokens at the current
        // `market_price` of $0.34. This means that they will take/receive $34, make/give up 100
        // `YES` tokens. This means that the `taker_amount` is `34000000` and the `maker_amount` is
        // `100000000`.
        let raw_amount = amount.as_inner();

        let (taker_amount, maker_amount) = match (side, amount.0) {
            // Spend USDC to buy shares
            (Side::Buy, AmountInner::Usdc(_)) => {
                let shares = (raw_amount / price).trunc_with_scale(decimals + LOT_SIZE_SCALE);
                (shares, raw_amount)
            }

            // Buy N shares: use cutoff `price` derived from ask depth
            (Side::Buy, AmountInner::Shares(_)) => {
                let usdc = (raw_amount * price).trunc_with_scale(decimals + LOT_SIZE_SCALE);
                (raw_amount, usdc)
            }

            // Sell N shares for USDC
            (Side::Sell, AmountInner::Shares(_)) => {
                let usdc = (raw_amount * price).trunc_with_scale(decimals + LOT_SIZE_SCALE);
                (usdc, raw_amount)
            }

            (Side::Sell, AmountInner::Usdc(_)) => {
                return Err(Error::validation(
                    "Sell Orders must specify their `amount`s in shares",
                ));
            }

            (side, _) => return Err(Error::validation(format!("Invalid side: {side}"))),
        };

        let salt = to_ieee_754_int((self.salt_generator)());

        let order = Order {
            salt: U256::from(salt),
            maker: self.funder.unwrap_or(self.signer),
            taker,
            tokenId: token_id,
            makerAmount: U256::from(to_fixed_u128(maker_amount)),
            takerAmount: U256::from(to_fixed_u128(taker_amount)),
            side: side as u8,
            feeRateBps: U256::from(fee_rate.base_fee),
            nonce: U256::from(nonce),
            signer: self.signer,
            expiration: U256::ZERO,
            signatureType: self.signature_type as u8,
        };

        #[cfg(feature = "tracing")]
        tracing::debug!(token_id = %token_id, side = ?side, price = %price, amount = %amount.as_inner(), "market order built");

        Ok(SignableOrder {
            order,
            order_type,
            post_only: None,
        })
    }
}

/// Removes trailing zeros, truncates to [`USDC_DECIMALS`] decimal places, and quanitizes as an
/// integer.
fn to_fixed_u128(d: Decimal) -> u128 {
    d.normalize()
        .trunc_with_scale(USDC_DECIMALS)
        .mantissa()
        .to_u128()
        .expect("The `build` call in `OrderBuilder<S, OrderKind, K>` ensures that only positive values are being multiplied/divided")
}

/// Mask the salt to be <= 2^53 - 1, as the backend parses as an IEEE 754.
fn to_ieee_754_int(salt: u64) -> u64 {
    salt & ((1 << 53) - 1)
}

#[must_use]
#[expect(
    clippy::float_arithmetic,
    reason = "We are not concerned with precision for the seed"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "We are not concerned with truncation for a seed"
)]
#[expect(clippy::cast_sign_loss, reason = "We only need positive integers")]
pub(crate) fn generate_seed() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");

    let seconds = now.as_secs_f64();
    let rand = rand::rng().random::<f64>();

    (seconds * rand).round() as u64
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn to_fixed_u128_should_succeed() {
        assert_eq!(to_fixed_u128(dec!(123.456)), 123_456_000);
        assert_eq!(to_fixed_u128(dec!(123.456789)), 123_456_789);
        assert_eq!(to_fixed_u128(dec!(123.456789111111111)), 123_456_789);
        assert_eq!(to_fixed_u128(dec!(3.456789111111111)), 3_456_789);
        assert_eq!(to_fixed_u128(Decimal::ZERO), 0);
    }

    #[test]
    #[should_panic(
        expected = "The `build` call in `OrderBuilder<S, OrderKind, K>` ensures that only positive values are being multiplied/divided"
    )]
    fn to_fixed_u128_panics() {
        to_fixed_u128(dec!(-123.456));
    }

    #[test]
    fn order_salt_should_be_less_than_or_equal_to_2_to_the_53_minus_1() {
        let raw_salt = u64::MAX;
        let masked_salt = to_ieee_754_int(raw_salt);

        assert!(masked_salt < (1 << 53));
    }
}
