use json2csv::write_json_to_csv;

use serde_json::Value as Json;
use std::io::BufReader;

pub fn parse_csv(input: &str) -> Result<Json, String> {
    let mut reader = csv::Reader::from_reader(input.as_bytes());
    let headers = reader.headers().map_err(|e| format!("CSV: {e}"))?.clone();
    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| format!("CSV: {e}"))?;
        let mut map = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            map.insert(
                headers.get(i).unwrap_or("unknown").to_string(),
                serde_json::Value::String(field.to_string()),
            );
        }
        rows.push(serde_json::Value::Object(map));
    }
    Ok(serde_json::Value::Array(rows))
}

pub fn stringify_csv(value: &Json) -> Result<String, String> {
    let json_str = serde_json::to_string(value).map_err(|e| format!("JSON: {e}"))?;
    let mut output = Vec::new();

    // BOM для Excel UTF-8
    output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);

    write_json_to_csv(
        BufReader::new(json_str.as_bytes()),
        &mut output,
        None,
        Some(",".into()), // точка с запятой
        true,
        None,
        None,
        true,
    )
    .map_err(|e| format!("CSV: {e}"))?;

    String::from_utf8(output).map_err(|e| format!("CSV: {e}"))
}
