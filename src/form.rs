use fltk::prelude::{GroupExt, WidgetBase, WidgetExt};

use fltk::frame::Frame;

use fltk::group::Grid;

use fltk::widget::Widget;

use anyhow::Result;

const SIZE: i32 = 20;

pub fn create_form(fields: Vec<(&str, &dyn FromWidget)>) -> Result<Widget> {
    let mut grid = Grid::default();
    grid.set_layout(1 + fields.len() as i32, 2);
    let mut height = 0;
    for (row, field) in fields.into_iter().enumerate() {
        grid.set_widget(&mut Frame::default().with_label(field.0), row, 0)?;
        let mut widget = field.1.to_widget();
        grid.add(&widget);
        grid.set_widget(&mut widget, row, 1)?;

        {
            let row = row as i32;
            let size = widget.height().max(SIZE);
            height += size;
            grid.set_row_height(row, size);
            grid.set_row_weight(row, 0);
        }
    }
    grid.set_size(0, height);
    grid.end();
    Ok(grid.as_base_widget())
}

pub trait FromWidget {
    fn to_widget(&self) -> Widget;
}

impl<W: WidgetBase> FromWidget for W {
    fn to_widget(&self) -> Widget {
        self.as_base_widget()
    }
}

/// This doesn't add value, but keeps us honest
pub trait Editor<T> {
    fn set_value(&mut self, value: &T);
    fn commit(&mut self);
}

pub fn display_error<T>(action: &str, result: Result<T>) {
    if let Err(err) = result {
        fltk::dialog::alert_default(&format!("{action}: {err}"));
    }
}
