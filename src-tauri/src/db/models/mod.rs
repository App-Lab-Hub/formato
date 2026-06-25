// src-tauri/src/models/mod.rs

pub mod formats;
pub mod conversions;

// Переэкспорт для удобства
pub use formats::Entity as Formats;
pub use formats::Model as FormatModel;
pub use formats::ActiveModel as FormatActiveModel;

pub use conversions::Entity as Conversions;
pub use conversions::Model as ConversionModel;
pub use conversions::ActiveModel as ConversionActiveModel;