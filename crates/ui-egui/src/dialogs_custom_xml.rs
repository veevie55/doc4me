//! Bounded editors for the DOCX custom XML parts carried by the document.
//!
//! The schema library edits only schema references in a store's datastore-properties part. The
//! attributes dialog edits one attribute on one element path. Both leave unknown XML content
//! intact and rely on the engine commands for validation and undo.

use egui::Ui;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::WordApp;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct XmlAttribute {
    name: String,
    value: String,
    namespace: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct XmlElement {
    path: String,
    name: String,
    namespace: Option<String>,
    attributes: Vec<XmlAttribute>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct XmlStore {
    id: String,
    bytes: usize,
    properties_part: Option<String>,
    schemas: Vec<String>,
    elements: Vec<XmlElement>,
    total_elements: usize,
    editable: bool,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "form", rename_all = "camelCase")]
pub enum CustomXmlDialog {
    SchemaLibrary { stores: Vec<XmlStore>, selected_store: String, schema: String, message: String, confirm_remove: Option<(String, String)> },
    ElementAttributes { stores: Vec<XmlStore>, selected_store: String, selected_path: String, name: String, value: String, message: String },
    Options { show_namespaces: bool, confirm_schema_removal: bool, message: String },
}

impl CustomXmlDialog {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::SchemaLibrary { .. } => "xmlSchemaLibrary",
            Self::ElementAttributes { .. } => "xmlElementAttributes",
            Self::Options { .. } => "xmlOptions",
        }
    }

    pub(crate) fn title(&self) -> &'static str {
        match self {
            Self::SchemaLibrary { .. } => "Schema Library",
            Self::ElementAttributes { .. } => "XML Element Attributes",
            Self::Options { .. } => "XML Options",
        }
    }
}

fn load_stores(app: &mut WordApp, params: Value) -> Result<Vec<XmlStore>, String> {
    let value = app.session.run("file.schemaLibrary", &params).map_err(|error| error.to_string())?;
    stores_from_result(&value)
}

fn load_attribute_stores(app: &mut WordApp) -> Result<Vec<XmlStore>, String> {
    let value = app.session.run("file.xmlElementAttributes", &json!({"list": true})).map_err(|error| error.to_string())?;
    stores_from_result(&value)
}

fn stores_from_result(value: &Value) -> Result<Vec<XmlStore>, String> {
    serde_json::from_value(value.get("stores").cloned().unwrap_or(Value::Null))
        .map_err(|error| format!("Could not read the custom XML store list: {error}"))
}

fn load_options(app: &WordApp) -> (bool, bool) {
    (app.session.prefs.xml_show_namespaces, app.session.prefs.xml_confirm_schema_removal)
}

/// Build a dialog in response to an engine request or `ui.dialog`.
pub(crate) fn open(name: &str, app: &mut WordApp) -> Option<CustomXmlDialog> {
    Some(match name {
        "xmlSchemaLibrary" => match load_stores(app, json!({"action": "list"})) {
            Ok(stores) => CustomXmlDialog::SchemaLibrary {
                selected_store: stores.first().map_or_else(String::new, |store| store.id.clone()),
                stores,
                schema: String::new(),
                message: String::new(),
                confirm_remove: None,
            },
            Err(error) => CustomXmlDialog::SchemaLibrary {
                stores: Vec::new(),
                selected_store: String::new(),
                schema: String::new(),
                message: error,
                confirm_remove: None,
            },
        },
        "xmlElementAttributes" => match load_attribute_stores(app) {
            Ok(stores) => CustomXmlDialog::ElementAttributes {
                selected_store: stores.first().map_or_else(String::new, |store| store.id.clone()),
                selected_path: stores.first().and_then(|store| store.elements.first()).map_or_else(String::new, |element| element.path.clone()),
                stores,
                name: String::new(),
                value: String::new(),
                message: String::new(),
            },
            Err(error) => CustomXmlDialog::ElementAttributes {
                stores: Vec::new(),
                selected_store: String::new(),
                selected_path: String::new(),
                name: String::new(),
                value: String::new(),
                message: error,
            },
        },
        "xmlOptions" => {
            let (show_namespaces, confirm_schema_removal) = load_options(app);
            CustomXmlDialog::Options { show_namespaces, confirm_schema_removal, message: String::new() }
        }
        _ => return None,
    })
}

