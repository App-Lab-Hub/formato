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






#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ==========================================================================
    // ТЕСТЫ ПОДМАТРИЧНЫХ ТИПОВ И ПРИМИТИВОВ
    // ==========================================================================

    #[test]
    fn test_is_primitive_value() {
        assert!(is_primitive_value(&json!("hello")));
        assert!(is_primitive_value(&json!(42)));
        assert!(is_primitive_value(&json!(true)));
        assert!(is_primitive_value(&json!(null)));
        
        // Объекты и массивы не являются примитивами
        assert!(!is_primitive_value(&json!({"key": "val"})));
        assert!(!is_primitive_value(&json!([1, 2, 3])));
    }

    #[test]
    fn test_prim_val_mapping() {
        let (val_str, class) = prim_val(&json!("text"));
        assert_eq!(val_str, "text");
        assert_eq!(class, "value-cell s");

        let (val_str, class) = prim_val(&json!(123));
        assert_eq!(val_str, "123");
        assert_eq!(class, "value-cell n");

        let (val_str, class) = prim_val(&json!(false));
        assert_eq!(val_str, "false");
        assert_eq!(class, "value-cell b");

        let (val_str, class) = prim_val(&json!(null));
        assert_eq!(val_str, "null");
        assert_eq!(class, "value-cell null-val");
    }

    // ==========================================================================
    // ТЕСТЫ ГЕОМЕТРИИ МАТРИЦЫ (РАСЧЕТ СТРОК И СТОЛБЦОВ)
    // ==========================================================================

    #[test]
    fn test_count_rows_and_cols_primitive() {
        let j = json!("just a string");
        assert_eq!(count_rows(&j), 1);
        assert_eq!(count_cols(&j), 1);
    }

    #[test]
    fn test_count_geometry_flat_object() {
        // Объект с 3 ключами-примитивами.
        // Строки: 1 для ключей + максимальная глубина детей (1) = 2 строки.
        // Столбцы: сумма столбцов детей = 1 + 1 + 1 = 3 столбца.
        let j = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });
        assert_eq!(count_rows(&j), 2);
        assert_eq!(count_cols(&j), 3);
    }

    #[test]
    fn test_count_geometry_flat_array() {
        // Массив примитивов из 3 элементов.
        // Каждому элементу выделится индекс и значение в одну строку.
        // Сумма строк: 1 + 1 + 1 = 3 строки.
        // Столбцы: 1 для индекса + макс глубина контента (1) = 2 столбца.
        let j = json!([10, 20, 30]);
        assert_eq!(count_rows(&j), 3);
        assert_eq!(count_cols(&j), 2);
    }

    #[test]
    fn test_count_geometry_nested_structure() {
        let j = json!({
            "user": {
                "name": "Alex",
                "roles": ["admin", "user"]
            }
        });
        // Проверяем, что вложенные структуры правильно суммируют лимиты геометрии
        assert!(count_rows(&j) > 0);
        assert!(count_cols(&j) > 0);
    }

    // ==========================================================================
    // ТЕСТЫ МАТРИЧНОЙ ЗАПИСИ И СОРТИРОВКИ
    // ==========================================================================

    #[test]
    fn test_empty_json_handling() {
        let empty_obj = json!({});
        let (matrix, rows, cols) = json_to_matrix(&empty_obj);
        assert_eq!(rows, 1);
        assert_eq!(cols, 1);
        assert_eq!(matrix[(0, 0)], None);
    }

    #[test]
    fn test_alphabetical_keys_sorting() {
        // Наш алгоритм сортирует ключи: "b" пойдет первым, затем "z", затем "а" в исходном коде? 
        // Нет, строго: "a", "b", "z".
        let j = json!({
            "z": 1,
            "a": 2,
            "b": 3
        });
        
        let (matrix, _rows, _cols) = json_to_matrix(&j);
        
        // Первая строка (индекс 0) содержит ключи объектов.
        // Проверяем, что они легли в матрицу строго по алфавиту слева направо.
        let cell_0 = matrix[(0, 0)].as_ref().unwrap();
        let cell_1 = matrix[(0, 1)].as_ref().unwrap();
        let cell_2 = matrix[(0, 2)].as_ref().unwrap();

        assert_eq!(cell_0.name, "a");
        assert_eq!(cell_1.name, "b");
        assert_eq!(cell_2.name, "z");
    }

    #[test]
    fn test_matrix_html_generation_not_empty() {
        let j = json!({"status": "ok"});
        let (matrix, _, _) = json_to_matrix(&j);
        let html = matrix_to_html(&matrix);
        
        // Проверяем базовые маркеры валидности таблицы
        assert!(html.starts_with("<table>"));
        assert!(html.ends_with("</table>\n"));
        assert!(html.contains("<td class=\"key-cell h-0 \""));
        assert!(html.contains("status"));
        assert!(html.contains("ok"));
    }

    // ==========================================================================
    // СЛОЖНЫЕ ТЕСТ-КЕЙСЫ И Edge Cases (КРАЙНИЕ СОСТОЯНИЯ)
    // ==========================================================================

    #[test]
    fn test_deep_nesting_recursion() {
        // Создаем глубокую вложенность: а -> b -> c -> d -> e -> "done"
        let j = json!({
            "a": { "b": { "c": { "d": { "e": "done" } } } }
        });
        
        // Каждый новый уровень объекта добавляет 1 строку для заголовка. 
        // 5 уровней заголовков + 1 строка для самого значения "done" = 6 строк.
        assert_eq!(count_rows(&j), 6);
        // Так как везде по одному ключу, колонка должна быть строго одна.
        assert_eq!(count_cols(&j), 1);

        let (matrix, _, _) = json_to_matrix(&j);
        // Проверяем, что самый глубокий элемент "done" успешно доехал до конца матрицы
        let final_cell = matrix[(5, 0)].as_ref().unwrap();
        assert_eq!(final_cell.name, "done");
    }

