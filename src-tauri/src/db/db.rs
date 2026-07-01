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

async fn init_formats(db: &DatabaseConnection) -> Result<(), DbErr> {
    let count = Formats::find().count(db).await?;
    if count > 0 {
        println!("✅ Formats already exist in DB");
        return Ok(());
    }
    
    let now = chrono::Utc::now();
    
    let formats = vec![
        ("json", "JSON", json!(["json", "hjson"]), 
         "JavaScript Object Notation — легковесный формат обмена данными, основанный на синтаксисе JavaScript",
         "FileBraces", "from-yellow-500/30 to-amber-500/15", "shadow-yellow-500/20",
         "text-yellow-400", "hover:border-yellow-500/60"),
        ("yaml", "YAML", json!(["yaml", "yml"]),
         "YAML Ain't Markup Language — человекочитаемый формат сериализации данных, популярный в конфигурациях и DevOps",
         "FileText", "from-blue-500/30 to-cyan-500/15", "shadow-blue-500/20",
         "text-blue-400", "hover:border-blue-500/60"),
        ("csv", "CSV", json!(["csv", "tsv"]),
         "Comma-Separated Values — табличный формат для хранения и обмена данными между базами, Excel и аналитическими системами",
         "FileSpreadsheet", "from-green-500/30 to-emerald-500/15", "shadow-green-500/20",
         "text-green-400", "hover:border-green-500/60"),
        ("xml", "XML", json!(["xml"]),
         "Extensible Markup Language — универсальный язык разметки с древовидной структурой, широко используется в API, SOAP и конфигурациях",
         "FileCode", "from-orange-500/30 to-red-500/15", "shadow-orange-500/20",
         "text-orange-400", "hover:border-orange-500/60"),
        ("toml", "TOML", json!(["toml"]),
         "Tom's Obvious Minimal Language — минималистичный формат конфигураций с чёткой структурой, любимец Rust-сообщества",
         "AlignLeft", "from-orange-400/30 to-yellow-500/15", "shadow-orange-400/20",
         "text-orange-400", "hover:border-orange-400/60"),
        ("ini", "INI", json!(["ini", "cfg", "conf"]),
         "Простейший формат конфигурационных файлов с секциями и парами ключ-значение, используется повсеместно",
         "ListOrdered", "from-gray-400/30 to-slate-500/15", "shadow-gray-400/20",
         "text-gray-400", "hover:border-gray-400/60"),
        ("markdown", "Markdown", json!(["md", "markdown", "mdown", "mkd"]),
         "Легковесный язык разметки для форматирования текста, конвертируется в HTML, PDF и другие форматы",
         "Braces", "from-purple-500/30 to-violet-500/15", "shadow-purple-500/20",
         "text-purple-400", "hover:border-purple-500/60"),
        ("html", "HTML", json!(["html", "htm"]),
         "HyperText Markup Language — стандартный язык веб-разметки, основа всех веб-страниц и шаблонов",
         "Globe", "from-orange-500/30 to-red-500/15", "shadow-orange-500/20",
         "text-orange-300", "hover:border-orange-500/60"),
    ];
    
    for (format_id, name, extensions, description, icon, color, glow, text_color, border_hover) in formats {
        let new_format = FormatActiveModel {
            format_id: Set(format_id.to_string()),
            name: Set(name.to_string()),
            extensions: Set(extensions),
            description: Set(description.to_string()),
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