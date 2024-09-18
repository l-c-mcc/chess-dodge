#![allow(dead_code)]

use bevy::prelude::*;
use nanorand::Rng;
use std::collections::HashMap;

const SCREEN_LEN: f32 = 300. * SCALE;
const N_TILES: usize = 8;
const SCALE: f32 = 2.5;
const SQUARE_LEN: f32 = 32. * SCALE;
const TILE_GAP: f32 = 2. * SCALE;
const TILE_DIS: f32 = TILE_GAP + SQUARE_LEN;
const FROM_ORIGIN: f32 = TILE_DIS / 2.;

const PLAYER_MOVE_SPEED: f32 = 0.15;

const PLAYER_SIDE: Side = Side::Black;
const OPP_SIDE: Side = Side::White;

const MAX_SPAWN_DUR: f32 = 1.5;
const MIN_SPAWN_DUR: f32 = 0.6;
const SPAWN_DUR_DECR: f32 = 0.1;

// min is faster than max
const MAX_OPP_SPEED: f32 = 1.2;
const MIN_OPP_SPEED: f32 = 0.4;
const OPP_SPEED_DECR: f32 = 0.05;

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
        .add_event::<ToDelete>()
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (
                player_input,
                opp_move,
                update_board,
                spawn_opp_pieces,
                move_pieces,
                clear_pieces,
            )
                .chain(),
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
        (Piece::Bishop, Side::Black, "chessPieces/bishopBlack.png"),
        (Piece::Knight, Side::Black, "chessPieces/knightBlack.png"),
        (Piece::Rook, Side::White, "chessPieces/rookWhite.png"),
        (Piece::Bishop, Side::White, "chessPieces/bishopWhite.png"),
        (Piece::Queen, Side::White, "chessPieces/queenWhite.png"),
    ];
    for sprite in piece_sprites {
        sprite_map.insert((sprite.0, sprite.1), asset_server.load(sprite.2));
    }
    let piece_sprites = PieceSprites { map: sprite_map };
    let player_piece = Piece::Knight;
    let player_id = commands
        .spawn(
            PlayerPiece::new(piece_sprites.get(player_piece, PLAYER_SIDE), start_vec, player_piece, PLAYER_MOVE_SPEED)
        ).id();
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
            timer: Timer::from_seconds(0.0, TimerMode::Once),
            cur_duration: MAX_SPAWN_DUR,
            cur_piece_speed: MAX_OPP_SPEED,
        },
    ));
    commands.insert_resource(piece_sprites);
    commands.insert_resource(GameOver(false));
}

type PieceSide = (Piece, Side);

#[derive(Resource)]
struct PieceSprites {
    map: HashMap<PieceSide, Handle<Image>>,
}

#[derive(Resource)]
struct GameOver(bool);

impl PieceSprites {
    fn get(&self, piece: Piece, side: Side) -> Handle<Image> {
        self.map.get(&(piece, side)).unwrap().clone()
    }
}

#[derive(Event, Debug)]
struct MoveReq {
    id: TileType,
    mov: Direction,
}

#[derive(Event)]
struct Move {
    id: Entity,
    mov: MoveResult,
}

#[derive(Event, Deref)]
struct ToDelete {
    id: Entity,
}

enum MoveResult {
    NewLoc(Vec3),
    Delete,
}

#[derive(Component)]
struct Board {
    board: [[TileType; N_TILES]; N_TILES],
}

#[derive(Component)]
struct Spawner {
    timer: Timer,
    cur_duration: f32,
    cur_piece_speed: f32,
}

#[derive(Bundle)]
struct PieceBundle<T: Component> {
    sprite: SpriteBundle,
    mob: T,
    piece: Piece,
}

type OpponentPiece = PieceBundle<Opponent>;
type PlayerPiece = PieceBundle<Player>;

#[derive(Component)]
struct Player {
    timer_dur: f32,
    timer: Timer,
    can_move: bool,
}