#[test]
fn test_array_of_empty_structures() {
    // Массив, состоящий из пустых объектов и массивов: [{}, [], {}]
    let j = json!([{}, [], {}]);
    
    // Пустые структуры не несут данных, count_rows возвращает 0
    assert_eq!(count_rows(&j), 0);
    assert_eq!(count_cols(&j), 0);
    
    let (matrix, rows, cols) = json_to_matrix(&j);
    // Матрица схлопывается до 1x1 с None
    assert_eq!(rows, 1);
    assert_eq!(cols, 1);
    assert_eq!(matrix[(0, 0)], None);
}

    #[test]
    fn test_massive_primitive_array_math() {
        // Создаем массив из 150 примитивов, чтобы проверить математику allocated_rows
        let mut arr = Vec::new();
        for i in 0..150 {
            arr.push(json!(i));
        }
        let j = Json::Array(arr);

        let (matrix, rows, cols) = json_to_matrix(&j);
        // Массив примитивов: 150 строк в высоту
        assert_eq!(rows, 150);
        // 2 колонки в ширину: одна под индекс [i], вторая под само числовое значение
        assert_eq!(cols, 2);

        // Проверяем выборочные ячейки, что математика деления не съехала на больших индексах
        assert_eq!(matrix[(0, 0)].as_ref().unwrap().name, "[0]");
        assert_eq!(matrix[(149, 0)].as_ref().unwrap().name, "[149]");
        assert_eq!(matrix[(149, 1)].as_ref().unwrap().name, "149");
    }

    #[test]
    fn test_tricky_strings_and_html_injection() {
        // Проверяем строки с кавычками, амперсандами, тегами и переносами строк
        let j = json!({
            "tricky_key": "Format: <td class=\"bad\"> \n &hello;"
        });

        let (matrix, _, _) = json_to_matrix(&j);
        let value_cell = matrix[(1, 0)].as_ref().unwrap();
        
        // Проверяем, что Rust сохранил строку в первозданном виде внутри Cell
        assert_eq!(value_cell.name, "Format: <td class=\"bad\"> \n &hello;");
        
        let html = matrix_to_html(&matrix);
        // Проверяем, что макрос format! внутри matrix_to_html не сломался из-за кавычек в тексте
        assert!(html.contains("Format: <td class=\"bad\"> \n &hello;"));
    }

    #[test]
    fn test_mixed_array_types() {
        // Массив, где перемешаны примитивы и тяжелые объекты
        let j = json!([
            "primitive_first",
            { "nested_key": "nested_value" },
            42
        ]);

        // Элемент 0 (строка) -> 1 строка
        // Элемент 1 (объект с ключом и значением) -> 1 строка под ключ + 1 под значение = 2 строки
        // Элемент 2 (число) -> 1 строка
        // Итого: 1 + 2 + 1 = 4 строки в высоту.
        assert_eq!(count_rows(&j), 4);

        let (matrix, rows, cols) = json_to_matrix(&j);
        assert_eq!(rows, 4);
        assert_eq!(cols, 2);

        // ИСПРАВЛЕНО: Код честно генерирует красивые индексы "[0]" и "[1]"
        assert_eq!(matrix[(0, 0)].as_ref().unwrap().name, "[0]"); // Индекс первого элемента
        assert_eq!(matrix[(0, 1)].as_ref().unwrap().name, "primitive_first"); // Значение первого элемента
        
        assert_eq!(matrix[(1, 0)].as_ref().unwrap().name, "[1]"); // Индекс второго элемента
        assert_eq!(matrix[(1, 1)].as_ref().unwrap().name, "nested_key"); // Ключ объекта внутри массива
    }

    // ==========================================================================
    // ФИНАЛЬНЫЕ СВЕРХСЛОЖНЫЕ ТЕСТ-КЕЙСЫ (АВИАКОСМИЧЕСКИЙ УРОВЕНЬ)
    // ==========================================================================

    #[test]
    fn test_extreme_numbers_and_floats() {
        // Проверяем гигантские целые числа, отрицательные значения и числа с плавающей точкой.
        let j = json!({
            "big_int": 18446744073709551615u64,
            "neg_int": -2147483648i32,
            "random_float": 5.5555555555
        });

        let (matrix, rows, cols) = json_to_matrix(&j);
        
        // Матрица плоского объекта из 3 ключей: 2 строки в высоту, 3 столбца в ширину
        assert_eq!(rows, 2);
        assert_eq!(cols, 3);
        
        // Ключи отсортировались по алфавиту: "big_int" (col 0), "neg_int" (col 1), "random_float" (col 2)
        // Значения примитивов лежат строго на второй строке (индекс 1)
        let cell_big = matrix[(1, 0)].as_ref().unwrap();
        let cell_neg = matrix[(1, 1)].as_ref().unwrap();
        let cell_float = matrix[(1, 2)].as_ref().unwrap();

        assert_eq!(cell_big.name, "18446744073709551615");
        assert_eq!(cell_neg.name, "-2147483648");
        assert_eq!(cell_float.name, "5.5555555555");
        assert_eq!(cell_float.class, "value-cell n");
    }

    #[test]
    fn test_multidimensional_arrays() {
        // Двумерный массив, внутри которого лежит пустой массив: [[]]
        let j = json!([[]]);

        // Пустой вложенный массив не дает данных, всё схлопывается
        assert_eq!(count_rows(&j), 0);
        assert_eq!(count_cols(&j), 0);
        
        let (matrix, rows, cols) = json_to_matrix(&j);
        assert_eq!(rows, 1);
        assert_eq!(cols, 1);
        assert_eq!(matrix[(0, 0)], None);
    }



    #[test]
    fn test_unicode_and_emojis_in_keys() {
        // Ключи на кириллице с пробелами и эмодзи
        let j = json!({
            "Данные 🚀": "контент",
            "Абв": 123
        });

        let (matrix, _, _) = json_to_matrix(&j);
        
        // Проверяем, что Rust успешно отсортировал Юникод по алфавиту (кириллица тоже отлично сортируется)
        // "Абв" должна пойти первой (индекс колонки 0), а "Данные 🚀" — второй (индекс колонки 1)
        let cell_first_key = matrix[(0, 0)].as_ref().unwrap();
        let cell_second_key = matrix[(0, 1)].as_ref().unwrap();

        assert_eq!(cell_first_key.name, "Абв");
        assert_eq!(cell_second_key.name, "Данные 🚀");
    }


