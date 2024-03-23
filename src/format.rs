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
    format!("\x1b[34;1m{}\x1b[0m", str)
}

pub fn bytes_to_human_readable(bytes: u64) -> String {
    let mut num = bytes as f64;
    let label: &str;
    if (KB_IN_BYTES..MB_IN_BYTES).contains(&num) {
        num /= KB_IN_BYTES;
        label = "K";
    } else if (MB_IN_BYTES..GB_IN_BYTES).contains(&num) {
        num /= MB_IN_BYTES;
        label = "M";
    } else if (GB_IN_BYTES..TB_IN_BYTES).contains(&num) {
        num /= GB_IN_BYTES;
        label = "G"
    } else {
        return format!("{}", num);
    }

    format!("{:.1}{}", num, label)
}

// This is horrible!!
// But to fix it we need to refactor the metadata logic :[
fn unescaped_length(str: &str) -> usize {
    str
        .replace(ESCAPE_BLUE_BOLD, "")
        .replace(ESCAPE_RESET, "")
        .to_string()
        .len()
}

pub enum TableAlignment {
    // Left,
    // Right,
    RightLastLeft
}

fn pad_right(input: String, length: &usize) -> String {
    let mut padded_string = input;

    if unescaped_length(&padded_string) == *length {
        return padded_string
    }

    let spaces_to_add = length - unescaped_length(&padded_string);
    for _ in 0..spaces_to_add {
        padded_string.push(' ');
    }

    padded_string
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

    padded_string
}

fn col_max_size_map(input_data: &[Vec<String>]) -> HashMap<usize, usize> {
    let mut col_max_size_map: HashMap<usize, usize> = HashMap::new();
    for row in input_data {
        for (index, col) in row.iter().enumerate() {
            let current_max = col_max_size_map.get(&index).unwrap_or(&0);
            if unescaped_length(col) > *current_max {
                col_max_size_map.insert(index, unescaped_length(col));
            }
        }
    }
    col_max_size_map
}

fn validate_table_equality(input_data: &Vec<Vec<String>>, num_cols: usize) -> Result<(), &'static str> {
    for row in input_data {
        if row.len() != num_cols {
            return Err("All rows must have the same number of columns");
        }
    }
    Ok(())
}

pub fn table(input_data: Vec<Vec<String>>, align: TableAlignment) -> Result<String, &'static str> {
    let num_cols = input_data[0].len();

    // Validate
    validate_table_equality(&input_data, num_cols)?;

    let col_max_size_map = col_max_size_map(&input_data);
    let output_string = input_data
        .iter()
        .map(|row| {
            return row
                .iter()
                .enumerate()
                .map(|(index, col)| {
                    if let Some(max_length) = col_max_size_map.get(&index) {
                        return match align {
                            // TableAlignment::Left => pad_right(col.to_string(), max_length),
                            // TableAlignment::Right => pad_left(col.to_string(), max_length),
                            TableAlignment::RightLastLeft => {
                                if index == num_cols - 1 {
                                    pad_right(col.clone(), max_length)
                                } else {
                                    pad_left(col.clone(), max_length)
                                }
                            }
                        };
                    }

                    col.to_string()
                })
                .collect::<Vec<String>>()
                .join(" ");
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(output_string)
}