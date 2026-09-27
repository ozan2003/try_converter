# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-09-27

### Added

- `Amount`: a decimal value held as a sign plus integer and fraction digit strings, so no amount
  is ever stored as a float. Canonical form (no leading integer zeros, no trailing fraction
  zeros, no negative zero) and a Turkish grouping renderer (`1.250.000,75`).
- Turkish dual-format parsing: `.` groups thousands and `,` starts the fraction; a lone dot
  followed by three digits counts as a thousands separator only when the whole text groups
  correctly, so `1.234` reads as `1234` and `1234567.89` as `1.234.567,89`, while mixed forms
  such as `1.234.56` and foreign forms such as `1,234,567.89` are rejected instead of guessed.
  Limits: 306 integer digits, 8 fraction digits.
- Turkish error messages for every parse failure, naming the offending token where there is one.
