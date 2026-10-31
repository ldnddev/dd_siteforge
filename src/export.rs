use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::model::{
    DdRichText, DdSection, Page, PageNode, RobotsDirective, SalAnimation, SchemaType, SectionClass,
    SectionColumn, SectionComponent, SectionItemBoxClass, Site, absolute_url, page_file_name,
    page_public_path, utc_date_ymd,
};
use crate::renderer::render_site_to_dir;
use crate::search_index::write_search_index;

const GRUNT_ASSET_DIRS: &[&str] = &["css", "js", "fonts", "favicon", "vendors"];

#[derive(Debug)]
pub struct ExportReport {
    pub pages: usize,
    pub wrote_404: bool,
}

pub fn export_site(
    site: &Site,
    output_dir: &Path,
    site_root: Option<&Path>,
) -> anyhow::Result<ExportReport> {
    fs::create_dir_all(output_dir).context("failed to create export directory")?;
    render_site_to_dir(site, output_dir, site_root)?;
    copy_built_assets(site_root, output_dir)?;
    copy_source_images(site_root, output_dir)?;
    write_sitemap(site, output_dir)?;
    write_search_index(site, output_dir)?;
    write_robots(site, output_dir)?;
    let wrote_404 = write_404_if_missing(site, output_dir, site_root)?;
    Ok(ExportReport {
        pages: site.pages.len(),
        wrote_404,
    })
}

/// Copy Grunt output (`<site>/web/assets/{css,js,fonts,favicon,vendors}`)
/// into the export dir. When exporting *to* `web/`, skip that copy so a
/// local `grunt build` is not overwritten. Then fill missing fonts and
/// favicon from `source/fonts` and `source/favicon`. `grunt build` copies
/// the font files; `grunt favicon` rebuilds `web/assets/favicon/` from
/// `source/favicon/favicon.png`.
fn copy_built_assets(site_root: Option<&Path>, output_dir: &Path) -> anyhow::Result<()> {
    let Some(root) = site_root else {
        return Ok(());
    };
    let dest_assets = output_dir.join("assets");
    let grunt_assets = root.join("web").join("assets");

    if grunt_assets.exists() && !same_assets_dir(&grunt_assets, &dest_assets) {
        for dir in GRUNT_ASSET_DIRS {
            let src = grunt_assets.join(dir);
            if src.exists() {
                let dest = ensure_real_dest(output_dir, &format!("assets/{dir}"))?;
                copy_dir_recursive(&src, &dest).with_context(|| {
                    format!("failed to copy grunt assets from '{}'", src.display())
                })?;
            }
        }
    }

    let source = root.join("source");
    let fonts = ensure_real_dest(output_dir, "assets/fonts")?;
    let favicon = ensure_real_dest(output_dir, "assets/favicon")?;
    fill_dir_if_empty(&source.join("fonts"), &fonts)?;
    fill_dir_if_empty(&source.join("favicon"), &favicon)?;
    Ok(())
}

fn same_assets_dir(a: &Path, b: &Path) -> bool {
    if let (Ok(ca), Ok(cb)) = (a.canonicalize(), b.canonicalize()) {
        return ca == cb;
    }
    a == b
}

fn fill_dir_if_empty(src: &Path, dest: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    if dest_has_files(dest) {
        return Ok(());
    }
    copy_dir_recursive(src, dest).with_context(|| format!("failed to copy '{}'", src.display()))
}

fn dest_has_files(dir: &Path) -> bool {
    fs::read_dir(dir)
        .ok()
        .map(|mut it| it.next().is_some())
        .unwrap_or(false)
}

