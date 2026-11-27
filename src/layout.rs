//! Where piles and cards sit on the table, plus hit-testing for pointer input.
//!
//! The table has two layouts: a roomy landscape one, and a portrait one for tall screens (phones)
//! that packs the columns closer together and uses the extra height for longer fans.
use bevy::prelude::*;

use crate::{
    cards::{CARD_H, CARD_W},
    game::{Fan, Game, Pile},
    games::GameKind,
};

/// Presses this close (in logical pixels) to the top of the window belong to the HUD.
pub const HUD_HEIGHT_PX: f32 = 40.0;
/// A dropped card must land within this distance of a pile to be placed there.
pub const DROP_DISTANCE: f32 = CARD_H * 0.75;
/// Windows narrower than this (width / height) use the portrait layout.
const PORTRAIT_ASPECT: f32 = 0.85;

/// Positions of the piles on the table, in table units (see [`crate::scene`]).
///
/// Each game places its piles on a grid of columns and rows (see [`crate::game::Spot`]); the
/// layout decides how big the grid's cells are, and how far fanned piles spread.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// The game laid out.
    pub kind: GameKind,
    /// Horizontal distance between the centers of adjacent columns.
    pub column_spacing: f32,
    /// Vertical distance between the centers of adjacent rows.
    pub row_spacing: f32,
    /// Vertical center of the top row.
    pub top_row_y: f32,
    /// Vertical fan offset below a face-down card in a fanned pile.
    pub face_down_offset: f32,
    /// Vertical fan offset below a face-up card in a fanned pile.
    pub face_up_offset: f32,
    /// The longest a fan may be (from its first card to its last). Longer fans are squeezed to
    /// fit, so the table needn't leave room for the longest possible pile.
    pub max_fan: f32,
    /// The area (centered on the table's origin) the camera keeps visible.
    pub size: Vec2,
    /// How far (in radians) the camera leans back from looking straight down.
    pub camera_tilt: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self::landscape(GameKind::default())
    }
}

impl Layout {
    /// The layout for wide windows.
    ///
    /// Window height is usually the limit here, so the table is only as tall as a typical deep
    /// pile needs (see [`crate::game::Shape::landscape_fan`]); longer piles squeeze their fans.
    pub fn landscape(kind: GameKind) -> Self {
        let shape = kind.rules().shape();
        let column_spacing = 96.0;
        let face_down_offset = 10.0;
        let face_up_offset = 28.0;
        let (down, up) = shape.landscape_fan;
        let max_fan = down as f32 * face_down_offset + up as f32 * face_up_offset;
        // From the top: room for the HUD, the rows down to the fanned one, then its fans.
        let hud = 50.0;
        let gap = 25.0;
        let bottom_margin = 15.0;
        let row_spacing = CARD_H + gap;
        let height = hud + CARD_H + shape.fan_row * row_spacing + max_fan + bottom_margin;
        let top_row_y = height / 2.0 - hud - CARD_H / 2.0;
        Self {
            kind,
            column_spacing,
            row_spacing,
            top_row_y,
            face_down_offset,
            face_up_offset,
            max_fan,
            size: Vec2::new(shape.columns as f32 * column_spacing + 30.0, height) * 1.04,
            camera_tilt: 0.3,
        }
    }

    /// The layout for tall windows of `window` logical pixels.
    ///
    /// The columns fill the window's width, and the table is as tall as the window, so the top
    /// row sits just below the HUD and fanned piles can use all of the height below them.
    pub fn portrait(kind: GameKind, window: Vec2) -> Self {
        let shape = kind.rules().shape();
        let column_spacing = CARD_W + 4.0;
        let width = shape.columns as f32 * column_spacing + 8.0;
        let height = width * window.y / window.x;
        let units_per_px = width / window.x;
        let top = height / 2.0 - (HUD_HEIGHT_PX + 8.0) * units_per_px;
        let top_row_y = top - CARD_H / 2.0 - 8.0;
        let row_spacing = CARD_H + 20.0;
        let fan_y = top_row_y - shape.fan_row * row_spacing;
        let face_down_offset = 12.0;
        let bottom = -height / 2.0 + 0.04 * height;
        let max_fan = fan_y - CARD_H / 2.0 - bottom;
        // Fit the fan the game asks for above the bottom.
        let (down, up) = shape.portrait_fan;
        let room = max_fan - down as f32 * face_down_offset;
        Self {
            kind,
            column_spacing,
            row_spacing,
            top_row_y,
            face_down_offset,
            face_up_offset: (room / up.max(1) as f32).clamp(28.0, 48.0),
            max_fan,
            size: Vec2::new(width, height),
            // Less tilt than landscape: perspective shrinks the far (top) row, and on a narrow
            // screen every pixel of width counts.
            camera_tilt: 0.15,
        }
    }

