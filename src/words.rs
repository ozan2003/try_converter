//! Turkish words: reading an [`Amount`] aloud and reading an amount back.

use crate::amount::Amount;
use crate::scale::{self, SCALES};
use crate::{Error, Sign};

/// Number words for 0..=9; index 0 is unused so a digit can index the array.
const UNITS: [&str; 10] = [
    "", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz",
];

/// Number words for 0, 10, .., 90; index 0 is unused.
const TENS: [&str; 10] = [
    "", "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen",
    "doksan",
];

/// How many 3-digit groups an integer part can have; group index 101 is
/// `sentilyon`.
const MAX_GROUPS: usize = 102;

/// Tokens that put the value before them on the lira side.
const LIRA_TOKENS: [&str; 5] = ["lira", "lirası", "türk", "tl", "try"];

/// Tokens that put the value before them on the kuruş side.
const KURUS_TOKENS: [&str; 3] = ["kuruş", "kuruşu", "kr"];

impl Amount
{
    /// Reads the amount aloud in Turkish, in lira and kuruş.
    ///
    /// The reading is exact while the fraction fits in two digits; otherwise it
    /// is rounded half away from zero to kuruş and marked `yaklaşık`, with the
    /// dropped digits spelled out. A negative amount is prefixed with `eksi`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::TooManyDigits`] when rounding to kuruş would carry past
    /// the integer digit limit.
    pub fn to_words_lira(&self) -> Result<String, Error>
    {
        let (rounded, dropped) = self.rounded_to(2)?;
        let mut clauses: Vec<String> = Vec::new();
        if rounded.int_digits() != "0"
        {
            clauses
                .push(format!("{} lira", int_to_words(rounded.int_digits())));
        }
        let kurus = kurus_of(rounded.frac_digits());
        if kurus > 0
        {
            clauses.push(format!("{} kuruş", group_words(u32::from(kurus))));
        }
        let mut reading = if clauses.is_empty()
        {
            String::from("sıfır lira")
        }
        else
        {
            clauses.join(", ")
        };
        if let Some(rest) = dropped
        {
            reading.push_str(" · yaklaşık (");
            reading.push_str(&rest);
            reading.push_str(" yok sayıldı)");
        }
        if self.sign() == crate::Sign::Negative
        {
            reading.insert_str(0, "eksi ");
        }
        Ok(reading)
    }
}

/// Reads an integer part in words, using the scale names for groups of three.
fn int_to_words(int: &str) -> String
{
    let groups = digit_groups(int);
    let top = groups.len().saturating_sub(1);
    let mut parts: Vec<String> = Vec::new();
    for (position, group) in groups.iter().enumerate()
    {
        let value = group.parse::<u32>().unwrap_or(0);
        if value == 0
        {
            continue;
        }
        let index = top.saturating_sub(position);
        let reading = group_words(value);
        if index == 0
        {
            parts.push(reading);
        }
        else if index == 1 && value == 1
        {
            parts.push(String::from("bin"));
        }
        else
        {
            let name = index.saturating_sub(1);
            if let Some(scale_name) = SCALES.get(name)
            {
                parts.push(format!("{reading} {scale_name}"));
            }
        }
    }
    parts.join(" ")
}

/// Splits ASCII digits into 3-digit groups, most significant first.
fn digit_groups(int: &str) -> Vec<&str>
{
    let head_len = int.len().checked_rem(3).unwrap_or(0);
    let (head, tail) = int.split_at(head_len);
    let mut groups: Vec<&str> = Vec::new();
    if !head.is_empty()
    {
        groups.push(head);
    }
    for chunk in tail.as_bytes().chunks(3)
    {
        groups.push(std::str::from_utf8(chunk).unwrap_or_default());
    }
    groups
}

/// Reads a value from 1 to 999, without a scale name.
fn group_words(value: u32) -> String
{
    let hundreds = value.checked_div(100).unwrap_or(0);
    let rest = value.checked_rem(100).unwrap_or(0);
    let tens = rest.checked_div(10).unwrap_or(0);
    let units = rest.checked_rem(10).unwrap_or(0);
    let mut parts: Vec<&str> = Vec::new();
    match hundreds
    {
        0 =>
        {},
        1 => parts.push("yüz"),
        other =>
        {
            if let Some(word) = UNITS.get(usize::try_from(other).unwrap_or(0))
            {
                parts.push(word);
            }
            parts.push("yüz");
        },
    }
    if let Some(word) = TENS.get(usize::try_from(tens).unwrap_or(0)) &&
        !word.is_empty()
    {
        parts.push(word);
    }
    if let Some(word) = UNITS.get(usize::try_from(units).unwrap_or(0)) &&
        !word.is_empty()
    {
        parts.push(word);
    }
    parts.join(" ")
}

