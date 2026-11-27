//! Bevy systems that render the game in play as 3D cards on the table and translate pointer input
//! into moves.
use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{platform::collections::HashMap, prelude::*, window::PrimaryWindow};

use crate::{
    cards::{CARD_H, CARD_W, Card},
    celebrate::Flying,
    game::{Game, Pile},
    games::GameKind,
    hud::{GameClock, UndoRequested},
    layout::{HUD_HEIGHT_PX, Layout},
    progress,
    scene::{TableCamera, pointer_on_plane, to_table, to_world},
    settings::{Settings, SettingsPanel},
    state::GameState,
};

/// How quickly cards glide toward their resting place (higher is faster).
const GLIDE_SPEED: f32 = 14.0;
/// How quickly cards turn over (higher is faster).
const FLIP_SPEED: f32 = 12.0;
/// A press that moves less than this (in logical pixels) before release is a tap, not a drag.
const TAP_SLOP_PX: f32 = 10.0;
/// Height of the bottom card of a pile above the felt.
pub const REST_HEIGHT: f32 = 0.5;
/// Height between cards stacked in a pile.
///
/// Generous enough that stacked cards never compete for the same depth (z-fighting), even with
/// the lower depth precision of some mobile GPUs.
const CARD_GAP: f32 = 0.5;
/// How far travelling cards rise, so they pass over resting cards.
const TRAVEL_LIFT: f32 = 25.0;
/// How long a card shakes when tapped with nowhere to go, in seconds.
const SHAKE_SECS: f32 = 0.4;
/// How far (in table units) a shaking card first swings to each side.
const SHAKE_AMPLITUDE: f32 = 7.0;
/// How many times a second a shaking card swings back and forth.
const SHAKE_FREQUENCY: f32 = 14.0;
/// Height of cards being dragged.
const DRAG_HEIGHT: f32 = 40.0;
/// Gap between a card's face and back quads.
const FACE_SEPARATION: f32 = 0.05;
/// Delay before auto-finish starts, once the game is certainly winnable.
const AUTO_FINISH_DELAY: f32 = 0.5;
/// Time between the first auto-finish moves.
const AUTO_FINISH_FIRST_INTERVAL: f32 = 0.35;
/// Each auto-finish move comes this much sooner than the one before.
const AUTO_FINISH_ACCELERATION: f32 = 0.85;
/// The shortest time between auto-finish moves.
const AUTO_FINISH_MIN_INTERVAL: f32 = 0.03;

/// The game being played.
#[derive(Resource, Default)]
pub struct Table(pub Game);

/// The game to set up on entering [`GameState::Deal`].
#[derive(Resource, Default, Clone, Copy)]
pub struct NextDeal {
    pub kind: GameKind,
    /// Pick up the saved game of this kind, if there is one, instead of dealing a new one.
    pub resume: bool,
}

/// Mesh and materials shared by the cards and pile outlines.
#[derive(Resource)]
struct CardAssets {
    mesh: Handle<Mesh>,
    back: Handle<StandardMaterial>,
    faces: HashMap<Card, Handle<StandardMaterial>>,
    slot: Handle<StandardMaterial>,
}

/// Maps each card to the entity that renders it.
#[derive(Resource, Default)]
pub struct CardEntities(pub HashMap<Card, Entity>);

/// Plays a game that is certainly winnable to the end, ever faster.
#[derive(Resource, Default)]
struct AutoFinish {
    /// When the next move is due (seconds since the app started), once auto-finish has begun.
    next_move_at: Option<f64>,
    /// Time until the move after that.
    interval: f32,
}

/// Speeds up card animations (1 is normal speed), so they keep up with auto-finish.
#[derive(Resource)]
struct AnimationSpeed(f32);

impl Default for AnimationSpeed {
    fn default() -> Self {
        Self(1.0)
    }
}

