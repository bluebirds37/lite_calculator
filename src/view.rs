//! 计算器界面：标题栏、显示区、模式切换与键盘。

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{
    h_flex, v_flex, ActiveTheme, Disableable, Selectable, Sizable as _, Size, Theme, ThemeMode,
    TitleBar,
};
use gpui_kit::*;

use crate::convert::{
    Base, Conversion, ConvertKey, ConvertKind, Converter, BASES, CATEGORIES, KINDS,
};
use crate::engine::{Calculator, Key, Mode};

/// 按键的视觉层级。
#[derive(Clone, Copy, PartialEq)]
enum Tone {
    Number,
    Operator,
    Function,
    Equals,
    Clear,
}

/// 一个按键方块：元素 id、标签、字号、视觉层级与语义。
type Spec = (&'static str, &'static str, f32, Tone, Key);

const STANDARD_PAD: [&[Spec]; 5] = [
    &[
        ("ac", "AC", 18., Tone::Clear, Key::Clear),
        ("back", "⌫", 20., Tone::Operator, Key::Backspace),
        ("percent", "%", 21., Tone::Operator, Key::Percent),
        ("divide", "÷", 24., Tone::Operator, Key::Div),
    ],
    &[
        ("d7", "7", 23., Tone::Number, Key::Digit('7')),
        ("d8", "8", 23., Tone::Number, Key::Digit('8')),
        ("d9", "9", 23., Tone::Number, Key::Digit('9')),
        ("multiply", "×", 24., Tone::Operator, Key::Mul),
    ],
    &[
        ("d4", "4", 23., Tone::Number, Key::Digit('4')),
        ("d5", "5", 23., Tone::Number, Key::Digit('5')),
        ("d6", "6", 23., Tone::Number, Key::Digit('6')),
        ("minus", "−", 24., Tone::Operator, Key::Sub),
    ],
    &[
        ("d1", "1", 23., Tone::Number, Key::Digit('1')),
        ("d2", "2", 23., Tone::Number, Key::Digit('2')),
        ("d3", "3", 23., Tone::Number, Key::Digit('3')),
        ("plus", "+", 24., Tone::Operator, Key::Add),
    ],
    &[
        ("negate", "±", 21., Tone::Number, Key::Negate),
        ("d0", "0", 23., Tone::Number, Key::Digit('0')),
        ("dot", ".", 23., Tone::Number, Key::Dot),
        ("equals", "=", 26., Tone::Equals, Key::Equals),
    ],
];

const SCIENTIFIC_PAD: [&[Spec]; 4] = [
    &[
        ("sin", "sin", 15., Tone::Function, Key::Sin),
        ("cos", "cos", 15., Tone::Function, Key::Cos),
        ("tan", "tan", 15., Tone::Function, Key::Tan),
        ("asin", "sin⁻¹", 13., Tone::Function, Key::ASin),
        ("acos", "cos⁻¹", 13., Tone::Function, Key::ACos),
        ("atan", "tan⁻¹", 13., Tone::Function, Key::ATan),
    ],
    &[
        ("ln", "ln", 15., Tone::Function, Key::Ln),
        ("log", "log", 15., Tone::Function, Key::Log),
        ("log2", "log₂", 14., Tone::Function, Key::Log2),
        ("exp", "eˣ", 15., Tone::Function, Key::Exp),
        ("sqrt", "√", 18., Tone::Function, Key::Sqrt),
        ("cbrt", "∛", 18., Tone::Function, Key::Cbrt),
    ],
    &[
        ("square", "x²", 15., Tone::Function, Key::Square),
        ("cube", "x³", 15., Tone::Function, Key::Cube),
        ("power", "xʸ", 15., Tone::Function, Key::Pow),
        ("reciprocal", "1/x", 14., Tone::Function, Key::Reciprocal),
        ("factorial", "n!", 15., Tone::Function, Key::Factorial),
        ("mod", "mod", 14., Tone::Function, Key::Mod),
    ],
    &[
        ("lparen", "(", 18., Tone::Function, Key::LParen),
        ("rparen", ")", 18., Tone::Function, Key::RParen),
        ("abs", "|x|", 14., Tone::Function, Key::Abs),
        ("pi", "π", 17., Tone::Function, Key::Pi),
        ("euler", "e", 15., Tone::Function, Key::E),
        ("answer", "Ans", 14., Tone::Function, Key::Answer),
    ],
];

