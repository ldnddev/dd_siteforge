//! Insert-picker component kinds.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ComponentKind {
    Hero,
    Section,
    Banner,
    Cta,
    Blockquote,
    Accordion,
    Alternating,
    Card,
    Filmstrip,
    Milestones,
    Modal,
    Slider,
    Alert,
    Image,
    RichText,
    Navigation,
    HeaderSearch,
    HeaderMenu,
    Spacer,
    Tabs,
    Timeline,
}

impl ComponentKind {
    pub(super) fn all() -> &'static [Self] {
        &[
            Self::Hero,
            Self::Section,
            Self::Cta,
            Self::Banner,
            Self::Blockquote,
            Self::Accordion,
            Self::Alternating,
            Self::Card,
            Self::Filmstrip,
            Self::Milestones,
            Self::Modal,
            Self::Slider,
            Self::Alert,
            Self::Image,
            Self::RichText,
            Self::Navigation,
            Self::HeaderSearch,
            Self::HeaderMenu,
            Self::Spacer,
            Self::Tabs,
            Self::Timeline,
        ]
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            ComponentKind::Hero => "dd-hero",
            ComponentKind::Section => "dd-section",
            ComponentKind::Cta => "dd-cta",
            ComponentKind::Banner => "dd-banner",
            ComponentKind::Blockquote => "dd-blockquote",
            ComponentKind::Accordion => "dd-accordion",
            ComponentKind::Alternating => "dd-alternating",
            ComponentKind::Card => "dd-card",
            ComponentKind::Filmstrip => "dd-filmstrip",
            ComponentKind::Milestones => "dd-milestones",
            ComponentKind::Modal => "dd-modal",
            ComponentKind::Slider => "dd-slider",
            ComponentKind::Alert => "dd-alert",
            ComponentKind::Image => "dd-image",
            ComponentKind::RichText => "dd-rich_text",
            ComponentKind::Navigation => "dd-navigation",
            ComponentKind::HeaderSearch => "dd-header-search",
            ComponentKind::HeaderMenu => "dd-header-menu",
            ComponentKind::Spacer => "dd-spacer",
            ComponentKind::Tabs => "dd-tabs",
            ComponentKind::Timeline => "dd-timeline",
        }
    }

    pub(super) fn from_section_component(c: &crate::model::SectionComponent) -> Self {
        match c {
            crate::model::SectionComponent::Cta(_) => Self::Cta,
            crate::model::SectionComponent::Banner(_) => Self::Banner,
            crate::model::SectionComponent::Blockquote(_) => Self::Blockquote,
            crate::model::SectionComponent::Accordion(_) => Self::Accordion,
            crate::model::SectionComponent::Alternating(_) => Self::Alternating,
            crate::model::SectionComponent::Card(_) => Self::Card,
            crate::model::SectionComponent::Filmstrip(_) => Self::Filmstrip,
            crate::model::SectionComponent::Milestones(_) => Self::Milestones,
            crate::model::SectionComponent::Modal(_) => Self::Modal,
            crate::model::SectionComponent::Slider(_) => Self::Slider,
            crate::model::SectionComponent::Alert(_) => Self::Alert,
            crate::model::SectionComponent::Image(_) => Self::Image,
            crate::model::SectionComponent::RichText(_) => Self::RichText,
            crate::model::SectionComponent::Navigation(_) => Self::Navigation,
            crate::model::SectionComponent::HeaderSearch(_) => Self::HeaderSearch,
            crate::model::SectionComponent::HeaderMenu(_) => Self::HeaderMenu,
            crate::model::SectionComponent::Spacer(_) => Self::Spacer,
            crate::model::SectionComponent::Tabs(_) => Self::Tabs,
            crate::model::SectionComponent::Timeline(_) => Self::Timeline,
        }
    }

    pub(super) fn default_component(self) -> crate::model::SectionComponent {
        match self {
            ComponentKind::Hero | ComponentKind::Section => {
                unreachable!("top-level kinds do not map to section components")
            }
            ComponentKind::Cta => crate::model::SectionComponent::Cta(crate::model::DdCta {
                parent_class: crate::model::CtaClass::TopLeft,
                parent_image_url: "https://dummyimage.com/1920x1080/000000/fff".to_string(),
                parent_image_alt: "Image alt".to_string(),
                sal: crate::model::SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                parent_title: "Title".to_string(),
                parent_subtitle: "Subtitle".to_string(),
                parent_copy: "Copy".to_string(),
                links: vec![crate::model::DdLink {
                    url: "/path".to_string(),
                    label: "Learn More".to_string(),
                    target: crate::model::CardLinkTarget::SelfTarget,
                    style: crate::model::ButtonStyle::Primary,
                }],
                parent_link_url: None,
                parent_link_target: None,
                parent_link_label: None,
            }),
            ComponentKind::Banner => {
                crate::model::SectionComponent::Banner(crate::model::DdBanner {
                    parent_class: crate::model::BannerClass::BgCenterCenter,
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_image_url: "https://dummyimage.com/1920x1080/000/fff".to_string(),
                    parent_image_alt: "Banner alt text".to_string(),
                    media: crate::model::Media::None,
                })
            }
            ComponentKind::Blockquote => {
                crate::model::SectionComponent::Blockquote(crate::model::DdBlockquote {
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_image_url: "https://dummyimage.com/512x512/000/fff".to_string(),
                    parent_image_alt: "blockquote Persons Name".to_string(),
                    parent_name: "blockquote Persons Name".to_string(),
                    parent_role: "blockquote Persons Title".to_string(),
                    parent_copy: "blockquote content".to_string(),
                })
            }
            ComponentKind::Accordion => {
                crate::model::SectionComponent::Accordion(crate::model::DdAccordion {
                    parent_type: crate::model::AccordionType::Default,
                    parent_class: crate::model::AccordionClass::Primary,
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_group_name: "group1".to_string(),
                    items: vec![crate::model::AccordionItem {
                        child_title: "Accordion Item".to_string(),
                        child_copy: "Accordion content".to_string(),
                    }],
                    multiple: Some(false),
                })
            }
            ComponentKind::Alternating => {
                crate::model::SectionComponent::Alternating(crate::model::DdAlternating {
                    parent_type: crate::model::AlternatingType::Default,
                    parent_class: "-default".to_string(),
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![crate::model::AlternatingItem {
                        child_image_url: "https://dummyimage.com/600x400/000/fff".to_string(),
                        child_image_alt: "Alternating image".to_string(),
                        child_title: "Alternating Item".to_string(),
                        child_subtitle: "Subtitle".to_string(),
                        child_copy: "Alternating content".to_string(),
                        links: Vec::new(),
                        media: crate::model::Media::None,
                    }],
                })
            }
            ComponentKind::Card => crate::model::SectionComponent::Card(crate::model::DdCard {
                parent_type: crate::model::CardType::Default,
                sal: crate::model::SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                parent_width: "dd-u-1-1 dd-u-md-12-24 dd-u-lg-8-24".to_string(),
                items: vec![crate::model::CardItem {
                    child_image_url: "https://dummyimage.com/720x720/000/fff".to_string(),
                    child_image_alt: "Image alt text".to_string(),
                    child_title: "Title".to_string(),
                    child_subtitle: "Subtitle".to_string(),
                    child_copy: "Copy".to_string(),
                    child_link_url: Some("/front".to_string()),
                    child_link_target: Some(crate::model::CardLinkTarget::SelfTarget),
                    child_link_label: Some("Learn More".to_string()),
                    child_link_style: crate::model::ButtonStyle::Primary,
                }],
            }),
            ComponentKind::Filmstrip => {
                crate::model::SectionComponent::Filmstrip(crate::model::DdFilmstrip {
                    parent_type: crate::model::FilmstripType::Default,
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![crate::model::FilmstripItem {
                        child_image_url: "https://dummyimage.com/256x256/000/fff".to_string(),
                        child_image_alt: "Image alt text".to_string(),
                        child_title: "Title".to_string(),
                    }],
                })
            }
            ComponentKind::Milestones => {
                crate::model::SectionComponent::Milestones(crate::model::DdMilestones {
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_width: "dd-u-1-1 dd-u-md-12-24".to_string(),
                    items: vec![crate::model::MilestonesItem {
                        child_percentage: "70".to_string(),
                        child_title: "Title".to_string(),
                        child_subtitle: "Subtitle".to_string(),
                        child_copy: "Copy".to_string(),
                        child_link_url: None,
                        child_link_target: Some(crate::model::CardLinkTarget::SelfTarget),
                        child_link_label: None,
                        child_link_style: crate::model::ButtonStyle::Primary,
                    }],
                })
            }
            ComponentKind::Modal => crate::model::SectionComponent::Modal(crate::model::DdModal {
                parent_title: "Title".to_string(),
                parent_copy: "Copy".to_string(),
            }),
            ComponentKind::Slider => {
                crate::model::SectionComponent::Slider(crate::model::DdSlider {
                    parent_title: String::new(),
                    items: vec![crate::model::SliderItem {
                        child_title: "Title".to_string(),
                        child_copy: "Copy".to_string(),
                        links: vec![crate::model::DdLink {
                            url: "/path".to_string(),
                            label: "Learn More".to_string(),
                            target: crate::model::CardLinkTarget::SelfTarget,
                            style: crate::model::ButtonStyle::Primary,
                        }],
                        child_link_url: None,
                        child_link_target: None,
                        child_link_label: None,
                        child_image_url: "https://dummyimage.com/720x720/000/fff".to_string(),
                        child_image_alt: "Image alt text".to_string(),
                        media: crate::model::Media::None,
                    }],
                })
            }
            ComponentKind::Alert => crate::model::SectionComponent::Alert(crate::model::DdAlert {
                parent_type: crate::model::AlertType::Default,
                parent_class: crate::model::AlertClass::Default,
                sal: crate::model::SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                parent_title: Some("Alert Title".to_string()),
                parent_copy: "Alert content".to_string(),
            }),
            ComponentKind::Image => crate::model::SectionComponent::Image(crate::model::DdImage {
                sal: crate::model::SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                parent_image_url: "https://dummyimage.com/1200x600/000/fff".to_string(),
                parent_image_url_dark: None,
                parent_image_alt: "Image alt text".to_string(),
                parent_link_url: None,
                parent_link_target: None,
            }),
            ComponentKind::RichText => {
                crate::model::SectionComponent::RichText(crate::model::DdRichText {
                    parent_class: None,
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    parent_copy: "Copy".to_string(),
                })
            }
            ComponentKind::Navigation => {
                crate::model::SectionComponent::Navigation(crate::model::DdNavigation {
                    parent_type: crate::model::NavigationType::HeaderNav,
                    parent_class: crate::model::NavigationClass::MainMenu,
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![crate::model::NavigationItem {
                        child_kind: crate::model::NavigationKind::Link,
                        child_link_label: "Home".to_string(),
                        child_link_url: Some("/".to_string()),
                        child_link_target: Some(crate::model::CardLinkTarget::SelfTarget),
                        child_link_css: None,
                        items: Vec::new(),
                    }],
                })
            }
            ComponentKind::HeaderSearch => {
                crate::model::SectionComponent::HeaderSearch(crate::model::DdHeaderSearch {
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                })
            }
            ComponentKind::HeaderMenu => {
                crate::model::SectionComponent::HeaderMenu(crate::model::DdHeaderMenu {
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                })
            }
            ComponentKind::Spacer => {
                crate::model::SectionComponent::Spacer(crate::model::DdSpacer {
                    size: crate::model::SpacerSize::Md,
                    divider: false,
                })
            }
            ComponentKind::Tabs => crate::model::SectionComponent::Tabs(crate::model::DdTabs {
                parent_id: "tabs".to_string(),
                parent_class: crate::model::TabsOrientation::Horizontal,
                aria_label: Some("Content tabs".to_string()),
                sal: crate::model::SalAnimation::Fade,
                sal_duration: None,
                sal_delay: None,
                items: vec![crate::model::TabsItem {
                    child_title: "Tab 1".to_string(),
                    child_copy: "Panel copy".to_string(),
                }],
            }),
            ComponentKind::Timeline => {
                crate::model::SectionComponent::Timeline(crate::model::DdTimeline {
                    aria_label: Some("Timeline".to_string()),
                    sal: crate::model::SalAnimation::Fade,
                    sal_duration: None,
                    sal_delay: None,
                    items: vec![crate::model::TimelineItem {
                        child_year: "2024".to_string(),
                        child_datetime: None,
                        child_title: "Title".to_string(),
                        heading_level: 3,
                        child_copy: "Copy".to_string(),
                        child_image_url: None,
                        child_image_alt: None,
                    }],
                })
            }
        }
    }
}
