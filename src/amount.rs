//! The decimal value behind every amount, held as digit strings.

use crate::Error;

/// Largest number of significant integer digits that can be read aloud.
///
/// 306 digits is 102 groups of three, reaching the top of the scale names:
/// group index 101 is `sentilyon`, that is 10^303.
pub const MAX_INT_DIGITS: usize = 306;

/// Largest number of fraction digits accepted from user input.
pub const MAX_FRAC_DIGITS: usize = 8;

/// How many decimal places the 2005 redenomination moved.
const REDENOMINATION_SHIFT: usize = 6;

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
/// [`MAX_FRAC_DIGITS`] digits from input, plus the six a down-shift can add.
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
                .saturating_add(self.int.len().checked_div(3).unwrap_or(0))
                .saturating_add(self.frac.len())
                .saturating_add(2),
        );

        if self.sign == Sign::Negative
        {
            out.push('-');
        }

        let (head, tail) = {
            let head_len = self.int.len().checked_rem(3).unwrap_or(0);
            self.int.split_at(head_len)
        };

        out.push_str(head);

        let mut is_first_group = head.is_empty();
        for chunk in tail.as_bytes().chunks(3)
        {
            if is_first_group
            {
                is_first_group = false;
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

    /// Builds an amount from digit strings, canonicalising both parts.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotANumber`] when a part holds a non-digit, and
    /// [`Error::TooManyDigits`] when the integer part is longer than
    /// [`MAX_INT_DIGITS`].
    pub(crate) fn from_parts(
        sign: Sign,
        int: &str,
        frac: &str,
    ) -> Result<Self, Error>
    {
        if !int
            .chars()
            .all(|character| character.is_ascii_digit()) ||
            !frac
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            return Err(Error::NotANumber);
        }

        let int = {
            let trimmed = int.trim_start_matches('0');
            if trimmed.is_empty() { "0" } else { trimmed }
        };

        if int.len() > MAX_INT_DIGITS
        {
            return Err(Error::TooManyDigits);
        }

        let frac = frac.trim_end_matches('0');
        let sign = if int == "0" && frac.is_empty()
        {
            Sign::Positive
        }
        else
        {
            sign
        };
        Ok(Self::assemble(sign, int, frac))
    }

    /// Parses digits written the Turkish way, also accepting a foreign decimal
    /// dot.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotANumber`] when the text is not an amount,
    /// [`Error::ForeignSeparators`] or [`Error::BadGrouping`] when separators
    /// do not follow Turkish convention, [`Error::TooManyDigits`] above
    /// [`MAX_INT_DIGITS`] integer digits and [`Error::FractionTooLong`]
    /// above [`MAX_FRAC_DIGITS`] fraction digits.
    pub fn parse_tr(input: &str) -> Result<Self, Error>
    {
        let (sign, body) = {
            let trimmed = input.trim();
            split_sign(trimmed)
        };

        let allowed = |ch: char| ch.is_ascii_digit() || ch == '.' || ch == ',';

        if body.is_empty() || !body.chars().all(allowed)
        {
            return Err(Error::NotANumber);
        }

        if !body
            .chars()
            .any(|character| character.is_ascii_digit())
        {
            return Err(Error::NotANumber);
        }

        let parts = split_separators(body)?;
        if parts.frac.len() > MAX_FRAC_DIGITS
        {
            return Err(Error::FractionTooLong);
        }
        Self::from_parts(sign, &parts.int, &parts.frac)
    }

    /// Moves the decimal point six places: the 2005 redenomination factor.
    ///
    /// `up` multiplies by 10^6 (new lira to old lira), `false` divides (old
    /// lira to new lira). The shift is exact and never rounds.
    ///
    /// A down-shift can widen the fraction beyond the [`MAX_FRAC_DIGITS`]
    /// input limit, by up to the six places the shift moves.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyDigits`] when the shifted integer part would
    /// exceed [`MAX_INT_DIGITS`] digits.
    pub fn shifted_by_million(&self, up: bool) -> Result<Self, Error>
    {
        if up
        {
            let take = REDENOMINATION_SHIFT.min(self.frac.len());
            let (moved, rest) = self.frac.split_at(take);
            let padding = "0".repeat(REDENOMINATION_SHIFT.saturating_sub(take));
            let int = format!("{}{}{}", self.int, moved, padding);
            Self::from_parts(self.sign, &int, rest)
        }
        else
        {
            let take = REDENOMINATION_SHIFT.min(self.int.len());
            let split = self.int.len().saturating_sub(take);
            let (head, moved) = self.int.split_at(split);
            let padding = "0".repeat(REDENOMINATION_SHIFT.saturating_sub(take));
            let frac = format!("{}{}{}", padding, moved, self.frac);
            Self::from_parts(self.sign, head, &frac)
        }
    }

    /// Rounds the fraction to `frac_digits` digits, half away from zero.
    ///
    /// Returns the rounded amount and, when digits were dropped, those digits
    /// written at their own place value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyDigits`] when rounding up carries past
    /// [`MAX_INT_DIGITS`] integer digits.
    pub fn rounded_to(
        &self,
        frac_digits: u8,
    ) -> Result<(Self, Option<String>), Error>
    {
        let keep_len = usize::from(frac_digits).min(self.frac.len());
        let (keep, dropped) = self.frac.split_at(keep_len);

        if dropped.is_empty()
        {
            return Ok((self.clone(), None));
        }

        let rounds_up = dropped
            .as_bytes()
            .first()
            .is_some_and(|byte| *byte >= b'5');

        let (int, frac) = if rounds_up
        {
            increment(self.int.as_str(), keep)
        }
        else
        {
            (self.int.clone(), keep.to_owned())
        };

        let amount = Self::from_parts(self.sign, &int, &frac)?;
        let places = "0".repeat(keep_len);
        Ok((amount, Some(format!("0,{places}{dropped}"))))
    }

    /// The integer digits, without a sign.
    pub(crate) const fn int_digits(&self) -> &str
    {
        self.int.as_str()
    }

    /// The fraction digits, without a leading separator.
    pub(crate) const fn frac_digits(&self) -> &str
    {
        self.frac.as_str()
    }

    /// The sign of the amount.
    pub(crate) const fn sign(&self) -> Sign
    {
        self.sign
    }
}

