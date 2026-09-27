//! Turkish words: reading an [`Amount`] aloud and reading an amount back.

use crate::Error;
use crate::amount::Amount;
use crate::scale::SCALES;

/// Number words for 0..=9; index 0 is unused so a digit can index the array.
const UNITS: [&str; 10] = [
    "", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz",
];

/// Number words for 0, 10, .., 90; index 0 is unused.
const TENS: [&str; 10] = [
    "", "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen",
    "doksan",
];

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
}
