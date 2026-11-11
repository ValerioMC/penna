use crate::model::selection::ScreenPoint;

const ANCHOR_OFFSET: f64 = 8.0;
const SCREEN_MARGIN: f64 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ScreenArea {
    pub fn contains(&self, point: ScreenPoint) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }
}

pub fn place_near_anchor(
    anchor: ScreenPoint,
    size: WindowSize,
    screen: Option<ScreenArea>,
) -> ScreenPoint {
    let preferred = ScreenPoint {
        x: anchor.x + ANCHOR_OFFSET,
        y: anchor.y + ANCHOR_OFFSET,
    };
    match screen {
        Some(area) => keep_inside(preferred, anchor, size, area),
        None => preferred,
    }
}

fn keep_inside(
    preferred: ScreenPoint,
    anchor: ScreenPoint,
    size: WindowSize,
    area: ScreenArea,
) -> ScreenPoint {
    let rightmost_x = area.x + area.width - size.width - SCREEN_MARGIN;
    let x = preferred.x.min(rightmost_x).max(area.x + SCREEN_MARGIN);
    let fits_below = preferred.y + size.height <= area.y + area.height - SCREEN_MARGIN;
    let y = if fits_below {
        preferred.y
    } else {
        (anchor.y - ANCHOR_OFFSET - size.height).max(area.y + SCREEN_MARGIN)
    };
    ScreenPoint { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: ScreenArea = ScreenArea {
        x: 0.0,
        y: 0.0,
        width: 1000.0,
        height: 800.0,
    };
    const SIZE: WindowSize = WindowSize {
        width: 300.0,
        height: 200.0,
    };

    #[test]
    fn sits_below_and_right_of_the_anchor_when_there_is_room() {
        let placed = place_near_anchor(ScreenPoint { x: 100.0, y: 100.0 }, SIZE, Some(SCREEN));
        assert_eq!(placed, ScreenPoint { x: 108.0, y: 108.0 });
    }

    #[test]
    fn shifts_left_when_it_would_cross_the_right_edge() {
        let placed = place_near_anchor(ScreenPoint { x: 950.0, y: 100.0 }, SIZE, Some(SCREEN));
        assert_eq!(placed.x, 692.0);
    }

    #[test]
    fn moves_above_the_anchor_when_it_would_cross_the_bottom_edge() {
        let placed = place_near_anchor(ScreenPoint { x: 100.0, y: 700.0 }, SIZE, Some(SCREEN));
        assert_eq!(placed.y, 492.0);
    }

    #[test]
    fn follows_the_anchor_without_a_known_screen() {
        let placed = place_near_anchor(
            ScreenPoint {
                x: 5000.0,
                y: 5000.0,
            },
            SIZE,
            None,
        );
        assert_eq!(
            placed,
            ScreenPoint {
                x: 5008.0,
                y: 5008.0
            }
        );
    }
}
