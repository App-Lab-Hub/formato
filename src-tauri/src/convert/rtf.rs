// src/convert/rtf.rs

use shiva::core::{Document, TransformerTrait};
use bytes::Bytes;
use serde_json::{Value as Json};
use shiva::rtf::Transformer as RtfTransformer;


pub fn parse_rtf(input: &str) -> Result<Json, String> {
    let input_bytes = Bytes::from(input.as_bytes().to_vec());
    let document = RtfTransformer::parse(&input_bytes)
        .map_err(|e| format!("RTF parse error: {}", e))?;
    
    let json_value = serde_json::to_value(&document)
        .map_err(|e| format!("Serialize to JSON error: {}", e))?;
    
    Ok(json_value)
}

pub fn stringify_rtf(value: &Json) -> Result<String, String> {
    let json_str = value.to_string();
    let parsed = parse_rtf(&json_str)?;
    
    let document: Document = serde_json::from_value(parsed)
        .map_err(|e| format!("Deserialize to Document error: {}", e))?;
    
    let rtf_bytes = RtfTransformer::generate(&document)
        .map_err(|e| format!("RtfTransformer generate error: {}", e))?;
    
    String::from_utf8(rtf_bytes.to_vec())
        .map_err(|e| format!("UTF-8 conversion error: {}", e))
}