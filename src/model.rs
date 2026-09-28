use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub theme: ThemeSettings,
    pub header: DdHeader,
    pub footer: DdFooter,
    pub pages: Vec<Page>,
    /// Persisted export output directory, relative to the site JSON file.
    /// `None` triggers a first-time prompt; user-confirmed value is written back.
    #[serde(default)]
    pub export_dir: Option<String>,
    /// Origin used for canonical URLs, Open Graph, and sitemap `<loc>`.
    /// Example: `https://example.com`. Empty/None skips absolute URL generation.
    #[serde(default)]
    pub base_url: Option<String>,
    /// `<html lang>` value. Defaults to `en` for legacy JSON.
    #[serde(default = "default_lang")]
    pub lang: String,
    /// Pasted GTM `<script>` snippet. Export emits a canonical snippet when a `GTM-XXXX` id is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header_gtm_tag: Option<String>,
    /// Pasted GTM `<noscript>` iframe snippet. Export emits a canonical noscript when a `GTM-XXXX` id is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_gtm_tag: Option<String>,
}

fn default_lang() -> String {
    "en".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    pub primary_color: String,
    pub secondary_color: String,
    pub tertiary_color: String,
    pub support_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub slug: String,
    #[serde(default)]
    pub slug_locked: bool,
    pub head: DdHead,
    pub nodes: Vec<PageNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "node_type", rename_all = "snake_case")]
