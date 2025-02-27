use std::{collections::HashSet, ops::{Index, IndexMut}};
use rand::{seq::{IndexedRandom, SliceRandom}, Rng};

//// Typedefs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn into_label(self) -> String {
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
    pub fn new() -> ResHand {
        ResHand([0; 5])
    }

    pub fn from_monopoly(resource: Resource, count: usize) -> ResHand {
        let mut  hand = ResHand::new();
        hand[resource] = count;
        hand
    }

    pub fn clear(&mut self) {
        for res in RESOURCES {
            self[res] = 0;
        }
    }

    pub fn size(&self) -> usize {
        self.0.iter().sum()
    }

    pub fn count_nonzero(&self) -> usize {
        self.0.iter().filter(|&res| *res != 0).count()
    }

    pub fn nth_nonzero(&self, n: usize) -> Option<Resource> {
        RESOURCES.iter().copied().filter(|c| self[*c] > 0).nth(n)
    }

    pub fn add(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            self[res] += rhs[res];
        }
    }

    pub fn add_card(&mut self, card: Resource) {
        self[card] += 1;
    }

    pub fn can_discard(&self, rhs: ResHand) -> bool {
        RESOURCES.iter().all(|&res| self[res] >= rhs[res])
    }

    pub fn discard(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            self[res] -= rhs[res];
        }
    }

    pub fn discard_max(&mut self, rhs: ResHand) {
        for res in RESOURCES {
            if self[res] >= rhs[res] {
                self[res] -= rhs[res];
            } else {
                self[res] = 0;
            }
        }
    }

    pub fn discard_all(&mut self, resource: Resource) {
        self[resource] = 0;
    }

    pub fn discard_random<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<Resource> {
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
    pub fn new() -> DVHand {
        DVHand([0; 5])
    }

    pub fn clear(&mut self) {
        for dv in DV_CARDS {
            self[dv] = 0;
        }
    }

    pub fn size(&self) -> usize {
        self.0.iter().sum()
    }

    pub fn count_nonzero(&self) -> usize {
        self.0.iter().filter(|&dv| *dv != 0).count()
    }

    pub fn nth_nonzero(&self, n: usize) -> Option<DVCard> {
        DV_CARDS.iter().copied().filter(|c| self[*c] > 0).nth(n)
    }

    pub fn add(&mut self, rhs: DVHand) {
        for dv in DV_CARDS {
            self[dv] += rhs[dv];
        }
    }

    pub fn add_card(&mut self, card: DVCard) {
        self[card] += 1;
    }

    pub fn can_discard_card(&self, card: DVCard) -> bool {
        self[card] != 0
    }

    pub fn discard_card(&mut self, card: DVCard) {
        self[card] -= 1;
    }

    pub fn discard_random<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<DVCard> {
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
    pub structure_type: StructureType,
    pub color: PlayerColor
}

#[derive(Debug, Clone, Copy)]
pub struct Hex {
    pub resource: Resource,
    pub number: usize
}

#[derive(Debug, Clone, Copy)]
pub enum Port {
    ThreeForOne,
    TwoForOne(Resource)
}

#[derive(Debug)]
pub struct Board {
    pub hexes: [[Option<Hex>; 5]; 5],
    pub ports: [Port; 9],
    pub structures: [[[Option<Structure>; 6]; 5]; 5],
    pub roads: [[[Option<PlayerColor>; 6]; 5]; 5],
    pub robber: [usize; 2],
    pub bank: ResHand,
    pub dv_bank: DVHand,
}

pub const HEX_COORDS: [[usize; 2]; 19] = [
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
    while hex < HEX_COORDS.len() {
        let [r, q] = HEX_COORDS[hex];
        let mut c = 0;
        while c < 6 {
            let [r_, q_, c_] = reduce_corner([r, q, c]);
            if r == r_ && q == q_ && c == c_ {
                corners[idx] = [r, q, c];
                idx += 1;
            }
            c += 1;
        } 
        hex += 1;
    }
    corners
}

const fn edge_coords() -> [[usize; 3]; 72] {
    let mut edges = [[0; 3]; 72];
    let mut idx = 0;

    let mut hex = 0;
    while hex < HEX_COORDS.len() {
        let [r, q] = HEX_COORDS[hex];
        let mut e = 0;
        while e < 6 {
            let [r_, q_, e_] = reduce_edge([r, q, e]);
            if r == r_ && q == q_ && e == e_ {
                edges[idx] = [r, q, e];
                idx += 1;
            }
            e += 1;
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
    pub fn new<R: Rng + ?Sized>(num_players: usize, rng: &mut R) -> Self {
        let mut hexes: [[Option<Hex>; 5]; 5] = [[None; 5]; 5];
        let structures: [[[Option<Structure>; 6]; 5]; 5] = [[[None; 6]; 5]; 5];
        let roads: [[[Option<PlayerColor>; 6]; 5]; 5] = [[[None; 6]; 5]; 5];
        let robber = *HEX_COORDS.choose(rng).unwrap();

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
        for [r, q] in HEX_COORDS {
            // Check for desert
            if robber != [r, q] {
                // No sixes or eights next to each other
                if numbers[i] == 6 || numbers[i] == 8 {
                    for dir in [[0, -1], [-1, 0], [-1, 1]] {
                        let test_r = (r as isize + dir[0]) as usize;
                        let test_q = (q as isize + dir[1]) as usize;
                        if is_on_board([test_r, test_q]) {
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
            hexes,
            ports,
            structures,
            roads,
            robber,
            bank: STARTING_BANK_HAND,
            dv_bank: STARTING_DV_BANK_HAND
        }
    }

    fn road_is_color(&self, edge: [usize; 3], color: PlayerColor) -> bool {
        let [r, q, e] = edge;
        match self.roads[r][q][e] {
            Some(c) => c == color,
            None => false
        }
    }

    pub fn structure_exists(&self, corner: [usize; 3]) -> bool {
        let [r, q, c] = corner;
        self.structures[r][q][c].is_some()
    }

    pub fn structure_is_color(&self, corner: [usize; 3], color: PlayerColor) -> bool {
        let [r, q, c] = corner;
        match self.structures[r][q][c] {
            Some(structure) => structure.color == color,
            None => false
        }
    }

    pub fn structure_isnt_color(&self, corner: [usize; 3], color: PlayerColor) -> bool {
        let [r, q, c] = corner;
        match self.structures[r][q][c] {
            Some(c) => c.color != color,
            None => false
        }
    }

    pub fn is_robbable(&self, corner: [usize; 3], robber: PlayerColor) -> bool {
        hexes_touched(corner).any(|hex| hex == self.robber)
        && self.structure_exists(corner)
        && self.structure_isnt_color(corner, robber)
    }

    pub fn get_colors_on_hex(&self, hex: [usize; 2]) -> HashSet<PlayerColor> {
        let [r, q] = hex;
        (0..6).into_iter().filter_map(|c| self.structures[r][q][c]).map(|s| s.color).collect()
    }

    pub fn can_place_road(&self, edge: [usize; 3], color: PlayerColor) -> bool {
        let [r, q, e] = edge;

        self.roads[r][q][e].is_none()
        && (
            edge_corner_neighbors(edge).any(|corner| self.structure_is_color(corner, color))
            || edge_edge_neighbors(edge).any(|neighbor_edge| {
                let int_corner = intersecting_corner(edge, neighbor_edge).unwrap();
                self.road_is_color(neighbor_edge, color) && !self.structure_isnt_color(int_corner, color)
            })
        )
    }

    pub fn can_place_setup_road(&self, edge: [usize; 3], settlement_coord: [usize; 3]) -> bool {
        let [r, q, e] = edge;

        self.roads[r][q][e].is_none()
        && edge_corner_neighbors(edge).any(
            |neighbor_corner| reduce_corner(neighbor_corner) == reduce_corner(settlement_coord)
        )
    }

    pub fn can_place_settlement(&self, corner: [usize; 3], color: PlayerColor) -> bool {
        let [r, q, c] = corner;

        self.structures[r][q][c].is_none()
        && corner_corner_neighbors(corner).all(
            |[r_, q_, c_]| self.structures[r_][q_][c_].is_none()
        )
        && corner_edge_neighbors(corner).any(|neighbor_edge| self.road_is_color(neighbor_edge, color))
    }

    pub fn can_place_setup_settlement(&self, corner: [usize; 3]) -> bool {
        let [r, q, c] = corner;

        self.structures[r][q][c].is_none()
        && corner_corner_neighbors(corner).all(
            |[r_, q_, c_]| self.structures[r_][q_][c_].is_none()
        )
    }

    pub fn can_place_city(&self, corner: [usize; 3], color: PlayerColor) -> bool {
        let [r, q, c] = corner;
        self.structure_is_color(corner, color)
        && self.structures[r][q][c].unwrap().structure_type == StructureType::Settlement
    }

    pub fn can_place_any_road(&self, color: PlayerColor) -> bool {
        EDGE_COORDS.iter().any(|&edge| self.can_place_road(edge, color))
    }

    pub fn can_place_any_settlement(&self, color: PlayerColor) -> bool {
        CORNER_COORDS.iter().any(|&corner| self.can_place_settlement(corner, color))
    }

    pub fn can_place_any_city(&self, color: PlayerColor) -> bool {
        CORNER_COORDS.iter().any(|&corner| self.can_place_city(corner, color))
    }

    pub fn place_road(&mut self, edge: [usize; 3], color: PlayerColor) {
        self.bank.add(ROAD_HAND);
        for [r, q, e] in get_dup_edges(edge) {
            self.roads[r][q][e] = Some(color);
        }
    }

    pub fn place_settlement(&mut self, corner: [usize; 3], color: PlayerColor) {
        self.bank.add(SETTLEMENT_HAND);
        for [r, q, c] in get_dup_corners(corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::Settlement,
                color
            });
        }
    }

    pub fn place_city(&mut self, corner: [usize; 3], color: PlayerColor) {
        self.bank.add(CITY_HAND);
        for [r, q, c] in get_dup_corners(corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::City,
                color
            });
        }
    }

    pub fn place_setup_road(&mut self, edge: [usize; 3], color: PlayerColor) {
        for [r, q, e] in get_dup_edges(edge) {
            self.roads[r][q][e] = Some(color);
        }
    }

    pub fn place_setup_settlement(&mut self, corner: [usize; 3], color: PlayerColor) {
        for [r, q, c] in get_dup_corners(corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::Settlement,
                color
            });
        }
    }

    pub fn can_draw_dv_card(&self) -> bool {
        self.dv_bank.size() > 0
    }

    pub fn draw_dv_card<R: Rng + ?Sized>(&mut self, rng: &mut R) -> DVCard {
        self.bank.add(DV_CARD_HAND);
        self.dv_bank.discard_random(rng).unwrap()
    }

    pub fn get_starting_resources(&self, corner: [usize; 3]) -> ResHand {
        let mut hand = ResHand::new();
        for [r, q] in hexes_touched(corner) {
            if let Some(hex) = self.hexes[r][q] {
                hand.add_card(hex.resource);
            }
        }
        hand
    }

    pub fn get_new_resources(&self, players: Vec<PlayerColor>, roll: usize) -> Vec<ResHand> {
        let mut new_cards = Vec::with_capacity(players.len());
        for _ in 0..players.len() {
            new_cards.push(ResHand::new());
        }

        for [r, q] in HEX_COORDS {
            if [r, q] == self.robber || self.hexes[r][q].is_none() {
                continue;
            }

            let hex = self.hexes[r][q].unwrap();
            if hex.number == roll {
                for corner in self.structures[r][q] {
                    if let Some(s) = corner {
                        let idx = players.iter().position(|&color| s.color == color).unwrap();
                        new_cards[idx].add_card(hex.resource);
                        if s.structure_type == StructureType::City {
                            new_cards[idx].add_card(hex.resource);
                        }
                    }
                }
            }
        }
        new_cards
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

const fn is_on_board(hex: [usize; 2]) -> bool {
    let [r, q] = hex;
    r < 5 && q < 5 && r + q >= 2 && r + q <= 6
}

const fn reduce_corner(corner: [usize; 3]) -> [usize; 3] {
    let [r, q, c] = corner;
    match c {
        0 => if r != 0 && is_on_board([r - 1, q]) {
            [r - 1, q, 2]
        } else if r != 0 && is_on_board([r - 1, q + 1]) {
            [r - 1, q + 1, 4]
        } else {
            [r, q, 0]
        },
        1 => if r != 0 && is_on_board([r - 1, q + 1]) {
            [r - 1, q + 1, 3]
        } else {
            [r, q, 1]
        },
        2 => [r, q, 2],
        3 => [r, q, 3],
        4 => if q != 0 && is_on_board([r, q - 1]) {
            [r, q - 1, 2]
        } else {
            [r, q, 4]
        },
        5 => if r != 0 && is_on_board([r - 1, q]) {
            [r - 1, q, 3]
        } else if q != 0 && is_on_board([r, q - 1]) {
            [r, q - 1, 1]
        } else {
            [r, q, 5]
        },
        _ => panic!("main::reduce_corner(): invalid corner")
    }
}

const fn reduce_edge(edge: [usize; 3]) -> [usize; 3] {
    let [r, q, e] = edge;
    match e {
        0 => if r != 0 && is_on_board([r - 1, q]) {
            [r - 1, q, 3]
        } else {
            [r, q, 0]
        },
        1 => if r != 0 && is_on_board([r - 1, q + 1]) {
            [r - 1, q + 1, 4]
        } else {
            [r, q, 1]
        },
        2 => [r, q, 2],
        3 => [r, q, 3],
        4 => [r, q, 4],
        5 => if q != 0 && is_on_board([r, q - 1]) {
            [r, q - 1, 2]
        } else {
            [r, q, 5]
        },
        _ => panic!("main::reduce_edge(): invalid edge")
    }
}

fn hexes_touched(corner: [usize; 3]) -> impl Iterator<Item = [usize; 2]> {
    let [r, q, c] = corner;
    let mut neighbors = vec![[r, q]];

    let neighbor1 = [(r as isize + DIRS[c][0]) as usize, (q as isize + DIRS[c][1]) as usize];
    if is_on_board(neighbor1) {
        neighbors.push(neighbor1);
    }
    let neighbor2 = [(r as isize + DIRS[(c + 1) % 6][0]) as usize, (q as isize + DIRS[(c + 1) % 6][1]) as usize];
    if is_on_board(neighbor2) {
        neighbors.push(neighbor2);
    }
    neighbors.into_iter()
}

fn get_dup_corners(corner: [usize; 3]) -> impl Iterator<Item = [usize; 3]> {
    let [r, q, c] = corner;
    let mut dups = vec![[r, q, c]];
    let neighbor1 = [(r as isize + DIRS[c][0]) as usize, (q as isize + DIRS[c][1]) as usize];
    if is_on_board(neighbor1) {
        dups.push([neighbor1[0], neighbor1[1], (c + 2) % 6]);
    }
    let neighbor2 = [(r as isize + DIRS[(c + 1) % 6][0]) as usize, (q as isize + DIRS[(c + 1) % 6][1]) as usize];
    if is_on_board(neighbor2) {
        dups.push([neighbor2[0], neighbor2[1], (c + 4) % 6]);
    }
    dups.into_iter()
}

fn get_dup_edges(edge: [usize; 3]) -> impl Iterator<Item = [usize; 3]> {
    let [r, q, e] = edge;
    let mut dups = vec![[r, q, e]];
    let neighbor = [(r as isize + DIRS[e][0]) as usize, (q as isize + DIRS[e][1]) as usize];
    if is_on_board(neighbor) {
        dups.push([neighbor[0], neighbor[1], (e + 3) % 6]);
    }
    dups.into_iter()
}

fn corner_corner_neighbors(corner: [usize; 3]) -> impl Iterator<Item = [usize; 3]> {
    let [r, q, c] = corner;
    let mut neighbors = vec![[r, q, (c + 5) % 6], [r, q, (c + 1) % 6]];

    let hex_neighbor1 = [(r as isize + DIRS[c][0]) as usize, (q as isize + DIRS[c][1]) as usize];
    let hex_neighbor2 = [(r as isize + DIRS[(c + 1) % 6][0]) as usize, (q as isize + DIRS[(c + 1) % 6][1]) as usize];
    if is_on_board(hex_neighbor1) {
        neighbors.push([hex_neighbor1[0], hex_neighbor1[1], (c + 1) % 6]);
    } else if is_on_board(hex_neighbor2) {
        neighbors.push([hex_neighbor2[0], hex_neighbor2[1], (c + 5) % 6]);
    }
    neighbors.into_iter()
}

fn edge_edge_neighbors(edge: [usize; 3]) -> impl Iterator<Item = [usize; 3]> {
    let [r, q, e] = edge;
    let mut neighbors = vec![[r, q, (e + 5) % 6], [r, q, (e + 1) % 6]];
    let full_neighbor = [(r as isize + DIRS[e][0]) as usize, (q as isize + DIRS[e][1]) as usize];
    let half_neighbor_l = [(r as isize + DIRS[(e + 5) % 6][0]) as usize, (q as isize + DIRS[(e + 5) % 6][1]) as usize];
    let half_neighbor_r = [(r as isize + DIRS[(e + 1) % 6][0]) as usize, (q as isize + DIRS[(e + 1) % 6][1]) as usize];
    if is_on_board(full_neighbor) {
        neighbors.push([full_neighbor[0], full_neighbor[1], (e + 2) % 6]);
        neighbors.push([full_neighbor[0], full_neighbor[1], (e + 4) % 6]);
    }
    else {
        if is_on_board(half_neighbor_l) {
            neighbors.push([half_neighbor_l[0], half_neighbor_l[1], (e + 1) % 6]);
        }
        if is_on_board(half_neighbor_r) {
            neighbors.push([half_neighbor_r[0], half_neighbor_r[1], (e + 5) % 6]);
        }
    }
    neighbors.into_iter()
}

fn corner_edge_neighbors(corner: [usize; 3]) -> impl Iterator<Item = [usize; 3]> {
    let [r, q, c] = corner;
    let mut neighbors = vec![[r, q, c], [r, q, (c + 1) % 6]];

    let hex_neighbor1 = [(r as isize + DIRS[c][0]) as usize, (q as isize + DIRS[c][1]) as usize];
    let hex_neighbor2 = [(r as isize + DIRS[(c + 1) % 6][0]) as usize, (q as isize + DIRS[(c + 1) % 6][1]) as usize];
    if is_on_board(hex_neighbor1) {
        neighbors.push([hex_neighbor1[0], hex_neighbor1[1], (c + 2) % 6]);
    } else if is_on_board(hex_neighbor2) {
        neighbors.push([hex_neighbor2[0], hex_neighbor2[1], (c + 5) % 6]);
    }
    neighbors.into_iter()
}

fn edge_corner_neighbors(edge: [usize; 3]) -> impl Iterator<Item = [usize; 3]> {
    let [r, q, e] = edge;
    [[r, q, e], [r, q, (e + 5) % 6]].into_iter()
}

fn intersecting_corner(edge1: [usize; 3], edge2: [usize; 3]) -> Option<[usize; 3]> {
    edge_corner_neighbors(edge1)
    .flat_map(|neighbor_corner| get_dup_corners(neighbor_corner))
    .find(|&c1|
        edge_corner_neighbors(edge2).any(|c2| c1 == c2)
    )
}

pub struct Player {
    color: PlayerColor,
    is_human: bool,
    base_vps: usize,
    hand: ResHand,
    dvs: DVHand,
    new_dvs: DVHand,
    knights: usize,
    largest_army: bool,
    road_len: usize,
    longest_road: bool,
    road_pool: usize,
    settlement_pool: usize,
    city_pool: usize,
}

impl Player {
    pub fn new(color: PlayerColor, is_human: bool) -> Player {
        Player {
            color,
            is_human,
            base_vps: 0,
            hand: ResHand::new(),
            dvs: DVHand::new(),
            new_dvs: DVHand::new(),
            knights: 0,
            largest_army: false,
            road_len: 0,
            longest_road: false,
            road_pool: 15,
            settlement_pool: 5,
            city_pool: 4,
        }
    }

    pub fn get_color(&self) -> PlayerColor {
        self.color
    }

    pub fn get_vps(&self) -> usize {
        self.base_vps
        + self.dvs[DVCard::VictoryPoint]
        + if self.largest_army {2} else {0}
        + if self.longest_road {2} else {0}
    }

    pub fn get_hand(&self) -> ResHand {
        self.hand
    }

    pub fn get_dvs(&self) -> DVHand {
        self.dvs
    }

    pub fn get_new_dvs(&self) -> DVHand {
        self.new_dvs
    }

    pub fn get_combined_dvs(&self) -> DVHand {
        let mut combined = self.dvs;
        combined.add(self.new_dvs);
        combined
    }

    pub fn get_knights(&self) -> usize {
        self.knights
    }

    pub fn set_largest_army(&mut self, value: bool) {
        self.largest_army = value;
    }

    pub fn set_longest_road(&mut self, value: bool) {
        self.longest_road = value;
    }

    pub fn set_road_len(&mut self, value: usize) {
        self.road_len = value;
    }

    pub fn is_color(&self, color: PlayerColor) -> bool {
        self.color == color
    }

    pub fn has_won(&self) -> bool {
        self.get_vps() >= 10
    }

    pub fn must_discard(&self) -> bool {
        self.hand.size() > 7
    }

    pub fn get_cards(&mut self, new: ResHand) {
        self.hand.add(new);
    }

    pub fn get_card(&mut self, new: Resource) {
        self.hand.add_card(new);
    }

    pub fn discard_cards(&mut self, lost: ResHand) {
        self.hand.discard(lost);
    }

    pub fn discard_all(&mut self, lost: Resource) {
        self.hand.discard_all(lost);
    }

    pub fn discard_random_card<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<Resource> {
        self.hand.discard_random(rng)
    }

    pub fn can_buy_dv(&self) -> bool {
        self.hand.can_discard(DV_CARD_HAND)
    }

    pub fn can_build_road(&self) -> bool {
        self.hand.can_discard(ROAD_HAND) && self.road_pool > 0
    }

    pub fn can_build_settlement(&self) -> bool {
        self.hand.can_discard(SETTLEMENT_HAND) && self.settlement_pool > 0
    }

    pub fn can_build_city(&self) -> bool {
        self.hand.can_discard(CITY_HAND) && self.city_pool > 0
    }

    pub fn buy_dv(&mut self, dv: DVCard) {
        self.hand.discard(DV_CARD_HAND);
        self.new_dvs[dv] += 1;
    }

    pub fn build_road(&mut self) {
        self.hand.discard(ROAD_HAND);
        self.road_pool -= 1;
    }

    pub fn build_settlement(&mut self) {
        self.hand.discard(SETTLEMENT_HAND);
        self.base_vps += 1;
        self.settlement_pool -= 1;
    }

    pub fn place_setup_road(&mut self) {
        self.road_pool -= 1;
    }

    pub fn place_setup_settlement(&mut self) {
        self.base_vps += 1;
        self.settlement_pool -= 1;
    }

    pub fn build_city(&mut self) {
        self.hand.discard(CITY_HAND);
        self.base_vps += 1;
        self.city_pool -= 1;
        self.settlement_pool += 1;
    }

    pub fn play_dv_card(&mut self, card: DVCard) {
        self.dvs[card] -= 1;
        if card == DVCard::Knight {
            self.knights += 1;
        }
    }

    pub fn cycle_dvs(&mut self) {
        self.dvs.add(self.new_dvs);
        self.new_dvs.clear();
    }
}