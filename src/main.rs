use crossterm::{
    cursor,
    terminal::{disable_raw_mode, enable_raw_mode, Clear},
    ExecutableCommand,
};
use home::home_dir;
use l1t::level::*;
use l1t::menu::*;
use l1t::userdata::*;
use std::io::stdout;
use std::{thread, time};

fn main() {
    const SLEEP_TIME: u64 = 500;
    let home = match home_dir() {
        Some(h) => h,
        None => return,
    };
    let home = home.to_str().unwrap_or("");
    let mut user_data = match UserData::read(home.to_string()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let mut stdout = stdout();
    enable_raw_mode().ok();
    stdout.execute(cursor::Hide).ok();
    loop {
        let selection = Menu::open(MenuType::MainSelection(
            user_data.completed_core_levels.clone(),
        ))
        .unwrap_or(Selection::Play(1));
        match selection {
            Selection::Play(level) => {
                let mut current_level = level;
                loop {
                    if current_level >= Level::NUM_CORE_LEVELS {
                        Menu::open(MenuType::Message(
                            "You've completed all core levels, thanks for playing!".to_string(),
                        ));
                        break;
                    }
                    let mut level = match Level::core(current_level) {
                        Ok(l) => l,
                        Err(e) => {
                            eprintln!("{e}");
                            return;
                        }
                    };
                    let result = level.play();
                    match result {
                        Ok(result) => {
                            if result.has_won {
                                thread::sleep(time::Duration::from_millis(SLEEP_TIME));
                                Menu::open(MenuType::Message("YAY, You Won!".to_string()));
                                current_level += 1;
                                match user_data.complete_core(current_level as usize) {
                                    Err(e) => {
                                        eprintln!("{e}");
                                        return;
                                    }
                                    _ => (),
                                };
                            } else if let Some(r) = result.reason_for_loss {
                                match r {
                                    LevelLossReason::Zapper => {
                                        thread::sleep(time::Duration::from_millis(SLEEP_TIME));
                                        Menu::open(MenuType::Message(
                                            "Uh oh, you lit a zapper!".to_string(),
                                        ));
                                    }
                                    LevelLossReason::Death => {
                                        thread::sleep(time::Duration::from_millis(SLEEP_TIME));
                                        Menu::open(MenuType::Message(
                                            "Uh oh, you got shot by a laser beam!".to_string(),
                                        ));
                                    }
                                    LevelLossReason::Quit => break,
                                }
                            } else {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    }
                }
            }
            Selection::Help => {
                Menu::open(MenuType::HelpMenu);
            }
            _ => break,
        }
    }
    stdout.execute(cursor::Show).ok();
    disable_raw_mode().ok();
    stdout.execute(cursor::MoveTo(0, 0)).ok();
    stdout
        .execute(Clear(crossterm::terminal::ClearType::All))
        .ok();
}
