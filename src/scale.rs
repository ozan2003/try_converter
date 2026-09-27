//! The 101 short-scale names used in Turkish, plus accepted spelling variants.

/// Number of scale names.
pub const SCALE_COUNT: usize = 101;

/// Scale names, index `x` reading 10^(3·(x+1)), so index 0 is `bin` and index
/// 100 is `sentilyon` (10^303).
///
/// Canonical spellings come from the Turkish Wikipedia table
/// <https://tr.wikipedia.org/wiki/B%C3%BCy%C3%BCk_say%C4%B1lar%C4%B1n_adlar%C4%B1>
/// (its extension table; the page states short scale is what Türkiye uses). The
/// 54 entries for indices 41-49, 51-59, 61-69, 71-79, 81-89 and 91-99 are
/// **derived — not tabulated by any Turkish source**; they follow that
/// article's own rule (unit root + tens root + `ilyon`) with the roots pinned
/// from the spellings the article attests, and are covered by
/// the `derived_names_follow_the_rule` test.
pub const SCALES: [&str; SCALE_COUNT] = [
    // 10^3 .. 10^30, tabulated by the source.
    "bin",
    "milyon",
    "milyar",
    "trilyon",
    "katrilyon",
    "kentilyon",
    "sekstilyon",
    "septilyon",
    "oktilyon",
    "nonilyon",
    // 10^33 .. 10^60, tabulated by the source (index 10 is andesilyon).
    "desilyon",
    "andesilyon",
    "dodesilyon",
    "tredesilyon",
    "katordesilyon",
    "kendesilyon",
    "seksdesilyon",
    "septendesilyon",
    "oktodesilyon",
    "novemdesilyon",
    // 10^63 .. 10^90, tabulated by the source.
    "vigintilyon",
    "anvigintilyon",
    "dovigintilyon",
    "tresvigintilyon",
    "katorvigintilyon",
    "kenvigintilyon",
    "sesvigintilyon",
    "septemvigintilyon",
    "oktovigintilyon",
    "novemvigintilyon",
    // 10^93 .. 10^120, tabulated by the source.
    "trigintilyon",
    "antrigintilyon",
    "dotrigintilyon",
    "trestrigintilyon",
    "katortrigintilyon",
    "kenkatrigintilyon",
    "sestrigintilyon",
    "septentrigintilyon",
    "oktotrigintilyon",
    "novemtrigintilyon",
    // 10^123 .. 10^150, derived: unit root + katraginta + ilyon.
    "katragintilyon",
    "ankatragintilyon",
    "dokatragintilyon",
    "treskatragintilyon",
    "katorkatragintilyon",
    "kenkatragintilyon",
    "seskatragintilyon",
    "septenkatragintilyon",
    "oktokatragintilyon",
    "novemkatragintilyon",
    // 10^153 .. 10^180, derived: kenkaginta root.
    "kenkagintilyon",
    "ankenkagintilyon",
    "dokenkagintilyon",
    "treskenkagintilyon",
    "katorkenkagintilyon",
    "kenkenkagintilyon",
    "seskenkagintilyon",
    "septenkenkagintilyon",
    "oktokenkagintilyon",
    "novemkenkagintilyon",
    // 10^183 .. 10^210, derived: seksaginta root.
    "seksagintilyon",
    "anseksagintilyon",
    "doseksagintilyon",
    "tresseksagintilyon",
    "katorseksagintilyon",
    "kenseksagintilyon",
    "sesseksagintilyon",
    "septenseksagintilyon",
    "oktoseksagintilyon",
    "novemseksagintilyon",
    // 10^213 .. 10^240, derived: septaginta root.
    "septagintilyon",
    "anseptagintilyon",
    "doseptagintilyon",
    "tresseptagintilyon",
    "katorseptagintilyon",
    "kenseptagintilyon",
    "sesseptagintilyon",
    "septenseptagintilyon",
    "oktoseptagintilyon",
    "novemseptagintilyon",
    // 10^243 .. 10^270, derived: oktoginta root.
    "oktogintilyon",
    "anoktogintilyon",
    "dooktogintilyon",
    "tresoktogintilyon",
    "katoroktogintilyon",
    "kenoktogintilyon",
    "sesoktogintilyon",
    "septenoktogintilyon",
    "oktooktogintilyon",
    "novemoktogintilyon",
    // 10^273 .. 10^300, derived: nonaginta root.
    "nonagintilyon",
    "annonagintilyon",
    "dononagintilyon",
    "tresnonagintilyon",
    "katornonagintilyon",
    "kennonagintilyon",
    "sesnonagintilyon",
    "septennonagintilyon",
    "oktononagintilyon",
    "novemnonagintilyon",
    // 10^303, tabulated by the source.
    "sentilyon",
];