/// Render the custom XML dialog; returns true when it should close.
pub(crate) fn body(app: &mut WordApp, ui: &mut Ui, dialog: &mut CustomXmlDialog) -> bool {
    match dialog {
        CustomXmlDialog::SchemaLibrary { stores, selected_store, schema, message, confirm_remove } => {
            schema_library_body(app, ui, stores, selected_store, schema, message, confirm_remove)
        }
        CustomXmlDialog::ElementAttributes { stores, selected_store, selected_path, name, value, message } => {
            attributes_body(app, ui, stores, selected_store, selected_path, name, value, message)
        }
        CustomXmlDialog::Options { show_namespaces, confirm_schema_removal, message } => {
            options_body(app, ui, show_namespaces, confirm_schema_removal, message)
        }
    }
}

fn schema_library_body(
    app: &mut WordApp,
    ui: &mut Ui,
    stores: &mut Vec<XmlStore>,
    selected_store: &mut String,
    schema: &mut String,
    message: &mut String,
    confirm_remove: &mut Option<(String, String)>,
) -> bool {
    ui.label(crate::tl!("View and manage schema references associated with custom XML stores."));
    ui.add_space(4.0);
    let mut action: Option<(&'static str, String, String)> = None;
    if stores.is_empty() {
        ui.label(crate::tl!("This document has no custom XML stores."));
    } else {
        let _ = store_picker(ui, "xml_schema_store", stores, selected_store);
        if confirm_remove.as_ref().is_some_and(|(store, _)| store != selected_store) {
            *confirm_remove = None;
        }
        if let Some(store) = stores.iter().find(|store| store.id == *selected_store) {
            ui.label(crate::i18n::fmt(crate::tl!("Store: {store}"), &[("store", &store.id)]));
            if let Some(error) = &store.error {
                ui.label(egui::RichText::new(error).small().color(ui.visuals().error_fg_color));
            }
            if store.properties_part.is_none() {
                ui.label(egui::RichText::new(crate::tl!("This store has no datastore properties part for schema references.")).small().weak());
            }
            ui.label(crate::tl!("Schemas:"));
            egui::ScrollArea::vertical().id_salt("xml_schema_refs").max_height(140.0).show(ui, |ui| {
                if store.schemas.is_empty() {
                    ui.label(crate::tl!("No schema references"));
                }
                for uri in &store.schemas {
                    ui.horizontal(|ui| {
                        ui.label(uri);
                        if ui.add_enabled(store.editable && store.properties_part.is_some(), egui::Button::new(crate::tl!("Remove"))).clicked() {
                            if app.session.prefs.xml_confirm_schema_removal {
                                *confirm_remove = Some((selected_store.clone(), uri.clone()));
                            } else {
                                action = Some(("remove", selected_store.clone(), uri.clone()));
                            }
                        }
                    });
                }
            });
            ui.add_space(4.0);
            ui.label(crate::tl!("Schema URI:"));
            ui.add(crate::widgets::text_line(schema));
            if !store.editable {
                ui.label(egui::RichText::new(crate::tl!("This XML store is preserved but cannot be edited here.")).small().weak());
            }
        }
    }

    if let Some((store_id, uri)) = confirm_remove.clone().filter(|(store, _)| store == selected_store) {
        ui.group(|ui| {
            ui.label(crate::i18n::fmt(crate::tl!("Remove schema reference {schema}?"), &[("schema", &uri)]));
            ui.horizontal(|ui| {
                if ui.button(crate::tl!("Remove")).clicked() {
                    action = Some(("remove", store_id.clone(), uri.clone()));
                }
                if ui.button(crate::tl!("Cancel")).clicked() {
                    *confirm_remove = None;
                }
            });
        });
    }

    let mut close = false;
    let (ok, cancel) = crate::dialogs::buttons(ui, crate::tl!("Add"));
    if ok {
        let uri = schema.trim();
        let editable = stores.iter().find(|store| store.id == *selected_store).is_some_and(|store| store.editable && store.properties_part.is_some());
        if uri.is_empty() {
            *message = crate::tl!("Enter an absolute HTTP(S) or URN schema URI.").to_string();
        } else if editable {
            action = Some(("add", selected_store.clone(), uri.to_string()));
        } else {
            *message = crate::tl!("Select a custom XML store with editable datastore properties first.").to_string();
        }
    }
    close |= cancel;

    if let Some((action, store, uri)) = action {
        if action == "remove" {
            *confirm_remove = None;
        }
        let params = json!({"action": action, "store": store, "schema": uri});
        match app.run("file.schemaLibrary", params) {
            Ok(_) => match load_stores(app, json!({"action": "list"})) {
                Ok(updated) => {
                    *stores = updated;
                    *message = String::new();
                }
                Err(error) => *message = error,
            },
            Err(error) => *message = error,
        }
    }

    show_message(ui, message);
    close || ui.input(|input| input.key_pressed(egui::Key::Escape))
}

fn attributes_body(
    app: &mut WordApp,
    ui: &mut Ui,
    stores: &mut Vec<XmlStore>,
    selected_store: &mut String,
    selected_path: &mut String,
    name: &mut String,
    value: &mut String,
    message: &mut String,
) -> bool {
    ui.label(crate::tl!("Select an XML element to add, change, or remove one of its attributes."));
    ui.add_space(4.0);
    if stores.is_empty() {
        ui.label(crate::tl!("This document has no custom XML stores."));
    } else {
        if store_picker(ui, "xml_attribute_store", stores, selected_store) {
            selected_path.clear();
            name.clear();
            value.clear();
        }
        if let Some(store) = stores.iter().find(|store| store.id == *selected_store) {
            if store.elements.iter().all(|element| element.path != *selected_path) {
                *selected_path = store.elements.first().map_or_else(String::new, |element| element.path.clone());
            }
            if let Some(error) = &store.error {
                ui.label(egui::RichText::new(error).small().color(ui.visuals().error_fg_color));
            }
            if store.elements.is_empty() {
                ui.label(crate::tl!("This store has no elements available for editing."));
            } else {
                let show_namespaces = app.session.prefs.xml_show_namespaces;
                let mut picked_path = None;
                ui.label(crate::tl!("Elements:"));
                egui::ScrollArea::vertical().id_salt("xml_elements").max_height(140.0).show_rows(ui, 24.0, store.elements.len(), |ui, range| {
                    for index in range {
                        let Some(element) = store.elements.get(index) else { continue };
                        let label = qualified_label(&element.name, element.namespace.as_deref(), show_namespaces);
                        if ui.selectable_label(*selected_path == element.path, label).on_hover_text(&element.path).clicked() {
                            picked_path = Some(element.path.clone());
                        }
                    }
                });
                if let Some(path) = picked_path {
                    *selected_path = path;
                    name.clear();
                    value.clear();
                }
                if let Some(element) = store.elements.iter().find(|element| element.path == *selected_path) {
                    ui.label(crate::i18n::fmt(
                        crate::tl!("Selected element: {name}"),
                        &[("name", &qualified_label(&element.name, element.namespace.as_deref(), show_namespaces))],
                    ));
                    ui.label(crate::tl!("Attributes:"));
                    egui::ScrollArea::vertical().id_salt("xml_attributes").max_height(100.0).show(ui, |ui| {
                        for attribute in &element.attributes {
                            let label = format!(
                                "{} = {}",
                                qualified_label(&attribute.name, attribute.namespace.as_deref(), show_namespaces),
                                attribute.value
                            );
                            if ui.selectable_label(*name == attribute.name, label).clicked() {
                                *name = attribute.name.clone();
                                *value = attribute.value.clone();
                            }
                        }
                    });
                    egui::Grid::new("xml_attribute_editor").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                        ui.label(crate::tl!("Name:"));
                        ui.add(crate::widgets::text_line(name));
                        ui.end_row();
                        ui.label(crate::tl!("Value:"));
                        ui.add(crate::widgets::text_line(value));
                        ui.end_row();
                    });
                }
                if !store.editable {
                    ui.label(egui::RichText::new(crate::tl!("This XML store is preserved but cannot be edited here.")).small().weak());
                }
            }
        }
    }

    let mut edit: Option<Option<String>> = None;
    let mut close = false;
    ui.horizontal(|ui| {
        let can_edit = stores.iter().find(|store| store.id == *selected_store).is_some_and(|store| store.editable)
            && !selected_path.is_empty()
            && !name.trim().is_empty();
        if ui.add_enabled(can_edit, egui::Button::new(crate::tl!("Set Attribute"))).clicked() {
            edit = Some(Some(value.clone()));
        }
        let exists = stores
            .iter()
            .find(|store| store.id == *selected_store)
            .and_then(|store| store.elements.iter().find(|element| element.path == *selected_path))
            .is_some_and(|element| element.attributes.iter().any(|attribute| attribute.name == *name));
        if ui.add_enabled(can_edit && exists, egui::Button::new(crate::tl!("Remove Attribute"))).clicked() {
            edit = Some(None);
        }
    });
    let (ok, cancel) = crate::dialogs::buttons(ui, crate::tl!("Close"));
    close |= ok || cancel;

    if let Some(value) = edit {
        let params = json!({"store": selected_store, "path": selected_path, "name": name, "value": value});
        match app.run("file.xmlElementAttributes", params) {
            Ok(_) => match load_attribute_stores(app) {
                Ok(updated) => {
                    *stores = updated;
                    *message = String::new();
                }
                Err(error) => *message = error,
            },
            Err(error) => *message = error,
        }
    }
    show_message(ui, message);
    close || ui.input(|input| input.key_pressed(egui::Key::Escape))
}

