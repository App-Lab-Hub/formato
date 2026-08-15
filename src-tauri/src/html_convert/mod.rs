use serde_json::{json, Value as Json};

use scraper::{ElementRef, Html};

pub fn parse_html(input: &str) -> Result<Json, String> {
    let document = Html::parse_document(input);

    fn node_to_hast(element: &ElementRef) -> Json {
        let tag_name = element.value().name().to_lowercase();

        // Собираем properties (атрибуты)
        let mut properties = serde_json::Map::new();
        for attr in element.value().attrs() {
            let key = match attr.0 {
                "class" => "className".to_string(),
                "for" => "htmlFor".to_string(),
                "tabindex" => "tabIndex".to_string(),
                "onclick" => "onClick".to_string(),
                "onchange" => "onChange".to_string(),
                "oninput" => "onInput".to_string(),
                _ => attr.0.to_string(),
            };

            if key == "className" {
                // class → массив строк
                let classes: Vec<Json> = attr
                    .1
                    .split_whitespace()
                    .map(|c| Json::String(c.to_string()))
                    .collect();
                properties.insert(key, Json::Array(classes));
            } else if attr.1.is_empty() {
                // Булевы атрибуты (disabled, checked и т.д.)
                properties.insert(key, Json::Bool(true));
            } else {
                properties.insert(key, Json::String(attr.1.to_string()));
            }
        }

        // Собираем детей
        let mut children: Vec<Json> = Vec::new();
        for child in element.children() {
            match child.value() {
                scraper::Node::Text(text) => {
                    let trimmed = text.text.trim();
                    if !trimmed.is_empty() {
                        children.push(json!({"type": "text", "value": trimmed}));
                    }
                }
                scraper::Node::Comment(comment) => {
                    children.push(json!({"type": "comment", "value": comment.trim()}));
                }
                scraper::Node::Element(_) => {
                    if let Some(el) = ElementRef::wrap(child) {
                        children.push(node_to_hast(&el));
                    }
                }
                _ => {}
            }
        }

        let mut map = serde_json::Map::new();
        map.insert("type".to_string(), Json::String("element".to_string()));
        map.insert("tagName".to_string(), Json::String(tag_name));

        if !properties.is_empty() {
            map.insert("properties".to_string(), Json::Object(properties));
        }
        if !children.is_empty() {
            map.insert("children".to_string(), Json::Array(children));
        }

        Json::Object(map)
    }

    let body_sel = scraper::Selector::parse("body").unwrap();

    let body_children: Vec<Json> = if let Some(body) = document.select(&body_sel).next() {
        match node_to_hast(&body) {
            Json::Object(map) => map
                .get("children")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default(),
            other => vec![other],
        }
    } else {
        let root = document.root_element();
        match node_to_hast(&root) {
            Json::Object(map) => map
                .get("children")
                .and_then(|c| c.as_array())
                .cloned()
                .unwrap_or_default(),
            other => vec![other],
        }
    };

    Ok(json!({"type": "root", "children": body_children}))
}
