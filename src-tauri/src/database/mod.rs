pub mod app_state;
pub mod history;
pub mod license;
pub mod models;
pub mod models_crud;
pub mod providers;
pub mod schema;
pub mod settings;

use rusqlite::Result;

pub use models::*;
pub use schema::Database;

impl Database {
    pub fn get_settings(&self) -> Result<AppSettings> {
        settings::get_settings(self)
    }

    pub fn update_settings(&self, settings: &AppSettings) -> Result<()> {
        settings::update_settings(self, settings)
    }

    pub fn update_setting(&self, key: &str, value: &str) -> Result<()> {
        settings::update_setting(self, key, value)
    }

    pub fn get_app_state(&self) -> Result<AppState> {
        app_state::get_app_state(self)
    }

    pub fn update_app_state(&self, state: &AppState) -> Result<()> {
        app_state::update_app_state(self, state)
    }

    pub fn set_setup_complete(&self, complete: bool) -> Result<()> {
        app_state::set_setup_complete(self, complete)
    }

    pub fn set_current_setup_step(&self, step: i32) -> Result<()> {
        app_state::set_current_setup_step(self, step)
    }

    pub fn get_models(&self) -> Result<Vec<WhisperModel>> {
        models_crud::get_models(self)
    }

    pub fn get_model(&self, id: &str) -> Result<Option<WhisperModel>> {
        models_crud::get_model(self, id)
    }

    pub fn set_model_downloaded(
        &self,
        id: &str,
        downloaded: bool,
        path: Option<&str>,
    ) -> Result<()> {
        models_crud::set_model_downloaded(self, id, downloaded, path)
    }

    pub fn set_selected_model(&self, model_id: Option<&str>) -> Result<()> {
        models_crud::set_selected_model(self, model_id)
    }

    pub fn add_transcription(
        &self,
        text: &str,
        model_id: &str,
        language: &str,
        duration_ms: i64,
    ) -> Result<i64> {
        history::add_transcription(self, text, model_id, language, duration_ms)
    }

    pub fn get_transcription_history(
        &self,
        search: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<TranscriptionHistory>> {
        history::get_transcription_history(self, search, offset, limit)
    }

    pub fn get_transcription_history_count(&self, search: Option<&str>) -> Result<i64> {
        history::get_transcription_history_count(self, search)
    }

    pub fn clear_transcription_history(&self) -> Result<()> {
        history::clear_transcription_history(self)
    }

    pub fn delete_transcription(&self, id: i64) -> Result<()> {
        history::delete_transcription(self, id)
    }

    pub fn get_license(&self) -> Result<LicenseData> {
        license::get_license(self)
    }

    pub fn save_license(&self, license: &LicenseData) -> Result<()> {
        license::save_license(self, license)
    }

    pub fn clear_license(&self) -> Result<()> {
        license::clear_license(self)
    }

    pub fn get_cloud_providers(&self) -> Result<Vec<CloudProviderRecord>> {
        providers::get_cloud_providers(self)
    }

    pub fn get_cloud_provider(&self, id: &str) -> Result<Option<CloudProviderRecord>> {
        providers::get_cloud_provider(self, id)
    }

    pub fn save_cloud_provider(&self, record: &CloudProviderRecord) -> Result<()> {
        providers::save_cloud_provider(self, record)
    }

    pub fn delete_cloud_provider(&self, id: &str) -> Result<()> {
        providers::delete_cloud_provider(self, id)
    }

    pub fn get_ai_formatting_providers(&self) -> Result<Vec<AiFormattingProviderRecord>> {
        providers::get_ai_formatting_providers(self)
    }

    pub fn get_ai_formatting_provider(
        &self,
        id: &str,
    ) -> Result<Option<AiFormattingProviderRecord>> {
        providers::get_ai_formatting_provider(self, id)
    }

    pub fn save_ai_formatting_provider(&self, record: &AiFormattingProviderRecord) -> Result<()> {
        providers::save_ai_formatting_provider(self, record)
    }

    pub fn delete_ai_formatting_provider(&self, id: &str) -> Result<()> {
        providers::delete_ai_formatting_provider(self, id)
    }
}