/// 转换面板的键盘：7 列 × 3 行，字母键只在十六进制下可用。
const CONVERT_PAD: [[(&str, &str, ConvertKey); 7]; 3] = [
    [
        ("c-1", "1", ConvertKey::Digit('1')),
        ("c-2", "2", ConvertKey::Digit('2')),
        ("c-3", "3", ConvertKey::Digit('3')),
        ("c-4", "4", ConvertKey::Digit('4')),
        ("c-5", "5", ConvertKey::Digit('5')),
        ("c-6", "6", ConvertKey::Digit('6')),
        ("c-7", "7", ConvertKey::Digit('7')),
    ],
    [
        ("c-8", "8", ConvertKey::Digit('8')),
        ("c-9", "9", ConvertKey::Digit('9')),
        ("c-0", "0", ConvertKey::Digit('0')),
        ("c-a", "A", ConvertKey::Digit('a')),
        ("c-b", "B", ConvertKey::Digit('b')),
        ("c-c", "C", ConvertKey::Digit('c')),
        ("c-d", "D", ConvertKey::Digit('d')),
    ],
    [
        ("c-e", "E", ConvertKey::Digit('e')),
        ("c-f", "F", ConvertKey::Digit('f')),
        ("c-dot", ".", ConvertKey::Dot),
        ("c-colon", ":", ConvertKey::Colon),
        ("c-sign", "−", ConvertKey::Sign),
        ("c-back", "⌫", ConvertKey::Back),
        ("c-clear", "AC", ConvertKey::Clear),
    ],
];

/// 渲染期使用的调色板，避免在构造元素时继续借用 `App`。
struct Palette {
    canvas: Hsla,
    surface: Hsla,
    border: Hsla,
    foreground: Hsla,
    muted: Hsla,
    danger: Hsla,
}

pub struct CalculatorView {
    calc: Calculator,
    converter: Converter,
    convert_open: bool,
    focus: FocusHandle,
    focused: bool,
}

