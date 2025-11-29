use anyhow::Result;
use enum_ordinalize::Ordinalize;
use fltk::{
    input::Input,
    menu::Choice,
    prelude::{InputExt, MenuExt},
    widget::Widget,
};

use crate::{business_obj::*, flok::*, form::*};

#[derive(Clone, Default)]
pub struct DateInput {
    pub input: Input,
}

impl DateInput {
    pub fn get_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        dateparser::parse(&self.input.value()).ok()
    }
}

impl From<DateInput> for Input {
    fn from(val: DateInput) -> Self {
        val.input
    }
}

pub struct AnimalForm<A: BusinessObject<Type = Animal>> {
    pub identity: Input,
    pub sex: Choice,
    pub born: DateInput,
    pub description: Input,
    pub animal: A,
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
    }

    fn commit(&mut self) {
        self.animal.exec(|animal| {
            let id = self
                .identity
                .value()
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect();
            animal.id = id;
            animal.born = self.born.get_date();
            animal.sire = None;
            animal.dame = None;
            animal.description = self.description.value();
            animal.events = Vec::default();
            animal.sex = Sex::from_ordinal(self.sex.value() as i8).unwrap_or_default();
        });
    }
}

impl<A: BusinessObject<Type = Animal>> AnimalForm<A> {
    pub fn create(animal: A) -> Result<(Self, Widget)> {
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
            animal,
        };

        let ui = create_form(vec![
            ("Identity", &form.identity),
            ("Sex", &form.sex),
            ("Born", &form.born.input),
            ("Description", &form.description),
        ])?;

        Ok((form, ui))
    }
}
