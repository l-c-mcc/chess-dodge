use bevy::prelude::*;
use std::collections::HashMap;

const N_TILES: usize = 8;
const SCALE: f32 = 2.;
const SQUARE_LEN: f32 = 32. * SCALE;
const TILE_GAP: f32 = 2. * SCALE;
const TILE_DIS: f32 = TILE_GAP + SQUARE_LEN;
const FROM_ORIGIN: f32 = TILE_DIS / 2.;
const PLAYER_SIDE: Side = Side::Black;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (600., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (player_input, update_board, player_move).chain(),
        )
        .run();
}

type PieceSide = (Piece, Side);

#[derive(Resource)]
struct PieceSprites {
    map: HashMap<PieceSide, Handle<Image>>,
}

impl PieceSprites {
    fn get(self: &Self, piece: Piece, side: Side) -> Handle<Image> {
        self.map.get(&(piece, side)).unwrap().clone()
    }
}

#[derive(Event)]
struct MoveReq {
    id: Entity,
    mov: Direction,
    player: bool,
}

#[derive(Event)]
struct Move {
    id: Entity,
    mov: MoveResult,
}

enum MoveResult {
    NewLoc(Vec3),
    Delete,
    NoMov,
}

#[derive(Component)]
struct Board {
    board: [[TileType; N_TILES]; N_TILES],
}

impl Default for Board {
    fn default() -> Self {
        Board {
            board: [[TileType::Empty; N_TILES]; N_TILES],
        }
    }
}

impl Board {
    fn coord_to_vec(x: usize, y: usize) -> Vec3 {
        let mut xy_screen;
        match (x > 3, y > 3) {
            (true, true) => xy_screen = (FROM_ORIGIN, -FROM_ORIGIN),
            (true, false) => xy_screen = (FROM_ORIGIN, FROM_ORIGIN),
            (false, true) => xy_screen = (-FROM_ORIGIN, -FROM_ORIGIN),
            (false, false) => xy_screen = (-FROM_ORIGIN, FROM_ORIGIN),
        }
        let x_board = (x - 3) as f32;
        let y_board = (y - 3) as f32;
        Vec3::new(
            xy_screen.0 + (x_board * TILE_DIS),
            xy_screen.1 + (y_board * TILE_DIS),
            1.0,
        )
    }

    fn place_piece(&mut self, x: usize, y: usize, entity: TileType) -> bool {
        if let TileType::Empty = self.board[y][x] {
            self.board[y][x] = entity;
            true
        } else {
            false
        }
    }

    // this seems inefficient but worse case scenario is 64 * 64 compares?
    fn mov(&mut self, req: MoveReq) -> Move {
        todo!()
    }
}

#[derive(Component, Deref)]
struct Player(Piece);

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum Piece {
    Rook,
    Bishop,
    Knight,
    Pawn,
    Queen,
    King,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
enum Side {
    White,
    Black,
}

#[derive(Clone, Copy)]
enum TileType {
    Empty,
    Player(Entity),
    Opponent(Entity),
}

enum Direction {
    Up,
    UpLeft,
    UpRight,
    Left,
    Right,
    Down,
    DownLeft,
    DownRight,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let board_sprite = asset_server.load("chessBoards/chessBoard.png");
    let mut sprite_map: HashMap<PieceSide, Handle<Image>> = HashMap::new();
    sprite_map.insert(
        (Piece::Rook, Side::Black),
        asset_server.load("chessPieces/rookBlack.png"),
    );
    let piece_sprites = PieceSprites { map: sprite_map };
    let player_id = commands
        .spawn((
            SpriteBundle {
                texture: piece_sprites.get(Piece::Rook, PLAYER_SIDE),
                transform: Transform {
                    scale: Vec3::new(SCALE, SCALE, 1.),
                    translation: Vec3::new(FROM_ORIGIN, -FROM_ORIGIN, 1.),
                    ..default()
                },
                ..default()
            },
            Player(Piece::Rook),
        ))
        .id();
    let mut board = Board::default();
    board.place_piece(4, 4, TileType::Player(player_id));
    commands.spawn((
        SpriteBundle {
            texture: board_sprite,
            transform: Transform {
                scale: Vec3::new(SCALE, SCALE, 1.),
                ..default()
            },
            ..default()
        },
        board,
    ));
}

fn move_board(mut query: Query<&mut Transform, With<Board>>) {
    let mut board_transform = query.single_mut();
    //board_transform.translation.x += 1.0;
}

fn player_input(
    query: Query<(&Player, Entity)>,
    mut move_req_writer: EventWriter<MoveReq>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    use KeyCode::{KeyA, KeyD, KeyS, KeyW};
    let kp = |kc| keyboard_input.pressed(kc);
    let mut mov = None;
    match (kp(KeyW), kp(KeyS)) {
        (true, false) => mov = Some(Direction::Up),
        (false, true) => mov = Some(Direction::Down),
        _ => (),
    }
    let (player, entity) = query.single();
    if let Some(dir) = mov {
        move_req_writer.send(MoveReq {
            id: entity,
            mov: dir,
            player: true,
        });
    }
}

fn update_board(
    query: Query<&mut Board>,
    mut move_req_reader: EventReader<MoveReq>,
    mut move_writer: EventWriter<Move>,
) {
}

fn player_move() {}
