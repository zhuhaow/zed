use anyhow;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct SemanticIndexSettings {
    pub enabled: bool,
}

#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, Debug)]
pub struct SemanticIndexSettingsContent {
    pub enabled: Option<bool>,
}

impl settings2::Settings for SemanticIndexSettings {
    const KEY: Option<&'static str> = Some("semantic_index");

    type FileContent = SemanticIndexSettingsContent;

    fn load(
        default_value: &Self::FileContent,
        user_values: &[&Self::FileContent],
        _: &mut gpui2::AppContext,
    ) -> anyhow::Result<Self> {
        Self::load_via_json_merge(default_value, user_values)
    }
}
