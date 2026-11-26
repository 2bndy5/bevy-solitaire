//! Where piles and cards sit on the table, plus hit-testing for pointer input.
//!
//! The table has two layouts: a roomy landscape one, and a portrait one for tall screens (phones)
//! that packs the columns closer together and uses the extra height for longer fans.
use bevy::prelude::*;

use crate::{
    cards::{CARD_H, CARD_W},
    game::{Klondike, Pile},
};

/// Presses this close (in logical pixels) to the top of the window belong to the HUD.
pub const HUD_HEIGHT_PX: f32 = 40.0;
/// A dropped card must land within this distance of a pile to be placed there.
pub const DROP_DISTANCE: f32 = CARD_H * 0.75;
/// Windows narrower than this (width / height) use the portrait layout.
const PORTRAIT_ASPECT: f32 = 0.85;

/// Positions of the piles on the table, in table units (see [`crate::scene`]).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Horizontal distance between the centers of adjacent columns.
    pub column_spacing: f32,
    /// Vertical center of the stock, waste and foundations.
    pub top_row_y: f32,
    /// Vertical center of the first card in each tableau.
    pub tableau_y: f32,
    /// Vertical fan offset below a face-down tableau card.
    pub face_down_offset: f32,
    /// Vertical fan offset below a face-up tableau card.
    pub face_up_offset: f32,
    /// The area (centered on the table's origin) the camera keeps visible.
    pub size: Vec2,
    /// How far (in radians) the camera leans back from looking straight down.
    pub camera_tilt: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self::landscape()
    }
}

impl Layout {
    /// The layout for wide windows. Its size leaves room above the top row for the HUD.
    pub fn landscape() -> Self {
        let column_spacing = 100.0;
        let top_row_y = 270.0;
        Self {
            column_spacing,
            top_row_y,
            tableau_y: top_row_y - CARD_H - 30.0,
            face_down_offset: 10.0,
            face_up_offset: 28.0,
            size: Vec2::new(7.0 * column_spacing + 40.0, 760.0) * 1.06,
            camera_tilt: 0.3,
        }
    }

    /// The layout for tall windows of `window` logical pixels.
    ///
    /// The seven columns fill the window's width, and the table is as tall as the window, so the
    /// top row sits just below the HUD and long tableau fans can use all of the height below it.
    pub fn portrait(window: Vec2) -> Self {
        let column_spacing = CARD_W + 4.0;
        let width = 7.0 * column_spacing + 8.0;
        let height = width * window.y / window.x;
        let units_per_px = width / window.x;
        let top = height / 2.0 - (HUD_HEIGHT_PX + 8.0) * units_per_px;
        let top_row_y = top - CARD_H / 2.0 - 8.0;
        let tableau_y = top_row_y - CARD_H - 20.0;
        let face_down_offset = 12.0;
        // Fit the worst case (6 face-down cards and a full King-to-Ace run) above the bottom.
        let bottom = -height / 2.0 + 0.04 * height;
        let room = tableau_y - CARD_H / 2.0 - bottom - 6.0 * face_down_offset;
        Self {
            column_spacing,
            top_row_y,
            tableau_y,
            face_down_offset,
            face_up_offset: (room / 12.0).clamp(28.0, 48.0),
            size: Vec2::new(width, height),
            // Less tilt than landscape: perspective shrinks the far (top) row, and on a narrow
            // screen every pixel of width counts.
            camera_tilt: 0.15,
        }
    }

    /// The best layout for a window of `window` logical pixels.
    pub fn for_window(window: Vec2) -> Self {
        if window.x > 0.0 && window.y > 0.0 && window.x / window.y < PORTRAIT_ASPECT {
            Self::portrait(window)
        } else {
            Self::landscape()
        }
    }

    fn column_x(&self, column: usize) -> f32 {
        (column as f32 - 3.0) * self.column_spacing
    }

    /// Center of an empty `pile` (where its bottom card goes).
    pub fn pile_origin(&self, pile: Pile) -> Vec2 {
        match pile {
            Pile::Stock => Vec2::new(self.column_x(0), self.top_row_y),
            Pile::Waste => Vec2::new(self.column_x(1), self.top_row_y),
            Pile::Foundation(i) => Vec2::new(self.column_x(3 + i), self.top_row_y),
            Pile::Tableau(i) => Vec2::new(self.column_x(i), self.tableau_y),
        }
    }

    /// Center of the card at `index` in `pile`.
    pub fn card_position(&self, game: &Klondike, pile: Pile, index: usize) -> Vec2 {
        let origin = self.pile_origin(pile);
        match pile {
            Pile::Tableau(_) => {
                let fan: f32 = game.pile(pile)[..index]
                    .iter()
                    .map(|slot| {
                        if slot.face_up {
                            self.face_up_offset
                        } else {
                            self.face_down_offset
                        }
                    })
                    .sum();
                origin - Vec2::new(0.0, fan)
            }
            _ => origin,
        }
    }

