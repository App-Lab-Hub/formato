use handlebars::{
    Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};
use serde_json::{json, Value as Json};

const MD_CSS: &str = include_str!("../../assets/md_styles.css");

const ENTRY_TEMPLATE: &str = "{{{md _value _key _depth}}}";

#[derive(Clone, Copy)]
struct MdHelper;

impl HelperDef for MdHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars<'reg>,
        _: &handlebars::Context,
        _: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        let value = h
            .param(0)
            .ok_or_else(|| RenderErrorReason::ParamNotFoundForIndex("md", 0))?
            .value();
        let key = h.param(1).and_then(|p| p.value().as_str()).unwrap_or("");
        let depth: usize = h.param(2).and_then(|p| p.value().as_u64()).unwrap_or(0) as usize;

        let indent = "> ".repeat(depth);
        let is_root = depth == 0 && key.is_empty();

        match value {
            Json::Object(obj) if obj.is_empty() => {
                if !key.is_empty() {
                    write!(out, "{}**{}** `{{}}`\n", indent, key)?;
                }
            }
            Json::Array(arr) if arr.is_empty() => {
                if !key.is_empty() {
                    write!(out, "{}**{}** `[]`\n", indent, key)?;
                }
            }
            Json::Object(obj) => {
                if !key.is_empty() {
                    write!(out, "{}### {}\n", indent, key)?;
                }
                let field_indent = if is_root {
                    String::new()
                } else {
                    format!("{}> ", indent)
                };
                let next_depth = if is_root { 0 } else { depth + 1 };
                for (k, v) in obj {
                    if k.starts_with('_') {
                        continue;
                    }
                    match v {
                        Json::Object(_) | Json::Array(_) => {
                            out.write(&render_entry(r, v, k, next_depth))?;
                        }
                        _ => write!(
                            out,
                            "{}**{}** {}\n",
                            field_indent,
                            k,
                            format_primitive_md(v)
                        )?,
                    }
                }
            }
            Json::Array(arr) if all_primitive(arr) => {
                if !key.is_empty() {
                    write!(out, "{}**{}**\n", indent, key)?;
                    let item_indent = format!("{}> ", indent);
                    for (i, item) in arr.iter().enumerate() {
                        write!(
                            out,
                            "{}- [{}] {}\n",
                            item_indent,
                            i,
                            format_primitive_md(item)
                        )?;
                    }
                } else {
                    let items: Vec<String> = arr.iter().map(format_primitive_md).collect();
                    write!(out, "{}", items.join(" "))?;
                }
            }
            Json::Array(arr) => {
                if !key.is_empty() {
                    write!(out, "{}**{}**\n", indent, key)?;
                }
                let item_indent = format!("{}> ", indent);
                for (i, item) in arr.iter().enumerate() {
                    write!(out, "{}## [{}]\n", item_indent, i)?;
                    match item {
                        Json::Object(obj) => {
                            let field_indent = format!("{}> ", item_indent);
                            for (k, v) in obj {
                                if k.starts_with('_') {
                                    continue;
                                }
                                match v {
                                    Json::Object(_) | Json::Array(_) => {
                                        out.write(&render_entry(r, v, k, depth + 2))?;
                                    }
                                    _ => write!(
                                        out,
                                        "{}**{}** {}\n",
                                        field_indent,
                                        k,
                                        format_primitive_md(v)
                                    )?,
                                }
                            }
                        }
                        Json::Array(_) => out.write(&render_entry(r, item, "", depth + 2))?,
                        _ => write!(out, "{}{}\n", item_indent, format_primitive_md(item))?,
                    }
                    if i < arr.len() - 1 {
                        write!(out, "{}---\n", item_indent)?;
                    }
                }
            }
            _ => {
                let s = format_primitive_md(value);
                if key.is_empty() {
                    write!(out, "{}", s)?;
                } else {
                    write!(out, "{}**{}** {}\n", indent, key, s)?;
                }
            }
        }
        Ok(())
    }
}

fn render_entry(reg: &Handlebars, value: &Json, key: &str, depth: usize) -> String {
    let mut params = serde_json::Map::new();
    params.insert("_value".to_string(), value.clone());
    params.insert("_key".to_string(), Json::String(key.to_string()));
    params.insert("_depth".to_string(), Json::Number(depth.into()));
    let ctx = Json::Object(params);
    reg.render_template(ENTRY_TEMPLATE, &ctx)
        .unwrap_or_else(|e| format!("*error: {}*", e))
}

