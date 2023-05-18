use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io::{self, ErrorKind};
use std::process::exit;
use std::str::FromStr;

const ITEM_DATA: &str = include_str!("recipes.toml");

fn hash(i: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    i.hash(&mut hasher);
    hasher.finish()
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub recipes: Vec<Recipe>,
    pub id: ItemId,
    pub classes: Vec<ClassId>,
}

impl Item {
    pub fn init() -> io::Result<Vec<Self>> {
        let toml_data = toml::from_str::<toml::Value>(ITEM_DATA)
            .map_err(|e| io::Error::new(ErrorKind::InvalidInput, e))?;

        let mut items = Vec::new();
        if let toml::Value::Table(data) = &toml_data.get("item").ok_or(io::Error::new(
            ErrorKind::InvalidInput,
            "There must be an `item` table in the recipes",
        ))? {
            for (name, info) in data {
                let classes = if let toml::Value::Array(classes) =
                    &info.get("class").ok_or(io::Error::new(
                        ErrorKind::InvalidInput,
                        format!("item.{name} must have an array, `class`"),
                    ))? {
                    let mut e = false;
                    for class in classes {
                        if let toml::Value::String(class) = class {
                            if class.starts_with(".") {
                                eprintln!("`item.{name}.class` contains `{class}` which shouldn't start with a dot");
                                e = true;
                            }
                        }
                    }

                    if e {
                        exit(1);
                    }

                    classes
                        .iter()
                        .filter_map(|v| Some(ClassId(hash(v.as_str()?))))
                        .collect()
                } else {
                    return Err(io::Error::new(
                        ErrorKind::InvalidInput,
                        format!("`item.{name}.class` must be an array"),
                    ));
                };

                let recipes = Recipe::from_value(info.get("recipe").ok_or(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("`item.{name}` must have an array, `recipe`"),
                ))?)
                .map_err(|_| {
                    io::Error::new(
                        ErrorKind::InvalidInput,
                        format!("`item.{name}.recipes` is invalid"),
                    )
                })?;

                items.push(Self {
                    name: name.to_string(),
                    recipes,
                    id: ItemId(hash(name)),
                    classes,
                });
            }
        } else {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "`item` must be a table",
            ));
        }

        Ok(items)
    }

    pub fn ids(&self) -> Vec<Id> {
        let mut ids = vec![Id::Item(self.id)];
        ids.append(&mut self.classes.iter().map(|id| Id::Class(*id)).collect());

        ids
    }

    pub fn can_make(&self, left: &Item, right: &Item) -> bool {
        for recipe in &self.recipes {
            for left_id in left.ids() {
                for right_id in right.ids() {
                    if recipe == (left_id, right_id) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Recipe {
    Starter,
    Normal(Id, Id),
}

impl Recipe {
    pub fn new(left: &str, right: &str) -> Result<Self, ()> {
        Ok(Self::Normal(Id::from_str(left)?, Id::from_str(right)?))
    }

    fn from_value(recipe: &toml::Value) -> Result<Vec<Self>, ()> {
        if recipe.as_str() == Some("$starter") {
            return Ok(vec![Self::Starter]);
        }

        let recipe = if let toml::Value::Array(recipe) = recipe {
            recipe
        } else {
            return Err(());
        };

        let mut recipes = Vec::new();
        if recipe.iter().all(|r| r.is_array()) {
            for sub in recipe {
                recipes.append(&mut Self::from_value(sub)?);
            }
            
            return Ok(recipes);
        }

        if let toml::Value::String(left) = &recipe.get(0).ok_or(())? {
            if let toml::Value::String(right) = &recipe.get(1).ok_or(())? {
                recipes.push(Recipe::new(left, right)?);
            } else if let toml::Value::Array(right) = &recipe.get(1).ok_or(())? {
                for right_id in right {
                    let right_id = right_id.as_str().ok_or(())?;
                    recipes.push(Recipe::new(left, &right_id)?);
                }
            } else {
                return Err(());
            }
        } else if let toml::Value::Array(left) = &recipe.get(0).ok_or(())? {
            for left_id in left {
                let left_id = left_id.as_str().ok_or(())?;
                if let toml::Value::String(right) = &recipe.get(1).ok_or(())? {
                    recipes.push(Recipe::new(left_id, right)?);
                } else if let toml::Value::Array(right) = &recipe.get(1).ok_or(())? {
                    for right_id in right {
                        let right_id = right_id.as_str().ok_or(())?;
                        recipes.push(Recipe::new(left_id, &right_id)?);
                    }
                } else {
                    return Err(());
                }
            }
        } else {
            return Err(());
        }

        Ok(recipes)
    }
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starter => write!(f, "$starter"),
            Self::Normal(left, right) => write!(f, "Recipe({:?} + {:?})", left, right),
        }
    }
}

impl PartialEq<(Id, Id)> for &Recipe {
    fn eq(&self, other: &(Id, Id)) -> bool {
        match self {
            Recipe::Normal(left, right) => {
                (left == &other.0 && right == &other.1) || (left == &other.1 && right == &other.0)
            }
            Recipe::Starter => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Id {
    Item(ItemId),
    Class(ClassId),
}

impl Id {
    pub fn class(id: &str) -> Self {
        Self::Class(ClassId(hash(id)))
    }

    pub fn item(id: &str) -> Self {
        Self::Item(ItemId(hash(id)))
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item(id) => write!(f, "#({})", id.0),
            Self::Class(id) => write!(f, ".({})", id.0),
        }
    }
}

impl PartialEq<Item> for Id {
    fn eq(&self, other: &Item) -> bool {
        match self {
            Id::Item(id) => {
                if &other.id == id {
                    true
                } else {
                    false
                }
            }
            Id::Class(id) => {
                if other.classes.contains(id) {
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl FromStr for Id {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(class_id) = s.strip_prefix('.') {
            Ok(Self::class(class_id))
        } else if let Some(item_id) = s.strip_prefix('#') {
            Ok(Self::item(item_id))
        } else {
            Err(())
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemId(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClassId(u64);