impl CalculatorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            calc: Calculator::new(),
            converter: Converter::new(),
            convert_open: false,
            focus: cx.focus_handle(),
            focused: false,
        }
    }

    /// 把一段状态变更包成点击回调。
    fn on(
        view: Entity<Self>,
        action: impl Fn(&mut Self, &mut Context<Self>) + Clone + 'static,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        move |_, _, cx| {
            view.update(cx, |this, cx| {
                action(this, cx);
                cx.notify();
            });
        }
    }

    fn pressed(
        view: Entity<Self>,
        key: Key,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        Self::on(view, move |this, _| this.calc.press(key))
    }

    fn pad_tile(
        id: &'static str,
        label: &'static str,
        font: f32,
        tone: Tone,
        height: f32,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Button {
        let button = Button::new(id)
            .child(
                div()
                    .text_size(px(font))
                    .font_weight(FontWeight::BOLD)
                    .line_height(relative(1.))
                    .child(label),
            )
            .flex_1()
            .h(px(height))
            .justify_center()
            .on_click(on_click)
            .rounded(px(16.));

        match tone {
            Tone::Number => button.secondary(),
            Tone::Operator => button.outline(),
            Tone::Equals => button.primary(),
            Tone::Clear => button.danger(),
            Tone::Function => button.ghost().with_size(Size::Small).rounded(px(11.)),
        }
    }

    fn tile(
        spec: &Spec,
        height: f32,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Button {
        let (id, label, font, tone, _) = *spec;
        Self::pad_tile(id, label, font, tone, height, on_click)
    }

    fn row(specs: &[Spec], view: &Entity<Self>, height: f32) -> Div {
        h_flex().w_full().gap_2().children(
            specs
                .iter()
                .map(|spec| Self::tile(spec, height, Self::pressed(view.clone(), spec.4))),
        )
    }

    fn render_display(&self, palette: &Palette) -> Div {
        let expr = self.calc.expr();
        let shown = if expr.is_empty() {
            self.calc.result().unwrap_or("0").to_string()
        } else {
            expr.to_string()
        };
        let font = match shown.chars().count() {
            0..=1 => 48.,
            2..=7 => 42.,
            8..=12 => 34.,
            13..=19 => 26.,
            _ => 20.,
        };

        let hint = if let Some(error) = self.calc.error() {
            div()
                .text_size(px(13.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.danger)
                .child(error)
                .into_any_element()
        } else if let Some(preview) = self.calc.preview() {
            h_flex()
                .gap_1p5()
                .text_color(palette.muted)
                .child(div().text_size(px(16.)).child("="))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.foreground)
                        .child(preview),
                )
                .into_any_element()
        } else {
            div()
                .text_size(px(12.))
                .text_color(palette.muted)
                .child("输入表达式")
                .into_any_element()
        };

        v_flex()
            .w_full()
            .flex_1()
            .min_h_0()
            .gap_1()
            .px_5()
            .py_4()
            .items_end()
            .justify_end()
            .rounded(px(22.))
            .bg(palette.surface)
            .border_1()
            .border_color(palette.border)
            .child(
                h_flex()
                    .w_full()
                    .justify_between()
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.muted)
                    .child(self.calc.angle().label())
                    .child(self.history_line()),
            )
            .child(
                div()
                    .w_full()
                    .text_right()
                    .overflow_hidden()
                    .text_ellipsis_start()
                    .text_size(px(font))
                    .font_weight(FontWeight::BOLD)
                    .text_color(palette.foreground)
                    .child(shown),
            )
            .child(hint)
    }

    fn history_line(&self) -> SharedString {
        match self.calc.history().last() {
            Some((expr, result)) => format!("{expr} = {result}").into(),
            None => "".into(),
        }
    }

    fn render_mode_bar(&self, view: &Entity<Self>, palette: &Palette) -> Div {
        let scientific = self.calc.mode() == Mode::Scientific;
        let segment = |id: &'static str, label: &'static str, mode: Mode, active: bool| {
            Button::new(id)
                .label(label)
                .ghost()
                .with_size(Size::Small)
                .selected(active)
                .rounded(px(11.))
                .on_click(Self::pressed(view.clone(), Key::SetMode(mode)))
        };

        h_flex()
            .w_full()
            .flex_none()
            .items_center()
            .gap_2()
            .child(
                h_flex()
                    .gap_1()
                    .p_1()
                    .rounded(px(14.))
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .child(segment(
                        "mode-standard",
                        "标准",
                        Mode::Standard,
                        !scientific,
                    ))
                    .child(segment(
                        "mode-scientific",
                        "科学",
                        Mode::Scientific,
                        scientific,
                    )),
            )
            .child(div().flex_1())
            .child(
                Button::new("convert")
                    .label("换算")
                    .ghost()
                    .with_size(Size::Small)
                    .selected(self.convert_open)
                    .tooltip("打开进制 / 角度 / 单位换算面板")
                    .on_click(Self::on(view.clone(), |this, _| {
                        this.convert_open = !this.convert_open;
                    })),
            )
            .child(
                Button::new("angle")
                    .label(self.calc.angle().label())
                    .outline()
                    .with_size(Size::Small)
                    .tooltip("在角度制与弧度制之间切换")
                    .on_click(Self::pressed(view.clone(), Key::ToggleAngle)),
            )
            .child(
                Button::new("clear-history")
                    .label("清空历史")
                    .ghost()
                    .with_size(Size::Small)
                    .disabled(self.calc.history().is_empty())
                    .on_click(Self::pressed(view.clone(), Key::ClearHistory)),
            )
    }

    fn render_keypad(&self, view: &Entity<Self>, palette: &Palette) -> Div {
        let scientific = self.calc.mode() == Mode::Scientific;
        let mut keypad = v_flex().w_full().flex_none().gap_2();

        if scientific {
            keypad = keypad.child(
                v_flex()
                    .w_full()
                    .gap_2()
                    .p_2()
                    .rounded(px(20.))
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .children(
                        SCIENTIFIC_PAD
                            .iter()
                            .map(|specs| Self::row(specs, view, 36.)),
                    ),
            );
        }

        keypad.children(
            STANDARD_PAD
                .iter()
                .map(|specs| Self::row(specs, view, if scientific { 46. } else { 58. })),
        )
    }

    fn convert_chip(
        id: String,
        label: &'static str,
        active: bool,
        action: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Button {
        Button::new(id)
            .label(label)
            .ghost()
            .with_size(Size::Small)
            .selected(active)
            .rounded(px(10.))
            .on_click(action)
    }

    /// 换算按键在当前类别下是否可用：字母只服务十六进制，冒号只服务度分秒。
    fn convert_key_enabled(key: ConvertKey, kind: ConvertKind, base: Base) -> bool {
        match key {
            ConvertKey::Digit(ch) if ch.is_ascii_alphabetic() => {
                kind == ConvertKind::Base && base == Base::Hex
            }
            ConvertKey::Colon => kind == ConvertKind::Angle,
            ConvertKey::Sign => kind != ConvertKind::Base,
            _ => true,
        }
    }

    fn convert_hint(&self) -> String {
        match self.converter.kind {
            ConvertKind::Base => format!("按 {} 输入整数", self.converter.base.label()),
            ConvertKind::Angle => "十进制度，或 度:分:秒".to_string(),
            ConvertKind::Unit => {
                format!("{} → {}", self.converter.from_unit, self.converter.to_unit)
            }
        }
    }

    /// 覆盖在键盘之上的换算面板。
    fn render_convert(&self, view: &Entity<Self>, palette: &Palette) -> Div {
        let mut scrim = palette.canvas;
        scrim.a = 0.82;

        div()
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .items_center()
            .pt(px(44.))
            .bg(scrim)
            .child(self.convert_card(view, palette))
    }

    fn convert_card(&self, view: &Entity<Self>, palette: &Palette) -> Div {
        let kind = self.converter.kind;
        let base = self.converter.base;
        let buffer = self.converter.buffer();

        let header = h_flex()
            .w_full()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.foreground)
                    .child("换算"),
            )
            .child(div().flex_1())
            .child(Self::convert_chip(
                "c-close".to_string(),
                "✕",
                false,
                Self::on(view.clone(), |this, _| this.convert_open = false),
            ));

        let kinds = h_flex()
            .w_full()
            .gap_1()
            .p_1()
            .rounded(px(12.))
            .bg(palette.surface)
            .border_1()
            .border_color(palette.border)
            .children(KINDS.iter().map(|entry| {
                Self::convert_chip(
                    format!("c-kind-{}", entry.id()),
                    entry.label(),
                    kind == *entry,
                    Self::on(view.clone(), move |this, _| this.converter.set_kind(*entry)),
                )
            }));

        let base_row = (kind == ConvertKind::Base).then(|| {
            h_flex()
                .w_full()
                .gap_1()
                .children(BASES.iter().map(|entry| {
                    Self::convert_chip(
                        format!("c-base-{}", entry.id()),
                        entry.label(),
                        base == *entry,
                        Self::on(view.clone(), move |this, _| this.converter.set_base(*entry)),
                    )
                }))
                .into_any_element()
        });

        let unit_rows = (kind == ConvertKind::Unit).then(|| {
            let category = self.converter.category;
            let units = category.units();
            let from = self.converter.from_unit;
            let to = self.converter.to_unit;

            let tray = |leading: &'static str, active: &'static str, target: &'static str| {
                h_flex()
                    .w_full()
                    .items_center()
                    .gap_1()
                    .flex_wrap()
                    .child(
                        div()
                            .flex_none()
                            .text_size(px(11.))
                            .text_color(palette.muted)
                            .child(leading),
                    )
                    .children(units.iter().enumerate().map(move |(index, unit)| {
                        let selected = active == unit.key;
                        let key = unit.key;
                        let on_select = move |this: &mut Self, _: &mut Context<Self>| match target {
                            "from" => this.converter.set_from(key),
                            _ => this.converter.set_to(key),
                        };
                        Self::convert_chip(
                            format!("c-{target}-{index}"),
                            unit.label,
                            selected,
                            Self::on(view.clone(), on_select),
                        )
                    }))
            };

            v_flex()
                .w_full()
                .gap_1()
                .child(
                    h_flex()
                        .w_full()
                        .gap_1()
                        .flex_wrap()
                        .children(CATEGORIES.iter().map(|entry| {
                            Self::convert_chip(
                                format!("c-cat-{}", entry.id()),
                                entry.label(),
                                category == *entry,
                                Self::on(view.clone(), move |this, _| {
                                    this.converter.set_category(*entry)
                                }),
                            )
                        })),
                )
                .child(tray("从", from, "from"))
                .child(tray("到", to, "to"))
                .into_any_element()
        });

        let input_line = h_flex()
            .w_full()
            .items_end()
            .justify_between()
            .gap_2()
            .child(
                div()
                    .flex_none()
                    .text_size(px(11.))
                    .text_color(palette.muted)
                    .child(self.convert_hint()),
            )
            .child(
                div()
                    .overflow_hidden()
                    .text_right()
                    .text_size(px(24.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(palette.foreground)
                    .child(if buffer.is_empty() {
                        "0".to_string()
                    } else {
                        buffer.to_string()
                    }),
            );

        let results = match self.converter.conversion() {
            Conversion::Empty => div()
                .w_full()
                .text_right()
                .text_size(px(12.))
                .text_color(palette.muted)
                .child("输入数值")
                .into_any_element(),
            Conversion::Error(text) => div()
                .w_full()
                .text_right()
                .text_size(px(13.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.danger)
                .child(text)
                .into_any_element(),
            Conversion::Rows(rows) => v_flex()
                .w_full()
                .gap_1()
                .children(rows.iter().map(|(label, value)| {
                    let font = match value.chars().count() {
                        0..=18 => 15.,
                        19..=30 => 13.,
                        _ => 11.,
                    };
                    h_flex()
                        .w_full()
                        .items_end()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex_none()
                                .text_size(px(11.))
                                .text_color(palette.muted)
                                .child(label.clone()),
                        )
                        .child(
                            div()
                                .overflow_hidden()
                                .text_right()
                                .text_size(px(font))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(palette.foreground)
                                .child(value.clone()),
                        )
                }))
                .into_any_element(),
        };

        let pad = v_flex()
            .w_full()
            .gap_2()
            .children(CONVERT_PAD.iter().map(|row| {
                h_flex()
                    .w_full()
                    .gap_1p5()
                    .children(row.iter().map(|&(id, label, key)| {
                        let tone = match key {
                            ConvertKey::Clear => Tone::Clear,
                            ConvertKey::Back => Tone::Operator,
                            _ => Tone::Number,
                        };
                        Self::pad_tile(
                            id,
                            label,
                            15.,
                            tone,
                            38.,
                            Self::on(view.clone(), move |this, _| this.converter.press(key)),
                        )
                        .disabled(!Self::convert_key_enabled(key, kind, base))
                    }))
            }));

        v_flex()
            .w(px(352.))
            .gap_2()
            .p_3()
            .rounded(px(22.))
            .bg(palette.canvas)
            .border_1()
            .border_color(palette.border)
            .child(header)
            .child(kinds)
            .children(base_row)
            .children(unit_rows)
            .child(input_line)
            .child(results)
            .child(pad)
    }

    fn keystroke_to_key(stroke: &Keystroke, scientific: bool) -> Option<Key> {
        if stroke.modifiers.control || stroke.modifiers.alt || stroke.modifiers.platform {
            return None;
        }
        match stroke.key.as_str() {
            "return" => return Some(Key::Equals),
            "backspace" => return Some(Key::Backspace),
            "escape" | "delete" => return Some(Key::Clear),
            _ => {}
        }
        let text = stroke.key_char.as_deref()?;
        let mut chars = text.chars();
        if let (Some(first), None) = (chars.next(), chars.next()) {
            if first.is_ascii_digit() {
                return Some(Key::Digit(first));
            }
        }
        Some(match text {
            "." => Key::Dot,
            "+" => Key::Add,
            "-" => Key::Sub,
            "*" => Key::Mul,
            "/" => Key::Div,
            "^" => Key::Pow,
            "%" => Key::Percent,
            "!" => Key::Factorial,
            "(" => Key::LParen,
            ")" => Key::RParen,
            "=" => Key::Equals,
            "π" => Key::Pi,
            "s" if scientific => Key::Sin,
            "c" if scientific => Key::Cos,
            "t" if scientific => Key::Tan,
            "l" if scientific => Key::Ln,
            "g" if scientific => Key::Log,
            "r" if scientific => Key::Sqrt,
            "q" if scientific => Key::Square,
            "p" if scientific => Key::Pi,
            _ => return None,
        })
    }

    /// 换算面板打开时，物理键盘只驱动换算输入。
    fn keystroke_to_convert_key(stroke: &Keystroke, converter: &Converter) -> Option<ConvertKey> {
        if stroke.modifiers.control || stroke.modifiers.alt || stroke.modifiers.platform {
            return None;
        }
        match stroke.key.as_str() {
            "backspace" => return Some(ConvertKey::Back),
            "delete" => return Some(ConvertKey::Clear),
            _ => {}
        }
        let mut chars = stroke.key_char.as_deref()?.chars();
        let (Some(first), None) = (chars.next(), chars.next()) else {
            return None;
        };
        if first.is_ascii_digit() {
            return Some(ConvertKey::Digit(first));
        }
        let lower = first.to_ascii_lowercase();
        Some(match lower {
            '.' => ConvertKey::Dot,
            ':' => ConvertKey::Colon,
            '-' => ConvertKey::Sign,
            'a'..='f' if converter.kind == ConvertKind::Base && converter.base == Base::Hex => {
                ConvertKey::Digit(lower)
            }
            _ => return None,
        })
    }
}

