//! Turkish lira amount reader: reads amounts aloud in Turkish and shows the
//! 2005 redenomination equivalent.
//!
//! The engine has no UI dependency: [`amount`] holds the decimal value,
//! [`scale`] names the short-scale powers, and [`words`] reads an amount
//! aloud.

pub mod amount;
pub mod scale;
pub mod words;

pub use amount::{Amount, MAX_FRAC_DIGITS, MAX_INT_DIGITS, Sign};
pub use words::parse_words;

/// Which era an amount is written in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Era
{
    /// Old Turkish lira, the `TRL` code used before 2005 (six extra zeros).
    OldTrl,
    /// Turkish lira since 2009 (`TRY`); the same scale as 2005-2008 `YTL`.
    #[default]
    NewTry,
}

impl Era
{
    /// Returns the era an amount converts into.
    #[must_use]
    pub const fn other_era(self) -> Self
    {
        match self
        {
            Self::OldTrl => Self::NewTry,
            Self::NewTry => Self::OldTrl,
        }
    }

    /// Reports whether converting out of `era` multiplies by 10^6.
    ///
    /// New lira go back to old lira by multiplying; old lira come to new lira
    /// by dividing.
    #[must_use]
    pub const fn conversion_multiplies(self) -> bool
    {
        matches!(self, Self::NewTry)
    }
}

impl std::fmt::Display for Era
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::OldTrl => f.write_str("Eski TL (TRL)"),
            Self::NewTry => f.write_str("Yeni TL (TRY)"),
        }
    }
}

/// What raw input turned out to be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome
{
    /// The input held only whitespace.
    Idle,
    /// The input was read as digits.
    FromNumber(Amount),
    /// The input was read as Turkish words.
    FromWords(Amount),
}

/// Every way an amount can fail to parse or to be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error
{
    /// Thousands separators do not sit between groups of three digits.
    BadGrouping,
    /// Separators mix Turkish and foreign conventions.
    ForeignSeparators,
    /// The integer part is longer than [`MAX_INT_DIGITS`].
    TooManyDigits,
    /// The fraction is longer than [`MAX_FRAC_DIGITS`].
    FractionTooLong,
    /// The text is not an amount at all.
    NotANumber,
    /// The raw input is longer than [`MAX_INPUT_BYTES`].
    InputTooLong,
    /// A word is not a Turkish number word (carries the word).
    UnknownWord(Box<str>),
    /// Scale names are not strictly descending.
    ScaleOrder,
    /// A scale name is used twice (carries the name).
    DuplicateScale(Box<str>),
    /// A group value is outside 1-999.
    GroupOutOfRange,
    /// A kuruş value is outside 0-99.
    KurusOutOfRange,
    /// The `kuruş` token appears twice.
    DuplicateKurus,
    /// A leftover value after a scale name is 1000 or more.
    ResidueTooLarge,
    /// `sıfır` is used next to other tokens.
    StrayZero,
    /// A token is in a position where it cannot appear (carries the token).
    UnexpectedToken(Box<str>),
    /// A fractional digits literal is combined with a `kuruş` token.
    ConflictingFraction,
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            Self::BadGrouping =>
            {
                f.write_str("Binlik gruplar üçerli olmalı — ör. 1.250.000,75")
            },
            Self::ForeignSeparators => f.write_str(
                "Virgül ondalık ayırıcıdır, binlik ayırıcı noktadır — ör. \
                 1.250.000,75",
            ),
            Self::TooManyDigits => f.write_str(
                "En fazla 306 basamak okunabilir (en büyük ölçek adı: \
                 sentilyon, 10^303)",
            ),
            Self::FractionTooLong =>
            {
                f.write_str("Ondalık kısım en fazla 8 basamak olabilir")
            },
            Self::NotANumber =>
            {
                f.write_str("Rakam ya da sayı sözcüğü bekleniyordu")
            },
            Self::InputTooLong => write!(
                f,
                "Girdi çok uzun: en fazla {MAX_INPUT_BYTES} bayt okunabilir"
            ),
            Self::UnknownWord(word) =>
            {
                write!(f, "Bilinmeyen sözcük: \"{word}\"")
            },
            Self::ScaleOrder => f.write_str(
                "Ölçek adları büyükten küçüğe sıralanmalı — ör. \"2 milyon \
                 500 bin\"",
            ),
            Self::DuplicateScale(word) =>
            {
                write!(f, "Ölçek adı iki kez kullanılmış: \"{word}\"")
            },
            Self::GroupOutOfRange =>
            {
                f.write_str("Ölçekten önceki grup 1-999 arasında olmalı")
            },
            Self::KurusOutOfRange => f.write_str("Kuruş 0-99 arasında olmalı"),
            Self::DuplicateKurus => f.write_str("Kuruş iki kez kullanılmış"),
            Self::ResidueTooLarge => f.write_str(
                "Ölçekten sonra gelen artık 1000'den küçük olmalı — ör. \"2 \
                 milyon 500 bin\"",
            ),
            Self::StrayZero =>
            {
                f.write_str("\"sıfır\" yalnızca tek başına kullanılabilir")
            },
            Self::UnexpectedToken(token) =>
            {
                write!(f, "\"{token}\" bu konumda kullanılamaz")
            },
            Self::ConflictingFraction =>
            {
                f.write_str("Kesirli rakam ile \"kuruş\" birlikte kullanılamaz")
            },
        }
    }
}

