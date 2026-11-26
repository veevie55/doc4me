//! Offline command help for people moving from WordPerfect. It indexes Doc4Me's actual command
//! registry, current keyboard assignments and menu locations; it does not ship WordPerfect help
//! content or imply that Doc4Me implements every WordPerfect behavior.

use egui::Ui;
use serde::{Deserialize, Serialize};

use crate::WordApp;

/// What the offline command index shows. These are saved with the other UI preferences.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct HelpOptions {
    /// Include commands without a currently assigned shortcut.
    pub all_commands: bool,
    /// Show the user's current shortcut for each command.
    pub shortcuts: bool,
    /// Show the ribbon or menu location for each command.
    pub locations: bool,
}

impl Default for HelpOptions {
    fn default() -> Self {
        Self { all_commands: true, shortcuts: true, locations: true }
    }
}

/// The two related dialogs share one state type so the help list can open its options without
/// losing the search text.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum WordPerfectHelpDialog {
    Help { query: String },
    Options { query: String, form: HelpOptions, back_to_help: bool },
}

impl WordPerfectHelpDialog {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Help { .. } => "wordPerfectHelp",
            Self::Options { .. } => "wordPerfectHelpOptions",
        }
    }

    pub(crate) fn title(&self) -> &'static str {
        match self {
            Self::Help { .. } => "Help",
            Self::Options { .. } => "Options",
        }
    }
}

/// Open from an engine request or `ui.dialog`.
pub(crate) fn open(name: &str, app: &WordApp) -> Option<WordPerfectHelpDialog> {
    Some(match name {
        "wordPerfectHelp" => WordPerfectHelpDialog::Help { query: String::new() },
        "wordPerfectHelpOptions" => WordPerfectHelpDialog::Options { query: String::new(), form: app.ui.wp_help.clone(), back_to_help: false },
        _ => return None,
    })
}

/// Render one of the help dialogs. `true` closes it.
pub(crate) fn body(app: &mut WordApp, ui: &mut Ui, dialog: &mut WordPerfectHelpDialog) -> bool {
    let mut next = None;
    let close = match dialog {
        WordPerfectHelpDialog::Help { query } => {
            let (close, open_options) = help_body(app, ui, query);
            if open_options {
                next = Some(WordPerfectHelpDialog::Options { query: query.clone(), form: app.ui.wp_help.clone(), back_to_help: true });
            }
            close
        }
        WordPerfectHelpDialog::Options { query, form, back_to_help } => {
            let (close, next_state) = options_body(app, ui, query, form, *back_to_help);
            next = next_state;
            close
        }
    };
    if let Some(next) = next {
        *dialog = next;
        false
    } else {
        close
    }
}

fn help_body(app: &mut WordApp, ui: &mut Ui, query: &mut String) -> (bool, bool) {
    let mut close = false;
    let mut open_options = false;
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(query).hint_text(crate::tl!("Search commands and help")).desired_width(f32::INFINITY));
        if ui.button(crate::tl!("Options…")).clicked() {
            open_options = true;
        }
    });

    let rows = command_rows(app, query, &app.ui.wp_help);
    ui.add_space(6.0);
    ui.label(crate::tl!("Commands:"));
    egui::ScrollArea::vertical().id_salt("wordperfect_help_commands").max_height(330.0).auto_shrink([false, false]).show_rows(
        ui,
        35.0,
        rows.len(),
        |ui, range| {
            for index in range {
                let Some(row) = rows.get(index) else { continue };
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(crate::tl!(row.label)).strong());
                        if app.ui.wp_help.shortcuts && !row.keys.is_empty() {
                            ui.label(egui::RichText::new(row.keys.join(" / ")).small().weak());
                        }
                    });
                    if app.ui.wp_help.locations {
                        ui.label(egui::RichText::new(crate::i18n::location(row.location)).small().weak());
                    }
                });
            }
        },
    );

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            close = ui.button(crate::tl!("Close")).clicked();
        });
    });
    (close || ui.input(|i| i.key_pressed(egui::Key::Escape)), open_options)
}

