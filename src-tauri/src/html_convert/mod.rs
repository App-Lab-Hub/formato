use nalgebra::DMatrix;
use serde_json::{Value as Json,json};


use scraper::{Html, ElementRef}; 

pub fn convert_to_html(value: &Json) -> String {
    let (matrix, _rows, _cols) = json_to_matrix(value);
    let table_html = matrix_to_html(&matrix);

    let css_content = include_str!("./styles.css");
    let js_content = include_str!("./script.js");
    let svg_content = include_str!("./favicon.svg");
    let base64_svg = b64_encode(svg_content);

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Formato</title>
<link rel="icon" type="image/svg+xml" href="data:image/svg+xml;base64,{}">
<style>
{}
</style>
</head>
<body>
    <div class="responsive-scroll-container">
        {}
    </div>
<script>
{}
</script>
</body>
</html>"#,
        base64_svg, css_content, table_html, js_content
    );

    // Минификация
    let lines: Vec<&str> = full_html.lines().map(|line| line.trim()).collect();
    let mut minified = lines.join("");
    
    let targets = vec![
        (" { ", "{"), (" } ", "}"), (" : ", ":"), (" ; ", ";"), (", ", ","),
        (" {", "{"), (" }", "}"), (" :", ":"), (" ;", ";"),
        ("{ ", "{"), ("} ", "}"), (": ", ":"), ("; ", ";"),
        (" = ", "="), (" => ", "=>"), (" === ", "===")
    ];
    for (from, to) in targets {
        minified = minified.replace(from, to);
    }

    minified
}


// Маленькая вспомогательная функция для перевода SVG в Base64 без сторонних крейтов
fn b64_encode(input: &str) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::with_capacity((bytes.len() + 2) / 3 * 4);
    
    for chunk in bytes.chunks(3) {
        let mut buffer = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            buffer |= (b as u32) << (24 - i * 8);
        }
        
        let num_encoded = match chunk.len() {
            1 => 2,
            2 => 3,
            _ => 4,
        };
        
        for i in 0..num_encoded {
            let val = ((buffer >> (26 - i * 6)) & 0x3F) as usize;
            result.push(CHARSET[val] as char);
        }
        
        match chunk.len() {
            1 => result.push_str("=="),
            2 => result.push_str("="),
            _ => {}
        }
    }
    result
}




#[derive(Debug, Clone, PartialEq)]
struct Cell {
    name: String,
    colspan: usize,
    rowspan: usize,
    class: String,
}

type Matrix = DMatrix<Option<Cell>>;

fn json_to_matrix(value: &Json) -> (Matrix, usize, usize) {
    let rows = count_rows(value);
    let cols = count_cols(value);
    
    if rows == 0 || cols == 0 {
        return (DMatrix::from_element(1, 1, None), 1, 1);
    }
    
    let mut matrix = DMatrix::from_element(rows, cols, None);
    fill_flat(value, &mut matrix, 0, 0, rows);
    (matrix, rows, cols)
}

fn count_rows(value: &Json) -> usize {
    match value {
        Json::Object(map) => {
            if map.is_empty() { return 0; }
            1 + map.values().map(|v| count_rows(v)).max().unwrap_or(0)
        }
        Json::Array(arr) => {
            if arr.is_empty() { return 0; }
            let non_empty: Vec<_> = arr.iter().filter(|v| count_rows(v) > 0).collect();
            if non_empty.is_empty() { return 0; }
            non_empty.iter().map(|v| count_rows(v)).sum()
        }
        _ => 1,
    }
}

fn count_cols(value: &Json) -> usize {
    match value {
        Json::Object(map) => {
            if map.is_empty() { return 0; }
            map.values().map(|v| count_cols(v)).sum()
        }
        Json::Array(arr) => {
            if arr.is_empty() { return 0; }
            // Фильтруем непустые элементы
            let non_empty: Vec<_> = arr.iter().filter(|v| count_cols(v) > 0).collect();
            if non_empty.is_empty() { return 0; }
            1 + non_empty.iter().map(|v| count_cols(v)).max().unwrap_or(0)
        }
        _ => 1,
    }
}
fn is_primitive_value(v: &Json) -> bool {
    v.is_string() || v.is_number() || v.is_boolean() || v.is_null()
}

fn prim_val(v: &Json) -> (String, &'static str) {
    match v {
        Json::String(s) => (s.clone(), "value-cell s"),
        Json::Number(n) => (n.to_string(), "value-cell n"),
        Json::Bool(b) => (b.to_string(), "value-cell b"),
        Json::Null => ("null".to_string(), "value-cell null-val"),
        _ => (String::new(), "value-cell s"),
    }
}

fn get_header_class(col: usize, extra: &str) -> String {
    if col <= 5 {
        format!("key-cell h-{} {}", col, extra)
    } else {
        format!("key-cell h-generic {}", extra)
    }
}

