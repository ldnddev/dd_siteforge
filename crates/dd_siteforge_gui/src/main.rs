//! dd_siteforge GUI. Phase 1: themed window, site.json load/save, browser preview.

mod document;
mod preview;
mod settings;
mod theme_load;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use document::Document;
use gpui::{
    App, AppContext, Bounds, Context, FocusHandle, Focusable, IntoElement, KeyBinding,
    ParentElement, Render, SharedString, Styled, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
use gpui_component::{Root, Theme, ThemeColor, ThemeMode, button::Button};
use ldnddev_theme::{GuiTheme, PrimaryButton, Rgb, ThemeMode as SiteMode};
use settings::GuiSettings;
use theme_load::{LiveTheme, fc_match_monospace};

use gpui::actions;
actions!(dd_siteforge_gui, [Checkpoint, OpenPreview]);

fn main() {
    let path = std::env::args().nth(1).map(PathBuf::from);
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        cx.bind_keys([KeyBinding::new("ctrl-s", Checkpoint, None)]);
        let path = path.clone();
        cx.spawn(async move |cx| {
            cx.update(|cx| {
                let bounds = Bounds::centered(None, size(px(960.0), px(640.0)), cx);
                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |window, cx| {
                        let view = cx.new(|cx| Shell::new(path, window, cx));
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("open window");
            });
        })
        .detach();
    });
}

struct Shell {
    focus: FocusHandle,
    paths: ldnddev_theme::OmarchyPaths,
    theme: LiveTheme,
    settings: GuiSettings,
    document: Option<Document>,
    preview_note: SharedString,
    last_theme_check: Instant,
}

impl Shell {
    fn new(path: Option<PathBuf>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = settings::load();
        let paths = ldnddev_theme::OmarchyPaths::system();
        let theme = LiveTheme::load(&paths, settings.reduced_motion);
        apply_component_theme(&theme, cx);
        let (document, preview_note) = match path {
            Some(path) => match Document::load(path.clone()) {
                Ok(doc) => {
                    record_recent(&doc);
                    (Some(doc), String::new())
                }
                Err(err) => (None, format!("Couldn't load {}: {err}", path.display())),
            },
            None => (None, String::new()),
        };
        let shell = Self {
            focus: cx.focus_handle(),
            paths,
            theme,
            settings,
            document,
            preview_note: SharedString::from(preview_note),
            last_theme_check: Instant::now(),
        };
        window.focus(&shell.focus, cx);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                let alive = this
                    .update(cx, |shell, cx| {
                        shell.tick(cx);
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();
        shell
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        let now = Instant::now();
        let mut theme_changed = false;
        if now.duration_since(self.last_theme_check) >= Duration::from_secs(2) {
            self.last_theme_check = now;
            theme_changed = self.theme.refresh(&self.paths);
            if theme_changed {
                apply_component_theme(&self.theme, cx);
            }
        }
        let saved = self
            .document
            .as_mut()
            .is_some_and(|doc| doc.autosave(Instant::now()));
        if theme_changed || saved {
            cx.notify();
        }
    }

    fn checkpoint(&mut self, cx: &mut Context<Self>) {
        if let Some(doc) = self.document.as_mut() {
            doc.checkpoint();
            cx.notify();
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        if let Some(doc) = self.document.as_mut() {
            if let Err(err) = doc.reload() {
                doc.notice = Some(format!("Failed to reload: {err}"));
            }
            cx.notify();
        }
    }

    fn keep_mine(&mut self, cx: &mut Context<Self>) {
        if let Some(doc) = self.document.as_mut() {
            doc.keep_mine();
            cx.notify();
        }
    }

    fn open_preview(&mut self, cx: &mut Context<Self>) {
        let Some(doc) = self.document.as_ref() else {
            self.preview_note = "Open a site.json to preview.".into();
            cx.notify();
            return;
        };
        let Some(dir) = doc.site.export_dir.as_deref() else {
            self.preview_note = "Preview opens in the browser once an export folder is set.".into();
            cx.notify();
            return;
        };
        let root = doc
            .path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(dir);
        match dd_siteforge_core::serve::StaticServer::start(root) {
            Ok(server) => {
                let url = server.url_for("index", doc.site.pretty_urls);
                // Keep the server alive for this process. Leak is intentional:
                // dropping it would close the port the browser is reading.
                let url_for_note = url.clone();
                std::mem::forget(server);
                if let Err(err) = preview::open_in_browser(&url) {
                    self.preview_note = format!("Couldn't open the browser: {err}").into();
                } else {
                    self.preview_note = format!("Opened {url_for_note} in the browser").into();
                }
            }
            Err(err) => {
                self.preview_note = format!("Couldn't start preview: {err}").into();
            }
        }
        cx.notify();
    }
}

impl Focusable for Shell {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Shell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = &self.theme.gui;
        let base = fill(theme, "surface.base");
        let panel = fill(theme, "surface.panel");
        let text = fill(theme, "text.primary");
        let secondary = fill(theme, "text.secondary");
        let accent = fill(theme, "text.accent");
        let family = SharedString::from(self.theme.fonts.ui.clone());
        let size = px(self.theme.metrics.type_scale.sm);
        let title = self
            .document
            .as_ref()
            .map(|doc| doc.site.name.clone())
            .unwrap_or_else(|| "dd_siteforge".to_string());
        let path = self
            .document
            .as_ref()
            .map(|doc| doc.path.display().to_string())
            .unwrap_or_else(|| "No site open".to_string());
        let save = save_label(self.document.as_ref());
        let conflict = self.document.as_ref().is_some_and(|doc| doc.conflict);
        let notice = self
            .document
            .as_ref()
            .and_then(|doc| doc.notice.clone())
            .unwrap_or_default();
        let mode = self.theme.mode_label();
        let button_label = if theme.primary_button == PrimaryButton::Outlined {
            "outlined primary"
        } else {
            "filled primary"
        };
        let status_rgb = icon_on_base(theme, "status.success");

        div()
            .id("shell")
            .track_focus(&self.focus)
            .on_action(cx.listener(|shell, _: &Checkpoint, _window, cx| {
                shell.checkpoint(cx);
            }))
            .on_action(cx.listener(|shell, _: &OpenPreview, _window, cx| {
                shell.open_preview(cx);
            }))
            .size_full()
            .flex()
            .flex_col()
            .bg(base)
            .text_color(text)
            .font_family(family)
            .text_size(size)
            .p(px(16.0))
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(self.theme.metrics.type_scale.lg))
                            .child(title),
                    )
                    .child(
                        div()
                            .text_color(secondary)
                            .child(format!("{} · {}", self.theme.label, mode)),
                    ),
            )
            .child(div().text_color(secondary).child(path))
            .child(div().child(save))
            .when(!notice.is_empty(), |row| row.child(div().child(notice)))
            .when(conflict, |row| {
                row.child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .bg(panel)
                        .p(px(8.0))
                        .child("Changed on disk.")
                        .child(
                            Button::new("reload")
                                .label("Reload")
                                .on_click(cx.listener(|shell, _, _, cx| shell.reload(cx))),
                        )
                        .child(
                            Button::new("keep")
                                .label("Keep mine")
                                .on_click(cx.listener(|shell, _, _, cx| shell.keep_mine(cx))),
                        ),
                )
            })
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .items_center()
                    .child(div().size(px(10.0)).rounded(px(9999.0)).bg(status_rgb))
                    .child(format!(
                        "Theme {} · font {} · {} · primary {button_label}",
                        mode,
                        self.theme.fonts.ui,
                        match self.settings.density {
                            settings::Density::Comfortable => "comfortable",
                            settings::Density::Compact => "compact",
                        }
                    ))
                    .child(
                        Button::new("preview")
                            .label("Preview")
                            .on_click(cx.listener(|shell, _, _, cx| shell.open_preview(cx))),
                    ),
            )
            .when(!self.preview_note.is_empty(), |row| {
                row.child(div().text_color(accent).child(self.preview_note.clone()))
            })
            .when(self.theme.warning.is_some(), |row| {
                row.child(
                    div()
                        .text_color(secondary)
                        .child(self.theme.warning.clone().unwrap_or_default()),
                )
            })
            .child(div().text_color(secondary).child(format!(
                "Ctrl+S writes a .backup checkpoint. Edits autosave after 2 seconds. Preview: {}.",
                preview::PREVIEW_ENGINE
            )))
    }
}

