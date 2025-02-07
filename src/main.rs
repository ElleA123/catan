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

const STARTING_BANK_HAND: Hand = Hand([19, 19, 19, 19, 19]);
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

    fn disc(&mut self, rhs: Hand) {
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

    fn can_place_road(&mut self, r: usize, q: usize, edge: usize) -> bool {
        self.structures[r][q][edge].is_none()
    }

    // fn place_road_safe(&mut self, r: usize, q: usize, edge: usize, color: Color) -> bool {
    //     if self.can_place_road(r, q, edge) {
    //         for (r, q, e) in get_dup_edges(r, q, edge) {
    //             self.roads[r][q][e] = Some(color);
    //         }
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn place_road(&mut self, r: usize, q: usize, edge: usize, color: Color) {
        for (r, q, e) in get_dup_edges(r, q, edge) {
            self.roads[r][q][e] = Some(color);
        }
    }

    fn can_place_settlement(&mut self, r: usize, q: usize, corner: usize) -> bool {
        // TODO: add distance reqs
        self.structures[r][q][corner].is_none()
    }

    // fn place_settlement_safe(&mut self, r: usize, q: usize, corner: usize, color: Color) -> bool {
    //     if self.can_place_settlement(r, q, corner) {
    //         for (r, q, c) in get_dup_corners(r, q, corner) {
    //             self.structures[r][q][c] = Some(Structure {
    //                 structure_type: StructureType::Settlement,
    //                 color
    //             });
    //         }
    //         true
    //     } else {
    //         false
    //     }
    // }

    fn place_settlement(&mut self, r: usize, q: usize, corner: usize, color: Color) {
        for (r, q, c) in get_dup_corners(r, q, corner) {
            self.structures[r][q][c] = Some(Structure {
                structure_type: StructureType::Settlement,
                color
            });
        }
    }

    fn upgrade_to_city(&mut self, r: usize, q: usize, corner: usize) -> bool {
        if let Some(s) = self.structures[r][q][corner] {
            if s.structure_type == StructureType::Settlement {
                let color = s.color;
                for (r, q, c) in get_dup_corners(r, q, corner) {
                    self.structures[r][q][c] = Some(Structure {
                        structure_type: StructureType::City,
                        color
                    });
                }
                return true;
            }
        }
        return false;
    }

    fn upgrade_to_city_unchecked(&mut self, r: usize, q: usize, corner: usize) {
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

struct Player<'a> {
    color: Color,
    board: &'a mut Board,
    vps: usize,
    hand: Hand,
    dvs: Hand,
    new_dvs: Hand,
    knights: usize,
    road_pool: usize,
    settlement_pool: usize,
    city_pool: usize,
}

impl Player<'_> {
    fn new(color: Color, board: &mut Board) -> Player {
        Player {
            color,
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
        self.board.bank.disc_max(got);
    }

    fn disc_resources(&mut self, lost: Hand) -> bool {
        if self.hand.can_disc(lost) {
            self.hand.disc(lost);
            self.board.bank.add(lost);
            true
        } else {
            false
        }
    }

    fn build_road(&mut self, r: usize, q: usize, edge: usize) -> bool {
        if self.board.can_place_road(r, q, edge) && self.hand.can_disc(ROAD_HAND) && self.road_pool > 0 {
            self.disc_resources(ROAD_HAND);
            self.board.place_road(r, q, edge, self.color);
            self.road_pool -= 1;
            true
        } else {
            false
        }
    }

    fn build_settlement(&mut self, r: usize, q: usize, corner: usize) -> bool {
        // println!("works on board: {}", board.can_place_settlement(r, q, corner));
        // println!("has cards: {}", self.hand.can_disc(SETTLEMENT_HAND));
        // println!("settlements available: {}", self.settlement_pool > 0);
        if self.board.can_place_settlement(r, q, corner) && self.hand.can_disc(SETTLEMENT_HAND) && self.settlement_pool > 0 {
            self.disc_resources(SETTLEMENT_HAND);
            self.board.place_settlement(r, q, corner, self.color);
            self.settlement_pool -= 1;
            true
        } else {
            false
        }
    }

    fn upgrade_to_city(&mut self, r: usize, q: usize, corner: usize) -> bool {
        if let Some(s) = self.board.structures[r][q][corner]  {
            if s.structure_type == StructureType::Settlement && s.color == self.color && self.hand.can_disc(CITY_HAND) {
                self.disc_resources(CITY_HAND);
                self.board.upgrade_to_city(r, q, corner);
                self.city_pool -= 1;
                self.settlement_pool += 1;
                return true;
            }
        }
        return false;
    }

    fn buy_dv_card(&mut self, r: usize, q: usize, corner: usize) -> bool {
        if self.hand.can_disc(DV_CARD_HAND) && self.board.dv_bank.len() > 0 {
            self.disc_resources(DV_CARD_HAND);
            self.new_dvs.add(Hand::from_card(self.board.draw_dv_card() as usize));
            true
        } else {
            false
        }
    }
}

//// Coordinate manipulation

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

fn neighbors(r: usize, q: usize) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::new();
    for dir in DIRS {
        if is_on_board(
            (r as isize + dir.0) as usize,
            (q as isize + dir.1) as usize
        ) {
            neighbors.push((
                (r as isize + dir.0) as usize,
                (q as isize + dir.1) as usize)
            );
        }
    }
    neighbors
}

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
        dups.push((neighbor.0, neighbor.1, edge + 3));
    }
    dups
}

fn play_game(num_players: usize) {
    let mut rng = rand::rng();
    let mut board = Board::new(num_players, &mut rng);
    let mut players = Vec::with_capacity(num_players);
    for i in 0..num_players {
        players.push(Player::new(Color::from(i), &mut board));
    }
}

fn main() {
    

    let mut board = Board::new(4, &mut rng);
    println!("{}", board);

    let mut player = Player::new(Color::Red, &mut board);
    player.hand.add(Hand([11, 11, 11, 11, 0]));
    println!("Built? {}", player.build_settlement(2, 2, 0));
    println!("Built? {}", player.build_settlement(2, 2, 0));
    println!("Upgraded? {}", player.upgrade_to_city(2, 2, 0));

    println!("{:?}\n{:?}", player.hand, board.structures);
}