pub enum PageNode {
    Hero(DdHero),
    Section(DdSection),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdHero {
    pub parent_image_url: String,
    pub parent_image_alt: Option<String>,
    pub parent_class: Option<HeroImageClass>,
    #[serde(default, alias = "parent_data_aos")]
    pub sal: Option<SalAnimation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_custom_css: Option<String>,
    pub parent_title: String,
    pub parent_subtitle: String,
    pub parent_copy: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DdLink>,
    /// Legacy hero slot 1. Used when `links` is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_1_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_1_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_1_target: Option<CtaTarget>,
    /// Legacy hero slot 2. Used when `links` is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_2_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_2_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_2_target: Option<CtaTarget>,
    pub parent_image_mobile: Option<String>,
    pub parent_image_tablet: Option<String>,
    pub parent_image_desktop: Option<String>,
    pub parent_image_class: Option<HeroImageClass>,
    #[serde(default, skip_serializing_if = "Media::is_none")]
    pub media: Media,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overlay: Option<HeroOverlay>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copy_position: Option<HeroCopyPosition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdSection {
    pub id: String,
    #[serde(default)]
    pub section_title: Option<String>,
    pub section_class: Option<SectionClass>,
    pub item_box_class: Option<SectionItemBoxClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<SectionBg>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<SectionPadding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_css: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
    #[serde(default = "default_section_sal")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    #[serde(default)]
    pub columns: Vec<SectionColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionColumn {
    pub id: String,
    pub width_class: String,
    pub components: Vec<SectionComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "component_type", rename_all = "snake_case")]
pub enum SectionComponent {
    Alternating(DdAlternating),
    Card(DdCard),
    Cta(DdCta),
    Filmstrip(DdFilmstrip),
    Milestones(DdMilestones),
    Slider(DdSlider),
    Modal(DdModal),
    Banner(DdBanner),
    Accordion(DdAccordion),
    Blockquote(DdBlockquote),
    Alert(DdAlert),
    Image(DdImage),
    RichText(DdRichText),
    Navigation(DdNavigation),
    HeaderSearch(DdHeaderSearch),
    HeaderMenu(DdHeaderMenu),
    Spacer(DdSpacer),
    Tabs(DdTabs),
    Timeline(DdTimeline),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdAlternating {
    #[serde(default = "default_alternating_parent_type")]
    pub parent_type: AlternatingType,
    #[serde(default = "default_alternating_parent_class")]
    pub parent_class: String,
    #[serde(default = "default_alternating_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub items: Vec<AlternatingItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternatingItem {
    pub child_image_url: String,
    pub child_image_alt: String,
    pub child_title: String,
    #[serde(default)]
    pub child_subtitle: String,
    pub child_copy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DdLink>,
    #[serde(default, skip_serializing_if = "Media::is_none")]
    pub media: Media,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdCard {
    #[serde(default = "default_card_parent_type")]
    pub parent_type: CardType,
    #[serde(default = "default_card_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    #[serde(default = "default_card_parent_width")]
    pub parent_width: String,
    pub items: Vec<CardItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardItem {
    pub child_image_url: String,
    pub child_image_alt: String,
    pub child_title: String,
    pub child_subtitle: String,
    pub child_copy: String,
    pub child_link_url: Option<String>,
    pub child_link_target: Option<CardLinkTarget>,
    pub child_link_label: Option<String>,
    #[serde(default)]
    pub child_link_style: ButtonStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdBanner {
    #[serde(default = "default_banner_parent_class")]
    pub parent_class: BannerClass,
    #[serde(default = "default_banner_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_image_url: String,
    pub parent_image_alt: String,
    #[serde(default, skip_serializing_if = "Media::is_none")]
    pub media: Media,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdCta {
    #[serde(default = "default_cta_parent_class")]
    pub parent_class: CtaClass,
    pub parent_image_url: String,
    pub parent_image_alt: String,
    #[serde(default = "default_cta_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_title: String,
    pub parent_subtitle: String,
    pub parent_copy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DdLink>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_link_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_link_target: Option<CardLinkTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_link_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdFilmstrip {
    #[serde(default = "default_filmstrip_parent_type")]
    pub parent_type: FilmstripType,
    #[serde(default = "default_filmstrip_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub items: Vec<FilmstripItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmstripItem {
    pub child_image_url: String,
    pub child_image_alt: String,
    pub child_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdMilestones {
    #[serde(default = "default_milestones_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    #[serde(default = "default_milestones_parent_width")]
    pub parent_width: String,
    pub items: Vec<MilestonesItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonesItem {
    pub child_percentage: String,
    pub child_title: String,
    pub child_subtitle: String,
    pub child_copy: String,
    pub child_link_url: Option<String>,
    pub child_link_target: Option<CardLinkTarget>,
    pub child_link_label: Option<String>,
    #[serde(default)]
    pub child_link_style: ButtonStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdModal {
    pub parent_title: String,
    pub parent_copy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdSlider {
    #[serde(default)]
    pub parent_title: String,
    pub items: Vec<SliderItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliderItem {
    pub child_title: String,
    pub child_copy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DdLink>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_link_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_link_target: Option<CardLinkTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_link_label: Option<String>,
    pub child_image_url: String,
    pub child_image_alt: String,
    #[serde(default, skip_serializing_if = "Media::is_none")]
    pub media: Media,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdAccordion {
    #[serde(default = "default_accordion_parent_type")]
    pub parent_type: AccordionType,
    #[serde(default = "default_accordion_parent_class")]
    pub parent_class: AccordionClass,
    #[serde(default = "default_accordion_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    #[serde(default = "default_accordion_parent_group_name")]
    pub parent_group_name: String,
    pub items: Vec<AccordionItem>,
    pub multiple: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccordionItem {
    pub child_title: String,
    pub child_copy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdAlert {
    #[serde(default = "default_alert_parent_type")]
    pub parent_type: AlertType,
    #[serde(default = "default_alert_parent_class")]
    pub parent_class: AlertClass,
    #[serde(default = "default_alert_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_title: Option<String>,
    pub parent_copy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpacerSize {
    #[serde(rename = "-sm")]
    Sm,
    #[serde(rename = "-md")]
    Md,
    #[serde(rename = "-lg")]
    Lg,
    #[serde(rename = "-xl")]
    Xl,
    #[serde(rename = "-xxl")]
    Xxl,
    #[serde(rename = "-xxxl")]
    Xxxl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdSpacer {
    #[serde(default = "default_spacer_size")]
    pub size: SpacerSize,
    #[serde(default)]
    pub divider: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabsOrientation {
    #[serde(rename = "-horizontal")]
    Horizontal,
    #[serde(rename = "-vertical")]
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdTabs {
    pub parent_id: String,
    #[serde(default = "default_tabs_orientation")]
    pub parent_class: TabsOrientation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
    #[serde(default = "default_tabs_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub items: Vec<TabsItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabsItem {
    pub child_title: String,
    pub child_copy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdTimeline {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
    #[serde(default = "default_timeline_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub items: Vec<TimelineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItem {
    pub child_year: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_datetime: Option<String>,
    pub child_title: String,
    #[serde(default = "default_heading_level")]
    pub heading_level: u8,
    pub child_copy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_image_alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdBlockquote {
    #[serde(default = "default_blockquote_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_image_url: String,
    pub parent_image_alt: String,
    pub parent_name: String,
    pub parent_role: String,
    pub parent_copy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdImage {
    #[serde(default = "default_image_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_image_url: String,
    #[serde(default)]
    pub parent_image_url_dark: Option<String>,
    pub parent_image_alt: String,
    pub parent_link_url: Option<String>,
    pub parent_link_target: Option<CardLinkTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdRichText {
    #[serde(default)]
    pub parent_class: Option<String>,
    #[serde(default = "default_rich_text_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub parent_copy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdNavigation {
    #[serde(default = "default_navigation_parent_type")]
    pub parent_type: NavigationType,
    #[serde(default = "default_navigation_parent_class")]
    pub parent_class: NavigationClass,
    #[serde(default = "default_navigation_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
    pub items: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationItem {
    #[serde(default = "default_navigation_child_kind")]
    pub child_kind: NavigationKind,
    pub child_link_label: String,
    pub child_link_url: Option<String>,
    pub child_link_target: Option<CardLinkTarget>,
    pub child_link_css: Option<String>,
    #[serde(default)]
    pub items: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdHeaderSearch {
    #[serde(default = "default_header_search_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdHeaderMenu {
    #[serde(default = "default_header_menu_sal", alias = "parent_data_aos")]
    pub sal: SalAnimation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_duration: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sal_delay: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdHeader {
    pub id: String,
    pub custom_css: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cta_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cta_url: Option<String>,
    /// Thin site-wide notice above `.dd-header__top`. Separate from optional `alert`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(default)]
    pub alert: Option<DdAlert>,
    #[serde(default)]
    pub sections: Vec<DdSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdFooter {
    pub id: String,
    pub custom_css: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blurb: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub copyright: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub social_linkedin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub social_x: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub social_github: Option<String>,
    #[serde(default)]
    pub sections: Vec<DdSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdHead {
    /// TUI / Pages-panel label. Not the HTML `<title>` unless `meta_title` is empty.
    pub title: String,
    /// Document `<title>`. When empty/None, export falls back to `title`.
    #[serde(default)]
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub canonical_url: Option<String>,
    #[serde(default = "default_head_robots")]
    pub robots: RobotsDirective,
    #[serde(default = "default_head_schema_type")]
    pub schema_type: SchemaType,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
}

impl DdHead {
    /// Value written to `<title>` (and OG/schema name fallbacks).
    pub fn html_title(&self) -> &str {
        self.meta_title
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.title.trim())
    }
}

fn default_alert_parent_type() -> AlertType {
    AlertType::Default
}

fn default_alert_parent_class() -> AlertClass {
    AlertClass::Default
}

fn default_alert_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_spacer_size() -> SpacerSize {
    SpacerSize::Md
}

fn default_tabs_orientation() -> TabsOrientation {
    TabsOrientation::Horizontal
}

fn default_tabs_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_timeline_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_heading_level() -> u8 {
    3
}

fn default_accordion_parent_group_name() -> String {
    "group1".to_string()
}

fn default_accordion_parent_type() -> AccordionType {
    AccordionType::Default
}

fn default_accordion_parent_class() -> AccordionClass {
    AccordionClass::Primary
}

fn default_accordion_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_alternating_parent_type() -> AlternatingType {
    AlternatingType::Default
}

fn default_alternating_parent_class() -> String {
    "-default".to_string()
}

fn default_alternating_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_card_parent_type() -> CardType {
    CardType::Default
}

fn default_card_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_section_sal() -> SalAnimation {
    SalAnimation::NoAnimation
}

fn default_card_parent_width() -> String {
    "dd-u-1-1 dd-u-md-12-24 dd-u-lg-8-24".to_string()
}

fn default_banner_parent_class() -> BannerClass {
    BannerClass::BgCenterCenter
}

fn default_banner_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_cta_parent_class() -> CtaClass {
    CtaClass::TopLeft
}

fn default_cta_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_filmstrip_parent_type() -> FilmstripType {
    FilmstripType::Default
}

fn default_filmstrip_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_milestones_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_milestones_parent_width() -> String {
    "dd-u-1-1 dd-u-md-12-24".to_string()
}

fn default_blockquote_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_image_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_rich_text_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_navigation_parent_type() -> NavigationType {
    NavigationType::HeaderNav
}

fn default_navigation_parent_class() -> NavigationClass {
    NavigationClass::MainMenu
}

fn default_navigation_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_navigation_child_kind() -> NavigationKind {
    NavigationKind::Link
}

fn default_header_search_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_header_menu_sal() -> SalAnimation {
    SalAnimation::Fade
}

fn default_head_robots() -> RobotsDirective {
    RobotsDirective::IndexFollow
}

fn default_head_schema_type() -> SchemaType {
    SchemaType::WebPage
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CtaTarget {
    #[serde(rename = "_self")]
    SelfTarget,
    #[serde(rename = "_blank")]
    Blank,
    #[serde(rename = "_parent")]
    Parent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeroImageClass {
    #[serde(rename = "-contained")]
    Contained,
    #[serde(rename = "-contained-md")]
    ContainedMd,
    #[serde(rename = "-contained-lg")]
    ContainedLg,
    #[serde(rename = "-contained-xl")]
    ContainedXl,
    #[serde(rename = "-contained-xxl")]
    ContainedXxl,
    #[serde(rename = "-full-full")]
    FullFull,
    #[serde(rename = "-full-contained")]
    FullContained,
    #[serde(rename = "-full-contained-md")]
    FullContainedMd,
    #[serde(rename = "-full-contained-lg")]
    FullContainedLg,
    #[serde(rename = "-full-contained-xl")]
    FullContainedXl,
    #[serde(rename = "-full-contained-xxl")]
    FullContainedXxl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeroOverlay {
    #[serde(rename = "-overlay-light")]
    Light,
    #[serde(rename = "-overlay-dark")]
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeroCopyPosition {
    #[serde(rename = "-left")]
    Left,
    #[serde(rename = "-center")]
    Center,
    #[serde(rename = "-right")]
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SalAnimation {
    #[serde(rename = "no-animation")]
    NoAnimation,
    #[serde(rename = "fade", alias = "fade-in")]
    Fade,
    #[serde(rename = "slide-up", alias = "fade-up")]
    SlideUp,
    #[serde(rename = "slide-down", alias = "fade-down")]
    SlideDown,
    #[serde(rename = "slide-left", alias = "fade-left")]
    SlideLeft,
    #[serde(rename = "slide-right", alias = "fade-right")]
    SlideRight,
    #[serde(rename = "zoom-in", alias = "zoom-in-up", alias = "zoom-in-down")]
    ZoomIn,
    #[serde(rename = "zoom-out")]
    ZoomOut,
    #[serde(rename = "flip-up")]
    FlipUp,
    #[serde(rename = "flip-down")]
    FlipDown,
    #[serde(rename = "flip-left")]
    FlipLeft,
    #[serde(rename = "flip-right")]
    FlipRight,
}

impl SalAnimation {
    pub fn is_animated(self) -> bool {
        !matches!(self, SalAnimation::NoAnimation)
    }
}

/// CSS honors `data-sal-duration` at 200–2000 ms in 50 ms steps.
pub fn is_valid_sal_duration(ms: u16) -> bool {
    (200..=2000).contains(&ms) && ms % 50 == 0
}

/// CSS honors `data-sal-delay` at 0–1000 ms in 50 ms steps.
pub fn is_valid_sal_delay(ms: u16) -> bool {
    ms <= 1000 && ms % 50 == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionClass {
    #[serde(rename = "-full-full")]
    FullFull,
    #[serde(rename = "-full-contained")]
    FullContained,
    #[serde(rename = "-full-xxl")]
    FullXxl,
    #[serde(rename = "-full-xl")]
    FullXl,
    #[serde(rename = "-full-lg")]
    FullLg,
    #[serde(rename = "-full-md")]
    FullMd,
    #[serde(rename = "-xxl")]
    Xxl,
    #[serde(rename = "-xl")]
    Xl,
    #[serde(rename = "-lg")]
    Lg,
    #[serde(rename = "-md")]
    Md,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionBg {
    #[serde(rename = "-bg-muted")]
    Muted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionPadding {
    #[serde(rename = "-no-padding")]
    NoPadding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionItemBoxClass {
    #[serde(rename = "no-box")]
    NoBox,
    #[serde(rename = "l-box")]
    LBox,
    #[serde(rename = "ll-box")]
    LlBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlternatingType {
    #[serde(rename = "-default")]
    Default,
    #[serde(rename = "-reverse")]
    Reverse,
    #[serde(rename = "-no-alternate")]
    NoAlternate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    #[serde(rename = "-default")]
    Default,
    #[serde(rename = "-horizontal")]
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardLinkTarget {
    #[serde(rename = "_self")]
    SelfTarget,
    #[serde(rename = "_blank")]
    Blank,
}

impl Default for CardLinkTarget {
    fn default() -> Self {
        Self::SelfTarget
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ButtonStyle {
    #[default]
    #[serde(rename = "-primary")]
    Primary,
    #[serde(rename = "-secondary")]
    Secondary,
    #[serde(rename = "-tertiary")]
    Tertiary,
    #[serde(rename = "-ghost")]
    Ghost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdLink {
    pub url: String,
    pub label: String,
    #[serde(default)]
    pub target: CardLinkTarget,
    #[serde(default)]
    pub style: ButtonStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Media {
    #[default]
    None,
    Image {
        url: String,
        #[serde(default)]
        alt: String,
    },
    Oembed {
        url: String,
    },
    LocalVideo {
        lg_mp4: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sm_mp4: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        poster: Option<String>,
        #[serde(default)]
        name: String,
        #[serde(default, rename = "loop")]
        loop_playback: bool,
        #[serde(default)]
        autoplay: bool,
    },
}

impl Media {
    pub fn is_none(&self) -> bool {
        matches!(self, Media::None)
    }

    pub fn from_legacy_image(url: &str, alt: &str) -> Self {
        let url = url.trim();
        if url.is_empty() {
            Media::None
        } else {
            Media::Image {
                url: url.to_string(),
                alt: alt.trim().to_string(),
            }
        }
    }

    pub fn kind_str(&self) -> &'static str {
        match self {
            Media::None => "none",
            Media::Image { .. } => "image",
            Media::Oembed { .. } => "oembed",
            Media::LocalVideo { .. } => "local-video",
        }
    }

    pub fn local_asset_urls(&self) -> Vec<String> {
        match self {
            Media::None | Media::Oembed { .. } => Vec::new(),
            Media::Image { url, .. } => vec![url.clone()],
            Media::LocalVideo {
                lg_mp4,
                sm_mp4,
                poster,
                ..
            } => {
                let mut v = vec![lg_mp4.clone()];
                if let Some(s) = sm_mp4 {
                    v.push(s.clone());
                }
                if let Some(p) = poster {
                    v.push(p.clone());
                }
                v
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OembedProvider {
    Youtube { id: String },
    Vimeo { id: String },
}

/// Parse a YouTube or Vimeo watch/share URL into a provider + id.
pub fn parse_oembed_url(raw: &str) -> Option<OembedProvider> {
    let url = raw.trim();
    if url.is_empty() {
        return None;
    }
    let lower = url.to_ascii_lowercase();
    if lower.contains("youtu") {
        youtube_id(url).map(|id| OembedProvider::Youtube { id })
    } else if lower.contains("vimeo.com") {
        vimeo_id(url).map(|id| OembedProvider::Vimeo { id })
    } else {
        None
    }
}

fn youtube_id(url: &str) -> Option<String> {
    // youtu.be/ID, youtube.com/watch?v=ID, /embed/ID, /shorts/ID
    let after = |needle: &str| {
        url.split(needle)
            .nth(1)
            .map(|rest| {
                rest.split(|c| c == '&' || c == '?' || c == '#' || c == '/')
                    .next()
                    .unwrap_or("")
            })
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };
    after("youtu.be/")
        .or_else(|| after("youtube.com/embed/"))
        .or_else(|| after("youtube-nocookie.com/embed/"))
        .or_else(|| after("youtube.com/shorts/"))
        .or_else(|| after("youtube.com/watch?v="))
        .or_else(|| after("youtube.com/watch?si=").and_then(|_| after("v=")))
        .or_else(|| after("v="))
}

fn vimeo_id(url: &str) -> Option<String> {
    let rest = url.split("vimeo.com/").nth(1)?;
    let id = rest
        .split(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())?;
    Some(id.to_string())
}

impl DdLink {
    pub fn from_parts(
        url: Option<&str>,
        label: Option<&str>,
        target: CardLinkTarget,
        style: ButtonStyle,
    ) -> Option<Self> {
        let url = url.map(str::trim).filter(|s| !s.is_empty())?.to_string();
        let label = label.map(str::trim).filter(|s| !s.is_empty())?.to_string();
        Some(Self {
            url,
            label,
            target,
            style,
        })
    }
}

pub fn cta_target_to_link_target(target: Option<CtaTarget>) -> CardLinkTarget {
    match target {
        Some(CtaTarget::Blank) => CardLinkTarget::Blank,
        Some(CtaTarget::SelfTarget | CtaTarget::Parent) | None => CardLinkTarget::SelfTarget,
    }
}

impl DdHero {
    pub fn resolved_links(&self) -> Vec<DdLink> {
        if !self.links.is_empty() {
            return self.links.clone();
        }
        [
            DdLink::from_parts(
                self.link_1_url.as_deref(),
                self.link_1_label.as_deref(),
                cta_target_to_link_target(self.link_1_target),
                ButtonStyle::Primary,
            ),
            DdLink::from_parts(
                self.link_2_url.as_deref(),
                self.link_2_label.as_deref(),
                cta_target_to_link_target(self.link_2_target),
                ButtonStyle::Ghost,
            ),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

impl DdHero {
    pub fn resolved_media(&self) -> Media {
        if !self.media.is_none() {
            return self.media.clone();
        }
        Media::from_legacy_image(
            &self.parent_image_url,
            self.parent_image_alt.as_deref().unwrap_or(""),
        )
    }
}

impl DdBanner {
    pub fn resolved_media(&self) -> Media {
        if !self.media.is_none() {
            return self.media.clone();
        }
        Media::from_legacy_image(&self.parent_image_url, &self.parent_image_alt)
    }
}

impl AlternatingItem {
    pub fn resolved_media(&self) -> Media {
        if !self.media.is_none() {
            return self.media.clone();
        }
        Media::from_legacy_image(&self.child_image_url, &self.child_image_alt)
    }
}

impl SliderItem {
    pub fn resolved_media(&self) -> Media {
        if !self.media.is_none() {
            return self.media.clone();
        }
        Media::from_legacy_image(&self.child_image_url, &self.child_image_alt)
    }
}

impl DdCta {
    pub fn resolved_links(&self) -> Vec<DdLink> {
        if !self.links.is_empty() {
            return self.links.clone();
        }
        DdLink::from_parts(
            self.parent_link_url.as_deref(),
            self.parent_link_label.as_deref(),
            self.parent_link_target.unwrap_or_default(),
            ButtonStyle::Primary,
        )
        .into_iter()
        .collect()
    }
}

impl SliderItem {
    pub fn resolved_links(&self) -> Vec<DdLink> {
        if !self.links.is_empty() {
            return self.links.clone();
        }
        DdLink::from_parts(
            self.child_link_url.as_deref(),
            self.child_link_label.as_deref(),
            self.child_link_target.unwrap_or_default(),
            ButtonStyle::Primary,
        )
        .into_iter()
        .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BannerClass {
    #[serde(rename = "-bg-top-left")]
    BgTopLeft,
    #[serde(rename = "-bg-top-center")]
    BgTopCenter,
    #[serde(rename = "-bg-top-right")]
    BgTopRight,
    #[serde(rename = "-bg-center-left")]
    BgCenterLeft,
    #[serde(rename = "-bg-center-center")]
    BgCenterCenter,
    #[serde(rename = "-bg-center-right")]
    BgCenterRight,
    #[serde(rename = "-bg-bottom-left")]
    BgBottomLeft,
    #[serde(rename = "-bg-bottom-center")]
    BgBottomCenter,
    #[serde(rename = "-bg-bottom-right")]
    BgBottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CtaClass {
    #[serde(rename = "-top-left")]
    TopLeft,
    #[serde(rename = "-top-center")]
    TopCenter,
    #[serde(rename = "-top-right")]
    TopRight,
    #[serde(rename = "-center-left")]
    CenterLeft,
    #[serde(rename = "-center-center")]
    CenterCenter,
    #[serde(rename = "-center-right")]
    CenterRight,
    #[serde(rename = "-bottom-left")]
    BottomLeft,
    #[serde(rename = "-bottom-center")]
    BottomCenter,
    #[serde(rename = "-bottom-right")]
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilmstripType {
    #[serde(rename = "-default")]
    Default,
    #[serde(rename = "-reverse")]
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccordionType {
    #[serde(rename = "-default")]
    Default,
    #[serde(rename = "-faq")]
    Faq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccordionClass {
    #[serde(rename = "-borderless")]
    Borderless,
    #[serde(rename = "-compact")]
    Compact,
    #[serde(rename = "-primary")]
    Primary,
    #[serde(rename = "-secondary")]
    Secondary,
    #[serde(rename = "-tertiary")]
    Tertiary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertType {
    #[serde(rename = "-default")]
    Default,
    #[serde(rename = "-info")]
    Info,
    #[serde(rename = "-warning")]
    Warning,
    #[serde(rename = "-error")]
    Error,
    #[serde(rename = "-success")]
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertClass {
    #[serde(rename = "-default")]
    Default,
    #[serde(rename = "-compact")]
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationKind {
    #[serde(rename = "link")]
    Link,
    #[serde(rename = "button")]
    Button,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationType {
    #[serde(rename = "dd-header__navigation")]
    HeaderNav,
    #[serde(rename = "dd-footer__navigation")]
    FooterNav,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationClass {
    #[serde(rename = "-main-menu")]
    MainMenu,
    #[serde(rename = "-menu-secondary")]
    MenuSecondary,
    #[serde(rename = "-menu-tertiary")]
    MenuTertiary,
    #[serde(rename = "-footer-menu")]
    FooterMenu,
    #[serde(rename = "-footer-menu-secondary")]
    FooterMenuSecondary,
    #[serde(rename = "-footer-menu-tertiary")]
    FooterMenuTertiary,
    #[serde(rename = "-social-menu")]
    SocialMenu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RobotsDirective {
    #[serde(rename = "index, follow")]
    IndexFollow,
    #[serde(rename = "noindex, follow")]
    NoindexFollow,
    #[serde(rename = "index, nofollow")]
    IndexNofollow,
    #[serde(rename = "noindex, nofollow")]
    NoindexNofollow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchemaType {
    WebPage,
    Article,
    AboutPage,
    ContactPage,
    CollectionPage,
    Organization,
    LocalBusiness,
    Product,
    Service,
}

impl Site {
    pub fn starter() -> Self {
        Self {
            schema_version: 1,
            id: "site-1".to_string(),
            name: "My Site".to_string(),
            theme: ThemeSettings {
                primary_color: "#88d9f7".to_string(),
                secondary_color: "#ffca76".to_string(),
                tertiary_color: "#f98971".to_string(),
                support_color: "#46be8c".to_string(),
            },
            header: DdHeader {
                id: "header".to_string(),
                custom_css: None,
                cta_label: Some("Contact".to_string()),
                cta_url: None,
                banner: None,
                alert: None,
                sections: vec![DdSection {
                    id: "header-section-1".to_string(),
                    section_title: None,
                    section_class: Some(SectionClass::FullContained),
                    item_box_class: Some(SectionItemBoxClass::LBox),
                    bg: None,
                    padding: None,
                    custom_css: None,
                    aria_label: None,
                    sal: SalAnimation::NoAnimation,
                    sal_duration: None,
                    sal_delay: None,
                    columns: vec![
                        SectionColumn {
                            id: "column-1".to_string(),
                            width_class: "dd-u-18-24 dd-u-md-18-24".to_string(),
                            components: Vec::new(),
                        },
                        SectionColumn {
                            id: "column-2".to_string(),
                            width_class: "dd-u-3-24 dd-u-sm-3-24 dd-u-md-3-24 dd-u-lg-4-24"
                                .to_string(),
                            components: vec![SectionComponent::HeaderSearch(DdHeaderSearch {
                                sal: SalAnimation::Fade,
                                sal_duration: None,
                                sal_delay: None,
                            })],
                        },
                        SectionColumn {
                            id: "column-3".to_string(),
                            width_class: "dd-u-3-24 dd-u-sm-3-24 dd-u-md-3-24".to_string(),
                            components: vec![SectionComponent::HeaderMenu(DdHeaderMenu {
                                sal: SalAnimation::Fade,
                                sal_duration: None,
                                sal_delay: None,
                            })],
                        },
                    ],
                }],
            },
            footer: DdFooter {
                id: "footer".to_string(),
                custom_css: None,
                blurb: None,
                copyright: None,
                social_linkedin: None,
                social_x: None,
                social_github: None,
                sections: vec![DdSection {
                    id: "footer-section-1".to_string(),
                    section_title: None,
                    section_class: Some(SectionClass::FullContained),
                    item_box_class: Some(SectionItemBoxClass::LBox),
                    bg: None,
                    padding: None,
                    custom_css: None,
                    aria_label: None,
                    sal: SalAnimation::NoAnimation,
                    sal_duration: None,
                    sal_delay: None,
                    columns: vec![SectionColumn {
                        id: "column-1".to_string(),
                        width_class: "dd-u-1-1".to_string(),
                        components: Vec::new(),
                    }],
                }],
            },
            export_dir: None,
            base_url: None,
            lang: default_lang(),
            header_gtm_tag: None,
            body_gtm_tag: None,
            pages: vec![Page {
                id: "page-home".to_string(),
                slug: "index".to_string(),
                slug_locked: false,
                head: DdHead {
                    title: "Home".to_string(),
                    meta_title: None,
                    meta_description: Some("Starter page".to_string()),
                    canonical_url: None,
                    robots: RobotsDirective::IndexFollow,
                    schema_type: SchemaType::WebPage,
                    og_title: None,
                    og_description: None,
                    og_image: None,
                },
                nodes: vec![
                    PageNode::Hero(DdHero {
                        parent_image_url: "assets/images/hero.jpg".to_string(),
                        parent_image_alt: Some("Decorative hero".to_string()),
                        parent_class: Some(HeroImageClass::FullFull),
                        sal: Some(SalAnimation::Fade),
                        sal_duration: None,
                        sal_delay: None,
                        parent_custom_css: None,
                        parent_title: "Build with dd-framework".to_string(),
                        parent_subtitle: "Framework-native static page builder".to_string(),
                        parent_copy: Some(
                            "Compose pages with typed component schemas.".to_string(),
                        ),
                        links: vec![
                            DdLink {
                                url: "#".to_string(),
                                label: "Get Started".to_string(),
                                target: CardLinkTarget::SelfTarget,
                                style: ButtonStyle::Primary,
                            },
                            DdLink {
                                url: "#".to_string(),
                                label: "Learn More".to_string(),
                                target: CardLinkTarget::SelfTarget,
                                style: ButtonStyle::Ghost,
                            },
                        ],
                        link_1_label: None,
                        link_1_url: None,
                        link_1_target: None,
                        link_2_label: None,
                        link_2_url: None,
                        link_2_target: None,
                        parent_image_mobile: None,
                        parent_image_tablet: None,
                        parent_image_desktop: None,
                        parent_image_class: Some(HeroImageClass::FullFull),
                        media: Media::None,
                        overlay: None,
                        copy_position: Some(HeroCopyPosition::Left),
                        id: None,
                        aria_label: Some("Introduction".to_string()),
                    }),
                    PageNode::Section(DdSection {
                        id: "section-1".to_string(),
                        section_title: Some("Ready to publish?".to_string()),
                        section_class: Some(SectionClass::FullContained),
                        item_box_class: Some(SectionItemBoxClass::LBox),
                        bg: None,
                        padding: None,
                        custom_css: None,
                        aria_label: None,
                        sal: SalAnimation::NoAnimation,
                        sal_duration: None,
                        sal_delay: None,
                        columns: vec![SectionColumn {
                            id: "column-1".to_string(),
                            width_class: "dd-u-1-1".to_string(),
                            components: Vec::new(),
                        }],
                    }),
                ],
            }],
        }
    }
}

/// Starter content to seed a new page with.
#[derive(Debug, Clone, Copy)]
pub enum PageTemplate {
    /// No nodes, just the `[HEAD]` metadata.
    Blank,
    /// One empty `dd-hero`.
    HeroOnly,
    /// `dd-hero` + `dd-section` with one empty column.
    HeroPlusSection,
}

impl Page {
    pub fn from_template(title: &str, template: PageTemplate) -> Self {
        let slug = slug_from_title(title);
        let id = format!("page-{}", slug);
        let nodes = match template {
            PageTemplate::Blank => Vec::new(),
            PageTemplate::HeroOnly => vec![PageNode::Hero(Self::empty_hero())],
            PageTemplate::HeroPlusSection => vec![
                PageNode::Hero(Self::empty_hero()),
                PageNode::Section(Self::empty_section()),
            ],
        };
        Page {
            id,
            slug,
            slug_locked: false,
            head: DdHead {
                title: title.to_string(),
                meta_title: None,
                meta_description: None,
                canonical_url: None,
                robots: RobotsDirective::IndexFollow,
                schema_type: SchemaType::WebPage,
                og_title: None,
                og_description: None,
                og_image: None,
            },
            nodes,
        }
    }

    pub fn duplicate_from(src: &Page) -> Self {
        let title = format!("{} (Copy)", src.head.title);
        let slug = slug_from_title(&title);
        let mut head = src.head.clone();
        head.title = title;
        Page {
            id: format!("page-{}", slug),
            slug,
            slug_locked: false,
            head,
            nodes: src.nodes.clone(),
        }
    }

    fn empty_hero() -> DdHero {
        DdHero {
            parent_image_url: String::new(),
            parent_image_alt: None,
            parent_class: Some(HeroImageClass::FullFull),
            sal: Some(SalAnimation::Fade),
            sal_duration: None,
            sal_delay: None,
            parent_custom_css: None,
            parent_title: String::new(),
            parent_subtitle: String::new(),
            parent_copy: None,
            links: Vec::new(),
            link_1_label: None,
            link_1_url: None,
            link_1_target: None,
            link_2_label: None,
            link_2_url: None,
            link_2_target: None,
            parent_image_mobile: None,
            parent_image_tablet: None,
            parent_image_desktop: None,
            parent_image_class: Some(HeroImageClass::FullFull),
            media: Media::None,
            overlay: None,
            copy_position: Some(HeroCopyPosition::Left),
            id: None,
            aria_label: Some("Introduction".to_string()),
        }
    }

    fn empty_section() -> DdSection {
        DdSection {
            id: "section-1".to_string(),
            section_title: None,
            section_class: Some(SectionClass::FullContained),
            item_box_class: Some(SectionItemBoxClass::LBox),
            bg: None,
            padding: None,
            custom_css: None,
            aria_label: None,
            sal: SalAnimation::NoAnimation,
            sal_duration: None,
            sal_delay: None,
            columns: vec![SectionColumn {
                id: "column-1".to_string(),
                width_class: "dd-u-1-1".to_string(),
                components: Vec::new(),
            }],
        }
    }
}

/// Convert a human title to a filesystem/URL-safe kebab-case slug.
/// ASCII-only, lowercase, alphanumerics and hyphens preserved,
/// whitespace collapsed to single `-`, everything else stripped.
/// Falls back to `"untitled"` for empty/whitespace-only inputs.
pub fn slug_from_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut prev_hyphen = false;
    for ch in title.chars() {
        let c = ch.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_hyphen = false;
        } else if c.is_whitespace() || c == '-' || c == '_' {
            if !prev_hyphen && !out.is_empty() {
                out.push('-');
                prev_hyphen = true;
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "untitled".to_string()
    } else {
        out
    }
}

/// HTML file name for a page slug. `index` → `index.html`.
pub fn page_file_name(slug: &str) -> String {
    if slug == "index" {
        "index.html".to_string()
    } else {
        format!("{}.html", slug)
    }
}

/// Same-directory href used in page picker output and in-page links.
pub fn page_href(slug: &str) -> String {
    page_file_name(slug)
}

/// True when `slug` is safe to join onto an export directory as `{slug}.html`.
pub fn is_safe_slug(slug: &str) -> bool {
    let s = slug.trim();
    !s.is_empty()
        && s != "."
        && s != ".."
        && !s.contains('/')
        && !s.contains('\\')
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// True when `year` is a machine-readable date label: YYYY, YYYY-MM, or YYYY-MM-DD.
/// Extract a `GTM-XXXX` container id from a pasted snippet. ASCII alphanumerics after the prefix.
pub fn extract_gtm_id(snippet: &str) -> Option<String> {
    let upper = snippet.to_ascii_uppercase();
    let bytes = upper.as_bytes();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if &bytes[i..i + 4] == b"GTM-" {
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_alphanumeric() {
                j += 1;
            }
            if j > i + 4 {
                return Some(upper[i..j].to_string());
            }
        }
        i += 1;
    }
    None
}

pub fn is_parseable_year_label(year: &str) -> bool {
    let s = year.trim();
    let b = s.as_bytes();
    match b.len() {
        4 => b.iter().all(u8::is_ascii_digit),
        7 => {
            b[4] == b'-'
                && b[..4].iter().all(u8::is_ascii_digit)
                && b[5..].iter().all(u8::is_ascii_digit)
        }
        10 => {
            b[4] == b'-'
                && b[7] == b'-'
                && b[..4].iter().all(u8::is_ascii_digit)
                && b[5..7].iter().all(u8::is_ascii_digit)
                && b[8..].iter().all(u8::is_ascii_digit)
        }
        _ => false,
    }
}

/// Join `base_url` (no trailing slash required) with a relative file path.
pub fn absolute_url(base_url: Option<&str>, rel: &str) -> Option<String> {
    let base = base_url.map(str::trim).filter(|s| !s.is_empty())?;
    let rel = rel.trim().trim_start_matches('/');
    Some(format!("{}/{}", base.trim_end_matches('/'), rel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_from_template_blank_has_no_nodes() {
        let p = Page::from_template("Contact Us", PageTemplate::Blank);
        assert_eq!(p.head.title, "Contact Us");
        assert_eq!(p.slug, "contact-us");
        assert!(!p.slug_locked);
        assert!(p.nodes.is_empty());
    }

    #[test]
    fn page_from_template_hero_only_has_one_hero_node() {
        let p = Page::from_template("Gallery", PageTemplate::HeroOnly);
        assert_eq!(p.nodes.len(), 1);
        assert!(matches!(p.nodes[0], PageNode::Hero(_)));
    }

    #[test]
    fn page_from_template_hero_plus_section_has_hero_then_section() {
        let p = Page::from_template("Services", PageTemplate::HeroPlusSection);
        assert_eq!(p.nodes.len(), 2);
        assert!(matches!(p.nodes[0], PageNode::Hero(_)));
        assert!(matches!(p.nodes[1], PageNode::Section(_)));
    }

    #[test]
    fn page_from_template_duplicate_deep_clones_and_appends_copy_suffix() {
        let mut starter = Site::starter();
        starter.pages[0].head.title = "Home".to_string();
        let dup = Page::duplicate_from(&starter.pages[0]);
        assert_eq!(dup.head.title, "Home (Copy)");
        assert_eq!(dup.slug, "home-copy");
        assert!(!dup.slug_locked);
        assert!(dup.head.meta_title.is_none());
        assert_eq!(dup.nodes.len(), starter.pages[0].nodes.len());
        assert_ne!(dup.id, starter.pages[0].id);
    }

    #[test]
    fn html_title_falls_back_to_page_label() {
        let mut head = DdHead {
            title: "Brands".to_string(),
            meta_title: None,
            meta_description: None,
            canonical_url: None,
            robots: RobotsDirective::IndexFollow,
            schema_type: SchemaType::WebPage,
            og_title: None,
            og_description: None,
            og_image: None,
        };
        assert_eq!(head.html_title(), "Brands");
        head.meta_title = Some("  ".to_string());
        assert_eq!(head.html_title(), "Brands");
        head.meta_title = Some("ldnddev | Brands".to_string());
        assert_eq!(head.html_title(), "ldnddev | Brands");
    }

    #[test]
    fn head_meta_title_missing_in_legacy_json() {
        let raw = r#"{
            "title": "Home",
            "meta_description": null,
            "canonical_url": null,
            "robots": "index, follow",
            "schema_type": "WebPage",
            "og_title": null,
            "og_description": null,
            "og_image": null
        }"#;
        let head: DdHead = serde_json::from_str(raw).expect("legacy head should load");
        assert_eq!(head.title, "Home");
        assert!(head.meta_title.is_none());
        assert_eq!(head.html_title(), "Home");
    }

    #[test]
    fn slug_from_title_basic_lowercase_and_hyphenate() {
        assert_eq!(slug_from_title("Contact Us"), "contact-us");
    }

    #[test]
    fn slug_from_title_strips_punctuation() {
        assert_eq!(slug_from_title("Hello, World!"), "hello-world");
    }

    #[test]
    fn slug_from_title_collapses_whitespace() {
        assert_eq!(slug_from_title("  a   b  "), "a-b");
    }

    #[test]
    fn slug_from_title_empty_fallback() {
        assert_eq!(slug_from_title(""), "untitled");
        assert_eq!(slug_from_title("   "), "untitled");
        assert_eq!(slug_from_title("!!!"), "untitled");
    }

    #[test]
    fn slug_from_title_preserves_existing_hyphens_without_duplicating() {
        assert_eq!(slug_from_title("already-hyphenated"), "already-hyphenated");
        assert_eq!(slug_from_title("about  -  us"), "about-us");
    }

    #[test]
    fn page_file_name_special_cases_index() {
        assert_eq!(page_file_name("index"), "index.html");
        assert_eq!(page_file_name("contact"), "contact.html");
    }

    #[test]
    fn is_safe_slug_rejects_path_traversal_and_separators() {
        assert!(is_safe_slug("index"));
        assert!(is_safe_slug("about-us"));
        assert!(!is_safe_slug(""));
        assert!(!is_safe_slug(".."));
        assert!(!is_safe_slug("../x"));
        assert!(!is_safe_slug("a/b"));
        assert!(!is_safe_slug("a\\b"));
        assert!(!is_safe_slug("hello world"));
    }

    #[test]
    fn absolute_url_joins_without_double_slash() {
        assert_eq!(
            absolute_url(Some("https://ex.com/"), "contact.html").as_deref(),
            Some("https://ex.com/contact.html")
        );
        assert!(absolute_url(None, "index.html").is_none());
        assert!(absolute_url(Some("  "), "index.html").is_none());
    }

    #[test]
    fn sal_field_loads_legacy_parent_data_aos_and_old_animation_names() {
        let banner: crate::model::DdBanner = serde_json::from_str(
            r#"{
                "parent_class": "-bg-center-center",
                "parent_data_aos": "fade-up",
                "parent_image_url": "/x.jpg",
                "parent_image_alt": "x"
            }"#,
        )
        .expect("legacy banner JSON should load");
        assert_eq!(banner.sal, SalAnimation::SlideUp);

        let zoom: crate::model::DdBanner = serde_json::from_str(
            r#"{
                "parent_class": "-bg-center-center",
                "parent_data_aos": "zoom-in-down",
                "parent_image_url": "/x.jpg",
                "parent_image_alt": "x"
            }"#,
        )
        .expect("legacy zoom-in-down should map to zoom-in");
        assert_eq!(zoom.sal, SalAnimation::ZoomIn);

        let fresh = serde_json::to_string(&banner).unwrap();
        assert!(fresh.contains("\"sal\""));
        assert!(!fresh.contains("parent_data_aos"));
        assert!(fresh.contains("slide-up"));
        assert!(!fresh.contains("fade-up"));
        assert!(!fresh.contains("sal_duration"));
        assert!(!fresh.contains("sal_delay"));
    }

    #[test]
    fn sal_no_animation_round_trips_and_duration_delay_skip_when_none() {
        let banner: crate::model::DdBanner = serde_json::from_str(
            r#"{
                "parent_class": "-bg-center-center",
                "sal": "no-animation",
                "parent_image_url": "/x.jpg",
                "parent_image_alt": "x"
            }"#,
        )
        .expect("no-animation banner JSON should load");
        assert_eq!(banner.sal, SalAnimation::NoAnimation);
        assert!(banner.sal_duration.is_none());
        assert!(banner.sal_delay.is_none());

        let with_timing: crate::model::DdBanner = serde_json::from_str(
            r#"{
                "parent_class": "-bg-center-center",
                "sal": "fade",
                "sal_duration": 600,
                "sal_delay": 150,
                "parent_image_url": "/x.jpg",
                "parent_image_alt": "x"
            }"#,
        )
        .expect("timed banner JSON should load");
        assert_eq!(with_timing.sal_duration, Some(600));
        assert_eq!(with_timing.sal_delay, Some(150));
        let timed_json = serde_json::to_string(&with_timing).unwrap();
        assert!(timed_json.contains("\"sal_duration\":600"));
        assert!(timed_json.contains("\"sal_delay\":150"));
    }

    #[test]
    fn sal_duration_delay_step_rules() {
        assert!(is_valid_sal_duration(200));
        assert!(is_valid_sal_duration(400));
        assert!(is_valid_sal_duration(2000));
        assert!(!is_valid_sal_duration(0));
        assert!(!is_valid_sal_duration(225));
        assert!(!is_valid_sal_duration(2050));
        assert!(is_valid_sal_delay(0));
        assert!(is_valid_sal_delay(50));
        assert!(is_valid_sal_delay(1000));
        assert!(!is_valid_sal_delay(25));
        assert!(!is_valid_sal_delay(1050));
    }

    #[test]
    fn image_loads_legacy_json_without_dark_url() {
        let image: crate::model::DdImage = serde_json::from_str(
            r#"{
                "sal": "fade",
                "parent_image_url": "/light.jpg",
                "parent_image_alt": "Alt"
            }"#,
        )
        .expect("legacy dd-image JSON should load");
        assert_eq!(image.parent_image_url, "/light.jpg");
        assert!(image.parent_image_url_dark.is_none());
        assert_eq!(image.parent_image_alt, "Alt");
    }

    #[test]
    fn alternating_item_loads_legacy_json_without_subtitle() {
        let item: crate::model::AlternatingItem = serde_json::from_str(
            r#"{
                "child_image_url": "/a.jpg",
                "child_image_alt": "a",
                "child_title": "Title",
                "child_copy": "Copy"
            }"#,
        )
        .expect("legacy alternating item JSON should load");
        assert_eq!(item.child_title, "Title");
        assert!(item.child_subtitle.is_empty());
        assert!(item.links.is_empty());
    }

    #[test]
    fn hero_resolved_links_from_legacy_slots() {
        let hero: DdHero = serde_json::from_str(
            r#"{
                "parent_image_url": "/h.jpg",
                "parent_title": "T",
                "parent_subtitle": "S",
                "link_1_label": "Go",
                "link_1_url": "/go",
                "link_1_target": "_self",
                "link_2_label": "More",
                "link_2_url": "/more",
                "link_2_target": "_parent"
            }"#,
        )
        .expect("legacy hero JSON should load");
        let links = hero.resolved_links();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].label, "Go");
        assert_eq!(links[0].style, ButtonStyle::Primary);
        assert_eq!(links[1].label, "More");
        assert_eq!(links[1].style, ButtonStyle::Ghost);
        assert_eq!(links[1].target, CardLinkTarget::SelfTarget);

        let fresh = serde_json::to_string(&Site::starter().pages[0].nodes[0]).unwrap();
        assert!(fresh.contains("\"links\""));
        assert!(!fresh.contains("link_1_label"));
    }

    #[test]
    fn cta_resolved_links_from_legacy_triple() {
        let cta: DdCta = serde_json::from_str(
            r#"{
                "parent_class": "-top-left",
                "parent_image_url": "/c.jpg",
                "parent_image_alt": "alt",
                "parent_title": "T",
                "parent_subtitle": "S",
                "parent_copy": "C",
                "parent_link_url": "/path",
                "parent_link_label": "Learn More",
                "parent_link_target": "_blank"
            }"#,
        )
        .expect("legacy cta JSON should load");
        let links = cta.resolved_links();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "/path");
        assert_eq!(links[0].style, ButtonStyle::Primary);
        assert_eq!(links[0].target, CardLinkTarget::Blank);
    }

    #[test]
    fn parse_oembed_youtube_and_vimeo() {
        match parse_oembed_url("https://www.youtube.com/watch?v=dQw4w9wgGcQ").unwrap() {
            OembedProvider::Youtube { id } => assert_eq!(id, "dQw4w9wgGcQ"),
            _ => panic!("expected youtube"),
        }
        match parse_oembed_url("https://youtu.be/dQw4w9wgGcQ").unwrap() {
            OembedProvider::Youtube { id } => assert_eq!(id, "dQw4w9wgGcQ"),
            _ => panic!("expected youtube"),
        }
        match parse_oembed_url("https://vimeo.com/123456789").unwrap() {
            OembedProvider::Vimeo { id } => assert_eq!(id, "123456789"),
            _ => panic!("expected vimeo"),
        }
        assert!(parse_oembed_url("https://example.com/watch").is_none());
    }

    #[test]
    fn section_legacy_json_defaults_visual_options() {
        let section: DdSection = serde_json::from_str(
            r#"{
                "id": "s1",
                "section_class": "-full-contained",
                "item_box_class": "l-box",
                "columns": []
            }"#,
        )
        .expect("legacy section JSON should load");
        assert!(section.bg.is_none());
        assert!(section.padding.is_none());
        assert!(section.custom_css.is_none());
        assert!(section.aria_label.is_none());
        assert_eq!(section.sal, SalAnimation::NoAnimation);
        let json = serde_json::to_string(&section).unwrap();
        assert!(!json.contains("sal_duration"));
        assert!(!json.contains("-bg-muted"));
    }

    #[test]
    fn hero_legacy_json_defaults_overlay_copy_position() {
        let hero: DdHero = serde_json::from_str(
            r#"{
                "parent_image_url": "/h.jpg",
                "parent_title": "T",
                "parent_subtitle": "S"
            }"#,
        )
        .expect("legacy hero JSON should load");
        assert!(hero.overlay.is_none());
        assert!(hero.copy_position.is_none());
        assert!(hero.id.is_none());
        assert!(hero.aria_label.is_none());
        let json = serde_json::to_string(&hero).unwrap();
        assert!(!json.contains("overlay"));
        assert!(!json.contains("copy_position"));
        assert!(!json.contains("aria_label"));
    }

    #[test]
    fn site_legacy_json_defaults_options() {
        let site: Site = serde_json::from_str(
            r##"{
                "schema_version": 1,
                "id": "s",
                "name": "N",
                "theme": {
                    "primary_color": "#000000",
                    "secondary_color": "#000000",
                    "tertiary_color": "#000000",
                    "support_color": "#000000"
                },
                "header": { "id": "h", "sections": [] },
                "footer": { "id": "f", "sections": [] },
                "pages": []
            }"##,
        )
        .expect("legacy site JSON should load");
        assert!(site.header_gtm_tag.is_none());
        assert!(site.body_gtm_tag.is_none());
        assert!(site.header.cta_url.is_none());
        assert!(site.header.banner.is_none());
        assert!(site.footer.blurb.is_none());
        assert!(site.footer.social_github.is_none());
    }

    #[test]
    fn extract_gtm_id_from_snippet() {
        assert_eq!(
            extract_gtm_id("https://www.googletagmanager.com/gtm.js?id=GTM-ABC123"),
            Some("GTM-ABC123".to_string())
        );
        assert_eq!(
            extract_gtm_id(
                "<noscript><iframe src=\"https://www.googletagmanager.com/ns.html?id=gtm-xyz9\"></iframe></noscript>"
            ),
            Some("GTM-XYZ9".to_string())
        );
        assert!(extract_gtm_id("not a tag").is_none());
        assert!(extract_gtm_id("GTM-").is_none());
        assert!(extract_gtm_id("").is_none());
    }

    #[test]
    fn parseable_year_label_accepts_iso_dates() {
        assert!(is_parseable_year_label("2024"));
        assert!(is_parseable_year_label("2024-07"));
        assert!(is_parseable_year_label("2024-07-01"));
        assert!(!is_parseable_year_label("Q3 2024"));
        assert!(!is_parseable_year_label("March 2024"));
        assert!(!is_parseable_year_label(""));
    }

    #[test]
    fn media_from_legacy_image_empty_is_none() {
        assert!(Media::from_legacy_image("", "alt").is_none());
        match Media::from_legacy_image("/x.jpg", "x") {
            Media::Image { url, alt } => {
                assert_eq!(url, "/x.jpg");
                assert_eq!(alt, "x");
            }
            other => panic!("expected image, got {other:?}"),
        }
    }
}