fn options_body(app: &mut WordApp, ui: &mut Ui, show_namespaces: &mut bool, confirm_schema_removal: &mut bool, message: &mut String) -> bool {
    ui.label(crate::tl!("Choose how the custom XML editor displays names and removes schema references."));
    ui.add_space(4.0);
    ui.checkbox(show_namespaces, crate::tl!("Show namespace URIs beside XML names"));
    ui.checkbox(confirm_schema_removal, crate::tl!("Confirm before removing a schema reference"));
    let (ok, cancel) = crate::dialogs::buttons(ui, crate::tl!("OK"));
    if ok {
        match app.run("file.xmlOptions", json!({"showNamespaces": show_namespaces, "confirmSchemaRemoval": confirm_schema_removal})) {
            Ok(_) => return true,
            Err(error) => *message = error,
        }
    }
    show_message(ui, message);
    cancel || ui.input(|input| input.key_pressed(egui::Key::Escape))
}

fn store_picker(ui: &mut Ui, id: &'static str, stores: &[XmlStore], selected: &mut String) -> bool {
    let before = selected.clone();
    if stores.iter().all(|store| store.id != *selected) {
        *selected = stores.first().map_or_else(String::new, |store| store.id.clone());
    }
    let shown = stores.iter().find(|store| store.id == *selected).map_or("", |store| store.id.as_str());
    egui::ComboBox::from_id_salt(id).selected_text(shown).show_ui(ui, |ui| {
        for store in stores {
            ui.selectable_value(selected, store.id.clone(), store.id.as_str());
        }
    });
    *selected != before
}

