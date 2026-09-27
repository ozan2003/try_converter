# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-27

### Added

- Reading an amount aloud in Turkish: `1.250.000,75` becomes `bir milyon iki yüz elli bin lira,
  yetmiş beş kuruş`, with the two forms Turkish drops (`yüz`, not `bir yüz`; `bin`, not `bir bin`),
  an `eksi` prefix for negative amounts, an omitted kuruş clause when there is no kuruş, and an
  explicit `yaklaşık` mark plus the dropped digits whenever kuruş cannot hold the fraction.
- Parsing Turkish amount expressions back into digits: `2 milyon 500 bin lira`, `1.250.000,75 lira`
  and the app's own readings all round-trip, with `lira`/`kuruş`/`TL`/`TRY`/`kr` tokens,
  Turkish-aware case folding, an `eksi` or `-` sign, and a Turkish error naming the token it could not
  use instead of guessing.
- Exact six-place decimal shift across the 2005 redenomination (multiplying and dividing by
  10^6) that never rounds or truncates, so `1.250.000,75` old lira is exactly `1,25000075` new
  lira; and half-away-from-zero rounding to kuruş that reports the digits it dropped instead of
  hiding them.
- The 101 short-scale names Türkiye uses, `bin` (10^3) through `sentilyon` (10^303), plus the
  alternative spellings Turkish sources print (`undesilyon`/`andesilyon`, `seksdesilyon`/
  `sesvigintilyon` and 18 more) accepted on input. The 54 names no source tabulates are derived by
  that source's own rule and are marked as derived in the code, with the provenance recorded in the
  module docs.
- `Amount`: a decimal value held as a sign plus integer and fraction digit strings, so no amount
  is ever stored as a float. Canonical form (no leading integer zeros, no trailing fraction
  zeros, no negative zero) and a Turkish grouping renderer (`1.250.000,75`).
- Turkish dual-format parsing: `.` groups thousands and `,` starts the fraction; a lone dot
  followed by three digits counts as a thousands separator only when the whole text groups
  correctly, so `1.234` reads as `1234` and `1234567.89` as `1.234.567,89`, while mixed forms
  such as `1.234.56` and foreign forms such as `1,234,567.89` are rejected instead of guessed.
  Limits: 306 integer digits, 8 fraction digits.
- Turkish error messages for every parse failure, naming the offending token where there is one.
