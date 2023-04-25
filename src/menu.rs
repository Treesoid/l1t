use crate::level::Level;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode},
    execute,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
        StyledContent, Stylize,
    },
    terminal::{size, Clear, ClearType},
};
use std::io::stdout;

#[derive(Clone)]
pub enum Selection {
    Play(usize),
    Help,
    Quit,
    Yes,
    No,
    Item(usize),
}

pub enum MenuType {
    /// Dialog box with a single message. Press `Enter` or `q` to close.
    Message(String),

    /// Dialog box that displays the message `String` and will
    /// show a list of the given option `Strings`. Returns the
    /// index of the selected option on enter.
    Selection(String, Vec<String>),

    /// Same as `MenuType::Selection` but only allows selecting
    /// `Yes` or `No`.
    YesNoSelection(String),

    /// Same as `Message` but displays the entire help menu for
    /// the application.
    HelpMenu,

    /// Same as `Message` but displays more content and can be scrolled in.
    ScrollableMenu(Vec<Vec<StyledContent<&'static str>>>),

    /// Prints out the `Main Menu` of the application with the logo
    /// and selections for `Play`, `Help`, and `Quit`.
    /// The `Help` option will open up the `HelpMenu` and not return as selection.
    MainSelection(Vec<usize>),

    ///
    CoreLevelSelection(Vec<usize>),
}

const RED: Color = Color::Rgb { r: 255, g: 0, b: 0 };
const YELLOW: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 0,
};

pub struct Menu;

impl Menu {
    fn draw_borders(
        start_row: u16,
        end_row: u16,
        start_col: u16,
        end_col: u16,
    ) -> crossterm::Result<()> {
        for r in start_row..=end_row {
            for c in start_col..=end_col {
                if r == start_row || r == end_row {
                    execute!(stdout(), MoveTo(c, r), Print("─"),)?;
                } else if c == start_col || c == end_col {
                    execute!(stdout(), MoveTo(c, r), Print("│"),)?;
                } else {
                    execute!(stdout(), MoveTo(c, r), Print(" "),)?;
                }
            }
        }
        execute!(
            stdout(),
            MoveTo(start_col, start_row),
            Print("┌"),
            MoveTo(end_col, start_row),
            Print("┐"),
            MoveTo(start_col, end_row),
            Print("└"),
            MoveTo(end_col, end_row),
            Print("┘"),
        )
    }