pub fn copy_source_images(site_root: Option<&Path>, output_dir: &Path) -> anyhow::Result<()> {
    let Some(root) = site_root else {
        return Ok(());
    };
    let src = root.join("source").join("images");
    if !src.exists() {
        return Ok(());
    }
    let dst = ensure_real_dest(output_dir, "assets/images")?;
    copy_dir_recursive(&src, &dst)
        .with_context(|| format!("failed to copy images from '{}'", src.display()))
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    let root = confinement_root(src)?;
    let mut stack = std::collections::HashSet::new();
    copy_confined(&root, &root, dst, &mut stack)
}

/// Directory under `output_dir` that is not a symlink.
///
/// `join_inside` rejects `..` and a path whose real location already leaves
/// the export root. A symlink is refused even when its target stays inside,
/// so a later `create_dir_all` or `copy` cannot follow it.
fn ensure_real_dest(output_dir: &Path, rel: &str) -> anyhow::Result<PathBuf> {
    let dest = join_inside(output_dir, rel)
        .ok_or_else(|| anyhow::anyhow!("refusing to write '{rel}' outside the export directory"))?;
    if fs::symlink_metadata(&dest).is_ok_and(|meta| meta.file_type().is_symlink()) {
        anyhow::bail!("refusing to write '{rel}' through a symlink");
    }
    Ok(dest)
}

/// Write `bytes` to `root/rel` only when every existing ancestor stays inside `root`.
pub(crate) fn write_inside(root: &Path, rel: &str, bytes: &[u8]) -> anyhow::Result<()> {
    let dest = join_inside(root, rel)
        .ok_or_else(|| anyhow::anyhow!("refusing to write '{rel}' outside the export directory"))?;
    if fs::symlink_metadata(&dest).is_ok_and(|meta| meta.file_type().is_symlink()) {
        anyhow::bail!("refusing to write '{rel}' through a symlink");
    }
    if let Some(parent) = dest.parent() {
        if parent != root {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory '{}'", parent.display()))?;
        }
    }
    fs::write(&dest, bytes).with_context(|| format!("failed to write '{}'", dest.display()))
}

/// Export directory for `serve`.
///
/// An absolute `--output-dir` is the operator's explicit destination.
/// A relative path, and `site.export_dir` when the flag is absent, must stay
/// inside the site directory.
pub fn resolve_serve_output(
    root: &Path,
    explicit: Option<&str>,
    site_export_dir: Option<&str>,
) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
        let path = Path::new(explicit);
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }
        return join_inside(root, explicit).ok_or_else(|| {
            anyhow::anyhow!("export path '{explicit}' must stay inside the site directory")
        });
    }
    let rel = site_export_dir.unwrap_or("web");
    join_inside(root, rel)
        .ok_or_else(|| anyhow::anyhow!("export path '{rel}' must stay inside the site directory"))
}

/// `src` itself must not be a symlink whose target leaves its parent.
/// Nested symlinks are checked again in `copy_confined`.
fn confinement_root(src: &Path) -> std::io::Result<PathBuf> {
    let meta = fs::symlink_metadata(src)?;
    let canon = fs::canonicalize(src)?;
    if meta.file_type().is_symlink() {
        let inside_parent = src.parent().and_then(|parent| {
            fs::canonicalize(parent)
                .ok()
                .map(|parent| canon.starts_with(parent))
        });
        if inside_parent != Some(true) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "refusing to copy a symlink that points outside its parent directory",
            ));
        }
    }
    Ok(canon)
}

fn copy_confined(
    root: &Path,
    src: &Path,
    dst: &Path,
    stack: &mut std::collections::HashSet<PathBuf>,
) -> std::io::Result<()> {
    let here = fs::canonicalize(src)?;
    if !here.starts_with(root) || !stack.insert(here.clone()) {
        return Ok(());
    }
    let copied = (|| {
        refuse_dest_symlink(dst)?;
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(&here)? {
            let entry = entry?;
            let path = entry.path();
            let target = dst.join(entry.file_name());
            let ft = entry.file_type()?;
            if ft.is_symlink() {
                let Ok(canon) = fs::canonicalize(&path) else {
                    continue;
                };
                if !canon.starts_with(root) {
                    continue;
                }
                let meta = fs::metadata(&canon)?;
                if meta.is_dir() {
                    copy_confined(root, &canon, &target, stack)?;
                } else if meta.is_file() {
                    refuse_dest_symlink(&target)?;
                    fs::copy(&canon, &target)?;
                }
                continue;
            }
            if ft.is_dir() {
                copy_confined(root, &path, &target, stack)?;
            } else if ft.is_file() {
                refuse_dest_symlink(&target)?;
                fs::copy(&path, &target)?;
            }
        }
        Ok(())
    })();
    stack.remove(&here);
    copied
}

