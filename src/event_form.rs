use fltk::input::Input;
use fltk::widget::Widget;
use fltk::{
    button::Button,
    enums::Align,
    frame::Frame,
    group::{Flex, FlexType},
    prelude::{GroupExt, InputExt, WidgetBase, WidgetExt},
    window::Window,
};

use simple_table::simple_model::SimpleModel;

use std::collections::HashMap;

use anyhow::Result;

use crate::{
    animal_form::DateInput,
    business_obj::BusinessObject,
    flok::{Animal, Event},
    form::{create_form, Editor},
};

#[derive(Default)]
pub struct EventForm<A: BusinessObject<Type = Event>> {
    pub name: Input,
    pub date: DateInput,
    pub value: Input,
    pub notes: Input,
    pub event: A,
}

impl<A: BusinessObject<Type = Event>> EventForm<A> {
    pub(crate) fn create(event: A) -> Result<(Self, Widget)> {
        let event_form = Self {
            name: Input::default(),
            date: DateInput::default(),
            value: Input::default(),
            notes: Input::default(),
            event,
        };
        let ui = create_form(vec![
            ("Name", &event_form.name),
            ("Date", &event_form.date.input),
            ("Value", &event_form.value),
            ("Notes", &event_form.notes),
        ])?;
        Ok((event_form, ui))
    }
}

impl<A: BusinessObject<Type = Event>> Editor<A> for EventForm<A> {
    fn set_value(&mut self, event: &A) {
        self.name.set_value(&event.exec(|e| e.name.clone()));
        self.date
            .input
            .set_value(&event.exec(|e| e.date.to_string()));
        self.value.set_value(&event.exec(|e| e.value.clone()));
        self.notes.set_value(&event.exec(|e| e.notes.clone()));
    }

    fn commit(&mut self) {
        self.event.exec(|e| {
            e.name = self.name.value();
            e.date = self.date.get_date().unwrap_or_else(chrono::Local::now);
            e.value = self.value.value();
            e.notes = self.notes.value();
        });
    }
}

#[derive(Default)]
pub struct EventTableModel<A: BusinessObject<Type = Animal>> {
    pub edit_buttons: HashMap<i32, Widget>,
    pub animal: A,
}

impl<A: BusinessObject<Type = Animal>> EventTableModel<A> {
    pub fn new(animal: A) -> Self {
        Self {
            edit_buttons: Default::default(),
            animal,
        }
    }
}

pub const COLUMNS: [(&str, u32); 5] = [
    ("Name", 60),
    ("Date", 60),
    ("Value", 60),
    ("Notes", 120),
    ("Edit", 40),
];

impl<A: 'static + BusinessObject<Type = Animal>> SimpleModel for EventTableModel<A> {
    fn row_count(&mut self) -> usize {
        self.animal.exec(|a| a.events.len())
    }

    fn column_count(&mut self) -> usize {
        COLUMNS.len()
    }

    fn header(&mut self, col: usize) -> String {
        COLUMNS[col].0.to_string()
    }

    fn column_width(&mut self, col: usize) -> u32 {
        COLUMNS[col].1
    }

    fn cell(&mut self, row: i32, col: i32) -> Option<String> {
        let row = row as usize;
        // column 4 uses a widget
        if row >= self.row_count() || col == 4 || col >= COLUMNS.len() as i32 {
            None
        } else {
            self.animal.exec(|f| {
                let animal = &f.events[row];
                let r = match col {
                    0 => animal.name.clone(),
                    1 => animal.date.to_string(),
                    2 => animal.value.clone(),
                    3 => animal.notes.clone(),
                    _ => panic!(),
                };
                Some(r)
            })
        }
    }

    fn cell_widget(&mut self, row_index: i32, _col: i32) -> Option<Widget> {
        let urow = row_index as usize;
        if urow >= self.animal.exec(|f| f.events.len()) {
            return None;
        }
        let event = self.animal.clone().map(move |f| &mut f.events[urow]);
        Some(
            self.edit_buttons
                .entry(row_index)
                .or_insert_with(|| {
                    let name = format!("Edit {}", event.exec(|a| a.name.to_string()));
                    let mut b = Button::default().with_size(30, 20).with_label(&name);
                    b.set_callback(move |_| {

                        let mut wind = Window::default().with_size(600, 600).with_label(
                            // leak() because fltk expects statics strings for window titles
                            format!("Edit {}", event.exec(|a| a.name.to_string())).leak(),
                        );

                        let mut page = Flex::default_fill()
                            .size_of_parent()
                            .with_type(FlexType::Column);

                        let (mut form, ui) =
                            EventForm::create(event.clone()).expect("Failed to create event form");

                        form.set_value(&event);
                        page.fixed(&ui, ui.height());

                        let mut buttons = Flex::default()
                            .row()
                            .with_align(Align::Right)
                            .size_of_parent();
                        buttons.resizable(&Frame::default());
                        {
                            let mut cancel =
                                Button::default().size_of_parent().with_label("Cancel");
                            let mut wind = wind.clone();
                            cancel.set_callback(move |_b| wind.hide());
                            buttons.fixed(&cancel, 60);
                        }
                        {
                            let mut save = Button::default().size_of_parent().with_label("Save");
                            let mut wind = wind.clone();
                            save.set_callback(move |_b| {
                                form.commit();

                                wind.hide();
                            });
                            buttons.fixed(&save, 60);
                        }
                        buttons.end();
                        page.fixed(&buttons, 25);

                        page.resizable(&Frame::default());

                        page.end();

                        wind.make_resizable(true);
                        wind.set_size(400, 5 + ui.height() + buttons.height());
                        wind.end();
                        wind.show();
                    });
                    b.as_base_widget()
                })
                .clone(),
        )
    }
}
