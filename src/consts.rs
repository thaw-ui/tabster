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
