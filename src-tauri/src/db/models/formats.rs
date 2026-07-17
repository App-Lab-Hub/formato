// src/db/models/formats.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "formats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub format_id: String,
    
    pub name: String,
    pub extensions: JsonValue,
    pub icon: String,
    pub color: String,
    pub glow: String,
    pub text_color: String,
    pub border_hover: String,
    pub format_type: String, // text, image, audio, video
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ActiveModelBehavior for ActiveModel {}