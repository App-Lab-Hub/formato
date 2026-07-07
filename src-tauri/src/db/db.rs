// src/db/db.rs

use sea_orm::prelude::Expr;
use sea_orm::sea_query::Func;
use sea_orm::{ConnectionTrait, DbErr, TransactionSession};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm::QueryFilter;
use sea_orm::ColumnTrait;
use serde_json::json;

use sea_orm::{entity::*, query::*};

use crate::create_tables;
use crate::paths::db_path;
use crate::db::models::{Formats, FormatModel, FormatActiveModel, Conversions, ConversionModel, ConversionActiveModel};
use crate::db::models::formats::Column as FormatColumn;
use crate::db::models::conversions::Column as ConversionColumn;

// ============================================================
// ИНИЦИАЛИЗАЦИЯ БД
// ============================================================

pub async fn db_init() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db_path = db_path();
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
    
    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(true);
    opt.max_connections(50);
    let db: DatabaseConnection = Database::connect(opt).await?;

    create_tables!(
        db,
        Formats,
        Conversions
    );
    
    init_formats(&db).await?;
    
    println!("✅ Database initialized at: {}", db_path.display());
    Ok(db)
}

// ============================================================
// INIT FORMATS
// ============================================================

// src/db/init.rs

async fn init_formats(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = Formats::find().count(db).await?;
    if count > 0 {
        println!("✅ Formats already exist in DB");
        return Ok(());
    }
    
    let now = chrono::Utc::now();

    let formats = vec![
        ("json", "JSON", json!(["json", "hjson"]), 
        "FileBraces", 
        "dark:from-yellow-500/30 light:from-yellow-600/40 dark:to-amber-500/15 light:to-amber-600/25", 
        "dark:shadow-yellow-500/20 light:shadow-yellow-600/30",
        "dark:text-yellow-400 light:text-yellow-700", 
        "dark:hover:border-yellow-500/60 light:hover:border-yellow-600/50"),
        
        ("yaml", "YAML", json!(["yaml", "yml"]),
        "FileText", 
        "dark:from-blue-500/30 light:from-blue-600/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-blue-500/20 light:shadow-blue-600/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50"),
        
        ("csv", "CSV", json!(["csv", "tsv"]),
        "FileSpreadsheet", 
        "dark:from-green-500/30 light:from-green-600/40 dark:to-emerald-500/15 light:to-emerald-600/25", 
        "dark:shadow-green-500/20 light:shadow-green-600/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50"),
        
        ("xml", "XML", json!(["xml"]),
        "FileCode", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50"),
        
        ("toml", "TOML", json!(["toml"]),
        "AlignLeft", 
        "dark:from-orange-400/30 light:from-orange-500/40 dark:to-yellow-500/15 light:to-yellow-600/25", 
        "dark:shadow-orange-400/20 light:shadow-orange-500/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-400/60 light:hover:border-orange-500/50"),
        
        ("ini", "INI", json!(["ini", "cfg", "conf"]),
        "ListOrdered", 
        "dark:from-gray-400/30 light:from-gray-500/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-400/20 light:shadow-gray-500/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-400/60 light:hover:border-gray-500/50"),
        
        ("md", "Markdown", json!(["md", "markdown", "mdown", "mkd"]),
        "Braces", 
        "dark:from-purple-500/30 light:from-purple-600/40 dark:to-violet-500/15 light:to-violet-600/25", 
        "dark:shadow-purple-500/20 light:shadow-purple-600/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50"),
        
        ("html", "HTML", json!(["html", "htm"]),
        "Globe", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-300 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50"),
    ];

    for (format_id, name, extensions, icon, color, glow, text_color, border_hover) in formats {
        let new_format = FormatActiveModel {
            format_id: Set(format_id.to_string()),
            name: Set(name.to_string()),
            extensions: Set(extensions),
            icon: Set(icon.to_string()),
            color: Set(color.to_string()),
            glow: Set(glow.to_string()),
            text_color: Set(text_color.to_string()),
            border_hover: Set(border_hover.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        
        new_format.insert(db).await?;
        println!("✅ Format inserted: {}", format_id);
    }
    
    println!("✅ All formats initialized!");
    Ok(())
}

// ============================================================
// CRUD ДЛЯ ФОРМАТОВ (экспортируемые функции)
// ============================================================

pub async fn get_all_formats(
    db: &DatabaseConnection,
) -> Result<Vec<FormatModel>, DbErr> {
    Formats::find().all(db).await
}

pub async fn get_format_by_id(
    db: &DatabaseConnection,
    format_id: &str,
) -> Result<Option<FormatModel>, DbErr> {
    Formats::find()
        .filter(FormatColumn::FormatId.eq(format_id))
        .one(db)
        .await
}







// src/db/conversions.rs

use chrono::Utc;
use crate::db::models::conversions;
use conversions::{Entity, Column};
pub async fn find_conversion(
    db: &DatabaseConnection,
    file_hash: &str,
) -> Option<String> {
  
    
    let result = Entity::find()
        .filter(Column::FileHash.eq(file_hash))
        .one(db)
        .await
        .ok()??;
    
    Some(result.converted_path)
}

pub async fn save_conversion(
    db: &DatabaseConnection,
    file_hash: &str,
    converted_path: &str,
) -> Result<(), String> {
    let model = conversions::ActiveModel {
        file_hash: Set(file_hash.to_string()),
        converted_path: Set(converted_path.to_string()),
        created_at: Set(Utc::now()),
    };
    
    model.insert(db).await.map_err(|e| format!("DB insert error: {e}"))?;
    Ok(())
}