#[derive(Component)]
struct Opponent {
    timer: Timer,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileType {
    Empty,
    Player(Entity),
    Opponent(Entity),
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Up,
    UpLeft,
    UpRight,
    Left,
    Right,
    Down,
    DownLeft,
    DownRight,
    UpLeftWide,
    UpLeftNarrow,
    UpRightNarrow,
    UpRightWide,
    DownRightWide,
    DownRightNarrow,
    DownLeftNarrow,
    DownLeftWide,
    None,
}

// to-do: organize
trait NewMob {
    fn new(move_time: f32) -> Self;
}

impl NewMob for Opponent {
    fn new(move_time: f32) -> Self {
        Self {
            timer: Timer::from_seconds(move_time, TimerMode::Repeating),
        }
    }
}

impl NewMob for Player {
    fn new(move_time: f32) -> Self {
        Self {
            timer_dur: move_time,
            timer: Timer::from_seconds(move_time, TimerMode::Repeating), // to-do: look into
            can_move: true,
        }
    }
}

impl<T: Component + NewMob> PieceBundle<T> {
    fn new(texture: Handle<Image>, coords: Vec3, piece: Piece, move_time: f32) -> Self {
        Self {
            sprite: SpriteBundle {
                texture,
                transform: Transform {
                    translation: coords,
                    scale: Vec3::new(SCALE, SCALE, 1.0),
                    ..default()
                },
                ..default()
            },
            mob: T::new(move_time),
            piece,
        }
    }
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
    fn mov(&mut self, req: &MoveReq, new_board: &mut Board) -> Option<Move> {
        if req.id == TileType::Empty {
            return None;
        }
        let mut orig_x = None;
        let mut orig_y = None;
        let mut xy = None;
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
        if xy.is_none() {
            match req.id {
                TileType::Player(x) => panic!("Player {:?} supposed to be on board not found", x),
                TileType::Opponent(x) => panic!(
                    "Piece {:?} supposed to be on board not found, {:?}",
                    x, self.board
                ),
                TileType::Empty => panic!("Searching for empty"),
            }
        }
        let orig_x = orig_x.unwrap();
        let orig_y = orig_y.unwrap();
        let xy = Self::new_xy(req.mov, xy.unwrap());
        let mut collision_check = |x, y, id, player| -> Option<Move> {
            let row: &mut [TileType; 8] = &mut new_board.board[y];
            match row[x] {
                TileType::Empty => {
                    if player {
                        row[x] = TileType::Player(id);
                    } else {
                        row[x] = TileType::Opponent(id);
                    }
                    Some(Move {
                        id,
                        mov: MoveResult::NewLoc(Self::coord_to_vec(x, y)),
                    })
                }
                TileType::Player(player_id) => {
                    row[x] = TileType::Opponent(id);
                    Some(Move {
                        id: player_id,
                        mov: MoveResult::Delete,
                    })
                }
                TileType::Opponent(_) => Some(Move {
                    id,
                    mov: MoveResult::Delete,
                }),
            }
        };
        match (xy, req.id) {
            (None, TileType::Player(id)) => {
                new_board.board[orig_y][orig_x] = TileType::Player(id);
                None
            }
            (None, TileType::Opponent(id)) => Some(Move {
                id,
                mov: MoveResult::Delete,
            }),
            (Some((x, y)), TileType::Player(id)) => collision_check(x, y, id, true),
            (Some((x, y)), TileType::Opponent(id)) => collision_check(x, y, id, false),
            (_, _) => panic!("Should not be here"),
        }
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
            Direction::DownLeft => {
                x -= 1;
                y += 1;
            }
            Direction::DownRight => {
                x += 1;
                y += 1;
            }
            Direction::UpLeft => {
                x -= 1;
                y -= 1;
            }
            Direction::UpRight => {
                x += 1;
                y -= 1;
            }
            Direction::UpLeftWide => {
                x -= 2;
                y -= 1;
            }
            Direction::UpLeftNarrow => {
                x -= 1;
                y -= 2;
            }
            Direction::UpRightNarrow => {
                x += 1;
                y -= 2;
            }
            Direction::UpRightWide => {
                x += 2;
                y -= 1;
            }
            Direction::DownRightWide => {
                x += 2;
                y += 1;
            }
            Direction::DownRightNarrow => {
                x += 1;
                y += 2;
            }
            Direction::DownLeftNarrow => {
                x -= 1;
                y += 2;
            }
            Direction::DownLeftWide => {
                x -= 2;
                y += 1;
            }
            Direction::None => (),
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
    mut query: Query<(&mut Player, Entity, &Piece)>,
    mut move_req_writer: EventWriter<MoveReq>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut update_sent = false;
    let (mut player, entity, piece) = query.single_mut();
    if player.timer.tick(time.delta()).just_finished() {
        player.can_move = true;
    }
    if player.can_move {
        let kp = |kc| keyboard_input.pressed(kc);
        let mov = match piece {
            Piece::Rook => rook_move(kp),
            Piece::Bishop => bishop_move(kp),
            Piece::Knight => knight_move(kp),
            _ => panic!("Other pieces not implemented"),
        };
        if let Some(dir) = mov {
            move_req_writer.send(MoveReq {
                id: TileType::Player(entity),
                mov: dir,
            });
            update_sent = true;
            player.can_move = false;
            player.timer = Timer::from_seconds(PLAYER_MOVE_SPEED, TimerMode::Once);
        }
    }
    if !update_sent {
        move_req_writer.send(MoveReq {
            id: TileType::Player(entity),
            mov: Direction::None,
        });
    }
}

fn rook_move(kp: impl Fn(KeyCode) -> bool) -> Option<Direction> {
    use KeyCode::{KeyA, KeyD, KeyS, KeyW};
    match (kp(KeyW), kp(KeyS), kp(KeyA), kp(KeyD)) {
        (true, false, false, false) => Some(Direction::Up),
        (false, true, false, false) => Some(Direction::Down),
        (false, false, true, false) => Some(Direction::Left),
        (false, false, false, true) => Some(Direction::Right),
        _ => None,
    }
}

fn bishop_move(kp: impl Fn(KeyCode) -> bool) -> Option<Direction> {
    use KeyCode::{KeyA, KeyD, KeyS, KeyW};
    match (kp(KeyW), kp(KeyS), kp(KeyA), kp(KeyD)) {
        (true, false, true, false) => Some(Direction::UpLeft),
        (true, false, false, true) => Some(Direction::UpRight),
        (false, true, false, true) => Some(Direction::DownRight),
        (false, true, true, false) => Some(Direction::DownLeft),
        _ => None,
    }
}

fn knight_move(kp: impl Fn(KeyCode) -> bool) -> Option<Direction> {
    use KeyCode::{KeyU, KeyI, KeyO, KeyP, KeyJ, KeyK, KeyL, Semicolon};
    if kp(KeyU) {
        Some(Direction::UpLeftWide)
    } else if kp(KeyI) {
        Some(Direction::UpLeftNarrow)
    } else if kp(KeyO) {
        Some(Direction::UpRightNarrow)
    } else if kp(KeyP) {
        Some(Direction::UpRightWide)
    } else if kp(KeyJ) {
        Some(Direction::DownLeftWide)
    } else if kp(KeyK) {
        Some(Direction::DownLeftNarrow)
    } else if kp(KeyL) {
        Some(Direction::DownRightNarrow)
    } else if kp(Semicolon) {
        Some(Direction::DownRightWide)
    } else {
        None
    }
}

fn opp_move(
    mut query: Query<(Entity, &mut Opponent, &Piece)>,
    mut move_req_writer: EventWriter<MoveReq>,
    time: Res<Time>,
) {
    for (entity, mut opponent, piece) in query.iter_mut() {
        if opponent.timer.tick(time.delta()).just_finished() {
            let mut rng = nanorand::pcg64::Pcg64::new();
            let dir = match piece {
                Piece::Rook => {
                    Some(Direction::Down)
                }
                Piece::Bishop => {
                    let mut options = vec![Direction::DownLeft, Direction::DownRight];
                    rng.shuffle(&mut options);
                    options.pop()
                }
                Piece::Queen => {
                    let mut options = vec![Direction::DownLeft, Direction::Down, Direction::DownRight];
                    rng.shuffle(&mut options);
                    options.pop()
                }
                _ => panic!("Spawned unimplemented piece"),
            };
            move_req_writer.send(MoveReq {
                id: TileType::Opponent(entity),
                mov: dir.unwrap(),
            });
        } else {
            move_req_writer.send(MoveReq {
                id: TileType::Opponent(entity),
                mov: Direction::None,
            });
        }
    }
}

fn update_board(
    mut query: Query<&mut Board>,
    game_over: Res<GameOver>,
    mut move_req_reader: EventReader<MoveReq>,
    mut move_writer: EventWriter<Move>,
) {
    if !game_over.0 {
        let mut new_board = Board::default();
        let mut old_board = query.single_mut();
        for req in move_req_reader.read() {
            if let Some(mov) = old_board.mov(req, &mut new_board) {
                move_writer.send(mov);
            }
        }
        old_board.board = new_board.board;
    }
}

fn spawn_opp_pieces(
    mut query: Query<(&mut Board, &mut Spawner)>,
    piece_sprites: Res<PieceSprites>,
    time: Res<Time>,
    game_over: Res<GameOver>,
    mut commands: Commands,
    mut move_writer: EventWriter<Move>,
) {
    if !game_over.0 {
        let (mut board, mut spawner) = query.get_single_mut().unwrap();
        if spawner.timer.tick(time.delta()).just_finished() {
            let mut rng = nanorand::pcg64::Pcg64::new();
            let mut spawn_locations = vec![];
            let top_row = &board.board[0];
            let mut is_player = None;
            for (elem, tile) in top_row.iter().enumerate().take(N_TILES) {
                match *tile {
                    TileType::Opponent(_) => (),
                    TileType::Player(x) => {
                        is_player = Some((elem, x));
                        spawn_locations.push(elem);
                    }
                    _ => spawn_locations.push(elem),
                }
            }
            rng.shuffle(&mut spawn_locations);
            let target = spawn_locations.pop();
            if let Some(col) = target {
                let target_coords = Board::coord_to_vec(col, 0);
                let cur_speed = spawner.cur_piece_speed;
                let offsets = vec![0.0, 0.3, 0.6, 0.9];
                let mut possible_speeds = vec![];
                for offset in offsets {
                    possible_speeds.push(cur_speed + offset);
                }
                rng.shuffle(&mut possible_speeds);
                let speed = possible_speeds.pop().unwrap();
                let piece_num = rng.generate_range(1..=17);
                let piece = if piece_num < 2 {
                    Piece::Queen
                } else if piece_num < 6 {
                    Piece::Bishop
                } else {
                    Piece::Rook
                };
                let new_piece = commands
                    .spawn(OpponentPiece::new(
                        (*piece_sprites.map.get(&(piece, OPP_SIDE)).unwrap()).clone(),
                        target_coords,
                        piece,
                        speed,
                    ))
                    .id();
                board.board[0][col] = TileType::Opponent(new_piece);
                if let Some((player_col, player_id)) = is_player {
                    if player_col == col {
                        move_writer.send(Move {
                            id: player_id,
                            mov: MoveResult::Delete,
                        });
                    }
                }
            }
            if spawner.cur_duration > MIN_SPAWN_DUR {
                spawner.cur_duration -= SPAWN_DUR_DECR;
            }
            if spawner.cur_piece_speed > MIN_OPP_SPEED {
                spawner.cur_piece_speed -= OPP_SPEED_DECR;
            }
            spawner.timer = Timer::from_seconds(spawner.cur_duration, TimerMode::Once);
        }
    }
}

fn move_pieces(
    mut query: Query<(Entity, &mut Transform, Option<&Player>), With<Piece>>,
    mut game_over: ResMut<GameOver>,
    mut move_reader: EventReader<Move>,
    mut delete_writer: EventWriter<ToDelete>,
) {
    let mut hash_map: HashMap<Entity, (Mut<'_, Transform>, Option<&Player>)> = HashMap::new();
    for (entity_id, transform, player) in query.iter_mut() {
        hash_map.insert(entity_id, (transform, player));
    }
    for event in move_reader.read() {
        let entity_id = event.id;
        let mut entity = hash_map.remove(&entity_id).unwrap();
        match (&event.mov, entity.1) {
            (MoveResult::NewLoc(vec), _) => entity.0.translation = *vec,
            (MoveResult::Delete, None) => {
                delete_writer.send(ToDelete { id: entity_id });
            }
            (MoveResult::Delete, _) => {
                // hide the player when the game ends
                entity.0.translation = Vec3::new(10000., 10000., 0.);
                game_over.0 = true;
            }
        }
        hash_map.insert(entity_id, entity);
    }
}

fn clear_pieces(mut commands: Commands, mut delete_reader: EventReader<ToDelete>) {
    for event in delete_reader.read() {
        let entity = **event;
        commands.entity(entity).despawn();
    }
}
