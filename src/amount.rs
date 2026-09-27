//! The decimal value behind every amount, held as digit strings.

/// Largest number of significant integer digits that can be read aloud.
///
/// 306 digits is 102 groups of three, reaching the top of the scale names:
/// group index 101 is `sentilyon`, that is 10^303.
pub const MAX_INT_DIGITS: usize = 306;

/// Largest number of fraction digits accepted from user input.
pub const MAX_FRAC_DIGITS: usize = 8;

/// Sign of an amount.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign
{
    /// Zero or a positive amount.
    Positive,
    /// A negative amount.
    Negative,
}

/// A decimal amount held as digit strings, never as a float.
///
/// Invariants: `int` holds ASCII digits with no leading zero except the single
/// digit `0`; `frac` holds ASCII digits, never ends in `0`, and may be empty;
/// `int` has at most [`MAX_INT_DIGITS`] digits and `frac` at most
/// [`MAX_FRAC_DIGITS`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Amount
{
    /// Sign of the amount.
    sign: Sign,
    /// Integer digits, canonical.
    int: String,
    /// Fraction digits, canonical.
    frac: String,
}

impl Amount
{
    /// The amount zero.
    #[must_use]
    pub fn zero() -> Self
    {
        Self::assemble(Sign::Positive, "0", "")
    }

    /// Moves digit strings into a new amount without touching them.
    ///
    /// The caller guarantees the type's invariants; see the type documentation.
    fn assemble(sign: Sign, int: &str, frac: &str) -> Self
    {
        Self {
            sign,
            int: int.to_owned(),
            frac: frac.to_owned(),
        }
    }

    /// Formats the amount the Turkish way: `.` between thousands, `,` before
    /// the fraction.
    #[must_use]
    pub fn grouped(&self) -> String
    {
        let mut out = String::with_capacity(
            self.int
                .len()
                .saturating_add(self.frac.len())
                .saturating_add(4),
        );
        if self.sign == Sign::Negative
        {
            out.push('-');
        }
        let head_len = self.int.len().checked_rem(3).unwrap_or(0);
        let (head, tail) = self.int.split_at(head_len);
        out.push_str(head);
        let mut first_group = head.is_empty();
        for chunk in tail.as_bytes().chunks(3)
        {
            if first_group
            {
                first_group = false;
            }
            else
            {
                out.push('.');
            }
            for byte in chunk
            {
                out.push(char::from(*byte));
            }
        }
        if !self.frac.is_empty()
        {
            out.push(',');
            out.push_str(&self.frac);
        }
        out
    }
}

#[cfg(test)]
mod tests
{
    //! Unit tests for the amount value and its Turkish formatting.

    use super::{Amount, Sign};

    /// Formats an amount built straight from digits.
    fn grouped(sign: Sign, int: &str, frac: &str) -> String
    {
        Amount::assemble(sign, int, frac).grouped()
    }

    /// Groups thousands with dots and separates the fraction with a comma.
    #[test]
    fn groups_thousands_and_fraction()
    {
        assert_eq!(grouped(Sign::Positive, "0", ""), "0");
        assert_eq!(grouped(Sign::Positive, "999", ""), "999");
        assert_eq!(grouped(Sign::Positive, "1000", ""), "1.000");
        assert_eq!(grouped(Sign::Positive, "1250000", "75"), "1.250.000,75");
        assert_eq!(grouped(Sign::Positive, "12345678", ""), "12.345.678");
        assert_eq!(grouped(Sign::Negative, "1250000", "75"), "-1.250.000,75");
    }

    /// The zero amount formats as a single digit.
    #[test]
    fn zero_is_a_single_digit()
    {
        assert_eq!(Amount::zero().grouped(), "0");
    }
}