/// Reads up to two fraction digits as a kuruş value.
fn kurus_of(frac: &str) -> u8
{
    let mut digits = frac.chars();
    let tens = digits
        .next()
        .and_then(|character| character.to_digit(10))
        .unwrap_or(0);
    let units = digits
        .next()
        .and_then(|character| character.to_digit(10))
        .unwrap_or(0);
    let value = tens.saturating_mul(10).saturating_add(units);
    u8::try_from(value).unwrap_or(0)
}

/// Which side of the reading the current tokens belong to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column
{
    /// The lira part.
    Integer,
    /// The kuruş part, read after a lira token.
    Kurus,
    /// After the `kuruş` token; no number words may follow.
    Closed,
}

/// Which slots of the current 3-digit group are already filled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GroupSlots
{
    /// A hundreds word is present.
    hundreds: bool,
    /// A tens word is present.
    tens: bool,
    /// A units word is present.
    units: bool,
}

/// Mutable state while walking the tokens of a words input.
struct WordParser
{
    /// Group values, indexed by `1000^i`.
    groups: [u32; MAX_GROUPS],
    /// Value of the group being read, if any.
    pending: Option<u32>,
    /// Slots already filled in the current group.
    filled: GroupSlots,
    /// Index of the scale used last, for the descending check.
    last_scale: Option<usize>,
    /// Bitset of scale indices already used.
    used_mask: [u64; 2],
    /// A digits literal standing for a whole side on its own.
    literal: Option<Amount>,
    /// The kuruş value, once known.
    kurus: Option<u32>,
    /// Which side the current tokens belong to.
    column: Column,
    /// Whether any scale name was consumed.
    any_scale: bool,
    /// Whether `sıfır` appeared.
    zero: bool,
    /// Whether a leading `eksi` was consumed.
    negative: bool,
    /// How many tokens other than a lone `sıfır` were consumed.
    other_tokens: usize,
    /// Index of the token being pushed, used to reject a misplaced `eksi`.
    token_index: usize,
}

impl WordParser
{
    /// Creates the parser state for a fresh input.
    fn start() -> Self
    {
        Self {
            groups: [0; MAX_GROUPS],
            pending: None,
            filled: GroupSlots::default(),
            last_scale: None,
            used_mask: [0; 2],
            literal: None,
            kurus: None,
            column: Column::Integer,
            any_scale: false,
            zero: false,
            negative: false,
            other_tokens: 0,
            token_index: 0,
        }
    }

    /// Feeds one lowercased token.
    fn push(&mut self, token: &str) -> Result<(), Error>
    {
        let index = self.token_index;
        self.token_index = index.saturating_add(1);
        let bare_zero = token == "sıfır" || token == "0";
        // The lira unit token carries no value of its own, so a lone `sıfır`
        // may keep it: `sıfır lira` is how the app reads zero aloud.
        let unit_token = LIRA_TOKENS.contains(&token);
        if !bare_zero && !unit_token
        {
            self.other_tokens = self.other_tokens.saturating_add(1);
        }
        if self.zero && self.other_tokens > 0
        {
            return Err(Error::StrayZero);
        }
        if token == "sıfır"
        {
            return self.push_zero();
        }
        if token == "eksi" || token == "-"
        {
            if index != 0 || self.negative
            {
                return Err(Error::UnexpectedToken(token.to_owned()));
            }
            self.negative = true;
            return Ok(());
        }
        if let Some(scale_index) = scale::index_of(token)
        {
            return self.push_scale(scale_index, token);
        }
        if LIRA_TOKENS.contains(&token)
        {
            return self.push_lira(token);
        }
        if KURUS_TOKENS.contains(&token)
        {
            return self.push_kurus();
        }
        if let Some((value, position)) = number_word(token)
        {
            return self.push_number_word(value, position, token);
        }
        if token
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
        {
            return self.push_literal(token);
        }
        Err(Error::UnknownWord(token.to_owned()))
    }

