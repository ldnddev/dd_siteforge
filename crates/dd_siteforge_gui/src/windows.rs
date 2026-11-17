//! App ids and window options for the editor and the preview companion.
//!
//! On Wayland `app_id` is the xdg app id. On X11 it is the window class.
//! Omarchy float rules target the companion id.

use gpui::{
    App, Bounds, Pixels, SharedString, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};

pub const MAIN_APP_ID: &str = "dev.ldnddev.dd_siteforge";
pub const PREVIEW_APP_ID: &str = "dev.ldnddev.dd_siteforge.preview";

pub const PREVIEW_DEFAULT_WIDTH: f32 = 360.0;
pub const PREVIEW_DEFAULT_HEIGHT: f32 = 248.0;
pub const PREVIEW_MIN_WIDTH: f32 = 320.0;
pub const PREVIEW_MIN_HEIGHT: f32 = 200.0;

pub fn preview_title(site_name: &str) -> String {
    format!("Preview \u{2014} {site_name}")
}

pub fn preview_hint() -> &'static str {
    "The page is open in your browser. It reloads after each build."
}

pub fn main_window_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        app_id: Some(MAIN_APP_ID.to_string()),
        ..Default::default()
    }
}

pub fn preview_window_options(cx: &App, title: &str) -> WindowOptions {
    let bounds = Bounds::centered(
        None,
        size(px(PREVIEW_DEFAULT_WIDTH), px(PREVIEW_DEFAULT_HEIGHT)),
        cx,
    );
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        app_id: Some(PREVIEW_APP_ID.to_string()),
        window_min_size: Some(size(px(PREVIEW_MIN_WIDTH), px(PREVIEW_MIN_HEIGHT))),
        titlebar: Some(TitlebarOptions {
            title: Some(SharedString::from(title.to_string())),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn followup_preview_title_and_hint_match_spec_07() {
        assert_eq!(preview_title("My Site"), "Preview \u{2014} My Site");
        assert_eq!(
            preview_hint(),
            "The page is open in your browser. It reloads after each build."
        );
    }

    #[gpui::test]
    fn followup_main_and_companion_window_options_carry_app_ids(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let main = main_window_options(Bounds::centered(None, size(px(1280.0), px(800.0)), cx));
            assert_eq!(main.app_id.as_deref(), Some(MAIN_APP_ID));
            assert_eq!(MAIN_APP_ID, "dev.ldnddev.dd_siteforge");

            let preview = preview_window_options(cx, "Preview \u{2014} My Site");
            assert_eq!(preview.app_id.as_deref(), Some(PREVIEW_APP_ID));
            assert_eq!(PREVIEW_APP_ID, "dev.ldnddev.dd_siteforge.preview");
            assert_eq!(PREVIEW_DEFAULT_WIDTH, 360.0);
            assert_eq!(PREVIEW_DEFAULT_HEIGHT, 248.0);
            assert_eq!(PREVIEW_MIN_WIDTH, 320.0);
            assert_eq!(PREVIEW_MIN_HEIGHT, 200.0);
            let bounds = preview.window_bounds.expect("bounds").get_bounds();
            assert_eq!(bounds.size, size(px(360.0), px(248.0)));
            assert_eq!(
                preview.window_min_size,
                Some(size(px(PREVIEW_MIN_WIDTH), px(PREVIEW_MIN_HEIGHT)))
            );
            let title = preview
                .titlebar
                .as_ref()
                .and_then(|bar| bar.title.as_ref())
                .map(SharedString::to_string);
            assert_eq!(title.as_deref(), Some("Preview \u{2014} My Site"));
        });
    }
}
