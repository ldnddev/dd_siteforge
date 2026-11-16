//! Workspace paths baked in when this crate is compiled.
use std::path::PathBuf;

/// Root of the dd_siteforge workspace.
///
/// The manifest for this crate is `crates/dd_siteforge_core/Cargo.toml`.
/// `templates/`, `source/`, `Gruntfile.js`, `package.json`, `.lando.yml`,
/// and `.ddev/` sit two directories above that manifest. `init-templates`
/// reads the live `templates/` tree from here when the checkout is still
/// on disk; a release binary falls back to the files compiled into the
/// binary.
pub(crate) fn workspace_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.pop();
    dir.pop();
    dir
}
