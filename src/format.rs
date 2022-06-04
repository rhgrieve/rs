use std::collections::HashMap;

// Escape codes
const ESCAPE_BLUE_BOLD: &str = "\x1b[34;1m";
const ESCAPE_RESET: &str = "\x1b[0m";

// Bytes
const KB_IN_BYTES: f64 = 1024.0;
const MB_IN_BYTES: f64 = 1048576.0;
const GB_IN_BYTES: f64 = 1073741824.0;
const TB_IN_BYTES: f64 = 1099511627776.0;

pub fn blue_bold(str: &String) -> String {
    return format!("\x1b[34;1m{}\x1b[0m", str);
}

pub fn bytes_to_human_readable(bytes: u64) -> String {
    let mut num = bytes as f64;
    let label: &str;
    if num >= KB_IN_BYTES && num < MB_IN_BYTES {
        num = num / KB_IN_BYTES;
        label = "K";
    } else if num >= MB_IN_BYTES && num < GB_IN_BYTES {
        num = num / MB_IN_BYTES;
        label = "M";
    } else if num >= GB_IN_BYTES && num < TB_IN_BYTES {
        num = num / GB_IN_BYTES;
        label = "G"
    } else {
        return format!("{}", num);
    }

    return format!("{:.1}{}", num, label);
}

// This is horrible!! 
// But to fix it we need to refactor the metadata logic :[
fn unescaped_length(str: &String) -> usize {
    str
        .replace(ESCAPE_BLUE_BOLD, "")
        .replace(ESCAPE_RESET, "")
        .to_string()
        .len()
}

pub enum TableAlignment {
    Left,
    Right,
    RightLastLeft
}

fn pad_right(input: String, length: &usize) -> String {
    let mut padded_string = String::from(input);
    
    if unescaped_length(&padded_string) == *length {
        return padded_string
    }

    let spaces_to_add = length - unescaped_length(&padded_string);
    for _ in 0..spaces_to_add {
        padded_string.push(' ');
    }

    return padded_string;
}

fn pad_left(input: String, length: &usize) -> String {
    let mut padded_string = String::new();
    
    if unescaped_length(&input) == *length {
        return input
    }

    let spaces_to_add = length - unescaped_length(&input);
    for _ in 0..spaces_to_add {
        padded_string.push(' ');
    }

    padded_string.push_str(input.as_str());

    return padded_string;
}

fn col_max_size_map(input_data: &Vec<Vec<String>>) -> HashMap<usize, usize> {
    let num_rows = input_data.len();
    let num_cols = input_data[0].len();
    let mut col_max_size_map: HashMap<usize, usize> = HashMap::new();
    for row in 0..num_rows {
        for col in 0..num_cols {
            let current_max = col_max_size_map.get(&col).unwrap_or(&0);
            if unescaped_length(&input_data[row][col]) > *current_max {
                col_max_size_map.insert(col, unescaped_length(&input_data[row][col]));
            }
        }
    }
    return col_max_size_map;
}

fn validate_table_equality(input_data: &Vec<Vec<String>>, num_rows: usize, num_cols: usize) -> Result<(), &'static str> {
    for row in 0..num_rows {
        if input_data[row].len() != num_cols {
            return Err("All rows must have the same number of columns");
        }
    }
    Ok(())
}

pub fn table(mut input_data: Vec<Vec<String>>, col_size: usize, align: TableAlignment) -> Result<String, &'static str> {
    let num_rows = input_data.len();
    let num_cols = input_data[0].len();

    // Validate
    validate_table_equality(&input_data, num_rows, num_cols)?;

    let col_max_size_map = col_max_size_map(&input_data);

    let mut output_string: Vec<String> = vec![];
    for row in 0..num_rows {
        for col in 0..num_cols {
            if let Some(max_length) = col_max_size_map.get(&col) {
                input_data[row][col] = match align {
                    TableAlignment::Left => pad_right(input_data[row][col].clone(), max_length),
                    TableAlignment::Right => pad_left(input_data[row][col].clone(), max_length),
                    TableAlignment::RightLastLeft => {
                        if col == num_cols - 1 {
                            pad_right(input_data[row][col].clone(), max_length)
                        } else {
                            pad_left(input_data[row][col].clone(), max_length)
                        }
                    }
                }
            }
        }
        output_string.push(input_data[row].join(String::from(" ").repeat(col_size).as_str()));
    }

    Ok(output_string.join("\n"))
}