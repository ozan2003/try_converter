# AGENTS.md

## What this is

`try_conv` is a Rust desktop app that reads a Turkish lira amount aloud in Turkish and shows the
same amount in the other lira era. An amount written without a date is ambiguous: before 2005 it is
old lira (TRL), from 2009 new lira (TRY). The app shows both, so the reader does not have to guess.
The 2005 redenomination moved the decimal point six places.

The app keeps no files or settings and never touches the network. All the logic lives in the library
crate; `src/main.rs` is the eframe window around it, and the engine has no UI dependency.

## Commands

- `cargo run` builds and opens the window. eframe defaults to the wgpu renderer, so it wants a
  working GPU driver.
- `cargo test` runs the unit tests. They are deterministic, with no I/O and no UI.
- `cargo clippy --all-targets` must stay clean. `pedantic` is denied, `correctness` and `perf` are
  forbidden, and the manifest turns on a long list of restriction lints, so treat every warning as
  a finding.
- `cargo +nightly fmt` formats the code. `rustfmt.toml` sets `unstable_features = true`, so stable
  rustfmt refuses to run; `rust-toolchain.toml` pins stable for everything else.
- `cargo doc --no-deps` builds the API docs. They are warning-free with
  `RUSTDOCFLAGS="-D warnings"` today, and should stay that way.

## Layout

- `src/lib.rs`: `interpret` (the routing entry point), `Outcome`, `Error`, `Era`, `MAX_INPUT_BYTES`.
- `src/amount.rs`: `Amount`, held as a sign plus integer and fraction digit strings. `parse_tr`,
  `shifted_by_million`, `rounded_to`.
- `src/scale.rs`: the 101 short-scale names, `bin` (10^3) through `sentilyon` (10^303), plus the
  synonyms accepted on input.
- `src/words.rs`: `to_words_lira` and `parse_words`, plus the token-level `WordParser`.
- `src/main.rs`: the eframe frontend: era selector, one input field, both era blocks, a red error
  line.
- `assets/icon.png`: window icon, embedded with `include_bytes!`.
- `README.md`: user-facing documentation. `CHANGELOG.md`: Keep a Changelog format, new entries under
  `[Unreleased]`.
- `docs/superpowers/` and `.ooda/`: design spec, plan and review notes. Local only, not tracked.

## Rules the engine depends on

- Digits: at most 306 integer digits and 8 fraction digits (`MAX_INT_DIGITS`, `MAX_FRAC_DIGITS`).
  A larger value gets a Turkish error, never a truncation.
- Input: at most 8192 bytes (`MAX_INPUT_BYTES`), checked before any lowercasing or splitting. Longer
  input is `Error::InputTooLong`.
- Separators: `.` groups thousands, `,` starts the fraction. A lone dot counts as a thousands
  separator only when the whole text groups in threes; otherwise it is the decimal mark.
- No guessing and no silent drops: unreadable input returns an `Error`, naming the offending token
  where there is one.
- Amounts stay exact: digit strings rather than floats, and the six-place shift neither rounds nor
  truncates.
- Route new input handling through `interpret`. It lowercases once and passes the same text to the
  routing gate and the words parser, so both classify a token the same way.

## Conventions

- Code, comments and documentation in English. The window, the error messages and the scale names
  are Turkish.
- A function that can be `const` must be (`missing-const-for-fn` is denied).
- Tests sit next to the code, in `#[cfg(test)]` modules.
- Wrap temporary values in a block so they do not leak into the surrounding scope:
  `let x = { let y = 1; y + 2 };` instead of `let y = 1; let x = y + 2;`. `cargo +nightly fmt`
  expands the block onto its own lines.

## Documentation

The rustdoc book, "How to write documentation", is the reference. Each item follows that structure:
a one-line summary, a blank line, then the detail.

- Every public item carries a doc comment (`missing-docs` is on). The crate front page is the `//!`
  block at the top of `src/lib.rs`.
- The summary is one sentence ending in a period. It has to read on its own, because rustdoc reuses
  it in module listings and search results.
- Do not restate the signature. rustdoc already links the parameter and return types.
- A public function that returns `Result` gets an `# Errors` section naming the variants it can
  return; one that can panic gets `# Panics`. The library is panic-free outside its tests, so a new
  panic path is a design decision, not a missing section.
- Link other items by name, like ``[`Amount`]`` or ``[`interpret`]``, instead of pasting a URL.
- Docs are in English. Turkish is for what the app shows, including the error messages.
- Do not type en dashes, em dashes or ellipses by hand. rustdoc converts `--`, `---` and `...`
  itself, and does the same for straight quotes.
- Doc examples are compiled and run by `cargo test`, so an example doubles as a test. Add one where
  the summary is not enough to use the item.

## Instructions

- Cover a behavior change with a test case. The UI has no tests, so check UI changes by running the
  app.
- Do not commit or push unless explicitly asked. The maintainer does that.
- After every code update, explicitly assess whether a new version bump is needed.