    /// Records `sıfır`, which may only stand alone.
    const fn push_zero(&mut self) -> Result<(), Error>
    {
        if self.other_tokens > 0 || self.zero
        {
            return Err(Error::StrayZero);
        }
        self.zero = true;
        Ok(())
    }

    /// Adds a number word to the current group.
    fn push_number_word(
        &mut self,
        value: u32,
        position: u8,
        token: &str,
    ) -> Result<(), Error>
    {
        if self.column == Column::Closed
        {
            return Err(Error::UnexpectedToken(token.to_owned()));
        }
        match position
        {
            2 =>
            {
                if self.filled.hundreds || self.filled.tens
                {
                    return Err(Error::GroupOutOfRange);
                }
                if self.filled.units
                {
                    let units = self.pending.unwrap_or(0);
                    if !(1..=9).contains(&units)
                    {
                        return Err(Error::GroupOutOfRange);
                    }
                    self.pending = Some(units.saturating_mul(100));
                    self.filled.units = false;
                }
                else
                {
                    self.pending = Some(100);
                }
                self.filled.hundreds = true;
            },
            1 =>
            {
                if self.filled.tens || self.filled.units
                {
                    return Err(Error::GroupOutOfRange);
                }
                self.pending = Some(
                    self.pending
                        .unwrap_or(0)
                        .saturating_add(value),
                );
                self.filled.tens = true;
            },
            _ =>
            {
                if self.filled.units
                {
                    return Err(Error::GroupOutOfRange);
                }
                self.pending = Some(
                    self.pending
                        .unwrap_or(0)
                        .saturating_add(value),
                );
                self.filled.units = true;
            },
        }
        Ok(())
    }

    /// Applies a scale name to the value read so far.
    fn push_scale(
        &mut self,
        scale_index: usize,
        token: &str,
    ) -> Result<(), Error>
    {
        if self.column != Column::Integer
        {
            return Err(Error::UnexpectedToken(token.to_owned()));
        }
        if self.literal.is_some()
        {
            return Err(Error::GroupOutOfRange);
        }
        if self.scale_used(scale_index)
        {
            return Err(Error::DuplicateScale(token.to_owned()));
        }
        if self
            .last_scale
            .is_some_and(|last| scale_index >= last)
        {
            return Err(Error::ScaleOrder);
        }
        let value = self.pending.unwrap_or(1);
        if !(1..=999).contains(&value)
        {
            return Err(Error::GroupOutOfRange);
        }
        if let Some(slot) = self
            .groups
            .get_mut(scale_index.saturating_add(1))
        {
            *slot = value;
        }
        self.mark_scale_used(scale_index);
        self.last_scale = Some(scale_index);
        self.any_scale = true;
        self.pending = None;
        self.filled = GroupSlots::default();
        Ok(())
    }

    /// Handles a lira-side token, which closes the integer part.
    fn push_lira(&mut self, token: &str) -> Result<(), Error>
    {
        match self.column
        {
            Column::Integer =>
            {
                self.commit_integer();
                self.column = Column::Kurus;
                Ok(())
            },
            Column::Kurus if self.pending.is_none() => Ok(()),
            Column::Kurus | Column::Closed =>
            {
                Err(Error::UnexpectedToken(token.to_owned()))
            },
        }
    }

    /// Handles the `kuruş` token, which closes the whole reading.
    fn push_kurus(&mut self) -> Result<(), Error>
    {
        match self.column
        {
            Column::Integer =>
            {
                if self.literal.is_some() || self.any_scale
                {
                    return Err(Error::KurusOutOfRange);
                }
                let value = self.pending.take().unwrap_or(0);
                self.filled = GroupSlots::default();
                self.set_kurus(value)?;
            },
            Column::Kurus =>
            {
                if let Some(value) = self.pending.take()
                {
                    self.filled = GroupSlots::default();
                    self.set_kurus(value)?;
                }
            },
            Column::Closed => return Err(Error::DuplicateKurus),
        }
        self.column = Column::Closed;
        Ok(())
    }

