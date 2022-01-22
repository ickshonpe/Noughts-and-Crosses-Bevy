use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use rand::prelude::SliceRandom;
use rand::thread_rng;

const TILE_SIZE: f32 = 100.0;
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.35);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.45);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.55);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum GameState {
    Menu,
    Playing,
    GameOver,
}

struct GameText(TextStyle);

impl GameText {
    fn text(&self, label: &str) -> Text {
        Text::with_section(label.to_owned(), self.0.clone(), TextAlignment::default())
    }

    fn bundle(&self, label: &str) -> TextBundle {
        TextBundle {
            text: self.text(label),
            focus_policy: FocusPolicy::Pass,
            ..Default::default()
        }
    }

    fn big(&self, label: &str) -> Text {
        Text::with_section(
            label.to_owned(),
            TextStyle {
                font_size: 4.0 * self.0.font_size,
                ..self.0.clone()
            },
            TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            },
        )
    }

    fn big_bundle(&self, label: &str) -> TextBundle {
        TextBundle {
            text: self.big(label),
            focus_policy: FocusPolicy::Pass,
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,

                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    X,
    O,
    Empty,
}

impl Tile {
    pub fn is_empty(self) -> bool {
        matches!(self, Tile::Empty)
    }

    pub fn piece(self) -> &'static str {
        match self {
            Tile::X => "X",
            Tile::O => "O",
            Tile::Empty => "",
        }
    }
}

struct Board {
    moves: u8,
    tiles: [Tile; 9],
    entities: [Entity; 9],
}

impl Board {
    fn clear(&mut self) {
        self.moves = 0;
        for tile in self.tiles.iter_mut() {
            *tile = Tile::Empty;
        }
    }

    fn play_move(&mut self, index: usize) -> bool {
        if self.tiles[index].is_empty() {
            self.tiles[index] = Tile::O;
            if self.winning() {
                return true;
            };
            self.moves += 1;
            if self.moves < 8 {
                self.moves += 1;
                let rng = &mut thread_rng();
                let mut possible_moves: Vec<usize> =
                    (0..=8usize).filter(|i| self.tiles[*i].is_empty()).collect();
                possible_moves.shuffle(rng);
                self.tiles[possible_moves[0]] = Tile::X;
                if self.winning() {
                    return true;
                };
            }
        }
        self.moves == 9
    }

    fn winning(&self) -> bool {
        for player in [Tile::O, Tile::X] {
            for i in 0..3 {
                if [0, 3, 6].into_iter().all(|j| self.tiles[i + j] == player)
                    || [0, 1, 2]
                        .into_iter()
                        .all(|j| self.tiles[i * 3 + j] == player)
                {
                    return true;
                }
            }
            if [0, 4, 8].into_iter().all(|i| self.tiles[i] == player)
                || [2, 4, 6].into_iter().all(|i| self.tiles[i] == player)
            {
                return true;
            }
        }
        false
    }
}

impl FromWorld for Board {
    fn from_world(world: &mut World) -> Self {
        Self {
            tiles: [Tile::Empty; 9],
            entities: [(); 9].map(|_| world.spawn().id()),
            moves: 0,
        }
    }
}

impl FromWorld for GameText {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let text_style = TextStyle {
            font: asset_server.load("FiraMono-Regular.ttf"),
            font_size: 16.0,
            color: Color::WHITE,
        };
        Self(text_style)
    }
}

struct UiNodes {
    root: Entity,
    states: [Entity; 3],
}

impl UiNodes {
    fn menu(&self) -> Entity {
        self.states[GameState::Menu as usize]
    }
    fn board(&self) -> Entity {
        self.states[GameState::Playing as usize]
    }
    fn game_over(&self) -> Entity {
        self.states[GameState::GameOver as usize]
    }
}

