use std::ops::Deref;

pub const FOCUSABLE_SELECTOR: &'static str = "a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), *[tabindex], *[contenteditable], details > summary, audio[controls], video[controls]";

pub const TABSTER_ATTRIBUTE_NAME: &'static str = "data-tabster";
pub const TABSTER_DUMMY_INPUT_ATTRIBUTE_NAME: &'static str = "data-tabster-dummy";

pub mod mover_directions {
    pub const BOTH: u8 = 0; // Default, both left/up keys move to the previous, right/down move to the next.
    pub const VERTICAL: u8 = 1; // Only up/down arrows move to the next/previous.
    pub const HORIZONTAL: u8 = 2; // Only left/right arrows move to the next/previous.
    pub const GRID: u8 = 3; // Two-dimentional movement depending on the visual placement.
    pub const GRID_LINEAR: u8 = 4; // Two-dimentional movement depending on the visual placement. Allows linear movement.
}

pub mod visibilities {
    pub const INVISIBLE: u8 = 0;
    pub const PARTIALLY_VISIBLE: u8 = 1;
    pub const VISIBLE: u8 = 2;
}

pub enum GroupperTabbabilities {
    Unlimited,
    // The tabbability is limited to the container and explicit Enter is needed to go inside.
    Limited,
    // The focus is limited as above, plus trapped when inside.
    LimitedTrapFocus,
}

impl Deref for GroupperTabbabilities {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Unlimited => &0,
            Self::Limited => &1,
            Self::LimitedTrapFocus => &2,
        }
    }
}

pub enum SysDummyInputsPositions {
    // Tabster will place dummy inputs depending on the container tag name and on the default behaviour.
    Auto,
    // Tabster will always place dummy inputs inside the container.
    Inside,
    // Tabster will always place dummy inputs outside of the container.
    Outside,
}

impl Deref for SysDummyInputsPositions {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Auto => &0,
            Self::Inside => &1,
            Self::Outside => &2,
        }
    }
}
