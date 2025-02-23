use std::ops::{Index, IndexMut};
use rand::{seq::{IndexedRandom, SliceRandom}, Rng};

use crate::render::{render_screen, ClickablePoints};
use crate::Player;

//// Typedefs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerColor {
    Red=0,
    Blue=1,
    Orange=2,
    White=3
}

pub const PLAYER_COLORS: [PlayerColor; 4] = [
    PlayerColor::Red,
    PlayerColor::Blue,
    PlayerColor::Orange,
    PlayerColor::White,
];

impl From<usize> for PlayerColor {
    fn from(value: usize) -> Self {
        PLAYER_COLORS[value % 4]
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
pub enum Resource {
    Wood,
    Brick,
    Wheat,
    Sheep,
    Ore,
}

pub const RESOURCES: [Resource; 5] = [
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
pub enum DVCard {
    Knight,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
    VictoryPoint
}

pub const DV_CARDS: [DVCard; 5] = [
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

pub const ROAD_HAND: ResHand = ResHand([1, 1, 0, 0, 0]);
pub const SETTLEMENT_HAND: ResHand = ResHand([1, 1, 1, 1, 0]);
pub const CITY_HAND: ResHand = ResHand([0, 0, 2, 0, 3]);
pub const DV_CARD_HAND: ResHand = ResHand([0, 0, 1, 1, 1]);

const STARTING_BANK_HAND: ResHand = ResHand([19, 19, 19, 19, 19]);
const STARTING_DV_BANK_HAND: DVHand = DVHand([14, 2, 2, 2, 5]);
// const STARTING_DV_BANK: [DVCard; 25] = [
//     DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight,
//     DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight,
//     DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::Knight, DVCard::RoadBuilding,
//     DVCard::RoadBuilding, DVCard::YearOfPlenty, DVCard::YearOfPlenty, DVCard::Monopoly, DVCard::Monopoly,
//     DVCard::VictoryPoint, DVCard::VictoryPoint, DVCard::VictoryPoint, DVCard::VictoryPoint, DVCard::VictoryPoint,
// ];

impl ResHand {
    fn new() -> ResHand {
        ResHand([0; 5])
    }

    fn clear(&mut self) {
        for res in RESOURCES {
            self[res] = 0;
        }
    }

    fn size(&self) -> usize {
        self.0.iter().sum()
    }

    fn count_nonzero(&self) -> usize {
        self.0.iter().filter(|&res| *res != 0).count()
    }

    fn nth_nonzero(&self, n: usize) -> Option<Resource> {
        println!("finding {n}th resource card");
        RESOURCES.iter().copied().filter(|c| self[*c] > 0).nth(n)
    }

    fn add(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            self[res] += rhs[res];
        }
    }

    fn add_card(&mut self, card: Resource) {
        self[card] += 1;
    }

    fn can_discard(&self, rhs: ResHand) -> bool {
        RESOURCES.iter().all(|&res| self[res] >= rhs[res])
    }

    fn discard(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            self[res] -= rhs[res];
        }
    }

    fn discard_max(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            if self[res] >= rhs[res] {
                self[res] -= rhs[res];
            } else {
                self[res] = 0;
            }
        }
    }

    fn discard_random<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<Resource> {
        if self.size() == 0 {
            return None;
        }
        let mut selected = rng.random_range(0..self.size());
        for res in RESOURCES {
            let count = self[res];
            if selected < count {
                self[res] -= 1;
                return Some(res);
            } else {
                selected -= count;
            }
        }
        panic!("ResHand::discard_random bugged");
    }
}

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

impl From<Resource> for ResHand {
    fn from(value: Resource) -> Self {
        let mut hand = ResHand::new();
        hand[value] = 1;
        hand
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DVHand([usize; 5]);

impl DVHand {
    fn new() -> DVHand {
        DVHand([0; 5])
    }

    fn clear(&mut self) {
        for dv in DV_CARDS {
            self[dv] = 0;
        }
    }

    fn size(&self) -> usize {
        self.0.iter().sum()
    }

    fn count_nonzero(&self) -> usize {
        self.0.iter().filter(|&dv| *dv != 0).count()
    }

    fn nth_nonzero(&self, n: usize) -> Option<DVCard> {
        println!("finding {n}th dv card");
        DV_CARDS.iter().copied().filter(|c| self[*c] > 0).nth(n)
    }

    fn add(&mut self, rhs: DVHand) {
        for dv in DV_CARDS {
            self[dv] += rhs[dv];
        }
    }

    fn add_card(&mut self, card: DVCard) {
        self[card] += 1;
    }

    fn can_discard_card(&self, card: DVCard) -> bool {
        self[card] != 0
    }

    fn discard_card(&mut self, card: DVCard) {
        self[card] -= 1;
    }

    fn discard_random<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<DVCard> {
        if self.size() == 0 {
            return None;
        }
        let mut selected = rng.random_range(0..self.size());
        for dv in DV_CARDS {
            let count = self[dv];
            if selected < count {
                self[dv] -= 1;
                return Some(dv);
            } else {
                selected -= count;
            }
        }
        panic!("DVHand::discard_random bugged");
    }
}

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

impl From<DVCard> for DVHand {
    fn from(value: DVCard) -> Self {
        let mut hand = DVHand::new();
        hand[value] = 1;
        hand
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructureType {
    Settlement,
    City
}

#[derive(Debug, Clone, Copy)]
pub struct Structure {
    structure_type: StructureType,
    color: PlayerColor
}

#[derive(Debug, Clone, Copy)]
pub struct Hex {
    resource: Resource,
    number: usize
}

#[derive(Debug, Clone, Copy)]
pub enum Port {
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
    dv_bank: DVHand,
}

pub const BOARD_COORDS: [[usize; 2]; 19] = [
    [0, 2], [0, 3], [0, 4],
    [1, 1], [1, 2], [1, 3], [1, 4],
    [2, 0], [2, 1], [2, 2], [2, 3], [2, 4],
    [3, 0], [3, 1], [3, 2], [3, 3],
    [4, 0], [4, 1], [4, 2]
];

pub const PORT_COORDS: [[usize; 3]; 9] = [
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

pub const CORNER_COORDS: [[usize; 3]; 54] = corner_coords();
pub const EDGE_COORDS: [[usize; 3]; 72] = edge_coords();

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
        // // Shuffle DV cards
        // let mut dv_bank = STARTING_DV_BANK;
        // dv_bank.shuffle(rng);

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
            dv_bank: STARTING_DV_BANK_HAND
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

    fn structure_isnt_color(&self, r: usize, q: usize, corner: usize, color: PlayerColor) -> bool {
        match self.structures[r][q][corner] {
            Some(c) => c.color != color,
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
            edge_corner_neighbors(r, q, edge).into_iter().any(|[r_, q_, c_]| self.structure_is_color(r_, q_, c_, color))
            || edge_edge_neighbors(r, q, edge).any(|[r_, q_, e_]| {
                let [r_int, q_int, c_int] = intersecting_corner([r, q, edge], [r_, q_, e_]).unwrap();
                self.road_is_color(r_, q_, e_, color) && !self.structure_isnt_color(r_int, q_int, c_int, color)
            })
        )
    }

    fn can_place_setup_road(&self, r: usize, q: usize, edge: usize, settlement_coord: [usize; 3]) -> bool {
        self.roads[r][q][edge].is_none()
        && edge_corner_neighbors(r, q, edge).into_iter().any(|coord|
            reduce_corner(coord[0], coord[1], coord[2]) == reduce_corner(settlement_coord[0], settlement_coord[1], settlement_coord[2])
        )
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

    fn can_upgrade_to_city(&self, r: usize, q: usize, corner: usize, color: PlayerColor) -> bool {
        self.structure_is_color(r, q, corner, color)
        && self.structures[r][q][corner].unwrap().structure_type == StructureType::Settlement
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

    fn give_resources(&self, players: &mut Vec<Player>, roll: usize) {
        for [r, q] in BOARD_COORDS {
            if [r, q] == self.robber {
                continue;
            }
            if let Some(hex) = self.hexes[r][q] {
                if hex.number == roll {
                    for corner in self.structures[r][q] {
                        if let Some(s) = corner {
                            let player = players.iter().position(|p| p.color == s.color).unwrap();
                            players[player].hand[hex.resource] += match s.structure_type {
                                StructureType::Settlement => 1,
                                StructureType::City => 2
                            };
                        }
                    }
                }
            }
        }
        // new_cards
    }

    fn draw_dv_card<R: Rng + ?Sized>(&mut self, rng: &mut R) -> DVCard {
        self.dv_bank.discard_random(rng)
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

const DIRS: [[isize; 2]; 6] = [
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

fn hexes_touched(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 2]> {
    let mut neighbors = vec![[r, q]];

    let neighbor1 = [(r as isize + DIRS[corner][0]) as usize, (q as isize + DIRS[corner][1]) as usize];
    if is_on_board(neighbor1[0], neighbor1[1]) {
        neighbors.push(neighbor1);
    }
    let neighbor2 = [(r as isize + DIRS[(corner + 1) % 6][0]) as usize, (q as isize + DIRS[(corner + 1) % 6][1]) as usize];
    if is_on_board(neighbor2[0], neighbor2[1]) {
        neighbors.push(neighbor2);
    }
    neighbors.into_iter()
}

fn get_dup_corners(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut dups = vec![[r, q, corner]];
    let neighbor1 = [(r as isize + DIRS[corner][0]) as usize, (q as isize + DIRS[corner][1]) as usize];
    if is_on_board(neighbor1[0], neighbor1[1]) {
        dups.push([neighbor1[0], neighbor1[1], (corner + 2) % 6]);
    }
    let neighbor2 = [(r as isize + DIRS[(corner + 1) % 6][0]) as usize, (q as isize + DIRS[(corner + 1) % 6][1]) as usize];
    if is_on_board(neighbor2[0], neighbor2[1]) {
        dups.push([neighbor2[0], neighbor2[1], (corner + 4) % 6]);
    }
    dups.into_iter()
}

fn get_dup_edges(r: usize, q: usize, edge: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut dups = vec![[r, q, edge]];
    let neighbor = [(r as isize + DIRS[edge][0]) as usize, (q as isize + DIRS[edge][1]) as usize];
    if is_on_board(neighbor[0], neighbor[1]) {
        dups.push([neighbor[0], neighbor[1], (edge + 3) % 6]);
    }
    dups.into_iter()
}

fn corner_corner_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut neighbors = vec![[r, q, (corner + 5) % 6], [r, q, (corner + 1) % 6]];

    let hex_neighbor1 = [(r as isize + DIRS[corner][0]) as usize, (q as isize + DIRS[corner][1]) as usize];
    let hex_neighbor2 = [(r as isize + DIRS[(corner + 1) % 6][0]) as usize, (q as isize + DIRS[(corner + 1) % 6][1]) as usize];
    if is_on_board(hex_neighbor1[0], hex_neighbor1[1]) {
        neighbors.push([hex_neighbor1[0], hex_neighbor1[1], (corner + 1) % 6]);
    } else if is_on_board(hex_neighbor2[0], hex_neighbor2[1]) {
        neighbors.push([hex_neighbor2[0], hex_neighbor2[1], (corner + 5) % 6]);
    }
    neighbors.into_iter()
}

fn edge_edge_neighbors(r: usize, q: usize, edge: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut neighbors = vec![[r, q, (edge + 5) % 6], [r, q, (edge + 1) % 6]];
    let full_neighbor = [(r as isize + DIRS[edge][0]) as usize, (q as isize + DIRS[edge][1]) as usize];
    let half_neighbor_l = [(r as isize + DIRS[(edge + 5) % 6][0]) as usize, (q as isize + DIRS[(edge + 5) % 6][1]) as usize];
    let half_neighbor_r = [(r as isize + DIRS[(edge + 1) % 6][0]) as usize, (q as isize + DIRS[(edge + 1) % 6][1]) as usize];
    if is_on_board(full_neighbor[0], full_neighbor[1]) {
        neighbors.push([full_neighbor[0], full_neighbor[1], (edge + 2) % 6]);
        neighbors.push([full_neighbor[0], full_neighbor[1], (edge + 4) % 6]);
    }
    else {
        if is_on_board(half_neighbor_l[0], half_neighbor_l[1]) {
            neighbors.push([half_neighbor_l[0], half_neighbor_l[1], (edge + 1) % 6]);
        }
        if is_on_board(half_neighbor_r[0], half_neighbor_r[1]) {
            neighbors.push([half_neighbor_r[0], half_neighbor_r[1], (edge + 5) % 6]);
        }
    }
    neighbors.into_iter()
}

fn corner_edge_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut neighbors = vec![[r, q, corner], [r, q, (corner + 1) % 6]];

    let hex_neighbor1 = [(r as isize + DIRS[corner][0]) as usize, (q as isize + DIRS[corner][1]) as usize];
    let hex_neighbor2 = [(r as isize + DIRS[(corner + 1) % 6][0]) as usize, (q as isize + DIRS[(corner + 1) % 6][1]) as usize];
    if is_on_board(hex_neighbor1[0], hex_neighbor1[1]) {
        neighbors.push([hex_neighbor1[0], hex_neighbor1[1], (corner + 2) % 6]);
    } else if is_on_board(hex_neighbor2[0], hex_neighbor2[1]) {
        neighbors.push([hex_neighbor2[0], hex_neighbor2[1], (corner + 5) % 6]);
    }
    neighbors.into_iter()
}

fn edge_corner_neighbors(r: usize, q: usize, edge: usize) -> [[usize; 3]; 2] {
    [[r, q, edge], [r, q, (edge + 5) % 6]]
}

fn intersecting_corner(edge1: [usize; 3], edge2: [usize; 3]) -> Option<[usize; 3]> {
    edge_corner_neighbors(edge1[0], edge1[1], edge1[2]).into_iter()
    .flat_map(|[r, q, c]| get_dup_corners(r, q, c))
    .find(|&c1|
        edge_corner_neighbors(edge2[0], edge2[1], edge2[2]).into_iter().any(|c2| c1 == c2)
    )
}