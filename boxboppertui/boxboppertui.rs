// Box Bopper: Sokoban clone in rust
// Copyright David Atkinson 2020-2021
//
// boxboppertui.rs: console game player
// TODO: Lots of TODOs below!
// TODO: Add highscore
// TODO: Make walls brown

use std::io;
use std::io::{BufRead,Write};

use std::time::Duration;

use crossterm::{terminal, event};
use crossterm::event::{Event, KeyCode};

use boxbopperbase::{Game};
use boxbopperbase::vector::{Move};
use boxbopperbase::level::{Level};
use boxbopperbase::builtins::BUILTIN_LEVELS;

use tui::Terminal;
use tui::backend::{CrosstermBackend};
use tui::widgets::{Block, Borders, Paragraph, BorderType};
use tui::layout::{Layout, Constraint, Direction, Alignment};
use tui::style::{Color, Style};
use tui::text::{Span,Spans};

//■□▣░▒▓█☐☒☓◦⬝⬞⁅⁆※ↀ⊏⊐⊗⊞⊠⊡╳⬚
// ✅❎❌⏹⬛⬜
// ♒♊🔘🔲🔳🔴🔵📀💿🟠🟡🟢🟣🟤🟥🟦🟧🟨🟩🟪🟫🧿🧍👷🙂🙃😀😃😄🤔🗿
// We use str here (instead of char) because unicode characters can be multi-width or multi-code
// Wall, Space, Boxx, Hole, Human, HumanInHole, BoxxInHole
const TEXT_OBJS: [[&str; 7]; 2] = [ ["#", " ", "*", "O", "&", "%", "@" ],
									["░░", "  ", "❎", "🔳", "😀", "🤔", "✅"] ];

pub fn basic_ui_get_user_input() -> String {
	let mut line = String::new();
	let stdin = io::stdin();
	return loop {
		stdin.lock().read_line(&mut line).unwrap();
		if line.len() > 0 { break line; }
	}
}

pub fn basic_ui_display_game(game: &Game, use_emoji: bool) {
	println!("------------------------------------------------------------------------------");
	println!("{} moves: {}", game.get_num_moves(), game.get_moves_string());
	println!("------------------------------------------------------------------------------");
	println!();
	println!("{}", get_level_string(game, use_emoji));
	println!();
}

fn get_level_string(game: &Game, use_emoji: bool) -> String {
	let base_str = game.get_level_string();
	
	if !use_emoji {
		String::from(base_str)
	} else {
		level_str_to_emoji_str(&base_str)
	}
}

fn level_str_to_emoji_str(base_str: &String) -> String {
	let mut alt_str: String = String::from("");
	for c in base_str.chars() {
		let cs = String::from(c);
		let alt = match c {
			'#' => TEXT_OBJS[0][0],
			' ' => TEXT_OBJS[0][1],
			'*' => TEXT_OBJS[0][2],
			'O' => TEXT_OBJS[0][3],
			'&' => TEXT_OBJS[0][4],
			'%' => TEXT_OBJS[0][5],
			'@' => TEXT_OBJS[0][6],
			_   => &cs,
		};	
		alt_str += alt;
	}
	alt_str
}

fn level_str_to_vecs(base_str: &String, use_emoji: bool) -> Vec<Spans> {
	let mut vecs: Vec<Spans> = vec![];
	let mut line: Vec<Span> = vec![];
	let ue = use_emoji as usize;
	for c in base_str.chars() {
		match c {
			'#' => line.push(Span::styled(TEXT_OBJS[ue][0], Style::default().fg(Color::Red))),
			' ' => line.push(Span::styled(TEXT_OBJS[ue][1], Style::default())),
			'*' => line.push(Span::styled(TEXT_OBJS[ue][2], Style::default().fg(Color::Green))),
			'O' => line.push(Span::styled(TEXT_OBJS[ue][3], Style::default())),
			'&' => line.push(Span::styled(TEXT_OBJS[ue][4], Style::default().fg(Color::LightYellow))),
			'%' => line.push(Span::styled(TEXT_OBJS[ue][5], Style::default().fg(Color::LightYellow))),
			'@' => line.push(Span::styled(TEXT_OBJS[ue][6], Style::default().fg(Color::LightGreen))),
			//'\n' | '\r' => { vecs.push(Spans::from(line.clone())); line.clear(); },
			_   => { vecs.push(Spans::from(line.clone())); line.clear(); },
		};
	}
	vecs.push(Spans::from(line.clone()));
	return vecs.iter().map(|v| { Spans::from(v.clone()) } ).collect();
}

