use chrono::{DateTime, Utc};
use enum_ordinalize::Ordinalize;
use serde::{Deserialize, Serialize};

pub type Id = String;
pub type Date = DateTime<Utc>;

#[derive(Serialize, Deserialize, Debug, Default, Ordinalize, Clone)]
pub enum Sex {
    Male,
    #[default]
    Female,
}
impl Sex {
    pub fn name(&self) -> &str {
        match self  {
            Sex::Male => "Male",
            Sex::Female => "Female",
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
    pub dame: Option<Id>,
    pub description: String,
    pub events: Vec<Event>,
    pub sex: Sex,
}

impl Animal {}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Flok {
    pub name: String,
    pub animals: Vec<Animal>,
}
