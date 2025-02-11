use core::panic;
use std::{cell::RefCell, rc::Rc};
use std::str::FromStr;
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use rand::{seq::{IndexedRandom, SliceRandom}, Rng};

//// Typedefs
#[derive(Debug, Clone, Copy)]
enum Resource {
    Wood=0,
    Brick=1,
    Wheat=2,
    Sheep=3,
    Ore=4,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Resource::Wood => "Wood",
            Resource::Brick => "Brick",
            Resource::Wheat => "Wheat",
            Resource::Sheep => "Sheep",
            Resource::Ore => "Ore"
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Copy)]
enum Port {
    Three,
    Two(Resource)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Color {
    Red=0,
    Blue=1,
    Orange=2,
    White=3
}

impl Color {
    fn from(id: usize) -> Color {
        match id {
            0 => Color::Red,
            1 => Color::Blue,
            2 => Color::Orange,
            3 => Color::White,
            _ => panic!("Invalid color ID")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StructureType {
    Settlement,
    City
}

#[derive(Debug, Clone, Copy)]
struct Structure {
    structure_type: StructureType,
    color: Color
}

#[derive(Debug, Clone, Copy)]
struct Hex {
    resource: Resource,
    number: usize
}

impl Display for Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.resource, self.number)
    }
}

#[derive(Debug, Clone, Copy)]
struct Hand([usize; 5]);

// const STARTING_PLAYER_HAND: Hand = Hand([4, 4, 2, 2, 0]);

const STARTING_BANK_HAND: Hand = Hand([19, 19, 19, 19, 19]);
// const STARTING_DV_BANK: Hand = Hand([14, 2, 2, 2, 5]);
const STARTING_DV_BANK: [DVCard; 25] = [
    DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight,
    DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight,
    DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::RoadBuilding,
    DVCard::RoadBuilding, DVCard::YOP, DVCard::YOP, DVCard::Monopoly, DVCard::Monopoly,
    DVCard::VP, DVCard::VP, DVCard::VP, DVCard::VP, DVCard::VP,
];

const ROAD_HAND: Hand = Hand([1, 1, 0, 0, 0]);
const SETTLEMENT_HAND: Hand = Hand([1, 1, 1, 1, 0]);
const CITY_HAND: Hand = Hand([0, 0, 2, 0, 3]);
const DV_CARD_HAND: Hand = Hand([0, 0, 1, 1, 1]);

impl Hand {
    fn new() -> Hand {
        Hand([0; 5])
    }

    fn from_card(card: usize) -> Hand {
        let mut hand = Hand::new();
        hand[card] = 1;
        hand
    }

    fn from_input() -> Hand {
        let mut hand = Hand::new();
        hand[0] = get_input_and_parse("wood: ", "type: usize");
        hand[1] = get_input_and_parse("brick: ", "type: usize");
        hand[2] = get_input_and_parse("wheat: ", "type: usize");
        hand[4] = get_input_and_parse("sheep: ", "type: usize");
        hand[0] = get_input_and_parse("ore: ", "type: usize");
        hand
    }

    fn size(&self) -> usize {
        self.0.iter().sum()
    }

    fn add(&mut self, rhs: Hand) {
        for i in 0..self.0.len() {
            self[i] += rhs[i];
        }
    }

    fn can_disc(&self, rhs: Hand) -> bool {
        (0..self.0.len()).all(|i| self[i] >= rhs[i])
    }

    fn pop_random<R: Rng + ?Sized>(&mut self, rng: &mut R) -> usize {
        let mut selected = rng.random_range(0..self.size());
        for (idx, count) in self.0.iter().enumerate() {
            if selected < *count {
                self[idx] -= 1;
                return idx;
            } else {
                selected -= count;
            }
        }
        println!("pop_random bugged");
        return 0
    }

    // fn disc_safe(&mut self, rhs: Hand) -> bool {
    //     if self.can_disc(rhs) {
    //         for i in 0..self.0.len() {
    //             self[i] -= rhs[i];
    //         }
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn discard(&mut self, rhs: Hand) {
        for i in 0..self.0.len() {
            self[i] -= rhs[i];
        }
    }

