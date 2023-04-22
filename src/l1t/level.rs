use crate::{direction::Direction, node::*};
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use std::fs;
use std::io::Stdout;

pub struct Level {
    pub name: String,
    pub author: String,
    pub description: String,
    pub nodes: Vec<Node>,
    pub rows: u16,
    pub cols: u16,
    pub term_rows: u16,
    pub term_cols: u16,
    pub row_offset: u16,
    pub col_offset: u16,
}

impl Level {
    fn set_lasers_looking_at(&mut self) {
        for i in 0..self.nodes.len() {
            self.nodes[i].looking_at.clear();
            let node = &self.nodes[i];
            if !matches!(node.node_type, NodeType::Laser(_)) {
                continue;
            }
            let mut current_row: i16 = node.row as i16;
            let mut current_col: i16 = node.col as i16;
            let mut current_dir: Direction = node.dir;
            loop {
                if current_row + current_dir.0 < 1
                    || current_col + current_dir.1 < 1
                    || current_row + current_dir.0 >= self.rows as i16 - 1
                    || current_col + current_dir.1 >= self.cols as i16 - 1
                {
                    break;
                }
                current_row += current_dir.0;
                current_col += current_dir.1;
                let ch: char = match current_dir {
                    Direction::UP | Direction::DOWN => '|',
                    _ => '-',
                };
                let arrow_ch: char = match current_dir {
                    Direction::UP => '^',
                    Direction::DOWN => 'v',
                    Direction::LEFT => '<',
                    _ => '>',
                };
                self.nodes[i].looking_at.push((
                    current_row as u16,
                    current_col as u16,
                    ch,
                    arrow_ch,
                ));
                if let Some(j) = self.get_node_by_position(current_row as u16, current_col as u16) {
                    let node_at_pos = &self.nodes[j];
                    if !matches!(node_at_pos.node_type, NodeType::Mirror(_)) {
                        break;
                    }
                    // Determines what the new direction is after encountering a mirror
                    if current_dir.0 == 0 {
                        current_dir.0 = current_dir.1.abs();
                        if current_dir.1 == (node_at_pos.dir.0 + node_at_pos.dir.1) {
                            current_dir.0 = -current_dir.0
                        }
                        current_dir.1 = 0;
                    } else {
                        current_dir.1 = current_dir.0.abs();
                        if current_dir.0 == (node_at_pos.dir.0 + node_at_pos.dir.1) {
                            current_dir.1 = -current_dir.1
                        }
                        current_dir.0 = 0;
                    }
                }
            }
        }
    }

    fn get_player(&self) -> Option<usize> {
        self.nodes
            .iter()
            .position(|n| matches!(n.node_type, NodeType::Player))
    }

    fn set_player_position(&mut self, row: u16, col: u16) {
        if row == 0 || row >= self.rows - 1 || col == 0 || col >= self.cols - 1 {
            return;
        }
        match self.get_player() {
            Some(i) => {
                self.nodes[i].row = row;
                self.nodes[i].col = col;
            }
            None => self.nodes.push(Node::new(NodeType::Player, row, col)),
        }
    }

    fn get_node_by_position(&self, row: u16, col: u16) -> Option<usize> {
        match self.nodes.iter().position(|n| n.row == row && n.col == col) {
            Some(i) => Some(i),
            None => None,
        }
    }

    fn draw_walls(&self, stdout: &mut Stdout) {
        for r in self.row_offset..(self.rows + self.row_offset) {
            for c in self.col_offset..(self.cols + self.col_offset) {
                if r == self.row_offset
                    || r == self.rows + self.row_offset - 1
                    || c == self.col_offset
                    || c == self.cols + self.col_offset - 1
                {
                    execute!(
                        stdout,
                        SetForegroundColor(Color::White),
                        SetBackgroundColor(Color::White),
                        cursor::MoveTo(c, r),
                        Print('I'),
                        ResetColor
                    )
                    .ok();
                }
            }
        }
    }

    fn draw_nodes(&self, stdout: &mut Stdout) {
        for (_, n) in self.nodes.iter().enumerate() {
            if matches!(n.node_type, NodeType::Player) {
                continue;
            }
            if matches!(n.node_type, NodeType::ToggleBlock) && n.toggled {
                continue;
            }
            execute!(
                stdout,
                cursor::MoveTo(
                    n.col as u16 + self.col_offset,
                    n.row as u16 + self.row_offset
                ),
                SetForegroundColor(if n.toggled {
                    n.toggled_fg_color
                } else {
                    n.fg_color
                }),
                SetBackgroundColor(if n.toggled {
                    n.toggled_bg_color
                } else {
                    n.bg_color
                }),
                Print(n.ch.bold()),
                ResetColor
            )
            .ok();
        }
    }

    fn draw_player(&self, stdout: &mut Stdout) {
        let player_index = match self.get_player() {
            Some(i) => i,
            None => return,
        };
        let player = &self.nodes[player_index];
        execute!(
            stdout,
            cursor::MoveTo(
                player.col as u16 + self.col_offset,
                player.row as u16 + self.row_offset
            ),
            SetForegroundColor(player.fg_color),
            SetBackgroundColor(player.bg_color),
            Print(player.ch.bold()),
            ResetColor
        )
        .ok();
    }

