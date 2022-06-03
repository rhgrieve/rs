mod format;
mod time;

use std::{
    collections::HashMap,
    fs::{self, Metadata, ReadDir},
    path::{Path},
    process::exit, time::SystemTime, string,
};

use rawrgs::{App, Arg};

use crate::format::{table, TableAlignment};

// Defaults
const DEFAULT_PATH: &str = ".";

// Argument names
const PATH_ARG_NAME: &str = "path";
const ALL_ARG_NAME: &str = "all";
const ALMOST_ALL_ARG_NAME: &str = "almost-all";
const ONE_LINE_ARG_NAME: &str = "one-line";
const LONG_ARG_NAME: &str = "long";

// Separators
const NEW_LINE: &str = "\n";
const ENTRY_SPACE: &str = "  ";

// Directory indicators
const CURRENT_DIR: &str = ".";
const PARENT_DIR: &str = "..";

// Time
const SECS_PER_DAY: u64 = 86400;

struct Options {
    is_show_all: bool,
    is_show_almost_all: bool,
    is_one_line: bool,
    is_long_output: bool,
}

fn process_entries(dir: ReadDir, base_path: &Path, options: Options) -> Result<(), String> {
    let mut dir_entries: Vec<String> = dir
        .into_iter()
        .filter_map(|d| d.ok())
        .map(|d| d.file_name())
        .filter_map(|o| o.into_string().ok())
        .filter(|s| {
            if !options.is_show_all && !options.is_show_almost_all {
                !s.starts_with(CURRENT_DIR)
            } else {
                true
            }
        })
        .collect();

    if options.is_show_all {
        dir_entries.push(String::from(CURRENT_DIR));
        dir_entries.push(String::from(PARENT_DIR));
    }

    let mut metadata_map: HashMap<String, Option<Metadata>> = HashMap::new();
    for dir_entry in dir_entries {
        let local_path = base_path.join(&dir_entry);
        let metadata = fs::metadata(&local_path);
        match metadata {
            Ok(meta) => metadata_map.insert(dir_entry, Some(meta)),
            Err(err) => {
                eprintln!("{}", err);
                metadata_map.insert(dir_entry, None)
            }
        };
    }

    let mut entry_keys_sorted: Vec<String> = metadata_map.keys().cloned().collect();

    entry_keys_sorted.sort();

    let mut output: Vec<Vec<String>> = vec![];
    for key in entry_keys_sorted {
        if let Some(file_metadata) = metadata_map.get(&key) {
            if let Some(meta) = file_metadata {
                let mut string_builder: Vec<String> = vec![];
                
                if options.is_long_output {
                    if let Ok(accessed) = meta.accessed() {
                        let duration = accessed.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                        let days = duration.as_secs() / SECS_PER_DAY;
                        let date = time::SimpleDate::from_days(days);
                        string_builder.push(date.month_display(time::DateFormat::ShortMonth));
                        string_builder.push(date.day());
                    } else {
                        string_builder.push(String::from(" "));
                    }

                    string_builder.push(meta.len().to_string())
                }

                if meta.is_dir() {
                    string_builder.push(format::blue_bold(key))
                } else {
                    string_builder.push(key);
                }

                output.push(string_builder)
            }
        }
    }

    if options.is_one_line || options.is_long_output {
        println!("{}", table(output, 2, TableAlignment::RightLastLeft).unwrap());
    } else {
        println!("{}", output.concat().join(ENTRY_SPACE));
    }

    Ok(())
}

fn run() -> Result<(), String> {
    let app = App::new("rs")
        .about("An ls clone in rust")
        .author("Harrison Grieve")
        .arg(Arg::with_name(PATH_ARG_NAME))
        .arg(Arg::with_name(ALL_ARG_NAME).short("a"))
        .arg(Arg::with_name(ALMOST_ALL_ARG_NAME).short("A"))
        .arg(Arg::with_name(ONE_LINE_ARG_NAME).short("1"))
        .arg(Arg::with_name(LONG_ARG_NAME).short("l"));

    let matches = app.get_matches();

    let base_path = match matches.value_of(PATH_ARG_NAME) {
        Some(path) => Path::new(path),
        None => Path::new(DEFAULT_PATH),
    };

    let options = Options {
        is_show_all: matches.is_present(ALL_ARG_NAME),
        is_show_almost_all: matches.is_present(ALMOST_ALL_ARG_NAME),
        is_one_line: matches.is_present(ONE_LINE_ARG_NAME),
        is_long_output: matches.is_present(LONG_ARG_NAME),
    };

    if let Ok(metadata) = fs::metadata(base_path) {
        if metadata.is_file() {
            println!("{}", base_path.display());
            exit(0);
        }
    }

    return match fs::read_dir(base_path) {
        Ok(read_dir) => process_entries(read_dir, base_path, options),
        Err(err) => Err(format!("rs: cannot access '{}': {}", base_path.display(), err).to_string()),
    };
}

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    })
}
