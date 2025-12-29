use anyhow::Result;
use chrono::DateTime;
use enum_ordinalize::Ordinalize;
use fltk::{
    input::Input,
    menu::Choice,
    prelude::{InputExt, WidgetExt,MenuExt},
    widget::Widget,
};
use simple_table::joe_table::JoeTable;

use crate::{business_obj::*, event_form::EventTableModel, flok::*, form::*};

#[derive(Clone, Default)]
pub struct DateInput {
    pub input: Input,
}

impl DateInput {
    pub fn get_date(&self) -> Option<chrono::DateTime<chrono::Local>> {
        dateparser::parse(&self.input.value())
            .map(DateTime::from)
            .ok()
    }
}

impl From<DateInput> for Input {
    fn from(val: DateInput) -> Self {
        val.input
    }
}

pub struct AnimalForm<A: 'static + BusinessObject<Type = Animal>> {
    pub identity: Input,
    pub sex: Choice,
    pub born: DateInput,
    pub description: Input,
    pub animal: A,
    pub dame: Input,
    pub sire: Input,
    pub events: JoeTable<EventTableModel<A>>,
}

impl<A: BusinessObject<Type = Animal>> Editor<A> for AnimalForm<A> {
    fn set_value(&mut self, a: &A) {
        self.identity.set_value(&a.exec(|a| a.id.join(", ")));
        self.sex.set_value(a.exec(|a| a.sex.ordinal() as i32));

        self.born
            .input
            .set_value(&a.exec(move |a| a.born.map(|d| d.to_string()).unwrap_or("".to_string())));
        self.description
            .set_value(&a.exec(|a| a.description.clone()));
        self.sire
            .set_value(&a.exec(|a| a.sire.clone().unwrap_or("".to_string())));
        self.dame
            .set_value(&a.exec(|a| a.dam.clone().unwrap_or("".to_string())));
        self.events.model.lock().unwrap().animal = a.clone();
    }

    fn commit(&mut self) {
        let e = self.events.model.lock().unwrap().animal.exec(|a|a.events.clone());
        self.animal.exec(|animal| {
            animal.id = self
                .identity
                .value()
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect();
            animal.born = self.born.get_date();
            animal.sire = if self.sire.value().is_empty() {
                None
            } else {
                Some(self.sire.value())
            };
            animal.dam = if self.dame.value().is_empty() {
                None
            } else {
                Some(self.dame.value())
            };
            animal.description = self.description.value();
            animal.events = e.clone();
            animal.sex = Sex::from_ordinal(self.sex.value() as i8).unwrap_or_default();
        });
    }
}

impl<A: BusinessObject<Type = Animal>> AnimalForm<A> {
    pub fn create(animal: A) -> Result<(Self, Widget)> {
        let events = JoeTable::new(EventTableModel::new(animal.clone()));
        let mut widget = events.to_widget();
        widget.set_size(0, 200);
        let form = Self {
            identity: Input::default(),
            sex: {
                let mut sex = Choice::default();
                for s in Sex::VARIANTS.iter() {
                    sex.add_choice(s.name());
                }
                sex
            },
            born: DateInput::default(),
            description: Input::default(),
            animal: animal.clone(),
            dame: Input::default(),
            sire: Input::default(),
            events,
        };

        let ui = create_form(vec![
            ("Identity", &form.identity),
            ("Sex", &form.sex),
            ("Born", &form.born.input),
            ("Description", &form.description),
            ("Sire", &form.sire),
            ("Dame", &form.dame),
            ("Events", &widget),
        ])?;

        Ok((form, ui))
    }
}
