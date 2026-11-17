//! Empty and error copy for each screen. The sentences are the ones the shell
//! already shows, or the TUI sentence the spec keeps.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenCopy {
    pub screen: &'static str,
    pub empty: &'static str,
    pub error: &'static str,
}

pub fn catalog() -> &'static [ScreenCopy] {
    &CATALOG
}

const CATALOG: &[ScreenCopy] = &[
    ScreenCopy {
        screen: "welcome",
        empty: "No recent sites",
        error: "Couldn't load site.json",
    },
    ScreenCopy {
        screen: "loading",
        empty: "Reading site.json…",
        error: "Couldn't load site.json",
    },
    ScreenCopy {
        screen: "sidebar",
        empty: "No pages",
        error: "No pages match",
    },
    ScreenCopy {
        screen: "outline",
        empty: "No nodes on this page.",
        error: "Issue",
    },
    ScreenCopy {
        screen: "inspector",
        empty: "Nothing selected",
        error: "is required.",
    },
    ScreenCopy {
        screen: "issues",
        empty: "No validation errors.",
        error: "Go to field",
    },
    ScreenCopy {
        screen: "find",
        empty: "No matches.",
        error: "No matches.",
    },
    ScreenCopy {
        screen: "palette",
        empty: "No command selected.",
        error: "No command selected.",
    },
    ScreenCopy {
        screen: "export",
        empty: "Exporting",
        error: "Export failed",
    },
    ScreenCopy {
        screen: "preview",
        empty: "Preview not built",
        error: "Preview failed · Show issue",
    },
    ScreenCopy {
        screen: "new-site",
        empty: "New site",
        error: "already exists",
    },
    ScreenCopy {
        screen: "build",
        empty: "Build CSS/JS",
        error: "Build thread stopped.",
    },
    ScreenCopy {
        screen: "image",
        empty: "Drop an image, or browse",
        error: "missing",
    },
    ScreenCopy {
        screen: "brand",
        empty: "dd_wcag",
        error: "dd_wcag missing",
    },
    ScreenCopy {
        screen: "settings",
        empty: "OK (loaded cleanly)",
        error: "Theme save failed",
    },
    ScreenCopy {
        screen: "shortcuts",
        empty: "Shortcut sheet",
        error: "Shortcut sheet",
    },
    ScreenCopy {
        screen: "template",
        empty: "New page — choose template",
        error: "Add page cancelled.",
    },
    ScreenCopy {
        screen: "switcher",
        empty: "No recent sites",
        error: "Recent sites",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_screen_has_an_empty_and_an_error_line() {
        assert!(catalog().len() >= 16);
        let mut names = Vec::new();
        for row in catalog() {
            assert!(!row.screen.is_empty(), "{row:?}");
            assert!(!row.empty.is_empty(), "{}", row.screen);
            assert!(!row.error.is_empty(), "{}", row.screen);
            names.push(row.screen);
        }
        for required in [
            "welcome",
            "loading",
            "sidebar",
            "outline",
            "inspector",
            "issues",
            "find",
            "palette",
            "export",
            "preview",
            "new-site",
            "build",
            "image",
            "brand",
            "settings",
            "shortcuts",
        ] {
            assert!(names.contains(&required), "{required}");
        }
    }
}