fn options_body(app: &mut WordApp, ui: &mut Ui, query: &str, form: &mut HelpOptions, back_to_help: bool) -> (bool, Option<WordPerfectHelpDialog>) {
    ui.checkbox(&mut form.all_commands, crate::tl!("All Commands"));
    ui.checkbox(&mut form.shortcuts, crate::tl!("Keyboard shortcuts:"));
    ui.checkbox(&mut form.locations, crate::tl!("Location"));

    let (apply, cancel) = crate::dialogs::buttons(ui, "Apply");
    if apply {
        app.ui.wp_help = form.clone();
        if back_to_help {
            return (false, Some(WordPerfectHelpDialog::Help { query: query.to_string() }));
        } else {
            return (true, None);
        }
    } else if cancel {
        if back_to_help {
            return (false, Some(WordPerfectHelpDialog::Help { query: query.to_string() }));
        } else {
            return (true, None);
        }
    }
    (false, None)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CommandRow {
    id: &'static str,
    label: &'static str,
    location: &'static str,
    keys: Vec<String>,
    params: &'static str,
}

impl CommandRow {
    fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        query.is_empty()
            || self.id.to_lowercase().contains(&query)
            || self.label.to_lowercase().contains(&query)
            || self.location.to_lowercase().contains(&query)
            || self.keys.iter().any(|k| k.to_lowercase().contains(&query))
            || self.params.to_lowercase().contains(&query)
    }
}

fn command_rows(app: &WordApp, query: &str, options: &HelpOptions) -> Vec<CommandRow> {
    let registry = &app.session.registry;
    let mut rows: Vec<CommandRow> = registry
        .all()
        .iter()
        .map(|spec| CommandRow {
            id: spec.id,
            label: spec.label,
            location: spec.location,
            keys: app.session.keymap.keys_for(registry, spec.id),
            params: spec.params,
        })
        .filter(|row| options.all_commands || !row.keys.is_empty())
        .filter(|row| row.matches(query))
        .collect();
    rows.sort_by(|a, b| a.label.cmp(b.label).then_with(|| a.id.cmp(b.id)));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &'static str, label: &'static str, location: &'static str, keys: Vec<String>, params: &'static str) -> CommandRow {
        CommandRow { id, label, location, keys, params }
    }

    #[test]
    fn command_search_checks_the_label_location_shortcut_and_parameters() {
        let r = row("view.marks", "Show/Hide ¶", "Home › Paragraph", vec!["Mod+Shift+8".into()], "toggles paragraph marks");
        for query in ["show/hide", "paragraph", "shift+8", "toggles"] {
            assert!(r.matches(query), "query {query:?} should find the command");
        }
        assert!(!r.matches("nonexistent"));
        assert!(r.matches("  "));
    }

    #[test]
    fn help_options_are_backward_compatible_and_round_trip() {
        let defaults: HelpOptions = serde_json::from_str("{}").unwrap();
        assert_eq!(defaults, HelpOptions::default());

        let options = HelpOptions { all_commands: false, shortcuts: false, locations: true };
        let saved = serde_json::to_string(&options).unwrap();
        let restored: HelpOptions = serde_json::from_str(&saved).unwrap();
        assert_eq!(restored, options);
    }

    #[test]
    fn command_index_uses_registered_commands_and_the_current_keyboard_map() {
        let mut app = WordApp::new(doc4me_engine::Session::new(doc4me_doc::Document::new()), crate::Services::default());
        let options = HelpOptions::default();
        let rows = command_rows(&app, "show/hide", &options);
        assert!(rows.iter().any(|row| row.id == "view.marks"));

        app.session.keymap.assign(&app.session.registry, "view.marks", "Mod+Shift+7").unwrap();
        let rows = command_rows(&app, "mod+shift+7", &options);
        assert!(rows.iter().any(|row| row.id == "view.marks" && row.keys.contains(&"Mod+Shift+7".to_string())));
    }

    #[test]
    fn command_index_options_filter_shortcutless_commands() {
        let app = WordApp::new(doc4me_engine::Session::new(doc4me_doc::Document::new()), crate::Services::default());
        let all = command_rows(&app, "", &HelpOptions::default());
        let with_shortcuts = command_rows(&app, "", &HelpOptions { all_commands: false, ..HelpOptions::default() });
        assert!(!all.is_empty());
        assert!(!with_shortcuts.is_empty());
        assert!(with_shortcuts.len() <= all.len());
        assert!(with_shortcuts.iter().all(|row| !row.keys.is_empty()));
    }
}
