use crossterm::{QueueableCommand, cursor, terminal, ExecutableCommand};
use crossterm::event::{Event, read, KeyEvent, KeyCode};
use costottorama::{text, back,style};
use std::time::{Duration, Instant};
use std::io::{Write, stdout};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

const FPS: f32 = 30f32;
const PRICE_MULTIPLIER: f32 = 1.25f32;

struct Upgrade {
    name: String,
    price: f32,
    mps_add: f32,
    level: u32,
    spaces: [usize; 4],
}

impl Upgrade{
    fn new(name: String, start_price: f32, mps_add: f32) -> Upgrade {
        Upgrade {
            name,
            price: start_price,
            mps_add,
            level: 0,
            spaces: [0, 0, 0, 0],
        }
    }
}

struct StatusManager {
    status_info: HashMap<u32, String>
}

impl StatusManager {
    fn new() -> StatusManager {
        StatusManager {
            status_info: make_statuses()
        }
    }

    fn get_status(&self, current_money: f32) -> String {
        let mut highest = &0; // highest status number 
        for key in self.status_info.keys() {
            if current_money > *key as f32 && key > highest {
                highest = key;
            }
        }
        self.status_info.get(highest).unwrap().to_string()   
    }
}

struct GameInfo {
    money: f32,
    mps: f32,
    upgrades: Vec<Upgrade>,
    upgrade_num: usize, // current upgrade up to
    status_manager: StatusManager,
    stdout: std::io::Stdout,
}

impl GameInfo {
    fn new() -> GameInfo {
        GameInfo {
            money: 5.0,
            mps: 1.0,
            upgrades: make_upgrades(),
            upgrade_num: 2,
            status_manager: StatusManager::new(),
            stdout: stdout(),
        }
    }

    /// returns the current status of the player
    fn get_status(&self) -> String {
        self.status_manager.get_status(self.money)
    }

    /// updates the number of upgrades unlocked
    fn update_upgrade_num(&mut self) {
        if self.upgrade_num >= self.upgrades.len() {return}
        if self.money < self.upgrades[self.upgrade_num].price * 0.1 {return}

        self.upgrade_num += 1;
        self.update_upgrade_num();
    }
}

/// makes and returns the upgrades at the start of the game
fn make_upgrades() -> Vec<Upgrade> {
    vec![
        Upgrade::new(String::from("tax fraud"), 5.0, 0.1), // (50s)
        Upgrade::new(String::from("lie to public"), 25.0, 0.75), // x5 x7.5 (33s) x0.66667
        Upgrade::new(String::from("pray to bojo"), 500.0, 25.0), // x20 x33.3 (20s) x0.6061
        Upgrade::new(String::from("public scandal"), 20_000.0, 2083.0), // x40 x80.9 (9.6s) x0.48
        Upgrade::new(String::from("break lockdown laws"), 4_000_000.0, 1_070_000.0), // x200 x514 (3.74s) x0.39
        Upgrade::new(String::from("privatise the NHS"), 20_000_000_000.0, 17_900_000_000.0) // x5000 x16728 (1.12) x0.3
    ]
}

/// makes and returns the statuses at the start of the game
fn make_statuses() -> HashMap<u32, String> {
    let mut statuses: HashMap<u32, String> = HashMap::new();
    statuses.insert(0, String::from("peasant"));
    statuses.insert(100, String::from("basically a labour voter"));
    statuses.insert(500, String::from("countryside bigot"));
    statuses.insert(1000, String::from("facebook conspiracist"));
    statuses.insert(5000, String::from("twitter terf"));
    statuses.insert(10_000, String::from("alt-right anti-vaxxer"));
    statuses.insert(100_000, String::from("liz truss"));
    statuses.insert(10_000_000, String::from("tory MP"));
    statuses.insert(1_000_000_000, String::from("resurrected margaret thatcher"));
    statuses
}

fn calculate_upgrade_spaces(game_info: &mut GameInfo){
    let mut maxes: [usize; 4] = [0, 0, 0, 0];
    for upgrade in &game_info.upgrades[0..game_info.upgrade_num] {
        if upgrade.name.chars().count() > maxes[0] {
            maxes[0] = upgrade.name.chars().count();
        }  
        let price_string_len = display_money(upgrade.price).chars().count();
        if price_string_len > maxes[1] {
            maxes[1] = price_string_len;
        }
        let add_string_len = display_money(upgrade.mps_add).chars().count();
        if add_string_len > maxes[2] {
            maxes[2] = add_string_len;
        }
        if upgrade.level.to_string().chars().count() > maxes[3] {
            maxes[3] = upgrade.level.to_string().chars().count();
        }
    }

    for u in game_info.upgrades[0..game_info.upgrade_num].iter_mut() {
        u.spaces[0] = maxes[0] - u.name.chars().count();
        u.spaces[1] = maxes[1] - display_money(u.price).chars().count();
        u.spaces[2] = maxes[2] - display_money(u.mps_add).chars().count();
        u.spaces[3] = maxes[3] - u.level.to_string().chars().count();
    }
}

