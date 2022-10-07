//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]
use chess::*;
use std::collections::HashMap;
use std::env;
use std::thread;
use std::sync::{Mutex, Arc};
use std::net::TcpStream;
mod networking;
mod net;

const image_folder : &'static str = "/Users/c14/repos/chess-ggez/target/debug";

const SCREEN_SIZE: (f32, f32) = (
    1080.0,
    1080.0,
);
use ggez::{
    event,
    graphics::{self, Color, Rect},
    Context, GameResult,
};

use mint::Point2;

struct MainState {
    pos_x: f32,
    circle: graphics::Mesh,
    squares : Vec<graphics::Mesh>,
    square_size : f32,
    img_map : HashMap<String, graphics::Image>,
    chess_board : Arc<Mutex<chess::ChessBoard>>,
    selected_piece : Option<chess::Square>,
    tcp_stream : Arc<Mutex<Option<TcpStream>>>,
    is_server : bool,
}

fn piece_to_string(piece : chess::Piece, color : chess::Color) -> String {
    let mut built = "".to_owned();
    match color {
        chess::Color::White => {
            built.push('w');
        }
        chess::Color::Black => {
            built.push('b');
        }
        _ => {}
    }
    match piece {
        chess::Piece::Pawn => {
            built.push('P');
        },
        chess::Piece::Knight => {
            built.push('N');
        },
        chess::Piece::Bishop => {
            built.push('B');
        },
        chess::Piece::Rook => {
            built.push('R');
        },       
        chess::Piece::Queen => {
            built.push('Q');
        },
        chess::Piece::King => {
            built.push('K');
        },
        _ => {}
    }
    built
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2 { x: 0.0, y : 0.0},
            100.0,
            2.0,
            Color::WHITE,
        )?;
        let scale = SCREEN_SIZE.0 / (8 as f32);
        
        let mut squares : Vec<graphics::Mesh> = Vec::new();
        squares.push(graphics::Mesh::new_rectangle(
                ctx, 
                graphics::DrawMode::fill(),
                Rect {x: 0.0, y : 0.0, w : scale as f32, h : scale as f32},
                Color::WHITE 
            )?);
        squares.push(graphics::Mesh::new_rectangle(
                ctx, 
                graphics::DrawMode::fill(),
                Rect {x: 0.0, y : 0.0, w : scale as f32, h : scale as f32},
                Color::BLACK 
            )?);
        
        let mut imgs : HashMap<String, graphics::Image>
  = HashMap::new();
        imgs.insert(
            "wP".to_string(), 
            graphics::Image::from_path(ctx, "/wP.png", true)?
        );
        imgs.insert(
            "wN".to_string(), 
            graphics::Image::from_path(ctx, "/wN.png", true)?
        );
        imgs.insert(
            "wB".to_string(), 
            graphics::Image::from_path(ctx, "/wB.png", true)?
        );
        imgs.insert(
            "wR".to_string(), 
            graphics::Image::from_path(ctx, "/wR.png", true)?
        );
        imgs.insert(
            "wQ".to_string(), 
            graphics::Image::from_path(ctx, "/wQ.png", true)?
        );
        imgs.insert(
            "wK".to_string(), 
            graphics::Image::from_path(ctx, "/wK.png", true)?
        );
        imgs.insert(
            "bP".to_string(), 
            graphics::Image::from_path(ctx, "/bP.png", true)?
        );
        imgs.insert(
            "bN".to_string(), 
            graphics::Image::from_path(ctx, "/bN.png", true)?
        );
        imgs.insert(
            "bB".to_string(), 
            graphics::Image::from_path(ctx, "/bB.png", true)?
        );
        imgs.insert(
            "bR".to_string(), 
            graphics::Image::from_path(ctx, "/BR.png", true)?
        );
        imgs.insert(
            "bQ".to_string(), 
            graphics::Image::from_path(ctx, "/BQ.png", true)?
        );
        imgs.insert(
            "bK".to_string(), 
            graphics::Image::from_path(ctx, "/BK.png", true)?
        );
        let chess_board = Arc::new(Mutex::new(chess::ChessBoard::new()));
        let netstuff = Self::start_networking(chess_board.clone());
        let tcp_stream = netstuff.0;
        let is_server = netstuff.1;
        
        Ok(MainState { pos_x: 0.0, 
            circle, 
            squares : squares, 
            square_size : scale, 
            img_map :imgs, 
            chess_board,
            selected_piece : None, 
            tcp_stream,
            is_server
        })
    }
    fn start_networking(chess_board : Arc<Mutex<chess::ChessBoard>>) -> (Arc<Mutex<Option<TcpStream>>>, bool) {
        let args : Vec<String> = env::args().collect();
        let is_server = args.len() <= 1;
        println!("{:?}", args);
        let tcp_stream = match is_server {
            true => Arc::new(Mutex::new(Some(net::host()))),
            false => Arc::new(Mutex::new(Some(net::connect())))
    
        }; 
        let tcp_stream_for_recv_thread = tcp_stream.clone();
        let board_for_recv_thread = chess_board.clone();
        thread::spawn(move || {
            net::recv(&tcp_stream_for_recv_thread, board_for_recv_thread, is_server);
        });
        (tcp_stream, is_server)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let m_down = ggez::input::mouse::button_pressed(&_ctx, ggez::input::mouse::MouseButton::Left);
        // the player selects a new square
        // todo: only fire on 1st m_down
        if self.selected_piece.is_none() && m_down {
            let m_pos = ggez::input::mouse::position(&_ctx);
            let chosen = chess::Square { row: (8.0 * (1.0 - (m_pos.y / SCREEN_SIZE.0))) as i32,  column :  ((8.0 * m_pos.x / SCREEN_SIZE.0))  as i32 };
            println!("chosen.y {} chosen.x {}", (m_pos.y * 8.0 / SCREEN_SIZE.0) as i32, ((8.0 * m_pos.x / SCREEN_SIZE.0))  as i32 );
            self.selected_piece = Some(chosen);
            println!("selp: x{} y{}", self.selected_piece.unwrap().column, self.selected_piece.unwrap().row)
        }
        else if !self.selected_piece.is_none() && !m_down {
            let m_pos = ggez::input::mouse::position(&_ctx);
            let chosen = chess::Square { row: (8.0 * (1.0 - (m_pos.y / SCREEN_SIZE.0))) as i32,  column :  ((8.0 * m_pos.x / SCREEN_SIZE.0))  as i32 };
            let from_square = self.selected_piece.unwrap();
            let to_square = chosen;
            let mv = chess::Move::new(from_square, to_square);
            let mut chess_board = self.chess_board.lock().unwrap();
            if(chess_board.get_legal_moves_from_square(from_square).contains(&mv) && from_square != to_square) {
                chess_board.make_move(mv, true);
                net::send_move(self.tcp_stream.clone(), mv, self.is_server);
                println!("sent move!");
            }
            self.selected_piece = None;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            graphics::CanvasLoadOp::Clear([0.1, 0.0, 0.3, 1.0].into()),
        );

        // canvas.draw(&self.circle, Point2 {x:  self.pos_x,y: 380.0 });
        for i in 0..64 {
            let offset_x = (i % 8) as f32;
            let offset_y = ((i as i32 / 8)) as f32;
            // this will generate a checkerboard pattern of 0s and 1s, and our "squares" arr has 
            // 2 elements, one white and one black square
            let color_idx = ((i % 2) + ((i / 8) % 2)) % 2;
            canvas.draw(&self.squares[color_idx as usize], Point2 {x :  (offset_x * self.square_size), y : offset_y * self.square_size});
        }
        
        let mut chess_board = self.chess_board.lock().unwrap();

        for i in 0..64 {
            let offset_x = (i % 8) as f32;
            let offset_y = ((i as i32 / 8)) as f32;
            let square : Square = chess::Square::new(7 - (i / 8), i % 8);
            match (chess_board.get_square_piece(square)) {
                chess::Piece::None => {},
                _ => {
                    let dst;
                    
                    if self.selected_piece.is_some() && self.selected_piece.unwrap() == square {

                        let m_pos = ggez::input::mouse::position(&ctx); 
                        dst = Point2 {x : -(self.square_size / 2.0) + m_pos.x, y : -(self.square_size / 2.0) + m_pos.y};
                        // println!("square: x{} y{}", self.selected_piece.unwrap().column, self.selected_piece.unwrap().row);
                    } 
                    else {
                        dst = Point2 {x : offset_x * self.square_size, y : offset_y * self.square_size};
                    }
                    
                    let drawparam = graphics::DrawParam::new()
                        .dest(dst)
                        .scale(Point2 {x : 0.35, y : 0.35});


                    let img_key = piece_to_string(chess_board.get_square_piece(square), chess_board.get_square_color(square));
                    canvas.draw(&self.img_map[&img_key], drawparam);
                }
            } 
        }
        canvas.finish(ctx)?;
    
        Ok(())
    }
}

pub fn main() -> GameResult {
    env::set_var("RUST_BACKTRACE", "1");
    let cb = ggez::ContextBuilder::new("chess", "dstrombe/castorm")
        .window_setup(ggez::conf::WindowSetup::default().title("chess 😳"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1));
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;

 

    event::run(ctx, event_loop, state)
}
