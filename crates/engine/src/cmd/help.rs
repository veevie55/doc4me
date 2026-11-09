//! Help surfaces that can be opened by the front end. The compatibility help is an offline
//! command and shortcut index; it does not load WordPerfect's proprietary help content.

use serde_json::json;

use crate::CommandSpec;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("help.about", "About", "Help", |s, _| {
            s.push_ui_request(json!({"open": "about"}));
            Ok(json!({"opened": "about"}))
        })
        .pure(),
        CommandSpec::new("help.wordPerfect", "Search commands and help", "Help", |s, _| {
            s.push_ui_request(json!({"open": "wordPerfectHelp"}));
            Ok(json!({"opened": "wordPerfectHelp"}))
        })
        .pure(),
        CommandSpec::new("help.wordPerfectOptions", "Search Options", "Help", |s, _| {
            s.push_ui_request(json!({"open": "wordPerfectHelpOptions"}));
            Ok(json!({"opened": "wordPerfectHelpOptions"}))
        })
        .pure(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Session;
    use doc4me_doc::Document;

    #[test]
    fn help_commands_open_their_offline_surfaces_without_changing_the_document() {
        let mut session = Session::new(Document::from_text("Keep this text."));
        let before = session.doc.clone();
        for (id, name) in [("help.about", "about"), ("help.wordPerfect", "wordPerfectHelp"), ("help.wordPerfectOptions", "wordPerfectHelpOptions")] {
            let result = session.run(id, &json!({})).unwrap();
            assert_eq!(result, json!({"opened": name}));
            assert_eq!(session.ui_requests.last(), Some(&json!({"open": name})));
        }
        assert_eq!(session.doc, before);
    }
}
