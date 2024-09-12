#![allow(dead_code)]

use bevy::prelude::*;
use nanorand::Rng;
use std::collections::HashMap;

const SCREEN_LEN: f32 = 300. * SCALE;
const N_TILES: usize = 8;
const SCALE: f32 = 3.;
const SQUARE_LEN: f32 = 32. * SCALE;
const TILE_GAP: f32 = 2. * SCALE;
const TILE_DIS: f32 = TILE_GAP + SQUARE_LEN;
const FROM_ORIGIN: f32 = TILE_DIS / 2.;
const PLAYER_MOVE_SPEED: f32 = 0.13;
const TILE_MIN: f32 = -4. * TILE_DIS;
const TILE_MAX: f32 = -1. * TILE_MIN;

const PLAYER_SIDE: Side = Side::Black;
const OPP_SIDE: Side = Side::White;

const MAX_SPAWN_DUR: f32 = 1.0;
const MIN_SPAWN_DUR: f32 = 0.05;
const SPAWN_DUR_DECR: f32 = 0.01;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (SCREEN_LEN, SCREEN_LEN).into(),
                ..default()
            }),
            ..default()
        }))
        .add_event::<MoveReq>()
        .add_event::<Move>()
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (player_input, update_board, spawn_opp_pieces, move_pieces).chain(),
        )
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start_x = 3;
    let start_y = 3;
    let start_vec = Board::coord_to_vec(start_x, start_y);
    commands.spawn(Camera2dBundle::default());

    let board_sprite = asset_server.load("chessBoards/chessBoard.png");
    let mut sprite_map: HashMap<PieceSide, Handle<Image>> = HashMap::new();
    let piece_sprites = [
        (Piece::Rook, Side::Black, "chessPieces/rookBlack.png"),
        (Piece::Pawn, Side::White, "chessPieces/pawnWhite.png"),
    ];
    for sprite in piece_sprites {
        sprite_map.insert((sprite.0, sprite.1), asset_server.load(sprite.2));
    }
    let piece_sprites = PieceSprites { map: sprite_map };
    let player_id = commands
        .spawn((
            SpriteBundle {
                texture: piece_sprites.get(Piece::Rook, PLAYER_SIDE),
                transform: Transform {
                    scale: Vec3::new(SCALE, SCALE, 1.),
                    translation: start_vec,
                    ..default()
                },
                ..default()
            },
            Player {
                timer: Timer::from_seconds(PLAYER_MOVE_SPEED, TimerMode::Repeating),
            },
            Piece::Rook,
        ))
        .id();
    let mut board = Board::default();
    board.place_piece(start_x, start_y, TileType::Player(player_id));
    commands.spawn((
        SpriteBundle {
            texture: board_sprite,
            transform: Transform {
                scale: Vec3::new(SCALE, SCALE, 0.),
                ..default()
            },
            ..default()
        },
        board,
        Spawner {
            timer: Timer::from_seconds(MAX_SPAWN_DUR, TimerMode::Once),
            cur_duration: MAX_SPAWN_DUR,
        },
    ));
    commands.insert_resource(piece_sprites);
}

type PieceSide = (Piece, Side);

#[derive(Resource)]
struct PieceSprites {
    map: HashMap<PieceSide, Handle<Image>>,
}

impl PieceSprites {
    fn get(&self, piece: Piece, side: Side) -> Handle<Image> {
        self.map.get(&(piece, side)).unwrap().clone()
    }
}

#[derive(Event)]
struct MoveReq {
    id: TileType,
    mov: Direction,
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

#[derive(Component)]
struct Spawner {
    timer: Timer,
    cur_duration: f32,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            board: [[TileType::Empty; N_TILES]; N_TILES],
        }
    }
}

#[derive(Bundle)]
struct OpponentPiece {
    sprite: SpriteBundle,
    opp: Opponent,
    piece: Piece,
}

#[derive(Component)]
struct Player {
    timer: Timer,
}

#[derive(Component)]
struct Opponent;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Component)]
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum TileType {
    Empty,
    Player(Entity),
    Opponent(Entity),
}

#[derive(Clone, Copy)]
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


impl OpponentPiece {
    fn new(texture: Handle<Image>, coords: Vec3, piece: Piece) -> OpponentPiece {
        OpponentPiece {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: coords,
                    scale: Vec3::new(SCALE, SCALE, 1.0),
                    ..default()
                },
                ..default()
            },
            opp: Opponent,
            piece,
        }
    }
}

impl Board {
    fn coord_to_vec(x: usize, y: usize) -> Vec3 {
        let x_board = x as f32;
        let y_board = y as f32;
        // treat (3,3) as origin
        let x_coord = -FROM_ORIGIN + (x_board - 3.) * TILE_DIS;
        let y_coord = FROM_ORIGIN - (y_board - 3.) * TILE_DIS;
        Vec3::new(x_coord, y_coord, 1.)
    }

    fn place_piece(&mut self, x: usize, y: usize, entity: TileType) -> bool {
        if let TileType::Empty = self.board[y][x] {
            self.board[y][x] = entity;
            true
        } else {
            false
        }
    }

