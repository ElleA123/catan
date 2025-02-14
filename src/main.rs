use std::{cell::RefCell, rc::Rc};
use std::ops::{Index, IndexMut};
use rand::{seq::{IndexedRandom, SliceRandom}, Rng};

pub mod render;
use render::render_screen;

//// Typedefs
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerColor {
    Red=0,
    Blue=1,
    Orange=2,
    White=3
}

impl From<usize> for PlayerColor {
    fn from(value: usize) -> Self {
        match value % 4 {
            0 => PlayerColor::Red,
            1 => PlayerColor::Blue,
            2 => PlayerColor::Orange,
            3 => PlayerColor::White,
            _ => panic!("PlayerColor::from(): Math has died :(")
        }
    }
}

impl Into<macroquad::color::Color> for PlayerColor {
    fn into(self) -> macroquad::color::Color {
        match self {
            PlayerColor::Red => macroquad::color::RED,
            PlayerColor::Blue => macroquad::color::BLUE,
            PlayerColor::Orange => macroquad::color::ORANGE,
            PlayerColor::White => macroquad::color::WHITE
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Resource {
    Wood,
    Brick,
    Wheat,
    Sheep,
    Ore,
}

const RESOURCES: [Resource; 5] = [
    Resource::Wood,
    Resource::Brick,
    Resource::Wheat,
    Resource::Sheep,
    Resource::Ore
];

impl Into<macroquad::color::Color> for Resource {
    fn into(self) -> macroquad::color::Color {
        match self {
            Resource::Wood => macroquad::color::DARKGREEN,
            Resource::Brick => macroquad::color::RED,
            Resource::Wheat => macroquad::color::GOLD,
            Resource::Sheep => macroquad::color::GREEN,
            Resource::Ore => macroquad::color::GRAY
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DVCard {
    Knight,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
    VictoryPoint
}

const DV_CARDS: [DVCard; 5] = [
    DVCard::Knight,
    DVCard::RoadBuilding,
    DVCard::YearOfPlenty,
    DVCard::Monopoly,
    DVCard::VictoryPoint
];

impl DVCard {
    fn into_label(self) -> String {
        String::from(match self {
            DVCard::Knight => "KN",
            DVCard::RoadBuilding => "RB",
            DVCard::YearOfPlenty => "YP",
            DVCard::Monopoly => "MN",
            DVCard::VictoryPoint => "VP"
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ResHand([usize; 5]);

const STARTING_BANK_HAND: ResHand = ResHand([19, 19, 19, 19, 19]);
const STARTING_DV_BANK: [DVCard; 25] = [
    DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight,
    DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight,
    DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::RoadBuilding,
    DVCard::RoadBuilding, DVCard::YearOfPlenty, DVCard::YearOfPlenty, DVCard::Monopoly, DVCard::Monopoly,
    DVCard::VictoryPoint, DVCard::VictoryPoint, DVCard::VictoryPoint, DVCard::VictoryPoint, DVCard::VictoryPoint,
];

const ROAD_HAND: ResHand = ResHand([1, 1, 0, 0, 0]);
const SETTLEMENT_HAND: ResHand = ResHand([1, 1, 1, 1, 0]);
const CITY_HAND: ResHand = ResHand([0, 0, 2, 0, 3]);
const DV_CARD_HAND: ResHand = ResHand([0, 0, 1, 1, 1]);

impl Index<Resource> for ResHand {
    type Output = usize;
    fn index(&self, index: Resource) -> &Self::Output {
        match index {
            Resource::Wood => &self.0[0],
            Resource::Brick => &self.0[1],
            Resource::Wheat => &self.0[2],
            Resource::Sheep => &self.0[3],
            Resource::Ore => &self.0[4],
        }
    }
}

impl IndexMut<Resource> for ResHand {
    fn index_mut(&mut self, index: Resource) -> &mut Self::Output {
        match index {
            Resource::Wood => &mut self.0[0],
            Resource::Brick => &mut self.0[1],
            Resource::Wheat => &mut self.0[2],
            Resource::Sheep => &mut self.0[3],
            Resource::Ore => &mut self.0[4],
        }
    }
}

impl ResHand {
    fn new() -> ResHand {
        ResHand([0; 5])
    }

    fn from(card: Resource) -> ResHand {
        let mut hand = ResHand::new();
        hand[card] = 1;
        hand
    }

    fn from_input() -> ResHand {
        // TODO
        let mut hand = ResHand::new();
        hand
    }

    fn sized_from_input(size: usize) -> ResHand {
        // TODO
        return ResHand::new();
        let mut hand;
        loop {
            hand = ResHand::new();
            if hand.size() == size {
                return hand;
            }
            println!("wrong size! ({})", size);
        }
    }

    fn clear(&mut self) {
        for res in RESOURCES {
            self[res] = 0;
        }
    }

    fn size(&self) -> usize {
        self.0.iter().sum()
    }

    fn add(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            self[res] += rhs[res];
        }
    }

    fn can_disc(&self, rhs: ResHand) -> bool {
        RESOURCES.iter().all(|&res| self[res] >= rhs[res])
    }

    fn pop_random<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Resource {
        let mut selected = rng.random_range(0..self.size());
        for res in RESOURCES {
            let count = self[res];
            if selected < count {
                self[res] -= 1;
                return res;
            } else {
                selected -= count;
            }
        }
        panic!("pop_random bugged");
    }

    fn discard(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            self[res] -= rhs[res];
        }
    }

    fn disc_max(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            if self[res] >= rhs[res] {
                self[res] -= rhs[res];
            } else {
                self[res] = 0;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DVHand([usize; 5]);

impl Index<DVCard> for DVHand {
    type Output = usize;
    fn index(&self, index: DVCard) -> &Self::Output {
        match index {
            DVCard::Knight => &self.0[0],
            DVCard::RoadBuilding => &self.0[1],
            DVCard::YearOfPlenty => &self.0[2],
            DVCard::Monopoly => &self.0[3],
            DVCard::VictoryPoint => &self.0[4],
        }
    }
}

impl IndexMut<DVCard> for DVHand {
    fn index_mut(&mut self, index: DVCard) -> &mut Self::Output {
        match index {
            DVCard::Knight => &mut self.0[0],
            DVCard::RoadBuilding => &mut self.0[1],
            DVCard::YearOfPlenty => &mut self.0[2],
            DVCard::Monopoly => &mut self.0[3],
            DVCard::VictoryPoint => &mut self.0[4],
        }
    }
}

impl DVHand {
    fn new() -> DVHand {
        DVHand([0; 5])
    }

    fn from(card: DVCard) -> DVHand {
        let mut hand = DVHand::new();
        hand[card] = 1;
        hand
    }

    fn clear(&mut self) {
        for dv in DV_CARDS {
            self[dv] = 0;
        }
    }

    fn size(&self) -> usize {
        self.0.iter().sum()
    }

    fn add_card(&mut self, card: DVCard) {
        self[card] += 1;
    }

    fn add(&mut self, rhs: DVHand) {
        for dv in DV_CARDS {
            self[dv] += rhs[dv];
        }
    }

    fn can_discard(&self, card: DVCard) -> bool {
        self[card] != 0
    }

    fn discard(&mut self, card: DVCard) {
        self[card] -= 1;
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
    color: PlayerColor
}

#[derive(Debug, Clone, Copy)]
struct Hex {
    resource: Resource,
    number: usize
}

#[derive(Debug, Clone, Copy)]
enum Port {
    ThreeForOne,
    TwoForOne(Resource)
}

#[derive(Debug)]
pub struct Board {
    num_players: usize,
    hexes: [[Option<Hex>; 5]; 5],
    ports: [Port; 9],
    structures: [[[Option<Structure>; 6]; 5]; 5],
    roads: [[[Option<PlayerColor>; 6]; 5]; 5],
    robber: [usize; 2],
    bank: ResHand,
    dv_bank: Vec<DVCard>,
}

const BOARD_COORDS: [[usize; 2]; 19] = [
    [0, 2], [0, 3], [0, 4],
    [1, 1], [1, 2], [1, 3], [1, 4],
    [2, 0], [2, 1], [2, 2], [2, 3], [2, 4],
    [3, 0], [3, 1], [3, 2], [3, 3],
    [4, 0], [4, 1], [4, 2]
];

const PORT_COORDS: [[usize; 3]; 9] = [
    [0, 3, 0], [0, 4, 1], [1, 4, 2],
    [3, 3, 2], [4, 2, 3], [4, 1, 4],
    [3, 0, 4], [2, 0, 5], [1, 1, 0]
];

const fn corner_coords() -> [[usize; 3]; 54] {
    let mut corners = [[0; 3]; 54];
    let mut idx = 0;

    let mut hex = 0;
    while hex < BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[hex];
        let mut corner = 0;
        while corner < 6 {
            let [r_, q_, corner_] = reduce_corner(r, q, corner);
            if r == r_ && q == q_ && corner == corner_ {
                corners[idx] = [r, q, corner];
                idx += 1;
            }
            corner += 1;
        } 
        hex += 1;
    }
    corners
}

const fn edge_coords() -> [[usize; 3]; 72] {
    let mut edges = [[0; 3]; 72];
    let mut idx = 0;

    let mut hex = 0;
    while hex < BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[hex];
        let mut edge = 0;
        while edge < 6 {
            let [r_, q_, edge_] = reduce_edge(r, q, edge);
            if r == r_ && q == q_ && edge == edge_ {
                edges[idx] = [r, q, edge];
                idx += 1;
            }
            edge += 1;
        } 
        hex += 1;
    }
    edges
}

const CORNER_COORDS: [[usize; 3]; 54] = corner_coords();
const EDGE_COORDS: [[usize; 3]; 72] = edge_coords();

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
    Port::ThreeForOne, Port::ThreeForOne, Port::ThreeForOne, Port::ThreeForOne,
    Port::TwoForOne(Resource::Wood),
    Port::TwoForOne(Resource::Brick),
    Port::TwoForOne(Resource::Sheep),
    Port::TwoForOne(Resource::Wheat),
    Port::TwoForOne(Resource::Ore)
];

impl Board {
    fn new<R: Rng + ?Sized>(num_players: usize, rng: &mut R) -> Self {
        let mut hexes: [[Option<Hex>; 5]; 5] = [[None; 5]; 5];
        let structures: [[[Option<Structure>; 6]; 5]; 5] = [[[None; 6]; 5]; 5];
        let roads: [[[Option<PlayerColor>; 6]; 5]; 5] = [[[None; 6]; 5]; 5];
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
        for [r, q] in BOARD_COORDS {
            // Check for desert
            if robber != [r, q] {
                // No sixes or eights next to each other
                if numbers[i] == 6 || numbers[i] == 8 {
                    for dir in [[0, -1], [-1, 0], [-1, 1]] {
                        let test_r = (r as isize + dir[0]) as usize;
                        let test_q = (q as isize + dir[1]) as usize;
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
            dv_bank: Vec::from(dv_bank)
        }
    }

    fn road_is_color(&self, r: usize, q: usize, edge: usize, color: PlayerColor) -> bool {
        match self.roads[r][q][edge] {
            Some(c) => c == color,
            None => false
        }
    }

    fn structure_is_color(&self, r: usize, q: usize, corner: usize, color: PlayerColor) -> bool {
        match self.structures[r][q][corner] {
            Some(c) => c.color == color,
            None => false
        }
    }

    fn get_colors_on_hex(&self, r: usize, q: usize) -> Vec<PlayerColor> {
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

    fn can_place_road(&self, r: usize, q: usize, edge: usize, color: PlayerColor) -> bool {
        self.roads[r][q][edge].is_none()
        && (
            edge_edge_neighbors(r, q, edge).any(|[r_, q_, e_]| self.road_is_color(r_, q_, e_, color))
            || edge_corner_neighbors(r, q, edge).any(|[r_, q_, c_]| self.structure_is_color(r_, q_, c_, color))
        )
    }

    fn can_place_setup_road(&self, r: usize, q: usize, edge: usize, settlement_coord: [usize; 3], color: PlayerColor) -> bool {
        self.roads[r][q][edge].is_none()
        && edge_corner_neighbors(r, q, edge).any(|coord| coord == settlement_coord)
    }

    fn place_road(&mut self, r: usize, q: usize, edge: usize, color: PlayerColor) {
        for [r, q, e] in get_dup_edges(r, q, edge) {
            self.roads[r][q][e] = Some(color);
        }
    }

    fn can_place_settlement(&self, r: usize, q: usize, corner: usize, color: PlayerColor) -> bool {
        self.structures[r][q][corner].is_none()
        && corner_corner_neighbors(r, q, corner).all(
            |[r_, q_, c_]| self.structures[r_][q_][c_].is_none()
        )
        && corner_edge_neighbors(r, q, corner).any(|[r_, q_, e_]| self.road_is_color(r_, q_, e_, color))
    }

    fn can_place_setup_settlement(&self, r: usize, q: usize, corner: usize) -> bool {
        self.structures[r][q][corner].is_none()
        && corner_corner_neighbors(r, q, corner).all(
            |[r_, q_, c_]| self.structures[r_][q_][c_].is_none()
        )
    }

    fn place_settlement(&mut self, r: usize, q: usize, corner: usize, color: PlayerColor) {
        for [r, q, c] in get_dup_corners(r, q, corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::Settlement,
                color
            });
        }
    }

    fn upgrade_to_city(&mut self, r: usize, q: usize, corner: usize) {
        let color = self.structures[r][q][corner].unwrap().color;
        for [r, q, c] in get_dup_corners(r, q, corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::City,
                color
            });
        }
    }

    fn give_resources(&self, roll: usize) -> Vec<ResHand> {
        let mut new_cards: Vec<ResHand> = vec![ResHand::new(); self.num_players];
        for [r, q] in BOARD_COORDS {
            if [r, q] == self.robber {
                continue;
            }
            if let Some(hex) = self.hexes[r][q] {
                if hex.number == roll {
                    for corner in self.structures[r][q] {
                        if let Some(s) = corner {
                            new_cards[s.color as usize][hex.resource] += match s.structure_type {
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

    fn draw_dv_card(&mut self) -> DVCard {
        self.dv_bank.pop().unwrap()
    }
}

struct Player {
    color: PlayerColor,
    is_human: bool,
    board: Rc<RefCell<Board>>,
    vps: usize,
    hand: ResHand,
    dvs: DVHand,
    new_dvs: DVHand,
    knights: usize,
    road_len: usize,
    road_pool: usize,
    settlement_pool: usize,
    city_pool: usize,
}

impl Player {
    fn new(color: PlayerColor, is_human: bool, board: Rc<RefCell<Board>>) -> Player {
        Player {
            color,
            is_human,
            board,
            vps: 0,
            hand: ResHand::new(),
            dvs: DVHand::new(),
            new_dvs: DVHand::new(),
            knights: 0,
            road_len: 0,
            road_pool: 15,
            settlement_pool: 5,
            city_pool: 4
        }
    }

    fn get_resources(&mut self, got: ResHand) {
        self.hand.add(got);
        self.board.borrow_mut().bank.disc_max(got);
    }

    fn discard_resources(&mut self, lost: ResHand) -> bool {
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
            return true;
        }
        return false;
    }

    // fn input_and_build_road(&mut self) {
    //     loop {
    //         let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
    //         let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
    //         let edge: usize = get_specific_input("edge: ", "it's a usize 0-6 silly! edge: ", |n| n < 6);
    //         if self.build_road(r, q, edge) {
    //             break;
    //         } else {
    //             println!("You can't build there stupid! Let's try again...");
    //         }
    //     }
    // }

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

    // fn input_and_build_settlement(&mut self) {
    //     loop {
    //         let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
    //         let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
    //         let corner: usize = get_specific_input("corner: ", "it's a usize 0-6 silly! corner: ", |n| n < 6);
    //         if self.build_settlement(r, q, corner) {
    //             break;
    //         } else {
    //             println!("You can't build there stupid! Let's try again...");
    //         }
    //     }
    // }

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

    // fn input_and_upgrade_to_city(&mut self) {
    //     loop {
    //         let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
    //         let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
    //         let corner: usize = get_specific_input("corner: ", "it's a usize 0-6 silly! corner: ", |n| n < 6);
    //         if self.upgrade_to_city(r, q, corner) {
    //             break;
    //         } else {
    //             println!("You can't upgrade there stupid! Let's try again...");
    //         }
    //     }
    // }

    fn buy_dv_card(&mut self) -> bool {
        let can_draw = self.board.borrow().dv_bank.len() > 0;
        if self.hand.can_disc(DV_CARD_HAND) && can_draw {
            self.discard_resources(DV_CARD_HAND);
            self.new_dvs.add_card(self.board.borrow_mut().draw_dv_card());
            true
        } else {
            false
        }
    }

    fn try_buy_dv_card(&mut self) {
        if self.buy_dv_card() {
            println!("Done!");
        } else {
            println!("You have not the materials!");
        }
    }

    fn play_dv_card(&mut self, card: DVCard) -> bool {
        if self.dvs[card] > 0 {
            self.dvs[card] -= 1;
            match card {
                DVCard::Knight => {}, // Knight
                DVCard::RoadBuilding => {}, // RB
                DVCard::YearOfPlenty => {}, // YearOfPlenty
                DVCard::Monopoly => {}, // Monopoly
                DVCard::VictoryPoint => return false
            }
            return true;
        }
        return false;
    }

    // fn input_and_play_dv_card(&mut self) -> DVCard {
    //     loop {
    //         let id: usize = get_specific_input("DV card:", "usize < 4 (can't play VPS)", |n| n < 4);
    //         let card = DV_CARDS[id];
    //         if self.play_dv_card(card) {
    //             return card;
    //         } else {
    //             println!("You don't even have that card bruh");
    //         }
    //     }
    // }

    fn offer_trade(&self) -> (ResHand, ResHand) {
        let mut give;
        loop {
            give = ResHand::from_input();
            if self.hand.can_disc(give) {
                break;
            }
        }
        let get = ResHand::from_input();
        (give, get)
    }

    fn handle_robber(&mut self) {
        if self.hand.size() > 7 {
            let amt_discarded = self.hand.size() / 2;
            let mut discarded = ResHand::sized_from_input(amt_discarded);
            while !(self.hand.can_disc(discarded)) {
                discarded = ResHand::sized_from_input(amt_discarded);
            }
            self.discard_resources(discarded);
        }
    }

    // fn move_robber(&self) -> Option<usize> {
    //     let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
    //     let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
    //     self.board.borrow_mut().robber = [r, q];
    //     let colors = self.board.borrow().get_colors_on_hex(r, q);
    //     if colors.len() > 0 {
    //         println!("Steal from one of: {}", colors.iter().map(|c| format!("{:?}", c)).collect::<Vec<String>>().join(","));
    //         Some(colors[0] as usize) // TODO - add choice
    //     } else {
    //         None
    //     }
    // }

    fn respond_to_trade(&self, give: ResHand, get: ResHand) -> bool {
        return true; // TODO - add choice
    }

    fn respond_to_trade_responses(&self, responses: Vec<bool>) -> Option<usize> {
        for (id, res) in responses.into_iter().enumerate() {
            if res {
                // TODO - add choice
                return Some(id);
            }
        }
        return None;
    } 

    // fn take_setup_turn(&mut self) {
    //     if self.is_human {
    //         // TODO: do better
    //         loop {
    //             let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
    //             let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
    //             let corner: usize = get_specific_input("corner: ", "it's a usize 0-6 silly! corner: ", |n| n < 6);
    //             if self.board.borrow().can_place_setup_settlement(r, q, corner) {
    //                 println!("Placing settlement at ({}, {}, {})", r, q, corner);
    //                 let conf = get_input("Type 'c' to confirm");
    //                 if conf == "c" {
    //                     self.board.borrow_mut().place_settlement(r, q, corner, self.color);
    //                     break;
    //                 }
    //             } else {
    //                 println!("You can't build there stupid! Let's try again...");
    //             }
    //         }
    //         loop {
    //             let r: usize = get_specific_input("r:", "it's a usize silly! r:", |n| n < 5);
    //             let q: usize = get_specific_input("q:", "it's a usize on the board, silly! q:", |n| is_on_board(r, n));
    //             let edge: usize = get_specific_input("edge: ", "it's a usize 0-6 silly! edge: ", |n| n < 6);
    //             if self.board.borrow().can_place_setup_road(r, q, edge, self.color) {
    //                 println!("Placing road at ({}, {}, {})", r, q, edge);
    //                 let conf = get_input("Type 'c' to confirm");
    //                 if conf == "c" {
    //                     self.board.borrow_mut().place_road(r, q, edge, self.color);
    //                     break;
    //                 }
    //             } else {
    //                 println!("You can't build there stupid! Let's try again...");
    //             }
    //         }
    //     }
    // }

    // fn take_preroll_turn(&mut self, has_played_dv: bool) -> TurnStatus {
    //     loop {
    //         let action: usize = get_specific_input("Action to take:", "usize < 2", |n| n < 2);
    //         match action {
    //             0 => return TurnStatus::Rolling, // Roll
    //             1 => { // Play DV
    //                 if !has_played_dv {
    //                     let card = self.input_and_play_dv_card();
    //                     if self.vps >= 10 {
    //                         return TurnStatus::Win;
    //                     }
    //                     return TurnStatus::PlayedDV(card);
    //                 } else {
    //                     println!("You've already played a DV this turn!");
    //                 }
    //             },
    //             _ => panic!("Player::take_preroll_turn(): invalid action")
    //         }
    //     }
    // }

    // fn take_postroll_turn(&mut self, has_played_dv: bool) -> TurnStatus {
    //     loop {
    //         let action: usize = get_specific_input("Action to take:", "usize < 7", |n| n < 7);
    //         match action {
    //             0 => self.input_and_build_road(), // Road
    //             1 => self.input_and_build_settlement(), // Settlement
    //             2 => self.input_and_upgrade_to_city(), // City
    //             3 => self.try_buy_dv_card(), // Buy DV card
    //             4 => { // Play DV card
    //                 if !has_played_dv {
    //                     return TurnStatus::PlayedDV(self.input_and_play_dv_card());
    //                 } else {
    //                     println!("You've already played a DV this turn!");
    //                 }
    //             },
    //             5 => { // Offer trade
    //                 let (give, get) = self.offer_trade();
    //                 return TurnStatus::TradeOffer(give, get);
    //             },
    //             6 => { // Finish turn
    //                 self.dvs.add(self.new_dvs);
    //                 self.new_dvs.clear();
    //                 return TurnStatus::Finished
    //             },
    //             _ => panic!("Player::take_turn(): invalid action")
    //         }
    //         if self.vps >= 10 {
    //             return TurnStatus::Win;
    //         }
    //     }
    // }

    // fn take_turn(&mut self, has_rolled: bool, has_played_dv: bool) -> TurnStatus {
    //     if !has_rolled {
    //         self.take_preroll_turn(has_played_dv)
    //     } else {
    //         self.take_postroll_turn(has_played_dv)
    //     }
    // }
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

const COORD_DIRS: [[isize; 2]; 6] = [
    [-1, 0],
    [-1, 1],
    [0, 1],
    [1, 0],
    [1, -1],
    [0, -1]
];

const fn is_on_board(r: usize, q: usize) -> bool {
    r < 5 && q < 5 && r + q >= 2 && r + q <= 6
}

fn get_dup_corners(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut dups = vec![[r, q, corner]];
    let neighbor1 = [(r as isize + COORD_DIRS[corner][0]) as usize, (q as isize + COORD_DIRS[corner][1]) as usize];
    if is_on_board(neighbor1[0], neighbor1[1]) {
        dups.push([neighbor1[0], neighbor1[1], (corner + 2) % 6]);
    }
    let neighbor2 = [(r as isize + COORD_DIRS[(corner + 1) % 6][0]) as usize, (q as isize + COORD_DIRS[(corner + 1) % 6][1]) as usize];
    if is_on_board(neighbor2[0], neighbor2[1]) {
        dups.push([neighbor2[0], neighbor2[1], (corner + 4) % 6]);
    }
    dups.into_iter()
}

fn get_dup_edges(r: usize, q: usize, edge: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut dups = vec![[r, q, edge]];
    let neighbor = [(r as isize + COORD_DIRS[edge][0]) as usize, (q as isize + COORD_DIRS[edge][1]) as usize];
    if is_on_board(neighbor[0], neighbor[1]) {
        dups.push([neighbor[0], neighbor[1], (edge + 3) % 6]);
    }
    dups.into_iter()
}

const fn reduce_corner(r: usize, q: usize, corner: usize) -> [usize; 3] {
    match corner {
        0 => if r != 0 && is_on_board(r - 1, q) {
            [r - 1, q, 2]
        } else if r != 0 && is_on_board(r - 1, q + 1) {
            [r - 1, q + 1, 4]
        } else {
            [r, q, 0]
        },
        1 => if r != 0 && is_on_board(r - 1, q + 1) {
            [r - 1, q + 1, 3]
        } else {
            [r, q, 1]
        },
        2 => [r, q, 2],
        3 => [r, q, 3],
        4 => if q != 0 && is_on_board(r, q - 1) {
            [r, q - 1, 2]
        } else {
            [r, q, 4]
        },
        5 => if r != 0 && is_on_board(r - 1, q) {
            [r - 1, q, 3]
        } else if q != 0 && is_on_board(r, q - 1) {
            [r, q - 1, 1]
        } else {
            [r, q, 5]
        },
        _ => panic!("main::reduce_corner(): invalid corner")
    }
}

const fn reduce_edge(r: usize, q: usize, edge: usize) -> [usize; 3] {
    match edge {
        0 => if r != 0 && is_on_board(r - 1, q) {
            [r - 1, q, 3]
        } else {
            [r, q, 0]
        },
        1 => if r != 0 && is_on_board(r - 1, q + 1) {
            [r - 1, q + 1, 4]
        } else {
            [r, q, 1]
        },
        2 => [r, q, 2],
        3 => [r, q, 3],
        4 => [r, q, 4],
        5 => if q != 0 && is_on_board(r, q - 1) {
            [r, q - 1, 2]
        } else {
            [r, q, 5]
        },
        _ => panic!("main::reduce_edge(): invalid edge")
    }
}

fn corner_corner_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    get_dup_corners(r, q, corner).map(move |[r_, q_, c]| [r_, q_, (c + 1) % 6])
}

fn edge_edge_neighbors(r: usize, q: usize, edge: usize) -> impl Iterator<Item = [usize; 3]> {
    get_dup_edges(r, q, edge).into_iter().flat_map(move |[r_, q_, e]|
        [1, 5].into_iter().map(move |step_e| [r_, q_, (e + step_e) % 6])
    )
}

fn corner_edge_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    get_dup_corners(r, q, corner) // hehe
}

fn edge_corner_neighbors(r: usize, q: usize, edge: usize) -> impl Iterator<Item = [usize; 3]> {
    get_dup_edges(r, q, edge).into_iter() // hehe
}

fn get_roll<R: Rng + ?Sized>(rng: &mut R) -> usize {
    rng.random_range(1..=6) + rng.random_range(1..=6)
}

// pub fn play_game(num_players: usize) {
//     let mut rng = rand::rng();
//     let board = Rc::new(RefCell::new(Board::new(num_players, &mut rng)));
//     let mut players = Vec::with_capacity(num_players);
//     for i in 0..num_players {
//         players.push(Player::new(PlayerColor::from(i), true, board.clone()));
//     }

//     for id in 0..num_players {
//         println!("{:?}", board.borrow());
//         players[id].take_setup_turn();
//     }
//     for id in (0..num_players).rev() {
//         players[id].take_setup_turn();
//     }
//     let mut largest_army = num_players;
//     let mut largest_army_size = 2;
//     let mut longest_road = num_players;
//     let mut longest_road_size = 4;
    
//     let mut turn = 0;
//     let winner;

//     let mut has_rolled = false;
//     let mut has_played_dv = false;
//     loop {
//         match players[turn].take_turn(has_rolled, has_played_dv) {
//             TurnStatus::Rolling => {
//                 has_rolled = true;
//                 let roll = get_roll(&mut rng);
//                 if roll != 7 {
//                     for (p, produced) in board.borrow().give_resources(roll).into_iter().enumerate() {
//                         players[p].get_resources(produced);
//                     }
//                 } else {
//                     turn += 1;
//                     for _ in 1..num_players {
//                         players[turn].handle_robber();
//                         turn += 1;
//                     }
//                     if let Some(robbed) = players[turn].move_robber() {
//                         let card_robbed = players[robbed].hand.pop_random(&mut rng);
//                         players[turn].get_resources(ResHand::from(card_robbed));
//                     }
//                 }
//             },
//             TurnStatus::PlayedDV(dv_card) => {
//                 has_played_dv = true;
//             }
//             TurnStatus::TradeOffer(give, get) => {
//                 let mut responses: Vec<bool> = Vec::with_capacity(num_players);
//                 turn += 1;
//                 for _ in 1..num_players {
//                     responses.push(players[turn].respond_to_trade(give, get));
//                     turn += 1;
//                 }
//                 if let Some(trader) = players[turn].respond_to_trade_responses(responses) {
//                     players[turn].discard_resources(give);
//                     players[turn].get_resources(get);
//                     players[trader].discard_resources(get);
//                     players[trader].get_resources(give);
//                 }
//             },
//             TurnStatus::Win => {
//                 winner = turn;
//                 break;
//             }
//             TurnStatus::Finished => {
//                 if players[turn].knights > largest_army_size && largest_army != turn {
//                     if largest_army != num_players {
//                         players[largest_army].vps -= 2;
//                     }
//                     largest_army = turn;
//                     players[largest_army].vps += 2;
//                     largest_army_size = players[turn].knights;
//                 }
//                 if players[turn].road_len > longest_road_size && longest_road != turn {
//                     if longest_road != num_players {
//                         players[longest_road].vps -= 2;
//                     }
//                     longest_road = turn;
//                     players[longest_road].vps += 2;
//                     longest_road_size = players[turn].road_len;
//                 }
//                 has_rolled = false;
//                 has_played_dv = false;
//                 turn += 1;
//             }
//         }
//     }
//     println!("{} wins!", winner);
// }

enum Action {
    Idling,
    Discarding(ResHand),
    MovingRobber,
    BuildingRoad,
    BuildingSettlement,
    UpgradingToCity,
}

pub struct TurnState {
    player: PlayerColor,
    action: Action,
    roll: Option<[usize; 2]>,
    played_dv: bool,
    offered_trades: Vec<(ResHand, ResHand)>
}

#[macroquad::main("Catan")]
async fn main() {
    let mut rng = rand::rng();
    let num_players = 4;
    let mut board = Board::new(num_players, &mut rng);

    board.place_settlement(2, 2, 3, PlayerColor::Orange);
    board.place_settlement(2, 2, 0, PlayerColor::Blue);
    // board.upgrade_to_city(2, 2, 0);
    board.place_road(2, 2, 0, PlayerColor::Blue);
    board.place_road(2, 2, 5, PlayerColor::Blue);
    board.place_road(1, 2, 4, PlayerColor::Blue);
    board.place_road(2, 1, 3, PlayerColor::Blue);


    let empty_hand = ResHand::new();
    let full_hand = ResHand([1, 1, 2, 0, 3]);
    let full_dv_hand = DVHand([1, 0, 0, 1, 1]);

    let pre_roll = TurnState {
        player: PlayerColor::Red,
        action: Action::Idling,
        roll: None,
        played_dv: false,
        offered_trades: Vec::new()
    };
    let post_roll = TurnState {
        player: PlayerColor::Blue,
        action: Action::Idling,
        roll: Some([3, 4]),
        played_dv: false,
        offered_trades: Vec::new()
    };
    let doing = TurnState {
        player: PlayerColor::Blue,
        action: Action::UpgradingToCity,
        roll: Some([3, 4]),
        played_dv: false,
        offered_trades: Vec::new()
    };

    loop {
        render_screen(&board, &full_hand, &full_dv_hand, &pre_roll);
        macroquad::window::next_frame().await
    }
}