//! Bevy systems that render the [`Klondike`] game and translate pointer input into moves.
use bevy::{platform::collections::HashMap, prelude::*, window::PrimaryWindow};

use crate::{
    cards::{CARD_H, CARD_W, Card, new_deck, shuffle_deck},
    game::{Klondike, Pile},
    layout::{FACE_UP_OFFSET, card_position, drop_target, hit_test, pile_origin},
    state::GameState,
};

/// How quickly cards glide toward their resting place (higher is faster).
const GLIDE_SPEED: f32 = 18.0;
/// Two presses on the same card within this many seconds count as a double-click.
const DOUBLE_CLICK_SECS: f64 = 0.4;
/// Z offset for cards that are travelling, so they pass over resting cards.
const Z_MOVING: f32 = 100.0;
/// Z offset for cards being dragged.
const Z_DRAGGING: f32 = 200.0;
/// Presses this close (in logical pixels) to the top of the window belong to the HUD.
const HUD_HEIGHT_PX: f32 = 40.0;

/// The game being played.
#[derive(Resource, Default)]
pub struct Table(pub Klondike);

/// Handles to every card image.
#[derive(Resource)]
struct CardImages {
    back: Handle<Image>,
    faces: HashMap<Card, Handle<Image>>,
}

/// Maps each card to the entity that renders it.
#[derive(Resource, Default)]
struct CardEntities(HashMap<Card, Entity>);

/// Marks the sprite entity of a card.
#[derive(Component)]
struct CardSprite(Card);

/// Where a card should come to rest.
#[derive(Component)]
struct Target(Vec3);

/// Cards currently picked up by the pointer.
struct Drag {
    from: Pile,
    index: usize,
    cards: Vec<Card>,
    /// Offset from the pointer to the center of the first dragged card.
    grab_offset: Vec2,
    /// Last known pointer position (world space).
    pointer: Vec2,
}

#[derive(Resource, Default)]
struct PointerState {
    drag: Option<Drag>,
    last_press: Option<(Card, f64)>,
}

impl PointerState {
    fn is_dragging(&self, card: &Card) -> bool {
        self.drag.as_ref().is_some_and(|d| d.cards.contains(card))
    }
}

/// Marks the HUD text showing the move count.
#[derive(Component)]
struct MovesText;

pub struct TablePlugin;

impl Plugin for TablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Table>()
            .init_resource::<CardEntities>()
            .init_resource::<PointerState>()
            .add_systems(Startup, (load_images, spawn_slots))
            .add_systems(OnEnter(GameState::Deal), deal)
            .add_systems(
                Update,
                (pointer_input, check_win)
                    .chain()
                    .run_if(in_state(GameState::Play)),
            )
            .add_systems(
                Update,
                (sync_cards, animate_cards, update_moves_text).chain(),
            );
    }
}

fn load_images(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CardImages {
        back: asset_server.load(Card::back_resource_name()),
        faces: new_deck()
            .into_iter()
            .map(|card| (card, asset_server.load(card.face_resource_name())))
            .collect(),
    });
}

/// Spawns faint outlines marking where each pile goes.
fn spawn_slots(mut commands: Commands) {
    for pile in Klondike::all_piles() {
        commands.spawn((
            Sprite::from_color(
                Color::WHITE.with_alpha(0.15),
                Vec2::new(CARD_W, CARD_H) - 4.0,
            ),
            Transform::from_translation(pile_origin(pile).extend(0.0)),
        ));
    }
}

