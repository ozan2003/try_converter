# TRY-TRL Converter

Reads a Turkish lira amount aloud in Turkish. Shows the same amount in the other lira era.

## Why this exists

On 2005-01-01, Turkey dropped six zeros from its currency. One new lira became equal to
1,000,000 old lira. The codes are `TRL` for the old lira and `TRY` for the new lira. From 2005 to
2008, the currency was called "Yeni Türk Lirası (YTL)". In 2009, the word "Yeni" was dropped.

An amount written without a date is ambiguous. A reader cannot tell which era it belongs to. This
app shows the other era next to the one you typed, so the ambiguity disappears.

## What it does

Pick the era the amount is written in, then type it. The app shows two things:

- The reading of the amount in Turkish, for example `bir milyon iki yüz elli bin lira, yetmiş beş kuruş`.
- The other era, with its exact digits and its reading.

The era you picked is the one you typed, so the app does not print it back.

The digits stay exact. If the fraction is longer than two digits, the reading rounds to kuruş and
adds `yaklaşık` with the digits it dropped.

The app never guesses. An input it cannot read produces a Turkish error message and no value.

## Screenshots

[![try-conv.png](https://i.postimg.cc/7Lj50npZ/try-conv.png)](https://postimg.cc/nsYVtBFy)

## Build and run

1. Install the stable Rust toolchain.
2. Run `cargo run`.
3. Type an amount in the input field.

The window and its messages are in Turkish. The app starts anchored to the new lira (TRY). If the
amount is an old-lira one, use the era selector.
The icon is embedded when `assets/icon.png` exists at build time. If it is missing or invalid, the
app still opens without a custom icon.

## Input

The input field accepts two forms.

Digits, in Turkish notation:

- `.` separates thousands and `,` starts the fraction: `1.250.000,75`.
- A single dot with three digits behind it is a thousands separator when the whole number groups
  correctly. So `1.234` is 1234, and `1234567.89` is `1.234.567,89`.
- A mixed form such as `1.234.56` is an error.

Turkish words:

- `2 milyon 500 bin lira`, `yüz iki lira`, `eksi beş lira`.
- The tokens `lira`, `kuruş`, `TL`, `TRY` and `kr` are optional.
- A leading `-` has the same meaning as the word `eksi`.
- `0` is an ordinary digit, so `5 lira 0` and `0 kuruş` are zero kuruş; only the word `sıfır`
  must stand alone.
- Only `,`, `.` and `·` around a token are ignored. Any other punctuation stays part of the
  token, so `bir! lira` is an error and not a silent `bir lira`.

## Examples

| Input | Reading |
| --- | --- |
| `1.250.000,75` | `bir milyon iki yüz elli bin lira, yetmiş beş kuruş` |
| `2 milyon 500 bin lira` | `iki milyon beş yüz bin lira` |
| `0,50` | `elli kuruş` |
| `101` | `yüz bir lira` |
| `-0,004` | `eksi sıfır lira · yaklaşık (0,004 yok sayıldı)` |

The same amount in the other era: `1.250.000,75` old lira is `1,25000075` new lira.

## Limits

- 306 integer digits and 8 fraction digits. A larger value is an error.
- Raw input is capped at 8192 bytes. The longest exact reading of a supported amount is 4096
  bytes, so every readable amount fits; a longer paste is an error instead of a stall.
- `sentilyon` (10^303) is the largest named scale, and the reason for the 306-digit limit.
- No exchange rates and no inflation adjustment. The app converts between the two eras by a factor
  of 10^6.
- No files and no settings.

## Scale names

The table holds 101 short-scale names, from `bin` (10^3) to `sentilyon` (10^303). Turkish sources
tabulate 47 of them. The other 54 names follow the rule in the Turkish Wikipedia article, and the
code marks them as derived. The source links are in `src/scale.rs`.

Twenty spelling variants that Turkish sources print, such as `undesilyon` beside `andesilyon`, are
accepted on input.

## Development

- `cargo test` runs all tests.
- `cargo clippy --all-targets` must stay clean. The lint levels are strict on purpose.
- `cargo +nightly fmt` formats the code, because `rustfmt.toml` uses unstable options.

## Acknowledgements

[Lira icons created by Md Tanvirul Haque - Flaticon](https://www.flaticon.com/free-icons/lira)

## License

MIT. See `LICENSE`.
