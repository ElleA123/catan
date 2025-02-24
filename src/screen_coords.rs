use crate::game::{HEX_COORDS, CORNER_COORDS, EDGE_COORDS, PORT_COORDS};
use macroquad::window::{screen_width, screen_height};

const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

const fn min(a: f32, b: f32) -> f32 {
    if a < b {a} else {b}
}

struct Zone {
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

impl Zone {
    fn new(width: f32, height: f32, x_factor: f32, y_factor: f32, width_factor: f32, height_factor: f32) -> Zone {
        Zone {
            x: width * x_factor,
            y: height * y_factor,
            width: width * width_factor,
            height: height * height_factor
        }
    }
}

pub struct ScreenCoords {
    pub centers: [[f32; 2]; 19],
    pub corners: [[f32; 2]; 54],
    pub edges: [[f32; 2]; 72],
    pub ports: [[f32; 3]; 9],
    pub hex_size: f32,
    pub cards: [[f32; 2]; 10],
    pub card_size: [f32; 2],
    pub buttons: [[f32; 2]; 5],
    pub button_size: f32,
    pub dice: [[f32; 2]; 2],
    pub dice_size: f32,
}

impl ScreenCoords {
    pub fn new() -> ScreenCoords {
        ScreenCoords {
            centers: [[0.0; 2]; 19],
            corners: [[0.0; 2]; 54],
            edges: [[0.0; 2]; 72],
            ports: [[0.0; 3]; 9],
            hex_size: 0.0,
            cards: [[0.0; 2]; 10],
            card_size: [0.0; 2],
            buttons: [[0.0; 2]; 5],
            button_size: 0.0,
            dice: [[0.0; 2]; 2],
            dice_size: 0.0,
        }
    }

    pub fn update(&mut self) {
        let width = screen_width();
        let height = screen_height();

        let board_zone = Zone::new(width, height, 0.0, 0.0, 1.0, 0.85);
        let hand_zone = Zone::new(width, height, 0.0, 0.85, 0.6, 0.15);
        let menu_zone = Zone::new(width, height, 0.6, 0.85, 0.4, 0.15);
        let dice_zone = Zone::new(width, height, 0.8, 0.70, 0.2, 0.15);
        let turn_zone = Zone::new(width, height, 0.0, 0.0, 0.2, 0.1);
        let selector_zone = Zone::new(width, height, 0.0, 0.50, 0.30, 0.35);

        self.update_board_coords(board_zone);
    }

    fn update_board_coords(&mut self, zone: Zone) {
        self.hex_size = 0.1 * min(zone.width, zone.height);

        self.update_centers(&zone);
        self.update_corners(&zone);
        self.update_edges(&zone);
        self.update_ports(&zone);
    }

    fn calculate_center(zone: &Zone, hex: &[usize; 2], hex_size: f32) -> [f32; 2] {
        let Zone {x, y, width, height} = zone;
        let [r, q] = *hex;

        let q_shift: f32 = hex_size * SQRT_3;
        let r_shift_x: f32 = hex_size * 0.5 * SQRT_3;
        let r_shift_y: f32 = hex_size * 1.5;

        let start_x: f32 = x + 0.5 * width - (2.0 * q_shift + 2.0 * r_shift_x);
        let start_y: f32 = y + 0.5 * height - (2.0 * r_shift_y);

        [start_x + q_shift * q as f32 + r_shift_x * r as f32,
        start_y + r_shift_y * r as f32]
    } 

    fn update_centers(&mut self, zone: &Zone) {
        for idx in 0..HEX_COORDS.len() {
            let pos = &HEX_COORDS[idx];
            self.centers[idx] = ScreenCoords::calculate_center(zone, &pos, self.hex_size);
        }
    }

    fn calculate_corner(zone: &Zone, corner: &[usize; 3], hex_size: f32) -> [f32; 2] {
        let [r, q, c] = corner;
        let [x, y] = ScreenCoords::calculate_center(zone, &[*r, *q], hex_size);
        [x + match c {
            0 => 0.0,
            1 => 0.5 * SQRT_3 * hex_size,
            2 => 0.5 * SQRT_3 * hex_size,
            3 => 0.0,
            4 => -0.5 * SQRT_3 * hex_size,
            5 => -0.5 * SQRT_3 * hex_size,
            _ => panic!("screen_coords::ScreenCoords::calculate_corner(): invalid corner")
        },
        y + match c {
            0 => -hex_size,
            1 => -0.5 * hex_size,
            2 => 0.5 * hex_size,
            3 => hex_size,
            4 => 0.5 * hex_size,
            5 => -0.5 * hex_size,
            _ => panic!("screen_coords::ScreenCoords::calculate_corner(): invalid corner")
        }]
    }

    fn update_corners(&mut self, zone: &Zone) {
        for idx in 0..CORNER_COORDS.len() {
            let corner = &CORNER_COORDS[idx];
            self.corners[idx] = ScreenCoords::calculate_corner(zone, corner, self.hex_size);
        }
    }