fn all_primitive(arr: &[Json]) -> bool {
    arr.iter()
        .all(|v| v.is_string() || v.is_number() || v.is_boolean() || v.is_null())
}

fn format_primitive_md(v: &Json) -> String {
    match v {
        Json::String(s) => format!("`{}`", s.replace('`', "\\`").replace('*', "\\*")),
        Json::Number(n) => format!("`{}`", n),
        Json::Bool(b) => format!("**`{}`**", b),
        Json::Null => "*null*".to_string(),
        _ => unreachable!(),
    }
}

pub fn stringify_markdown(value: &Json) -> Result<String, String> {
    let mut reg = Handlebars::new();
    reg.register_escape_fn(handlebars::no_escape);
    reg.register_helper("md", Box::new(MdHelper));

    let result = match value {
        Json::Object(_) | Json::Array(_) => render_entry(&reg, value, "", 0),
        _ => format_primitive_md(value),
    };

    // Стили вставляем здесь, в начало результата
    Ok(format!(
        "<span style=\"display:none\"></span>\n\n<style>{}</style>\n\n{}",
        MD_CSS,
        result.trim()
    ))
}

pub fn parse_markdown(input: &str) -> Result<Json, String> {
    let parser = pulldown_cmark::Parser::new_ext(input, pulldown_cmark::Options::all());
    let events: Vec<pulldown_cmark::Event> = parser.collect();

    let mut stack: Vec<(String, Vec<Json>, serde_json::Map<String, Json>)> = Vec::new();
    let mut root_children: Vec<Json> = Vec::new();
    let mut current_text = String::new();

    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<Json> = Vec::new();
    let mut table_cells: Vec<String> = Vec::new();

    fn flush_text(text: &mut String, target: &mut Vec<Json>) {
        let t = text.trim().to_string();
        if !t.is_empty() {
            target.push(json!({"type": "text", "value": t}));
        }
        text.clear();
    }

    fn make_node(
        node_type: &str,
        children: Vec<Json>,
        extra: serde_json::Map<String, Json>,
    ) -> Json {
        let mut map = extra;
        map.insert("type".to_string(), Json::String(node_type.to_string()));
        if !children.is_empty() {
            map.insert("children".to_string(), Json::Array(children));
        }
        Json::Object(map)
    }

    fn make_text(value: &str) -> Json {
        json!({"type": "text", "value": value})
    }

    for event in &events {
        match event {
            pulldown_cmark::Event::Start(tag) => {
                flush_text(
                    &mut current_text,
                    if let Some((_, children, _)) = stack.last_mut() {
                        children
                    } else {
                        &mut root_children
                    },
                );

                match tag {
                    pulldown_cmark::Tag::Heading { level, .. } => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("depth".to_string(), Json::Number((*level as u64).into()));
                        stack.push(("heading".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::Paragraph => {
                        stack.push(("paragraph".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::BlockQuote(_) => {
                        stack.push(("blockquote".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::CodeBlock(kind) => {
                        let mut attrs = serde_json::Map::new();
                        if let pulldown_cmark::CodeBlockKind::Fenced(lang) = kind {
                            if !lang.is_empty() {
                                attrs.insert("lang".to_string(), Json::String(lang.to_string()));
                            }
                        }
                        stack.push(("code".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::List(ordered) => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("ordered".to_string(), Json::Bool(ordered.is_some()));
                        if let Some(start) = ordered {
                            attrs.insert("start".to_string(), Json::Number((*start).into()));
                        }
                        stack.push(("list".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::Item => {
                        stack.push(("listItem".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::Table(_) => {
                        in_table = true;
                        table_rows.clear();
                        // headers не чистим — они перезапишутся в TableHead
                    }
                    pulldown_cmark::Tag::TableHead => {
                        table_cells.clear();
                    }
                    pulldown_cmark::Tag::TableRow => {
                        table_cells.clear();
                    }
                    pulldown_cmark::Tag::TableCell => {}
                    pulldown_cmark::Tag::Emphasis => {
                        stack.push(("emphasis".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::Strong => {
                        stack.push(("strong".to_string(), Vec::new(), serde_json::Map::new()));
                    }
                    pulldown_cmark::Tag::Link {
                        link_type: _,
                        dest_url,
                        title,
                        id: _,
                    } => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("url".to_string(), Json::String(dest_url.to_string()));
                        if !title.is_empty() {
                            attrs.insert("title".to_string(), Json::String(title.to_string()));
                        }
                        stack.push(("link".to_string(), Vec::new(), attrs));
                    }
                    pulldown_cmark::Tag::Image {
                        link_type: _,
                        dest_url,
                        title,
                        id: _,
                    } => {
                        let mut attrs = serde_json::Map::new();
                        attrs.insert("url".to_string(), Json::String(dest_url.to_string()));
                        if !title.is_empty() {
                            attrs.insert("title".to_string(), Json::String(title.to_string()));
                        }
                        stack.push(("image".to_string(), Vec::new(), attrs));
                    }
                    _ => {}
                }
            }
            pulldown_cmark::Event::End(tag_end) => {
                flush_text(
                    &mut current_text,
                    if let Some((_, children, _)) = stack.last_mut() {
                        children
                    } else {
                        &mut root_children
                    },
                );

                match tag_end {
                    pulldown_cmark::TagEnd::Heading(_)
                    | pulldown_cmark::TagEnd::Paragraph
                    | pulldown_cmark::TagEnd::BlockQuote(_)
                    | pulldown_cmark::TagEnd::CodeBlock
                    | pulldown_cmark::TagEnd::List(_)
                    | pulldown_cmark::TagEnd::Item
                    | pulldown_cmark::TagEnd::Emphasis
                    | pulldown_cmark::TagEnd::Strong
                    | pulldown_cmark::TagEnd::Link
                    | pulldown_cmark::TagEnd::Image => {
                        if let Some((node_type, children, extra)) = stack.pop() {
                            let node = make_node(&node_type, children, extra);
                            if let Some((_, parent_children, _)) = stack.last_mut() {
                                parent_children.push(node);
                            } else {
                                root_children.push(node);
                            }
                        }
                    }
                    pulldown_cmark::TagEnd::Table => {
                        in_table = false;
                        let mut table_children: Vec<Json> = Vec::new();

                        // Первая строка — заголовок с пометкой header: true
                        if !table_headers.is_empty() {
                            let mut header_attrs = serde_json::Map::new();
                            header_attrs.insert("header".to_string(), Json::Bool(true));
                            let header_cells: Vec<Json> = table_headers
                                .iter()
                                .map(|h| {
                                    make_node(
                                        "tableCell",
                                        vec![make_text(h)],
                                        serde_json::Map::new(),
                                    )
                                })
                                .collect();
                            table_children.push(make_node("tableRow", header_cells, header_attrs));
                        }

                        // Строки данных
                        for row in &table_rows {
                            if let Json::Object(cells) = row {
                                let cell_nodes: Vec<Json> = cells
                                    .values()
                                    .map(|v| {
                                        make_node(
                                            "tableCell",
                                            vec![make_text(v.as_str().unwrap_or(""))],
                                            serde_json::Map::new(),
                                        )
                                    })
                                    .collect();
                                table_children.push(make_node(
                                    "tableRow",
                                    cell_nodes,
                                    serde_json::Map::new(),
                                ));
                            }
                        }

                        root_children.push(make_node(
                            "table",
                            table_children,
                            serde_json::Map::new(),
                        ));
                    }
                    pulldown_cmark::TagEnd::TableHead => {
                        // Сохраняем заголовки и очищаем
                        table_headers = std::mem::take(&mut table_cells);
                    }
                    pulldown_cmark::TagEnd::TableRow => {
                        let mut row = serde_json::Map::new();
                        for (i, cell) in table_cells.iter().enumerate() {
                            let key = if table_headers.is_empty() {
                                format!("col{}", i)
                            } else {
                                table_headers
                                    .get(i)
                                    .cloned()
                                    .unwrap_or_else(|| format!("col{}", i))
                            };
                            row.insert(key, Json::String(cell.clone()));
                        }
                        table_rows.push(Json::Object(row));
                    }
                    _ => {}
                }
            }
            pulldown_cmark::Event::Text(text) => {
                if in_table {
                    table_cells.push(text.to_string());
                } else {
                    current_text.push_str(text);
                }
            }
            pulldown_cmark::Event::Code(code) => {
                if in_table {
                    table_cells.push(format!("`{}`", code));
                } else {
                    let node = json!({"type": "inlineCode", "value": code.to_string()});
                    if let Some((_, children, _)) = stack.last_mut() {
                        children.push(node);
                    } else {
                        root_children.push(node);
                    }
                }
            }
            pulldown_cmark::Event::SoftBreak | pulldown_cmark::Event::HardBreak => {
                current_text.push(' ');
            }
            _ => {}
        }
    }

    flush_text(&mut current_text, &mut root_children);

    if root_children.is_empty() {
        Ok(json!({"type": "root", "children": []}))
    } else {
        Ok(json!({"type": "root", "children": root_children}))
    }
}
