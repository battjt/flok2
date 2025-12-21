use chrono::{DateTime, Local};
use enum_ordinalize::Ordinalize;
use serde::{Deserialize, Serialize};

pub type Id = String;
pub type Date = DateTime<Local>;

#[derive(Serialize, Deserialize, Debug, Default, Ordinalize, Clone)]
pub enum Sex {
    Male,
    #[default]
    Female,
}
impl Sex {
    pub fn name(&self) -> &str {
        match self {
            Sex::Male => "Male",
            Sex::Female => "Female",
        }
    }
}
impl From<String> for Sex {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "m"| "male" => Sex::Male,
            _ => Sex::Female,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Event {
    pub name: String,
    pub value: String,
    pub date: Date,
    pub notes: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Animal {
    // first is most recent
    pub id: Vec<Id>,
    pub born: Option<Date>,
    pub sire: Option<Id>,
    pub dam: Option<Id>,
    pub description: String,
    pub events: Vec<Event>,
    pub sex: Sex,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Flok {
    pub name: String,
    pub animals: Vec<Animal>,
}

impl Flok {
    pub fn find(&self, id: Id) -> Option<&Animal> {
        self.animals.iter().find(|a| a.id.contains(&id))
    }
    pub fn dam(&self, id: Id) -> Option<Id> {
        self.find(id).and_then(|a| a.dam.clone())
    }
}
