//! Keyboard focus rings from design spec 01 §4.
//!
//! Outer rings sit 1 px outside buttons, inputs, search, selects, and toggles.
//! Inset rings stay inside tree rows, tabs, menu items, palette rows, and cards
//! so adjacent rows do not overlap. Pointer focus does not draw a ring.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Buttons, icon buttons, inputs, search, select, toggle, checkbox.
    Outer,
    /// Tree rows, tabs, menu items, palette rows, block cards.
    Inset,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    pub visible: bool,
    pub width_px: f32,
    pub offset_px: f32,
    pub color_token: &'static str,
}

pub fn ring(place: Place, from_keyboard: bool) -> Ring {
    Ring {
        visible: from_keyboard,
        width_px: 2.0,
        offset_px: match place {
            Place::Outer => 1.0,
            Place::Inset => 0.0,
        },
        color_token: "focus.ring",
    }
}

pub fn place_for(surface: &str) -> Place {
    match surface {
        "tree" | "tab" | "menu" | "palette" | "find" | "block" => Place::Inset,
        _ => Place::Outer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_focus_draws_the_ring_and_pointer_focus_does_not() {
        let outer = ring(Place::Outer, true);
        assert!(outer.visible);
        assert_eq!(outer.width_px, 2.0);
        assert_eq!(outer.offset_px, 1.0);
        assert_eq!(outer.color_token, "focus.ring");
        let inset = ring(Place::Inset, true);
        assert_eq!(inset.offset_px, 0.0);
        assert!(!ring(Place::Outer, false).visible);
        assert!(!ring(Place::Inset, false).visible);
        assert_eq!(place_for("palette"), Place::Inset);
        assert_eq!(place_for("tree"), Place::Inset);
        assert_eq!(place_for("input"), Place::Outer);
        assert_eq!(place_for("button"), Place::Outer);
    }
}
