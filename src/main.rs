use log::*;
use std::{fs, path::Path};
use wxml_compiler::*;

fn main() {
    println!("Hello, world!");
}

fn load_wxml_files(group: &mut TmplGroup, dir: &Path, wxml_path: &mut Vec<String>) -> u64 {
    trace!("Search in path: {}", dir.to_str().unwrap_or(""));
    let mut size: u64 = 0;
    match fs::read_dir(dir) {
        Err(_) => {
            warn!("List dir failed: {}", dir.to_str().unwrap_or(""));
        }
        Ok(list) => {
            for entry in list {
                match entry {
                    Err(_) => {
                        warn!("Get path failed: {}", dir.to_str().unwrap_or(""));
                    }
                    Ok(entry) => {
                        let path = entry.path();
                        let file_size = entry.metadata().unwrap().len();
                        if path.is_dir() {
                            wxml_path.push(entry.file_name().to_str().unwrap().into());
                            size += load_wxml_files(group, &path, wxml_path);
                            wxml_path.pop();
                        } else if path
                            .extension()
                            .map(|x| x.to_str().unwrap_or(""))
                            .unwrap_or("")
                            == "wxml"
                        {
                            match fs::read_to_string(&path) {
                                Err(_) => {
                                    warn!("Read wxml failed: {}", path.to_str().unwrap_or(""));
                                }
                                Ok(content) => {
                                    trace!("Found wxml file: {}", path.to_str().unwrap_or(""));
                                    wxml_path.push(
                                        entry
                                            .path()
                                            .file_stem()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .to_string(),
                                    );
                                    group.add_tmpl(&wxml_path.join("/"), &content).unwrap();
                                    wxml_path.pop();
                                    size += file_size;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_loads_wxml_files() {
        let test_dir =
            std::env::temp_dir().join("glass_easel_template_compiler_tests_load_wxml_files");

        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir(&test_dir).unwrap();
        fs::write(
            test_dir.join("index.wxml"),
            "<view><text>test wxml-compiler</text></view>",
        )
        .unwrap();

        fs::write(
            test_dir.join("index2.wxml"),
            r#"<view><template is="odd" data="{{data.a}}">{{data.b}}</template></view>"#,
        )
        .unwrap();

        let mut group = TmplGroup::new();
        load_wxml_files(&mut group, &test_dir, &mut vec![]);

        println!("{:?}", group);

        fs::remove_dir_all(test_dir).unwrap();
    }
}
