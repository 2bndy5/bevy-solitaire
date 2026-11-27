//! Player preferences (card art, table color and lighting), how they're saved, and the settings
//! panel that changes them.
use bevy::{prelude::*, ui::FocusPolicy, ui_widgets::Button};

use crate::{
    cards::{Card, Rank, Suit},
    menu::{button_node, font},
};

/// One of a fixed set of options for a setting.
pub trait Choice: Copy + PartialEq + Sized + 'static {
    /// Every option, in the order the settings panel shows them.
    const ALL: &'static [Self];
    /// A stable name used when saving.
    fn key(self) -> &'static str;
    /// The name shown to the player.
    fn label(self) -> &'static str;

    fn from_key(key: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|choice| choice.key() == key)
    }
}

/// The art on the cards' faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Deck {
    /// The hand-drawn deck from bevy-52-card-pickup.
    #[default]
    HandDrawn,
}

impl Choice for Deck {
    const ALL: &'static [Self] = &[Deck::HandDrawn];

    fn key(self) -> &'static str {
        match self {
            Deck::HandDrawn => "hand-drawn",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Deck::HandDrawn => "Hand-drawn",
        }
    }
}

impl Deck {
    /// Path (relative to the `assets` folder) of `card`'s face in this deck.
    pub fn face_path(self, card: Card) -> String {
        match self {
            Deck::HandDrawn => card.face_resource_name(),
        }
    }
}

/// The art on the cards' backs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardBack {
    #[default]
    Blue,
    Red,
}

impl Choice for CardBack {
    const ALL: &'static [Self] = &[CardBack::Blue, CardBack::Red];

    fn key(self) -> &'static str {
        match self {
            CardBack::Blue => "blue",
            CardBack::Red => "red",
        }
    }

    fn label(self) -> &'static str {
        match self {
            CardBack::Blue => "Blue",
            CardBack::Red => "Red",
        }
    }
}

impl CardBack {
    /// Path (relative to the `assets` folder) of this back's image.
    pub fn path(self) -> &'static str {
        match self {
            CardBack::Blue => "images/Back Blue.png",
            CardBack::Red => "images/Back Red.png",
        }
    }
}

/// The color of the table's felt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableColor {
    #[default]
    Green,
    Blue,
    Burgundy,
    Purple,
    Charcoal,
}

impl Choice for TableColor {
    const ALL: &'static [Self] = &[
        TableColor::Green,
        TableColor::Blue,
        TableColor::Burgundy,
        TableColor::Purple,
        TableColor::Charcoal,
    ];

    fn key(self) -> &'static str {
        match self {
            TableColor::Green => "green",
            TableColor::Blue => "blue",
            TableColor::Burgundy => "burgundy",
            TableColor::Purple => "purple",
            TableColor::Charcoal => "charcoal",
        }
    }

    fn label(self) -> &'static str {
        match self {
            TableColor::Green => "Green",
            TableColor::Blue => "Blue",
            TableColor::Burgundy => "Burgundy",
            TableColor::Purple => "Purple",
            TableColor::Charcoal => "Charcoal",
        }
    }
}

impl TableColor {
    /// The felt's color.
    pub fn felt(self) -> Color {
        match self {
            TableColor::Green => Color::srgb(0.11, 0.42, 0.23),
            TableColor::Blue => Color::srgb(0.07, 0.27, 0.45),
            TableColor::Burgundy => Color::srgb(0.42, 0.08, 0.13),
            TableColor::Purple => Color::srgb(0.27, 0.13, 0.40),
            TableColor::Charcoal => Color::srgb(0.17, 0.18, 0.19),
        }
    }

    /// The color of the outlines marking where piles go: a lighter shade of the felt.
    pub fn outline(self) -> Color {
        self.felt().mix(&Color::WHITE, 0.15)
    }
}

/// How the table is lit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lighting {
    #[default]
    Daylight,
    Lamp,
    Evening,
    Soft,
}

impl Choice for Lighting {
    const ALL: &'static [Self] = &[
        Lighting::Daylight,
        Lighting::Lamp,
        Lighting::Evening,
        Lighting::Soft,
    ];

    fn key(self) -> &'static str {
        match self {
            Lighting::Daylight => "daylight",
            Lighting::Lamp => "lamp",
            Lighting::Evening => "evening",
            Lighting::Soft => "soft",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Lighting::Daylight => "Daylight",
            Lighting::Lamp => "Lamp",
            Lighting::Evening => "Evening",
            Lighting::Soft => "Soft",
        }
    }
}

