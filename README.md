# Lite Calculator · a native calculator written with GPUI

[中文](README.zh-CN.md) | English

A native desktop calculator built on [GPUI Kit](https://gpui-kit.com): one click between a standard and a scientific keypad, DEG/RAD toggle, dark and light themes, a live preview while you type, and physical-keyboard input. Pure Rust — no Electron, no WebView, the artifact is a single native binary.

## Features

- **Two keypads**: 19 standard keys; scientific mode adds a 24-key function panel and narrows the number pad, all inside the same window.
- **What you see is the expression**: the display shows a readable expression rather than internal state, postfix keys are written as `^2`, `^-1`, `!`, `mod`, and unbalanced parentheses are closed automatically on `=`.
- **Live preview**: the third line shows `= result` as you type; an incomplete or invalid expression never interrupts input.
- **DEG / RAD**: applies to `sin cos tan` and the inverse functions.
- **Dark / light theme**: one button in the title bar; every color comes from the GPUI theme (`background / muted / foreground / border / danger`).
- **Physical keyboard**: digits, operators, parentheses, `Enter`, `Backspace`, `Esc` work directly; letter keys trigger functions in scientific mode.
- **History and `Ans`**: the most recent expression and result are kept, `Ans` inserts the result into the current expression, and one button clears history.
- **Errors that mean something**: `invalid expression`, `division by zero`, `out of domain`, `out of range` — never a blank field or `NaN`.
- **Regression-tested**: `cargo test` runs 11 cases — 7 engine unit tests plus 4 headless GPUI UI integration tests that use hit-tested clicks and real key events.

## Quick start

```bash
cargo run              # debug build (opt-level = 1 is set for the dev profile; GPUI is slow without it)
cargo build --release  # release build
./scripts/build-app.sh # bundles dist/LiteCalculator.app and dist/*.zip
```

Requirements:

- Rust 1.85 or newer (see `rust-version` in `Cargo.toml`).
- Built and run on macOS 26.4 / Apple Silicon. GPUI itself also targets Linux and Windows, but this project has not been tested there.

## Interface

Top to bottom:

1. **Title bar**: app name, current mode label, theme toggle on the right.
2. **Display**: line 1 is the angle unit plus the previous record, the middle line is the expression or the result (font size adapts to length, long values ellipsize from the start), line 3 is the error, the `= preview`, or an input hint.
3. **Mode bar**: segmented `Standard / Scientific`, with `DEG/RAD` and `Clear history` on the right.
4. **Keypad**: in scientific mode the function panel appears above the number pad.

Standard pad:

| | | | |
| --- | --- | --- | --- |
| `AC` | `⌫` | `%` | `÷` |
| `7` | `8` | `9` | `×` |
| `4` | `5` | `6` | `−` |
| `1` | `2` | `3` | `+` |
| `±` | `0` | `.` | `=` |

Scientific panel (4 × 6):

| | | | | | |
| --- | --- | --- | --- | --- | --- |
| `sin` | `cos` | `tan` | `sin⁻¹` | `cos⁻¹` | `tan⁻¹` |
| `ln` | `log` | `log₂` | `eˣ` | `√` | `∛` |
| `x²` | `x³` | `xʸ` | `1/x` | `n!` | `mod` |
| `(` | `)` | `|x|` | `π` | `e` | `Ans` |

## Physical keyboard

| Key | Action |
| --- | --- |
| `0`–`9`, `.` | digits and decimal point |
| `+` `-` `*` `/` | arithmetic |
| `^` `%` `!` | power, percent, factorial |
| `(` `)` | parentheses |
| `Enter` or `=` | evaluate |
| `Backspace` | delete one whole token (`sin(`, `^2`, ` mod ` are removed as units) |
| `Esc` or `Delete` | clear everything |
| `π` | pi (`⌥G` on macOS) |

Scientific mode only: `s` sin · `c` cos · `t` tan · `l` ln · `g` log · `r` √ · `q` x² · `p` π.

Shortcuts with `⌘` / `⌥` / `Ctrl` are ignored so they never collide with system bindings.

## Expression syntax

Precedence, lowest to highest:

```
+ −            addition, subtraction
× ÷ mod        multiplication, division, modulo (includes implicit multiplication)
unary −        sign
^              exponent, right-associative
! %            postfix factorial and percent
atom           number / constant / function / parenthesized group
```

- **Implicit multiplication**: `2π`, `3(1+2)` and `2sin(30)` are treated as products.
- **Postfix keys write text**: `x²` → `^2`, `x³` → `^3`, `1/x` → `^-1`, `n!` → `!`, `%` divides by 100.
- **Functions**: `sin cos tan asin acos atan ln log log2 sqrt cbrt abs exp`; the engine also implements `sinh cosh tanh round floor ceil`, which have no key on the pad yet.
- **Constants**: `π`, `e`.
- **Auto-closing parentheses**: before evaluation the missing count of `)` is appended, so `sin(30` + `=` still computes.
- **Number formatting**: about 12 significant digits; values outside `1e-6`–`1e12` switch to scientific notation; factorial covers `0!` to `170!`.

## Tests

```bash
cargo test
```

Covered: operator precedence and implicit multiplication, display-glyph normalization, trigonometry under DEG and RAD, error classification, number formatting, the state machine (what happens when you keep typing after `=`), whole-token deletion — and on the UI side, pad clicks that evaluate, scientific mode revealing its keys, the physical keyboard driving the same state, and the angle switch changing trig results.

## Layout

```
Cargo.toml              one dependency: gpui-kit = "0.7.1" (dev-dependencies adds test-support)
src/main.rs          ~30 lines   app entry, title-bar window options, initial and minimum size
src/engine.rs       ~900 lines   key semantics, calculator state machine, recursive-descent parser, evaluation, formatting
src/view.rs         ~570 lines   GPUI UI: display, mode bar, keypad, key mapping, headless UI tests
scripts/build-app.sh            macOS .app bundle and zip
```

## Known limitations

- No code signing or notarization. On first launch use right-click → **Open**, or run `xattr -dr com.apple.quarantine LiteCalculator.app`.
- `f64` decimal results only: no arbitrary precision, complex numbers, fractions or unit conversion.
- No memory keys (`M+` / `MR`), and the expression can't be edited with a cursor — only whole-token backspace or clear.
- UI strings are Chinese; i18n is not wired up.
- Linux and Windows are untested.

## Contributing

Before committing:

```bash
cargo fmt --check
cargo test
```

## License

[MIT](LICENSE). Upstream `gpui-kit` 0.7.1 (which bundles `gpui`, `gpui-base`, `gpui-component` and `gpui-kit-assets`) keeps its own licenses.