/// Alternative spellings seen in Turkish sources, mapped to the canonical name
/// (`undesilyon` appears in kerimusta and webtekno, `andesilyon` in the
/// Wikipedia table).
pub const SYNONYMS: [(&str, &str); 20] = [
    ("undesilyon", "andesilyon"),
    ("sexdesilyon", "seksdesilyon"),
    ("kattuordesilyon", "katordesilyon"),
    ("unvigintilyon", "anvigintilyon"),
    ("trevigintilyon", "tresvigintilyon"),
    ("seksvigintilyon", "sesvigintilyon"),
    ("septenvigintilyon", "septemvigintilyon"),
    ("katortrisintilyon", "katortrigintilyon"),
    ("kenkatrisintilyon", "kenkatrigintilyon"),
    ("sekstrisintilyon", "sestrigintilyon"),
    ("septentrisintilyon", "septentrigintilyon"),
    ("oktotricintilyon", "oktotrigintilyon"),
    ("noventricintilyon", "novemtrigintilyon"),
    ("katracintilyon", "katragintilyon"),
    ("kenkacintilyon", "kenkagintilyon"),
    ("seksacintilyon", "seksagintilyon"),
    ("septacintilyon", "septagintilyon"),
    ("oktocintilyon", "oktogintilyon"),
    ("nonacintilyon", "nonagintilyon"),
    ("untrigintilyon", "antrigintilyon"),
];

/// Returns the `SCALES` index of a canonical name or of a synonym.
#[must_use]
pub fn index_of(name: &str) -> Option<usize>
{
    if let Some(index) = SCALES
        .iter()
        .position(|scale| *scale == name)
    {
        return Some(index);
    }
    SYNONYMS
        .iter()
        .find(|(alternative, _)| *alternative == name)
        .and_then(|(_, canonical)| {
            SCALES
                .iter()
                .position(|scale| scale == canonical)
        })
}

/// Returns `10^(3·(index+1))` written in digits, or `None` past the table.
#[must_use]
pub fn value_digits(index: usize) -> Option<String>
{
    if index >= SCALES.len()
    {
        return None;
    }
    let zeros = index.saturating_add(1).saturating_mul(3);
    let mut digits = String::with_capacity(zeros.saturating_add(1));
    digits.push('1');
    digits.extend(std::iter::repeat_n('0', zeros));
    Some(digits)
}

#[cfg(test)]
mod tests
{
    //! Unit tests for the scale table, its provenance and its lookup.

    use super::{SCALE_COUNT, SCALES, SYNONYMS, index_of, value_digits};

    /// The canonical table, in one string, so a typo cannot slip in unnoticed.
    const GOLDEN: &str =
        "bin milyon milyar trilyon katrilyon kentilyon sekstilyon septilyon \
         oktilyon nonilyon desilyon andesilyon dodesilyon tredesilyon \
         katordesilyon kendesilyon seksdesilyon septendesilyon oktodesilyon \
         novemdesilyon vigintilyon anvigintilyon dovigintilyon \
         tresvigintilyon katorvigintilyon kenvigintilyon sesvigintilyon \
         septemvigintilyon oktovigintilyon novemvigintilyon trigintilyon \
         antrigintilyon dotrigintilyon trestrigintilyon katortrigintilyon \
         kenkatrigintilyon sestrigintilyon septentrigintilyon \
         oktotrigintilyon novemtrigintilyon katragintilyon ankatragintilyon \
         dokatragintilyon treskatragintilyon katorkatragintilyon \
         kenkatragintilyon seskatragintilyon septenkatragintilyon \
         oktokatragintilyon novemkatragintilyon kenkagintilyon \
         ankenkagintilyon dokenkagintilyon treskenkagintilyon \
         katorkenkagintilyon kenkenkagintilyon seskenkagintilyon \
         septenkenkagintilyon oktokenkagintilyon novemkenkagintilyon \
         seksagintilyon anseksagintilyon doseksagintilyon tresseksagintilyon \
         katorseksagintilyon kenseksagintilyon sesseksagintilyon \
         septenseksagintilyon oktoseksagintilyon novemseksagintilyon \
         septagintilyon anseptagintilyon doseptagintilyon tresseptagintilyon \
         katorseptagintilyon kenseptagintilyon sesseptagintilyon \
         septenseptagintilyon oktoseptagintilyon novemseptagintilyon \
         oktogintilyon anoktogintilyon dooktogintilyon tresoktogintilyon \
         katoroktogintilyon kenoktogintilyon sesoktogintilyon \
         septenoktogintilyon oktooktogintilyon novemoktogintilyon \
         nonagintilyon annonagintilyon dononagintilyon tresnonagintilyon \
         katornonagintilyon kennonagintilyon sesnonagintilyon \
         septennonagintilyon oktononagintilyon novemnonagintilyon sentilyon";

