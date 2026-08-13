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
        "FileGif", 
        "dark:from-pink-500/30 light:from-pink-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-pink-500/20 light:shadow-pink-600/30",
        "dark:text-pink-400 light:text-pink-700", 
        "dark:hover:border-pink-500/60 light:hover:border-pink-600/50",
        "image"),

        ("bmp", "BMP", json!(["bmp"]),
        "FileBmp", 
        "dark:from-gray-500/30 light:from-gray-600/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-500/20 light:shadow-gray-600/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-500/60 light:hover:border-gray-600/50",
        "image"),

        ("tiff", "TIFF", json!(["tiff", "tif"]),
        "FileTiff", 
        "dark:from-indigo-500/30 light:from-indigo-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-500/20 light:shadow-indigo-600/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "image"),

        ("ico", "ICO", json!(["ico"]),
        "FileIco", 
        "dark:from-yellow-500/30 light:from-yellow-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-yellow-500/20 light:shadow-yellow-600/30",
        "dark:text-yellow-400 light:text-yellow-700", 
        "dark:hover:border-yellow-500/60 light:hover:border-yellow-600/50",
        "image"),

        ("qoi", "QOI", json!(["qoi"]),
        "FileQoi", 
        "dark:from-emerald-500/30 light:from-emerald-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-emerald-500/20 light:shadow-emerald-600/30",
        "dark:text-emerald-400 light:text-emerald-700", 
        "dark:hover:border-emerald-500/60 light:hover:border-emerald-600/50",
        "image"),

        ("tga", "TGA", json!(["tga"]),
        "FileTga", 
        "dark:from-rose-500/30 light:from-rose-600/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-rose-500/20 light:shadow-rose-600/30",
        "dark:text-rose-400 light:text-rose-700", 
        "dark:hover:border-rose-500/60 light:hover:border-rose-600/50",
        "image"),

        ("exr", "EXR", json!(["exr"]),
        "FileExr", 
        "dark:from-red-500/30 light:from-red-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-red-500/20 light:shadow-red-600/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "image"),

        ("hdr", "HDR", json!(["hdr"]),
        "FileHdr", 
        "dark:from-amber-500/30 light:from-amber-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-amber-500/20 light:shadow-amber-600/30",
        "dark:text-amber-400 light:text-amber-700", 
        "dark:hover:border-amber-500/60 light:hover:border-amber-600/50",
        "image"),

        ("pnm", "PNM", json!(["pnm", "pgm", "ppm"]),
        "FilePnm", 
        "dark:from-lime-500/30 light:from-lime-600/40 dark:to-green-500/15 light:to-green-600/25", 
        "dark:shadow-lime-500/20 light:shadow-lime-600/30",
        "dark:text-lime-400 light:text-lime-700", 
        "dark:hover:border-lime-500/60 light:hover:border-lime-600/50",
        "image"),

        ("ff", "Farbfeld", json!(["ff"]),
        "FileFarbfeld", 
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
        "FileAac", 
        "dark:from-purple-500/30 light:from-purple-600/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-purple-500/20 light:shadow-purple-600/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50",
        "audio"),

        ("flac", "FLAC", json!(["flac"]),
        "FileFlac", 
        "dark:from-emerald-500/30 light:from-emerald-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-emerald-500/20 light:shadow-emerald-600/30",
        "dark:text-emerald-400 light:text-emerald-700", 
        "dark:hover:border-emerald-500/60 light:hover:border-emerald-600/50",
        "audio"),

        ("ogg", "OGG", json!(["ogg"]),
        "FileOgg", 
        "dark:from-orange-500/30 light:from-orange-600/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-500/20 light:shadow-orange-600/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "audio"),

        ("opus", "OPUS", json!(["opus"]),
        "FileOpus", 
        "dark:from-green-500/30 light:from-green-600/40 dark:to-emerald-500/15 light:to-emerald-600/25", 
        "dark:shadow-green-500/20 light:shadow-green-600/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50",
        "audio"),

        ("wma", "WMA", json!(["wma"]),
        "FileWma", 
        "dark:from-blue-500/30 light:from-blue-600/40 dark:to-indigo-500/15 light:to-indigo-600/25", 
        "dark:shadow-blue-500/20 light:shadow-blue-600/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "audio"),

        ("m4a", "M4A", json!(["m4a"]),
        "FileM4a", 
        "dark:from-cyan-500/30 light:from-cyan-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-cyan-500/20 light:shadow-cyan-600/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "audio"),

        ("aiff", "AIFF", json!(["aiff", "aif", "aifc"]),
        "FileAiff", 
        "dark:from-pink-500/30 light:from-pink-600/40 dark:to-rose-500/15 light:to-rose-600/25", 
        "dark:shadow-pink-500/20 light:shadow-pink-600/30",
        "dark:text-pink-400 light:text-pink-700", 
        "dark:hover:border-pink-500/60 light:hover:border-pink-600/50",
        "audio"),

        ("ac3", "AC3", json!(["ac3"]),
        "FileAc3", 
        "dark:from-indigo-500/30 light:from-indigo-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-500/20 light:shadow-indigo-600/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "audio"),

        ("eac3", "E-AC-3", json!(["eac3"]),
        "FileEac3", 
        "dark:from-violet-500/30 light:from-violet-600/40 dark:to-purple-500/15 light:to-purple-600/25", 
        "dark:shadow-violet-500/20 light:shadow-violet-600/30",
        "dark:text-violet-400 light:text-violet-700", 
        "dark:hover:border-violet-500/60 light:hover:border-violet-600/50",
        "audio"),

        ("tta", "True Audio", json!(["tta"]),
        "FileTta", 
        "dark:from-emerald-500/30 light:from-emerald-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-emerald-500/20 light:shadow-emerald-600/30",
        "dark:text-emerald-400 light:text-emerald-700", 
        "dark:hover:border-emerald-500/60 light:hover:border-emerald-600/50",
        "audio"),

        ("wv", "WavPack", json!(["wv"]),
        "FileWv", 
        "dark:from-cyan-500/30 light:from-cyan-600/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-cyan-500/20 light:shadow-cyan-600/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "audio"),

        ("voc", "Creative Voice", json!(["voc"]),
        "FileVoc", 
        "dark:from-amber-500/30 light:from-amber-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-amber-500/20 light:shadow-amber-600/30",
        "dark:text-amber-400 light:text-amber-700", 
        "dark:hover:border-amber-500/60 light:hover:border-amber-600/50",
        "audio"),

        ("adx", "ADX", json!(["adx"]),
        "FileAdx", 
        "dark:from-fuchsia-500/30 light:from-fuchsia-600/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-fuchsia-500/20 light:shadow-fuchsia-600/30",
        "dark:text-fuchsia-400 light:text-fuchsia-700", 
        "dark:hover:border-fuchsia-500/60 light:hover:border-fuchsia-600/50",
        "audio"),

        ("aptx", "aptX", json!(["aptx"]),
        "FileAptx", 
        "dark:from-blue-500/30 light:from-blue-600/40 dark:to-indigo-500/15 light:to-indigo-600/25", 
        "dark:shadow-blue-500/20 light:shadow-blue-600/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "audio"),

        ("sbc", "SBC", json!(["sbc"]),
        "FileSbc", 
        "dark:from-gray-500/30 light:from-gray-600/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-500/20 light:shadow-gray-600/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-500/60 light:hover:border-gray-600/50",
        "audio"),

        ("caf", "CAF", json!(["caf"]),
        "FileCaf", 
        "dark:from-teal-500/30 light:from-teal-600/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-teal-500/20 light:shadow-teal-600/30",
        "dark:text-teal-400 light:text-teal-700", 
        "dark:hover:border-teal-500/60 light:hover:border-teal-600/50",
        "audio"),

        ("w64", "W64", json!(["w64"]),
        "FileW64", 
        "dark:from-indigo-500/30 light:from-indigo-600/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-500/20 light:shadow-indigo-600/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
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
        "FileAvi", 
        "dark:from-blue-600/30 light:from-blue-700/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-blue-600/20 light:shadow-blue-700/30",
        "dark:text-blue-400 light:text-blue-700", 
        "dark:hover:border-blue-500/60 light:hover:border-blue-600/50",
        "video"),

        ("mkv", "MKV", json!(["mkv"]),
        "FileMkv", 
        "dark:from-purple-600/30 light:from-purple-700/40 dark:to-violet-500/15 light:to-violet-600/25", 
        "dark:shadow-purple-600/20 light:shadow-purple-700/30",
        "dark:text-purple-400 light:text-purple-700", 
        "dark:hover:border-purple-500/60 light:hover:border-purple-600/50",
        "video"),

        ("webm", "WEBM", json!(["webm"]),
        "FileWebm", 
        "dark:from-cyan-600/30 light:from-cyan-700/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-cyan-600/20 light:shadow-cyan-700/30",
        "dark:text-cyan-400 light:text-cyan-700", 
        "dark:hover:border-cyan-500/60 light:hover:border-cyan-600/50",
        "video"),

        ("wmv", "WMV", json!(["wmv"]),
        "FileWmv", 
        "dark:from-indigo-600/30 light:from-indigo-700/40 dark:to-blue-500/15 light:to-blue-600/25", 
        "dark:shadow-indigo-600/20 light:shadow-indigo-700/30",
        "dark:text-indigo-400 light:text-indigo-700", 
        "dark:hover:border-indigo-500/60 light:hover:border-indigo-600/50",
        "video"),

        ("flv", "FLV", json!(["flv"]),
        "FileFlv", 
        "dark:from-orange-600/30 light:from-orange-700/40 dark:to-red-500/15 light:to-red-600/25", 
        "dark:shadow-orange-600/20 light:shadow-orange-700/30",
        "dark:text-orange-400 light:text-orange-700", 
        "dark:hover:border-orange-500/60 light:hover:border-orange-600/50",
        "video"),

        ("3gp", "3GP", json!(["3gp"]),
        "File3gp", 
        "dark:from-green-600/30 light:from-green-700/40 dark:to-teal-500/15 light:to-teal-600/25", 
        "dark:shadow-green-600/20 light:shadow-green-700/30",
        "dark:text-green-400 light:text-green-700", 
        "dark:hover:border-green-500/60 light:hover:border-green-600/50",
        "video"),

        ("m4v", "M4V", json!(["m4v"]),
        "FileM4v", 
        "dark:from-rose-600/30 light:from-rose-700/40 dark:to-pink-500/15 light:to-pink-600/25", 
        "dark:shadow-rose-600/20 light:shadow-rose-700/30",
        "dark:text-rose-400 light:text-rose-700", 
        "dark:hover:border-rose-500/60 light:hover:border-rose-600/50",
        "video"),

        ("ts", "MPEG-TS", json!(["ts", "m2ts"]),
        "FileTs", 
        "dark:from-teal-600/30 light:from-teal-700/40 dark:to-cyan-500/15 light:to-cyan-600/25", 
        "dark:shadow-teal-600/20 light:shadow-teal-700/30",
        "dark:text-teal-400 light:text-teal-700", 
        "dark:hover:border-teal-500/60 light:hover:border-teal-600/50",
        "video"),

        ("vob", "VOB", json!(["vob"]),
        "FileVob", 
        "dark:from-amber-600/30 light:from-amber-700/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-amber-600/20 light:shadow-amber-700/30",
        "dark:text-amber-400 light:text-amber-700", 
        "dark:hover:border-amber-500/60 light:hover:border-amber-600/50",
        "video"),

        ("mpg", "MPEG", json!(["mpg", "mpeg"]),
        "FileMpg", 
        "dark:from-red-500/30 light:from-red-600/40 dark:to-orange-500/15 light:to-orange-600/25", 
        "dark:shadow-red-500/20 light:shadow-red-600/30",
        "dark:text-red-400 light:text-red-700", 
        "dark:hover:border-red-500/60 light:hover:border-red-600/50",
        "video"),

        ("nut", "NUT", json!(["nut"]),
        "FileNut", 
        "dark:from-gray-500/30 light:from-gray-600/40 dark:to-slate-500/15 light:to-slate-600/25", 
        "dark:shadow-gray-500/20 light:shadow-gray-600/30",
        "dark:text-gray-400 light:text-gray-700", 
        "dark:hover:border-gray-500/60 light:hover:border-gray-600/50",
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












#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseConnection, DbErr};

    /// Создает тестовую БД в памяти
    async fn create_test_db() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        
        // Создаем таблицы
        crate::create_tables!(
            db,
            Formats,
            Conversions
        );
        
        Ok(db)
    }

    /// Инициализирует тестовую БД с форматами
    async fn init_test_db() -> Result<DatabaseConnection, String> {
        let db = create_test_db()
            .await
            .map_err(|e| format!("Failed to create test DB: {}", e))?;
        
        init_formats(&db)
            .await
            .map_err(|e| format!("Failed to init formats: {}", e))?;
        
        Ok(db)
    }

    // ============================================================
    // ТЕСТЫ: ИНИЦИАЛИЗАЦИЯ БД
    // ============================================================

    #[tokio::test]
    async fn test_db_init_creates_tables() {
        let db = create_test_db().await.unwrap();
        
        // Проверяем, что таблица formats существует
        let formats_count = Formats::find().count(&db).await.unwrap();
        assert_eq!(formats_count, 0, "Formats table should be empty initially");
        
        // Проверяем, что таблица conversions существует
        let conversions_count = Conversions::find().count(&db).await.unwrap();
        assert_eq!(conversions_count, 0, "Conversions table should be empty initially");
    }

    #[tokio::test]
    async fn test_init_formats_populates_data() {
        let db = create_test_db().await.unwrap();
        init_formats(&db).await.unwrap();
        
        let formats = Formats::find().all(&db).await.unwrap();
        assert!(!formats.is_empty(), "Formats should be populated");
        
        // Проверяем несколько ключевых форматов
        let json = get_format_by_id(&db, "json").await.unwrap().unwrap();
        assert_eq!(json.format_id, "json");
        assert_eq!(json.name, "JSON");
        assert_eq!(json.format_type, "text");
        
        let pdf = get_format_by_id(&db, "pdf").await.unwrap().unwrap();
        assert_eq!(pdf.format_id, "pdf");
        assert_eq!(pdf.name, "PDF");
        assert_eq!(pdf.format_type, "document");
        
        let mp3 = get_format_by_id(&db, "mp3").await.unwrap().unwrap();
        assert_eq!(mp3.format_id, "mp3");
        assert_eq!(mp3.name, "MP3");
        assert_eq!(mp3.format_type, "audio");
        
        let mp4 = get_format_by_id(&db, "mp4").await.unwrap().unwrap();
        assert_eq!(mp4.format_id, "mp4");
        assert_eq!(mp4.name, "MP4");
        assert_eq!(mp4.format_type, "video");
    }

    #[tokio::test]
    async fn test_init_formats_skips_if_exists() {
        let db = create_test_db().await.unwrap();
        
        // Первая инициализация
        init_formats(&db).await.unwrap();
        let count1 = Formats::find().count(&db).await.unwrap();
        
        // Вторая инициализация (должна пропустить)
        init_formats(&db).await.unwrap();
        let count2 = Formats::find().count(&db).await.unwrap();
        
        assert_eq!(count1, count2, "Formats should not be duplicated");
    }

    // ============================================================
    // ТЕСТЫ: CRUD ДЛЯ ФОРМАТОВ
    // ============================================================

    #[tokio::test]
    async fn test_get_all_formats() {
        let db = init_test_db().await.unwrap();
        
        let formats = get_all_formats(&db).await.unwrap();
        assert!(formats.len() > 50, "Should have many formats");
        
        // Проверяем, что есть все типы
        let types: Vec<String> = formats.iter().map(|f| f.format_type.clone()).collect();
        assert!(types.contains(&"text".to_string()));
        assert!(types.contains(&"document".to_string()));
        assert!(types.contains(&"image".to_string()));
        assert!(types.contains(&"audio".to_string()));
        assert!(types.contains(&"video".to_string()));
    }

    #[tokio::test]
    async fn test_get_format_by_id_found() {
        let db = init_test_db().await.unwrap();
        
        let format = get_format_by_id(&db, "json").await.unwrap();
        assert!(format.is_some());
        let format = format.unwrap();
        assert_eq!(format.format_id, "json");
        assert_eq!(format.name, "JSON");
    }

    #[tokio::test]
    async fn test_get_format_by_id_not_found() {
        let db = init_test_db().await.unwrap();
        
        let format = get_format_by_id(&db, "nonexistent").await.unwrap();
        assert!(format.is_none());
    }

    // ============================================================
    // ТЕСТЫ: КОНВЕРТАЦИИ (CRUD)
    // ============================================================

    #[tokio::test]
    async fn test_save_and_find_conversion() {
        let db = init_test_db().await.unwrap();
        
        let hash = "test_hash_123";
        let path = "/tmp/converted/file.pdf";
        
        // Сохраняем конвертацию
        save_conversion(&db, hash, path).await.unwrap();
        
        // Находим
        let found = find_conversion(&db, hash).await;
        assert_eq!(found, Some(path.to_string()));
    }

    #[tokio::test]
    async fn test_save_conversion_duplicate() {
        let db = init_test_db().await.unwrap();
        
        let hash = "test_hash_456";
        let path1 = "/tmp/converted/file1.pdf";
        let path2 = "/tmp/converted/file2.pdf";
        
        // Сохраняем первую
        save_conversion(&db, hash, path1).await.unwrap();
        
        // Сохраняем вторую (дубликат)
        save_conversion(&db, hash, path2).await.unwrap();
        
        // Находим — должен быть только первый
        let found = find_conversion(&db, hash).await;
        assert_eq!(found, Some(path1.to_string()), "Should keep first path");
    }

    #[tokio::test]
    async fn test_find_conversion_not_found() {
        let db = init_test_db().await.unwrap();
        
        let found = find_conversion(&db, "nonexistent_hash").await;
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_conversion_by_path() {
        let db = init_test_db().await.unwrap();
        
        let hash = "test_hash_789";
        let path = "/tmp/converted/file.pdf";
        
        // Сохраняем
        save_conversion(&db, hash, path).await.unwrap();
        
        // Проверяем, что есть
        let found = find_conversion(&db, hash).await;
        assert_eq!(found, Some(path.to_string()));
        
        // Удаляем
        delete_conversion_by_path(&db, path).await.unwrap();
        
        // Проверяем, что удалилось
        let found2 = find_conversion(&db, hash).await;
        assert!(found2.is_none());
    }

    #[tokio::test]
    async fn test_delete_conversion_by_path_not_found() {
        let db = init_test_db().await.unwrap();
        
        // Удаляем несуществующий путь — должно быть Ok
        let result = delete_conversion_by_path(&db, "/nonexistent/path").await;
        assert!(result.is_ok());
    }

    // ============================================================
    // ТЕСТЫ: ИНТЕГРАЦИОННЫЕ
    // ============================================================

    #[tokio::test]
    async fn test_format_types_count() {
        let db = init_test_db().await.unwrap();
        
        let formats = get_all_formats(&db).await.unwrap();
        
        let mut text_count = 0;
        let mut document_count = 0;
        let mut image_count = 0;
        let mut audio_count = 0;
        let mut video_count = 0;
        
        for format in formats {
            match format.format_type.as_str() {
                "text" => text_count += 1,
                "document" => document_count += 1,
                "image" => image_count += 1,
                "audio" => audio_count += 1,
                "video" => video_count += 1,
                _ => {}
            }
        }
        
        // Проверяем, что все типы присутствуют
        assert!(text_count > 0, "Should have text formats");
        assert!(document_count > 0, "Should have document formats");
        assert!(image_count > 0, "Should have image formats");
        assert!(audio_count > 0, "Should have audio formats");
        assert!(video_count > 0, "Should have video formats");
        
        println!("📊 Format counts: text={}, document={}, image={}, audio={}, video={}",
            text_count, document_count, image_count, audio_count, video_count);
    }

    #[tokio::test]
    async fn test_extensions_are_valid_json() {
        let db = init_test_db().await.unwrap();
        
        let formats = get_all_formats(&db).await.unwrap();
        
        for format in formats {
            // Проверяем, что extensions - это валидный JSON массив
            let extensions: Vec<String> = serde_json::from_value(format.extensions)
                .unwrap_or_else(|_| panic!("Invalid extensions JSON for {}", format.format_id));
            
            assert!(!extensions.is_empty(), "Format {} has no extensions", format.format_id);
            
            // Проверяем, что каждый extension - строка
            for ext in extensions {
                assert!(!ext.is_empty(), "Format {} has empty extension", format.format_id);
            }
        }
    }

    #[tokio::test]
    async fn test_all_formats_have_required_fields() {
        let db = init_test_db().await.unwrap();
        
        let formats = get_all_formats(&db).await.unwrap();
        
        for format in formats {
            assert!(!format.format_id.is_empty(), "Format ID is empty");
            assert!(!format.name.is_empty(), "Format name is empty for {}", format.format_id);
            assert!(!format.icon.is_empty(), "Format icon is empty for {}", format.format_id);
            assert!(!format.color.is_empty(), "Format color is empty for {}", format.format_id);
            assert!(!format.glow.is_empty(), "Format glow is empty for {}", format.format_id);
            assert!(!format.text_color.is_empty(), "Format text_color is empty for {}", format.format_id);
            assert!(!format.border_hover.is_empty(), "Format border_hover is empty for {}", format.format_id);
            assert!(!format.format_type.is_empty(), "Format type is empty for {}", format.format_id);
            
            // Проверяем, что format_type валидный
            let valid_types = ["text", "document", "image", "audio", "video"];
            assert!(
                valid_types.contains(&format.format_type.as_str()),
                "Invalid format_type '{}' for format '{}'",
                format.format_type,
                format.format_id
            );
        }
    }

    #[tokio::test]
    async fn test_conversion_timestamps() {
        let db = init_test_db().await.unwrap();
        
        let hash = "test_hash_timestamp";
        let path = "/tmp/converted/timestamp.pdf";
        
        let before = chrono::Utc::now();
        save_conversion(&db, hash, path).await.unwrap();
        let after = chrono::Utc::now();
        
        let found = find_conversion(&db, hash).await.unwrap();
        assert_eq!(found, path);
        
        // Проверяем, что запись создалась с временем
        let record = conversions::Entity::find()
            .filter(conversions::Column::FileHash.eq(hash))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        
        assert!(record.created_at >= before && record.created_at <= after);
    }
}