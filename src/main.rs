#![allow(unused_parens)]

use std::{self, io::Write};
use rand::Rng;
use getch::Getch;
use core::cmp::max;

#[derive(PartialEq)]
enum MineState {
	Covered,
	Uncovered,
	Flagged,
}

struct Square {
	state: MineState,
	is_mine: bool,
	neighbour_cache: Option<u64>,
}

struct Board {
	width: usize,
	height: usize,
	board: Vec<Square>,
	selected_square: (usize, usize),
	highlight_square: Option<(usize, usize)>,
	uncovered_squares: usize,
	alive: bool,
	started: bool,
	won: bool,
}
impl Board {
	fn get_mut(&mut self, x:usize, y:usize) -> &mut Square { &mut self.board[y*self.width+x] }
	fn get(&self, x:usize, y:usize) -> &Square { &self.board[y*self.width+x] }
	fn neighbours(&mut self, x:usize, y:usize) -> u64 {
		match self.get(x,y).neighbour_cache {
			Some(v)=>v,
			None => {
				let mut neighbours = 0;
				self.itterate_neighbours(x,y,false,|slf, x,y| {
					if slf.get(x,y).is_mine { neighbours+=1; }
				});
				self.get_mut(x,y).neighbour_cache = Some(neighbours);
				return neighbours;
			}
		}
	}
	fn uncover_all(&mut self) {
		for y in 0..self.height {
			for x in 0..self.width {
				self.get_mut(x,y).state = match self.get(x,y).state {
					MineState::Covered => MineState::Uncovered,
					MineState::Uncovered => MineState::Uncovered,
					MineState::Flagged if !self.get(x,y).is_mine => MineState::Uncovered,
					MineState::Flagged => MineState::Flagged,
				}
			}
		}
	}
	fn uncover(&mut self, x:usize, y:usize) {
		if self.get(x,y).state == MineState::Flagged
		{
			self.get_mut(x,y).state = MineState::Covered;
			return
		}

		if !self.started && self.get(x, y).is_mine {
			self.get_mut(x,y).is_mine = false;
			self.uncovered_squares -= 1;
		}
		self.started = true;
		match self.get(x, y).state {
			MineState::Uncovered => return,
			MineState::Covered | MineState::Flagged => {
				self.get_mut(x,y).state = MineState::Uncovered;
				self.uncovered_squares += 1;
				if self.get(x, y).is_mine {
					self.alive = false;
					self.uncover_all()
				}
				if self.uncovered_squares == self.width*self.height {self.won=true;}
				if self.neighbours(x,y) == 0 {
					self.itterate_neighbours(x,y,false,|slf,x,y| slf.uncover(x,y));
				}
			},
		}
	}
	fn itterate_neighbours<F: FnMut(&mut Board,usize,usize)>(&mut self,x:usize,y:usize,include_self:bool,mut f:F) {
		for x_off in (-1 as isize)..=1 {
			for y_off in (-1 as isize)..=1 {
				if ((!include_self) && x_off==0 && y_off==0) || (
					(x as isize)+x_off < 0 ||
					((x as isize)+x_off) as usize >= self.width ||
					(y as isize)+y_off < 0 ||
					((y as isize)+y_off) as usize >= self.height
				) { continue; }
				f(
					self,
					((x as isize)+x_off) as usize,
					((y as isize)+y_off) as usize
				);
			}
		}
	}
}

const MINE_CHANCE:f32 = 0.2;

