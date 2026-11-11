use crate::model::error::PennaResult;
use crate::model::selection::SelectionReading;

pub trait SelectionReader {
    fn read_selection(&mut self) -> PennaResult<SelectionReading>;
}
