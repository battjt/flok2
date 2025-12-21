use fltk::{
    button::Button,
    enums::Align,
    frame::Frame,
    group::{Flex, FlexType, Pack, PackType},
    input::Input,
    prelude::{GroupExt, InputExt, WidgetBase, WidgetExt},
    widget::Widget,
    window::Window,
};
use simple_table::joe_table::JoeTable;
use simple_table::simple_model::SimpleModel;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{animal_form::*, business_obj::*, flok::*, form::*};

pub struct FlokForm {
    pub pack: Pack,
    pub name: Input,
    pub table: JoeTable<FlokTableModel>,
    pub flok: Arc<Mutex<Flok>>,
}

impl FlokForm {
    pub fn create(flok: Flok) -> Self {
        let flok = Arc::new(Mutex::new(flok));
        let model = FlokTableModel::new(flok.clone());
        let pack = Pack::default_fill().with_type(PackType::Vertical);
        let name = Default::default();
        let table = JoeTable::new(model);
        pack.resizable(&*table);
        pack.end();
        Self {
            pack,
            name,
            table,
            flok,
        }
    }
    pub fn update(&mut self) {
        self.table.redraw();
    }
}

impl Editor<Flok> for FlokForm {
    fn set_value(&mut self, flok: &Flok) {
        (*self.flok.lock().unwrap()) = flok.clone();
        self.update();
    }
    fn commit(&mut self) {
        self.flok.exec(|f| f.name = self.name.value());
    }
}

#[derive(Default)]
pub struct FlokTableModel {
    pub edit_buttons: HashMap<i32, Widget>,
    pub flok: Arc<Mutex<Flok>>,
}

impl FlokTableModel {
    pub fn new(flok: Arc<Mutex<Flok>>) -> Self {
        Self {
            edit_buttons: Default::default(),
            flok,
        }
    }
}

pub const COLUMNS: [(&str, u32); 5] = [
    ("ID", 60),
    ("Born", 60),
    ("Dame", 60),
    ("Description", 120),
    ("Edit", 40),
];

impl SimpleModel for FlokTableModel {
    fn row_count(&mut self) -> usize {
        self.flok.exec(|f| f.animals.len())
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
            self.flok.exec(|f| {
                let animal = &f.animals[row];
                let r = match col {
                    0 => animal.id[0].clone(),
                    1 => animal.born.map(|b| b.to_string()).unwrap_or_default(),
                    2 => animal.dam.clone().unwrap_or_default(),
                    3 => animal.description.clone(),
                    5 => "junk".to_string(),
                    _ => panic!(),
                };
                Some(r)
            })
        }
    }

    fn cell_widget(&mut self, row_index: i32, _col: i32) -> Option<Widget> {
        let urow = row_index as usize;
        if urow >= self.flok.exec(|f| f.animals.len()) {
            return None;
        }
        let a = self.flok.clone().map(move |f| &mut f.animals[urow]);
        Some(
            self.edit_buttons
                .entry(row_index)
                .or_insert_with(|| {
                    let mut b = Button::default().with_size(30, 20).with_label("Edit");
                    b.set_callback(move |_| {
                        let mut wind = Window::default().with_size(600, 600).with_label(
                            // leak() because fltk expects statics strings for window titles
                            format!("Edit {}", a.exec(|a| a.description.to_string())).leak(),
                        );

                        let mut page = Flex::default_fill()
                            .size_of_parent()
                            .with_type(FlexType::Column);

                        let (mut form, ui) =
                            AnimalForm::create(a.clone()).expect("Unable to create Animal Form");

                        form.set_value(&a);
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