    fn calculate_edge(zone: &Zone, edge: &[usize; 3], hex_size: f32) -> [f32; 2] {
        let [r, q, e] = edge;
        let [x, y] = ScreenCoords::calculate_center(zone, &[*r, *q], hex_size);
        [x + match e {
            0 => -0.25 * SQRT_3 * hex_size,
            1 => 0.25 * SQRT_3 * hex_size,
            2 => 0.5 * SQRT_3 * hex_size,
            3 => 0.25 * SQRT_3 * hex_size,
            4 => -0.25 * SQRT_3 * hex_size,
            5 => -0.5 * SQRT_3 * hex_size,
            _ => panic!("screen_coords::ScreenCoords::calculate_edge(): invalid edge")
        },
        y + match e {
            0 => -0.75 * hex_size,
            1 => -0.75 * hex_size,
            2 => 0.0,
            3 => 0.75 * hex_size,
            4 => 0.75 * hex_size,
            5 => 0.0,
            _ => panic!("screen_coords::ScreenCoords::calculate_edge(): invalid corner")
        }]
    }

    fn update_edges(&mut self, zone: &Zone) {
        for idx in 0..EDGE_COORDS.len() {
            let edge = &EDGE_COORDS[idx];
            self.edges[idx] = ScreenCoords::calculate_edge(zone, edge, self.hex_size);
        }
    }

    fn calculate_port(zone: &Zone, port: &[usize; 3], hex_size: f32) -> [f32; 3] {
        let [r, q, e] = port;
        let [x, y] = ScreenCoords::calculate_center(zone, &[*r, *q], hex_size);
        [x + match e {
            0 => -0.25 * SQRT_3 * hex_size,
            1 => 0.25 * SQRT_3 * hex_size,
            2 => 0.5 * SQRT_3 * hex_size,
            3 => 0.25 * SQRT_3 * hex_size,
            4 => -0.25 * SQRT_3 * hex_size,
            5 => -0.5 * SQRT_3 * hex_size,
            _ => panic!("render::get_ports(): invalid edge")
        },
        y + match e {
            0 => -0.75 * hex_size,
            1 => -0.75 * hex_size,
            2 => 0.0,
            3 => 0.75 * hex_size,
            4 => 0.75 * hex_size,
            5 => 0.0,
            _ => panic!("render::get_ports(): invalid edge")
        },
        (e * 60 + 60 - 45) as f32]
    }

    fn update_ports(&mut self, zone: &Zone) {
        for idx in 0..PORT_COORDS.len() {
            let port = &PORT_COORDS[idx];
            self.ports[idx] = ScreenCoords::calculate_port(zone, port, self.hex_size);
        }
    }

    fn update_cards(&mut self, zone: Zone) {
        let Zone { x, y, width, height } = zone;

        let card_height = min(0.9 * height, width / 10.2);
        let card_width = 0.7 * card_height;
        self.card_size = [card_width, card_height];

        let shift = card_height;
        let start_x = x + shift - card_width;
        let card_y = y + 0.5 * height - 0.5 * card_height;
        
        for idx in 0..self.cards.len() {
            self.cards[idx] = [start_x + idx as f32 * shift, card_y];
        }
    }

    fn update_buttons(&mut self, zone: Zone) {
        let Zone { x, y, width, height } = zone;
        let button_size = min(height, width / 5.0);
        self.button_size = button_size;

        let shift = self.button_size + (width - 5.0 * button_size) / 5.0;
        let start_x = x + shift - button_size;
        let button_y = y + 0.5 * height - 0.5 * button_size;
    
        for idx in 0..self.buttons.len() {
            self.buttons[idx] = [start_x + idx as f32 * shift, button_y];
        }
    }

    fn update_dice(&mut self, zone: Zone) {
        let Zone { x, y, width, height } = zone;
        let dice_size = 0.4 * min(0.4 * width, 0.8 * height);
        let y = y + 0.5 * height - 0.5 * dice_size;
        let x1 = x + 0.5 * width - 1.1 * dice_size;
        let x2 = x + 0.5 * width + 0.1 * dice_size;
        self.dice = [[x1, y], [x2, y]];
    }
}

fn get_dice(x: f32, y: f32, width: f32, height: f32, scale: f32) -> [[f32; 2]; 2] {
    let y = y + height - 1.1 * scale;
    let x1 = x + width - 2.2 * scale;
    let x2 = x + width - 1.1 * scale;
    [[x1, y], [x2, y]]
}

fn get_selected_cards(x: f32, y: f32, _width: f32, height: f32, scale: f32) -> [[f32; 2]; 5] {
    let shift = scale;

    let start_x = x + shift - 0.7 * scale;
    let y = y + 0.25 * height - 0.5 * scale;
    
    let mut cards = [[0.0, y]; 5];
    for i in 0..cards.len() {
        cards[i] = [start_x + i as f32 * shift, y];
    }
    cards
}

fn get_pool_cards(x: f32, y: f32, _width: f32, height: f32, scale: f32) -> [[f32; 2]; 5] {
    let shift = scale;

    let start_x = x + shift - 0.7 * scale;
    let y = y + 0.75 * height - 0.5 * scale;
    
    let mut cards = [[0.0, y]; 5];
    for i in 0..cards.len() {
        cards[i] = [start_x + i as f32 * shift, y];
    }
    cards
}

fn get_selector_buttons(x: f32, y: f32, _width: f32, height: f32, scale: f32) -> [[f32; 2]; 5] {
    let shift = scale;

    let start_x = x + shift - 0.7 * scale;
    let y = y + 0.9 * height - 0.5 * 0.7 * scale;
    
    let mut cards = [[0.0, y]; 5];
    for i in 0..cards.len() {
        cards[i] = [start_x + i as f32 * shift, y];
    }
    cards
}