use std::{cell::Cell, cell::RefCell, rc::Rc};

use gtk::{prelude::*, Box as GtkBox, Orientation, Overflow};
use webkit6::WebContext;

use crate::linux_terminal::settings::Settings;

use super::build_web_pane;

#[derive(Clone)]
pub(in crate::linux_terminal) struct WebPaneHost {
    root: GtkBox,
    // Rc<Cell<bool>> is enough here because lazy-load state is a small main-thread boolean shared across callbacks.
    loaded: Rc<Cell<bool>>,
    settings: Rc<RefCell<Settings>>,
    context: WebContext,
}

impl WebPaneHost {
    pub(in crate::linux_terminal) fn new(
        settings: Rc<RefCell<Settings>>,
        context: WebContext,
    ) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_hexpand(true);
        root.set_vexpand(true);
        root.set_width_request(0);
        root.set_overflow(Overflow::Hidden);
        Self {
            root,
            loaded: Rc::new(Cell::new(false)),
            settings,
            context,
        }
    }

    pub(in crate::linux_terminal) fn widget(&self) -> &GtkBox {
        &self.root
    }

    pub(in crate::linux_terminal) fn ensure_loaded(&self) {
        if self.loaded.replace(true) {
            return;
        }

        // WebContext clone keeps one shared WebKit context handle across any lazily-created browser widgets.
        let pane = build_web_pane(self.settings.clone(), self.context.clone());
        self.root.append(&pane);
    }
}
