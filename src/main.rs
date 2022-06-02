use std::{
    path::Path,
    collections::HashMap,
    fs::{self, Metadata},
};

use rawrgs::{App, Arg};

fn blue_bold(str: String) -> String {
    return format!("\x1b[34;1m{}\x1b[0m", str);
}

fn main() {
    let app = App::new("rs")
        .about("An ls clone in rust")
        .author("Harrison Grieve")
        .arg(Arg::with_name("path"))
        .arg(Arg::with_name("all").short("a"))
        .arg(Arg::with_name("almost-all").short("A"))
        .arg(Arg::with_name("one-line").short("1"));

    let matches = app.get_matches();

    let path = match matches.value_of("path") {
        Some(path) => Path::new(path),
        None => panic!(),
    };

    // OPTIONS
    let is_show_all = matches.is_present("all");
    let is_show_almost_all = matches.is_present("almost-all");
    let is_one_line = matches.is_present("one-line");

    let dir = fs::read_dir(path).expect("Failed to read directory");
    let mut dir_entries: Vec<String> = dir
        .into_iter()
        .filter_map(|d| d.ok())
        .map(|d| d.file_name())
        .filter_map(|o| o.into_string().ok())
        .filter(|s| {
            if !is_show_all && !is_show_almost_all {
                !s.starts_with(".")
            } else {
                true
            }
        })
        .collect();

    if is_show_all {
        dir_entries.push(String::from("."));
        dir_entries.push(String::from(".."));
    }

    let mut metadata_map: HashMap<String, Option<Metadata>> = HashMap::new();
    for dir_entry in dir_entries {
        let local_path = path.join(&dir_entry);
        let metadata = fs::metadata(&local_path);
        match metadata {
            Ok(meta) => metadata_map.insert(dir_entry, Some(meta)),
            Err(err) => {
                eprintln!("{}", err);
                metadata_map.insert(dir_entry, None)
            }
        };
    }

    let mut entry_keys_sorted: Vec<String> = metadata_map
        .keys()
        .cloned()
        .collect();

    entry_keys_sorted.sort();

    let mut output: Vec<String> = vec!();
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

    let join_str = match is_one_line {
        true => "\n",
        false => "  ",
    };

    println!("{}", output.join(join_str));
}
