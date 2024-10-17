use crate::focusable::FocusableAPI;
use web_sys::Window;

pub fn create_tabster(win: Window, props: TabsterCoreProps) {
    TabsterCore::new(win, props);
}

struct TabsterCoreProps {}

struct TabsterCore {
    focusable: FocusableAPI,
}

impl TabsterCore {
    fn new(win: Window, props: TabsterCoreProps) -> Self {
        let focusable = FocusableAPI::new();

        Self { focusable }
    }
}
