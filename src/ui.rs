use conrod::{ widget, color, Colorable, Sizeable, Positionable, Widget };

struct Scale {
    w: f64,
    h: f64,
}

struct Position {
    x: f64,
    y: f64,
}


///
/// line util
/// 
pub struct Line {
    id: widget::Id,
    col: conrod::Color
}

impl Line {
    pub fn new(id: widget::Id, col: conrod::Color) -> Line{
        Line { id, col }
    }
    pub fn update(&mut self, ui: &mut conrod::UiCell, x0: f64, y0: f64, x1: f64, y1: f64){
        widget::Line::new([x0, y0], [x1, y1])
            .color(self.col)
            .thickness(0.5)
            .set(self.id, ui);
    }
}


///
/// text util
/// 
pub struct Text {
    pub value: String,
    id: widget::Id,
    position: Position,
    scale: Scale,
}

impl Text {
    pub fn new(id: widget::Id, value:&str, x:f64, y:f64, w:f64, h:f64) -> Text {
        Text { id, value: value.to_string(), position: Position{x, y}, scale: Scale{w, h} }
    }

    pub fn update(&mut self, ui: &mut conrod::UiCell, text: &str){
        self.value = text.to_string();
        widget::Text::new(text)
            .font_size(10)
            .top_left_with_margins_on(ui.window, self.position.y, self.position.x)
            .line_spacing(10.0)
            .w_h(self.scale.w, self.scale.h)
            .color(color::BLACK)
            .set(self.id, ui);
    }
}


///
/// slider util
/// 
pub struct Slider {
    pub value: f64,
    min: f64,
    max: f64,
    id: widget::Id,
    position: Position,
    scale: Scale,
}

impl Slider {
    pub fn new(id: widget::Id, value:f64, min: f64, max: f64, x:f64, y:f64, w:f64, h:f64) -> Slider {
        Slider{id, value, min, max, position: Position{x, y}, scale: Scale{w, h}}
    }

    pub fn update(&mut self, ui: &mut conrod::UiCell){
        let s = widget::Slider::new(self.value, self.min, self.max)
            .top_left_with_margins_on(ui.window, self.position.y, self.position.x)
            .w_h(self.scale.w, self.scale.h)
            .color(color::GRAY)
            .set(self.id, ui);
        
        if let Some(v) = s { self.value = v; }
    }
}