fn refuse_dest_symlink(path: &Path) -> std::io::Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("refusing to write through symlink '{}'", path.display()),
        ));
    }
    Ok(())
}

/// Join `rel` onto `root` when the result stays inside `root`.
///
/// Empty, `.`, and `./` mean `root`. Absolute paths, `..`, NUL, and a
/// destination whose real path already leaves `root` are rejected.
pub fn join_inside(root: &Path, rel: &str) -> Option<PathBuf> {
    let dest = lexical_inside(root, rel)?;
    destination_inside(root, &dest).then_some(dest)
}

fn lexical_inside(root: &Path, rel: &str) -> Option<PathBuf> {
    let mut s = rel.trim();
    if s.contains('\0') {
        return None;
    }
    while let Some(rest) = s.strip_prefix("./") {
        s = rest.trim_start_matches('/');
    }
    s = s.trim_end_matches('/');
    if s.is_empty() || s == "." {
        return Some(root.to_path_buf());
    }
    let rel_path = Path::new(s);
    if rel_path.is_absolute() {
        return None;
    }
    let safe = rel_path.components().all(|component| {
        matches!(
            component,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    });
    if !safe {
        return None;
    }
    Some(root.join(rel_path))
}

fn destination_inside(root: &Path, dest: &Path) -> bool {
    let Ok(root_canon) = root.canonicalize() else {
        return false;
    };
    let mut probe = dest.to_path_buf();
    loop {
        if probe.exists() {
            return probe
                .canonicalize()
                .is_ok_and(|canon| canon.starts_with(&root_canon));
        }
        if !probe.pop() {
            return false;
        }
    }
}

/// True when `candidate` canonicalizes to `root` or a path inside it.
pub fn path_is_inside(root: &Path, candidate: &Path) -> bool {
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    let Ok(candidate) = candidate.canonicalize() else {
        return false;
    };
    candidate.starts_with(root)
}

fn write_sitemap(site: &Site, output_dir: &Path) -> anyhow::Result<()> {
    let mut body = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );
    let lastmod = utc_date_ymd();
    for page in &site.pages {
        if matches!(
            page.head.robots,
            RobotsDirective::NoindexFollow | RobotsDirective::NoindexNofollow
        ) {
            continue;
        }
        let path = page_public_path(&page.slug, site.pretty_urls);
        let loc = absolute_url(site.base_url.as_deref(), &path).unwrap_or(path);
        body.push_str("  <url>\n    <loc>");
        body.push_str(&xml_escape(&loc));
        body.push_str("</loc>\n    <lastmod>");
        body.push_str(&xml_escape(&lastmod));
        body.push_str("</lastmod>\n  </url>\n");
    }
    body.push_str("</urlset>\n");
    write_inside(output_dir, "sitemap.xml", body.as_bytes()).context("failed to write sitemap.xml")
}

fn write_robots(site: &Site, output_dir: &Path) -> anyhow::Result<()> {
    let mut body = String::from("User-agent: *\nAllow: /\n");
    if let Some(sitemap) = absolute_url(site.base_url.as_deref(), "sitemap.xml") {
        body.push('\n');
        body.push_str("Sitemap: ");
        body.push_str(&sitemap);
        body.push('\n');
    }
    write_inside(output_dir, "robots.txt", body.as_bytes()).context("failed to write robots.txt")
}

