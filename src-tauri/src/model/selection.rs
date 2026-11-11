#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSnapshot {
    pub text: String,
    pub anchor: ScreenPoint,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectionReading {
    Selected(SelectionSnapshot),
    NothingSelected,
    OwnApplication,
}