/// Starts a new game: shuffles, deals, and spawns a sprite for every card.
fn deal(
    mut commands: Commands,
    old_cards: Query<Entity, With<CardSprite>>,
    images: Res<CardImages>,
    mut table: ResMut<Table>,
    mut entities: ResMut<CardEntities>,
    mut pointer: ResMut<PointerState>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for entity in &old_cards {
        commands.entity(entity).despawn();
    }
    *pointer = PointerState::default();
    table.0 = Klondike::deal(shuffle_deck());
    // All cards start on the stock, then glide out to their dealt positions.
    let start = pile_origin(Pile::Stock).extend(1.0);
    entities.0 = new_deck()
        .into_iter()
        .map(|card| {
            let entity = commands
                .spawn((
                    CardSprite(card),
                    Sprite {
                        image: images.back.clone(),
                        custom_size: Some(Vec2::new(CARD_W, CARD_H)),
                        ..default()
                    },
                    Transform::from_translation(start),
                    Target(start),
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
    }
    (pressed, released, current)
}

/// Handles clicking the stock, double-clicking cards, and dragging cards between piles.
#[allow(clippy::too_many_arguments)]
fn pointer_input(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    time: Res<Time>,
    entities: Res<CardEntities>,
    mut table: ResMut<Table>,
    mut pointer: ResMut<PointerState>,
    mut transforms: Query<&mut Transform, With<CardSprite>>,
) {
    let (camera, camera_transform) = *camera;
    let to_world = |pos: Vec2| camera.viewport_to_world_2d(camera_transform, pos).ok();
    let (pressed, released, current) = read_pointer(&window, &mouse, &touches);

    let pressed = pressed.filter(|pos| pos.y > HUD_HEIGHT_PX);
    if let Some(point) = pressed.and_then(to_world) {
        let game = &table.0;
        match hit_test(game, point) {
            Some((Pile::Stock, _)) => {
                table.0.draw();
            }
            Some((pile, Some(index))) if game.can_pick_up(pile, index) => {
                let card = game.pile(pile)[index].card;
                let now = time.elapsed_secs_f64();
                let is_top = index + 1 == game.pile(pile).len();
                let double_click = pointer
                    .last_press
                    .is_some_and(|(last, at)| last == card && now - at < DOUBLE_CLICK_SECS);
                if double_click && is_top && table.0.move_to_foundation(pile) {
                    pointer.last_press = None;
                } else {
                    pointer.last_press = Some((card, now));
                    let game = &table.0;
                    pointer.drag = Some(Drag {
                        from: pile,
                        index,
                        cards: game.pile(pile)[index..].iter().map(|s| s.card).collect(),
                        grab_offset: card_position(game, pile, index) - point,
                        pointer: point,
                    });
                }
            }
            _ => {}
        }
    }

    let Some(drag) = pointer.drag.as_mut() else {
        return;
    };
    if let Some(point) = current.and_then(to_world) {
        drag.pointer = point;
    }
    let lead = drag.pointer + drag.grab_offset;
    for (i, card) in drag.cards.iter().enumerate() {
        if let Ok(mut transform) = transforms.get_mut(entities.0[card]) {
            let offset = Vec2::new(0.0, i as f32 * FACE_UP_OFFSET);
            transform.translation = (lead - offset).extend(Z_DRAGGING + i as f32);
        }
    }

    if released.is_some() || !(mouse.pressed(MouseButton::Left) || touches.iter().next().is_some())
    {
        let drag = pointer.drag.take().expect("checked above");
        if let Some(to) = drop_target(&table.0, lead) {
            table.0.try_move(drag.from, drag.index, to);
        }
    }
}

fn check_win(table: Res<Table>, mut game_state: ResMut<NextState<GameState>>) {
    if table.is_changed() && table.0.is_won() {
        info!("Won in {} moves", table.0.moves);
        game_state.set(GameState::Win);
    }
}

/// Sets each card's face and resting place from the game state.
fn sync_cards(
    table: Res<Table>,
    pointer: Res<PointerState>,
    images: Option<Res<CardImages>>,
    entities: Res<CardEntities>,
    mut cards: Query<(&mut Sprite, &mut Target), With<CardSprite>>,
) {
    let Some(images) = images else {
        return;
    };
    let game = &table.0;
    for pile in Klondike::all_piles() {
        for (index, slot) in game.pile(pile).iter().enumerate() {
            let Some(Ok((mut sprite, mut target))) =
                entities.0.get(&slot.card).map(|&e| cards.get_mut(e))
            else {
                continue;
            };
            let image = if slot.face_up {
                &images.faces[&slot.card]
            } else {
                &images.back
            };
            if sprite.image != *image {
                sprite.image = image.clone();
            }
            if !pointer.is_dragging(&slot.card) {
                let z = 1.0 + index as f32 * 0.5;
                target.0 = card_position(game, pile, index).extend(z);
            }
        }
    }
}

/// Glides cards toward their [`Target`].
fn animate_cards(
    time: Res<Time>,
    pointer: Res<PointerState>,
    mut cards: Query<(&CardSprite, &mut Transform, &Target)>,
) {
    let t = 1.0 - (-GLIDE_SPEED * time.delta_secs()).exp();
    for (card, mut transform, target) in &mut cards {
        if pointer.is_dragging(&card.0) {
            continue;
        }
        let goal = target.0.truncate();
        let mut position = transform.translation.truncate().lerp(goal, t);
        if position.distance(goal) < 0.5 {
            position = goal;
        }
        let z = if position == goal {
            target.0.z
        } else {
            Z_MOVING + target.0.z
        };
        transform.translation = position.extend(z);
    }
}

/// Spawns the heads-up display: move counter and a "New Game" button.
pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = TextFont {
        font: FontSource::Handle(asset_server.load("fonts/UbuntuNerdFont-Medium.ttf")),
        font_size: FontSize::Px(20.0),
        ..default()
    };
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text::new("Moves: 0"),
                TextColor(Color::WHITE),
                font.clone(),
                MovesText
            ),
            crate::menu::start_button("\u{F0453} New Game", font),
        ],
    ));
}

fn update_moves_text(table: Res<Table>, mut text: Query<&mut Text, With<MovesText>>) {
    if table.is_changed() {
        for mut text in &mut text {
            text.0 = format!("Moves: {}", table.0.moves);
        }
    }
}