/// A card on the table, and where it is heading.
#[derive(Component)]
struct CardView {
    card: Card,
    /// Where the card should come to rest.
    target: Vec3,
    /// Whether the card should end up face up.
    face_up: bool,
    /// The card's current height before any lift from turning over.
    height: f32,
    /// When the card started shaking (seconds since the app started), if it is shaking.
    shake_started: Option<f32>,
    /// The sideways shake offset applied last frame, so it can be taken out again.
    shake_offset: f32,
}

/// The sideways offset of a card `t` seconds into a shake, or `None` once the shake is over.
///
/// A few quick swings that die away, like a head shaking "no".
fn shake_offset(t: f32) -> Option<f32> {
    if !(0.0..SHAKE_SECS).contains(&t) {
        return None;
    }
    let fade = 1.0 - t / SHAKE_SECS;
    Some(SHAKE_AMPLITUDE * fade * (std::f32::consts::TAU * SHAKE_FREQUENCY * t).sin())
}

/// Cards currently picked up by the pointer.
struct Drag {
    from: Pile,
    index: usize,
    cards: Vec<Card>,
    /// Offset (on the table) from the pointer to the center of the first dragged card.
    grab_offset: Vec2,
    /// Last known pointer position, on the drag plane, in table coordinates.
    pointer: Vec2,
    /// Where the press started, in window coordinates.
    press_position: Vec2,
    /// Whether the pointer has moved far enough for this to be a drag rather than a tap.
    moved: bool,
}

#[derive(Resource, Default)]
struct PointerState {
    drag: Option<Drag>,
}

impl PointerState {
    fn is_dragging(&self, card: &Card) -> bool {
        self.drag.as_ref().is_some_and(|d| d.cards.contains(card))
    }
}

pub struct TablePlugin;

impl Plugin for TablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Table>()
            .init_resource::<NextDeal>()
            .init_resource::<CardEntities>()
            .init_resource::<PointerState>()
            .init_resource::<AutoFinish>()
            .init_resource::<AnimationSpeed>()
            .add_systems(Startup, load_card_assets)
            .add_systems(Update, (place_outlines, apply_settings))
            .add_systems(OnEnter(GameState::Deal), (deal, reset_auto_finish))
            .add_systems(
                Update,
                (
                    // The table ignores the pointer while the settings panel is open.
                    pointer_input.run_if(in_state(SettingsPanel::Closed)),
                    undo_input,
                    auto_finish,
                    check_win,
                )
                    .chain()
                    .run_if(in_state(GameState::Play)),
            )
            .add_systems(Update, (sync_cards, animate_cards).chain().after(check_win));
    }
}

/// Rotation of a card lying flat on the table, face up or face down.
///
/// A card's meshes lie in their local XY plane with the face toward +Z. Face up, that +Z points up
/// (+Y) and the card's top points away from the player (-Z). Face down, the card is additionally
/// turned over about its long side.
pub fn card_rotation(face_up: bool) -> Quat {
    let face_up_rotation = Quat::from_rotation_x(-FRAC_PI_2);
    if face_up {
        face_up_rotation
    } else {
        Quat::from_rotation_z(PI) * face_up_rotation
    }
}

fn load_card_assets(
    mut commands: Commands,
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut card_material = |path: String| {
        materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load(path)),
            alpha_mode: AlphaMode::Mask(0.5),
            perceptual_roughness: 0.7,
            ..default()
        })
    };
    let back = card_material(settings.back.path().to_string());
    // Every game is played with (some of) the standard 52 cards.
    let faces = crate::cards::new_deck()
        .into_iter()
        .map(|card| (card, card_material(settings.deck.face_path(card))))
        .collect();
    commands.insert_resource(CardAssets {
        mesh: meshes.add(Rectangle::new(CARD_W, CARD_H)),
        back,
        faces,
        slot: materials.add(StandardMaterial {
            base_color: settings.table.outline(),
            perceptual_roughness: 1.0,
            ..default()
        }),
    });
}

