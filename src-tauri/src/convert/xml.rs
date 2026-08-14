use serde_json::Value as Json;
use xml2json_rs::XmlBuilder;

pub fn parse_xml(input: &str) -> Result<Json, String> {
    let mut reader = quick_xml::Reader::from_str(input);
    let mut buf = Vec::new();
    let mut stack: Vec<serde_json::Value> = Vec::new();
    let mut root: Option<serde_json::Value> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(_)) => {
                stack.push(serde_json::Value::Object(serde_json::Map::new()));
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if !text.trim().is_empty() {
                    if let Some(obj) = stack.last_mut() {
                        *obj = serde_json::Value::String(text);
                    }
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if let Some(val) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                        if let Some(obj) = parent.as_object_mut() {
                            obj.insert(name, val);
                        }
                    } else {
                        root = Some(val);
                    }
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(format!("XML: {e}")),
            _ => {}
        }
        buf.clear();
    }
    Ok(root.unwrap_or(serde_json::Value::Null))
}

pub fn stringify_xml(value: &Json) -> Result<String, String> {
    let json_str = serde_json::to_string(value).map_err(|e| format!("JSON: {e}"))?;
    XmlBuilder::default()
        .build_from_json_string(&json_str)
        .map_err(|e| format!("XML: {e}"))
}
