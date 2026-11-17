//! Companion preview window. The page itself opens in the default browser
//! (Phase 1 spike). This window holds Follow editor and Ctrl+R.

use gpui::{
    App, Context, Entity, FocusHandle, Focusable, IntoElement, KeyDownEvent, ParentElement, Render,
    Styled, Window, div, prelude::*, px,
};
use gpui_component::button::Button;

use crate::shell::{Shell, chord};
use crate::windows;

pub struct PreviewPane {
    shell: Entity<Shell>,
    focus: FocusHandle,
}

impl PreviewPane {
    fn new(shell: Entity<Shell>, focus: FocusHandle) -> Self {
        Self { shell, focus }
    }
}

impl Focusable for PreviewPane {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for PreviewPane {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.shell.read(cx).preview_chrome();
        let title = windows::preview_title(&chrome.site_name);
        window.set_window_title(&title);
        let follow = if chrome.follow {
            "Follow editor on"
        } else {
            "Follow editor"
        };
        div()
            .id("preview-window")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|pane, event: &KeyDownEvent, _, cx| {
                let key = chord(event);
                if key == "ctrl-r" || key == "f5" {
                    pane.shell.update(cx, |shell, cx| {
                        shell.refresh_preview();
                        cx.notify();
                    });
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(8.0))
            .child(div().child(title))
            .child(div().id("preview-url").child(chrome.url))
            .child(div().id("preview-state").child(chrome.status))
            .child(
                Button::new("follow-editor")
                    .label(follow)
                    .on_click(cx.listener(|pane, _: &gpui::ClickEvent, _, cx| {
                        pane.shell.update(cx, |shell, cx| {
                            shell.toggle_follow();
                            cx.notify();
                        });
                    })),
            )
            .child(
                Button::new("preview-refresh")
                    .label("Refresh")
                    .on_click(cx.listener(|pane, _: &gpui::ClickEvent, _, cx| {
                        pane.shell.update(cx, |shell, cx| {
                            shell.refresh_preview();
                            cx.notify();
                        });
                    })),
            )
            .child(div().child(windows::preview_hint()))
    }
}

pub fn open(shell: Entity<Shell>, cx: &mut App) -> bool {
    let title = shell.read(cx).preview_window_title();
    let opened = cx.open_window(
        windows::preview_window_options(cx, &title),
        move |window, cx| {
            let view = cx.new(|cx| {
                let focus = cx.focus_handle();
                window.focus(&focus, cx);
                PreviewPane::new(shell, focus)
            });
            cx.new(|cx| gpui_component::Root::new(view, window, cx))
        },
    );
    opened.is_ok()
}