/// An integer part and a fraction part, both holding only digits.
struct Separated
{
    /// Integer digits, with thousands separators removed.
    int: String,
    /// Fraction digits, empty when there is none.
    frac: String,
}

/// Splits a leading sign off an amount.
fn split_sign(input: &str) -> (Sign, &str)
{
    if let Some(rest) = input.strip_prefix('-')
    {
        (Sign::Negative, rest)
    }
    else if let Some(rest) = input.strip_prefix('+')
    {
        (Sign::Positive, rest)
    }
    else
    {
        (Sign::Positive, input)
    }
}

/// Splits digits into an integer and a fraction part, following the dual-format
/// rule.
fn split_separators(body: &str) -> Result<Separated, Error>
{
    let Some((int_part, frac)) = body.split_once(',')
    else
    {
        return split_without_comma(body);
    };

    if frac.contains(',') || frac.contains('.')
    {
        return Err(Error::ForeignSeparators);
    }

    let int = strip_thousands(int_part, Error::BadGrouping)?;
    Ok(Separated {
        int,
        frac: frac.to_owned(),
    })
}

/// Splits an amount that has no comma.
///
/// A lone dot is a thousands separator when it is followed by exactly three
/// digits *and* the whole text groups correctly; otherwise it is the decimal
/// mark. Two or more dots must all be thousands separators.
fn split_without_comma(body: &str) -> Result<Separated, Error>
{
    let Some((int_part, tail)) = body.split_once('.')
    else
    {
        return Ok(Separated {
            int: body.to_owned(),
            frac: String::new(),
        });
    };

    if tail.contains('.')
    {
        let int = strip_thousands(body, Error::ForeignSeparators)?;
        return Ok(Separated {
            int,
            frac: String::new(),
        });
    }

    if tail.len() == 3 && is_valid_grouping(body)
    {
        return Ok(Separated {
            int: remove_dots(body),
            frac: String::new(),
        });
    }

    if int_part.is_empty()
    {
        return Err(Error::NotANumber);
    }

    Ok(Separated {
        int: int_part.to_owned(),
        frac: tail.to_owned(),
    })
}

/// Checks that every dot separates groups of three digits, the first group
/// being 1-3.
fn is_valid_grouping(text: &str) -> bool
{
    let mut groups = text.split('.');

    let first_ok = groups.next().is_some_and(|first| {
        (1..=3).contains(&first.len()) &&
            first
                .chars()
                .all(|character| character.is_ascii_digit())
    });

    first_ok &&
        groups.all(|group| {
            group.len() == 3 &&
                group
                    .chars()
                    .all(|character| character.is_ascii_digit())
        })
}

