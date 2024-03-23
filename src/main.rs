mod format;
mod time;
mod user;

use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    fs::{self, Metadata, ReadDir},
    io::IsTerminal,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
    process::exit,
    time::SystemTime,
};

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;

#[cfg(target_os = "macos")]
use std::os::macos::fs::MetadataExt;

#[cfg(target_os = "unix")]
use std::os::unix::fs::MetadataExt;

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
const NUMERIC_UID_GID_ARG_NAME: &str = "numeric-uid-gid";
const HUMAN_READABLE_ARG_NAME: &str = "human-readable";
const GROUP_DIRECTORIES_FIRST_ARG_NAME: &str = "group-directories-first";
const IGNORE_BACKUPS_ARG_NAME: &str = "ignore-backups";
const TIME_SORT_ARG_NAME: &str = "sort-time";
const SIZE_SORT_ARG_NAME: &str = "sort-size";
const EXT_SORT_ARG_NAME: &str = "sort-extension";
const REVERSE_ARG_NAME: &str = "reverse";
const SIZE_ARG_NAME: &str = "size";
const ACCESS_TIME_ARG_NAME: &str = "access-time";
const INODE_ARG_NAME: &str = "inode";
const KIBIBYTES_ARG_NAME: &str = "kibibytes";
const COMMA_SEPARATED_ARG_NAME: &str = "comma-separated";

// Separators
const ENTRY_SPACE: &str = "  ";

// Directory indicators
const CURRENT_DIR: &str = ".";
const PARENT_DIR: &str = "..";

// Time
const SECS_PER_DAY: u64 = 86400;

// Size
const MB_BYTES: u64 = 1024;

enum RSSort {
    Time,
    AccessTime,
    Size,
    Directory,
    Extension,
    Default,
}

struct RSEntries {
    entries: Vec<RSEntry>,
    block_size: u64,
}

impl RSEntries {
    fn sort_by(&mut self, kind: RSSort) {
        self.entries.sort_by(|a, b| {
            if let (Some(meta_a), Some(meta_b)) = (&a.metadata, &b.metadata) {
                return match kind {
                    RSSort::Directory => meta_b.is_dir().cmp(&meta_a.is_dir()),
                    RSSort::Time => meta_b.st_mtime().cmp(&meta_a.st_mtime()),
                    RSSort::AccessTime => meta_b.st_atime().cmp(&meta_a.st_atime()),
                    RSSort::Size => meta_b.len().cmp(&meta_a.len()),
                    RSSort::Extension => {
                        return match (a.path.extension(), b.path.extension()) {
                            (Some(ext_a), Some(ext_b)) => ext_a.cmp(ext_b),
                            (Some(_), None) => Ordering::Greater,
                            (None, Some(_)) => Ordering::Less,
                            (None, None) => a.cmp(b),
                        };
                    }
                    RSSort::Default => a.cmp(b),
                };
            }
            a.cmp(b)
        })
    }

    fn reverse(&mut self) {
        self.entries.reverse();
    }

    fn to_tabular(&self, options: &Options) -> Vec<Vec<String>> {
        let mut output: Vec<Vec<String>> = vec![];
        for entry in &self.entries {
            let row = entry.get_table_row(options);
            output.push(row);
        }
        output
    }
}

