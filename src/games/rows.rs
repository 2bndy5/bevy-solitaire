//! Cards laid out in overlapping rows (TriPeaks, Pyramid), one card per tableau pile.
//!
//! Each card sits at a column and a row. A card is covered by the cards in the next row that sit
//! half a column to either side of it, and can only be played once those are gone.
use crate::game::{Board, Pile};

/// Where each tableau card sits: `(column, row)`, indexed by tableau number.
pub type RowSpots = [(f32, usize)];

/// The tableau cards that cover tableau card `index`, whether or not they're still there.
pub fn covering(spots: &RowSpots, index: usize) -> impl Iterator<Item = usize> + '_ {
    let (column, row) = spots[index];
    spots
        .iter()
        .enumerate()
        .filter(move |(_, (c, r))| *r == row + 1 && (c - column).abs() == 0.5)
        .map(|(i, _)| i)
}

/// Is tableau card `index` still there, with no card left on top of it?
pub fn is_exposed(board: &Board, spots: &RowSpots, index: usize) -> bool {
    !board.pile(Pile::Tableau(index)).is_empty()
        && covering(spots, index).all(|i| board.pile(Pile::Tableau(i)).is_empty())
}

/// The tableau piles, front row first, so the cards in front are hit first.
pub fn piles_front_first(spots: &RowSpots) -> impl Iterator<Item = Pile> + '_ {
    let mut order: Vec<usize> = (0..spots.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(spots[i].1));
    order.into_iter().map(Pile::Tableau)
}

/// Is every tableau pile empty?
pub fn all_cleared(board: &Board, spots: &RowSpots) -> bool {
    (0..spots.len()).all(|i| board.pile(Pile::Tableau(i)).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small pyramid: one card on top of two.
    const SPOTS: [(f32, usize); 3] = [(0.5, 0), (0.0, 1), (1.0, 1)];

    #[test]
    fn cards_are_covered_by_the_row_in_front() {
        assert_eq!(covering(&SPOTS, 0).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(covering(&SPOTS, 1).count(), 0);
        let order: Vec<Pile> = piles_front_first(&SPOTS).collect();
        assert_eq!(order[2], Pile::Tableau(0));
    }
}