    /// Handles a token that starts with a digit.
    fn push_literal(&mut self, token: &str) -> Result<(), Error>
    {
        if self.column == Column::Closed
        {
            return Err(Error::UnexpectedToken(token.to_owned()));
        }
        let plain = token
            .chars()
            .all(|character| character.is_ascii_digit());
        if plain && token.len() <= 3
        {
            if self.pending.is_some()
            {
                return Err(Error::GroupOutOfRange);
            }
            let value = token.parse::<u32>().unwrap_or(0);
            if value == 0
            {
                return self.push_zero();
            }
            if self.column == Column::Kurus
            {
                return self.set_kurus(value);
            }
            self.pending = Some(value);
            self.filled = GroupSlots {
                hundreds: true,
                tens: true,
                units: true,
            };
            return Ok(());
        }
        if self.column == Column::Kurus
        {
            return Err(Error::KurusOutOfRange);
        }
        if self.any_scale || self.pending.is_some() || self.literal.is_some()
        {
            return Err(Error::ResidueTooLarge);
        }
        self.literal = Some(Amount::parse_tr(token)?);
        Ok(())
    }

    /// Files a leftover group value into the units group.
    fn commit_integer(&mut self)
    {
        if let Some(value) = self.pending.take() &&
            let Some(slot) = self.groups.get_mut(0)
        {
            *slot = value;
        }
        self.filled = GroupSlots::default();
    }

    /// Records the kuruş value, rejecting anything above 99.
    const fn set_kurus(&mut self, value: u32) -> Result<(), Error>
    {
        if value > 99
        {
            return Err(Error::KurusOutOfRange);
        }
        self.kurus = Some(value);
        Ok(())
    }

    /// Reports whether a scale index was used before.
    fn scale_used(&self, index: usize) -> bool
    {
        let word = index.checked_div(64).unwrap_or(0);
        let flag = scale_flag(index);
        self.used_mask
            .get(word)
            .is_some_and(|mask| mask & flag != 0)
    }

    /// Marks a scale index as used.
    fn mark_scale_used(&mut self, index: usize)
    {
        let word = index.checked_div(64).unwrap_or(0);
        let flag = scale_flag(index);
        if let Some(mask) = self.used_mask.get_mut(word)
        {
            *mask |= flag;
        }
    }

    /// Builds the amount from the collected state.
    fn finish(mut self) -> Result<Amount, Error>
    {
        if self.zero
        {
            return Ok(Amount::zero());
        }
        match self.column
        {
            Column::Integer => self.commit_integer(),
            Column::Kurus =>
            {
                if let Some(value) = self.pending.take()
                {
                    self.filled = GroupSlots::default();
                    self.set_kurus(value)?;
                }
            },
            Column::Closed =>
            {},
        }
        let sign = if self.negative
        {
            Sign::Negative
        }
        else
        {
            Sign::Positive
        };
        let int = match &self.literal
        {
            Some(amount) => amount.int_digits().to_owned(),
            None => groups_to_digits(&self.groups),
        };
        let frac = if let Some(amount) = &self.literal
        {
            if self.kurus.is_some()
            {
                return Err(Error::ConflictingFraction);
            }
            amount.frac_digits().to_owned()
        }
        else
        {
            match self.kurus
            {
                Some(value) => format!("{value:02}"),
                None => String::new(),
            }
        };
        Amount::from_parts(sign, &int, &frac)
    }
}

/// Builds the bit flag of a scale index inside its 64-bit word.
fn scale_flag(index: usize) -> u64
{
    let bit = u32::try_from(index.checked_rem(64).unwrap_or(0)).unwrap_or(0);
    1u64.checked_shl(bit).unwrap_or(0)
}

/// Writes group values as digits, most significant group first.
fn groups_to_digits(groups: &[u32]) -> String
{
    let Some(top) = groups.iter().rposition(|value| *value != 0)
    else
    {
        return String::from("0");
    };
    let mut digits = String::new();
    for index in (0..=top).rev()
    {
        let value = groups.get(index).copied().unwrap_or(0);
        if index == top
        {
            digits.push_str(&value.to_string());
        }
        else
        {
            let text = value.to_string();
            for _ in 0..3usize.saturating_sub(text.len())
            {
                digits.push('0');
            }
            digits.push_str(&text);
        }
    }
    digits
}

