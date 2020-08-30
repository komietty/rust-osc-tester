#[macro_use]
extern crate conrod;

pub mod ui;

widget_ids!(
    pub struct Ids{
        sender,
        output,
        sliders[],
        line_h,
     }
);