fn display_board(board: &mut Board){
	let mut highlighting = false;
	let mut highlight_min_x: isize = 0;
	let mut highlight_min_y: isize = 0;
	match board.highlight_square {
		Some(highlight) => {
			highlighting = true;
			highlight_min_x = highlight.0 as isize-1;
			highlight_min_y = highlight.1 as isize-1;
		},
		None => {},
	}
	board.highlight_square = None;
	let highlight_max_x = highlight_min_x+2;
	let highlight_max_y = highlight_min_y+2;

	highlight_min_x = max(0,highlight_min_x);
	highlight_min_y = max(0,highlight_min_y);

	for y in 0..board.height {
		for x in 0..board.width {
			if (
				highlighting &&
				x>=highlight_min_x as usize &&
				x<=highlight_max_x as usize + 1 &&
				y>=highlight_min_y as usize &&
				y<=highlight_max_y as usize
			) {std::io::stdout().write_all(b"\x1B[46m").map_err(|err| println!("{:?}", err)).ok();}

			std::io::stdout().write_all(match board.selected_square {
				(a,b) if a==x && b==y => b"[",
				(a,b) if x>0 && a==x-1 && b==y => b"]",
				(_,_) => b" ",
			}).map_err(|err| println!("{:?}", err)).ok();

			if (
				highlighting &&
				x>=highlight_min_x as usize &&
				x==highlight_max_x as usize + 1 &&
				y>=highlight_min_y as usize &&
				y<=highlight_max_y as usize
			) {std::io::stdout().write_all(b"\x1B[49m").map_err(|err| println!("{:?}", err)).ok();}


			let sqr = board.get(x,y);
			std::io::stdout().write_all(
				match sqr.state {
					MineState::Covered => b".",
					MineState::Uncovered => match sqr.is_mine {
						false => match board.neighbours(x,y) {
							0 => " ",
							1 => "\x1B[34m1\x1B[39m",
							2 => "\x1B[34m2\x1B[39m",
							3 => "\x1B[32m3\x1B[39m",
							4 => "\x1B[32m4\x1B[39m",
							5 => "\x1B[33m5\x1B[39m",
							6 => "\x1B[33m6\x1B[39m",
							7 => "\x1B[31m7\x1B[39m",
							8 => "\x1B[31m8\x1B[39m",
							_ => panic!("ERROR INVALID NEIGHBOUR COUNT"),
						}.as_bytes(),
						true => b"*",
					},
					MineState::Flagged => b"F",
				}
			).map_err(|err| println!("{:?}", err)).ok();

			std::io::stdout().write_all(b"\x1B[49m").map_err(|err| println!("{:?}", err)).ok();
		}
		if (
			highlighting &&
			highlight_max_x as usize==board.width &&
			y>=highlight_min_y as usize &&
			y<=highlight_max_y as usize
		) {std::io::stdout().write_all(b"\x1B[46m").map_err(|err| println!("{:?}", err)).ok();}

		if board.selected_square == (board.width-1,y)
		{std::io::stdout().write_all(b"]").map_err(|err| println!("{:?}", err)).ok();}
		else{std::io::stdout().write_all(b" ").map_err(|err| println!("{:?}", err)).ok();}

		if (
			highlighting &&
			y>=highlight_min_y as usize &&
			y<=highlight_max_y as usize
		) {std::io::stdout().write_all(b"\x1B[49m").map_err(|err| println!("{:?}", err)).ok();}
		
		std::io::stdout().write_all(b"\n").map_err(|err| println!("{:?}", err)).ok();
	}
}


fn main(){
	let width = 20;
	let height = 20;
	let mut board = Board {
		width: width,
		height: height,
		board: (0..=width*height).map(|_| Square {
			state: MineState::Covered,
			is_mine: rand::thread_rng().gen_range(1..=100) < ((MINE_CHANCE*100.0) as u64),
			neighbour_cache: None,
		}).collect(),
		selected_square: (0,0),
		highlight_square: None,
		uncovered_squares: 0,
		alive: true,
		started: false,
		won: false,
	};
	//say that all mines are uncovered then it is easier to detect a win
	board.uncovered_squares = board.board.iter().filter(|&sqr| sqr.is_mine).count();
	let getch = Getch::new();
	std::io::stdout().write_all("\n".repeat(board.height+1).as_bytes()).map_err(|err| println!("{:?}", err)).ok();
	// std::io::stdout().write_all(b"\x1B[s").map_err(|err| println!("{:?}", err)).ok();
	loop {
		std::io::stdout().write_all(("\x1B[".to_owned()+&(board.height).to_string()+"A").as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		// println!("{0},{1}",board.selected_square.0,board.selected_square.1);
		// std::io::stdout().write_all("\x1BM".repeat(board.height).as_bytes()).map_err(|err| println!("{:?}", err)).ok();
		display_board(&mut board);
		if !board.alive {
			std::io::stdout().write_all(b"YOU FAILED").map_err(|err| println!("{:?}", err)).ok();
			return;
		}
		if board.won {
			std::io::stdout().write_all(b"YOU WON! WELL DONE").map_err(|err| println!("{:?}", err)).ok();
			return;
		}
		match getch.getch() {
			Ok(3) => return,
			Ok(77)|Ok(100) if board.selected_square.0 < board.width - 1 => board.selected_square.0+=1,
			Ok(75)|Ok(97) if board.selected_square.0 > 0 => board.selected_square.0-=1,
			Ok(80)|Ok(115) if board.selected_square.1 < board.height - 1 => board.selected_square.1+=1,
			Ok(72)|Ok(119) if board.selected_square.1 > 0 => board.selected_square.1-=1,
			Ok(32)|Ok(13) => board.uncover(board.selected_square.0,board.selected_square.1),
			Ok(102) => board.get_mut(
				board.selected_square.0,
				board.selected_square.1
			).state = match board.get(
				board.selected_square.0,
				board.selected_square.1
			).state{
				MineState::Covered => MineState::Flagged,
				MineState::Uncovered => MineState::Uncovered,
				MineState::Flagged => MineState::Covered,
			},
			Ok(104) => board.highlight_square = Some(board.selected_square),
			// Ok(a) => println!("{a}"),
			Ok(_)|Err(_) => continue,
		};
	}
}