fn save_label(doc: Option<&Document>) -> String {
    let Some(doc) = doc else {
        return "No file".to_string();
    };
    if doc.conflict {
        "Changed on disk".to_string()
    } else if doc.dirty {
        "Unsaved".to_string()
    } else {
        "Saved".to_string()
    }
}

fn fill(theme: &GuiTheme, name: &str) -> gpui::Hsla {
    hsla(theme_load::rgb(theme, name))
}

fn icon_on_base(theme: &GuiTheme, name: &str) -> gpui::Hsla {
    let token = theme
        .on_surface_base
        .get(name)
        .map(String::as_str)
        .unwrap_or(name);
    fill(theme, token)
}

fn hsla(color: Rgb) -> gpui::Hsla {
    gpui::Rgba {
        r: f32::from(color.r) / 255.0,
        g: f32::from(color.g) / 255.0,
        b: f32::from(color.b) / 255.0,
        a: 1.0,
    }
    .into()
}

fn apply_component_theme(theme: &LiveTheme, cx: &mut App) {
    let mode = match theme.gui.mode {
        SiteMode::Light => ThemeMode::Light,
        SiteMode::Dark => ThemeMode::Dark,
    };
    let base = if theme.gui.mode == SiteMode::Light {
        *ThemeColor::light()
    } else {
        *ThemeColor::dark()
    };
    let mut colors = base;
    colors.background = hsla(theme_load::rgb(&theme.gui, "surface.base"));
    colors.foreground = hsla(theme_load::rgb(&theme.gui, "text.primary"));
    colors.accent = hsla(theme_load::rgb(&theme.gui, "accent.fill"));
    colors.accent_foreground = hsla(theme_load::rgb(&theme.gui, "text.on-accent"));
    colors.border = hsla(theme_load::rgb(&theme.gui, "border.input"));
    colors.caret = hsla(theme_load::rgb(&theme.gui, "caret"));
    colors.link = hsla(theme_load::rgb(&theme.gui, "text.accent"));
    let family = SharedString::from(theme.fonts.ui.clone());
    Theme::update(cx, |ui| {
        ui.mode = mode;
    });
    Theme::update(cx, |ui| {
        ui.colors = colors;
        ui.font_family = family.clone();
        ui.mono_font_family = family;
        ui.font_size = px(theme.metrics.type_scale.sm);
        ui.mono_font_size = px(theme.metrics.type_scale.sm);
        ui.radius = px(theme.metrics.radius_sm);
    });
}

fn record_recent(doc: &Document) {
    let mut session = dd_siteforge_core::session::load();
    session.record(&doc.path, 0, 0, "page");
    let _ = dd_siteforge_core::session::save(&session);
}

#[allow(dead_code)]
fn font_now() -> String {
    fc_match_monospace()
}