/// The lights for a [`Lighting`] choice.
pub struct LightRig {
    pub sun_color: Color,
    /// The sun's brightness, in lux.
    pub sun_lux: f32,
    /// Where the sun shines from (it always points at the table's center).
    pub sun_position: Vec3,
    pub shadows: bool,
    pub ambient_color: Color,
    pub ambient_brightness: f32,
}

impl Lighting {
    pub fn rig(self) -> LightRig {
        match self {
            // Bright and neutral, from high above: short, crisp shadows.
            Lighting::Daylight => LightRig {
                sun_color: Color::WHITE,
                sun_lux: 2200.0,
                sun_position: Vec3::new(-250.0, 1000.0, -350.0),
                shadows: true,
                ambient_color: Color::WHITE,
                ambient_brightness: 300.0,
            },
            // A warm lamp low to one side: long shadows.
            Lighting::Lamp => LightRig {
                sun_color: Color::srgb(1.0, 0.9, 0.75),
                sun_lux: 2500.0,
                sun_position: Vec3::new(-900.0, 700.0, -250.0),
                shadows: true,
                ambient_color: Color::srgb(1.0, 0.94, 0.86),
                ambient_brightness: 220.0,
            },
            // Cool, dim light: easy on the eyes at night.
            Lighting::Evening => LightRig {
                sun_color: Color::srgb(0.8, 0.85, 1.0),
                sun_lux: 1300.0,
                sun_position: Vec3::new(300.0, 1000.0, -500.0),
                shadows: true,
                ambient_color: Color::srgb(0.7, 0.78, 1.0),
                ambient_brightness: 260.0,
            },
            // Even, shadowless light.
            Lighting::Soft => LightRig {
                sun_color: Color::WHITE,
                sun_lux: 1100.0,
                sun_position: Vec3::new(0.0, 1000.0, 0.0),
                shadows: false,
                ambient_color: Color::WHITE,
                ambient_brightness: 850.0,
            },
        }
    }
}

/// The player's preferences.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings {
    pub deck: Deck,
    pub back: CardBack,
    pub table: TableColor,
    pub lighting: Lighting,
}

impl Settings {
    /// Saves the settings as `key=value` lines.
    pub fn to_text(self) -> String {
        format!(
            "deck={}\nback={}\ntable={}\nlighting={}\n",
            self.deck.key(),
            self.back.key(),
            self.table.key(),
            self.lighting.key()
        )
    }

    /// Reads settings saved by [`Settings::to_text`]. Unknown or missing values keep their
    /// defaults, so older or newer save files still load.
    pub fn from_text(text: &str) -> Self {
        let mut settings = Settings::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "deck" => settings.deck = Deck::from_key(value).unwrap_or_default(),
                "back" => settings.back = CardBack::from_key(value).unwrap_or_default(),
                "table" => settings.table = TableColor::from_key(value).unwrap_or_default(),
                "lighting" => settings.lighting = Lighting::from_key(value).unwrap_or_default(),
                _ => {}
            }
        }
        settings
    }
}

/// Saving settings in the browser's local storage.
#[cfg(target_arch = "wasm32")]
mod storage {
    const KEY: &str = "bevy-solitaire-settings";

    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn load() -> Option<String> {
        local_storage()?.get_item(KEY).ok()?
    }

    pub fn save(text: &str) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(KEY, text);
        }
    }
}

/// Saving settings in a file in the user's config folder.
#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use std::{env, fs, path::PathBuf};

    fn path() -> Option<PathBuf> {
        let config = env::var_os("APPDATA")
            .or_else(|| env::var_os("XDG_CONFIG_HOME"))
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(config.join("bevy-solitaire").join("settings.txt"))
    }

    pub fn load() -> Option<String> {
        fs::read_to_string(path()?).ok()
    }

    pub fn save(text: &str) {
        let Some(path) = path() else {
            return;
        };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Err(error) = fs::write(&path, text) {
            bevy::log::warn!("Couldn't save settings to {}: {error}", path.display());
        }
    }
}

/// Whether the settings panel is showing.
#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsPanel {
    #[default]
    Closed,
    Open,
}

/// Marks buttons that open or close the settings panel (the HUD's cog, and the panel's close
/// button).
#[derive(Component)]
pub struct SettingsButton;

/// A button choosing one option of a setting.
#[derive(Component, Clone, Copy, PartialEq)]
enum Pick {
    Deck(Deck),
    Back(CardBack),
    Table(TableColor),
    Lighting(Lighting),
}

impl Pick {
    fn is_selected(self, settings: &Settings) -> bool {
        match self {
            Pick::Deck(deck) => settings.deck == deck,
            Pick::Back(back) => settings.back == back,
            Pick::Table(table) => settings.table == table,
            Pick::Lighting(lighting) => settings.lighting == lighting,
        }
    }