impl Render for CalculatorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let palette = Palette {
            canvas: theme.background,
            surface: theme.muted,
            border: theme.border,
            foreground: theme.foreground,
            muted: theme.muted_foreground,
            danger: theme.danger,
        };
        let dark = theme.mode.is_dark();
        let view = cx.entity().clone();

        if !self.focused {
            self.focused = true;
            window.focus(&self.focus, cx);
        }

        v_flex()
            .id("calculator")
            .size_full()
            .relative()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if this.convert_open {
                    if event.keystroke.key == "escape" {
                        this.convert_open = false;
                    } else if let Some(key) =
                        Self::keystroke_to_convert_key(&event.keystroke, &this.converter)
                    {
                        this.converter.press(key);
                    }
                    cx.notify();
                    return;
                }
                let scientific = this.calc.mode() == Mode::Scientific;
                if let Some(key) = Self::keystroke_to_key(&event.keystroke, scientific) {
                    this.calc.press(key);
                    cx.notify();
                }
            }))
            .bg(palette.canvas)
            .child(
                TitleBar::new().h(px(34.)).child(
                    h_flex()
                        .w_full()
                        .px_3()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child("计算器"),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(palette.muted)
                                        .child(self.calc.mode().label()),
                                ),
                        )
                        .child(
                            Button::new("theme")
                                .label(if dark { "☀" } else { "☾" })
                                .ghost()
                                .with_size(Size::Small)
                                .tooltip("切换深色 / 浅色主题")
                                .on_click(|_, window, cx| {
                                    let mode = if cx.theme().mode.is_dark() {
                                        ThemeMode::Light
                                    } else {
                                        ThemeMode::Dark
                                    };
                                    Theme::change(mode, Some(window), cx);
                                }),
                        ),
                ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .px_4()
                    .pt_2()
                    .pb_4()
                    .gap_3()
                    .child(self.render_display(&palette))
                    .child(self.render_mode_bar(&view, &palette))
                    .child(self.render_keypad(&view, &palette)),
            )
            .children(
                self.convert_open
                    .then(|| self.render_convert(&view, &palette).into_any_element()),
            )
    }
}

