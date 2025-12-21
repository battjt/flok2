#![feature(mapped_lock_guards)]

use anyhow::Result;
use calamine::{DataType, Reader};
use chrono::{Local, TimeZone};
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
use layout::{
    backends::svg::SVGWriter,
    gv::{DotParser, GraphBuilder},
};
use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};
use tempfile::NamedTempFile;

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
                    dam: None,
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
            "&Action/Lineage Report\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                display_error(
                    "Unable to report lineage",
                    form.lock().unwrap().flok.exec(report_lineage),
                );
            },
        );
    }
    {
        let form = form.clone();
        menu.add(
            "&Action/Import...\t",
            Shortcut::None,
            menu::MenuFlag::Normal,
            move |_| {
                display_error("Unable to import file", import_flok(form.clone()));
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

fn import_flok(form: Arc<Mutex<flok_form::FlokForm>>) -> Result<()> {
    if let Some(file) = file_chooser(
        "File to load from",
        "*.xlsx\t*.xls\t*.csv\t*.ods",
        ".",
        true,
    ) {
        let form_guard = form.lock().unwrap();
        let mut flok_guard = form_guard.flok.lock().unwrap();
        let mut wb = calamine::open_workbook_auto(file)?;
        let sheet = &wb.worksheets()[0].1;
        let headers = sheet
            .headers()
            .ok_or(anyhow::anyhow!("No headers found in sheet"))?;
        for row in sheet.rows() {
            let mut animal = Animal::default();
            for (header, cell) in headers.iter().zip(row.iter()) {
                match header.to_lowercase().as_str() {
                    "id" => animal.id.push(cell.to_string()),
                    "born" => {
                        animal.born = cell
                            .get_datetime()
                            .and_then(|d| d.as_datetime())
                            .map(|d| Local.from_local_datetime(&d).unwrap())
                    }
                    "sire" => animal.sire = some_string(cell.to_string()),
                    "dam" => animal.dam = some_string(cell.to_string()),
                    "sex" => animal.sex = Sex::from(cell.to_string()),
                    _ => (),
                }
            }
            eprintln!("Animal: {:?}", animal);
            flok_guard.animals.push(animal);
        }
    }

    Ok(())
}

fn some_string(to_string: String) -> Option<String> {
    let t = to_string.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn report_lineage(f: &mut Flok) -> Result<()> {
    let mut file = NamedTempFile::with_suffix(".svg")?;

    let unknown = "unknown".to_string();
    let input = "digraph {\n".to_string()
        + "rankdir=LR;\n"
        + &f.animals
            .iter()
            .map(|animal| {
                let id = animal.id.first().unwrap_or(&unknown);
                let mut result = String::new();
                if let Some(dame_id) = &animal.dam {
                    result += &format!("    {} -> {}\n", id, dame_id);
                }
                // if let Some(sire_id) = &animal.sire {
                //     result += &format!("    {} -> {}\n", id, sire_id);
                // }
                result
            })
            .collect::<String>()
        + "}\n";
    let input = input.replace("?", "000").replace("#N/A", "000");
    eprintln!("DOT:\n{}", input);

    // Render the nodes to some rendering backend.
    let gaph = match DotParser::new(input.as_str()).process() {
        Ok(graph) => graph,
        Err(e) => panic!("Unable to parse DOT: {}", e),
    };
    let mut graph_builder = GraphBuilder::new();
    graph_builder.visit_graph(&gaph);
    let mut visual_graph = graph_builder.get();

    let mut svg = SVGWriter::new();
    visual_graph.do_it(false, false, false, &mut svg);

    file.as_file().write_all(svg.finalize().as_bytes())?;
    file.disable_cleanup(true);

    webbrowser::open(&("file://".to_owned() + file.path().to_str().unwrap()))?;

    Ok(())
}