/// Swaps in the chosen card art and outline color when the settings change.
fn apply_settings(
    settings: Res<Settings>,
    assets: Res<CardAssets>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.is_changed() || settings.is_added() {
        return;
    }
    if let Some(mut back) = materials.get_mut(&assets.back) {
        back.base_color_texture = Some(asset_server.load(settings.back.path()));
    }
    for (&card, face) in &assets.faces {
        if let Some(mut face) = materials.get_mut(face) {
            face.base_color_texture = Some(asset_server.load(settings.deck.face_path(card)));
        }
    }
    if let Some(mut slot) = materials.get_mut(&assets.slot) {
        slot.base_color = settings.table.outline();
    }
}

/// Marks the outline showing where a pile goes.
#[derive(Component)]
struct PileOutline(Pile);

/// Moves the pile outlines into place whenever the layout changes.
fn place_outlines(layout: Res<Layout>, mut outlines: Query<(&PileOutline, &mut Transform)>) {
    if !layout.is_changed() {
        return;
    }
    for (outline, mut transform) in &mut outlines {
        transform.translation = to_world(layout.pile_origin(outline.0), 0.1);
    }
}

/// Cards and pile outlines: everything a deal replaces.
type OnTable = Or<(With<CardView>, With<PileOutline>)>;

/// Sets up the game [`NextDeal`] asks for: picks up its saved game or deals a new one, and spawns
/// the pile outlines and every card.
#[allow(clippy::too_many_arguments)]
fn deal(
    mut commands: Commands,
    old_table: Query<Entity, OnTable>,
    window: Single<&Window, With<PrimaryWindow>>,
    assets: Res<CardAssets>,
    next: Res<NextDeal>,
    mut layout: ResMut<Layout>,
    mut table: ResMut<Table>,
    mut clock: ResMut<GameClock>,
    mut entities: ResMut<CardEntities>,
    mut pointer: ResMut<PointerState>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for entity in &old_table {
        commands.entity(entity).despawn();
    }
    *pointer = PointerState::default();
    let kind = next.kind;
    let saved = next.resume.then(|| progress::load(kind)).flatten();
    let (game, seconds) = match saved {
        Some(saved) => {
            info!("Resuming {} after {} moves", kind.name(), saved.game.moves);
            (saved.game, saved.seconds)
        }
        None => {
            info!("Dealing {}", kind.name());
            let deck = crate::cards::shuffled(kind.rules().deck());
            (Game::deal(kind, deck), 0.0)
        }
    };
    table.0 = game;
    *clock = GameClock::resumed(seconds);
    let new_layout = Layout::for_window(kind, window.size());
    if *layout != new_layout {
        *layout = new_layout;
    }

    for pile in table.0.piles() {
        commands.spawn((
            PileOutline(pile),
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.slot.clone()),
            Transform::from_translation(to_world(layout.pile_origin(pile), 0.1))
                .with_rotation(card_rotation(true))
                .with_scale(Vec3::new(0.95, 0.96, 1.0)),
        ));
    }

    // All cards start face down where cards are dealt from, then glide out to their places.
    let origin = layout.deal_origin(&table.0);
    entities.0 = table
        .0
        .board()
        .cards()
        .enumerate()
        .map(|(i, slot)| {
            let card = slot.card;
            let height = REST_HEIGHT + i as f32 * CARD_GAP;
            let start = to_world(origin, height);
            let entity = commands
                .spawn((
                    CardView {
                        card,
                        target: start,
                        face_up: false,
                        height,
                        shake_started: None,
                        shake_offset: 0.0,
                    },
                    Transform::from_translation(start).with_rotation(card_rotation(false)),
                    Visibility::default(),
                    children![
                        (
                            Mesh3d(assets.mesh.clone()),
                            MeshMaterial3d(assets.faces[&card].clone()),
                            Transform::from_xyz(0.0, 0.0, FACE_SEPARATION),
                        ),
                        (
                            Mesh3d(assets.mesh.clone()),
                            MeshMaterial3d(assets.back.clone()),
                            // Facing the other way: with back-face culling only one side of the
                            // card is ever drawn, so the face can't show through the back.
                            Transform::from_xyz(0.0, 0.0, -FACE_SEPARATION)
                                .with_rotation(Quat::from_rotation_y(PI)),
                        ),
                    ],
                ))
                .id();
            (card, entity)
        })
        .collect();
    game_state.set(GameState::Play);
}

