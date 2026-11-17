//! Compact density from design spec 06 §8 and the control sizes in spec 01 §3.9.
//!
//! Typography, panel widths, tree indent, block cards, insert rows, and image
//! wells stay the same. Compact drops spacing one notch and shortens controls.

use crate::settings::Density;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub row: f32,
    pub input: f32,
    pub button: f32,
    pub icon_button: f32,
    pub tab: f32,
    pub toolbar: f32,
    pub status_bar: f32,
    pub title_bar: f32,
    /// Sidebar search padding. Comfortable `space.3`, compact `space.2`.
    pub sidebar_search: f32,
    /// Outline header and inspector padding.
    pub pad_x: f32,
    pub pad_y: f32,
    /// Gap between outline blocks. Compact is `space.1-5`.
    pub outline_gap: f32,
    /// Command palette inset. Compact is `space.1`.
    pub palette_inset: f32,
    /// Label sits `space.1` above the control in both densities.
    pub label_gap: f32,
    /// Nested rows keep a 16 px indent in both densities.
    pub tree_indent: f32,
    pub sidebar_width: f32,
    pub inspector_width: f32,
}

pub fn metrics(density: Density) -> Metrics {
    match density {
        Density::Comfortable => Metrics {
            row: 28.0,
            input: 28.0,
            button: 28.0,
            icon_button: 28.0,
            tab: 32.0,
            toolbar: 40.0,
            status_bar: 24.0,
            title_bar: 40.0,
            sidebar_search: 12.0,
            pad_x: 16.0,
            pad_y: 12.0,
            outline_gap: 8.0,
            palette_inset: 12.0,
            label_gap: 4.0,
            tree_indent: 16.0,
            sidebar_width: 280.0,
            inspector_width: 360.0,
        },
        Density::Compact => Metrics {
            row: 24.0,
            input: 24.0,
            button: 24.0,
            icon_button: 24.0,
            tab: 28.0,
            toolbar: 34.0,
            status_bar: 22.0,
            title_bar: 34.0,
            sidebar_search: 8.0,
            pad_x: 12.0,
            pad_y: 8.0,
            outline_gap: 6.0,
            palette_inset: 4.0,
            label_gap: 4.0,
            tree_indent: 16.0,
            sidebar_width: 280.0,
            inspector_width: 360.0,
        },
    }
}

/// Issues drawer. Spec 05: 280 default, drag from 160 to half the window.
pub fn clamp_issues_height(requested: f32, window_height: f32) -> f32 {
    let half = if window_height > 0.0 {
        window_height * 0.5
    } else {
        400.0
    };
    let max = half.max(160.0);
    requested.clamp(160.0, max)
}

pub const ISSUES_DEFAULT: f32 = 280.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_drops_one_notch_and_keeps_type_and_panel_widths() {
        let comfortable = metrics(Density::Comfortable);
        let compact = metrics(Density::Compact);
        assert_eq!(compact.title_bar, 34.0);
        assert_eq!(compact.status_bar, 22.0);
        assert_eq!(compact.row, 24.0);
        assert_eq!(compact.input, 24.0);
        assert_eq!(compact.button, 24.0);
        assert_eq!(compact.tab, 28.0);
        assert_eq!(compact.sidebar_search, 8.0);
        assert_eq!(comfortable.sidebar_search, 12.0);
        assert_eq!(compact.pad_x, 12.0);
        assert_eq!(compact.pad_y, 8.0);
        assert_eq!(comfortable.pad_x, 16.0);
        assert_eq!(comfortable.pad_y, 12.0);
        assert_eq!(compact.outline_gap, 6.0);
        assert_eq!(compact.palette_inset, 4.0);
        assert_eq!(compact.label_gap, comfortable.label_gap);
        assert_eq!(compact.tree_indent, 16.0);
        assert_eq!(compact.sidebar_width, 280.0);
        assert_eq!(compact.inspector_width, 360.0);
    }

    #[test]
    fn issues_drawer_defaults_to_280_and_clamps_the_drag() {
        assert_eq!(ISSUES_DEFAULT, 280.0);
        assert_eq!(clamp_issues_height(280.0, 800.0), 280.0);
        assert_eq!(clamp_issues_height(10.0, 800.0), 160.0);
        assert_eq!(clamp_issues_height(900.0, 800.0), 400.0);
        assert_eq!(clamp_issues_height(200.0, 300.0), 160.0);
    }
}
