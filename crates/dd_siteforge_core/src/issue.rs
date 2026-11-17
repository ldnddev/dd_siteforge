//! Structured validation results.
//!
//! `message` is the sentence the TUI and CLI already print. Frontends that
//! can select a field use `target`. Nothing here changes the wording.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Where an issue sits in the outline, plus the inspector field id when the
/// check is about one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueTarget {
    pub node: IssueNode,
    pub field: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueNode {
    Site,
    Page(usize),
    Hero {
        page: usize,
        node: usize,
    },
    Section {
        page: usize,
        node: usize,
    },
    Column {
        page: usize,
        node: usize,
        column: usize,
    },
    Component {
        page: usize,
        node: usize,
        column: usize,
        component: usize,
    },
    Header,
    HeaderSection(usize),
    HeaderColumn {
        section: usize,
        column: usize,
    },
    Footer,
    FooterSection(usize),
    FooterColumn {
        section: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
    pub target: Option<IssueTarget>,
}

impl Issue {
    pub fn error(message: impl Into<String>, target: Option<IssueTarget>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            target,
        }
    }

    /// Same as `str::contains` on `message`, so existing checks keep working.
    pub fn contains(&self, pat: &str) -> bool {
        self.message.contains(pat)
    }
}

impl std::fmt::Display for Issue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
