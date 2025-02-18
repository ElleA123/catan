use std::{ops::{Index, IndexMut}, thread::current};
use rand::{seq::{IndexedRandom, SliceRandom}, Rng};

pub mod render;
use render::{render_screen, ClickablePoints};

//// Typedefs
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerColor {
    Red=0,
    Blue=1,
    Orange=2,
    White=3
}

const PLAYER_COLORS: [PlayerColor; 4] = [
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

    fn can_place_setup_road(&self, r: usize, q: usize, edge: usize, settlement_coord: [usize; 3], color: PlayerColor) -> bool {
        self.roads[r][q][edge].is_none()
        && edge_corner_neighbors(r, q, edge).into_iter().any(|coord| coord == settlement_coord)
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

fn corner_corner_neighbors(r: usize, q: usize, corner: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut neighbors = vec![[r, q, (corner + 5) % 6], [r, q, (corner + 1) % 6]];

    let hex_neighbor1 = [(r as isize + COORD_DIRS[corner][0]) as usize, (q as isize + COORD_DIRS[corner][1]) as usize];
    let hex_neighbor2 = [(r as isize + COORD_DIRS[(corner + 1) % 6][0]) as usize, (q as isize + COORD_DIRS[(corner + 1) % 6][1]) as usize];
    if is_on_board(hex_neighbor1[0], hex_neighbor1[1]) {
        neighbors.push([hex_neighbor1[0], hex_neighbor1[1], (corner + 1) % 6]);
    } else if is_on_board(hex_neighbor2[0], hex_neighbor2[1]) {
        neighbors.push([hex_neighbor2[0], hex_neighbor2[1], (corner + 5) % 6]);
    }
    neighbors.into_iter()
}

fn edge_edge_neighbors(r: usize, q: usize, edge: usize) -> impl Iterator<Item = [usize; 3]> {
    let mut neighbors = vec![[r, q, (edge + 5) % 6], [r, q, (edge + 1) % 6]];
    let full_neighbor = [(r as isize + COORD_DIRS[edge][0]) as usize, (q as isize + COORD_DIRS[edge][1]) as usize];
    let half_neighbor_l = [(r as isize + COORD_DIRS[(edge + 5) % 6][0]) as usize, (q as isize + COORD_DIRS[(edge + 5) % 6][1]) as usize];
    let half_neighbor_r = [(r as isize + COORD_DIRS[(edge + 1) % 6][0]) as usize, (q as isize + COORD_DIRS[(edge + 1) % 6][1]) as usize];
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

    let hex_neighbor1 = [(r as isize + COORD_DIRS[corner][0]) as usize, (q as isize + COORD_DIRS[corner][1]) as usize];
    let hex_neighbor2 = [(r as isize + COORD_DIRS[(corner + 1) % 6][0]) as usize, (q as isize + COORD_DIRS[(corner + 1) % 6][1]) as usize];
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

fn get_roll<R: Rng + ?Sized>(rng: &mut R) -> usize {
    rng.random_range(1..=6) + rng.random_range(1..=6)
}

struct Player {
    color: PlayerColor,
    is_human: bool,
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
    fn new(color: PlayerColor) -> Player {
        Player {
            color,
            is_human: true,
            vps: 0,
            hand: ResHand::new(),
            dvs: DVHand::new(),
            new_dvs: DVHand::new(),
            knights: 0,
            road_len: 0,
            road_pool: 15,
            settlement_pool: 5,
            city_pool: 4,
        }
    }

    fn can_buy_dv(&self) -> bool {
        self.hand.can_disc(DV_CARD_HAND)
    }

    fn buy_dv(&mut self, dv: DVCard) {
        self.hand.discard(DV_CARD_HAND);
        self.new_dvs[dv] += 1;
    }

    fn can_build_road(&self) -> bool {
        self.hand.can_disc(ROAD_HAND)
    }

    fn can_build_settlement(&self) -> bool {
        self.hand.can_disc(SETTLEMENT_HAND)
    }

    fn can_upgrade_to_city(&self) -> bool {
        self.hand.can_disc(CITY_HAND)
    }
}

struct Players(Vec<Player>);

impl Players {
    fn new(num_players: usize) -> Players {
        let mut players = Vec::with_capacity(num_players);
        for i in 0..num_players {
            players.push(Player::new(PlayerColor::from(i)));
        }
        Players(players)
    }
}

impl Index<PlayerColor> for Players {
    type Output = Player;
    fn index(&self, index: PlayerColor) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<PlayerColor> for Players {
    fn index_mut(&mut self, index: PlayerColor) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

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
    offering_trade: bool,
    offered_trades: Vec<(ResHand, ResHand)>,
    passed_turn: bool,
}

impl TurnState {
    fn pass_turn(&mut self, new_player: PlayerColor) {
        self.player = new_player;
        self.action = Action::Idling;
        self.roll = None;
        self.played_dv = false;
        self.offering_trade = false;
        self.passed_turn = false;
    }
}

fn mouse_is_on_circle(mouse_pos: (f32, f32), center: &[f32; 2], radius: f32) -> bool {
    (mouse_pos.0 - center[0]).powi(2) + (mouse_pos.1 - center[1]).powi(2) <= radius.powi(2)
}

fn mouse_is_on_square(mouse_pos: (f32, f32), pos: [f32; 2], size: f32) -> bool {
    mouse_pos.0 > pos[0] && mouse_pos.0 < pos[0] + size
    && mouse_pos.1 > pos[1] && mouse_pos.1 < pos[1] + size
}

fn handle_idle_click(clickables: &ClickablePoints, mouse_pos: (f32, f32), player: &mut Player, board: &mut Board, state: &mut TurnState) {
    let maybe_menu_id = clickables.buttons.iter().position(|&pos| mouse_is_on_square(mouse_pos, pos, clickables.button_size));
    if let Some(id) = maybe_menu_id {
        match id {
            0 => {
                if player.can_buy_dv() {
                    player.buy_dv(board.draw_dv_card());
                }
            },
            1 => {
                if player.can_build_road() {
                    state.action = Action::BuildingRoad;
                }
            },
            2 => {
                if player.can_build_settlement() {
                    state.action = Action::BuildingSettlement;
                }
            },
            3 => {
                if player.can_upgrade_to_city() {
                    state.action = Action::UpgradingToCity;
                }
            },
            4 => {
                state.passed_turn = true;
            },
            _ => panic!("handle_idle_click(): illegal menu button")
        }
    }

}

fn handle_road_click(clickables: &ClickablePoints, mouse_pos: (f32, f32), player: &mut Player, board: &mut Board, state: &mut TurnState) {
    if mouse_is_on_square(mouse_pos, clickables.buttons[1], clickables.button_size) {
        state.action = Action::Idling;
        return
    }
    let radius = 0.2 * clickables.board_scale;
    let color = state.player;
    let maybe_idx = clickables.edges.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let [r, q, edge] = EDGE_COORDS[idx];
        if board.can_place_road(r, q, edge, color) {
            player.hand.discard(ROAD_HAND);
            board.place_road(r, q, edge, color);
            state.action = Action::Idling;
        }
    }
}

fn handle_structure_click(structure_type: StructureType, clickables: &ClickablePoints, mouse_pos: (f32, f32), player: &mut Player, board: &mut Board, state: &mut TurnState) {
    let cancel_button = if structure_type == StructureType::Settlement {clickables.buttons[2]} else {clickables.buttons[3]};
    if mouse_is_on_square(mouse_pos, cancel_button, clickables.button_size) {
        state.action = Action::Idling;
        return
    }
    let radius = 0.2 * clickables.board_scale;
    let color = state.player;
    let maybe_idx = clickables.corners.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let [r, q, corner] = CORNER_COORDS[idx];
        if structure_type == StructureType::Settlement
        && board.can_place_settlement(r, q, corner, color) {
            player.hand.discard(SETTLEMENT_HAND);
            board.place_settlement(r, q, corner, color);
            state.action = Action::Idling;
        }
        else if board.can_upgrade_to_city(r, q, corner, color) {
            player.hand.discard(CITY_HAND);
            board.upgrade_to_city(r, q, corner);
            state.action = Action::Idling;
        }
    }
}

fn handle_click(clickables: &ClickablePoints, player: &mut Player, board: &mut Board, state: &mut TurnState) {
    let mouse_pos = macroquad::input::mouse_position();
    match state.action {
        Action::Idling => handle_idle_click(clickables, mouse_pos, player, board, state),
        Action::Discarding(_) => (),
        Action::MovingRobber => (),
        Action::BuildingRoad => handle_road_click(clickables, mouse_pos, player, board, state),
        Action::BuildingSettlement => handle_structure_click(StructureType::Settlement, clickables, mouse_pos, player, board, state),
        Action::UpgradingToCity => handle_structure_click(StructureType::City, clickables, mouse_pos, player, board, state),
    }
}

#[macroquad::main("Catan")]
async fn main() {
    let mut rng = rand::rng();
    let num_players = 4;
    let mut board = Board::new(num_players, &mut rng);

    board.place_settlement(2, 2, 3, PlayerColor::Orange);
    board.place_settlement(2, 2, 0, PlayerColor::Blue);
    board.place_road(2, 2, 0, PlayerColor::Blue);
    board.place_road(2, 2, 5, PlayerColor::Blue);
    board.place_road(1, 2, 4, PlayerColor::Blue);
    board.place_road(2, 1, 3, PlayerColor::Blue);

    let mut players = Players::new(num_players);
    
    let order: Vec<PlayerColor> = PLAYER_COLORS.choose_multiple(&mut rng, num_players).copied().collect();
    let mut current_player = 0;

    let mut state = TurnState {
        player: PlayerColor::Orange,
        action: Action::Idling,
        roll: Some([3, 4]),
        played_dv: false,
        offering_trade: false,
        offered_trades: Vec::new(),
        passed_turn: false,
    };

    players[PlayerColor::Orange].hand.add(ROAD_HAND);
    players[PlayerColor::Orange].hand.add(ROAD_HAND);
    players[PlayerColor::Orange].hand.add(SETTLEMENT_HAND);
    players[PlayerColor::Orange].dvs.add(DVHand([1, 1, 1, 1, 1]));

    let mut clickables;
    loop {
        if state.passed_turn {
            current_player = (current_player + 1) % 4;
            state.pass_turn(order[current_player]);
        }

        let player = &mut players[state.player];

        clickables = render_screen(&board, &player.hand, &player.dvs, &state);
        if macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
            handle_click(&clickables, player, &mut board, &mut state);
        }
        macroquad::window::next_frame().await
    }
}