    fn disc_max(&mut self, rhs: Hand) {
        for i in 0..self.0.len() {
            if self[i] >= rhs[i] {
                self[i] -= rhs[i];
            } else {
                self[i] = 0;
            }
        }
    }
}

impl Index<usize> for Hand {
    type Output = usize;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Hand {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

const BOARD_COORDS: [(usize, usize); 19] = [
    (0, 2), (0, 3), (0, 4),
    (1, 1), (1, 2), (1, 3), (1, 4),
    (2, 0), (2, 1), (2, 2), (2, 3), (2, 4),
    (3, 0), (3, 1), (3, 2), (3, 3),
    (4, 0), (4, 1), (4, 2)
];
const PORT_COORDS: [(usize, usize, usize); 9] = [
    (0, 3, 5), (0, 4, 0), (1, 4, 1),
    (3, 3, 1), (4, 2, 2), (4, 1, 3),
    (3, 0, 3), (2, 0, 4), (1, 1, 5)
];

const BOARD_RESOURCES: [Resource; 18] = [
    Resource::Wood, Resource::Wood, Resource::Wood, Resource::Wood,
    Resource::Brick, Resource::Brick, Resource::Brick,
    Resource::Wheat, Resource::Wheat, Resource::Wheat, Resource::Wheat,
    Resource::Sheep, Resource::Sheep, Resource::Sheep, Resource::Sheep,
    Resource::Ore, Resource::Ore, Resource::Ore,
];
const BOARD_NUMBERS: [usize; 18] = [
    2, 3, 3, 4, 4, 5, 5, 6, 6, 8, 8, 9, 9, 10, 10, 11, 11, 12
];
const BOARD_PORTS: [Port; 9] = [
    Port::Three, Port::Three, Port::Three, Port::Three,
    Port::Two(Resource::Wood),
    Port::Two(Resource::Brick),
    Port::Two(Resource::Sheep),
    Port::Two(Resource::Wheat),
    Port::Two(Resource::Ore)
];

struct Board {
    num_players: usize,
    hexes: [[Option<Hex>; 5]; 5],
    ports: [Port; 9],
    structures: [[[Option<Structure>; 6]; 5]; 5],
    roads: [[[Option<Color>; 6]; 5]; 5],
    robber: (usize, usize),
    bank: Hand,
    dv_bank: Vec<DVCard>,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for r in 0..5 {
            for q in 0..5 {
                if let Some(hex) = self.get_hex(r, q) {
                    if hex.is_some() {
                        s += &hex.unwrap().to_string();
                        s += " ";
                    } else {
                        s += "Desert ";
                    }
                }
            }
            s += "\n";
        }
        write!(f, "{}", s)
    }
}

impl Board {
    fn new<R: Rng + ?Sized>(num_players: usize, rng: &mut R) -> Self {
        let mut hexes: [[Option<Hex>; 5]; 5] = [[None; 5]; 5];
        let structures: [[[Option<Structure>; 6]; 5]; 5] = [[[None; 6]; 5]; 5];
        let roads: [[[Option<Color>; 6]; 5]; 5] = [[[None; 6]; 5]; 5];
        let robber = *BOARD_COORDS.choose(rng).unwrap();

        // Shuffle resources
        let mut resources = BOARD_RESOURCES;
        resources.shuffle(rng);
        // Shuffle numbers
        let mut numbers = BOARD_NUMBERS;
        numbers.shuffle(rng);
        // Shuffle ports
        let mut ports = BOARD_PORTS;
        ports.shuffle(rng);
        // Shuffle DV cards
        let mut dv_bank = STARTING_DV_BANK;
        dv_bank.shuffle(rng);

        let mut i = 0;
        for (r, q) in BOARD_COORDS {
            // Check for desert
            if robber != (r, q) {
                // No sixes or eights next to each other
                if numbers[i] == 6 || numbers[i] == 8 {
                    for dir in [(0, -1), (-1, 0), (-1, 1)] {
                        let test_r = (r as isize + dir.0) as usize;
                        let test_q = (q as isize + dir.1) as usize;
                        if is_on_board(test_r, test_q) {
                            if let Some(h) = hexes[test_r][test_q] {
                                if h.number == 6 || h.number == 8 {
                                    return Board::new(num_players, rng);
                                }
                            }
                        }
                    }
                }

                // Create hex
                let hex = Hex {
                    resource: resources[i],
                    number: numbers[i]
                };
                hexes[r][q] = Some(hex);

                i += 1;
            }
        }

        Board {
            num_players,
            hexes,
            ports,
            structures,
            roads,
            robber,
            bank: STARTING_BANK_HAND,
            dv_bank: Vec::from(dv_bank) // maybe this is bad? idrk
        }
    }

