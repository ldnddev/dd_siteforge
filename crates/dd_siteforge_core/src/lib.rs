//! Site document and the commands that load, check, and publish it.
//!
//! `site.json` is read and written here. Field contracts (`EditForm`), the
//! insert-kind list, validation, page health, HTML export, the preview
//! server, template seeding, and the Grunt/Lando kit live here too. A
//! frontend keeps its own screen, focus, and toasts and calls these
//! functions instead of copying them.

pub mod asset_build;
pub mod component_kind;
pub mod cursor;
pub mod editform;
pub mod export;
pub mod health;
pub mod labels;
pub mod markdown;
pub mod model;
pub mod og_image;
pub mod paths;
pub mod renderer;
pub mod scaffold;
pub mod search_index;
pub mod serve;
pub mod session;
pub mod storage;
pub mod templates;
pub mod validate;
