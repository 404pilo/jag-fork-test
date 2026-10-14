//! The cost of a search (FEATURE-20): the input tokens of each answered
//! call, at the price of the model.
//!
//! No value here holds a text from the command line, a body, or the key.

use crate::config::price_of;

/// What one attempt adds to the bill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bill {
    /// The attempt got no answer (no HTTP 200). It adds nothing.
    NoAnswer,
    /// HTTP 200 with a count of input tokens that jag can read.
    Tokens(u64),
    /// HTTP 200 with no count that jag can read.
    NoCount,
}

/// The sums of all answered attempts of one search.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    /// The input tokens that the service gave.
    pub tokens: u64,
    /// The estimate for the attempts with no readable count.
    pub estimated_tokens: u64,
    /// The count of attempts with no readable count.
    pub no_count: u64,
}

impl Usage {
    /// Add one attempt. `body_bytes` is the size of the request body, for the
    /// estimate of decision 6: the bytes divided by 3, rounded down.
    pub fn add(&mut self, bill: Bill, body_bytes: usize) {
        match bill {
            Bill::NoAnswer => {}
            Bill::Tokens(tokens) => self.tokens = self.tokens.saturating_add(tokens),
            Bill::NoCount => {
                let estimate = u64::try_from(body_bytes).unwrap_or(u64::MAX) / 3;
                self.estimated_tokens = self.estimated_tokens.saturating_add(estimate);
                self.no_count = self.no_count.saturating_add(1);
            }
        }
    }
}

/// Why a search made no call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NoCallReason {
    /// The search ended before it had a count.
    #[default]
    NotStarted,
    /// Plain grep: no question.
    NoQuestion,
    /// The term matched nothing.
    NoMatch,
    /// The term matched, but jag could read no result.
    NoReadableMatch,
}

/// The cost of a search, in millionths of a US dollar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    /// Counts of the service, at the price of the model.
    Billed(u64),
    /// At least one answered attempt has an estimated count.
    Estimate(u64),
    /// The model has no price.
    NotAvailable,
    /// jag made no call.
    NoCall(NoCallReason),
}

impl Default for Cost {
    fn default() -> Self {
        Cost::NoCall(NoCallReason::NotStarted)
    }
}

impl Cost {
    /// The cost of `usage` for `model` (decision 7). The order is: no price,
    /// then an estimate, then billed.
    pub fn of(usage: &Usage, model: &str) -> Self {
        let Some(price) = price_of(model) else {
            return Cost::NotAvailable;
        };
        let tokens = usage.tokens.saturating_add(usage.estimated_tokens);
        let millionths = millionths(tokens.saturating_mul(price));
        if usage.no_count > 0 {
            Cost::Estimate(millionths)
        } else {
            Cost::Billed(millionths)
        }
    }
}

/// Round billionths to millionths, once. 500 or more goes up.
fn millionths(billionths: u64) -> u64 {
    billionths.saturating_add(500) / 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_MODEL;

    fn usage(tokens: u64, estimated_tokens: u64, no_count: u64) -> Usage {
        Usage {
            tokens,
            estimated_tokens,
            no_count,
        }
    }

    #[test]
    fn the_rounding_goes_up_at_500_billionths() {
        assert_eq!(millionths(0), 0);
        assert_eq!(millionths(499), 0);
        assert_eq!(millionths(500), 1);
        assert_eq!(millionths(1499), 1);
        assert_eq!(millionths(1500), 2);
        assert_eq!(millionths(u64::MAX), u64::MAX / 1000);
    }

    #[test]
    fn tokens_are_billed_at_the_price_of_the_model() {
        assert_eq!(
            Cost::of(&usage(48_000, 0, 0), DEFAULT_MODEL),
            Cost::Billed(2016)
        );
        assert_eq!(Cost::of(&Usage::default(), DEFAULT_MODEL), Cost::Billed(0));
    }

    #[test]
    fn a_sum_at_the_largest_number_does_not_overflow() {
        let cost = Cost::of(&usage(u64::MAX, u64::MAX, 0), DEFAULT_MODEL);
        assert_eq!(cost, Cost::Billed(u64::MAX / 1000));
    }

    #[test]
    fn an_attempt_with_no_count_makes_an_estimate() {
        assert_eq!(
            Cost::of(&usage(24_000, 8_000, 1), DEFAULT_MODEL),
            Cost::Estimate(1344)
        );
    }

    #[test]
    fn a_model_with_no_price_is_not_available_also_with_no_count() {
        assert_eq!(Cost::of(&usage(1, 0, 0), "jev-9.9.9"), Cost::NotAvailable);
        assert_eq!(Cost::of(&usage(1, 5, 2), "jev-9.9.9"), Cost::NotAvailable);
        assert_eq!(Cost::of(&Usage::default(), ""), Cost::NotAvailable);
    }

    #[test]
    fn add_follows_the_bill() {
        let mut usage = Usage::default();
        usage.add(Bill::NoAnswer, 900);
        assert_eq!(usage, Usage::default());
        usage.add(Bill::Tokens(10), 900);
        assert_eq!(usage, self::usage(10, 0, 0));
        usage.add(Bill::NoCount, 100);
        assert_eq!(usage, self::usage(10, 33, 1));
        usage.add(Bill::NoCount, 2);
        assert_eq!(usage, self::usage(10, 33, 2));
    }

    #[test]
    fn add_saturates() {
        let mut usage = usage(u64::MAX, u64::MAX, u64::MAX);
        usage.add(Bill::Tokens(1), 3);
        usage.add(Bill::NoCount, 3);
        assert_eq!(usage, self::usage(u64::MAX, u64::MAX, u64::MAX));
    }

    #[test]
    fn the_default_cost_is_no_call_not_started() {
        assert_eq!(Cost::default(), Cost::NoCall(NoCallReason::NotStarted));
    }
}