    fn get_hex(&self, r: usize, q: usize) -> Option<Option<Hex>> {
        if is_on_board(r, q) {
            Some(self.hexes[r][q])
        }
        else {
            None
        }
    }

    fn get_colors_on_hex(&self, r: usize, q: usize) -> Vec<Color> {
        let mut colors = Vec::with_capacity(self.num_players);
        for corner in 0..6 {
            if let Some(s) = self.structures[r][q][corner] {
                if !colors.contains(&s.color) {
                    colors.push(s.color);
                }
            }
        }
        colors
    }

    fn road_is_color(&self, r: usize, q: usize, edge: usize, color: Color) -> bool {
        match self.roads[r][q][edge] {
            Some(c) => c == color,
            None => false
        }
    }

    // fn get_players_roads<'a>(&'a self, color: Color) -> impl Iterator<Item = (usize, usize, usize)> + 'a {
    //     BOARD_COORDS.into_iter().flat_map(|(r, q)| (0..6).map(move |e| (r, q, e)))
    //     .filter(move |&(r, q, e)|self.road_is_color(r, q, e, color))
    // }

    fn can_place_road(&self, r: usize, q: usize, edge: usize, color: Color) -> bool {
        self.roads[r][q][edge].is_none()
        && edge_edge_neighbors(r, q, edge).any(|(r_, q_, e_)| self.road_is_color(r_, q_, e_, color))
    }

    fn can_place_setup_road(&self, r: usize, q: usize, edge: usize) -> bool {
        self.roads[r][q][edge].is_none()
    }

    fn place_road(&mut self, r: usize, q: usize, edge: usize, color: Color) {
        for (r, q, e) in get_dup_edges(r, q, edge) {
            self.roads[r][q][e] = Some(color);
        }
    }

    fn can_place_settlement(&self, r: usize, q: usize, corner: usize, color: Color) -> bool {
        self.structures[r][q][corner].is_none()
        && corner_corner_neighbors(r, q, corner).all(
            |(r_, q_, c_)| self.structures[r_][q_][c_].is_none()
        )
        && cor_edge_neighbors(r, q, corner).any(|(r_, q_, e_)| self.road_is_color(r_, q_, e_, color))
    }

    fn can_place_setup_settlement(&self, r: usize, q: usize, corner: usize) -> bool {
        self.structures[r][q][corner].is_none()
        && corner_corner_neighbors(r, q, corner).all(
            |(r_, q_, c_)| self.structures[r_][q_][c_].is_none()
        )
    }

    fn place_settlement(&mut self, r: usize, q: usize, corner: usize, color: Color) {
        for (r, q, c) in get_dup_corners(r, q, corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::Settlement,
                color
            });
        }
    }

    fn upgrade_to_city(&mut self, r: usize, q: usize, corner: usize) {
        let color = self.structures[r][q][corner].unwrap().color;
        for (r, q, c) in get_dup_corners(r, q, corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::City,
                color
            });
        }
    }

    fn draw_dv_card(&mut self) -> DVCard {
        self.dv_bank.pop().unwrap()
    }

    fn give_resources(&self, roll: usize) -> Vec<Hand> {
        let mut new_cards: Vec<Hand> = vec![Hand::new(); self.num_players];
        for (r, q) in BOARD_COORDS {
            if (r, q) == self.robber {
                continue;
            }
            if let Some(hex) = self.hexes[r][q] {
                if hex.number == roll {
                    for corner in self.structures[r][q] {
                        if let Some(s) = corner {
                            new_cards[s.color as usize][hex.resource as usize] += match s.structure_type {
                                StructureType::Settlement => 1,
                                StructureType::City => 2
                            };
                        }
                    }
                }
            }
        }
        new_cards
    }
}

enum DVCard {
    Knight=0,
    RoadBuilding=1,
    YOP=2,
    Monopoly=3,
    VP=4
}

impl DVCard {
    fn from(idx: usize) -> DVCard {
        match idx {
            0 => DVCard::Knight,
            1 => DVCard::RoadBuilding,
            2 => DVCard::YOP,
            3 => DVCard::Monopoly,
            4 => DVCard::VP,
            _ => panic!("error: invalid DVCard index")
        }
    }
}

