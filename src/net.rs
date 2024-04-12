use crate::networking;
use crate::networking::*;
use std::env;
use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use prost::Message;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chess::*;

pub fn connect() -> TcpStream {
    let args : Vec<String> = env::args().collect();
    let mut stream = TcpStream::connect(args[1].clone()).unwrap();
    
    // send the connection request
    let c2s_conn_pkt = C2sMessage {
        msg : Some(c2s_message::Msg::ConnectRequest(C2sConnectRequest {
            game_id : 41414141,
            spectate : false
        }))
    }; 
    
    let mut w_buf : Vec<u8> = Vec::new();
    c2s_conn_pkt.encode(&mut w_buf).unwrap();
    stream.write(&w_buf);

    // read the response for the connection request
    //let mut r_buf : [u8; 512]= [0_u8; 512];
    //let response_len = stream.read(&mut r_buf).unwrap();
    //let s2c_msg_pkt = S2cMessage::decode(&r_buf[..]).unwrap();
    
    //println!("{} {}", response_len, s2c_msg_pkt.msg.is_some());

    
    stream.set_nonblocking(true);
    stream
    // send C2sConnectRequest 
}

pub fn fen_parse(fen : String) -> chess::ChessBoard {
    ChessBoard::new()
}

pub fn host() -> TcpStream {
    let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
    let stream =  (listener.incoming().next().unwrap().unwrap());
    stream.set_nonblocking(true);
    stream
}

pub fn recv(tcp_stream : &Arc<Mutex<Option<TcpStream>>>, chess_board : Arc<Mutex<chess::ChessBoard>>, is_server : bool) { 
    loop { 
        let mut mg = (tcp_stream.lock().unwrap());
        let mut stream = mg.as_mut().unwrap();
        
        // read the incoming packet
        let mut r_buf : [u8; 512] = [0_u8; 512];
        //println!("ping");
        let buf_len = match(stream.read(&mut r_buf)) {
            Ok(len) => {println!("nonzero packet! {}", len); len},
            Err(e) => {
                //println!("{}", e); 
                continue; 0
            }
        };
        if is_server {
            let c2s_msg_pkt = C2sMessage::decode(&r_buf[0..buf_len]).unwrap();
            recv_server(&mut stream, chess_board.clone(), c2s_msg_pkt);
        }
        else {
            let s2c_msg_pkt = S2cMessage::decode(&r_buf[0..buf_len]).unwrap();
            recv_client(chess_board.clone(), s2c_msg_pkt);
        }
        
        println!("server recv loop exit");
        thread::sleep(Duration::from_millis(30));
    }
}
pub fn recv_server (stream : &mut TcpStream, chess_board : Arc<Mutex<chess::ChessBoard>>, c2s_msg_pkt : C2sMessage) {
    
    match c2s_msg_pkt.msg {
        Some(msg) => {
            match msg {
               
                c2s_message::Msg::ConnectRequest(c2s_conn_req) => {
                    println!("conn req");
                    let mut s2c_msg = S2cMessage {
                        msg : Some(s2c_message::Msg::ConnectAck(networking::S2cConnectAck {
                            success : true,
                            starting_position : Some(BoardState { fen_string : "fen. lol!".to_owned() }),
                            game_id : Some(31337),
                            client_is_white : Some(true),
                        }))
                    };
                    
                    stream.write(&s2c_msg.encode_to_vec());
                }
                c2s_message::Msg::Move(mv) => {
                    recv_move(mv, chess_board.clone());
                }
                _ => {
                   println!("unknown c2s received");
                }
            }
        }
        None => {} 
    }
}

pub fn recv_client (chess_board : Arc<Mutex<chess::ChessBoard>>, s2c_msg_pkt : S2cMessage) {
    match s2c_msg_pkt.msg {
        Some(msg) => {
            match msg {
                s2c_message::Msg::ConnectAck(s2c_conn_ack) => {
                    if !s2c_conn_ack.success {
                        panic!("connection rejected");
                    }
                    else {
                        println!("connection accepted!");
                        let mut board = chess_board.lock().unwrap();
                        *board = fen_parse(s2c_conn_ack.starting_position.unwrap().fen_string);
                    }
                }
                s2c_message::Msg::Move(mv) => {
                    recv_move(mv, chess_board.clone());
                }
                _ => { println!("unknown s2c packet received"); }
            }       
        }
        None => {
            //panic!("invalid s2c_message. Dump: {:?}", r_buf);
        }
    }   
}

pub fn recv_move (mv : networking::Move, chess_board : Arc<Mutex<chess::ChessBoard>>) {
    let from_square = chess::Square::new((mv.from_square / 8).try_into().unwrap(), (mv.from_square % 8).try_into().unwrap());
    let to_square = chess::Square::new((mv.to_square / 8).try_into().unwrap(), (mv.to_square % 8).try_into().unwrap());
    println!("from, to = {},{} {},{}", from_square.column, from_square.row, to_square.column, to_square.row);
    let chess_move = chess::Move::new(from_square, to_square);
    let mut board = chess_board.lock().unwrap();
    board.make_move(chess_move, true);

}

pub fn send_move (tcp_stream : Arc<Mutex<Option<TcpStream>>>, mv : chess::Move, is_server : bool) {
    let mg = tcp_stream.lock().unwrap();
    let mut stream = mg.as_ref().unwrap();

    // send the connection request
    let mv = networking::Move {
            from_square : mv.from.column as u32 + (mv.from.row * 8) as u32,
            to_square : mv.to.column as u32 + (mv.to.row * 8) as u32,
            promotion : None,
    };

    let s2c_msg = S2cMessage {
        msg : Some(s2c_message::Msg::Move(mv.clone()))
    };
    let c2s_msg = C2sMessage {
        msg : Some(c2s_message::Msg::Move(mv.clone()))
    }; 

    let mut w_buf : Vec<u8> = match (is_server) {
        true => {
            s2c_msg.encode_to_vec()
        }
        false => {
            c2s_msg.encode_to_vec()
        }
    };
    stream.write(&w_buf);


}