    pub fn open(menu_type: MenuType) -> Option<Selection> {
        match menu_type {
            MenuType::MainSelection(completed_levels) => {
                let options: Vec<Selection> =
                    vec![Selection::Play(0), Selection::Help, Selection::Quit];
                let row_padding = 2;
                let col_padding = 3;
                let mut current_selection = 0;
                loop {
                    let (term_cols, term_rows) = size().unwrap_or((0, 0));
                    let start_row: u16 =
                        (term_rows - options.len() as u16 * 2 - 10 - row_padding) / 2 - row_padding;
                    let start_col: u16 = (term_cols - 23) / 2 - col_padding;
                    let end_row: u16 =
                        (term_rows + options.len() as u16 + 10 + row_padding) / 2 + row_padding;
                    let end_col: u16 = (term_cols + 23) / 2 + col_padding;
                    execute!(stdout(), Clear(ClearType::All)).ok();
                    Menu::draw_borders(start_row, end_row, start_col, end_col).ok();
                    execute!(
                        stdout(),
                        SetAttribute(Attribute::Bold),
                        MoveTo(start_col + col_padding, start_row + row_padding + 1),
                        Print("          /"),
                        SetForegroundColor(RED),
                        Print("-------"),
                        SetBackgroundColor(RED),
                        Print("L"),
                        SetBackgroundColor(Color::Reset),
                        MoveTo(start_col + col_padding, start_row + row_padding + 2),
                        SetForegroundColor(Color::Green),
                        Print(" ___      "),
                        SetForegroundColor(RED),
                        Print("|"),
                        SetForegroundColor(Color::Green),
                        Print("__      _"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 3),
                        Print("|_  |  "),
                        SetForegroundColor(RED),
                        Print("<--"),
                        SetForegroundColor(Color::White),
                        Print("/"),
                        SetForegroundColor(Color::Green),
                        Print("  |    | \\_"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 4),
                        Print("  | |     `| |    | __|"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 5),
                        Print("  | |      | |    | |"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 6),
                        Print("  | |_    _|_|_   | |_ "),
                        MoveTo(start_col + col_padding, start_row + row_padding + 7),
                        SetForegroundColor(RED),
                        Print("--"),
                        SetForegroundColor(Color::White),
                        Print("\\"),
                        SetForegroundColor(Color::Green),
                        Print("___\\  |_____| "),
                        SetForegroundColor(RED),
                        Print("--"),
                        SetForegroundColor(Color::White),
                        Print("\\"),
                        SetForegroundColor(Color::Green),
                        Print("__|"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 8),
                        SetForegroundColor(RED),
                        Print("  |                v"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 9),
                        Print("  v"),
                        MoveTo(start_col + col_padding, start_row + row_padding + 10),
                        Print("  "),
                        SetForegroundColor(YELLOW),
                        SetBackgroundColor(YELLOW),
                        Print("S"),
                        ResetColor,
                    )
                    .ok();
                    for i in 0..options.len() {
                        let option = match options[i] {
                            Selection::Play(_) => "P L A Y",
                            Selection::Help => "H E L P",
                            Selection::Quit => "Q U I T",
                            _ => "",
                        };
                        execute!(
                            stdout(),
                            SetForegroundColor(if i == current_selection {
                                Color::Black
                            } else {
                                Color::White
                            }),
                            SetBackgroundColor(if i == current_selection {
                                Color::White
                            } else {
                                Color::Reset
                            }),
                            MoveTo(
                                (term_cols - option.len() as u16) / 2 - 8,
                                start_row + row_padding * 2 + i as u16 * 2 + 10,
                            ),
                            SetAttribute(Attribute::Bold),
                            Print(format!("{:^23}", option)),
                            ResetColor,
                        )
                        .ok();
                    }
                    match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Enter => match options[current_selection] {
                                Selection::Play(_) => {
                                    if let Some(Selection::Item(i)) = Menu::open(
                                        MenuType::CoreLevelSelection(completed_levels.clone()),
                                    ) {
                                        return Some(Selection::Play(i));
                                    }
                                }
                                _ => break,
                            },
                            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                                if current_selection == 0 {
                                    current_selection = options.len() - 1;
                                } else {
                                    current_selection -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                                current_selection = (current_selection + 1) % options.len();
                            }
                            KeyCode::Char('q') => return Some(Selection::Quit),
                            _ => (),
                        },
                        _ => (),
                    }
                }
                return Some(options[current_selection].clone());
            }
            MenuType::Message(message) => {
                let row_padding = 1;
                let col_padding = 3;
                loop {
                    let (term_cols, term_rows) = size().unwrap_or((0, 0));
                    let start_row: u16 = term_rows / 2 - row_padding - 1;
                    let start_col: u16 = (term_cols - message.len() as u16) / 2 - col_padding;
                    let end_row: u16 = (term_rows + row_padding) / 2 + row_padding;
                    let end_col: u16 = (term_cols + message.len() as u16) / 2 + col_padding;
                    Menu::draw_borders(start_row, end_row, start_col, end_col).ok();
                    execute!(
                        stdout(),
                        MoveTo((term_cols - message.len() as u16) / 2, term_rows / 2),
                        Print(message.clone()),
                    )
                    .ok();
                    match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Enter => break,
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            MenuType::YesNoSelection(message) => {
                let row_padding = 1;
                let col_padding = 3;
                let mut current_selection = Selection::No;
                loop {
                    let (term_cols, term_rows) = size().unwrap_or((0, 0));
                    let start_row: u16 = term_rows / 2 - row_padding - 1;
                    let start_col: u16 = (term_cols - message.len() as u16) / 2 - col_padding;
                    let end_row: u16 = (term_rows + row_padding) / 2 + row_padding + 2;
                    let end_col: u16 = (term_cols + message.len() as u16) / 2 + col_padding;
                    Menu::draw_borders(start_row, end_row, start_col, end_col).ok();
                    execute!(
                        stdout(),
                        MoveTo((term_cols - message.len() as u16) / 2, term_rows / 2),
                        Print(message.clone()),
                    )
                    .ok();
                    execute!(
                        stdout(),
                        MoveTo(term_cols / 2 - 6, end_row - row_padding - 1),
                        SetForegroundColor(if matches!(current_selection, Selection::Yes) {
                            Color::Black
                        } else {
                            Color::White
                        }),
                        SetBackgroundColor(if matches!(current_selection, Selection::Yes) {
                            Color::White
                        } else {
                            Color::Reset
                        }),
                        Print(" YES ".bold()),
                        MoveTo(term_cols / 2 + 1, end_row - row_padding - 1),
                        SetForegroundColor(if matches!(current_selection, Selection::No) {
                            Color::Black
                        } else {
                            Color::White
                        }),
                        SetBackgroundColor(if matches!(current_selection, Selection::No) {
                            Color::White
                        } else {
                            Color::Reset
                        }),
                        Print(" NO ".bold())
                    )
                    .ok();
                    match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Left
                            | KeyCode::Right
                            | KeyCode::Char('a')
                            | KeyCode::Char('d')
                            | KeyCode::Char('h')
                            | KeyCode::Char('l') => {
                                if matches!(current_selection, Selection::No) {
                                    current_selection = Selection::Yes
                                } else {
                                    current_selection = Selection::No
                                }
                            }
                            KeyCode::Char('q') => return Some(Selection::No),
                            KeyCode::Enter => return Some(current_selection),
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            MenuType::HelpMenu => loop {
                return Menu::open(MenuType::ScrollableMenu(vec![
                    vec![
                        "l1t".bold().green(),
                        " - A terminal strategy game about moving".stylize(),
                    ],
                    vec!["      lasers and lighting statues.".stylize()],
                    vec![],
                    vec![
                        "In ".stylize(),
                        "l1t".bold().green(),
                        ", your goal is to use the available lasers ".stylize(),
                    ],
                    vec!["to light up all of the statues in the level.".stylize()],
                    vec![],
                    vec!["CONTROLS".bold().underlined()],
                    vec![],
                    vec![" W - ".bold(), "Move Up".stylize()],
                    vec![" S - ".bold(), "Move Down".stylize()],
                    vec![" A - ".bold(), "Move Left".stylize()],
                    vec![" D - ".bold(), "Move Right".stylize()],
                    vec![
                        " Space - ".bold(),
                        "Toggle surrounding blocks (if able)".stylize(),
                    ],
                    vec![" Shift-H - ".bold(), "Show this help menu".stylize()],
                    vec![" Q - ".bold(), "Quit".stylize()],
                    vec![],
                    vec!["Arrow keys can also be used to move around the ".stylize()],
                    vec!["level".stylize()],
                    vec![],
                    vec![
                        "X".green().on_green(),
                        " ".stylize(),
                        "PLAYER".bold().underlined(),
                    ],
                    vec![],
                    vec!["Hey, that's you!".stylize()],
                    vec![],
                    vec![
                        "L".with(RED).on(RED),
                        " ".stylize(),
                        "LASERS".bold().underlined(),
                    ],
                    vec![],
                    vec!["Lasers shoot laser beams in their set direction".stylize()],
                    vec!["(".stylize(), "UP, DOWN, LEFT, RIGHT".bold(), "). Laser beams are the key".stylize()],
                    vec!["to winning the game and can affect various ".stylize()],
                    vec!["blocks.".stylize()],
                    vec![],
                    vec!["Lasers cannot change directions but they can".stylize()],
                    vec!["be toggled on and off.".stylize()],
                    vec![],
                    vec![
                        "If a laser hits you, you'll ".stylize(),
                        "die".with(RED).bold(),
                        " and have to ".stylize(),
                    ],
                    vec!["restart the level.".stylize()],
                    vec![],
                    vec!["If a laser is hit by a laser beam, it will".stylize()],
                    vec!["turn off and must be toggled on by the player.".stylize()],
                    vec![],
                    vec![
                        "S".with(YELLOW).on(YELLOW),
                        " ".stylize(),
                        "STATUES".bold().underlined(),
                    ],
                    vec![],
                    vec!["All statues in a level must be lit up by a ".stylize()],
                    vec!["laser beam to ".stylize(), "win".with(YELLOW).bold(), " the level.".stylize()],
                    vec![],
                    vec!["Statues can not be moved or manually toggled.".stylize()],
                    vec![],
                    vec![
                        "R".bold().black().on(YELLOW),
                        " ".stylize(),
                        "REVERSE STATUES".bold().underlined(),
                    ],
                    vec![],
                    vec![
                        "Same as statues except they must ".stylize(),
                        "NOT".bold().italic(),
                        " be lit up ".stylize(),
                    ],
                    vec!["to ".stylize(), "win".with(YELLOW).bold(), " the level.".stylize()],
                    vec![],
                    vec!["/ ".bold(), "MIRRORS".bold().underlined()],
                    vec![],
                    vec!["Mirrors reflect laser beams in different".stylize()],
                    vec!["directions.".stylize()],
                    vec![],
                    vec!["             ".stylize(), "L".with(RED).on(RED)],
                    vec!["             |".bold().with(RED)],
                    vec![
                        "L".with(RED).on(RED),
                        "----".bold().with(RED),
                        "\\".bold(),
                        "    <--".bold().with(RED),
                        "/".bold(),
                    ],
                    vec!["     |".with(RED).bold()],
                    vec!["     V".with(RED).bold()],
                    vec![],
                    vec!["Mirrors cannot be moved but their direction can ".stylize()],
                    vec!["be toggled by the player.".stylize()],
                    vec![],
                    vec![
                        "/".black().on_white().bold(),
                        " ".stylize(),
                        "MOVEABLE MIRRORS".bold().underlined(),
                    ],
                    vec![],
                    vec!["Moveable Mirrors are the same as mirrors except ".stylize()],
                    vec!["they ".stylize(), "CAN ".bold().italic(), "be moved.".stylize()],
                    vec![],
                    vec![
                        "Z".bold().yellow().on_black(),
                        " ".stylize(),
                        "ZAPPERS".bold().underlined(),
                    ],
                    vec![],
                    vec!["If any Zappers are lit by a laser beam, you".stylize()],
                    vec!["will immediately ".stylize(), "lose".with(RED).bold(), " the level.".stylize()],
                    vec![],
                    vec![
                        "I".bold().white().on_white(),
                        " ".stylize(),
                        "B".bold().grey().on_grey(),
                        " ".stylize(),
                        "s".bold().black().on_red(),
                        " ".stylize(),
                        "OTHER BLOCKS".bold().underlined(),
                    ],
                    vec![],
                    vec![
                        "I".bold().white().on_white(),
                        " Walls - ".bold(),
                        "Cannot be moved by player, will block".stylize(),
                    ],
                    vec!["          laser beams.".stylize()],
                    vec![],
                    vec![
                        "B".bold().grey().on_grey(),
                        " Blocks - ".bold(),
                        "Can be moved around and will block".stylize(),
                    ],
                    vec!["           laser beams.".stylize()],
                    vec![],
                    vec![
                        "T".bold().magenta().on_magenta(),
                        " Toggle Blocks - ".bold(),
                        "Cannot be moved. Switches and".stylize(),
                    ],
                    vec!["                  buttons can toggle these on".stylize()],
                    vec!["                  and off.".stylize()],
                    vec![],
                    vec![
                        "s".bold().black().on_red(),
                        " Switches - ".bold(),
                        "When toggled, will turn toggle".stylize(),
                    ],
                    vec!["             blocks on/off.".stylize()],
                    vec![],
                    vec![
                        "b".bold().black().on_red(),
                        " Buttons - ".bold(),
                        "When pressed, will turn toggle".stylize(),
                    ],
                    vec!["            blocks on/off. Player must be".stylize()],
                    vec!["            next to button to press.".stylize()],
                ]));
            },
            MenuType::ScrollableMenu(content) => {
                let row_padding = 1;
                let col_padding = 2;
                let mut start_index: usize = 0;
                let scroll_message = "  USE ARROW KEYS OR W, S TO SCROLL  ";
                loop {
                    let (term_cols, term_rows) = size().unwrap_or((0, 0));
                    let lines: usize = (term_rows - row_padding * 2) as usize - 3;
                    let start_row = (term_rows - lines as u16) / 2 - row_padding;
                    let end_row = (term_rows + lines as u16) / 2 + row_padding;
                    let start_col = (term_cols - 50) / 2 - col_padding;
                    let end_col = (term_cols + 50) / 2 + col_padding;
                    execute!(
                        stdout(),
                        Clear(ClearType::All),
                        MoveTo((term_cols - scroll_message.len() as u16) / 2, start_row - 1),
                        Print(scroll_message.on_white().black().bold())
                    )
                    .ok();
                    Menu::draw_borders(start_row, end_row, start_col, end_col).ok();
                    for i in start_index..(start_index + lines).min(content.len()) {
                        execute!(
                            stdout(),
                            MoveTo(
                                start_col + col_padding + 1,
                                start_row + row_padding + (i - start_index) as u16 + 1
                            )
                        )
                        .ok();
                        for piece in content[i].iter() {
                            execute!(stdout(), Print(piece)).ok();
                        }
                    }
                    match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                                if start_index == 0 {
                                    continue;
                                }
                                start_index -= 1;
                            }
                            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                                if start_index + lines >= content.len() {
                                    continue;
                                }
                                start_index += 1;
                            }
                            KeyCode::Char('g') => start_index = 0,
                            KeyCode::Char('G') => start_index = content.len() - lines,
                            KeyCode::Enter | KeyCode::Char('q') => break,
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            MenuType::CoreLevelSelection(completed_levels) => {
                let levels_per_row = 10;
                let row_padding = 1;
                let col_padding = 3;
                let highest_completed_level = match completed_levels.iter().max() {
                    Some(n) => *n.min(&(Level::NUM_CORE_LEVELS - 1)),
                    None => 0,
                };
                let mut current_selection = highest_completed_level;
                let message = "  SELECT A LEVEL  ";
                loop {
                    let (term_cols, term_rows) = size().unwrap_or((0, 0));
                    let start_row: u16 = (term_rows
                        - row_padding * 2
                        - levels_per_row / Level::NUM_CORE_LEVELS as u16
                        + 1)
                        / 2;
                    let start_col: u16 = (term_cols - levels_per_row * 2) / 2 - col_padding;
                    let end_row: u16 = (term_rows
                        + row_padding * 2
                        + levels_per_row / Level::NUM_CORE_LEVELS as u16
                        + 1)
                        / 2;
                    let end_col: u16 = (term_cols + levels_per_row * 2) / 2 + col_padding;
                    execute!(
                        stdout(),
                        Clear(ClearType::All),
                        MoveTo((term_cols - message.len() as u16) / 2, start_row - 1),
                        Print(message.on_white().black().bold())
                    )
                    .ok();
                    Menu::draw_borders(start_row, end_row, start_col, end_col).ok();
                    for i in 0..Level::NUM_CORE_LEVELS {
                        let is_available = i <= highest_completed_level;
                        if is_available {
                            execute!(
                                stdout(),
                                MoveTo(
                                    (i as u16 % levels_per_row) * 2 + start_col + col_padding - 1,
                                    term_rows / 2
                                ),
                                SetForegroundColor(if current_selection == i {
                                    Color::Black
                                } else {
                                    Color::Reset
                                }),
                                SetBackgroundColor(if current_selection == i {
                                    Color::White
                                } else {
                                    Color::Reset
                                }),
                                Print((i + 1).to_string().bold()),
                            )
                            .ok();
                        } else {
                            execute!(
                                stdout(),
                                MoveTo(
                                    (i as u16 % levels_per_row) * 2 + start_col + col_padding - 1,
                                    term_rows / 2
                                ),
                                Print((i + 1).to_string().bold().black()),
                            )
                            .ok();
                        }
                    }
                    match read().unwrap() {
                        Event::Key(event) => match event.code {
                            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => {
                                if current_selection as isize - 1 < 0 {
                                    current_selection = highest_completed_level;
                                } else {
                                    current_selection -= 1;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                                if current_selection + 1 > highest_completed_level {
                                    current_selection = 0;
                                } else {
                                    current_selection += 1;
                                }
                            }
                            KeyCode::Char('q') => {
                                return None;
                            }
                            KeyCode::Enter => return Some(Selection::Item(current_selection)),
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }
        None
    }
}
