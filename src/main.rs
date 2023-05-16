use std::{io::{stdin, stdout, Write}, str::FromStr, process::exit, path::{Path, PathBuf}, fs::{read_to_string, OpenOptions}};

use dirs::home_dir;

mod recipe;
use recipe::{Item, Recipe, Id};

const ITEM_DATA: &str = include_str!("recipes.toml");

fn get_item(items: &Vec<Item>, filepath: &PathBuf) -> Result<Item, ()> {
    let mut line = String::new();
    stdin().read_line(&mut line).expect("Failed to read input");

    let id = Id::from_str(&("#".to_string() + line.trim_end()))?;
    if &line == "exit\n" || &line == "quit\n" {
        println!("you had:");
        for item in items {
            println!("\t{}", item.name);
        }

        let mut save_data = String::new();
        for item in items {
            save_data.push_str(&format!("{}\n", item.name));
        }

        match OpenOptions::new().write(true).truncate(true).create(true).open(&filepath) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(save_data.as_bytes()) {
                    eprintln!("failed to save game (to {}): {e}", filepath.display());
                }
            },
            Err(e) => eprintln!("failed to save game (to {}): {e}", filepath.display()),
        }

        exit(0);
    }

    if let Some(item) = items.iter().find(|i| Id::Item(i.id) == id) {
        Ok(item.clone())
    } else {
        println!("{}[2J{0}[999F", 27 as char);
        eprintln!("please enter an available item");
        Err(())
    }
}

fn get_available(items: &Vec<Item>, filepath: &PathBuf) -> Vec<Item> {
    if filepath.is_file() {
        let data = match read_to_string(&filepath) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("starting new game; failed to load save file ({}): {e}", filepath.display());
                return items
                    .iter()
                    .filter_map(|item| {
                        if item.recipes[0] == Recipe::Starter {
                            Some(item.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            },
        };

        println!("loading save from {}", filepath.display());
        let mut available = Vec::new();
        for line in data.lines() {
            let id = Id::from_str(&("#".to_string() + line.trim_end())).unwrap();
            if let Some(item) = items.iter().find(|i| Id::Item(i.id) == id) {
                available.push(item.clone());
            } else {
                eprintln!("invalid item in save file ({}): {line}", filepath.display());
            }
        }

        available    
    } else {
        println!("starting new game");
        items
            .iter()
            .filter_map(|item| {
                if item.recipes[0] == Recipe::Starter {
                    Some(item.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

fn main() {
    let items = recipe::Item::from_toml(ITEM_DATA).expect("Failed to load recipes");

    let mut filepath = home_dir().unwrap_or(Path::new("~/").to_path_buf());
    filepath.push(".little-chemistry-save");
    let mut available = get_available(&items, &filepath);

    loop {
        if items.iter().all(|i| available.contains(i)) {
            println!("!!! you found all the items !!!");
            
            if filepath.exists() {
                if let Err(e) = std::fs::remove_file(&filepath) {
                    eprintln!("failed to remove save file ({}): {e}", filepath.display());
                }
            }
            
            return;
        }

        println!("\n\n\nyou have:");
        for item in &available {
            println!("\t{}", item.name);
        }

        print!("item 1: ");
        stdout().flush().unwrap();
        let left = match get_item(&available, &filepath) {
            Ok(i) => i,
            Err(_) => continue,
        };

        print!("item 2: ");
        stdout().flush().unwrap();
        let right = match get_item(&available, &filepath) {
            Ok(i) => i,
            Err(_) => {continue},
        };

        println!("{}[2J{0}[999F", 27 as char);

        let mut made = false;
        for item in &items {
            if item.can_make(&left, &right) {
                if available.contains(item) {
                    println!("you made {} again", item.name);
                } else {
                    println!("! you found {} !", item.name);
                    available.push(item.clone());
                    made = true;
                }
            }
        }

        if !made {
            println!("you didn't find anything");
        }
    }
}