    /// The best layout for `kind` in a window of `window` logical pixels.
    pub fn for_window(kind: GameKind, window: Vec2) -> Self {
        if window.x > 0.0 && window.y > 0.0 && window.x / window.y < PORTRAIT_ASPECT {
            Self::portrait(kind, window)
        } else {
            Self::landscape(kind)
        }
    }

    fn column_x(&self, column: f32) -> f32 {
        let columns = self.kind.rules().shape().columns as f32;
        (column - (columns - 1.0) / 2.0) * self.column_spacing
    }

    /// Center of an empty `pile` (where its bottom card goes).
    pub fn pile_origin(&self, pile: Pile) -> Vec2 {
        let spot = self.kind.rules().spot(pile);
        Vec2::new(
            self.column_x(spot.column),
            self.top_row_y - spot.row * self.row_spacing,
        )
    }

    /// Where cards are dealt from: the stock, or the top middle of the table if there is none.
    pub fn deal_origin(&self, game: &Game) -> Vec2 {
        if game.board().has(Pile::Stock) {
            self.pile_origin(Pile::Stock)
        } else {
            Vec2::new(0.0, self.top_row_y)
        }
    }

    /// Center of the card at `index` in `pile`.
    pub fn card_position(&self, game: &Game, pile: Pile, index: usize) -> Vec2 {
        let origin = self.pile_origin(pile);
        match self.kind.rules().spot(pile).fan {
            Fan::Down => {
                let faces: Vec<bool> = game.pile(pile).iter().map(|slot| slot.face_up).collect();
                origin - Vec2::new(0.0, self.fan_offsets(&faces)[index])
            }
            Fan::Stacked => origin,
        }
    }

    /// How far below the first card each card of a fanned pile sits, given which cards are face
    /// up (bottom card first). Fans longer than [`Layout::max_fan`] are squeezed evenly to fit.
    pub fn fan_offsets(&self, face_up: &[bool]) -> Vec<f32> {
        let mut offsets = Vec::with_capacity(face_up.len());
        let mut fan = 0.0;
        for &up in face_up {
            offsets.push(fan);
            fan += if up {
                self.face_up_offset
            } else {
                self.face_down_offset
            };
        }
        let length = offsets.last().copied().unwrap_or_default();
        if length > self.max_fan {
            let squeeze = self.max_fan / length;
            offsets.iter_mut().for_each(|offset| *offset *= squeeze);
        }
        offsets
    }