const UNITS: [&str; 5] = ["K", "M", "B", "T", "Q"];
const VALUES: [u64; 5] = [1_000, 1_000_000, 1_000_000_000, 1_000_000_000_000, 1_000_000_000_000_000];
/// converts a given money into its display form
fn display_money(money: f32) -> String {
    if money < 1_000f32 {
        return format!("{:.2}", money);
    }
    let mut unit_i = 0;
    for (i, value) in VALUES.into_iter().enumerate() {
        if value as f32 > money {
            unit_i = i - 1;
            break;
        }
    }

    format!("{:.3}{}", money / VALUES[unit_i] as f32, UNITS[unit_i])
}

/// draws the UI for each frame
fn display_ui(game_info: &mut GameInfo)  {
    game_info.stdout.queue(terminal::Clear(terminal::ClearType::FromCursorUp)).unwrap();

    let mut output = String::from("");
    output += format!(
        "{}{}{}_____________\n| Idle Tory |{} ",
        style::UNDERLINED,
        style::BOLD,
        text::LIGHT_RED,
        style::RESET_ALL
    ).as_str();
    output += format!(
        "{}{}{} Status: {} {}\n\n",
        style::BOLD,
        back::WHITE,
        text::BLACK,
        game_info.get_status(),
        style::RESET_ALL
    ).as_str();

    output += format!(
        "{}{}£{} {}[+£{} per second]{}\n\n", 
        style::BOLD,
        text::LIGHT_YELLOW,
        display_money(game_info.money), 
        text::GREEN,
        display_money(game_info.mps),
        style::RESET_ALL
    ).as_str();

    calculate_upgrade_spaces(game_info);
    for i in 0..game_info.upgrade_num {
        output += format!(
            "{}{}[{}] {}{} £{}{} (+£{}){}{} {}[{}]{} [+£{}]{}\n", 
            style::BOLD,
            match game_info.money >= game_info.upgrades[i].price {
                true => text::WHITE,
                false => text::LIGHT_BLACK,
            },
            i+1,
            game_info.upgrades[i].name,
            " ".repeat(game_info.upgrades[i].spaces[0]),
            display_money(game_info.upgrades[i].price),
            " ".repeat(game_info.upgrades[i].spaces[1]),
            display_money(game_info.upgrades[i].mps_add),
            " ".repeat(game_info.upgrades[i].spaces[2]),
            style::RESET_ALL,
            match (game_info.money >= game_info.upgrades[i].price, game_info.upgrades[i].level > 0) {
                (true, true) | (false, true) => text::GREEN,
                (true, false) => text::CYAN,
                (false, false) => text::LIGHT_BLACK,
            },
            game_info.upgrades[i].level,
            " ".repeat(game_info.upgrades[i].spaces[3]),
            display_money(game_info.upgrades[i].mps_add * game_info.upgrades[i].level as f32),
            style::RESET_ALL
        ).as_str();
    }
    if game_info.upgrade_num < game_info.upgrades.len() {
        output += format!(
            "{}[{}] ??? {}",
            text::LIGHT_BLACK,
            game_info.upgrade_num + 1,
            style::RESET_ALL,
        ).as_str();
    }
    output += "\n";

    game_info.stdout.queue(cursor::MoveTo(0, 0)).unwrap();
    game_info.stdout.write_all(output.as_bytes()).unwrap();
    game_info.stdout.flush().unwrap();
}

/// process what happens when an input is recieved into the channel
fn recieve_input(rx: &std::sync::mpsc::Receiver<char>, game_info: &mut GameInfo) {
    // recieve the input
    #[allow(unused_assignments)]
    let mut input: char = ' ';
    if let Ok(inp) = rx.try_recv() {
        input = inp;
    } else {return}

    // converts input to a digit (if it can)
    let input = match input.to_digit(10) {
        Some(input) => input,
        None => return,
    };

    // makes sure digit is valid
    for i in 1..game_info.upgrade_num+1 {
        if input != i as u32 {continue}
        buy_upgrade(game_info, i-1);
    }
}

/// buys a given upgrade, updating related values
fn buy_upgrade(game_info: &mut GameInfo, upgrade_index: usize) {
    if game_info.upgrades[upgrade_index].price > game_info.money {return}

    game_info.money -= game_info.upgrades[upgrade_index].price;   // spend money
    game_info.mps += game_info.upgrades[upgrade_index].mps_add;   // increase mps
    game_info.upgrades[upgrade_index].level += 1;                 // increase level
    game_info.upgrades[upgrade_index].price *= PRICE_MULTIPLIER;  // increase price
}

fn main() {
    let (tx, rx) = mpsc::channel();

    let mut game_info = GameInfo::new();

    // thread to handle user input
    thread::spawn(move || {
        loop {
            let key_pressed = read().unwrap();

            let user_input = match key_pressed {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => c,
                _ => continue,
            };
            
            tx.send(user_input).unwrap();
        }
    });

    // prepare for start
    game_info.stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
    game_info.stdout.execute(cursor::Hide).unwrap();

    let mut last_frame_time = Instant::now();
    let mut dt;

    // main thread
    loop {
        thread::sleep(Duration::from_millis((1000f32/FPS) as u64));

        dt = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        
        display_ui(&mut game_info);

        // increase the money with the mps
        game_info.money += game_info.mps * dt.as_secs_f32();

        recieve_input(&rx, &mut game_info);

        game_info.update_upgrade_num();
    }
}
