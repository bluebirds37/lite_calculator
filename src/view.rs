//! 计算器界面：标题栏、显示区、模式切换与键盘。

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{
    h_flex, v_flex, ActiveTheme, Disableable, Selectable, Sizable as _, Size, Theme, ThemeMode,
    TitleBar,
};
use gpui_kit::*;

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
    focus: FocusHandle,
    focused: bool,
}

impl CalculatorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            calc: Calculator::new(),
            focus: cx.focus_handle(),
            focused: false,
        }
    }

    fn pressed(
        view: Entity<Self>,
        key: Key,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        move |_, _, cx| {
            view.update(cx, |this, cx| {
                this.calc.press(key);
                cx.notify();
            });
        }
    }

    fn tile(
        spec: &Spec,
        height: f32,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Button {
        let (id, label, font, tone, _) = *spec;
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
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
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
}