fn write_404_if_missing(
    site: &Site,
    output_dir: &Path,
    site_root: Option<&Path>,
) -> anyhow::Result<bool> {
    if site.pages.iter().any(|p| p.slug == "404") {
        return Ok(false);
    }
    let page = not_found_page();
    let r = crate::templates::Renderer::load(site_root)?;
    let html = crate::renderer::render_page_html_with_chrome(
        &r,
        &page,
        &crate::renderer::render_header(&r, &site.header)?,
        &crate::renderer::render_footer(&r, &site.footer, &site.name)?,
        site,
        site_root,
    )?;
    write_inside(output_dir, &page_file_name("404", false), html.as_bytes())
        .context("failed to write 404.html")?;
    Ok(true)
}

fn not_found_page() -> Page {
    Page {
        id: "page-404".to_string(),
        slug: "404".to_string(),
        slug_locked: true,
        head: crate::model::DdHead {
            title: "Not Found".to_string(),
            meta_title: None,
            meta_description: Some("This page does not exist.".to_string()),
            canonical_url: None,
            robots: RobotsDirective::NoindexNofollow,
            schema_type: SchemaType::WebPage,
            og_title: None,
            og_description: None,
            og_image: None,
            og_image_alt: None,
            og_image_width: None,
            og_image_height: None,
            twitter_card: None,
        },
        nodes: vec![PageNode::Section(DdSection {
            id: "section-404".to_string(),
            section_title: None,
            section_class: Some(SectionClass::FullContained),
            item_box_class: Some(SectionItemBoxClass::LBox),
            bg: None,
            padding: None,
            custom_css: None,
            aria_label: Some("Not Found".to_string()),
            sal: SalAnimation::NoAnimation,
            sal_duration: None,
            sal_delay: None,
            columns: vec![SectionColumn {
                id: "column-1".to_string(),
                width_class: "dd-u-1-1".to_string(),
                components: vec![SectionComponent::RichText(DdRichText {
                    parent_class: None,
                    sal: crate::model::SalAnimation::NoAnimation,
                    sal_duration: None,
                    sal_delay: None,
                    parent_copy: "# Not Found\n\nThis page does not exist.".to_string(),
                })],
            }],
        })],
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Site;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir(prefix: &str) -> std::path::PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{n}"))
    }

    #[test]
    fn export_writes_html_sitemap_robots_and_404() {
        let out = tmp_dir("dd_export_full");
        let site = Site::starter();
        let report = export_site(&site, &out, None).expect("export");
        assert_eq!(report.pages, 1);
        assert!(report.wrote_404);
        assert!(out.join("index.html").exists());
        assert!(out.join("sitemap.xml").exists());
        assert!(out.join("search-index.json").exists());
        assert!(out.join("robots.txt").exists());
        assert!(out.join("404.html").exists());
        let not_found = fs::read_to_string(out.join("404.html")).unwrap();
        assert!(not_found.contains("<h1>Not Found</h1>"), "{not_found}");
        assert!(
            not_found.contains("href=\"/assets/css/style.min.css\""),
            "404 CSS must be root-relative for deep missing URLs: {not_found}"
        );
        assert!(
            !not_found.contains("rel=\"canonical\""),
            "auto 404 must not canonicalize to /404/index.html: {not_found}"
        );
        assert!(!not_found.contains("404/index.html"), "{not_found}");
        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("assets/css/style.min.css"));
        assert!(html.contains("lang=\"en\""));
        assert!(
            html.contains("data-search-index=\"search-index.json\"")
                || html.contains("data-search-index=\"/search-index.json\"")
        );
        let index: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out.join("search-index.json")).unwrap())
                .unwrap();
        assert_eq!(index["v"], 1);
        assert_eq!(index["pages"][0]["url"], "index.html");
        assert_eq!(index["pages"][0]["path"], "/");
        assert_eq!(index["pages"][0]["title"], "Home");
        assert!(
            !out.join("assets/css/style.min.css").exists(),
            "css comes from grunt, not a bundled pack"
        );
        std::fs::remove_dir_all(&out).ok();
    }

    fn crate_source() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("source")
    }

    #[test]
    fn export_copies_source_fonts_and_favicon_when_present() {
        let root = tmp_dir("dd_export_fonts_root");
        copy_dir_recursive(&crate_source().join("fonts"), &root.join("source/fonts")).unwrap();
        copy_dir_recursive(
            &crate_source().join("favicon"),
            &root.join("source/favicon"),
        )
        .unwrap();
        let out = root.join("out");
        export_site(&Site::starter(), &out, Some(&root)).expect("export");
        assert!(
            out.join("assets/fonts/Lato-Regular.woff2").exists(),
            "Lato files in source/fonts fill assets/fonts"
        );
        assert!(!out.join("assets/webfonts").exists());
        assert!(out.join("assets/favicon/favicon.png").exists());
        assert!(
            out.join("assets/favicon/favicon.svg").exists(),
            "source/favicon is the icon set the head template links"
        );
        assert!(out.join("assets/favicon/favicon.ico").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn export_copies_grunt_assets_when_dest_is_not_web() {
        let root = tmp_dir("dd_export_grunt_root");
        let grunt_css = root.join("web/assets/css");
        fs::create_dir_all(&grunt_css).unwrap();
        fs::write(grunt_css.join("style.min.css"), b"/* grunt */").unwrap();
        let out = root.join("dist");
        export_site(&Site::starter(), &out, Some(&root)).expect("export");
        let css = fs::read_to_string(out.join("assets/css/style.min.css")).unwrap();
        assert_eq!(css, "/* grunt */");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn export_to_web_does_not_clobber_grunt_css() {
        let root = tmp_dir("dd_export_noclobber");
        let grunt_css = root.join("web/assets/css");
        fs::create_dir_all(&grunt_css).unwrap();
        fs::write(grunt_css.join("style.min.css"), b"/* keep */").unwrap();
        export_site(&Site::starter(), &root.join("web"), Some(&root)).expect("export");
        let css = fs::read_to_string(root.join("web/assets/css/style.min.css")).unwrap();
        assert_eq!(css, "/* keep */");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn export_copies_source_images() {
        let root = tmp_dir("dd_export_imgs_root");
        let imgs = root.join("source").join("images");
        fs::create_dir_all(&imgs).unwrap();
        fs::write(imgs.join("hero.jpg"), b"fake").unwrap();
        let out = root.join("web");
        export_site(&Site::starter(), &out, Some(&root)).expect("export");
        assert!(out.join("assets/images/hero.jpg").exists());
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sitemap_uses_base_url_and_skips_noindex() {
        let out = tmp_dir("dd_export_sitemap");
        let mut site = Site::starter();
        site.base_url = Some("https://ex.com".to_string());
        site.pages[0].head.robots = RobotsDirective::IndexFollow;
        site.pages.push(Page::from_template(
            "Hidden",
            crate::model::PageTemplate::HeroPlusSection,
        ));
        site.pages[1].head.robots = RobotsDirective::NoindexNofollow;
        export_site(&site, &out, None).expect("export");
        let map = fs::read_to_string(out.join("sitemap.xml")).unwrap();
        assert!(map.contains("https://ex.com/"), "{map}");
        assert!(map.contains("<lastmod>"), "{map}");
        assert!(!map.contains("hidden.html"));
        assert!(!map.contains("index.html"), "{map}");
        let robots = fs::read_to_string(out.join("robots.txt")).unwrap();
        assert!(robots.contains("Sitemap: https://ex.com/sitemap.xml"));
        let index: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out.join("search-index.json")).unwrap())
                .unwrap();
        assert_eq!(index["pages"].as_array().unwrap().len(), 1);
        assert_eq!(index["pages"][0]["title"], "Home");
        std::fs::remove_dir_all(&out).ok();
    }

    #[test]
    fn pretty_urls_write_nested_index_html() {
        let out = tmp_dir("dd_export_pretty");
        let mut site = Site::starter();
        site.pretty_urls = true;
        site.pages[0].slug = "blog".to_string();
        site.pages.push(Page::from_template(
            "Entry",
            crate::model::PageTemplate::Blank,
        ));
        site.pages[1].slug = "blog/entry".to_string();
        export_site(&site, &out, None).expect("export");
        assert!(out.join("blog/index.html").exists());
        assert!(out.join("blog/entry/index.html").exists());
        let listing = fs::read_to_string(out.join("blog/index.html")).unwrap();
        assert!(
            listing.contains("../assets/css/style.min.css"),
            "blog index needs one ../ for assets: {listing}"
        );
        assert!(
            listing.contains("data-search-index=\"../search-index.json\"")
                || listing.contains("data-search-index=\"/search-index.json\""),
            "blog index needs a resolvable search-index: {listing}"
        );
        let entry = fs::read_to_string(out.join("blog/entry/index.html")).unwrap();
        assert!(
            entry.contains("../../assets/css/style.min.css"),
            "blog entry needs two ../ for assets: {entry}"
        );
        assert!(
            entry.contains("data-search-index=\"../../search-index.json\"")
                || entry.contains("data-search-index=\"/search-index.json\""),
            "blog entry needs a resolvable search-index: {entry}"
        );
        let index: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out.join("search-index.json")).unwrap())
                .unwrap();
        assert_eq!(index["pages"][0]["url"], "blog/index.html");
        assert_eq!(index["pages"][0]["path"], "/blog/");
        assert_eq!(index["pages"][1]["url"], "blog/entry/index.html");
        assert_eq!(index["pages"][1]["path"], "/blog/entry/");
        let map = fs::read_to_string(out.join("sitemap.xml")).unwrap();
        assert!(map.contains("<loc>/blog/</loc>"), "{map}");
        assert!(map.contains("<loc>/blog/entry/</loc>"), "{map}");
        assert!(map.contains("<lastmod>"), "{map}");
        assert!(!map.contains("blog/index.html"), "{map}");
        std::fs::remove_dir_all(&out).ok();
    }

    #[test]
    fn pretty_404_uses_root_assets_h1_and_skips_canonical() {
        let out = tmp_dir("dd_export_pretty_404");
        let mut site = Site::starter();
        site.pretty_urls = true;
        site.base_url = Some("https://www.ldnddev.com".to_string());
        export_site(&site, &out, None).expect("export");
        assert!(out.join("404.html").exists());
        assert!(!out.join("404/index.html").exists());
        let html = fs::read_to_string(out.join("404.html")).unwrap();
        assert!(html.contains("<h1>Not Found</h1>"), "{html}");
        assert!(
            html.contains("href=\"https://www.ldnddev.com/assets/css/style.min.css\""),
            "404 CSS must be absolute so deep missing URLs still load: {html}"
        );
        assert!(!html.contains("rel=\"canonical\""), "{html}");
        assert!(!html.contains("404/index.html"), "{html}");
        let home = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(
            home.contains("rel=\"canonical\" href=\"https://www.ldnddev.com/\""),
            "{home}"
        );
        std::fs::remove_dir_all(&out).ok();
    }

    #[test]
    fn does_not_overwrite_author_404_page() {
        let out = tmp_dir("dd_export_custom_404");
        let mut site = Site::starter();
        let mut p = Page::from_template("Not Found", crate::model::PageTemplate::HeroPlusSection);
        p.slug = "404".to_string();
        p.head.title = "Custom 404".to_string();
        site.pages.push(p);
        let report = export_site(&site, &out, None).expect("export");
        assert!(!report.wrote_404);
        let html = fs::read_to_string(out.join("404.html")).unwrap();
        assert!(html.contains("Custom 404"));
        std::fs::remove_dir_all(&out).ok();
    }

    #[test]
    fn join_inside_keeps_relative_paths_and_rejects_escape() {
        let root = tmp_dir("dd_join_inside");
        fs::create_dir_all(&root).unwrap();
        assert_eq!(join_inside(&root, "web").unwrap(), root.join("web"));
        assert_eq!(join_inside(&root, "./web/").unwrap(), root.join("web"));
        assert_eq!(join_inside(&root, "").unwrap(), root);
        assert_eq!(join_inside(&root, ".").unwrap(), root);
        assert!(join_inside(&root, "../x").is_none());
        assert!(join_inside(&root, "/tmp/x").is_none());
        assert!(join_inside(&root, "web/../../etc").is_none());
        assert!(join_inside(&root, "web\0").is_none());
        let sibling = std::env::temp_dir().join(format!(
            "{}-evil",
            root.file_name().unwrap().to_string_lossy()
        ));
        fs::create_dir_all(&sibling).unwrap();
        fs::write(sibling.join("secret.txt"), b"nope").unwrap();
        assert!(!path_is_inside(&root, &sibling));
        assert!(!path_is_inside(&root, &sibling.join("secret.txt")));
        fs::write(root.join("inside.txt"), b"yes").unwrap();
        assert!(path_is_inside(&root, &root.join("inside.txt")));
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&sibling).ok();
    }

    #[cfg(unix)]
    #[test]
    fn copy_skips_symlinks_that_leave_the_source_tree() {
        let root = tmp_dir("dd_copy_symlink");
        let outside = tmp_dir("dd_copy_symlink_out");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), b"SECRET").unwrap();
        let images = root.join("images");
        fs::create_dir_all(&images).unwrap();
        fs::write(images.join("hero.jpg"), b"hero").unwrap();
        std::os::unix::fs::symlink(outside.join("secret.txt"), images.join("leak.txt")).unwrap();
        std::os::unix::fs::symlink(&outside, images.join("leak-dir")).unwrap();
        std::os::unix::fs::symlink(images.join("hero.jpg"), images.join("alias.jpg")).unwrap();
        let nested = images.join("nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("a.jpg"), b"a").unwrap();
        std::os::unix::fs::symlink(&nested, images.join("linked")).unwrap();

        let dst = root.join("out");
        copy_dir_recursive(&images, &dst).unwrap();
        assert_eq!(fs::read(dst.join("hero.jpg")).unwrap(), b"hero");
        assert_eq!(fs::read(dst.join("alias.jpg")).unwrap(), b"hero");
        assert_eq!(fs::read(dst.join("linked/a.jpg")).unwrap(), b"a");
        assert!(!dst.join("leak.txt").exists());
        assert!(!dst.join("leak-dir").exists());
        assert!(!dst.join("secret.txt").exists());

        let link_root = root.join("escaped");
        std::os::unix::fs::symlink(&outside, &link_root).unwrap();
        let err = copy_dir_recursive(&link_root, &root.join("escaped-out")).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(!root.join("escaped-out/secret.txt").exists());

        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[cfg(unix)]
    #[test]
    fn join_inside_rejects_a_symlink_that_leaves_the_site() {
        let root = tmp_dir("dd_join_symlink");
        let outside = tmp_dir("dd_join_symlink_out");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("web")).unwrap();
        assert!(join_inside(&root, "web").is_none());
        assert_eq!(join_inside(&root, "dist").unwrap(), root.join("dist"));
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[test]
    fn export_refuses_an_unsafe_page_slug() {
        let base = tmp_dir("dd_export_unsafe_base");
        let out = base.join("out");
        fs::create_dir_all(&out).unwrap();
        let marker = base.join("index.html");
        fs::write(&marker, b"keep").unwrap();
        let mut site = Site::starter();
        site.pretty_urls = true;
        site.pages[0].slug = "..".to_string();
        let err = export_site(&site, &out, None).unwrap_err();
        assert!(err.to_string().contains("unsafe page slug"), "{err}");
        assert_eq!(fs::read(&marker).unwrap(), b"keep");
        assert!(!out.join("index.html").exists());
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn resolve_serve_output_keeps_relative_paths_inside_the_site() {
        let root = tmp_dir("dd_serve_output");
        fs::create_dir_all(&root).unwrap();
        let web = resolve_serve_output(&root, None, None).unwrap();
        assert_eq!(web, root.join("web"));
        let nested = resolve_serve_output(&root, Some("./dist/preview"), None).unwrap();
        assert_eq!(nested, root.join("dist/preview"));
        assert!(resolve_serve_output(&root, Some("../outside"), None).is_err());
        assert!(resolve_serve_output(&root, None, Some("../outside")).is_err());
        let abs = std::env::temp_dir().join("dd_serve_abs");
        assert_eq!(
            resolve_serve_output(&root, Some(abs.to_str().unwrap()), None).unwrap(),
            abs
        );
        std::fs::remove_dir_all(&root).ok();
    }

    #[cfg(unix)]
    #[test]
    fn export_does_not_follow_a_symlink_out_of_the_output_dir() {
        let out = tmp_dir("dd_export_symlink_page");
        let outside = tmp_dir("dd_export_symlink_page_out");
        fs::create_dir_all(&out).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let secret = outside.join("secret.txt");
        fs::write(&secret, b"SECRET").unwrap();
        std::os::unix::fs::symlink(&secret, out.join("index.html")).unwrap();
        let err = export_site(&Site::starter(), &out, None).unwrap_err();
        assert!(err.to_string().contains("refusing to write"), "{err}");
        assert_eq!(fs::read(&secret).unwrap(), b"SECRET");
        std::fs::remove_dir_all(&out).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[cfg(unix)]
    #[test]
    fn export_does_not_copy_assets_through_a_symlink() {
        let root = tmp_dir("dd_export_symlink_assets_root");
        let outside = tmp_dir("dd_export_symlink_assets_out");
        fs::create_dir_all(&outside).unwrap();
        fs::write(outside.join("secret.txt"), b"SECRET").unwrap();
        copy_dir_recursive(&crate_source().join("fonts"), &root.join("source/fonts")).unwrap();
        let out = root.join("out");
        fs::create_dir_all(&out).unwrap();
        std::os::unix::fs::symlink(&outside, out.join("assets")).unwrap();
        let err = export_site(&Site::starter(), &out, Some(&root)).unwrap_err();
        assert!(err.to_string().contains("refusing to write"), "{err}");
        assert_eq!(fs::read(outside.join("secret.txt")).unwrap(), b"SECRET");
        assert!(!outside.join("fonts").exists());
        assert!(!outside.join("Lato-Regular.woff2").exists());
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }

    #[cfg(unix)]
    #[test]
    fn copy_refuses_to_overwrite_a_destination_symlink() {
        let root = tmp_dir("dd_copy_dest_symlink");
        let outside = tmp_dir("dd_copy_dest_symlink_out");
        fs::create_dir_all(&outside).unwrap();
        let secret = outside.join("secret.txt");
        fs::write(&secret, b"SECRET").unwrap();
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("style.css"), b"css").unwrap();
        let dst = root.join("dst");
        fs::create_dir_all(&dst).unwrap();
        std::os::unix::fs::symlink(&secret, dst.join("style.css")).unwrap();
        let err = copy_dir_recursive(&src, &dst).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert_eq!(fs::read(&secret).unwrap(), b"SECRET");
        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&outside).ok();
    }
}
