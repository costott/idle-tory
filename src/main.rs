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
    price: u64,
    mps_add: u64,
    level: u32,
    spaces: [usize; 4],
}

impl Upgrade{
    fn new(name: String, start_price: u64, mps_add: u64) -> Upgrade {
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
    status_info: HashMap<u64, String>
}

impl StatusManager {
    fn new() -> StatusManager {
        StatusManager {
            status_info: make_statuses()
        }
    }

    /// returns player status with current money
    fn get_status(&self, current_money: u64) -> String {
        let mut highest = &0; // highest status number 
        for key in self.status_info.keys() {
            if current_money > *key && key > highest {
                highest = key;
            }
        }
        self.status_info.get(highest).unwrap().to_string()   
    }
}

struct GameInfo {
    money: u64,
    mps: u64,
    upgrades: Vec<Upgrade>,
    upgrade_num: usize, // current upgrade up to
    status_manager: StatusManager,
    stdout: std::io::Stdout,
}

impl GameInfo {
    fn new() -> GameInfo {
        GameInfo {
            money: 5_00,
            mps: 1_00,
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
        if self.money < self.upgrades[self.upgrade_num].price / 10 {return}

        self.upgrade_num += 1;
        self.update_upgrade_num();
    }
}

/// makes and returns the upgrades at the start of the game
fn make_upgrades() -> Vec<Upgrade> {
    vec![
        Upgrade::new(String::from("tax fraud"), 500, 10), // (50s)
        Upgrade::new(String::from("lie to public"), 25_00, 75), // x5 x7.5 (33s) x0.66667
        Upgrade::new(String::from("pray to bojo"), 500_00, 25_00), // x20 x33.3 (20s) x0.6061
        Upgrade::new(String::from("public scandal"), 20_000_00, 2083_00), // x40 x80.9 (9.6s) x0.48
        Upgrade::new(String::from("break lockdown laws"), 4_000_000_00, 1_070_000_00), // x200 x514 (3.74s) x0.39
        Upgrade::new(String::from("privatise the NHS"), 20_000_000_000_00, 17_900_000_000_00) // x5000 x16728 (1.12) x0.3
    ]
}

/// makes and returns the statuses at the start of the game
fn make_statuses() -> HashMap<u64, String> {
    let mut statuses: HashMap<u64, String> = HashMap::new();
    statuses.insert(0, String::from("peasant"));
    statuses.insert(100_00, String::from("basically a labour voter"));
    statuses.insert(500_00, String::from("countryside conservative"));
    statuses.insert(1000_00, String::from("facebook conspiracist"));
    statuses.insert(5000_00, String::from("twitter terf"));
    statuses.insert(10_000_00, String::from("alt-right anti-vaxxer"));
    statuses.insert(100_000_00, String::from("liz truss"));
    statuses.insert(10_000_000_00, String::from("tory MP"));
    statuses.insert(1_000_000_000_00, String::from("resurrected margaret thatcher"));
    statuses
}

/// calculate the number of spaces upgrades need to add to their strings
/// so that the upgrade outputs are all aligned
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
const VALUES: [u64; 5] = [1_000_00, 1_000_000_00, 1_000_000_000_00, 1_000_000_000_000_00, 1_000_000_000_000_000_00];
/// converts a given money into its display form
fn display_money(money: u64) -> String {
    let money_str = money.to_string();
    if money < 1_000_00 {
        let l = money_str.len();
        if l == 1 {
            return format!("0.0{}", money_str);
        } else if l == 2{
            return format!("0.{}", &money_str[0..2])
        }
        return format!("{}.{}", &money_str[0..l-2], &money_str[l-2..l]);
    }

    let unit_i = (money_str.len()-3) / 3 - 1; // (n-1) DIV 3 - 1 where n = len-2 (get rid of pennies)
    let over_point = money / VALUES[unit_i];
    let l = over_point.to_string().len();
    format!("{}.{}{}", over_point, &money_str[l..l+3], UNITS[unit_i])
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
            display_money(game_info.upgrades[i].mps_add * game_info.upgrades[i].level as u64),
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

/// process what happens when an input is received into the channel
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
    game_info.upgrades[upgrade_index].price = (game_info.upgrades[upgrade_index].price as f32 
        * PRICE_MULTIPLIER) as u64;                               // increase price
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

    let mut increase: f32;
    // keeps track of any overflowed values as money is using
    // integers, so fractions of pennies are truncated
    let mut overflow: f32  = 0.0; 

    // main thread
    loop {
        thread::sleep(Duration::from_millis((1000f32/FPS) as u64));

        dt = last_frame_time.elapsed();
        last_frame_time = Instant::now();
        
        display_ui(&mut game_info);

        // increase the money with the mps
        increase = game_info.mps as f32 * dt.as_secs_f32();

        game_info.money += increase as u64;
        overflow += increase - increase.trunc(); 
        if overflow > 1f32 { // enough overflow to make an impact
            game_info.money += overflow as u64;
            overflow -= overflow.trunc();
        }

        recieve_input(&rx, &mut game_info);

        game_info.update_upgrade_num();
    }
}