fn fill_flat(value: &Json, matrix: &mut Matrix, row: usize, col: usize, available_rows: usize) -> usize {
    match value {
        Json::Object(map) => {
            if map.is_empty() { return 0; }
            let mut max_used_rows = 1;
            let mut c = col;
            for (k, v) in map {
                let child_cols = count_cols(v);
                let child_rows = count_rows(v);
                
                let calculated_rowspan = if child_rows <= 1 {
                    if available_rows > 1 { available_rows - 1 } else { 1 }
                } else {
                    1
                };

                matrix[(row, c)] = Some(Cell {
                    name: k.clone(),
                    colspan: child_cols,
                    rowspan: calculated_rowspan,
                    class: get_header_class(col, ""),
                });
                
                let mut current_used = calculated_rowspan;
                if child_rows > 0 {
                    let sub_used = fill_flat(v, matrix, row + calculated_rowspan, c, available_rows - calculated_rowspan);
                    current_used += sub_used;
                }
                if current_used > max_used_rows {
                    max_used_rows = current_used;
                }
                c += child_cols;
            }
            max_used_rows
        }
        Json::Array(arr) => {
            if arr.is_empty() { return 0; }
            
            // Фильтруем только непустые элементы
            let non_empty: Vec<&Json> = arr.iter().filter(|v| count_rows(v) > 0).collect();
            if non_empty.is_empty() { return 0; }
            
            let total_child_rows: usize = arr.iter().map(|v| {
                let r = count_rows(v);
                if r == 0 { 1 } else { r }
            }).sum();

            let mut r = row;
            let mut remaining_available = available_rows;
            let mut remaining_nominal = total_child_rows;
            let mut total_used_rows = 0;

            for (i, v) in arr.iter().enumerate() {
                let current_child_rows = count_rows(v);
                let nominal_item_rows = if current_child_rows == 0 { 1 } else { current_child_rows };

                let allocated_rows = if i == arr.len() - 1 {
                    remaining_available
                } else {
                    if remaining_nominal > 0 {
                        (nominal_item_rows * remaining_available) / remaining_nominal
                    } else {
                        remaining_available / (arr.len() - i)
                    }
                };

                remaining_available -= allocated_rows;
                remaining_nominal -= nominal_item_rows;

                let is_prim = is_primitive_value(v);
                
                if is_prim {
                    matrix[(r, col)] = Some(Cell {
                        name: format!("[{}]", i),
                        colspan: 1,
                        rowspan: 1,
                        class: get_header_class(col, "idx"),
                    });
                    let (val_str, val_class) = prim_val(v);
                    matrix[(r, col + 1)] = Some(Cell {
                        name: val_str,
                        colspan: 1,
                        rowspan: 1,
                        class: val_class.to_string(),
                    });
                    r += 1;
                    total_used_rows += 1;
                } else {
                    matrix[(r, col)] = Some(Cell {
                        name: format!("[{}]", i),
                        colspan: 1,
                        rowspan: 1,
                        class: get_header_class(col, "idx"),
                    });

                    // Рекурсия сообщает нам, сколько РЕАЛЬНЫХ строк занял внутренний контент
                    let child_actual_rows = fill_flat(v, matrix, r, col + 1, allocated_rows);
                    
                    // Мутируем rowspan индекса, делая его строго вровень с контентом
                    if let Some(ref mut cell) = matrix[(r, col)] {
                        cell.rowspan = child_actual_rows;
                    }

                    // Шагаем строго на количество фактически созданных строк (child_actual_rows)
                    r += child_actual_rows;
                    total_used_rows += child_actual_rows;
                }
            }
            total_used_rows
        }
        _ => {
            let (val_str, val_class) = prim_val(value);
            matrix[(row, col)] = Some(Cell {
                name: val_str,
                colspan: 1,
                rowspan: 1,
                class: val_class.to_string(),
            });
            1
        }
    }
}

fn matrix_to_html(matrix: &Matrix) -> String {
    let rows = matrix.nrows();
    let cols = matrix.ncols();
    
    let mut covered = vec![vec![false; cols]; rows];
    let mut html = String::from("<table>\n");
    
    for r in 0..rows {
        html.push_str("  <tr>\n");
        for c in 0..cols {
            if covered[r][c] {
                continue;
            }
            if let Some(ref cell) = matrix[(r, c)] {
                for dr in 0..cell.rowspan {
                    for dc in 0..cell.colspan {
                        if r + dr < rows && c + dc < cols {
                            covered[r + dr][c + dc] = true;
                        }
                    }
                }
                html.push_str(&format!(
                    "    <td class=\"{}\" colspan=\"{}\" rowspan=\"{}\">{}</td>\n",
                    cell.class, cell.colspan, cell.rowspan, cell.name
                ));
            } else {
                html.push_str("    <td></td>\n");
            }
        }
        html.push_str("  </tr>\n");
    }
    html.push_str("</table>\n");
    html
}



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
                let classes: Vec<Json> = attr.1.split_whitespace()
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
            Json::Object(map) => {
                map.get("children")
                    .and_then(|c| c.as_array())
                    .cloned()
                    .unwrap_or_default()
            }
            other => vec![other],
        }
    } else {
        let root = document.root_element();
        match node_to_hast(&root) {
            Json::Object(map) => {
                map.get("children")
                    .and_then(|c| c.as_array())
                    .cloned()
                    .unwrap_or_default()
            }
            other => vec![other],
        }
    };

    Ok(json!({"type": "root", "children": body_children}))
}
