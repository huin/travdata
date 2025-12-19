use egui::{Key, KeyboardShortcut, Modifiers};

pub struct Shortcuts {
    pub open: Shortcut,
    pub save: Shortcut,
}

impl Shortcuts {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            open: Shortcut::new(ctx, KeyboardShortcut::new(Modifiers::CTRL, Key::O)),
            save: Shortcut::new(ctx, KeyboardShortcut::new(Modifiers::CTRL, Key::S)),
        }
    }
}

pub struct Shortcut {
    pub formatted: String,
    pub shortcut: KeyboardShortcut,
}

impl Shortcut {
    fn new(ctx: &egui::Context, shortcut: KeyboardShortcut) -> Self {
        Self {
            formatted: ctx.format_shortcut(&shortcut),
            shortcut,
        }
    }

    pub fn consume(&self, ui: &mut egui::Ui) -> bool {
        ui.input_mut(|i| i.consume_shortcut(&self.shortcut))
    }
}