/// Maps a number word to its value and slot (2 = hundreds, 1 = tens, 0 =
/// units).
fn number_word(token: &str) -> Option<(u32, u8)>
{
    if token == "yüz"
    {
        return Some((100, 2));
    }
    if let Some(index) = UNITS.iter().position(|word| *word == token)
    {
        return u32::try_from(index)
            .ok()
            .map(|value| (value, 0));
    }
    TENS.iter()
        .position(|word| *word == token)
        .and_then(|index| u32::try_from(index).ok())
        .map(|value| (value.saturating_mul(10), 1))
}

/// Lowercases text with Turkish rules: `I` becomes `ı` and `İ` becomes `i`.
pub(crate) fn turkish_lowercase(input: &str) -> String
{
    input
        .chars()
        .map(|character| match character
        {
            'I' => 'ı',
            'İ' => 'i',
            other => other.to_lowercase().next().unwrap_or(other),
        })
        .collect()
}

/// Reads a Turkish amount expression back into an [`Amount`].
///
/// # Errors
///
/// Returns the first problem found: an unknown word ([`Error::UnknownWord`]),
/// a bad scale order ([`Error::ScaleOrder`], [`Error::DuplicateScale`]), an
/// out-of-range group or kuruş ([`Error::GroupOutOfRange`],
/// [`Error::ResidueTooLarge`], [`Error::KurusOutOfRange`]), a misplaced token
/// ([`Error::UnexpectedToken`], [`Error::StrayZero`]), a digits literal that
/// does not parse ([`Error::NotANumber`] and friends), or a fractional literal
/// combined with a `kuruş` token ([`Error::ConflictingFraction`]).
///
/// Surrounding punctuation (`,`, `.`, `·`) is ignored, so the readings this app
/// prints parse back unchanged; a leading `-` is a sign, like the word `eksi`.
pub fn parse_words(input: &str) -> Result<Amount, Error>
{
    let lowered = turkish_lowercase(input);
    let mut parser = WordParser::start();
    for raw in lowered.split_whitespace()
    {
        let trimmed = raw.trim_matches(|character: char| {
            !character.is_alphanumeric() && character != '-'
        });
        if trimmed.is_empty()
        {
            continue;
        }
        // A leading `-` means negative, exactly like the word `eksi`.
        let (marker, token) = match trimmed.strip_prefix('-')
        {
            Some(rest) => ("-", rest),
            None => ("", trimmed),
        };
        if !marker.is_empty()
        {
            parser.push(marker)?;
        }
        if !token.is_empty()
        {
            parser.push(token)?;
        }
    }
    parser.finish()
}

#[cfg(test)]
mod tests
{
    //! Unit tests for the two word directions.

    use crate::testing::digits;

    /// Reads whole amounts, including the special `yüz` and `bin` forms.
    #[test]
    fn reads_amounts_aloud()
    {
        let cases = [
            ("0", "sıfır lira"),
            ("100", "yüz lira"),
            ("1000", "bin lira"),
            ("1100", "bin yüz lira"),
            ("100.000", "yüz bin lira"),
            ("100.100", "yüz bin yüz lira"),
            ("1.000.001", "bir milyon bir lira"),
            (
                "1.250.000,75",
                "bir milyon iki yüz elli bin lira, yetmiş beş kuruş",
            ),
            ("0,50", "elli kuruş"),
            ("1,25", "bir lira, yirmi beş kuruş"),
            ("-5", "eksi beş lira"),
            ("101", "yüz bir lira"),
            ("999", "dokuz yüz doksan dokuz lira"),
            ("-0,004", "eksi sıfır lira · yaklaşık (0,004 yok sayıldı)"),
        ];
        for (input, expected) in cases
        {
            assert_eq!(
                digits(input)
                    .to_words_lira()
                    .expect("should read"),
                expected,
                "input: {input}"
            );
        }
    }

    /// Reads the extremes of the scale table.
    #[test]
    fn reads_the_scale_extremes()
    {
        let million_of_millions = format!("1{}", "0".repeat(303));
        let sentilyon = crate::testing::digits(&million_of_millions);
        assert_eq!(
            sentilyon
                .to_words_lira()
                .expect("should read"),
            "bir sentilyon lira"
        );

        let ankatragintilyon =
            crate::testing::digits(&format!("1{}", "0".repeat(126)));
        assert_eq!(
            ankatragintilyon
                .to_words_lira()
                .expect("should read"),
            "bir ankatragintilyon lira"
        );
    }