// OK will return bool (true=keep going), Err will return string
fn tui_inner(state: &mut Game, current_level: &mut u32, use_emoji: bool) -> Result <bool, String> {
	let stdout = io::stdout();
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = match Terminal::new(backend) {
		Ok(t) => t,
		Err(_) => return Err("Failed to open terminal.".to_string()),
	};

	let r = terminal.draw(|rect| {
		let size = rect.size();
		let chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(2)
			.constraints(
				[
					Constraint::Length(3),
					Constraint::Min(10),
					Constraint::Length(3),
				]
				.as_ref(),
			).split(size);
		
		let current_level_str = format!("{:2}", current_level);
		let num_moves_str = format!("{:3}", state.get_num_moves());

		// TODO: Format get_moves_string() so that it shows the most recent moves,
		// or goes multiline, 
		// when there are too many to display on screen

		let level_text = vec![ Spans::from(vec![
				Span::raw("Level: "),
				Span::styled(current_level_str, Style::default().fg(Color::LightMagenta)),
				Span::raw(" Moves: "),
				Span::styled(num_moves_str, Style::default().fg(Color::LightMagenta)),
				Span::raw(" "),
				Span::styled(state.get_moves_string(), Style::default().fg(Color::Blue)),
			]) ];
		
		let top_widget = Paragraph::new(level_text)
			.style(Style::default().fg(Color::LightCyan))
			.block(
				Block::default()
					.borders(Borders::ALL)
					.style(Style::default().fg(Color::White))
					.title("BoxBopper by David Atkinson")
					.border_type(BorderType::Plain),
			);

		rect.render_widget(top_widget, chunks[0]);

		// TODO: Highlight the first letter of the commands

		let mut menu_text = String::from("Quit   `reset   Next level   Previous level   ");
		if !state.have_win_condition() {
			let opts = &state.get_move_options();
			for x in opts {
				let xtra = match x.to_string().as_ref() {
					"U" => "Up   ",
					"D" => "Down   ",
					"L" => "Left   ",
					"R" => "Right   ",
					_   => "",
				};
				menu_text += xtra;
			}
		}

		let menu_col: Color;
		let menu_title: &str;
		if state.have_win_condition() {
			menu_col = Color::LightGreen;
			menu_title = "Level has been completed!";
		} else {
			menu_col = Color::White;
			menu_title = "Commands"
		}

		let menu_widget = Paragraph::new(menu_text)
			.style(Style::default().fg(Color::LightCyan))
			.alignment(Alignment::Left)
			.block(
				Block::default()
					.borders(Borders::ALL)
					.style(Style::default().fg(menu_col))
					.title(menu_title)
					.border_type(BorderType::Plain),
			);

		rect.render_widget(menu_widget, chunks[2]);

		let base_str = state.get_level_string();
		let game_text_vecs = level_str_to_vecs(&base_str, use_emoji);
		let game_widget = Paragraph::new(game_text_vecs) 
			.alignment(Alignment::Center)
			.block(
				Block::default().style(Style::default().fg(Color::White)),
			);		

		rect.render_widget(game_widget, chunks[1]);

	});
	
	if r.is_err() {
		return Err("terminal.draw() failed.".to_string());
	}

	// we don't do anything without user input (an event), so we can wait a long time
	let r = event::poll(Duration::from_millis(1000));
	if r.is_err() {
		return Err("Failed to poll() temrinal.".to_string());
	}
	// let r = r.unwrap(); // we can find out if an event occured or it was just a timeout

	let r = event::read();
	if r.is_err() {
		return Err("Failed to read() terminal.".to_string())
	}
	match r.unwrap() {
		Event::Key(ev) => match ev.code {
			KeyCode::Char('Q') | KeyCode::Char('q') | KeyCode::Esc => { return Ok(false); },
			KeyCode::Char('`')                       => { *state = Game::new(*current_level); },
			KeyCode::Char('N') | KeyCode::Char('n')  => { 
				if *current_level < BUILTIN_LEVELS.len() as u32 - 1 {
					*current_level += 1;
					*state = Game::new(*current_level);
				}},
			KeyCode::Char('P') | KeyCode::Char('p')  => { 
				if *current_level > 0 {
					*current_level -= 1;
					*state = Game::new(*current_level);
				}},
			KeyCode::Char('U') | KeyCode::Char('u') | KeyCode::Up    => state.append_move(&Move::Up),
			KeyCode::Char('R') | KeyCode::Char('r') | KeyCode::Right => state.append_move(&Move::Right),
			KeyCode::Char('D') | KeyCode::Char('d') | KeyCode::Down  => state.append_move(&Move::Down),
			KeyCode::Char('L') | KeyCode::Char('l') | KeyCode::Left  => state.append_move(&Move::Left),
			_ => {}
		},
		Event::Mouse(_event) => {},
		Event::Resize(_width, _height) => {},
	}

	return Ok(true); // keep going
}


