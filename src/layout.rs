//! Where piles and cards sit on the table, plus hit-testing for pointer input.
use bevy::prelude::*;

use crate::{
    cards::{CARD_H, CARD_W},
    game::{Klondike, Pile},
};

/// Horizontal distance between the centers of adjacent columns.
pub const COLUMN_SPACING: f32 = 100.0;
/// Vertical center of the stock, waste and foundations.
pub const TOP_ROW_Y: f32 = 270.0;
/// Vertical center of the first card in each tableau.
pub const TABLEAU_Y: f32 = TOP_ROW_Y - CARD_H - 30.0;
/// Vertical fan offset below a face-down tableau card.
pub const FACE_DOWN_OFFSET: f32 = 10.0;
/// Vertical fan offset below a face-up tableau card.
pub const FACE_UP_OFFSET: f32 = 28.0;
/// The smallest area (world units) the camera always keeps visible.
///
/// The extra height above the top row leaves room for the HUD.
pub const TABLE_SIZE: Vec2 = Vec2::new(7.0 * COLUMN_SPACING + 40.0, 760.0);
/// A dropped card must land within this distance of a pile to be placed there.
pub const DROP_DISTANCE: f32 = CARD_H * 0.75;

fn column_x(column: usize) -> f32 {
    (column as f32 - 3.0) * COLUMN_SPACING
}

/// Center of an empty `pile` (where its bottom card goes).
pub fn pile_origin(pile: Pile) -> Vec2 {
    match pile {
        Pile::Stock => Vec2::new(column_x(0), TOP_ROW_Y),
        Pile::Waste => Vec2::new(column_x(1), TOP_ROW_Y),
        Pile::Foundation(i) => Vec2::new(column_x(3 + i), TOP_ROW_Y),
        Pile::Tableau(i) => Vec2::new(column_x(i), TABLEAU_Y),
    }
}

/// Center of the card at `index` in `pile`.
pub fn card_position(game: &Klondike, pile: Pile, index: usize) -> Vec2 {
    let origin = pile_origin(pile);
    match pile {
        Pile::Tableau(_) => {
            let fan: f32 = game.pile(pile)[..index]
                .iter()
                .map(|slot| {
                    if slot.face_up {
                        FACE_UP_OFFSET
                    } else {
                        FACE_DOWN_OFFSET
                    }
                })
                .sum();
            origin - Vec2::new(0.0, fan)
        }
        _ => origin,
    }
}

fn contains(center: Vec2, point: Vec2) -> bool {
    (point - center)
        .abs()
        .cmple(Vec2::new(CARD_W, CARD_H) / 2.0)
        .all()
}

/// What is under `point`: a pile and, unless the pile is empty there, the index of the card.
pub fn hit_test(game: &Klondike, point: Vec2) -> Option<(Pile, Option<usize>)> {
    for pile in Klondike::all_piles() {
        let len = game.pile(pile).len();
        // Check from the top card down, since upper cards overlap lower ones.
        for index in (0..len).rev() {
            if contains(card_position(game, pile, index), point) {
                return Some((pile, Some(index)));
            }
        }
        if len == 0 && contains(pile_origin(pile), point) {
            return Some((pile, None));
        }
    }
    None
}

/// The pile (foundation or tableau) closest to where a card was dropped.
pub fn drop_target(game: &Klondike, lead_card_center: Vec2) -> Option<Pile> {
    (0..Pile::FOUNDATIONS)
        .map(Pile::Foundation)
        .chain((0..Pile::TABLEAUS).map(Pile::Tableau))
        .map(|pile| {
            let len = game.pile(pile).len();
            let top = match len {
                0 => pile_origin(pile),
                _ => card_position(game, pile, len - 1),
            };
            (pile, top.distance(lead_card_center))
        })
        .filter(|(_, distance)| *distance < DROP_DISTANCE)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(pile, _)| pile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::new_deck;

    #[test]
    fn hit_top_card_of_tableau() {
        let game = Klondike::deal(new_deck());
        let pile = Pile::Tableau(6);
        let top = card_position(&game, pile, 6);
        assert_eq!(hit_test(&game, top), Some((pile, Some(6))));
        // The visible sliver of a lower card hits that card.
        let lower =
            card_position(&game, pile, 2) + Vec2::new(0.0, (CARD_H - FACE_DOWN_OFFSET) / 2.0);
        assert_eq!(hit_test(&game, lower), Some((pile, Some(2))));
    }

    #[test]
    fn hit_empty_piles() {
        let game = Klondike::deal(new_deck());
        assert_eq!(
            hit_test(&game, pile_origin(Pile::Waste)),
            Some((Pile::Waste, None))
        );
        assert_eq!(
            hit_test(&game, pile_origin(Pile::Foundation(1))),
            Some((Pile::Foundation(1), None))
        );
        assert_eq!(hit_test(&game, Vec2::new(0.0, -1000.0)), None);
    }

    #[test]
    fn drop_on_nearest_pile() {
        let game = Klondike::deal(new_deck());
        let near_t3 = card_position(&game, Pile::Tableau(3), 3) + Vec2::new(20.0, -15.0);
        assert_eq!(drop_target(&game, near_t3), Some(Pile::Tableau(3)));
        let near_f0 = pile_origin(Pile::Foundation(0)) + Vec2::new(-10.0, 10.0);
        assert_eq!(drop_target(&game, near_f0), Some(Pile::Foundation(0)));
        assert_eq!(drop_target(&game, Vec2::new(0.0, -1000.0)), None);
    }

    #[test]
    fn deepest_tableau_fits_on_table() {
        // Worst case: 6 face-down cards and a full King-to-Ace run.
        let bottom = TABLEAU_Y - 6.0 * FACE_DOWN_OFFSET - 12.0 * FACE_UP_OFFSET - CARD_H / 2.0;
        assert!(bottom >= -TABLE_SIZE.y / 2.0, "bottom = {bottom}");
        const { assert!(TOP_ROW_Y + CARD_H / 2.0 <= TABLE_SIZE.y / 2.0) };
    }
}