    fn draw_laser_overlays(&self, stdout: &mut Stdout) {
        for (_, n) in self.nodes.iter().enumerate() {
            if !matches!(n.node_type, NodeType::Laser(_)) {
                continue;
            }
            for i in 0..n.looking_at.len() {
                let pos = &n.looking_at[i];
                execute!(
                    stdout,
                    SetForegroundColor(Color::Red),
                    cursor::MoveTo(pos.1 + self.col_offset, pos.0 + self.row_offset),
                    Print(if i == n.looking_at.len() - 1 {
                        pos.3
                    } else {
                        pos.2
                    }),
                    SetForegroundColor(Color::Reset),
                )
                .ok();
            }
        }
    }

    fn draw(&self, stdout: &mut Stdout) {
        stdout.execute(Clear(ClearType::All)).ok();
        self.draw_walls(stdout);
        self.draw_laser_overlays(stdout);
        self.draw_nodes(stdout);
        self.draw_player(stdout);
    }

    fn move_player(&mut self, dir: Direction) {
        let player_index = match self.get_player() {
            Some(i) => i,
            None => return,
        };
        let players_next_row = (self.nodes[player_index].row as i16 + dir.0) as u16;
        let players_next_col = (self.nodes[player_index].col as i16 + dir.1) as u16;
        if let Some(i) = self.get_node_by_position(players_next_row, players_next_col) {
            match self.nodes[i].node_type {
                NodeType::Block | NodeType::Mirror(_) | NodeType::Laser(_) => {
                    let blocks_next_row = (players_next_row as i16 + dir.0) as u16;
                    let blocks_next_col = (players_next_col as i16 + dir.1) as u16;
                    if let Some(_) = self.get_node_by_position(blocks_next_row, blocks_next_col) {
                        return;
                    }
                    if blocks_next_row < 1
                        || blocks_next_row >= self.rows - 1
                        || blocks_next_col < 1
                        || blocks_next_col >= self.cols - 1
                    {
                        return;
                    }
                    self.set_player_position(players_next_row, players_next_col);
                    self.nodes[i].row = blocks_next_row;
                    self.nodes[i].col = blocks_next_col;
                }
                NodeType::Switch => {
                    self.nodes[i].toggled = !self.nodes[i].toggled;
                    //self.toggle_blocks();
                }
                NodeType::ToggleBlock => {
                    if self.nodes[i].toggled {
                        self.set_player_position(players_next_row, players_next_col);
                    }
                }
                _ => (),
            }
        } else {
            self.set_player_position(players_next_row, players_next_col);
        }
    }

    pub fn new(filename: String, term_rows: u16, term_cols: u16) -> Result<Level, &'static str> {
        let file_content = fs::read_to_string(filename).unwrap_or("".to_string());
        if file_content.trim().len() == 0 {
            return Err("Empty level file.");
        }
        let lines: Vec<&str> = file_content.trim().split('\n').collect();
        let rows = lines.len() as u16;
        if rows < 6 {
            return Err("Level file must include a line for the `name`, `author`, `description`, and lines representing the level grid.");
        }
        let cols = lines[4].len() as u16;
        if cols < 3 {
            return Err("Level grid must be made up of at least one grid space and an even wall of `I` characters representing the walls.");
        }
        let name: String = lines[0].to_string();
        let author: String = lines[1].to_string();
        let description: String = lines[2].to_string();
        let mut nodes: Vec<Node> = vec![];
        for r in 3..rows {
            for (c, ch) in lines[r as usize].chars().enumerate() {
                if r == 3 || r == rows - 1 || c == 0 || c == cols as usize - 1 {
                    if ch != 'I' {
                        return Err("Level grid must be made up of at least one grid space and an even wall of `I` characters representing the walls.");
                    }
                    continue;
                }
                if let Some(n) = Node::parse(ch, r - 3, c as u16) {
                    nodes.push(n);
                }
            }
        }
        Ok(Level {
            name,
            author,
            description,
            nodes,
            rows: rows - 3,
            cols,
            term_rows,
            term_cols,
            row_offset: (term_rows - rows - 3) / 2,
            col_offset: (term_cols - cols) / 2,
        })
    }

    pub fn play(&mut self, stdout: &mut Stdout) -> Result<bool, &str> {
        enable_raw_mode().ok();
        stdout.execute(cursor::Hide).ok();
        loop {
            self.set_lasers_looking_at();
            self.draw(stdout);
            match read().unwrap() {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('w') => self.move_player(Direction::UP),
                        KeyCode::Char('s') => self.move_player(Direction::DOWN),
                        KeyCode::Char('a') => self.move_player(Direction::LEFT),
                        KeyCode::Char('d') => self.move_player(Direction::RIGHT),
                        //KeyCode::Up => self.change_player_direction(Direction::UP),
                        //KeyCode::Down => self.change_player_direction(Direction::DOWN),
                        //KeyCode::Left => self.change_player_direction(Direction::LEFT),
                        //KeyCode::Right => self.change_player_direction(Direction::RIGHT),
                        //KeyCode::Tab => self.toggle_portal(),
                        //KeyCode::Char(' ') => self.shoot_portal(),
                        KeyCode::Char('q') => break,
                        _ => (),
                    }
                }
                _ => (),
            }
        }
        stdout.execute(cursor::Show).ok();
        disable_raw_mode().ok();
        Ok(false)
    }
}
