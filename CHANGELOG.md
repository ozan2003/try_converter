# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `MAX_INPUT_BYTES` (8192): `interpret` and `parse_words` reject a longer raw input with
  `Error::InputTooLong` before they lowercase or split it, so a pasted value cannot make the
  per-frame parse work unbounded. The longest exact reading of a supported amount is 4096 bytes,
  so every readable amount still fits.

### Changed

- `interpret` lowercases the input once and hands the same text to the routing gate and the words
  parser, so the two cannot disagree about a token.

### Fixed

- A numeric `0` is an ordinary digit token: `5 lira 0`, `5 lira 0 kuruş` and `0 kuruş` are zero
  kuruş instead of `StrayZero`. Only the word `sıfır` must stand alone.
- A `kuruş` token next to a fractional literal is `ConflictingFraction` whether or not it carries
  a value: `1,50 lira kuruş` no longer parses as `1,5`, and `1,50 kuruş` no longer reports
  `KurusOutOfRange`.
- Only `,`, `.` and `·` are stripped from the ends of a token, so unsupported punctuation is no
  longer dropped silently: `bir! lira` is `UnknownWord` instead of `1`.
- A punctuation-only token is no longer classified as a number word when routing, so `1 .` reaches
  the digits parser and reports `NotANumber` instead of parsing as `1`.

## [0.1.2] - 2026-09-27

### Changed

- Replaced the `used_scales` array with a `BitArr` to reduce the memory footprint of the `Words` struct.

## [0.1.1] - 2026-09-27

### Added

- `README.md`, with the build steps, the input rules, the examples and the scale provenance, plus an
  MIT license (the manifest now carries `license = "MIT"`).

### Changed

- The default input era is now the new lira (TRY): the app opens anchored to the era people
  actually use today, and the selector still switches back to the old lira (TRL).
- Both output lines wrap. The largest representable amount (306 digits plus separators, about 408
  characters) is wider than any sensible window, and wrapping shows every digit instead of
  clipping the line.
- The package and the binary are named `try_conv` (were `try_trl_conv`), and the window id now
  follows the package name instead of being repeated as a literal.
- `Error` variants carrying a `String` now carry a `Box<str>` instead.

### Fixed

- A long amount no longer pushes the second era block and the footer notes out of the window: the
  content scrolls when it is taller than the window.
- The minimum window width fits the era selector row (520 logical points instead of 420, which cut
  the tail off `Yeni TL (TRY, 2009-)`).
- The typed era's reading is built once per frame instead of twice.
- `Amount::grouped` reserves room for the separators it inserts, so the largest amounts no longer
  reallocate the output string.
- The scale-table comments mark exactly the derived ranges (10^126..10^150 and so on, not the
  attested base stems), and the `value_digits(100)` test pins the whole 10^303 string rather than
  only its length.

## [0.1.0] - 2026-09-27

### Added

- Reading an amount aloud in Turkish: `1.250.000,75` becomes `bir milyon iki yüz elli bin lira,
  yetmiş beş kuruş`, with the two forms Turkish drops (`yüz`, not `bir yüz`; `bin`, not `bir bin`),
  an `eksi` prefix for negative amounts, an omitted kuruş clause when there is no kuruş, and an
  explicit `yaklaşık` mark plus the dropped digits whenever kuruş cannot hold the fraction.
- Parsing Turkish amount expressions back into digits: `2 milyon 500 bin lira`, `1.250.000,75 lira`
  and the app's own exact readings round-trip, while an approximate reading's `yaklaşık` mark is
  extra text and is not accepted as input, with `lira`/`kuruş`/`TL`/`TRY`/`kr` tokens,
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
- Input routing between the digits and the words paths, and the 2005 redenomination era model: an
  amount is anchored to the old lira (TRL, before 2005) or the new lira (TRY, from 2009), and the
  app shows both sides, multiplying back to old lira only when the typed amount is a new-lira one.
- An `eframe` desktop frontend (`src/main.rs`): an era selector, one input, the Turkish reading and
  both era blocks with their digits and readings, a red error line for anything that cannot parse,
  and the two footer notes about the six dropped zeros and the 2005–2008 `YTL` name.

### Changed

- Replaced the scaffold's `egui` dependency with `eframe 0.36.2`, which re-exports `egui`.