/// Validates thousands separators and returns the digits without dots.
fn strip_thousands(text: &str, error: Error) -> Result<String, Error>
{
    if text.is_empty()
    {
        return Err(Error::NotANumber);
    }

    if !text
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        return Err(Error::NotANumber);
    }

    if text.contains('.')
    {
        if !is_valid_grouping(text)
        {
            return Err(error);
        }
        return Ok(text
            .chars()
            .filter(|character| *character != '.')
            .collect());
    }
    Ok(text.to_owned())
}

/// Removes thousands dots from a text already proven to be well grouped.
fn remove_dots(text: &str) -> String
{
    text.chars()
        .filter(|character| *character != '.')
        .collect()
}

/// Adds one to the last digit of `int` + `frac`, carrying leftwards.
fn increment(int: &str, frac: &str) -> (String, String)
{
    let mut digits: Vec<u8> = int
        .bytes()
        .chain(frac.bytes())
        .map(|byte| byte.saturating_sub(b'0'))
        .collect();

    let mut index = digits.len().saturating_sub(1);
    loop
    {
        let digit = digits.get(index).copied().unwrap_or(0);
        if digit < 9
        {
            if let Some(slot) = digits.get_mut(index)
            {
                *slot = digit.saturating_add(1);
            }
            break;
        }

        if let Some(slot) = digits.get_mut(index)
        {
            *slot = 0;
        }

        if index == 0
        {
            digits.insert(0, 1);
            break;
        }
        index = index.saturating_sub(1);
    }
    let all: String = digits
        .iter()
        .map(|digit| char::from(digit.saturating_add(b'0')))
        .collect();

    let (int_out, frac_out) = {
        let split = all.len().saturating_sub(frac.len());
        all.split_at(split)
    };

    (int_out.to_owned(), frac_out.to_owned())
}

#[cfg(test)]
mod tests
{
    //! Unit tests for the amount value and its Turkish formatting.

    use super::{Amount, MAX_FRAC_DIGITS, MAX_INT_DIGITS, Sign};
    use crate::Error;
    use crate::testing::digits;

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

    /// Accepts Turkish digits and a foreign-only decimal dot.
    #[test]
    fn parses_dual_format()
    {
        let cases = [
            ("1.250.000,75", "1.250.000,75"),
            ("1234567.89", "1.234.567,89"),
            ("1.234", "1.234"),
            ("1.2345", "1,2345"),
            ("1234.567", "1.234,567"),
            ("1.234.567", "1.234.567"),
            ("12,345", "12,345"),
            ("0,5", "0,5"),
            ("-5", "-5"),
            ("+5", "5"),
            ("1,", "1"),
        ];
        for (input, expected) in cases
        {
            assert_eq!(digits(input).grouped(), expected, "input: {input}");
        }
    }

    /// Rejects foreign and ambiguous separators, and overlong input.
    #[test]
    fn rejects_foreign_separators()
    {
        let rejected = [
            ("1,234,567.89", Error::ForeignSeparators),
            ("1.234.56", Error::ForeignSeparators),
            ("12.34.567", Error::ForeignSeparators),
            ("1,2.3", Error::ForeignSeparators),
            ("1 250", Error::NotANumber),
            ("", Error::NotANumber),
            ("abc", Error::NotANumber),
            ("..", Error::NotANumber),
            (",,", Error::NotANumber),
            (".,", Error::NotANumber),
            ("1.23,45", Error::BadGrouping),
        ];
        for (input, expected) in rejected
        {
            assert_eq!(
                super::Amount::parse_tr(input),
                Err(expected),
                "input: {input}"
            );
        }
    }

    /// Strips leading integer zeros, trailing fraction zeros and a negative
    /// zero.
    #[test]
    fn canonicalises_digits()
    {
        assert_eq!(digits("007,500").grouped(), "7,5");
        assert_eq!(digits("-0,0").grouped(), "0");
        assert_eq!(digits("0,50").grouped(), "0,5");
    }