pub fn basic_ui_inner(state: &mut Game, current_level: &mut u32, use_emoji: bool) -> Result<bool, String> {
	println!("\n\n");
	println!("==============================================================================");			
	println!("Level {}", *current_level);
	basic_ui_display_game(state, use_emoji);
	
	if state.have_win_condition() {
		println!(r"    \  /\  / | |\ |");
		println!(r"     \/  \/  | | \|");
		println!("\n");
	}
	
	if !state.have_win_condition() {
		print!("Commands (Quit `reset Next Prev Up Down Left Right) > ");
	} else {
		print!("Level has beel completed! (Quit `reset Next Prev) > ");
	}

	let r = io::stdout().flush();
	if r.is_err() {
		return Err("Failed to flush stdout.".to_string());
	}
	
	let mut quit = false;

	// this function blocks, waiting for user input (it is meant to)
	basic_ui_get_user_input().chars().for_each( |c| match c {
		'q' | 'Q' => quit = true,
		'`' => *state = Game::new(*current_level),
		'n' | 'N' =>  { if *current_level < BUILTIN_LEVELS.len() as u32 {
					*current_level += 1;
					*state = Game::new(*current_level);
				}},
		'p' | 'P' => { if *current_level > 0 {
					*current_level -= 1;
					*state = Game::new(*current_level);
				}},
		'u' | 'U' => state.append_move(&Move::Up),
		'r' | 'R' => state.append_move(&Move::Right),
		'd' | 'D' => state.append_move(&Move::Down),
		'l' | 'L' => state.append_move(&Move::Left),
		_ => {}
	});
	Ok(!quit)
}

fn main() -> Result<(), String> {
	let args: Vec::<String> = std::env::args().collect();
	let mut filename: String = String::from("");
	let mut builtin: u32 = 0;
	let mut use_emoji: bool = false;
	let mut basic_ui: bool = false;
	let mut quit = false;
	let mut show_help = false;

	// process params
	for (count,arg) in args.into_iter().enumerate() {
		if count >= 1 {
			let eq_idx = arg.find('=');
			if eq_idx.is_none() {
				println!("No equals symbol found in var");
				show_help = true;
				continue;
			}
			let eq_idx = eq_idx.unwrap();
			let left = &arg[0..eq_idx];
			let right = &arg[eq_idx+1..];
			match left {
				"filename"  => { filename = String::from(right); },
				"builtin"   => { builtin = right.parse::<u32>().unwrap(); }
				"use_emoji"   => { use_emoji = right.parse::<bool>().unwrap(); },
				"basic_ui"   => { basic_ui = right.parse::<bool>().unwrap(); },
				_ => {
					println!("Unrecognised variable {}", left);
					show_help = true;
				}
			}
		}
	}

	if show_help || basic_ui {
		println!("BoxBopper Copyright 2020-2021 David Atkinson");
	}

	if show_help {
		println!("filename=FILENAME      load level from FILENAME");
		println!("builtin=NUM            start with builtin level NUM       0-78");
		println!("use_emoji=true         use emoji for display              true / false");
		println!("basic_ui=true          use a basic ui only                true / false");
	}
	
	let mut state = if filename.len() > 0 {
		Game::new_from_level(&Level::from_file(&filename).expect("Unable to open specified file"), 0)
	} else {
		Game::new(builtin)
	};
		
	let mut current_level: u32 = builtin;
	
	if quit || show_help {
		return Ok(());
	}

	if !basic_ui {
		let backend = CrosstermBackend::new(io::stdout());
		let mut terminal = Terminal::new(backend).expect("Failed to open terminal.");
		terminal::enable_raw_mode().expect("Failed to enable terminal raw mode.");
		let _ = terminal.clear(); // don't care if this fails at this stage
	}

	let mut error_string = "".to_string();

	while !quit {
		// process move queue
		while state.is_queued_moves() {
			state.process_moves();
		}

		// run display/input function
		let r = if basic_ui {
			// run basic ui
			basic_ui_inner(&mut state, &mut current_level, use_emoji)
		} else {
			// run tui
			tui_inner(&mut state, &mut current_level, use_emoji)
		};

		// are we quitting?
		match r {
			Err(s) => { // quit with error message
				error_string += &s;
				quit = true;
			},
			Ok(false) => { // we got asked to quit by user
				quit = true;
			},
			Ok(true) => {} // continue
		};
	}

	if !basic_ui {
		let backend = CrosstermBackend::new(io::stdout());
		let mut terminal = Terminal::new(backend).expect("Failed to open terminal.");
		let _ = terminal::disable_raw_mode(); 		// we don't care if any of these return errors, as we are quitting
		let _ = terminal.clear();
		let _ = terminal.show_cursor();
		let _ = terminal.set_cursor(0,0);
	}

	println!("{}", error_string);

	return Ok(());
}