enum TurnStatus {
    Finished,
    Robber,
    TradeOffer(Hand, Hand),
    Win
}

struct Player {
    color: Color,
    is_human: bool,
    board: Rc<RefCell<Board>>,
    vps: usize,
    hand: Hand,
    dvs: Hand,
    new_dvs: Hand,
    knights: usize,
    road_pool: usize,
    settlement_pool: usize,
    city_pool: usize,
}

impl Player {
    fn new(color: Color, is_human: bool, board: Rc<RefCell<Board>>) -> Player {
        Player {
            color,
            is_human,
            board,
            vps: 0,
            hand: Hand::new(),
            dvs: Hand::new(),
            new_dvs: Hand::new(),
            knights: 0,
            road_pool: 15,
            settlement_pool: 5,
            city_pool: 4
        }
    }

    fn get_resources(&mut self, got: Hand) {
        self.hand.add(got);
        self.board.borrow_mut().bank.disc_max(got);
    }

    fn discard_resources(&mut self, lost: Hand) -> bool {
        if self.hand.can_disc(lost) {
            self.hand.discard(lost);
            self.board.borrow_mut().bank.add(lost);
            true
        } else {
            false
        }
    }

    fn build_road(&mut self, r: usize, q: usize, edge: usize) -> bool {
        let can_place_road = self.board.borrow().can_place_road(r, q, edge, self.color);
        if can_place_road && self.hand.can_disc(ROAD_HAND) && self.road_pool > 0 {
            self.discard_resources(ROAD_HAND);
            self.board.borrow_mut().place_road(r, q, edge, self.color);
            self.road_pool -= 1;
            true
        } else {
            false
        }
    }

    fn build_settlement(&mut self, r: usize, q: usize, corner: usize) -> bool {
        println!("works on board: {}", self.board.borrow().can_place_settlement(r, q, corner, self.color));
        println!("has cards: {}", self.hand.can_disc(SETTLEMENT_HAND));
        println!("settlements available: {}", self.settlement_pool > 0);
        let can_place_settlement = self.board.borrow().can_place_settlement(r, q, corner, self.color);
        if can_place_settlement && self.hand.can_disc(SETTLEMENT_HAND) && self.settlement_pool > 0 {
            self.discard_resources(SETTLEMENT_HAND);
            self.board.borrow_mut().place_settlement(r, q, corner, self.color);
            self.settlement_pool -= 1;
            true
        } else {
            false
        }
    }

    fn upgrade_to_city(&mut self, r: usize, q: usize, corner: usize) -> bool {
        let Some(s) = self.board.borrow().structures[r][q][corner] else { return false; };
        if s.structure_type == StructureType::Settlement && s.color == self.color && self.hand.can_disc(CITY_HAND) {
            self.discard_resources(CITY_HAND);
            self.board.borrow_mut().upgrade_to_city(r, q, corner);
            self.city_pool -= 1;
            self.settlement_pool += 1;
            true
        } else {
            false
        }
    }

    fn buy_dv_card(&mut self) -> bool {
        let can_draw = self.board.borrow().dv_bank.len() > 0;
        if self.hand.can_disc(DV_CARD_HAND) && can_draw {
            self.discard_resources(DV_CARD_HAND);
            self.new_dvs.add(Hand::from_card(self.board.borrow_mut().draw_dv_card() as usize));
            true
        } else {
            false
        }
    }

    fn handle_robber(&mut self) {
        if self.hand.size() > 7 {
            let amt_discarded = self.hand.size() / 2;
            let mut discarded = Hand::from_input();
            while !(discarded.size() == amt_discarded && self.hand.can_disc(discarded)) {
                discarded = Hand::from_input();
            }
            self.discard_resources(discarded);
        }
    }

    fn move_robber(&self) -> Option<usize> {
        // TODO - add choice
        let r = 2;
        let q = 2;
        self.board.borrow_mut().robber = (r, q);
        let colors = self.board.borrow().get_colors_on_hex(r, q);
        if colors.len() > 0 {
            Some(colors[0] as usize) // TODO - add choice
        } else {
            None
        }
    }

    fn respond_to_trade(&self, give: Hand, get: Hand) -> bool {
        return true; // TODO - add choice
    }

    fn respond_to_trade_responses(&self, responses: Vec<bool>) -> Option<usize> {
        for (id, res) in responses.into_iter().enumerate() {
            if res {
                return Some(id);
            }
        }
        return None;
    } 

