#![feature(mapped_lock_guards)]

use std::{
    collections::HashMap,
    fs::File,
    io::stderr,
    sync::{Arc, Mutex},
};

use anyhow::{Error, Result};
use clap::Parser;
use enum_ordinalize::Ordinalize;
use flok::Flok;
use fltk::{
    app::{self},
    dialog::file_chooser,
    enums::Align,
    group::{Flex, FlexType, Grid},
};
use fltk::{
    button::Button,
    dialog,
    enums::{Mode, Shortcut},
    frame::Frame,
    group::Pack,
    input::Input,
    menu::{self, Choice, SysMenuBar},
    prelude::{GroupExt, InputExt, MenuExt, WidgetBase, WidgetExt},
    widget::Widget,
    window::Window,
};
use simple_table::{joe_table::JoeTable, simple_model::SimpleModel};

use crate::flok::{Animal, Sex};

mod flok;

mod business_obj;
use business_obj::*;
#[derive(clap::Parser)]
struct Cli {
    file: Option<String>,
}
pub fn main() -> Result<()> {
    let app = app::App::default().with_scheme(app::Scheme::Plastic);
    app.set_visual(Mode::MultiSample | Mode::Alpha)?;

    let mut wind = Window::default()
        .with_size(400, 600)
        // FLTK expects the window label to be static
        .with_label(format!("Flok Editor {}", &env!("CARGO_PKG_VERSION")).leak());
    let pack = Pack::default_fill();
    let mut menu = SysMenuBar::default().with_size(100, 35);

    let form = Arc::new(Mutex::new(FlokForm::create(Flok::default())));

    if let Some(file) = Cli::parse().file {
        eprintln!("Opening {file}");
        *(form.lock().unwrap().flok.lock().unwrap()) = serde_json::from_reader(File::open(file)?)?;
    }

    {
        let form = form.clone();
        menu.add(
            "&Action/@fileopen Load Records...\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                (|| {
                    if let Some(file) = file_chooser("File to save to", "*.flok", ".", true) {
                        let flok: Flok = serde_json::from_reader(File::open(file)?)?;
                        form.lock().unwrap().set_value(&flok);
                    }
                    Ok(())
                })()
                .unwrap_or_else(|err: Error| {
                    fltk::dialog::alert_default(&format!("Unable to write file: {err}"));
                })
            },
        );
    }
    {
        let form = form.clone();
        menu.add(
            "&Action/New Flock\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                form.lock()
                    .expect("Unable to lock editor")
                    .set_value(&Default::default());
            },
        );
    }
    {
        let flok = form.lock().unwrap().flok.clone();
        let form = form.clone();
        menu.add(
            "&Action/New Animal\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                let mut f = flok.lock().unwrap();
                f.animals.push(Animal {
                    id: vec!["new".to_string()],
                    born: None,
                    sire: None,
                    dame: None,
                    events: vec![],
                    description: "".to_string(),
                    sex: Sex::Female,
                });
                form.lock().unwrap().update();
            },
        );
    }
    {
        let form = form.clone();
        menu.add(
            "&Action/@filesave Save Records...\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                (|| {
                    if let Some(file) = file_chooser("File to save to", "*.flok", ".", true) {
                        let mut form = form.lock().expect("Unable to lock flok");
                        form.commit();
                        let flok = form.flok.lock().unwrap();
                        serde_json::to_writer_pretty(File::create(file)?, &*flok)?;
                    }
                    Ok(())
                })()
                .unwrap_or_else(|err: Error| {
                    fltk::dialog::alert_default(&format!("Unable to write file: {err}"));
                })
            },
        );
    }
    menu.end();
    // let widget = &form.lock().unwrap().table.as_base_widget();
    // pack.add(widget);
    pack.end();

    //wind.resizable(widget);
    wind.end();
    wind.show();

    // run the app
    app.run().unwrap();

    Ok(())
}

struct FlokForm {
    name: Input,
    table: JoeTable<FlokTableModel>,
    flok: Arc<Mutex<Flok>>,
}

impl FlokForm {
    fn create(flok: Flok) -> Self {
        let flok = Arc::new(Mutex::new(flok));
        let model = FlokTableModel::new(flok.clone());
        let table = JoeTable::new(model);
        Self {
            name: Default::default(),
            table,
            flok,
        }
    }
    fn update(&mut self) {
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
trait Editor<T> {
    fn set_value(&mut self, value: &T);
    fn commit(&mut self);
}
#[derive(Default)]
struct FlokTableModel {
    edit_buttons: HashMap<i32, Widget>,
    flok: Arc<Mutex<Flok>>,
}

impl FlokTableModel {
    fn new(flok: Arc<Mutex<Flok>>) -> Self {
        Self {
            edit_buttons: Default::default(),
            flok,
        }
    }
}
const COLUMNS: [(&str, u32); 6] = [
    ("ID", 60),
    ("Born", 60),
    ("Dame", 60),
    ("Description", 120),
    ("Edit", 40),
    ("Junk", 20),
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
                    2 => animal.dame.clone().unwrap_or_default(),
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

#[derive(Clone, Default)]
struct DateInput {
    input: Input,
}
impl DateInput {
    fn get_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        dateparser::parse(&self.input.value()).ok()
    }
}
impl From<DateInput> for Input {
    fn from(val: DateInput) -> Self {
        val.input
    }
}

struct AnimalForm<A: BusinessObject<Type = Animal>> {
    identity: Input,
    sex: Choice,
    born: DateInput,
    description: Input,
    animal: A,
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

const SIZE: i32 = 20;
fn create_form(fields: Vec<(&str, &dyn FromWidget)>) -> Result<Widget> {
    let mut grid = Grid::default();
    grid.set_layout(1 + fields.len() as i32, 2);
    grid.set_size(0, SIZE * (fields.len() as i32));

    for (row, field) in fields.into_iter().enumerate() {
        grid.set_widget(&mut Frame::default().with_label(field.0), row, 0)?;
        let mut widget = field.1.to_widget();
        grid.add(&widget);
        grid.set_widget(&mut widget, row, 1)?;

        {
            let row = row as i32;
            grid.set_row_height(row, SIZE);
            grid.set_row_weight(row, 0);
        }
    }
    grid.end();
    Ok(grid.as_base_widget())
}

trait FromWidget {
    fn to_widget(&self) -> Widget;
}

impl<W: WidgetBase> FromWidget for W {
    fn to_widget(&self) -> Widget {
        self.as_base_widget()
    }
}

impl<A: BusinessObject<Type = Animal>> AnimalForm<A> {
    fn create(animal: A) -> Result<(Self, Widget)> {
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
