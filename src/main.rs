use std::{
    collections::HashMap,
    fs::{self, Metadata, ReadDir},
    path::{Path},
    process::exit,
};

use rawrgs::{App, Arg};

// Defaults
const DEFAULT_PATH: &str = ".";

// Argument names
const PATH_ARG_NAME: &str = "path";
const ALL_ARG_NAME: &str = "all";
const ALMOST_ALL_ARG_NAME: &str = "almost-all";
const ONE_LINE_ARG_NAME: &str = "one-line";

// Separators
const NEW_LINE: &str = "\n";
const ENTRY_SPACE: &str = "  ";

// Directory indicators
const CURRENT_DIR: &str = ".";
const PARENT_DIR: &str = "..";

struct Options {
    is_show_all: bool,
    is_show_almost_all: bool,
    is_one_line: bool,
}

fn blue_bold(str: String) -> String {
    return format!("\x1b[34;1m{}\x1b[0m", str);
}

fn process_entries(dir: ReadDir, base_path: &Path, opts: Options) -> Result<(), String> {
    let mut dir_entries: Vec<String> = dir
        .into_iter()
        .filter_map(|d| d.ok())
        .map(|d| d.file_name())
        .filter_map(|o| o.into_string().ok())
        .filter(|s| {
            if !opts.is_show_all && !opts.is_show_almost_all {
                !s.starts_with(CURRENT_DIR)
            } else {
                true
            }
        })
        .collect();

    if opts.is_show_all {
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

    let mut output: Vec<String> = vec![];
    for key in entry_keys_sorted {
        if let Some(file_metadata) = metadata_map.get(&key) {
            if let Some(meta) = file_metadata {
                if meta.is_dir() {
                    output.push(blue_bold(key))
                } else {
                    output.push(key);
                }
            }
        }
    }

    let join_str = match opts.is_one_line {
        true => NEW_LINE,
        false => ENTRY_SPACE,
    };

    println!("{}", output.join(join_str));
    Ok(())
}

fn run() -> Result<(), String> {
    let app = App::new("rs")
        .about("An ls clone in rust")
        .author("Harrison Grieve")
        .arg(Arg::with_name(PATH_ARG_NAME))
        .arg(Arg::with_name(ALL_ARG_NAME).short("a"))
        .arg(Arg::with_name(ALMOST_ALL_ARG_NAME).short("A"))
        .arg(Arg::with_name(ONE_LINE_ARG_NAME).short("1"));

    let matches = app.get_matches();

    let base_path = match matches.value_of(PATH_ARG_NAME) {
        Some(path) => Path::new(path),
        None => Path::new(DEFAULT_PATH),
    };

    let options = Options {
        is_show_all: matches.is_present(ALL_ARG_NAME),
        is_show_almost_all: matches.is_present(ALMOST_ALL_ARG_NAME),
        is_one_line: matches.is_present(ONE_LINE_ARG_NAME),
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