    fn take_setup_turn(&mut self) {
        if self.is_human {
            loop {
                let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
                let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
                let corner: usize = get_specific_input("corner: ", "it's a usize 0-6 silly! corner: ", |n| n < 6);
                if self.board.borrow().can_place_setup_settlement(r, q, corner) {
                    println!("Placing settlement at ({}, {}, {})", r, q, corner);
                    let conf = get_input("Type 'c' to confirm");
                    if conf == "c" {
                        self.board.borrow_mut().place_settlement(r, q, corner, self.color);
                        break;
                    }
                } else {
                    println!("You can't build there stupid! Let's try again...");
                }
            }
            loop {
                let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
                let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
                let edge: usize = get_specific_input("edge: ", "it's a usize 0-6 silly! edge: ", |n| n < 6);
                if self.board.borrow().can_place_setup_road(r, q, edge) {
                    println!("Placing road at ({}, {}, {})", r, q, edge);
                    let conf = get_input("Type 'c' to confirm");
                    if conf == "c" {
                        self.board.borrow_mut().place_road(r, q, edge, self.color);
                        break;
                    }
                } else {
                    println!("You can't build there stupid! Let's try again...");
                }
            }
        }
    }

    fn take_turn(&mut self) -> TurnStatus {
        TurnStatus::Finished
    }
}

//// Coordinate manipulation
// - Hex coords: axial coordinates (r, q)
// r loosely corresponds with row, q with col.
//
// - Corner coords: (r, q, corner)
// Defined as an absolute position on a hex,
// starting from 0 at the top corner and incrementing clockwise.
// This means corners can have up to three sets of coordinates (one for each touching hex)
//
// - Edge coords: (r, q, edge)
// Defined very similarly to corners, starting from the top-left edge
// the edge (a, b, c) is a half-step counterclockwise from the corner (a, b, c)

fn is_on_board(r: usize, q: usize) -> bool {
    r < 5 && q < 5 && r + q >= 2 && r + q <= 6
}

const DIRS: [(isize, isize); 6] = [
    (-1, 0),
    (-1, 1),
    (0, 1),
    (1, 0),
    (1, -1),
    (0, -1)
];

fn get_s(r: usize, q: usize) -> usize {
    6 - r - q
}

// fn neighbors(r: usize, q: usize) -> Vec<(usize, usize)> {
//     let mut neighbors = Vec::new();
//     for dir in DIRS {
//         if is_on_board(
//             (r as isize + dir.0) as usize,
//             (q as isize + dir.1) as usize
//         ) {
//             neighbors.push((
//                 (r as isize + dir.0) as usize,
//                 (q as isize + dir.1) as usize)
//             );
//         }
//     }
//     neighbors
// }

fn get_dup_corners(r: usize, q: usize, corner: usize) -> Vec<(usize, usize, usize)> {
    let mut dups = vec![(r, q, corner)];
    let neighbor1 = ((r as isize + DIRS[corner].0) as usize, (q as isize + DIRS[corner].1) as usize);
    if is_on_board(neighbor1.0, neighbor1.1) {
        dups.push((neighbor1.0, neighbor1.1, (corner + 2) % 6));
    }
    let neighbor2 = ((r as isize + DIRS[(corner + 1) % 6].0) as usize, (q as isize + DIRS[(corner + 1) % 6].1) as usize);
    if is_on_board(neighbor2.0, neighbor2.1) {
        dups.push((neighbor2.0, neighbor2.1, (corner + 4) % 6));
    }
    dups
}

fn get_dup_edges(r: usize, q: usize, edge: usize) -> Vec<(usize, usize, usize)> {
    let mut dups = vec![(r, q, edge)];
    let neighbor = ((r as isize + DIRS[edge].0) as usize, (q as isize + DIRS[edge].1) as usize);
    if is_on_board(neighbor.0, neighbor.1) {
        dups.push((neighbor.0, neighbor.1, (edge + 3) % 6));
    }
    dups
}

fn corner_corner_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    get_dup_corners(r, q, corner).into_iter().map(move |(r_, q_, c)| (r_, q_, (c + 1) % 6))
}

fn edge_edge_neighbors(r: usize, q: usize, edge: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    get_dup_edges(r, q, edge).into_iter().flat_map(move |(r_, q_, e)|
        [1, 5].into_iter().map(move |step_e| (r_, q_, (e + step_e) % 6))
    )
}