impl FromWorld for UiNodes {
    fn from_world(world: &mut World) -> Self {
        Self {
            root: world.spawn().id(),
            states: [(); 3].map(|_| world.spawn().id()),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

fn make_ui_root(mut commands: Commands, ui_nodes: Res<UiNodes>) {
    commands
        .entity(ui_nodes.root)
        .insert_bundle(NodeBundle {
            color: UiColor(Color::TURQUOISE),
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                flex_direction: FlexDirection::RowReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .push_children(&ui_nodes.states[..2]);
}

fn make_menu(mut commands: Commands, text: Res<GameText>, ui_nodes: Res<UiNodes>) {
    let menu_node = commands
        .entity(ui_nodes.menu())
        .insert_bundle(NodeBundle {
            color: UiColor(Color::DARK_GRAY),
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    let title = commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            ..text.bundle("noughts and crosses")
        })
        .id();
    let [play_button, quit_button] = [("play", ButtonCommand::Play), ("quit", ButtonCommand::Quit)]
        .map(|(label, button_command)| {
            let button_label = commands.spawn_bundle(text.bundle(label)).id();
            commands
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        margin: Rect::all(Val::Px(10.0)),
                        padding: Rect::all(Val::Px(10.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(button_command)
                .push_children(&[button_label])
                .id()
        });
    commands
        .entity(menu_node)
        .push_children(&[title, play_button, quit_button]);
}

fn make_board(
    mut commands: Commands,
    ui_nodes: Res<UiNodes>,
    text: Res<GameText>,
    board: Res<Board>,
) {
    commands.entity(ui_nodes.board()).insert_bundle(NodeBundle {
        color: UiColor(Color::DARK_GRAY),
        style: Style {
            display: Display::None,
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            size: Size {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
            },
            ..Default::default()
        },
        ..Default::default()
    });
    let s = commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::DARK_GRAY),
            style: Style {
                size: Size {
                    width: Val::Percent(50.0),
                    height: Val::Percent(70.0),
                },
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    let t = commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::DARK_GRAY),
            style: Style {
                size: Size {
                    width: Val::Percent(50.0),
                    height: Val::Percent(30.0),
                },
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands.entity(ui_nodes.board()).push_children(&[s, t]);
    commands.entity(t).push_children(&[ui_nodes.game_over()]);
    commands
        .entity(ui_nodes.game_over())
        .insert_bundle(NodeBundle {
            color: UiColor(Color::DARK_GRAY),
            style: Style {
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                },
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::Center,
                display: Display::None,
                ..Default::default()
            },
            ..Default::default()
        });
    let title = commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: Rect::all(Val::Px(10.0)),
                ..Default::default()
            },
            ..text.bundle("board")
        })
        .id();
    let grid = commands
        .spawn_bundle(NodeBundle {
            color: UiColor(Color::YELLOW),
            style: Style {
                size: Size {
                    width: Val::Auto,
                    height: Val::Auto,
                },
                padding: Rect::all(Val::Px(5.0)),
                margin: Rect::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    for y in 0..3 {
        let row = commands
            .spawn_bundle(NodeBundle {
                color: UiColor(Color::GREEN),
                style: Style {
                    size: Size {
                        width: Val::Auto,
                        height: Val::Auto,
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();
        let mut out = vec![];
        for x in 0..3 {
            let index = x + y * 3;
            let contents = commands
                .entity(board.entities[index])
                .insert_bundle(text.big_bundle(""))
                .id();
            let color = if (x + y) % 2 == 0 {
                Color::MAROON
            } else {
                Color::BLACK
            };
            let square = commands
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(TILE_SIZE), Val::Px(TILE_SIZE)),
                        margin: Rect::all(Val::Px(5.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    color: UiColor(color),
                    ..Default::default()
                })
                .insert(ButtonCommand::Grid(index))
                .push_children(&[contents])
                .id();
            out.push(square);
        }
        commands.entity(row).push_children(&out);
        commands.entity(grid).push_children(&[row]);
    }
    commands.entity(s).push_children(&[title, grid]);
}

fn make_game_over(mut commands: Commands, ui_nodes: Res<UiNodes>, text: Res<GameText>) {
    let text_bundle = text.bundle("game over");
    let game_over_message = commands
        .spawn_bundle(TextBundle {
            style: Style {
                margin: Rect {
                    bottom: Val::Px(20.0),
                    ..Default::default()
                },
                ..text_bundle.style
            },
            ..text_bundle
        })
        .id();
    let [play_button, quit_button] = [
        ("play again", ButtonCommand::Play),
        ("quit", ButtonCommand::Quit),
    ]
    .map(|(label, button_command)| {
        let button_label = commands.spawn_bundle(text.bundle(label)).id();
        commands
            .spawn_bundle(ButtonBundle {
                style: Style {
                    margin: Rect::all(Val::Px(10.0)),
                    padding: Rect::all(Val::Px(10.0)),
                    size: Size {
                        height: Val::Px(40.0),
                        width: Val::Px(100.0),
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(button_command)
            .push_children(&[button_label])
            .id()
    });
    commands.entity(ui_nodes.game_over()).push_children(&[
        game_over_message,
        play_button,
        quit_button,
    ]);
}

fn update_display(
    game_state: Res<State<GameState>>,
    ui_nodes: Res<UiNodes>,
    mut query: Query<&mut Style>,
) {
    match game_state.current() {
        GameState::Menu => [true, false, false],
        GameState::Playing => [false, true, false],
        GameState::GameOver => [false, true, true],
    }
    .into_iter()
    .zip(ui_nodes.states)
    .for_each(|(d, entity)| {
        query.get_mut(entity).unwrap().display = if d { Display::Flex } else { Display::None }
    });
}

#[derive(Component)]
enum ButtonCommand {
    Play,
    Quit,
    Grid(usize),
}

fn button_system(
    mut event: EventWriter<AppExit>,
    mut state: ResMut<State<GameState>>,
    mut board: ResMut<Board>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ButtonCommand),
        (Changed<Interaction>, With<Button>),
    >,
    mut text: Query<&mut Text>,
) {
    for (interaction, mut color, button_command) in interaction_query.iter_mut() {
        *color = match *interaction {
            Interaction::Clicked => {
                match button_command {
                    ButtonCommand::Play => {
                        board.clear();
                        state.set(GameState::Playing).unwrap();
                    }
                    ButtonCommand::Quit => event.send(AppExit),
                    ButtonCommand::Grid(index) => {
                        if *state.current() == GameState::Playing
                            && matches!(board.tiles[*index], Tile::Empty)
                        {
                            if board.play_move(*index) {
                                state.set(GameState::GameOver).unwrap();
                            }
                            for i in 0..9 {
                                let e = board.entities[i];
                                let mut label = text.get_mut(e).unwrap();
                                label.sections[0].value = board.tiles[i].piece().to_owned();
                            }
                        }
                    }
                }
                PRESSED_BUTTON.into()
            }
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn clear_grid(board: Res<Board>, mut text: Query<&mut Text>) {
    for &e in board.entities.iter() {
        text.get_mut(e).unwrap().sections[0].value = "".to_string();
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_state(GameState::Menu)
        .init_resource::<GameText>()
        .init_resource::<UiNodes>()
        .init_resource::<Board>()
        .add_startup_system_set(
            SystemSet::new()
            .with_system(setup)
            .with_system(make_ui_root)
            .with_system(make_menu)
            .with_system(make_board)
            .with_system(make_game_over),
        )
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(clear_grid))
        .add_system(update_display)
        .add_system(button_system)
        .run();
}