    /// The 54 names no Turkish source tabulates, derived by the article's own
    /// rule.
    const DERIVED_UNITS: [&str; 9] = [
        "an", "do", "tres", "kator", "ken", "ses", "septen", "okto", "novem",
    ];

    /// Tens stems, in the order 40, 50, 60, 70, 80, 90. Each is the attested
    /// tens root (`katraginta`, `kenkaginta`, `seksaginta`, `septaginta`,
    /// `oktoginta`, `nonaginta`) with its final `a` elided before `ilyon`,
    /// matching the spellings the table prints.
    const DERIVED_TENS: [&str; 6] = [
        "katragint",
        "kenkagint",
        "seksagint",
        "septagint",
        "oktogint",
        "nonagint",
    ];

    /// The full table matches the reviewed transcription, entry for entry.
    #[test]
    fn table_matches_the_golden_transcription()
    {
        assert_eq!(SCALES.len(), SCALE_COUNT);
        assert_eq!(SCALES.join(" "), GOLDEN);
    }

    /// Every name is distinct.
    #[test]
    fn names_are_unique()
    {
        let mut sorted = SCALES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), SCALE_COUNT);
    }

    /// The 54 derived names follow the unit + tens + `ilyon` rule.
    #[test]
    fn derived_names_follow_the_rule()
    {
        for (tens_index, tens) in DERIVED_TENS.iter().enumerate()
        {
            let base = 40usize.saturating_add(tens_index.saturating_mul(10));
            for (unit_index, unit) in DERIVED_UNITS.iter().enumerate()
            {
                let index = base
                    .saturating_add(unit_index)
                    .saturating_add(1);
                let expected = format!("{unit}{tens}ilyon");
                assert_eq!(
                    SCALES.get(index),
                    Some(&expected.as_str()),
                    "scale index {index}"
                );
            }
        }
    }

    /// Ten to the power of three times the index plus three.
    #[test]
    fn values_are_powers_of_a_thousand()
    {
        assert_eq!(value_digits(0).as_deref(), Some("1000"));
        assert_eq!(value_digits(100).map(|digits| digits.len()), Some(304));
        assert_eq!(
            value_digits(100).map(|digits| digits.ends_with('1')),
            Some(false)
        );
        assert_eq!(value_digits(101), None);
    }

    /// Canonical names and synonyms both resolve, unknown words do not.
    #[test]
    fn lookup_accepts_synonyms()
    {
        assert_eq!(index_of("bin"), Some(0));
        assert_eq!(index_of("sentilyon"), Some(100));
        assert_eq!(index_of("undesilyon"), index_of("andesilyon"));
        assert_eq!(index_of("untrigintilyon"), index_of("antrigintilyon"));
        assert_eq!(index_of("milyarr"), None);
    }

    /// Every synonym resolves to the index of its canonical name, and that
    /// canonical name is itself in the table.
    #[test]
    fn every_synonym_matches_its_canonical_name()
    {
        for (alternative, canonical) in SYNONYMS
        {
            assert!(
                index_of(canonical).is_some(),
                "canonical name {canonical} is missing from SCALES"
            );
            assert_eq!(
                index_of(alternative),
                index_of(canonical),
                "alternative: {alternative} canonical: {canonical}"
            );
        }
    }
}
