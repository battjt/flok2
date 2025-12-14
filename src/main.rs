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
use mermaid_rs::Mermaid;
use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

mod animal_form;
mod business_obj;
mod flok;
mod flok_form;
mod form;

use flok::*;
use form::*;

use crate::business_obj::BusinessObject;

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

    let mut menu = SysMenuBar::default().with_size(0, 35);
    menu.end();

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
    {
        let form = form.clone();
        menu.add(
            "&Action/Dame Report\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                let mut flok_form = form.lock().unwrap();
                flok_form.flok.exec(|f| {
                    // find all animals with no dame
                    let tree = f
                        .animals
                        .iter()
                        .map(|a| a.dame.clone())
                        .filter(|d| d.is_some());

                    let file = "/tmp/temp.svg";
                    let mut out = File::create(file).expect("Unable to open file");
                    let mermaid = Mermaid::new().unwrap(); // An error may occur if the embedded Chromium instance fails to initialize
                    let input = "graph TB\n".to_string()
                        + &tree
                            .flatten()
                            .map(|id| tree_fn(f, &id, &mut vec![]))
                            .collect::<String>();
                    File::create("tree")
                        .expect("Unable to open file")
                        .write_all(input.as_bytes())
                        .expect("Failed to write");
                    let render = mermaid.render(&input).unwrap();
                    out.write_all(render.as_bytes()).expect("Failed to write");
                    webbrowser::open(&("file://".to_owned() + file))
                        .expect("failed to open browser");
                });
            },
        );
    }

    pack.resizable(&form.lock().unwrap().pack);
    pack.end();

    wind.resizable(&pack);
    wind.end();
    wind.show();

    // run the app
    app.run().unwrap();

    Ok(())
}

fn tree_fn(flok: &Flok, id: &Id, visited: &mut Vec<Id>) -> String {
    if visited.contains(id) {
        return String::new();
    }
    visited.push(id.clone());
    if let Some(animal) = flok.find(id.clone()) {
        let mut result = String::new();
        if let Some(dame_id) = &animal.dame {
            result += &format!("    \"{}\" --> \"{}\"\n", id, dame_id);
            result += &tree_fn(flok, dame_id, visited);
        }
        if let Some(sire_id) = &animal.sire {
            result += &format!("    \"{}\" --> \"{}\"\n", id, sire_id);
            result += &tree_fn(flok, sire_id, visited);
        }
        result
    } else {
        String::new()
    }
}

fn report(flok: Flok) {
    todo!()
}

fn save_flok(form: Arc<Mutex<flok_form::FlokForm>>) -> Result<()> {
    if let Some(mut file) = file_chooser("File to save to", "*.flok", ".", true) {
        if !file.ends_with(".flok") {
            file += ".flok"
        }
        let mut form = form.lock().expect("Unable to lock flok");
        // commit data from UI to data structure
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

trait FlokBo: BusinessObject<Type = Flok> {}