    fn apply(self, settings: &mut Settings) {
        match self {
            Pick::Deck(deck) => settings.deck = deck,
            Pick::Back(back) => settings.back = back,
            Pick::Table(table) => settings.table = table,
            Pick::Lighting(lighting) => settings.lighting = lighting,
        }
    }
}

/// A card shown in the settings panel's preview; `None` shows the back.
#[derive(Component)]
struct PreviewCard(Option<Card>);

/// The cards shown in the preview.
const PREVIEW_CARDS: [Card; 4] = [
    Card::new(Rank::Ace, Suit::Spades),
    Card::new(Rank::King, Suit::Hearts),
    Card::new(Rank::Queen, Suit::Clubs),
    Card::new(Rank::Ten, Suit::Diamonds),
];

const SELECTED: Color = Color::srgb(0.125, 0.6, 0.125);
const UNSELECTED: Color = Color::srgb(0.25, 0.27, 0.26);

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let settings = storage::load()
            .map(|text| Settings::from_text(&text))
            .unwrap_or_default();
        app.insert_resource(settings)
            .init_state::<SettingsPanel>()
            .add_observer(on_press)
            .add_systems(OnEnter(SettingsPanel::Open), spawn_panel)
            .add_systems(
                Update,
                (close_on_escape, show_selection, update_preview)
                    .run_if(in_state(SettingsPanel::Open)),
            )
            .add_systems(Update, save_settings);
    }
}

/// Handles presses on the settings, close and option buttons.
///
/// Acts on the press (see [`crate::menu::on_start_button`] for why).
fn on_press(
    press: On<PointerPress>,
    picks: Query<&Pick>,
    toggles: Query<(), With<SettingsButton>>,
    panel: Res<State<SettingsPanel>>,
    mut next_panel: ResMut<NextState<SettingsPanel>>,
    mut settings: ResMut<Settings>,
) {
    if let Ok(pick) = picks.get(press.entity) {
        let mut changed = *settings;
        pick.apply(&mut changed);
        if changed != *settings {
            *settings = changed;
        }
    } else if toggles.contains(press.entity) {
        next_panel.set(match panel.get() {
            SettingsPanel::Closed => SettingsPanel::Open,
            SettingsPanel::Open => SettingsPanel::Closed,
        });
    }
}

fn close_on_escape(keys: Res<ButtonInput<KeyCode>>, mut panel: ResMut<NextState<SettingsPanel>>) {
    if keys.just_pressed(KeyCode::Escape) {
        panel.set(SettingsPanel::Closed);
    }
}

fn save_settings(settings: Res<Settings>) {
    if settings.is_changed() && !settings.is_added() {
        storage::save(&settings.to_text());
    }
}

