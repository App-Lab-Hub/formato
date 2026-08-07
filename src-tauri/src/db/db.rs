// src/db/db.rs

use sea_orm::prelude::Expr;
use sea_orm::sea_query::{Func,Table};
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
use crate::AppState;
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
        // ============ ТЕКСТОВЫЕ И КОНФИГУРАЦИОННЫЕ ============
        ("json", "JSON", json!(["json", "hjson"]), 
        "FileBraces", 
        "dark:from-yellow-500/30 light:from-yellow-600/40 dark:to-amber-500/15 light:to-amber-600/25", 
        "dark:shadow-yellow-500/20 light:shadow-yellow-600/30",
        "dark:text-yellow-400 light:text-yellow-700", 
        "dark:hover:border-yellow-500/60 light:hover:border-yellow-600/50",
        "text"),
        
        ("yaml", "YAML", json!(["yaml", "yml"]),
        "FileText", 
        "dark:from-blue-500/30 light:from-blue-600/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-blue-500/20 light:shadow-blue-600/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "text"),
        
        ("csv", "CSV", json!(["csv", "tsv"]),
        "FileSpreadsheet", 
        "dark:from-green-500/30 light:from-green-600/40 dark:to-emerald-500/15 light:to-emerald-600/25", 
        "dark:shadow-green-500/20 light:shadow-green-600/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50",
        "text"),
        
        ("xml", "XML", json!(["xml"]),
        "FileCode", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "text"),
        
        ("toml", "TOML", json!(["toml"]),
        "AlignLeft", 
        "dark:from-orange-400/30 light:from-orange-500/40 dark:to-yellow-500/15 light:to-yellow-600/25", 
        "dark:shadow-orange-400/20 light:shadow-orange-500/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-400/60 light:hover:border-orange-500/50",
        "text"),
        
        ("ini", "INI", json!(["ini", "cfg", "conf"]),
        "ListOrdered", 
        "dark:from-gray-400/30 light:from-gray-500/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-400/20 light:shadow-gray-500/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-400/60 light:hover:border-gray-500/50",
        "text"),
        
        ("md", "Markdown", json!(["md", "markdown", "mdown", "mkd"]),
        "Braces", 
        "dark:from-purple-500/30 light:from-purple-600/40 dark:to-violet-500/15 light:to-violet-600/25", 
        "dark:shadow-purple-500/20 light:shadow-purple-600/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50",
        "text"),
        
        ("html", "HTML", json!(["html", "htm"]),
        "Globe", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-300 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "text"),
        
        ("txt", "TXT", json!(["txt", "text"]),
        "FileAlt", 
        "dark:from-gray-500/30 light:from-gray-600/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-500/20 light:shadow-gray-600/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-500/60 light:hover:border-gray-600/50",
        "text"),
        
        ("rtf", "RTF", json!(["rtf"]),
        "FileRtf", 
        "dark:from-blue-400/30 light:from-blue-500/40 dark:to-indigo-500/15 light:to-indigo-600/25", 
        "dark:shadow-blue-400/20 light:shadow-blue-500/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-400/60 light:hover:border-blue-500/50",
        "text"),

        // ============ ДОКУМЕНТЫ ============
        ("pdf", "PDF", json!(["pdf"]),
        "FilePdf", 
        "dark:from-red-500/30 light:from-red-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-red-500/20 light:shadow-red-600/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "document"),
        
        ("docx", "DOCX", json!(["docx"]),
        "FileWord", 
        "dark:from-blue-600/30 light:from-blue-700/40 dark:to-indigo-500/15 light:to-indigo-600/25", 
        "dark:shadow-blue-600/20 light:shadow-blue-700/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "document"),
        
        ("odt", "ODT", json!(["odt"]),
        "FileOdt", 
        "dark:from-green-600/30 light:from-green-700/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-green-600/20 light:shadow-green-700/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50",
        "document"),
        
        ("xlsx", "XLSX", json!(["xlsx", "xls"]),
        "FileExcel", 
        "dark:from-emerald-500/30 light:from-emerald-600/40 dark:to-green-500/15 light:to-green-600/25", 
        "dark:shadow-emerald-500/20 light:shadow-emerald-600/30",
        "dark:text-emerald-400 light:text-emerald-700", 
        "dark:hover:border-emerald-500/60 light:hover:border-emerald-600/50",
        "document"),
        
        // ============ ИЗОБРАЖЕНИЯ ============
        ("jpg", "JPG", json!(["jpg", "jpeg", "jfif", "pjpeg"]),
        "FileJpg", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-amber-500/15 light:to-amber-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "image"),

        ("png", "PNG", json!(["png"]),
        "FilePng", 
        "dark:from-blue-400/30 light:from-blue-500/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-blue-400/20 light:shadow-blue-500/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-400/60 light:hover:border-blue-500/50",
        "image"),

        ("webp", "WEBP", json!(["webp"]),
        "FileWebp", 
        "dark:from-cyan-500/30 light:from-cyan-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-cyan-500/20 light:shadow-cyan-600/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "image"),

        ("avif", "AVIF", json!(["avif"]),
        "FileAvif", 
        "dark:from-purple-500/30 light:from-purple-600/40 dark:to-violet-500/15 light:to-violet-600/25", 
        "dark:shadow-purple-500/20 light:shadow-purple-600/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50",
        "image"),

        ("gif", "GIF", json!(["gif"]),
        "FileImage", 
        "dark:from-pink-500/30 light:from-pink-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-pink-500/20 light:shadow-pink-600/30",
        "dark:text-pink-400 light:text-pink-700", 
        "dark:hover:border-pink-500/60 light:hover:border-pink-600/50",
        "image"),

        ("bmp", "BMP", json!(["bmp"]),
        "FileImage", 
        "dark:from-gray-500/30 light:from-gray-600/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-500/20 light:shadow-gray-600/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-500/60 light:hover:border-gray-600/50",
        "image"),

        ("tiff", "TIFF", json!(["tiff", "tif"]),
        "FileImage", 
        "dark:from-indigo-500/30 light:from-indigo-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-500/20 light:shadow-indigo-600/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "image"),

        ("ico", "ICO", json!(["ico"]),
        "FileImage", 
        "dark:from-yellow-500/30 light:from-yellow-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-yellow-500/20 light:shadow-yellow-600/30",
        "dark:text-yellow-400 light:text-yellow-700", 
        "dark:hover:border-yellow-500/60 light:hover:border-yellow-600/50",
        "image"),

        ("qoi", "QOI", json!(["qoi"]),
        "FileImage", 
        "dark:from-emerald-500/30 light:from-emerald-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-emerald-500/20 light:shadow-emerald-600/30",
        "dark:text-emerald-400 light:text-emerald-700", 
        "dark:hover:border-emerald-500/60 light:hover:border-emerald-600/50",
        "image"),

        ("tga", "TGA", json!(["tga"]),
        "FileImage", 
        "dark:from-rose-500/30 light:from-rose-600/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-rose-500/20 light:shadow-rose-600/30",
        "dark:text-rose-400 light:text-rose-700", 
        "dark:hover:border-rose-500/60 light:hover:border-rose-600/50",
        "image"),

        ("exr", "EXR", json!(["exr"]),
        "FileImage", 
        "dark:from-red-500/30 light:from-red-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-red-500/20 light:shadow-red-600/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "image"),

        ("hdr", "HDR", json!(["hdr"]),
        "FileImage", 
        "dark:from-amber-500/30 light:from-amber-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-amber-500/20 light:shadow-amber-600/30",
        "dark:text-amber-400 light:text-amber-700", 
        "dark:hover:border-amber-500/60 light:hover:border-amber-600/50",
        "image"),

        ("pnm", "PNM", json!(["pnm", "pgm", "ppm"]),
        "FileImage", 
        "dark:from-lime-500/30 light:from-lime-600/40 dark:to-green-500/15 light:to-green-600/25", 
        "dark:shadow-lime-500/20 light:shadow-lime-600/30",
        "dark:text-lime-400 light:text-lime-700", 
        "dark:hover:border-lime-500/60 light:hover:border-lime-600/50",
        "image"),

        ("ff", "Farbfeld", json!(["ff"]),
        "FileImage", 
        "dark:from-violet-500/30 light:from-violet-600/40 dark:to-purple-500/15 light:to-purple-600/25", 
        "dark:shadow-violet-500/20 light:shadow-violet-600/30",
        "dark:text-violet-400 light:text-violet-700", 
        "dark:hover:border-violet-500/60 light:hover:border-violet-600/50",
        "image"),

        // ============ АУДИО ============
        ("mp3", "MP3", json!(["mp3"]),
        "FileMp3", 
        "dark:from-rose-500/30 light:from-rose-600/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-rose-500/20 light:shadow-rose-600/30",
        "dark:text-rose-400 light:text-rose-700", 
        "dark:hover:border-rose-500/60 light:hover:border-rose-600/50",
        "audio"),

        ("wav", "WAV", json!(["wav"]),
        "FileWav", 
        "dark:from-teal-500/30 light:from-teal-600/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-teal-500/20 light:shadow-teal-600/30",
        "dark:text-teal-400 light:text-teal-700", 
        "dark:hover:border-teal-500/60 light:hover:border-teal-600/50",
        "audio"),

        ("aac", "AAC", json!(["aac"]),
        "FileAudio", 
        "dark:from-purple-500/30 light:from-purple-600/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-purple-500/20 light:shadow-purple-600/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50",
        "audio"),

        ("flac", "FLAC", json!(["flac"]),
        "FileAudio", 
        "dark:from-emerald-500/30 light:from-emerald-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-emerald-500/20 light:shadow-emerald-600/30",
        "dark:text-emerald-400 light:text-emerald-700", 
        "dark:hover:border-emerald-500/60 light:hover:border-emerald-600/50",
        "audio"),

        ("ogg", "OGG", json!(["ogg"]),
        "FileAudio", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "audio"),

        ("opus", "OPUS", json!(["opus"]),
        "FileAudio", 
        "dark:from-green-500/30 light:from-green-600/40 dark:to-emerald-500/15 light:to-emerald-600/25", 
        "dark:shadow-green-500/20 light:shadow-green-600/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50",
        "audio"),

        ("wma", "WMA", json!(["wma"]),
        "FileAudio", 
        "dark:from-blue-500/30 light:from-blue-600/40 dark:to-indigo-500/15 light:to-indigo-600/25", 
        "dark:shadow-blue-500/20 light:shadow-blue-600/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "audio"),

        ("m4a", "M4A", json!(["m4a"]),
        "FileAudio", 
        "dark:from-cyan-500/30 light:from-cyan-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-cyan-500/20 light:shadow-cyan-600/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "audio"),

        ("aiff", "AIFF", json!(["aiff", "aif", "aifc"]),
        "FileAudio", 
        "dark:from-pink-500/30 light:from-pink-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-pink-500/20 light:shadow-pink-600/30",
        "dark:text-pink-400 light:text-pink-700", 
        "dark:hover:border-pink-500/60 light:hover:border-pink-600/50",
        "audio"),

        ("ac3", "AC3", json!(["ac3"]),
        "FileAudio", 
        "dark:from-indigo-500/30 light:from-indigo-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-500/20 light:shadow-indigo-600/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "audio"),

        // ============ НОВЫЕ АУДИО ФОРМАТЫ ============
        ("voc", "Creative Voice", json!(["voc"]),
        "FileAudio", 
        "dark:from-amber-500/30 light:from-amber-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-amber-500/20 light:shadow-amber-600/30",
        "dark:text-amber-400 light:text-amber-700", 
        "dark:hover:border-amber-500/60 light:hover:border-amber-600/50",
        "audio"),

        ("wv", "WavPack", json!(["wv"]),
        "FileAudio", 
        "dark:from-cyan-500/30 light:from-cyan-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-cyan-500/20 light:shadow-cyan-600/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "audio"),
        // ============ ВИДЕО ============
        ("mp4", "MP4", json!(["mp4"]),
        "FileMp4", 
        "dark:from-red-600/30 light:from-red-700/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-red-600/20 light:shadow-red-700/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "video"),

        ("mov", "MOV", json!(["mov"]),
        "FileMov", 
        "dark:from-pink-500/30 light:from-pink-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-pink-500/20 light:shadow-pink-600/30",
        "dark:text-pink-400 light:text-pink-700", 
        "dark:hover:border-pink-500/60 light:hover:border-pink-600/50",
        "video"),

        ("avi", "AVI", json!(["avi"]),
        "FileVideo", 
        "dark:from-blue-600/30 light:from-blue-700/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-blue-600/20 light:shadow-blue-700/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "video"),

        ("mkv", "MKV", json!(["mkv"]),
        "FileVideo", 
        "dark:from-purple-600/30 light:from-purple-700/40 dark:to-violet-500/15 light:to-violet-600/25", 
        "dark:shadow-purple-600/20 light:shadow-purple-700/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50",
        "video"),

        ("webm", "WEBM", json!(["webm"]),
        "FileVideo", 
        "dark:from-cyan-600/30 light:from-cyan-700/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-cyan-600/20 light:shadow-cyan-700/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "video"),

        ("wmv", "WMV", json!(["wmv"]),
        "FileVideo", 
        "dark:from-indigo-600/30 light:from-indigo-700/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-600/20 light:shadow-indigo-700/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "video"),

        ("flv", "FLV", json!(["flv"]),
        "FileVideo", 
        "dark:from-orange-600/30 light:from-orange-700/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-600/20 light:shadow-orange-700/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "video"),

        ("3gp", "3GP", json!(["3gp"]),
        "FileVideo", 
        "dark:from-green-600/30 light:from-green-700/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-green-600/20 light:shadow-green-700/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50",
        "video"),

        ("m4v", "M4V", json!(["m4v"]),
        "FileVideo", 
        "dark:from-rose-600/30 light:from-rose-700/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-rose-600/20 light:shadow-rose-700/30",
        "dark:text-rose-400 light:text-rose-700", 
        "dark:hover:border-rose-500/60 light:hover:border-rose-600/50",
        "video"),

        ("ts", "MPEG-TS", json!(["ts", "m2ts"]),
        "FileVideo", 
        "dark:from-teal-600/30 light:from-teal-700/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-teal-600/20 light:shadow-teal-700/30",
        "dark:text-teal-400 light:text-teal-700", 
        "dark:hover:border-teal-500/60 light:hover:border-teal-600/50",
        "video"),

        ("vob", "VOB", json!(["vob"]),
        "FileVideo", 
        "dark:from-amber-600/30 light:from-amber-700/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-amber-600/20 light:shadow-amber-700/30",
        "dark:text-amber-400 light:text-amber-700", 
        "dark:hover:border-amber-500/60 light:hover:border-amber-600/50",
        "video"),

        ("mpg", "MPEG", json!(["mpg", "mpeg"]),
        "FileVideo", 
        "dark:from-red-500/30 light:from-red-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-red-500/20 light:shadow-red-600/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "video"),

        // ============ НОВЫЕ ВИДЕО ФОРМАТЫ ============
        ("roq", "RoQ", json!(["roq"]),
        "FileVideo", 
        "dark:from-red-500/30 light:from-red-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-red-500/20 light:shadow-red-600/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "video"),

        ("mxf", "MXF", json!(["mxf"]),
        "FileVideo", 
        "dark:from-slate-500/30 light:from-slate-600/40 dark:to-gray-500/15 light:to-gray-600/25", 
        "dark:shadow-slate-500/20 light:shadow-slate-600/30",
        "dark:text-slate-400 light:text-slate-700", 
        "dark:hover:border-slate-500/60 light:hover:border-slate-600/50",
        "video"),

        ("prores", "ProRes", json!(["prores"]),
        "FileVideo", 
        "dark:from-teal-500/30 light:from-teal-600/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-teal-500/20 light:shadow-teal-600/30",
        "dark:text-teal-400 light:text-teal-700", 
        "dark:hover:border-teal-500/60 light:hover:border-teal-600/50",
        "video"),

        ("dnxhd", "DNxHD", json!(["dnxhd"]),
        "FileVideo", 
        "dark:from-indigo-500/30 light:from-indigo-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-500/20 light:shadow-indigo-600/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "video"),


    ];

    for (format_id, name, extensions, icon, color, glow, text_color, border_hover, format_type) in formats {
        let new_format = FormatActiveModel {
            format_id: Set(format_id.to_string()),
            name: Set(name.to_string()),
            extensions: Set(extensions),
            icon: Set(icon.to_string()),
            color: Set(color.to_string()),
            glow: Set(glow.to_string()),
            text_color: Set(text_color.to_string()),
            border_hover: Set(border_hover.to_string()),
            format_type: Set(format_type.to_string()),
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
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    
    // Проверяем, существует ли уже запись с таким хэшем
    let existing = conversions::Entity::find()
        .filter(conversions::Column::FileHash.eq(file_hash))
        .one(db)
        .await
        .map_err(|e| format!("DB query error: {}", e))?;
    
    if existing.is_some() {
        // Запись уже существует - ничего не делаем
        println!("Conversion already exists for hash: {}", file_hash);
        return Ok(());
    }
    
    // Записи нет - создаем новую
    let model = conversions::ActiveModel {
        file_hash: Set(file_hash.to_string()),
        converted_path: Set(converted_path.to_string()),
        created_at: Set(Utc::now()),
    };
    
    model.insert(db)
        .await
        .map_err(|e| format!("DB insert error: {}", e))?;
    
    println!("Saved conversion: {} -> {}", file_hash, converted_path);
    Ok(())
}







// ✅ Новая функция для удаления из БД по пути
pub async fn delete_conversion_by_path(
    db: &DatabaseConnection,
    file_path: &str,
) -> Result<(), String> {
    let result = Entity::find()
        .filter(Column::ConvertedPath.eq(file_path))
        .one(db)
        .await
        .map_err(|e| format!("DB find error: {e}"))?;
    
    if let Some(model) = result {
        let active_model: conversions::ActiveModel = model.into();
        active_model.delete(db).await.map_err(|e| format!("DB delete error: {e}"))?;
        println!("✅ Deleted from DB: {}", file_path);
    }
    
    Ok(())
}





#[tauri::command]
pub async fn reset_database(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let db_guard = state.db.lock().await;
    let db = db_guard.as_ref().ok_or("Database not initialized")?;

    println!("🔄 Resetting database and cleaning files...");

    // 1. Удаляем папки и создаем заново
    let converted_dir = crate::paths::converted_dir();
    if converted_dir.exists() {
        std::fs::remove_dir_all(&converted_dir)
            .map_err(|e| format!("Failed to remove converted dir: {e}"))?;
        println!("🗑️ Deleted converted dir");
    }
    // Создаем папку заново
    std::fs::create_dir_all(&converted_dir)
        .map_err(|e| format!("Failed to create converted dir: {e}"))?;
    println!("✅ Recreated converted dir");

    let temp_dir = crate::paths::temp_dir();
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to remove temp dir: {e}"))?;
        println!("🗑️ Deleted temp dir");
    }
    // Создаем папку заново
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {e}"))?;
    println!("✅ Recreated temp dir");

    // 2. Выполняем сырые SQL запросы для удаления таблиц
    let sql = "
        DROP TABLE IF EXISTS conversions;
        DROP TABLE IF EXISTS formats;
    ";

    db.execute_unprepared(sql)
        .await
        .map_err(|e| format!("Failed to reset database: {e}"))?;

    println!("✅ Tables dropped");

    // 3. Создаем таблицы заново
    crate::create_tables!(
        db,
        Formats,
        Conversions
    );

    println!("✅ Tables recreated");

    init_formats(db).await
        .map_err(|e| format!("Failed to reinit formats: {e}"))?;

    println!("✅ Database reset complete!");
    Ok(())
}