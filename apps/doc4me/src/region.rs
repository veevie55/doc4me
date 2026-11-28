//! The system region new documents follow: paper size, default units, and decimal separator.

pub fn system() -> doc4me_engine::Region {
    let region = doc4me_ui_egui::i18n::system_region();
    #[cfg(target_os = "windows")]
    if std::env::var_os("DOC4ME_REGION").is_none() && std::env::var_os("WORDCRAFT_REGION").is_none() {
        return match windows::System::UserProfile::GlobalizationPreferences::HomeGeographicRegion() {
            Ok(home) => region.with_territory(&home.to_string()),
            Err(e) => {
                log::debug!("system home region: {e}");
                region
            }
        };
    }
    region
}
