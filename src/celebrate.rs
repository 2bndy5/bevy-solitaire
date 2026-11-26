//! The win celebration: cards leap off the foundations and bounce across the table, then the win
//! screen appears.
use std::f32::consts::TAU;

use bevy::{prelude::*, window::PrimaryWindow};
use rand::{RngExt, rng};

use crate::{
    cards::Card,
    game::{Klondike, Pile},
    hud::{GameClock, format_duration},
    layout::HUD_HEIGHT_PX,
    menu::spawn_win_overlay,
    state::GameState,
    table::{CardEntities, REST_HEIGHT, Table, card_rotation},
};

/// Downward acceleration, in table units per second squared.
const GRAVITY: f32 = 1400.0;
/// Fraction of its speed a card keeps when it bounces off the table.
const BOUNCE: f32 = 0.55;
/// Fraction of its horizontal speed a card keeps per second while sliding on the table.
const SLIDE_FRICTION: f32 = 0.2;
/// Time between cards leaping off the foundations.
const LAUNCH_INTERVAL: f64 = 0.07;
/// How long after the last card leaps before the win screen appears.
const OVERLAY_DELAY: f64 = 1.5;
/// Height between landed cards, so cards lying on each other don't z-fight.
const LANDED_GAP: f32 = 0.3;

/// A card flying (or sliding) freely during the celebration.
#[derive(Component)]
pub struct Flying {
    velocity: Vec3,
    /// How fast the card spins about the vertical axis, in radians per second.
    spin: f32,
    /// How far the card has turned about the vertical axis.
    yaw: f32,
    /// The height at which this card lands.
    floor: f32,
}

/// Progress of the celebration.
#[derive(Resource)]
struct Celebration {
    /// Cards still to leap, in leaping order (last first).
    to_launch: Vec<Card>,
    next_launch_at: f64,
    landed: usize,
    overlay_at: Option<f64>,
}

pub struct CelebratePlugin;

impl Plugin for CelebratePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Win), start_celebration)
            .add_systems(
                Update,
                (launch_cards, fly_cards, show_overlay)
                    .chain()
                    .run_if(in_state(GameState::Win)),
            )
            .add_systems(OnExit(GameState::Win), end_celebration);
    }
}

/// The order in which cards leap: Kings first, round-robin across the foundations.
fn launch_order(game: &Klondike) -> Vec<Card> {
    let mut order = Vec::new();
    for rank in (0..13).rev() {
        for foundation in 0..Pile::FOUNDATIONS {
            if let Some(slot) = game.pile(Pile::Foundation(foundation)).get(rank) {
                order.push(slot.card);
            }
        }
    }
    order.reverse(); // popped from the end
    order
}

fn start_celebration(mut commands: Commands, table: Res<Table>, time: Res<Time>) {
    commands.insert_resource(Celebration {
        to_launch: launch_order(&table.0),
        // Let the last card settle on its foundation first.
        next_launch_at: time.elapsed_secs_f64() + 0.4,
        landed: 0,
        overlay_at: None,
    });
}

fn end_celebration(mut commands: Commands) {
    commands.remove_resource::<Celebration>();
}

fn launch_cards(
    mut commands: Commands,
    time: Res<Time>,
    entities: Res<CardEntities>,
    mut celebration: ResMut<Celebration>,
    transforms: Query<&Transform>,
) {
    let now = time.elapsed_secs_f64();
    let mut rng = rng();
    while now >= celebration.next_launch_at
        && let Some(card) = celebration.to_launch.pop()
    {
        celebration.next_launch_at += LAUNCH_INTERVAL;
        let Some(&entity) = entities.0.get(&card) else {
            continue;
        };
        // Leap up and out in a random direction, mostly toward the player.
        let angle = rng.random_range(0.15 * TAU..0.85 * TAU) / 2.0;
        let speed = rng.random_range(180.0..420.0);
        let velocity = Vec3::new(
            angle.cos() * speed,
            rng.random_range(350.0..650.0),
            angle.sin() * speed,
        );
        let floor = REST_HEIGHT + celebration.landed as f32 * LANDED_GAP;
        celebration.landed += 1;
        let yaw = transforms
            .get(entity)
            .map(|t| t.rotation.to_euler(EulerRot::YXZ).0)
            .unwrap_or_default();
        commands.entity(entity).insert(Flying {
            velocity,
            spin: rng.random_range(-12.0..12.0),
            yaw,
            floor,
        });
        if celebration.to_launch.is_empty() {
            celebration.overlay_at = Some(now + OVERLAY_DELAY);
        }
    }
}

fn fly_cards(time: Res<Time>, mut cards: Query<(&mut Transform, &mut Flying)>) {
    let dt = time.delta_secs();
    for (mut transform, mut flying) in &mut cards {
        let Flying {
            velocity,
            spin,
            yaw,
            floor,
        } = &mut *flying;
        velocity.y -= GRAVITY * dt;
        transform.translation += *velocity * dt;
        if transform.translation.y <= *floor {
            transform.translation.y = *floor;
            if velocity.y < 0.0 {
                velocity.y = -velocity.y * BOUNCE;
                if velocity.y < 40.0 {
                    velocity.y = 0.0;
                }
                *spin *= BOUNCE;
            }
            // Slide to a stop.
            let keep = SLIDE_FRICTION.powf(dt);
            velocity.x *= keep;
            velocity.z *= keep;
            *spin *= keep;
        }
        *yaw += *spin * dt;
        transform.rotation = Quat::from_rotation_y(*yaw) * card_rotation(true);
    }
}

/// Shows the win screen after the celebration, or as soon as the player clicks or taps.
#[allow(clippy::too_many_arguments)]
fn show_overlay(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    table: Res<Table>,
    clock: Res<GameClock>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut celebration: ResMut<Celebration>,
) {
    let now = time.elapsed_secs_f64();
    let pressed = mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed();
    let due = celebration.overlay_at.is_some_and(|at| now >= at);
    let shown = celebration.overlay_at == Some(f64::INFINITY);
    if shown || !(due || pressed) {
        return;
    }
    // Don't treat a press on the HUD (e.g. New Game) as "skip ahead".
    if pressed
        && !due
        && window
            .cursor_position()
            .is_some_and(|pos| pos.y < HUD_HEIGHT_PX)
    {
        return;
    }
    celebration.overlay_at = Some(f64::INFINITY);
    let time_played = format_duration(clock.elapsed(now));
    spawn_win_overlay(&mut commands, &asset_server, table.0.moves, &time_played);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::{Rank, new_deck};

    #[test]
    fn kings_leap_first_and_every_card_leaps_once() {
        let mut won = Klondike::with_waste(new_deck().into_iter().rev().collect());
        while won.move_to_foundation(Pile::Waste) {}
        assert!(won.is_won());

        let mut order = launch_order(&won);
        assert_eq!(order.len(), 52);
        let unique: std::collections::HashSet<_> = order.iter().collect();
        assert_eq!(unique.len(), 52);
        for _ in 0..4 {
            assert_eq!(order.pop().unwrap().rank, Rank::King);
        }
        assert_eq!(order.pop().unwrap().rank, Rank::Queen);
        assert_eq!(order.first().unwrap().rank, Rank::Ace);
    }
}