    /// What is under `point`: a pile and, unless the pile is empty there, the index of the card.
    pub fn hit_test(&self, game: &Klondike, point: Vec2) -> Option<(Pile, Option<usize>)> {
        for pile in Klondike::all_piles() {
            let len = game.pile(pile).len();
            // Check from the top card down, since upper cards overlap lower ones.
            for index in (0..len).rev() {
                if contains(self.card_position(game, pile, index), point) {
                    return Some((pile, Some(index)));
                }
            }
            if len == 0 && contains(self.pile_origin(pile), point) {
                return Some((pile, None));
            }
        }
        None
    }

    /// The pile (foundation or tableau) closest to where a card was dropped.
    pub fn drop_target(&self, game: &Klondike, lead_card_center: Vec2) -> Option<Pile> {
        (0..Pile::FOUNDATIONS)
            .map(Pile::Foundation)
            .chain((0..Pile::TABLEAUS).map(Pile::Tableau))
            .map(|pile| {
                let len = game.pile(pile).len();
                let top = match len {
                    0 => self.pile_origin(pile),
                    _ => self.card_position(game, pile, len - 1),
                };
                (pile, top.distance(lead_card_center))
            })
            .filter(|(_, distance)| *distance < DROP_DISTANCE)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(pile, _)| pile)
    }
}

fn contains(center: Vec2, point: Vec2) -> bool {
    (point - center)
        .abs()
        .cmple(Vec2::new(CARD_W, CARD_H) / 2.0)
        .all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::new_deck;

    /// A phone held upright, in logical pixels.
    const PHONE: Vec2 = Vec2::new(412.0, 839.0);

    fn layouts() -> [Layout; 2] {
        [Layout::landscape(), Layout::portrait(PHONE)]
    }

    #[test]
    fn picks_layout_by_window_shape() {
        assert_eq!(
            Layout::for_window(Vec2::new(1280.0, 720.0)),
            Layout::landscape()
        );
        assert_eq!(
            Layout::for_window(Vec2::new(800.0, 800.0)),
            Layout::landscape()
        );
        assert_eq!(Layout::for_window(PHONE), Layout::portrait(PHONE));
        assert_eq!(Layout::for_window(Vec2::ZERO), Layout::landscape());
    }

    #[test]
    fn portrait_cards_are_bigger_on_a_phone() {
        // On screen, a card's width is its share of the visible table width.
        let landscape_share = CARD_W / Layout::landscape().size.x;
        let portrait_share = CARD_W / Layout::portrait(PHONE).size.x;
        assert!(portrait_share > landscape_share * 1.2);
        // And the fan of face-up cards uses the extra height.
        assert!(Layout::portrait(PHONE).face_up_offset > Layout::landscape().face_up_offset);
    }

    #[test]
    fn hit_top_card_of_tableau() {
        let game = Klondike::deal(new_deck());
        for layout in layouts() {
            let pile = Pile::Tableau(6);
            let top = layout.card_position(&game, pile, 6);
            assert_eq!(layout.hit_test(&game, top), Some((pile, Some(6))));
            // The visible sliver of a lower card hits that card.
            let lower = layout.card_position(&game, pile, 2)
                + Vec2::new(0.0, (CARD_H - layout.face_down_offset) / 2.0);
            assert_eq!(layout.hit_test(&game, lower), Some((pile, Some(2))));
        }
    }

    #[test]
    fn hit_empty_piles() {
        let game = Klondike::deal(new_deck());
        for layout in layouts() {
            assert_eq!(
                layout.hit_test(&game, layout.pile_origin(Pile::Waste)),
                Some((Pile::Waste, None))
            );
            assert_eq!(
                layout.hit_test(&game, layout.pile_origin(Pile::Foundation(1))),
                Some((Pile::Foundation(1), None))
            );
            assert_eq!(layout.hit_test(&game, Vec2::new(0.0, -2000.0)), None);
        }
    }

    #[test]
    fn drop_on_nearest_pile() {
        let game = Klondike::deal(new_deck());
        for layout in layouts() {
            let near_t3 = layout.card_position(&game, Pile::Tableau(3), 3) + Vec2::new(20.0, -15.0);
            assert_eq!(layout.drop_target(&game, near_t3), Some(Pile::Tableau(3)));
            let near_f0 = layout.pile_origin(Pile::Foundation(0)) + Vec2::new(-10.0, 10.0);
            assert_eq!(
                layout.drop_target(&game, near_f0),
                Some(Pile::Foundation(0))
            );
            assert_eq!(layout.drop_target(&game, Vec2::new(0.0, -2000.0)), None);
        }
    }

    #[test]
    fn deepest_tableau_fits_on_table() {
        for layout in layouts() {
            // Worst case: 6 face-down cards and a full King-to-Ace run.
            let bottom = layout.tableau_y
                - 6.0 * layout.face_down_offset
                - 12.0 * layout.face_up_offset
                - CARD_H / 2.0;
            assert!(
                bottom >= -layout.size.y / 2.0,
                "{layout:?}: bottom = {bottom}"
            );
            assert!(layout.top_row_y + CARD_H / 2.0 <= layout.size.y / 2.0);
            // The columns fit side by side.
            assert!(7.0 * layout.column_spacing <= layout.size.x);
            assert!(layout.column_spacing > CARD_W);
        }
    }
}