    /// What is under `point`: a pile and, unless the pile is empty there, the index of the card.
    pub fn hit_test(&self, game: &Game, point: Vec2) -> Option<(Pile, Option<usize>)> {
        for pile in game.piles() {
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

    /// The pile closest to where a card was dropped, among those `accepts` allows.
    pub fn drop_target(
        &self,
        game: &Game,
        lead_card_center: Vec2,
        accepts: impl Fn(Pile) -> bool,
    ) -> Option<Pile> {
        game.piles()
            .filter(|&pile| accepts(pile))
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

    const KLONDIKE: GameKind = GameKind::Klondike;

    fn deal() -> Game {
        Game::deal(KLONDIKE, new_deck())
    }

    /// A phone held upright, in logical pixels.
    const PHONE: Vec2 = Vec2::new(412.0, 839.0);

    fn layouts() -> [Layout; 2] {
        [
            Layout::landscape(KLONDIKE),
            Layout::portrait(KLONDIKE, PHONE),
        ]
    }

    #[test]
    fn picks_layout_by_window_shape() {
        assert_eq!(
            Layout::for_window(KLONDIKE, Vec2::new(1280.0, 720.0)),
            Layout::landscape(KLONDIKE)
        );
        assert_eq!(
            Layout::for_window(KLONDIKE, Vec2::new(800.0, 800.0)),
            Layout::landscape(KLONDIKE)
        );
        assert_eq!(
            Layout::for_window(KLONDIKE, PHONE),
            Layout::portrait(KLONDIKE, PHONE)
        );
        assert_eq!(
            Layout::for_window(KLONDIKE, Vec2::ZERO),
            Layout::landscape(KLONDIKE)
        );
    }

    #[test]
    fn portrait_cards_are_bigger_on_a_phone() {
        // On screen, a card's width is its share of the visible table width.
        let landscape_share = CARD_W / Layout::landscape(KLONDIKE).size.x;
        let portrait_share = CARD_W / Layout::portrait(KLONDIKE, PHONE).size.x;
        assert!(portrait_share > landscape_share * 1.1);
        // And the fan of face-up cards uses the extra height.
        assert!(
            Layout::portrait(KLONDIKE, PHONE).face_up_offset
                > Layout::landscape(KLONDIKE).face_up_offset
        );
    }

    #[test]
    fn hit_top_card_of_tableau() {
        let game = deal();
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
        let game = deal();
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
        let game = deal();
        for layout in layouts() {
            let near_t3 = layout.card_position(&game, Pile::Tableau(3), 3) + Vec2::new(20.0, -15.0);
            let any = |_| true;
            assert_eq!(
                layout.drop_target(&game, near_t3, any),
                Some(Pile::Tableau(3))
            );
            let near_f0 = layout.pile_origin(Pile::Foundation(0)) + Vec2::new(-10.0, 10.0);
            assert_eq!(
                layout.drop_target(&game, near_f0, any),
                Some(Pile::Foundation(0))
            );
            assert_eq!(
                layout.drop_target(&game, Vec2::new(0.0, -2000.0), any),
                None
            );
            // Piles that don't accept the cards are passed over for ones that do.
            let not_t3 = |pile| pile != Pile::Tableau(3);
            assert_ne!(
                layout.drop_target(&game, near_t3, not_t3),
                Some(Pile::Tableau(3))
            );
        }
    }

    #[test]
    fn short_fans_are_not_squeezed() {
        let layout = Layout::landscape(KLONDIKE);
        let offsets = layout.fan_offsets(&[false, false, true, true]);
        assert_eq!(offsets, [0.0, 10.0, 20.0, 48.0]);
        assert!(layout.fan_offsets(&[]).is_empty());
    }

    #[test]
    fn long_fans_are_squeezed_to_fit() {
        let layout = Layout::landscape(KLONDIKE);
        // Worst case: 6 face-down cards and a full King-to-Ace run.
        let mut faces = vec![false; 6];
        faces.extend([true; 13]);
        let offsets = layout.fan_offsets(&faces);
        assert!((offsets[18] - layout.max_fan).abs() < 0.01);
        // Still in order, and face-up cards still further apart than face-down ones.
        assert!(offsets.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(offsets[18] - offsets[17] > offsets[1] - offsets[0]);
    }

    #[test]
    fn landscape_cards_are_bigger_on_wide_windows() {
        // On a short, wide window the table's height is the limit, and a card's share of it
        // decides its size on screen. The old landscape table was 806 units tall.
        let share = CARD_H / Layout::landscape(KLONDIKE).size.y;
        assert!(share > CARD_H / 806.0 * 1.3, "share = {share}");
    }

    #[test]
    fn deepest_tableau_fits_on_table() {
        for layout in layouts() {
            // Worst case: 6 face-down cards and a full King-to-Ace run.
            let mut faces = vec![false; 6];
            faces.extend([true; 13]);
            let fan = layout.fan_offsets(&faces)[18];
            let tableau_y = layout.pile_origin(Pile::Tableau(0)).y;
            let bottom = tableau_y - fan - CARD_H / 2.0;
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