/// Largest raw input the engine reads, in bytes.
///
/// The widest expression the engine supports is the exact reading of a
/// 306-digit amount, which is 4096 bytes; a fully grouped 306-digit amount with
/// its sign, comma and fraction is 411 bytes. Twice the longer of the two
/// leaves room for the punctuation and whitespace the parsers tolerate, and it
/// keeps a pasted value from making the per-frame parse work unbounded.
///
/// [`interpret`] and [`parse_words`] reject a longer input with
/// [`Error::InputTooLong`] before lowercasing or splitting it.
pub const MAX_INPUT_BYTES: usize = 8192;

/// Routes raw input to the digits path or the words path.
///
/// The words path runs when any token is a Turkish number word or a currency
/// token, so `2 milyon 500 bin`, `1.250.000,75 lira` and `1.250.000,75` all
/// reach the same amount.
///
/// # Errors
///
/// Returns whatever the chosen parser reports; see [`Amount::parse_tr`] and
/// [`parse_words`]. An input longer than [`MAX_INPUT_BYTES`] is
/// [`Error::InputTooLong`].
pub fn interpret(input: &str) -> Result<Outcome, Error>
{
    if input.len() > MAX_INPUT_BYTES
    {
        return Err(Error::InputTooLong);
    }

    if input.trim().is_empty()
    {
        return Ok(Outcome::Idle);
    }

    // Lowercased once (borrowed when there is nothing to lower) for both the
    // routing gate and the words parser, so the two cannot disagree about a
    // token.
    let lowered = words::turkish_lowercase(input);
    if words::looks_like_words(&lowered)
    {
        return words::parse_words_lowered(&lowered).map(Outcome::FromWords);
    }
    Amount::parse_tr(input).map(Outcome::FromNumber)
}

/// Helpers shared by the unit tests of this crate.
#[cfg(test)]
pub(crate) mod testing
{
    use crate::Amount;

    /// Parses digits, expecting success.
    pub fn digits(input: &str) -> Amount
    {
        Amount::parse_tr(input).expect("digits should parse")
    }
}

#[cfg(test)]
mod tests
{
    //! Unit tests for input routing and the redenomination era.

    use super::{Amount, Era, Outcome, interpret};

    /// Borrows the amount out of an outcome, if it holds one.
    fn amount_of(outcome: &Outcome) -> Option<&Amount>
    {
        match outcome
        {
            Outcome::Idle => None,
            Outcome::FromNumber(amount) | Outcome::FromWords(amount) =>
            {
                Some(amount)
            },
        }
    }

    /// Routes digits, words and empty input to the right path.
    #[test]
    fn routes_input()
    {
        assert_eq!(interpret("").expect("empty is idle"), Outcome::Idle);
        assert_eq!(interpret("   ").expect("blank is idle"), Outcome::Idle);
        assert_eq!(interpret("abc"), Err(super::Error::NotANumber));
        let digits = interpret("1.250.000,75").expect("digits should parse");
        assert!(matches!(digits, Outcome::FromNumber(_)), "got {digits:?}");
        for input in ["2 milyon 500 bin", "1.250.000,75 lira", "elli kuruş"]
        {
            let words = interpret(input).expect("words should parse");
            assert!(matches!(words, Outcome::FromWords(_)), "input: {input}");
        }
        let punctuated = interpret("5 lira.").expect("punctuation is ignored");
        assert!(
            matches!(punctuated, Outcome::FromWords(_)),
            "got {punctuated:?}"
        );
        // A punctuation-only token is not a number word, so a malformed
        // numeric input is not rescued by the words parser.
        assert_eq!(interpret("1 ."), Err(super::Error::NotANumber));
        assert_eq!(interpret("1 ,"), Err(super::Error::NotANumber));
        assert_eq!(
            interpret("bir! lira"),
            Err(super::Error::UnknownWord("bir!".into()))
        );
    }

    /// Both routing paths agree on the same amount.
    #[test]
    fn both_paths_agree()
    {
        let from_digits =
            interpret("1.250.000,75").expect("digits should parse");
        let from_words =
            interpret("1.250.000,75 lira").expect("words should parse");
        assert!(matches!(from_digits, Outcome::FromNumber(_)));
        assert!(matches!(from_words, Outcome::FromWords(_)));
        assert_eq!(amount_of(&from_digits), amount_of(&from_words));
    }

    /// The era mapping multiplies only when going from new lira back to old
    /// lira.
    #[test]
    fn maps_the_redenomination_direction()
    {
        assert_eq!(Era::OldTrl.other_era(), Era::NewTry);
        assert_eq!(Era::NewTry.other_era(), Era::OldTrl);
        assert!(!Era::OldTrl.conversion_multiplies());
        assert!(Era::NewTry.conversion_multiplies());
        assert_eq!(Era::default(), Era::NewTry);
    }

    /// Refuses an oversized input before any full-input transform, while the
    /// widest expression the engine supports still fits.
    #[test]
    fn bounds_the_input_size()
    {
        let too_long = "9".repeat(super::MAX_INPUT_BYTES.saturating_add(1));
        assert_eq!(interpret(&too_long), Err(super::Error::InputTooLong));

        let widest = crate::testing::digits(&"9".repeat(super::MAX_INT_DIGITS));
        assert_eq!(widest.int_digits().len(), super::MAX_INT_DIGITS);
        let reading = widest.to_words_lira().expect("should read");
        assert!(
            reading.len() <= super::MAX_INPUT_BYTES,
            "reading is {} bytes",
            reading.len()
        );
        assert_eq!(interpret(&reading), Ok(Outcome::FromWords(widest)));
    }
}