/// Reads mouse and touch input and returns `(pressed, released, current)` positions in
/// window coordinates.
fn read_pointer(
    window: &Window,
    mouse: &ButtonInput<MouseButton>,
    touches: &Touches,
) -> (Option<Vec2>, Option<Vec2>, Option<Vec2>) {
    let mut current = window.cursor_position();
    let mut pressed = mouse
        .just_pressed(MouseButton::Left)
        .then_some(current)
        .flatten();
    let mut released = mouse
        .just_released(MouseButton::Left)
        .then_some(current)
        .flatten();
    if let Some(touch) = touches.iter().next() {
        current = Some(touch.position());
    }
    if let Some(touch) = touches.iter_just_pressed().next() {
        pressed = Some(touch.position());
    }
    if let Some(touch) = touches.iter_just_released().next() {
        released = Some(touch.position());
        // The touch is gone from `touches.iter()` now; where it lifted is its last position.
        // (The mouse cursor may be somewhere else entirely on a touch screen.)
        current = released;
    }
    (pressed, released, current)
}

/// Handles clicking the stock, tapping cards to move them, and dragging cards between piles.
#[allow(clippy::too_many_arguments)]
fn pointer_input(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<TableCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    time: Res<Time>,
    entities: Res<CardEntities>,
    layout: Res<Layout>,
    mut table: ResMut<Table>,
    mut pointer: ResMut<PointerState>,
    auto_finish: Res<AutoFinish>,
    mut cards: Query<(&mut Transform, &mut CardView)>,
) {
    if auto_finish.next_move_at.is_some() {
        // The game is finishing itself.
        return;
    }
    let (camera, camera_transform) = *camera;
    let on_plane = |pos: Vec2, height: f32| pointer_on_plane(camera, camera_transform, pos, height);
    let (pressed, released, current) = read_pointer(&window, &mouse, &touches);

    let pressed = pressed.filter(|pos| pos.y > HUD_HEIGHT_PX);
    if let Some(pos) = pressed
        && let Some(point) = on_plane(pos, REST_HEIGHT)
    {
        let game = &table.0;
        match layout.hit_test(game, point) {
            Some((Pile::Stock, _)) => {
                table.0.draw();
            }
            Some((pile, Some(index))) if game.can_pick_up(pile, index) => {
                if let Some(drag_point) = on_plane(pos, DRAG_HEIGHT) {
                    pointer.drag = Some(Drag {
                        from: pile,
                        index,
                        cards: game.pile(pile)[index..].iter().map(|s| s.card).collect(),
                        grab_offset: layout.card_position(game, pile, index) - drag_point,
                        pointer: drag_point,
                        press_position: pos,
                        moved: false,
                    });
                }
            }
            _ => {}
        }
    }

    let Some(drag) = pointer.drag.as_mut() else {
        return;
    };
    if let Some(pos) = current {
        drag.moved |= pos.distance(drag.press_position) > TAP_SLOP_PX;
        if let Some(point) = on_plane(pos, DRAG_HEIGHT) {
            drag.pointer = point;
        }
    }
    let lead = drag.pointer + drag.grab_offset;
    // Cards only lift once the pointer moves, so a tap doesn't make them jump.
    for (i, card) in drag.cards.iter().enumerate().filter(|_| drag.moved) {
        if let Ok((mut transform, mut view)) = cards.get_mut(entities.0[card]) {
            let offset = Vec2::new(0.0, i as f32 * layout.face_up_offset);
            let height = DRAG_HEIGHT + i as f32 * CARD_GAP;
            transform.translation = to_world(lead - offset, height);
            view.height = height;
        }
    }

    if released.is_some() || !(mouse.pressed(MouseButton::Left) || touches.iter().next().is_some())
    {
        let drag = pointer.drag.take().expect("checked above");
        if !drag.moved {
            // A tap: send the cards to the best place for them, if there is one.
            if table.0.auto_move(drag.from, drag.index).is_none() {
                // Nowhere to go: shake the cards.
                for card in &drag.cards {
                    if let Ok((_, mut view)) = cards.get_mut(entities.0[card]) {
                        view.shake_started = Some(time.elapsed_secs());
                    }
                }
            }
        } else {
            let game = &table.0;
            let accepts = |to| game.can_move(drag.from, drag.index, to);
            if let Some(to) = layout.drop_target(game, lead, accepts) {
                table.0.try_move(drag.from, drag.index, to);
            }
        }
        // Even if the move failed, the cards must glide back to their piles.
        table.set_changed();
    }
}