fn spawn_panel(mut commands: Commands, asset_server: Res<AssetServer>) {
    let title_font = font(&asset_server, 26.0);
    let label_font = font(&asset_server, 18.0);
    let chip_font = font(&asset_server, 16.0);

    // A row of option buttons for one setting.
    let row = |parent: &mut ChildSpawnerCommands, title: &str, picks: Vec<(Pick, &str, Color)>| {
        parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(title),
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    label_font.clone(),
                ));
                row.spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|chips| {
                    for (pick, label, swatch) in picks {
                        chips
                            .spawn((
                                Button,
                                pick,
                                BackgroundColor(UNSELECTED),
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                                    border_radius: BorderRadius::all(Val::Px(8.0)),
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(6.0),
                                    ..default()
                                },
                            ))
                            .with_children(|chip| {
                                if swatch != Color::NONE {
                                    chip.spawn((
                                        Node {
                                            width: Val::Px(14.0),
                                            height: Val::Px(14.0),
                                            border_radius: BorderRadius::all(Val::Px(7.0)),
                                            ..default()
                                        },
                                        BackgroundColor(swatch),
                                        FocusPolicy::Pass,
                                    ));
                                }
                                chip.spawn((
                                    Text::new(label),
                                    TextColor(Color::WHITE),
                                    TextLayout::no_wrap(),
                                    chip_font.clone(),
                                ));
                            });
                    }
                });
            });
    };

    commands
        .spawn((
            // Fills the screen to center the panel, but lets presses outside it through to the
            // HUD (the panel blocks those on itself).
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(2),
            DespawnOnExit(SettingsPanel::Open),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    // Fill narrow screens (wrapping the options) but don't sprawl on wide ones.
                    width: Val::Percent(100.0),
                    max_width: Val::Px(560.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(14.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..default()
                },
                // Opaque: blending happens in linear color, so even a 4% see-through panel shows
                // the cards behind it clearly.
                BackgroundColor(Color::srgb(0.05, 0.06, 0.06)),
            ))
            .with_children(|panel| {
                // Title and close button.
                panel
                    .spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|header| {
                        // embedded the FontAwesome cog icon via unicode code-point
                        header.spawn((
                            Text::new("\u{F013} Settings"),
                            TextColor(Color::WHITE),
                            title_font.clone(),
                        ));
                        header.spawn((
                            button_node(UiRect::axes(Val::Px(10.0), Val::Px(4.0))),
                            SettingsButton,
                            // embedded the FontAwesome close (times) icon via unicode code-point
                            children![(
                                Text::new("\u{F00D}"),
                                TextColor(Color::WHITE),
                                label_font.clone()
                            )],
                        ));
                    });

                // Preview of the chosen card art.
                panel
                    .spawn(Node {
                        column_gap: Val::Px(8.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|preview| {
                        for card in PREVIEW_CARDS.into_iter().map(Some).chain([None]) {
                            preview.spawn((
                                ImageNode::default(),
                                Node {
                                    width: Val::Px(66.0),
                                    height: Val::Px(94.0),
                                    ..default()
                                },
                                PreviewCard(card),
                            ));
                        }
                    });

                row(
                    panel,
                    "Card faces",
                    Deck::ALL
                        .iter()
                        .map(|&deck| (Pick::Deck(deck), deck.label(), Color::NONE))
                        .collect(),
                );
                row(
                    panel,
                    "Card back",
                    CardBack::ALL
                        .iter()
                        .map(|&back| (Pick::Back(back), back.label(), Color::NONE))
                        .collect(),
                );
                row(
                    panel,
                    "Table",
                    TableColor::ALL
                        .iter()
                        .map(|&table| (Pick::Table(table), table.label(), table.felt()))
                        .collect(),
                );
                row(
                    panel,
                    "Lighting",
                    Lighting::ALL
                        .iter()
                        .map(|&lighting| (Pick::Lighting(lighting), lighting.label(), Color::NONE))
                        .collect(),
                );
            });
        });
}

/// Highlights the chosen option of each setting.
fn show_selection(settings: Res<Settings>, mut picks: Query<(&Pick, &mut BackgroundColor)>) {
    for (pick, mut background) in &mut picks {
        let color = if pick.is_selected(&settings) {
            SELECTED
        } else {
            UNSELECTED
        };
        if background.0 != color {
            background.0 = color;
        }
    }
}

/// Shows the chosen card art in the preview.
fn update_preview(
    settings: Res<Settings>,
    asset_server: Res<AssetServer>,
    mut previews: Query<(Ref<PreviewCard>, &mut ImageNode)>,
) {
    for (preview, mut image) in &mut previews {
        if preview.is_added() || settings.is_changed() {
            image.image = asset_server.load(match preview.0 {
                Some(card) => settings.deck.face_path(card),
                None => settings.back.path().to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let settings = Settings {
            deck: Deck::HandDrawn,
            back: CardBack::Red,
            table: TableColor::Purple,
            lighting: Lighting::Evening,
        };
        assert_eq!(Settings::from_text(&settings.to_text()), settings);
    }

    #[test]
    fn unknown_or_missing_values_keep_defaults() {
        let settings = Settings::from_text("back=red\ntable=plaid\nnonsense\nfuture=thing\n");
        assert_eq!(
            settings,
            Settings {
                back: CardBack::Red,
                ..Settings::default()
            }
        );
        assert_eq!(Settings::from_text(""), Settings::default());
    }

    #[test]
    fn choice_keys_are_unique_and_parse_back() {
        fn check<C: Choice + std::fmt::Debug>() {
            for &choice in C::ALL {
                assert_eq!(C::from_key(choice.key()), Some(choice));
            }
            let keys: std::collections::HashSet<_> = C::ALL.iter().map(|c| c.key()).collect();
            assert_eq!(keys.len(), C::ALL.len());
        }
        check::<Deck>();
        check::<CardBack>();
        check::<TableColor>();
        check::<Lighting>();
    }

    #[test]
    fn card_art_exists() {
        let exists = |path: &str| std::path::Path::new(&format!("assets/{path}")).exists();
        for &deck in Deck::ALL {
            for card in crate::cards::new_deck() {
                assert!(exists(&deck.face_path(card)), "{deck:?} {card}");
            }
        }
        for &back in CardBack::ALL {
            assert!(exists(back.path()), "{back:?}");
        }
    }
}