fn qualified_label(name: &str, namespace: Option<&str>, show_namespaces: bool) -> String {
    if show_namespaces && let Some(namespace) = namespace {
        return format!("{name}  ·  {namespace}");
    }
    name.to_string()
}

fn show_message(ui: &mut Ui, message: &str) {
    if !message.is_empty() {
        ui.label(egui::RichText::new(message).small().color(ui.visuals().error_fg_color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_names_route_to_the_right_custom_xml_editor() {
        let app = &mut WordApp::new(doc4me_engine::Session::new(doc4me_doc::Document::new()), crate::Services::default());
        assert_eq!(open("xmlOptions", app).map(|dialog| dialog.name()), Some("xmlOptions"));
        assert_eq!(open("xmlSchemaLibrary", app).map(|dialog| dialog.name()), Some("xmlSchemaLibrary"));
        assert_eq!(open("xmlElementAttributes", app).map(|dialog| dialog.name()), Some("xmlElementAttributes"));
        assert!(open("notAnXmlDialog", app).is_none());
    }

    #[test]
    fn namespace_display_is_opt_in_and_handles_missing_uris() {
        assert_eq!(qualified_label("x:item", Some("urn:x"), false), "x:item");
        assert_eq!(qualified_label("x:item", Some("urn:x"), true), "x:item  ·  urn:x");
        assert_eq!(qualified_label("item", None, true), "item");
    }
}