    /// Marks the words line as approximate when kuruş cannot hold the fraction.
    #[test]
    fn marks_approximate_readings()
    {
        assert_eq!(
            digits("3,14159")
                .to_words_lira()
                .expect("should read"),
            "üç lira, on dört kuruş · yaklaşık (0,00159 yok sayıldı)"
        );
    }

    /// Parses a words expression, expecting success.
    fn words(input: &str) -> crate::Amount
    {
        super::parse_words(input).expect("words should parse")
    }

    /// Every canonical name parses to its power of a thousand.
    #[test]
    fn parses_every_scale_name()
    {
        for (index, name) in crate::scale::SCALES.iter().enumerate()
        {
            let input = format!("bir {name}");
            let parsed = words(&input);
            let expected =
                crate::scale::value_digits(index).expect("index is in range");
            assert_eq!(parsed.int_digits(), expected, "input: {input}");
        }
    }

    /// Accepts the shapes people actually type.
    #[test]
    fn parses_tolerant_forms()
    {
        let cases = [
            ("iki yüz elli bin lira yetmiş beş kuruş", "250.000,75"),
            ("2 milyon 500 bin lira", "2.500.000"),
            ("bir yüz", "100"),
            ("1.250.000,75 lira", "1.250.000,75"),
            ("üç kuruş", "0,03"),
            ("250 lira 75", "250,75"),
            (
                "bir undesilyon",
                "1.000.000.000.000.000.000.000.000.000.000.000.000",
            ),
            ("eksi beş lira", "-5"),
            ("-5 lira", "-5"),
            ("yüz iki lira", "102"),
            ("iki yüz üç lira", "203"),
            ("sıfır lira", "0"),
        ];
        for (input, expected) in cases
        {
            assert_eq!(words(input).grouped(), expected, "input: {input}");
        }
    }

    /// Rejects malformed expressions with the documented error.
    #[test]
    fn rejects_malformed_expressions()
    {
        let cases = [
            (
                "iki milyon üç milyon",
                crate::Error::DuplicateScale(String::from("milyon")),
            ),
            ("bin bin", crate::Error::DuplicateScale(String::from("bin"))),
            ("üç milyon iki milyar", crate::Error::ScaleOrder),
            (
                "milyarr",
                crate::Error::UnknownWord(String::from("milyarr")),
            ),
            ("sıfır iki", crate::Error::StrayZero),
            ("sıfır kuruş", crate::Error::StrayZero),
            ("yüz kuruş", crate::Error::KurusOutOfRange),
            ("250 kuruş", crate::Error::KurusOutOfRange),
            ("75 kuruş kuruş", crate::Error::DuplicateKurus),
            (
                "75 kuruş 25",
                crate::Error::UnexpectedToken(String::from("25")),
            ),
            ("iki milyon 1500", crate::Error::ResidueTooLarge),
            ("2500 bin", crate::Error::GroupOutOfRange),
            (
                "iki lira üç milyon",
                crate::Error::UnexpectedToken(String::from("milyon")),
            ),
            (
                "üç kuruş iki",
                crate::Error::UnexpectedToken(String::from("iki")),
            ),
            ("1,50 lira 75 kuruş", crate::Error::ConflictingFraction),
            ("yüz yüz", crate::Error::GroupOutOfRange),
        ];
        for (input, expected) in cases
        {
            assert_eq!(
                super::parse_words(input),
                Err(expected),
                "input: {input}"
            );
        }
    }

    /// Words go out and come back unchanged.
    #[test]
    fn round_trips_through_words()
    {
        let corpus = [
            "0",
            "1",
            "100",
            "1000",
            "1.000.000",
            "999.999.999.999",
            "1.250.000,75",
            "0,03",
            "5.432.100.000.000.000.000.000.000.000",
            "1.000.000.000.000.000.000.000.000.000.000.000.000.000.000.000.\
             000.000.000.000",
        ];
        for input in corpus
        {
            let amount = crate::testing::digits(input);
            let reading = amount.to_words_lira().expect("should read");
            let parsed =
                super::parse_words(&reading).expect("reading should parse");
            assert_eq!(parsed, amount, "input: {input} (reading: {reading})");
        }
    }
}