#[cfg(test)]
mod tests {
    // 显式导入：`use gpui_kit::*` 会把 GPUI 的 `test` 属性宏带进作用域，
    // 覆盖 Rust 的 `#[test]` 并让 `#[gpui_kit::test]` 的展开无限递归。
    use crate::engine::Mode;
    use crate::view::CalculatorView;
    use gpui_kit::test::TestWindowExt as _;
    use gpui_kit::{
        px, size, AnyWindowHandle, AppContext as _, Bounds, Entity, Point, TestAppContext,
        WindowBounds, WindowOptions,
    };

    fn open(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<CalculatorView>) {
        cx.update(|cx| {
            gpui_kit::init(cx);
            let (window, view) = gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::default(),
                        size: size(px(400.), px(720.)),
                    })),
                    ..Default::default()
                },
                cx,
                |_, cx| cx.new(CalculatorView::new),
            )
            .expect("打开测试窗口失败");
            (window.into(), view)
        })
    }

    fn snapshot(
        view: &Entity<CalculatorView>,
        cx: &mut TestAppContext,
    ) -> (String, Option<String>, Mode) {
        cx.update(|cx| {
            let view = view.read(cx);
            (
                view.calc.expr().to_string(),
                view.calc.result().map(str::to_string),
                view.calc.mode(),
            )
        })
    }

    #[gpui_kit::test]
    fn clicking_the_pad_evaluates_an_expression(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            for id in ["d1", "d2", "plus", "d6", "equals"] {
                window.click(id, cx);
            }
        })
        .unwrap();

        let (expr, result, mode) = snapshot(&view, cx);
        assert_eq!(mode, Mode::Standard);
        assert_eq!(result.as_deref(), Some("18"));
        assert_eq!(expr, "18");
    }

    #[gpui_kit::test]
    fn scientific_mode_reveals_the_extra_keys(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click("mode-scientific", cx);
            window.render_frame(cx);
            for id in ["lparen", "d3", "d0", "rparen", "square", "equals"] {
                window.click(id, cx);
            }
        })
        .unwrap();

        let (expr, result, mode) = snapshot(&view, cx);
        assert_eq!(mode, Mode::Scientific);
        assert_eq!(expr, "900");
        assert_eq!(result.as_deref(), Some("900"));
    }

    #[gpui_kit::test]
    fn the_physical_keyboard_drives_the_same_state(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            for key in ["7", "*", "8", "return"] {
                window.press(key, cx);
            }
            window.press("backspace", cx);
        })
        .unwrap();

        let (expr, result, _) = snapshot(&view, cx);
        assert_eq!(result.as_deref(), Some("56"));
        assert_eq!(expr, "5");
    }

    #[gpui_kit::test]
    fn the_angle_switch_changes_trig_results(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click("mode-scientific", cx);
            window.render_frame(cx);
            window.click("angle", cx);
            for id in ["sin", "d1", "equals"] {
                window.click(id, cx);
            }
        })
        .unwrap();

        let (_, result, _) = snapshot(&view, cx);
        assert_eq!(result.as_deref(), Some("0.841470984808"));
    }

    /// 打开换算面板所需的点击次数很少，这里统一读回面板状态与结果。
    fn convert_state(
        view: &Entity<CalculatorView>,
        cx: &mut TestAppContext,
        label: &str,
    ) -> (bool, Option<String>) {
        cx.update(|cx| {
            let view = view.read(cx);
            (
                view.convert_open,
                view.converter.conversion().get(label).map(str::to_string),
            )
        })
    }

    #[gpui_kit::test]
    fn the_convert_overlay_turns_hex_into_every_radix(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click("convert", cx);
            window.render_frame(cx);
            window.click("c-base-hex", cx);
            window.render_frame(cx);
            for id in ["c-7", "c-f"] {
                window.click(id, cx);
            }
        })
        .unwrap();

        let expr = cx.update(|cx| view.read(cx).calc.expr().to_string());
        assert_eq!(expr, "", "换算面板的输入不应写回表达式");
        let (open, decimal) = convert_state(&view, cx, "十进制");
        assert!(open);
        assert_eq!(decimal.as_deref(), Some("127"));
        assert_eq!(
            convert_state(&view, cx, "二进制").1.as_deref(),
            Some("1111111")
        );
        assert_eq!(convert_state(&view, cx, "位宽").1.as_deref(), Some("7 bit"));
    }

    #[gpui_kit::test]
    fn the_convert_overlay_changes_units(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click("convert", cx);
            window.click("c-kind-unit", cx);
            window.render_frame(cx);
            for id in ["c-1", "c-0", "c-0"] {
                window.click(id, cx);
            }
        })
        .unwrap();

        assert_eq!(
            convert_state(&view, cx, "换算结果").1.as_deref(),
            Some("62.1371192237 mi"),
            "默认长度换算 km → mi"
        );

        cx.update_window(window, |_, window, cx| {
            window.click("c-from-7", cx);
            window.click("c-to-3", cx);
        })
        .unwrap();
        assert_eq!(
            convert_state(&view, cx, "换算结果").1.as_deref(),
            Some("160.9344 km"),
            "换成 mi → km 后结果同步"
        );
    }

    #[gpui_kit::test]
    fn the_convert_overlay_takes_physical_keys_and_closes_on_escape(cx: &mut TestAppContext) {
        let (window, view) = open(cx);
        cx.update_window(window, |_, window, cx| {
            window.render_frame(cx);
            window.click("convert", cx);
            window.click("c-kind-angle", cx);
            window.render_frame(cx);
            for key in ["4", "0", ":", "2", "6", ":", "4", "6"] {
                window.press(key, cx);
            }
        })
        .unwrap();

        assert_eq!(
            convert_state(&view, cx, "十进制度").1.as_deref(),
            Some("40.4461111111"),
            "度分秒输入应转成十进制度"
        );
        assert_eq!(
            convert_state(&view, cx, "度分秒").1.as_deref(),
            Some("40° 26′ 46″")
        );
        assert_eq!(
            cx.update(|cx| view.read(cx).calc.expr().to_string()),
            "",
            "面板内的按键不落入表达式"
        );

        cx.update_window(window, |_, window, cx| {
            window.press("escape", cx);
        })
        .unwrap();
        assert!(!convert_state(&view, cx, "十进制度").0, "Esc 关闭面板");
    }
}
