use std::{fs, error, path, process, io};
use std::io::Read;

use clap;
use serde::Deserialize;

fn main() {
    let matches = clap::App::new("backup")
        .arg(
            clap::Arg::with_name("configpath")
                .long("config")
                .short("c")
                .value_name("PATH")
                .takes_value(true)
                .default_value("~/backupkern.yaml")
        )
        .get_matches();

    let path = matches.value_of("configpath").expect("Config path is required.");
    let config = match read_config(&shellexpand::tilde(path)) {
        Err(e) => {
            println!("Error while reading config '{}': {}", path, e);
            return;
        },
        Ok(config) => config,
    };

    if config.to.len() < 1 {
        println!("No location to write to specified!");
        return;
    }

    println!("{:#?}", config);

    // Start copying
    if let Err(e) = run_backup(&config) {
        println!("{}", e);
    }

}

#[derive(Deserialize, Debug)]
struct Config {
    from: String,
    to: Vec<String>,
    prefix: String,
    exclude: ExcludeOptions,
}
#[derive(Deserialize, Debug)]
struct ExcludeOptions {
    locations: Vec<String>,
}
impl Config {
    /// Returns true if the file should not be backed up.
    fn ignore(&self, f: &path::PathBuf) -> bool {
        for l in &self.exclude.locations {
            if f.starts_with(l) {
                return true;
            }
        }
        return false;
    }
}

fn read_config(path: &str) -> Result<Config, Box<error::Error>> {

    let mut file = fs::File::open(path)?;
    let mut config = String::new();
    file.read_to_string(&mut config)?;
    let config = serde_yaml::from_str(&config)?;

    Ok(config)

}

fn get_latest_backup(backup_root: &str) -> Option<path::PathBuf> {

    let all_old_dirs = match fs::read_dir(backup_root) {
        Err(_) => return None,
        Ok(rd) => rd,
    };

    let mut all_old_dirs: Vec<path::PathBuf> = all_old_dirs
        .filter(|e| e.is_ok())
        .map(|e| e.unwrap().path())
        .collect();

    all_old_dirs.sort();
    all_old_dirs.reverse();

    if all_old_dirs.len() > 0 {
        Some(all_old_dirs[0].to_owned())
    } else {
        None
    }

}

/// Compares two paths. If the are not *files* with the same name,
/// this returns false. They will then be compared by size and contents
fn files_equal(a: &path::PathBuf, b: &path::PathBuf) -> bool {
    if a.file_name() != b.file_name() {
        return false;
    }

    if !a.is_file() || !b.is_file() {
        return false;
    }

    match (a.metadata(), b.metadata()) {
        (Ok(a_meta), Ok(b_meta)) => {
            if a_meta.len() != b_meta.len() {
                return false;
            }
            if a_meta.permissions() != b_meta.permissions() {
                return false;
            }
            match (a_meta.modified(), b_meta.modified()) {
                (Ok(a_time), Ok(b_time)) => {
                    return a_time == b_time;
                },
                _ => {
                    return false;
                }
            }

            // TODO Compare file contents if you set a flag
            // This is an implementation of my lazyness. It would be faster to read
            // small chunks of both files and compare them.
//            match (fs::read(a), fs::read(b)) {
//                (Ok(a_content), Ok(b_content)) => {
//                    return md5::compute(a_content) == md5::compute(b_content);
//                },
//                _ => {
//                    return false;
//                },
//            }
        },
        _ => {
            return false;
        },
    }
}

fn cp(copy_from: &path::Path, copy_to: &path::Path) -> io::Result<()> {
    // Use cp -p to preserve timestamps and permissions
    process::Command::new("cp").arg("-p").arg(copy_from).arg(copy_to).output()?;
    Ok(())
}

fn copy_file(copy_from: &path::Path, copy_to: &path::Path, suffix: &path::Path, latest_backup: &Option<path::PathBuf>) -> Result<(), Box<error::Error>> {
    println!("{:?}", copy_from);

    match latest_backup {
        Some(backup) => {
            // Find previous version of file
            let previous_version = backup.join(suffix).to_path_buf();
            if files_equal(&previous_version, &copy_from.to_path_buf()) {
                fs::hard_link(previous_version, copy_to)?;
            } else {
                cp(&copy_from, &copy_to)?;
            }
            Ok(())
        },
        None => {
            cp(&copy_from, &copy_to)?;
            Ok(())
        }
    }

}

fn run_backup(config: &Config) -> Result<(), Box<error::Error>> {

    let pattern = chrono::Local::now().format(&format!("{}_%Y-%m-%d_%H-%M-%S", &config.prefix)).to_string();

    if config.to.len() == 0 {
        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "No locations to backup to.")));
    }
    let mut to_root = None;
    for t in &config.to {
        if path::Path::new(t).is_dir() {
            to_root = Some(t);
        }
    }
    let to_root = match to_root {
        Some(t) => t,
        None => return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, "No locations to backup to."))),
    };
    let to_root = path::Path::new(to_root).join(pattern);
    let from_root = &config.from;
    let latest_backup = get_latest_backup(&config.to[0]);

    println!("get_latest_backup: {:?}", latest_backup);

    println!("Backup running. to_root = {:?}, from_root = {:?}", to_root, from_root);

    for file_entry in walkdir::WalkDir::new(&config.from).min_depth(1) {
        match file_entry {
            Ok(entry) => {

                if entry.path().is_dir() {
                    continue;
                }

                if config.ignore(&entry.path().to_path_buf()) {
                    continue;
                }

                let copy_from = entry.path();
                let suffix = copy_from.strip_prefix(from_root)?;
                let copy_to = to_root.join(suffix);
                if let Some(p) = copy_to.parent() {
                    fs::create_dir_all(p)?;
                }
                if let Err(err) = copy_file(&copy_from, &copy_to, &suffix, &latest_backup) {
                    println!("{}", err);
                }

            },
            Err(err) => println!("{}", err),
        }
    }

    Ok(())
}