/// Takes back the last move when the Undo button is pressed (or Ctrl+Z, Cmd+Z or U).
fn undo_input(
    mut requested: ResMut<UndoRequested>,
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<PointerState>,
    auto_finish: Res<AutoFinish>,
    mut table: ResMut<Table>,
) {
    let modifier = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let shortcut =
        keys.just_pressed(KeyCode::KeyU) || (modifier && keys.just_pressed(KeyCode::KeyZ));
    let button = std::mem::take(&mut requested.0);
    if button || shortcut {
        // Not while cards are in hand, or while the game is finishing itself.
        if pointer.drag.is_none() && auto_finish.next_move_at.is_none() && table.0.can_undo() {
            table.0.undo();
        }
    }
}

/// Clears per-game state for a new deal.
fn reset_auto_finish(
    mut auto_finish: ResMut<AutoFinish>,
    mut speed: ResMut<AnimationSpeed>,
    mut undo: ResMut<UndoRequested>,
) {
    *auto_finish = AutoFinish::default();
    *speed = AnimationSpeed::default();
    undo.0 = false;
}

/// Once the game is certainly winnable, plays it out to the end, each move coming sooner than the
/// last, and speeds up the card animations to match.
fn auto_finish(
    time: Res<Time>,
    pointer: Res<PointerState>,
    mut auto_finish: ResMut<AutoFinish>,
    mut speed: ResMut<AnimationSpeed>,
    mut table: ResMut<Table>,
) {
    let now = time.elapsed_secs_f64();
    let Some(due) = auto_finish.next_move_at else {
        if pointer.drag.is_none() && !table.0.is_won() && table.0.is_certainly_winnable() {
            info!("Auto-finishing");
            auto_finish.next_move_at = Some(now + f64::from(AUTO_FINISH_DELAY));
            auto_finish.interval = AUTO_FINISH_FIRST_INTERVAL;
        }
        return;
    };
    // Make every move that is due; on a slow device several can fall due in one frame.
    let mut due = due;
    while now >= due {
        if !table.0.auto_finish_step() {
            auto_finish.next_move_at = None;
            return;
        }
        let interval = auto_finish.interval;
        due += f64::from(interval);
        auto_finish.interval = (interval * AUTO_FINISH_ACCELERATION).max(AUTO_FINISH_MIN_INTERVAL);
        // Animations speed up as the moves do, so cards don't lag far behind.
        speed.0 = (AUTO_FINISH_FIRST_INTERVAL / interval).sqrt().max(1.0);
    }
    auto_finish.next_move_at = Some(due);
}

fn check_win(table: Res<Table>, mut game_state: ResMut<NextState<GameState>>) {
    if table.is_changed() && table.0.is_won() {
        info!("Won in {} moves", table.0.moves);
        game_state.set(GameState::Win);
    }
}