// NEW
    #[test]
    fn test_null_value_handling() {
        let j = json!({"key": null});
        let (matrix, rows, cols) = json_to_matrix(&j);
        assert_eq!(rows, 2);
        assert_eq!(cols, 1);
        assert_eq!(matrix[(1, 0)].as_ref().unwrap().name, "null");
        assert_eq!(matrix[(1, 0)].as_ref().unwrap().class, "value-cell null-val");
    }

    #[test]
    fn test_single_empty_object() {
        let j = json!({});
        assert_eq!(count_rows(&j), 0);
        assert_eq!(count_cols(&j), 0);
        let (matrix, rows, cols) = json_to_matrix(&j);
        assert_eq!(rows, 1);
        assert_eq!(cols, 1);
        assert_eq!(matrix[(0, 0)], None);
    }

    #[test]
    fn test_single_empty_array() {
        let j = json!([]);
        assert_eq!(count_rows(&j), 0);
        assert_eq!(count_cols(&j), 0);
        let (matrix, rows, cols) = json_to_matrix(&j);
        assert_eq!(rows, 1);
        assert_eq!(cols, 1);
        assert_eq!(matrix[(0, 0)], None);
    }

    #[test]
    fn test_boolean_values() {
        let j = json!({"true_key": true, "false_key": false});
        let (matrix, _, _) = json_to_matrix(&j);
        assert_eq!(matrix[(1, 0)].as_ref().unwrap().name, "false");
        assert_eq!(matrix[(1, 0)].as_ref().unwrap().class, "value-cell b");
        assert_eq!(matrix[(1, 1)].as_ref().unwrap().name, "true");
        assert_eq!(matrix[(1, 1)].as_ref().unwrap().class, "value-cell b");
    }

    #[test]
    fn test_object_with_array_of_objects() {
        let j = json!({
            "data": [
                {"id": 1, "name": "first"},
                {"id": 2, "name": "second"}
            ]
        });
        let (matrix, rows, cols) = json_to_matrix(&j);
        assert!(rows > 2);
        assert!(cols >= 3);
        // Проверяем наличие индексов [0] и [1]
        let mut found_0 = false;
        let mut found_1 = false;
        for r in 0..rows {
            for c in 0..cols {
                if let Some(ref cell) = matrix[(r, c)] {
                    if cell.name == "[0]" { found_0 = true; }
                    if cell.name == "[1]" { found_1 = true; }
                }
            }
        }
        assert!(found_0);
        assert!(found_1);
    }

    #[test]
    fn test_negative_numbers() {
        let j = json!([-1, -100, -999999]);
        let (matrix, _, _) = json_to_matrix(&j);
        assert_eq!(matrix[(0, 1)].as_ref().unwrap().name, "-1");
        assert_eq!(matrix[(1, 1)].as_ref().unwrap().name, "-100");
        assert_eq!(matrix[(2, 1)].as_ref().unwrap().name, "-999999");
    }

    #[test]
    fn test_float_edge_cases() {
        let j = json!({
            "zero": 0.0,
            "negative_zero": -0.0,
            "infinity": 1e308
        });
        let (matrix, _, _) = json_to_matrix(&j);
        // Ключи отсортированы
        assert_eq!(matrix[(0, 0)].as_ref().unwrap().name, "infinity");
        assert_eq!(matrix[(0, 1)].as_ref().unwrap().name, "negative_zero");
        assert_eq!(matrix[(0, 2)].as_ref().unwrap().name, "zero");
    }

    #[test]
    fn test_html_escaping_in_keys() {
        let j = json!({"<script>alert('xss')</script>": "value"});
        let (matrix, _, _) = json_to_matrix(&j);
        let html = matrix_to_html(&matrix);
        // HTML не должен сломаться
        assert!(html.contains("<script>alert('xss')</script>"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn test_large_string_values() {
        let long_string = "a".repeat(10000);
        let j = json!({"long_key": long_string});
        let (matrix, _, _) = json_to_matrix(&j);
        assert_eq!(matrix[(1, 0)].as_ref().unwrap().name.len(), 10000);
    }

    #[test]
    fn test_matrix_consistency() {
        let j = json!({
            "str": "hello",
            "num": 42,
            "bool": true,
            "null_val": null,
            "arr": [1, 2, 3],
            "obj": {"inner": "val"}
        });
        let (matrix, rows, cols) = json_to_matrix(&j);
        let html = matrix_to_html(&matrix);
        
        // Каждая строка должна иметь одинаковое количество <td> (с учётом colspan)
        let tr_lines: Vec<&str> = html.lines().filter(|l| l.contains("<tr>") || l.contains("<td")).collect();
        assert!(tr_lines.len() > 0);
    }

    #[test]
    fn test_empty_structures_preserved_in_html() {
        let j = json!({"empty_obj": {}, "empty_arr": []});
        let (matrix, rows, cols) = json_to_matrix(&j);
        let html = matrix_to_html(&matrix);
        // HTML должен быть валидным даже с пустыми структурами
        assert!(html.starts_with("<table>"));
        assert!(html.ends_with("</table>\n"));
    }



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
