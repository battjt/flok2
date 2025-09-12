use std::{
    collections::HashMap,
    fs::File,
    io::stderr,
    rc::Weak,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use enum_ordinalize::Ordinalize;
use flok::Flok;
use fltk::{
    app::{self, lock},
    button::Button,
    dialog,
    enums::{Mode, Shortcut},
    frame::Frame,
    group::{Pack, PackType},
    input::Input,
    menu::{self, Choice, SysMenuBar},
    prelude::{GroupExt, InputExt, MenuExt, WidgetBase, WidgetExt},
    table::Table,
    widget::Widget,
    window::Window,
};
use simple_table::{
    joe_table::JoeTable,
    simple_model::{DrawDelegate, SimpleModel},
};

use crate::flok::{Animal, Date, Sex};
use crate::reference::*;


mod flok;
mod form;
mod reference;

 pub fn main() -> Result<()> {
    let app = app::App::default().with_scheme(app::Scheme::Plastic);
    app.set_visual(Mode::MultiSample | Mode::Alpha)?;

    let mut wind = Window::default()
        .with_size(400, 600)
        .with_label(&format!("J1939 Log {}", &env!("CARGO_PKG_VERSION")));
    let mut pack = Pack::default_fill();
    let mut menu = SysMenuBar::default().with_size(100, 35);
    let form: Arc<Mutex<Option<FlokForm>>> = Default::default();
    {
        let form = form.clone();
        menu.add(
            "&Action/@fileopen Load Records...\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                let mut chooser = dialog::FileChooser::new(
                    ".",                             // directory
                    "*",                             // filter or pattern
                    dialog::FileChooserType::Single, // chooser type
                    "Restore DB",                    // title
                );
                chooser.window().set_pos(300, 300);
                // Block until user picks something.
                //     (The other way to do this is to use a callback())
                //
                while chooser.shown() {
                    app::wait();
                }
                // User hit cancel?
                if chooser.value(1).is_none() {
                    println!("(User hit 'Cancel')");
                    return;
                }
                let str = chooser.value(1).unwrap();
                let file: File = File::open(str).unwrap();

                let mut flok: Flok = serde_json::from_reader(file).unwrap();
                *form.lock().expect("Unable to lock editor") = Some(FlokForm::create(flok));
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
                let flok = Flok {
                    name: "new".to_string(),
                    animals: vec![],
                };
                *form.lock().expect("Unable to lock editor") = Some(FlokForm::create(flok));
            },
        );
    }
    {
        let form = form.clone();
        menu.add(
            "&Action/New Animal\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                let mut flok = form.lock().expect("Unable to lock editor");
                if let Some(form) = flok.as_mut() {
                    form.flok.exec(|f| {
                        f.animals.push(Animal {
                            id: vec!["new".to_string()],
                            born: None,
                            sire: None,
                            dame: None,
                            events: vec![],
                            description: "".to_string(),
                            sex: Sex::Female,
                        })
                    });
                }
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
                let flok = form.lock().expect("Unable to lock flok");
                if let Some(form) = &*flok {
                    form.flok
                        .exec(|flok| serde_json::to_writer_pretty(stderr(), flok));
                }
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

struct FlokForm<'a> {
    name: Input,
    table: JoeTable<FlokTableModel>,
    flok: BusinessObject<Flok, Flok>,
}

impl FlokForm {
    fn create(flok: Flok) -> Self {
        let business_object = BusinessObject::new(Arc::new(Mutex::new(flok)),|a|a);
        let animals = business_object.map(Box::new(|mut f| &mut f.animals));
        Self {
            name: Default::default(),
            table: JoeTable::new(FlokTableModel::new(animals)),
            flok: business_object,
        }
    }
    fn commit(&mut self) {
        self.flok.exec(|f| f.name = self.name.value())
    }
}

trait Editor<T> {
    fn set_value(&mut self, value: BusinessObject<Flok, T>);
    fn commit(&mut self);
}
struct FlokTableModel {
    animals: BusinessObject<Flok, Vec<Animal>>,
    edit_buttons: HashMap<i32, Widget>,
}
impl FlokTableModel {
    fn new(animals: BusinessObject<Flok, Vec<Animal>>) -> Self {
        Self {
            animals,
            edit_buttons: Default::default(),
        }
    }
}

const COLUMNS: [(&str, u32); 5] = [
    ("ID", 60),
    ("Born", 60),
    ("Dame", 60),
    ("Description", 120),
    ("Edit", 40),
];
impl SimpleModel for FlokTableModel {
    fn row_count(&mut self) -> usize {
        self.edit_buttons.clear();
        self.animals.exec(|animals| animals.len())
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
        if row >= self.row_count() || col as usize >= COLUMNS.len() - 1 {
            None
        } else {
            self.animals.exec(|animals| {
                let animal = &mut animals[row];
                let r = match col {
                    0 => animal.id[0].clone(),
                    1 => animal.born.map(|b| b.to_string()).unwrap_or_default(),
                    2 => animal.dame.clone().unwrap_or_default(),
                    3 => animal.description.clone(),
                    _ => panic!(),
                };
                Some(r)
            })
        }
    }

    fn cell_widget(&mut self, row: i32, col: i32) -> Option<Widget> {
        Some(
            self.edit_buttons
                .entry(row)
                .or_insert_with(|| {
                    let mut b = Button::default().with_size(30, 20).with_label("Edit");
                    let r = self.animals;
                    b.set_callback(move |_| {
                        let mut form = AnimalForm::create(
                            r.map(move |f: &mut Vec<Animal>| &mut f[row as usize]),
                        );
                        let mut button = Button::default();
                        button.set_callback(move |b| {
                            form.commit();
                        });
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
impl Into<Input> for DateInput {
    fn into(self) -> Input {
        self.input
    }
}

struct AnimalForm {
    identity: Input,
    sex: Choice,
    born: DateInput,
    description: Input,
    animal: BusinessObject<Flok, Animal>,
}
impl Editor<Animal> for AnimalForm {
    fn set_value(&mut self, a: BusinessObject<Flok, Animal>) {
        a.exec(|a| {
            self.identity.set_value(&a.id.join(", "));
            self.sex.set_value(a.sex.ordinal() as i32);
        });
    }

    fn commit(&mut self) {
        self.animal.exec(|mut animal| {
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
fn row_into<I, W>(title: &str, widget: I) -> I
where
    I: Into<W> + Clone,
    W: WidgetExt,
{
    let w: W = widget.clone().into();
    row(title, w);
    widget
}
fn row<W>(title: &str, widget: W) -> W
where
    W: WidgetExt,
{
    let mut p = Pack::default_fill().with_type(PackType::Horizontal);
    p.add(&Frame::default().with_label(title));
    p.add(&widget);
    widget
}

impl AnimalForm {
    fn create(animal: BusinessObject<Flok, Animal>) -> Self {
        let identity = row("Identity", Input::default());
        let mut sex_choices = Choice::default();
        sex_choices.add_choice("female");
        sex_choices.add_choice("male");
        let sex = row("Sex", sex_choices);
        let born: DateInput = row_into::<_, Input>("Born", DateInput::default());
        let description = row("Description", Input::default());
        Self {
            identity,
            sex,
            born,
            description,
            animal,
        }
    }
}