/// Sets where each card should rest, and which way up, from the game state.
fn sync_cards(
    table: Res<Table>,
    pointer: Res<PointerState>,
    entities: Res<CardEntities>,
    layout: Res<Layout>,
    mut cards: Query<&mut CardView>,
) {
    if !table.is_changed() && !pointer.is_changed() && !layout.is_changed() {
        return;
    }
    let game = &table.0;
    for pile in game.piles() {
        for (index, slot) in game.pile(pile).iter().enumerate() {
            let Some(Ok(mut view)) = entities.0.get(&slot.card).map(|&e| cards.get_mut(e)) else {
                continue;
            };
            view.face_up = slot.face_up;
            if !pointer.is_dragging(&slot.card) {
                let height = REST_HEIGHT + index as f32 * CARD_GAP;
                view.target = to_world(layout.card_position(game, pile, index), height);
            }
        }
    }
}

/// Glides cards toward their targets, lifting them over other cards on the way, and turns them
/// over.
fn animate_cards(
    time: Res<Time>,
    pointer: Res<PointerState>,
    speed: Res<AnimationSpeed>,
    mut cards: Query<(&mut Transform, &mut CardView), Without<Flying>>,
) {
    let dt = time.delta_secs() * speed.0;
    let glide = 1.0 - (-GLIDE_SPEED * dt).exp();
    let flip = 1.0 - (-FLIP_SPEED * dt).exp();
    for (mut transform, mut view) in &mut cards {
        let goal_rotation = card_rotation(view.face_up);
        transform.rotation = if transform.rotation.angle_between(goal_rotation) < 0.01 {
            goal_rotation
        } else {
            transform.rotation.slerp(goal_rotation, flip)
        };
        if pointer.is_dragging(&view.card) {
            continue;
        }

        let goal = to_table(view.target);
        // Glide from where the card would be without last frame's shake.
        let unshaken = to_table(transform.translation) - Vec2::new(view.shake_offset, 0.0);
        let mut position = unshaken.lerp(goal, glide);
        if position.distance(goal) < 0.5 {
            position = goal;
        }
        let travelling = position.distance(goal) > 2.0;
        let goal_height = view.target.y + if travelling { TRAVEL_LIFT } else { 0.0 };
        view.height += (goal_height - view.height) * glide;
        if (view.height - goal_height).abs() < 0.05 {
            view.height = goal_height;
        }
        // While turning over, raise the card by enough that its edge clears the table.
        let tilt = transform.rotation.angle_between(card_rotation(true));
        let flip_lift = CARD_W / 2.0 * tilt.sin();
        let shake = view
            .shake_started
            .and_then(|start| shake_offset(time.elapsed_secs() - start));
        if shake.is_none() {
            view.shake_started = None;
        }
        view.shake_offset = shake.unwrap_or_default();
        let shaken = position + Vec2::new(view.shake_offset, 0.0);
        transform.translation = to_world(shaken, view.height + flip_lift);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_swings_both_ways_then_settles() {
        let samples: Vec<f32> = (0..40)
            .filter_map(|i| shake_offset(i as f32 * SHAKE_SECS / 40.0))
            .collect();
        assert_eq!(samples.len(), 40);
        assert!(samples.iter().any(|&x| x > SHAKE_AMPLITUDE * 0.5));
        assert!(samples.iter().any(|&x| x < -SHAKE_AMPLITUDE * 0.5));
        assert!(samples.iter().all(|x| x.abs() <= SHAKE_AMPLITUDE));
        // It dies away, and stops.
        let late = shake_offset(SHAKE_SECS * 0.95).unwrap();
        assert!(late.abs() < SHAKE_AMPLITUDE * 0.1);
        assert_eq!(shake_offset(SHAKE_SECS), None);
        assert_eq!(shake_offset(-0.1), None);
    }
}
