
use fltk::{
    app::awake, enums::*, frame::Frame, group::*, prelude::{GroupExt, WidgetBase, WidgetExt}, widget::Widget
};

pub struct Form {
    pub pack: Pack,
    name_width: i32,
    height: i32,
}
impl Form {
    pub fn new() -> Form {
        let mut pack = Pack::default_fill();
        pack.set_spacing(3);
        Form {
            pack,
            name_width: 50,
            height: 20,
        }
    }
    pub fn end(&self) {
        self.pack.end()
    }

//    pub fn master_detail(&mut self, label:&str,create_table:fn(f:&mut Form)->Widget, create_detail)

    pub fn row(&mut self, label: &str, create_widget: fn(f: &mut Form) -> Widget) -> Widget {
        let mut row = Row::default().with_size(0, self.height);
        row.add(&Frame::default().with_size(0, self.height).with_label(label));
        row.add(&create_widget(self));
        row.end();
        let base_widget = row.as_base_widget();
        self.pack.add(&base_widget);
        self.pack.redraw();
        awake();
        base_widget
    }
    pub fn sub(
        &mut self,
        label: &str,
        create_widgets: Vec<fn(f: &mut Form) -> Widget>,
    ) -> Vec<Widget> {
        const MARGIN: i32 = 10;
        let sz = self.height;

        let mut row = Row::default_fill();
        row.set_margins(MARGIN, MARGIN, MARGIN, MARGIN);
        row.add(&Frame::default().with_size(MARGIN, MARGIN));
        row.set_frame(FrameType::GleamUpFrame);
        row.set_color(Color::Blue);

        let mut frame = Pack::default_fill();
        frame.set_spacing(MARGIN);
        frame.set_color(Color::Green);

        frame.add(&Frame::default().with_size(100, sz).with_label(label));

        let collect: Vec<Widget> = create_widgets.iter().map(|c| c(self)).collect();
        collect.iter().for_each(|r| frame.add(r));

        frame.end();
        row.add(&Frame::default().with_size(MARGIN, MARGIN));
        row.end();
        row.as_base_widget().set_size(
            0,
            MARGIN + sz + MARGIN + collect.iter().map(|w| w.height() + MARGIN).sum::<i32>(),
        );
        collect
    }

    pub(crate) fn clear(&mut self) {
        self.pack.clear();
    }

    pub(crate) fn as_base_widget(&self) -> Widget {
        self.pack.as_base_widget()
    }
}