struct Options {
    is_show_all: bool,
    is_show_almost_all: bool,
    is_one_line: bool,
    is_long_output: bool,
    is_numeric_uid_gid: bool,
    is_human_readable: bool,
    is_group_directories_first: bool,
    is_ignore_backups: bool,
    is_sort_by_time: bool,
    is_sort_by_size: bool,
    is_sort_by_extension: bool,
    is_sort_reverse: bool,
    is_show_size_blocks: bool,
    is_access_time: bool,
    is_show_inode: bool,
    is_kibibytes: bool,
    is_comma_separated: bool,
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
                permission_string_prefix.push('d');
            } else if file_metadata.is_file() {
                permission_string_prefix.push('-');
            } else if file_metadata.is_symlink() {
                permission_string_prefix.push('l');
            } else {
                permission_string_prefix.push('?');
            }

            let mode = file_metadata.permissions().mode();
            let mode_string = format!("{:o}", mode);
            let permission_bits = mode_string[mode_string.len() - 3..].to_string();
            permission_string = permission_string_prefix;
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
        permission_string
    }

    fn get_file_size(&self) -> u64 {
        if let Some(file_metadata) = &self.metadata {
            return file_metadata.len();
        }
        0
    }

    fn get_file_size_human(&self) -> String {
        let mut human_readable_string = String::new();
        if let Some(file_metadata) = &self.metadata {
            human_readable_string = format::bytes_to_human_readable(file_metadata.len())
        }
        human_readable_string
    }

    fn get_table_row(&self, options: &Options) -> Vec<String> {
        let mut string_builder: Vec<String> = vec![];
        if let Some(ref file_metadata) = &self.metadata {
            // size blocks
            if options.is_show_size_blocks {
                let mut blocks = file_metadata.st_blocks();
                if options.is_kibibytes {
                    blocks /= 2;
                }
                string_builder.push((blocks).to_string())
            }

            // index node
            if options.is_show_inode {
                // TODO: handle other platforms
                if cfg!(target_os = "macos") {
                    string_builder.push(file_metadata.st_ino().to_string())
                }
            }

            if options.is_long_output || options.is_numeric_uid_gid {
                // permission string
                let permission_string = self.get_permission_string();
                string_builder.push(permission_string);

                // number of hardlinks
                string_builder.push(file_metadata.st_nlink().to_string());

                // owner
                let uid_string = match options.is_numeric_uid_gid {
                    true => file_metadata.st_uid().to_string(),
                    false => {
                        if let Ok(user_name) = user::get_by_uid(file_metadata.st_uid()) {
                            user_name
                        } else {
                            "?".to_string()
                        }
                    }
                };
                string_builder.push(uid_string);

                // group
                let gid_string = match options.is_numeric_uid_gid {
                    true => file_metadata.st_gid().to_string(),
                    false => {
                        if let Ok(group_name) = user::group_by_gid(file_metadata.st_gid()) {
                            group_name
                        } else {
                            "?".to_string()
                        }
                    }
                };
                string_builder.push(gid_string);

                // file size
                let file_size_string = match options.is_human_readable {
                    true => self.get_file_size_human(),
                    false => self.get_file_size().to_string(),
                };
                string_builder.push(file_size_string);

                let mut time_to_parse = file_metadata.modified();
                if options.is_access_time {
                    time_to_parse = file_metadata.accessed();
                }

                // last modified time
                if let Ok(system_time) = time_to_parse {
                    let duration = system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
                    let days = duration.as_secs() / SECS_PER_DAY;
                    let date = time::SimpleDate::from_days(days);
                    string_builder.push(date.month_display(time::DateFormat::ShortMonth));
                    string_builder.push(date.day());
                } else {
                    string_builder.push(String::from(" "));
                }
            }

            if file_metadata.is_dir() && std::io::stdout().is_terminal() {
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
            if meta.is_dir() && std::io::stdout().is_terminal() {
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

impl Borrow<str> for RSEntry {
    fn borrow(&self) -> &str {
        &self.name
    }
}

fn get_entries(dir_entries: Vec<String>, base_path: &Path) -> RSEntries {
    let mut block_size = 0;
    let mut rs_entries: Vec<RSEntry> = vec![];
    for dir_entry in dir_entries {
        let local_path = base_path.join(&dir_entry);
        let metadata = fs::metadata(&local_path);
        match metadata {
            Ok(meta) => {
                block_size += meta.st_blksize() / MB_BYTES;
                rs_entries.push(RSEntry {
                    name: dir_entry,
                    path: local_path,
                    metadata: Some(meta),
                })
            }
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
    RSEntries {
        entries: rs_entries,
        block_size,
    }
}

fn get_dir_entries(dir: ReadDir, options: &Options) -> Vec<String> {
    dir.into_iter()
        .filter_map(|d| d.ok())
        .map(|d| d.file_name())
        .filter_map(|o| o.into_string().ok())
        .filter(|s| {
            (options.is_show_all || options.is_show_almost_all) || !s.starts_with(CURRENT_DIR)
        })
        .filter(|s| !(options.is_ignore_backups && s.ends_with('~')))
        .collect()
}

// fn get_tabular_entries(rs_entries: RSEntries, options: &Options) -> Vec<Vec<String>> {
//     let mut output: Vec<Vec<String>> = vec![];
//     for entry in rs_entries.entries {
//         let row = entry.get_table_row(options);
//         output.push(row);
//     }
//     output
// }

fn process_entries(dir: ReadDir, base_path: &Path, options: Options) -> Result<(), String> {
    let mut dir_entries = get_dir_entries(dir, &options);

    if options.is_show_all {
        dir_entries.push(String::from(CURRENT_DIR));
        dir_entries.push(String::from(PARENT_DIR));
    }

    let mut rs_entries = get_entries(dir_entries, base_path);

    let sort_type = match options {
        Options {
            is_group_directories_first: true,
            ..
        } => RSSort::Directory,
        Options {
            is_sort_by_size: true,
            ..
        } => RSSort::Size,
        Options {
            is_access_time: true,
            is_long_output: true,
            is_sort_by_time: true,
            ..
        } => RSSort::AccessTime,
        Options {
            is_sort_by_time: true,
            ..
        } => RSSort::Time,
        Options {
            is_sort_by_extension: true,
            ..
        } => RSSort::Extension,
        _ => RSSort::Default,
    };

    rs_entries.sort_by(sort_type);
    if options.is_sort_reverse {
        rs_entries.reverse();
    }

    if options.is_one_line || options.is_long_output || options.is_numeric_uid_gid {
        let table = table(
            rs_entries.to_tabular(&options),
            TableAlignment::RightLastLeft,
        )
        .unwrap();
        if !options.is_one_line {
            println!("total {}", rs_entries.block_size);
        }
        println!("{}", table);
    } else if options.is_comma_separated {
        // TODO: figure out how to handle coloured folders
        println!("{}", rs_entries.entries.join(", "))
    } else {
        println!(
            "{}",
            rs_entries.to_tabular(&options).concat().join(ENTRY_SPACE)
        );
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
        .arg(Arg::with_name(LONG_ARG_NAME).short("l"))
        .arg(Arg::with_name(NUMERIC_UID_GID_ARG_NAME).short("n"))
        .arg(Arg::with_name(HUMAN_READABLE_ARG_NAME).short("H"))
        .arg(
            Arg::with_name(GROUP_DIRECTORIES_FIRST_ARG_NAME).long(GROUP_DIRECTORIES_FIRST_ARG_NAME),
        )
        .arg(Arg::with_name(IGNORE_BACKUPS_ARG_NAME).short("B"))
        .arg(Arg::with_name(TIME_SORT_ARG_NAME).short("t"))
        .arg(Arg::with_name(SIZE_ARG_NAME).short("s"))
        .arg(Arg::with_name(SIZE_SORT_ARG_NAME).short("S"))
        .arg(Arg::with_name(EXT_SORT_ARG_NAME).short("X"))
        .arg(Arg::with_name(REVERSE_ARG_NAME).short("r"))
        .arg(Arg::with_name(ACCESS_TIME_ARG_NAME).short("u"))
        .arg(
            Arg::with_name(INODE_ARG_NAME)
                .short("i")
                .long(INODE_ARG_NAME),
        )
        .arg(
            Arg::with_name(KIBIBYTES_ARG_NAME)
                .short("k")
                .long(KIBIBYTES_ARG_NAME),
        )
        .arg(Arg::with_name(COMMA_SEPARATED_ARG_NAME).short("m"));

    let matches = app.get_matches();

    let options = Options {
        is_show_all: matches.is_present(ALL_ARG_NAME),
        is_show_almost_all: matches.is_present(ALMOST_ALL_ARG_NAME),
        is_one_line: matches.is_present(ONE_LINE_ARG_NAME),
        is_long_output: matches.is_present(LONG_ARG_NAME),
        is_numeric_uid_gid: matches.is_present(NUMERIC_UID_GID_ARG_NAME),
        is_human_readable: matches.is_present(HUMAN_READABLE_ARG_NAME),
        is_group_directories_first: matches.is_present(GROUP_DIRECTORIES_FIRST_ARG_NAME),
        is_ignore_backups: matches.is_present(IGNORE_BACKUPS_ARG_NAME),
        is_sort_by_time: matches.is_present(TIME_SORT_ARG_NAME),
        is_sort_by_size: matches.is_present(SIZE_SORT_ARG_NAME),
        is_sort_by_extension: matches.is_present(EXT_SORT_ARG_NAME),
        is_sort_reverse: matches.is_present(REVERSE_ARG_NAME),
        is_show_size_blocks: matches.is_present(SIZE_ARG_NAME),
        is_access_time: matches.is_present(ACCESS_TIME_ARG_NAME),
        is_show_inode: matches.is_present(INODE_ARG_NAME),
        is_kibibytes: matches.is_present(KIBIBYTES_ARG_NAME),
        is_comma_separated: matches.is_present(COMMA_SEPARATED_ARG_NAME),
    };

    let base_path = match matches.value_of(PATH_ARG_NAME) {
        Some(path) => Path::new(path),
        None => Path::new(DEFAULT_PATH),
    };

    if let Ok(metadata) = fs::metadata(base_path) {
        if metadata.is_file() {
            println!("{}", base_path.display());
            exit(0);
        }
    }

    match fs::read_dir(base_path) {
        Ok(read_dir) => process_entries(read_dir, base_path, options),
        Err(err) => {
            Err(format!("rs: cannot access '{}': {}", base_path.display(), err).to_string())
        }
    }
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