    // this seems inefficient but worse case scenario is 64 * 64 compares per update?
    // todo: add collision detection
    fn mov(&mut self, req: &MoveReq) -> Option<Move> {
        if req.id == TileType::Empty {
            return None;
        }
        let mut xy = None;
        let mut orig_x = None;
        let mut orig_y = None;
        for row in 0..N_TILES {
            for col in 0..N_TILES {
                let cur = self.board[row][col];
                if cur == req.id && xy.is_none() {
                    orig_x = Some(col);
                    orig_y = Some(row);
                    xy = Some((col, row));
                } else if cur == req.id && xy.is_some() {
                    panic!("Entity on board multiple times.")
                }
            }
        }
        let orig_x = orig_x.unwrap();
        let orig_y = orig_y.unwrap();
        if xy.is_none() {
            panic!("Piece supposed to be on board not found")
        }
        let xy = Self::new_xy(req.mov, xy.unwrap());
        let mov = match (xy, req.id) {
            (None, TileType::Player(_)) => MoveResult::NoMov,
            (None, TileType::Opponent(_)) => MoveResult::Delete,
            (Some((x, y)), _) => {
                self.board[y][x] = req.id;
                self.board[orig_y][orig_x] = TileType::Empty; // will be refreshed later
                MoveResult::NewLoc(Self::coord_to_vec(x, y))
            }
            (_, _) => panic!("Should not be here"),
        };
        let id = match req.id {
            TileType::Player(x) => x,
            TileType::Opponent(x) => x,
            _ => panic!("Trying to find entity in empty tile"),
        };
        Some(Move { id, mov })
    }

    fn new_xy(dir: Direction, xy: (usize, usize)) -> Option<(usize, usize)> {
        fn in_bounds(val: i32) -> bool {
            !(val < 0 || val >= N_TILES as i32)
        }
        let mut x = xy.0 as i32;
        let mut y = xy.1 as i32;
        match dir {
            Direction::Up => y -= 1,
            Direction::Down => y += 1,
            Direction::Left => x -= 1,
            Direction::Right => x += 1,
            _ => panic!("undefined movement"),
        }
        if in_bounds(x) && in_bounds(y) {
            Some((x as usize, y as usize))
        } else {
            None
        }
    }
}

fn player_input(
    time: Res<Time>,
    mut query: Query<(&mut Player, Entity)>,
    mut move_req_writer: EventWriter<MoveReq>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut player, entity) = query.single_mut();
    if player.timer.tick(time.delta()).just_finished() {
        use KeyCode::{KeyA, KeyD, KeyS, KeyW};
        let kp = |kc| keyboard_input.pressed(kc);
        let mut mov = None;
        match (kp(KeyW), kp(KeyS), kp(KeyA), kp(KeyD)) {
            (true, false, false, false) => mov = Some(Direction::Up),
            (false, true, false, false) => mov = Some(Direction::Down),
            (false, false, true, false) => mov = Some(Direction::Left),
            (false, false, false, true) => mov = Some(Direction::Right),
            _ => (),
        }
        if let Some(dir) = mov {
            move_req_writer.send(MoveReq {
                id: TileType::Player(entity),
                mov: dir,
            });
        }
    }
}

fn update_board(
    mut query: Query<&mut Board>,
    mut move_req_reader: EventReader<MoveReq>,
    mut move_writer: EventWriter<Move>,
) {
    let mut board = query.single_mut();
    for req in move_req_reader.read() {
        if let Some(mov) = board.mov(req) {
            move_writer.send(mov);
        }
    }
}

fn spawn_opp_pieces(
    mut query: Query<(&mut Board, &mut Spawner)>,
    piece_sprites: Res<PieceSprites>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let (mut board, mut spawner) = query.get_single_mut().unwrap();
    if spawner.timer.tick(time.delta()).just_finished() {
        spawner.cur_duration -= SPAWN_DUR_DECR;
        spawner.timer = Timer::from_seconds(spawner.cur_duration, TimerMode::Once);
        let mut rng = nanorand::pcg64::Pcg64::new();
        let mut spawn_locations = vec![];
        let top_row = &board.board[0];
        // todo: player can avoid all danger by living in top row
        for (elem, tile) in top_row.iter().enumerate().take(N_TILES) {
            if tile == &TileType::Empty {
                spawn_locations.push(elem);
            }
        }
        rng.shuffle(&mut spawn_locations);
        let target = spawn_locations.pop();
        if let Some(col) = target {
            let target_coords = Board::coord_to_vec(col, 0);
            let new_piece = commands
                .spawn(OpponentPiece::new(
                    (*piece_sprites.map.get(&(Piece::Pawn, OPP_SIDE)).unwrap()).clone(),
                    target_coords,
                    Piece::Pawn,
                ))
                .id();
            board.board[0][col] = TileType::Opponent(new_piece);
        }
    }
}

fn move_pieces(
    mut query: Query<(Entity, &mut Transform, Option<&Player>), With<Piece>>,
    mut move_reader: EventReader<Move>,
) {
    let mut hash_map: HashMap<Entity, (Mut<'_, Transform>, Option<&Player>)> = HashMap::new();
    for (entity_id, transform, player) in query.iter_mut() {
        hash_map.insert(entity_id, (transform, player));
    }
    for event in move_reader.read() {
        let entity_id = event.id;
        // there should be only one move per entity per cycle, so this panicking is a sign of something moving more than it should
        let mut entity = hash_map.remove(&entity_id).unwrap();
        if entity.1.is_some() {
            if let MoveResult::NewLoc(vec) = event.mov {
                entity.0.translation = vec;
            }
        }
    }
}