fn cor_edge_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    get_dup_corners(r, q, corner).into_iter() // hehe
}

fn edge_cor_neighbors(r: usize, q: usize, edge: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    get_dup_edges(r, q, edge).into_iter()
}

fn get_input(msg: &str) -> String {
    println!("{}", msg);
    let mut buf = String::new();
    std::io::stdin()
        .read_line(&mut buf)
        .expect("Failed to read line");
    buf.trim().to_owned()
}

fn get_input_and_parse<T: FromStr>(msg: &str, err_msg: &str) -> T {
    println!("{}", msg);
    let mut buf = String::new();
    loop {
        buf.clear();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read line");
        if let Ok(t) = buf.trim().parse::<T>() {
            return t;
        }
        println!("{}", err_msg);
    }
}

// I'll remove this fn someday (the ones above it too)
// I WILL make a better ui than text input
fn get_specific_input<T, F>(msg: &str, err_msg: &str, pred: F) -> T where T: FromStr + Copy, F: Fn(T) -> bool {
    println!("{}", msg);
    let mut buf = String::new();
    loop {
        buf.clear();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read line");
        if let Ok(t) = buf.trim().parse::<T>() {
            if pred(t) {
                return t;
            }
        }
        println!("{}", err_msg);
    }
}

fn play_game(num_players: usize) {
    let mut rng = rand::rng();
    let board = Rc::new(RefCell::new(Board::new(num_players, &mut rng)));
    let mut players = Vec::with_capacity(num_players);
    for i in 0..num_players {
        players.push(Player::new(Color::from(i), true, board.clone()));
    }

    for id in 0..num_players {
        players[id].take_setup_turn();
    }
    for id in (0..num_players).rev() {
        players[id].take_setup_turn();
    }
    let mut largest_army: Option<usize> = None;
    let mut longest_road: Option<usize> = None;
    
    let mut turn = 0;
    let mut winner = 0;
    loop {
        match players[turn].take_turn() {
            TurnStatus::Robber => {
                turn += 1;
                for _ in 1..num_players {
                    players[turn].handle_robber();
                    turn += 1;
                }
                if let Some(robbed) = players[turn].move_robber() {
                    let card_robbed = players[robbed].hand.pop_random(&mut rng);
                    players[turn].get_resources(Hand::from_card(card_robbed));
                }
            },
            TurnStatus::TradeOffer(give, get) => {
                let mut responses: Vec<bool> = Vec::with_capacity(num_players);
                turn += 1;
                for _ in 1..num_players {
                    responses.push(players[turn].respond_to_trade(give, get));
                    turn += 1;
                }
                if let Some(trader) = players[turn].respond_to_trade_responses(responses) {
                    players[turn].discard_resources(give);
                    players[turn].get_resources(get);
                    players[trader].discard_resources(get);
                    players[trader].get_resources(give);
                }
            },
            TurnStatus::Win => {
                winner = turn;
                break;
            }
            TurnStatus::Finished => turn += 1
        }
    }
    println!("{:?} wins!", Color::from(winner));
}

fn main() {
    // println!("{:?}", get_dup_edges(2, 2, 0));
    // for e in edge_edge_neighbors(2, 2, 0) {
    //     println!("({}, {}, {})", e.0, e.1, e.2);
    // }

    // println!("{:?}", get_dup_corners(2, 2, 0));
    // for e in corner_corner_neighbors(2, 2, 0) {
    //     println!("({}, {}, {})", e.0, e.1, e.2);
    // }
    let num_players = 4;
    // play_game(num_players);

    let mut rng = rand::rng();
    let board = Rc::new(RefCell::new(Board::new(num_players, &mut rng)));
    let mut players = Vec::with_capacity(num_players);
    for i in 0..num_players {
        players.push(Player::new(Color::from(i), true, board.clone()));
    }

    board.borrow_mut().place_settlement(2, 2, 0, Color::Red);
    board.borrow_mut().place_road(2, 2, 1, Color::Red);
    println!("{}", players[0].build_road(2, 2, 2));
    println!("{}", players[0].build_settlement(2, 2, 2));
    println!("{}", players[0].build_settlement(1, 2, 0));
    println!("{}", players[0].upgrade_to_city(2, 2, 0));
}