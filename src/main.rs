// gpui_kit::test 宏展开层次较深。
#![recursion_limit = "256"]

mod engine;
mod view;

use gpui_kit::component::TitleBar;
use gpui_kit::*;
use view::CalculatorView;

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);

        let mut options = TitleBar::window_options();
        if let Some(titlebar) = options.titlebar.as_mut() {
            titlebar.title = Some("计算器".into());
        }
        options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(400.), px(720.)),
            cx,
        )));
        options.window_min_size = Some(size(px(380.), px(680.)));

        gpui_kit::open_window(options, cx, |_, cx| cx.new(CalculatorView::new))
            .expect("打开计算器窗口失败");
    });
}
