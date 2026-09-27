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
    /// A word is not a Turkish number word (carries the word).
    UnknownWord(String),
    /// Scale names are not strictly descending.
    ScaleOrder,
    /// A scale name is used twice (carries the name).
    DuplicateScale(String),
    /// A group value is outside 1–999.
    GroupOutOfRange,
    /// A kuruş value is outside 0–99.
    KurusOutOfRange,
    /// The `kuruş` token appears twice.
    DuplicateKurus,
    /// A leftover value after a scale name is 1000 or more.
    ResidueTooLarge,
    /// `sıfır` is used next to other tokens.
    StrayZero,
    /// A token is in a position where it cannot appear (carries the token).
    UnexpectedToken(String),
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
                f.write_str("Ölçekten önceki grup 1–999 arasında olmalı")
            },
            Self::KurusOutOfRange => f.write_str("Kuruş 0–99 arasında olmalı"),
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