    /// Rejects letters, oversized integers and overlong fractions.
    #[test]
    fn rejects_bad_digits()
    {
        assert_eq!(super::Amount::parse_tr("1a"), Err(Error::NotANumber));
        assert_eq!(
            super::Amount::parse_tr(
                &"9".repeat(MAX_INT_DIGITS.saturating_add(1))
            ),
            Err(Error::TooManyDigits)
        );
        assert_eq!(
            super::Amount::parse_tr(&format!(
                "1,{}",
                "1".repeat(MAX_FRAC_DIGITS.saturating_add(1))
            )),
            Err(Error::FractionTooLong)
        );
        assert_eq!(
            digits(&"9".repeat(MAX_INT_DIGITS))
                .grouped()
                .matches('9')
                .count(),
            MAX_INT_DIGITS,
            "the maximum integer length must be accepted"
        );
        let longest_frac = format!("1,{}", "1".repeat(MAX_FRAC_DIGITS));
        assert_eq!(
            digits(&longest_frac).grouped(),
            longest_frac,
            "the maximum fraction length must be accepted"
        );
    }

    /// Shifts six decimal places both ways, exactly.
    #[test]
    fn shifts_across_the_redenomination()
    {
        let old = super::Amount::parse_tr("1.250.000,75")
            .expect("input should parse");
        let new = old
            .shifted_by_million(false)
            .expect("shift should fit");
        assert_eq!(new.grouped(), "1,25000075");
        assert_eq!(
            new.shifted_by_million(true)
                .expect("shift back")
                .grouped(),
            "1.250.000,75"
        );

        let small = super::Amount::parse_tr("5").expect("input should parse");
        let tiny = small
            .shifted_by_million(false)
            .expect("shift should fit");
        assert_eq!(tiny.grouped(), "0,000005");
        assert_eq!(
            tiny.shifted_by_million(true)
                .expect("shift back")
                .grouped(),
            "5"
        );

        let ceiling = super::Amount::parse_tr(&"9".repeat(MAX_INT_DIGITS))
            .expect("input should parse");
        assert_eq!(ceiling.shifted_by_million(true), Err(Error::TooManyDigits));
    }

    /// Rounding a ceiling-sized amount past the integer limit is an error, not
    /// a panic and not a dropped digit.
    #[test]
    fn rounding_the_ceiling_past_the_limit_is_an_error()
    {
        let ceiling = format!("{},995", "9".repeat(MAX_INT_DIGITS));
        let amount =
            super::Amount::parse_tr(&ceiling).expect("input should parse");
        assert_eq!(amount.rounded_to(2), Err(Error::TooManyDigits));
    }

    /// Rounds half away from zero and reports the dropped digits.
    #[test]
    fn rounds_half_away_from_zero()
    {
        let cases = [
            ("1,25000075", "1,25", Some("0,00000075")),
            ("1,25", "1,25", None),
            ("1,005", "1,01", Some("0,005")),
            ("0,999", "1", Some("0,009")),
            ("1,004", "1", Some("0,004")),
            ("-1,005", "-1,01", Some("0,005")),
            ("9,999", "10", Some("0,009")),
        ];
        for (input, expected, remainder) in cases
        {
            let amount =
                super::Amount::parse_tr(input).expect("input should parse");
            let (rounded, dropped) = amount
                .rounded_to(2)
                .expect("rounding should fit");
            assert_eq!(rounded.grouped(), expected, "input: {input}");
            assert_eq!(dropped.as_deref(), remainder, "input: {input}");
        }
    }

    /// Rounds away every fraction digit when none is kept.
    #[test]
    fn rounds_to_whole_lira()
    {
        let amount =
            super::Amount::parse_tr("1,5").expect("input should parse");
        let (rounded, dropped) = amount
            .rounded_to(0)
            .expect("rounding should fit");
        assert_eq!(rounded.grouped(), "2");
        assert_eq!(dropped.as_deref(), Some("0,5"));
    }

    /// A down-shift can widen the fraction past the input limit.
    #[test]
    fn down_shift_widens_the_fraction()
    {
        let widest = format!("1,{}", "1".repeat(MAX_FRAC_DIGITS));
        let old = super::Amount::parse_tr(&widest).expect("input should parse");
        let new = old
            .shifted_by_million(false)
            .expect("shift should fit");
        assert_eq!(new.frac_digits(), "00000111111111");
        assert_eq!(
            new.frac_digits().len(),
            MAX_FRAC_DIGITS.saturating_add(super::REDENOMINATION_SHIFT),
            "a down-shift may build a wider fraction than input accepts"
        );
    }
}
