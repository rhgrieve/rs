mod format;
mod time;
mod user;

use std::{
    cmp::Ordering,
    fmt,
    fs::{self, Metadata, ReadDir},
    os::{linux::fs::MetadataExt, unix::prelude::PermissionsExt},
    path::{Path, PathBuf},
    process::exit,
    time::SystemTime,
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
const ENTRY_SPACE: &str = "  ";
const TABLE_COL_SIZE: usize = 1;

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

struct RSEntry {
    name: String,
    path: PathBuf,
    metadata: Option<Metadata>,
}

impl RSEntry {
    fn get_permission_string(&self) -> String {
        let mut permission_string = String::new();
        if let Some(file_metadata) = &self.metadata {
            let mut permission_string_prefix = String::new();
            if file_metadata.is_dir() {
                permission_string_prefix.push_str("d");
            } else if file_metadata.is_file() {
                permission_string_prefix.push_str("-");
            } else if file_metadata.is_symlink() {
                permission_string_prefix.push_str("l");
            }

            let mode = file_metadata.permissions().mode();
            let mode_string = format!("{:o}", mode);
            let permission_bits = mode_string[mode_string.len() - 3..].to_string();
            permission_string = String::from(permission_string_prefix);
            for bit in permission_bits.chars() {
                match bit {
                    '4' => permission_string.push_str("r--"),
                    '5' => permission_string.push_str("r-x"),
                    '6' => permission_string.push_str("rw-"),
                    '7' => permission_string.push_str("rwx"),
                    _ => continue,
                }
            }
        }
        return permission_string;
    }

    fn get_file_size(&self) -> u64 {
        if let Some(file_metadata) = &self.metadata {
            return file_metadata.len();
        }
        0
    }

    fn get_table_row(&self, is_long_output: bool) -> Vec<String> {
        let mut string_builder: Vec<String> = vec![];
        if let Some(ref file_metadata) = &self.metadata {
            if is_long_output {
                // permission string
                let permission_string = self.get_permission_string();
                string_builder.push(permission_string);

                // number of hardlinks
                string_builder.push(file_metadata.st_nlink().to_string());

                // owner
                if let Ok(user_name) = user::get_by_uid(file_metadata.st_uid()) {
                    string_builder.push(user_name)
                } else {
                    string_builder.push("?".to_string());
                }

                // group
                if let Ok(group_name) = user::group_by_gid(file_metadata.st_gid()) {
                    string_builder.push(group_name);
                } else {
                    string_builder.push("?".to_string());
                }

                // last modified time
                if let Ok(accessed) = file_metadata.modified() {
                    let duration = accessed.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                    let days = duration.as_secs() / SECS_PER_DAY;
                    let date = time::SimpleDate::from_days(days);
                    string_builder.push(date.month_display(time::DateFormat::ShortMonth));
                    string_builder.push(date.day());
                } else {
                    string_builder.push(String::from(" "));
                }

                // file size
                let file_size = &self.get_file_size();
                string_builder.push(file_size.to_string())
            }

            if file_metadata.is_dir() {
                string_builder.push(format::blue_bold(&self.name))
            } else {
                string_builder.push(self.name.to_string());
            }
        }
        string_builder
    }
}

impl Ord for RSEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for RSEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for RSEntry {}

impl PartialEq for RSEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl fmt::Display for RSEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(meta) = &self.metadata {
            if meta.is_dir() {
                write!(f, "{}", format::blue_bold(&self.name))?;
            } else {
                write!(f, "{}", self.name)?;
            }
        } else {
            write!(f, "{}", self.name)?;
        }

        Ok(())
    }
}

fn get_entries(dir_entries: Vec<String>, base_path: &Path) -> Vec<RSEntry> {
    let mut rs_entries: Vec<RSEntry> = vec![];
    for dir_entry in dir_entries {
        let local_path = base_path.join(&dir_entry);
        let metadata = fs::metadata(&local_path);
        match metadata {
            Ok(meta) => rs_entries.push(RSEntry {
                name: dir_entry,
                path: local_path,
                metadata: Some(meta),
            }),
            Err(err) => {
                eprintln!("{}", err);
                rs_entries.push(RSEntry {
                    name: dir_entry,
                    path: local_path,
                    metadata: None,
                });
            }
        }
    }
    rs_entries
}

fn get_dir_entries(dir: ReadDir, options: &Options) -> Vec<String> {
    return dir
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
}

fn get_tabular_entries(rs_entries: Vec<RSEntry>, options: &Options) -> Vec<Vec<String>> {
    let mut output: Vec<Vec<String>> = vec![];
    for entry in rs_entries {
        let row = entry.get_table_row(options.is_long_output);
        output.push(row);
    }
    output
}

fn process_entries(dir: ReadDir, base_path: &Path, options: Options) -> Result<(), String> {
    let mut dir_entries = get_dir_entries(dir, &options);

    if options.is_show_all {
        dir_entries.push(String::from(CURRENT_DIR));
        dir_entries.push(String::from(PARENT_DIR));
    }

    let mut rs_entries = get_entries(dir_entries, base_path);
    rs_entries.sort();

    let tabular_entries = get_tabular_entries(rs_entries, &options);

    if options.is_one_line || options.is_long_output {
        let table = table(
            tabular_entries,
            TABLE_COL_SIZE,
            TableAlignment::RightLastLeft,
        )
        .unwrap();
        println!("{}", table);
    } else {
        println!("{}", tabular_entries.concat().join(ENTRY_SPACE));
    }

    Ok(())
}

fn run() -> Result<(), String> {
    let app = App::new("rs")
        .about("An ls clone in rust")
        .author("Harrison Grieve")
        .version(env!("CARGO_PKG_VERSION"))
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
        Err(err) => {
            Err(format!("rs: cannot access '{}': {}", base_path.display(), err).to_string())
        }
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
