//! dd_siteforge GUI. Phase 2 shell on the phase 1 load, save, and theme path.

mod document;
mod keys;
mod new_site;
mod paint;
mod present;
mod preview;
mod settings;
mod shell;
mod theme_load;

#[cfg(test)]
mod tests;

use std::path::PathBuf;

use gpui::{App, AppContext, Bounds, Focusable, WindowBounds, WindowOptions, px, size};
use gpui_component::Root;
use shell::{Shell, ShellOptions};

fn main() {
    let path = std::env::args().nth(1).map(PathBuf::from);
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        let path = path.clone();
        cx.spawn(async move |cx| {
            cx.update(|cx| {
                let bounds = Bounds::centered(None, size(px(1280.0), px(800.0)), cx);
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |window, cx| {
                        let view = cx.new(|cx| {
                            let shell = Shell::open(ShellOptions::production(path), cx);
                            window.focus(&shell.focus_handle(cx), cx);
                            shell
                        });
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("open window");
            });
        })
        .detach();
    });
}
