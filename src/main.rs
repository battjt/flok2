#![feature(mapped_lock_guards)]

use anyhow::Result;
use clap::Parser;
use fltk::{
    app::{self},
    dialog::file_chooser,
    enums::{Mode, Shortcut},
    group::Pack,
    menu::{self, SysMenuBar},
    prelude::{GroupExt, MenuExt, WidgetBase, WidgetExt},
    window::Window,
};
use std::{
    fs::File,
    sync::{Arc, Mutex},
};

mod business_obj;
mod flok;
mod form;
mod flok_form;
mod animal_form;

use flok::*;
use form::*;

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

    let form = Arc::new(Mutex::new(flok_form::FlokForm::create(Flok::default())));

    if let Some(file) = Cli::parse().file {
        *(form.lock().unwrap().flok.lock().unwrap()) = serde_json::from_reader(File::open(file)?)?;
    }

    {
        let form = form.clone();
        menu.add(
            "&Action/@fileopen Load Records...\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| display_error("Unable to load file", load_flok(form.clone())),
        );
    }
    {
        let form = form.clone();
        menu.add(
            "&Action/@filesave Save Records...\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| display_error("Unable to save file", save_flok(form.clone())),
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
                    .set_value(&Default::default())
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
                let mut flok_form = form.lock().unwrap();
                flok_form.flok.lock().unwrap().animals.push(Animal {
                    id: vec!["new".to_string()],
                    born: None,
                    sire: None,
                    dame: None,
                    events: vec![],
                    description: "".to_string(),
                    sex: Sex::Female,
                });
                flok_form.update();
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

fn save_flok(form: Arc<Mutex<flok_form::FlokForm>>) -> Result<()> {
    if let Some(mut file) = file_chooser("File to save to", "*.flok", ".", true) {
        if !file.ends_with(".flok") {
            file += ".flok"
        }
        let mut form = form.lock().expect("Unable to lock flok");
        form.commit();
        let flok = form.flok.lock().unwrap();
        serde_json::to_writer_pretty(File::create(file)?, &*flok)?;
    }
    Ok(())
}
fn load_flok(form: Arc<Mutex<flok_form::FlokForm>>) -> Result<()> {
    if let Some(file) = file_chooser("File to load from", "*.flok", ".", true) {
        let flok: Flok = serde_json::from_reader(File::open(file)?)?;
        form.lock().unwrap().set_value(&flok);
    }
    Ok(())
}
