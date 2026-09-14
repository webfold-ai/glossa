//! Offline / Mock Translation Provider.
//!
//! Provides rule-based translations for development, testing, and offline CI
//! without requiring external network access or paid API keys.

use async_trait::async_trait;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct OfflineProvider {
    dictionary: BTreeMap<(&'static str, &'static str), &'static str>,
}

impl OfflineProvider {
    pub fn new() -> Self {
        let mut dictionary = BTreeMap::new();
        // Seed common English -> Spanish / French / German translations
        dictionary.insert(("Hello", "es"), "Hola");
        dictionary.insert(("Hello", "fr"), "Bonjour");
        dictionary.insert(("Hello", "de"), "Hallo");
        dictionary.insert(("Welcome", "es"), "Bienvenido");
        dictionary.insert(("Welcome", "fr"), "Bienvenue");
        dictionary.insert(("Welcome", "de"), "Willkommen");
        dictionary.insert(("Settings", "es"), "Configuración");
        dictionary.insert(("Settings", "fr"), "Paramètres");
        dictionary.insert(("Settings", "de"), "Einstellungen");
        dictionary.insert(("Save", "es"), "Guardar");
        dictionary.insert(("Save", "fr"), "Enregistrer");
        dictionary.insert(("Save", "de"), "Speichern");
        dictionary.insert(("Cancel", "es"), "Cancelar");
        dictionary.insert(("Cancel", "fr"), "Annuler");
        dictionary.insert(("Cancel", "de"), "Abbrechen");

        Self { dictionary }
    }

    /// Mock translation maintaining slots and tags.
    pub fn translate_text(&self, text: &str, target_locale: &str) -> String {
        // Direct dictionary match
        if let Some(trans) = self.dictionary.get(&(text, target_locale)) {
            return (*trans).to_string();
        }

        // Prefix-based pseudo-localization for development: e.g. [es] Hello {name}
        match target_locale {
            "es" => format!("¡{text}!"),
            "fr" => format!("« {text} »"),
            "de" => format!("[DE] {text}"),
            "ja" => format!("【{text}】"),
            _ => format!("[{target_locale}] {text}"),
        }
    }
}

#[async_trait]
impl super::super::TranslationProvider for OfflineProvider {
    async fn translate_batch(
        &self,
        items: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, super::super::EngineError> {
        let results = items
            .iter()
            .map(|text| self.translate_text(text, target_locale))
            .collect();
        Ok(results)
    }

    fn provider_name(&self) -> &'static str {
        "offline"
    }
}
