// src/db/models/conversions.rs

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "conversions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub file_hash: String,

    pub converted_path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ActiveModelBehavior for ActiveModel {}
