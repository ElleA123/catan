use macroquad::prelude::*;

use crate::{Action, Board, DVCard, DVHand, GameState, Hex, PlayerColor, Port, ResHand, Resource, StructureType};
use crate::{BOARD_COORDS, PORT_COORDS, CORNER_COORDS, EDGE_COORDS, RESOURCES, DV_CARDS, DV_CARD_HAND, ROAD_HAND, SETTLEMENT_HAND, CITY_HAND};

const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

struct Zone {
    x: f32,
    y: f32,
    width: f32,
    height: f32
}

impl Zone {
    fn new(
        screen_width: f32, screen_height: f32, x_factor: f32, y_factor: f32, width_factor: f32, height_factor: f32
    ) -> Zone {
        Zone {
            x: screen_width * x_factor,
            y: screen_height * y_factor,
            width: screen_width * width_factor,
            height: screen_height * height_factor
        }
    }
}

struct BoardPoints {
    centers: [[f32; 2]; 19],
    corners: [[f32; 2]; 54],
    edges: [[f32; 2]; 72],
    board_scale: f32
}

struct HandPoints {
    cards: [[f32; 2]; 10],
    card_size: [f32; 2],
    num_cards: usize,
}

struct MenuPoints {
    buttons: [[f32; 2]; 5],
    button_size: f32
}

struct DicePoints {
    dice: [[f32; 2]; 2],
    dice_size: f32,
}

pub struct ClickablePoints {
    pub centers: [[f32; 2]; 19],
    pub corners: [[f32; 2]; 54],
    pub edges: [[f32; 2]; 72],
    pub board_scale: f32,
    pub cards: [[f32; 2]; 10],
    pub card_size: [f32; 2],
    pub num_cards: usize,
    pub buttons: [[f32; 2]; 5],
    pub button_size: f32,
    pub dice: [[f32; 2]; 2],
    pub dice_size: f32,
}

impl ClickablePoints {
    fn new(board_points: BoardPoints, hand_points: HandPoints, menu_points: MenuPoints, dice_points: DicePoints) -> ClickablePoints {
        ClickablePoints {
            centers: board_points.centers,
            corners: board_points.corners,
            edges: board_points.edges,
            board_scale: board_points.board_scale,
            cards: hand_points.cards,
            card_size: hand_points.card_size,
            num_cards: hand_points.num_cards,
            buttons: menu_points.buttons,
            button_size: menu_points.button_size,
            dice: dice_points.dice,
            dice_size: dice_points.dice_size
        }
    }
}

fn get_centers(x: f32, y: f32, width: f32, height: f32, scale: f32) -> [[f32; 2]; 19] {
    let q_shift: f32 = scale * SQRT_3;
    let r_shift_x: f32 = scale * 0.5 * SQRT_3;
    let r_shift_y: f32 = scale * 1.5;

    let start_x: f32 = x + 0.5 * width - (2.0 * q_shift + 2.0 * r_shift_x);
    let start_y: f32 = y + 0.5 * height - (2.0 * r_shift_y);

    let mut centers = [[0.0; 2]; 19];
    for idx in 0..BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[idx];
        let x_ = start_x + q_shift * q as f32 + r_shift_x * r as f32;
        let y_ = start_y + r_shift_y * r as f32;
        centers[idx] = [x_, y_];
    }
    centers
}

fn get_corner(center: [f32; 2], corner: usize, scale: f32) -> [f32; 2] {
    [center[0] + match corner {
        0 => 0.0,
        1 => 0.5 * SQRT_3 * scale,
        2 => 0.5 * SQRT_3 * scale,
        3 => 0.0,
        4 => -0.5 * SQRT_3 * scale,
        5 => -0.5 * SQRT_3 * scale,
        _ => panic!("render::render_structure(): invalid corner")
    },
    center[1] + match corner {
        0 => -scale,
        1 => -0.5 * scale,
        2 => 0.5 * scale,
        3 => scale,
        4 => 0.5 * scale,
        5 => -0.5 * scale,
        _ => panic!("render::render_structure(): invalid corner")
    }]
}

fn get_corners(centers: &[[f32; 2]; 19], scale: f32) -> [[f32; 2]; 54] {
    let mut corners = [[0.0; 2]; 54];
    for idx in 0..CORNER_COORDS.len() {
        let [r, q, c] = CORNER_COORDS[idx];
        let hex = centers[BOARD_COORDS.iter().position(|c| *c == [r, q]).unwrap()]; // prob slow
        corners[idx] = get_corner(hex, c, scale);
    }
    corners
}

// fn get_corners(centers: &[[f32; 2]; 19], scale: f32) -> [[[f32; 2]; 6]; 19] {
//     let mut corners = [[[0.0; 2]; 6]; 19];
//     for center in 0..BOARD_COORDS.len() {
//         for n in 0..6 {
//             corners[center][n] = get_corner(centers[center], n, scale);
//         }
//     }
//     corners
// }

fn get_edge(center: [f32; 2], edge: usize, scale: f32) -> [f32; 2] {
    [center[0] + match edge {
        0 => -0.25 * SQRT_3 * scale,
        1 => 0.25 * SQRT_3 * scale,
        2 => 0.5 * SQRT_3 * scale,
        3 => 0.25 * SQRT_3 * scale,
        4 => -0.25 * SQRT_3 * scale,
        5 => -0.5 * SQRT_3 * scale,
        _ => panic!("render::render_structure(): invalid corner")
    },
    center[1] + match edge {
        0 => -0.75 * scale,
        1 => -0.75 * scale,
        2 => 0.0,
        3 => 0.75 * scale,
        4 => 0.75 * scale,
        5 => 0.0,
        _ => panic!("render::render_structure(): invalid corner")
    }]
}

fn get_edges(centers: &[[f32; 2]; 19], scale: f32) -> [[f32; 2]; 72] {
    let mut edges = [[0.0; 2]; 72];
    for idx in 0..EDGE_COORDS.len() {
        let [r, q, e] = EDGE_COORDS[idx];
        let hex = centers[BOARD_COORDS.iter().position(|c| *c == [r, q]).unwrap()]; // prob slow
        edges[idx] = get_edge(hex, e, scale);
    }
    edges
}

// fn get_edges(corners: &[[[f32; 2]; 6]; 19]) -> [[[f32; 2]; 6]; 19] {
//     let mut edges = [[[0.0; 2]; 6]; 19];
//     for center in 0..BOARD_COORDS.len() {
//         for n in 0..6 {
//             let [x1, y1] = corners[center][n];
//             let [x2, y2] = corners[center][(n + 5) % 6];
//             edges[center][n] = [(x1 + y2) / 2.0, (y1 + y2) / 2.0];
//         }
//     }
//     edges
// }

fn get_ports(x: f32, y: f32, width: f32, height: f32, scale: f32) -> [[f32; 3]; 9] {
    let stretch_factor = 1.5;
    let q_shift: f32 = scale * SQRT_3;
    let r_shift_x: f32 = scale * 0.5 * SQRT_3;
    let r_shift_y: f32 = scale * 1.5;

    let start_x: f32 = x + 0.5 * width - (2.0 * q_shift + 2.0 * r_shift_x);
    let start_y: f32 = y + 0.5 * height - (2.0 * r_shift_y);

    let mut coords = [[0.0; 3]; 9];
    for idx in 0..PORT_COORDS.len() {
        let [r, q, e] = PORT_COORDS[idx];
        let x_ = start_x + q_shift * q as f32 + r_shift_x * r as f32
        + stretch_factor * match e {
            0 => -0.25 * SQRT_3 * scale,
            1 => 0.25 * SQRT_3 * scale,
            2 => 0.5 * SQRT_3 * scale,
            3 => 0.25 * SQRT_3 * scale,
            4 => -0.25 * SQRT_3 * scale,
            5 => -0.5 * SQRT_3 * scale,
            _ => panic!("render::get_ports(): invalid edge")
        };
        let y_ = start_y + r_shift_y * r as f32
        + stretch_factor * match e {
            0 => -0.75 * scale,
            1 => -0.75 * scale,
            2 => 0.0,
            3 => 0.75 * scale,
            4 => 0.75 * scale,
            5 => 0.0,
            _ => panic!("render::get_ports(): invalid edge")
        };
        coords[idx] = [x_, y_, (e * 60 + 15) as f32];
    }
    coords
}

fn render_hex(center: [f32; 2], hex: Hex, scale: f32, ) {
    let radius = scale;
    let hex_thickness = scale / 20.0;
    let circle_radius = scale / 2.0;
    let circle_thickness = scale / 30.0;
    let num_offset = 0.24 * scale;
    let font_size = scale;

    let [x, y] = center;
    let color = hex.resource.into();
    let num_color = if hex.number == 6 || hex.number == 8 {MAROON} else {BLACK};
    let digit_offset = if hex.number >= 10 {num_offset * 0.8} else {0.0};

    draw_poly(x, y, 6, radius, 30.0, color);
    draw_poly_lines(x, y, 6, radius, 30.0, hex_thickness, BLACK);
    draw_circle(x, y, circle_radius, BEIGE);
    draw_circle_lines(x, y, circle_radius, circle_thickness, BLACK);
    draw_text(hex.number.to_string().as_str(), x - num_offset - digit_offset, y + num_offset, font_size, num_color);
}

fn render_desert(center: [f32; 2], scale: f32) {
    let radius = scale;
    let thickness = scale / 20.0;

    let [x, y] = center;
    draw_poly(x, y, 6, radius, 30.0, YELLOW);
    draw_poly_lines(x, y, 6, radius, 30.0, thickness, BLACK);
}

fn render_hexes(board: &Board, centers: &[[f32; 2]; 19], scale: f32) {
    for idx in 0..19 {
        let [r, q] = BOARD_COORDS[idx];
        match board.hexes[r][q] {
            Some(hex) => render_hex(centers[idx], hex, scale),
            None => render_desert(centers[idx], scale)
        }
    }
}

fn render_port(coord: [f32; 3], port: Port, scale: f32) {
    let radius = scale / 3.0;
    let thickness = scale / 30.0;

    let [x, y, rotation] = coord;
    let color = match port {
        Port::ThreeForOne => WHITE,
        Port::TwoForOne(res) => res.into()
    };

    draw_poly(x, y, 4, radius, rotation, color);
    draw_poly_lines(x, y, 4, radius, rotation, thickness, BLACK);
}

fn render_ports(board: &Board, ports: &[[f32; 3]; 9], scale: f32) {
    for idx in 0..ports.len() {
        render_port(ports[idx], board.ports[idx], scale);
    }
}

fn render_settlement(center: [f32; 2], corner: usize, color: Color, scale: f32) {
    let base = scale / 2.5;
    let height = scale / 3.0;
    let thickness = scale / 20.0;

    let [mut x, mut y] = get_corner(center, corner, scale);
    x -= 0.5 * base;
    y -= 0.5 * height;

    let v1 = vec2(x, y);
    let v2 = vec2(x + base, y);
    let v3 = vec2(x + 0.5 * base, y - 0.5 * height);
    
    draw_rectangle(x, y, base, height, color);
    draw_rectangle_lines(x, y, base, height, thickness, BLACK);
    draw_triangle_lines(v1, v2, v3, thickness, BLACK);
    draw_triangle(v1, v2, v3, color);
}

fn render_city(center: [f32; 2], corner: usize, color: Color, scale: f32) {
    let base = scale / 2.0;
    let height = scale / 4.0;
    let thickness = scale / 20.0;

    let [mut x, mut y] = get_corner(center, corner, scale);
    x -= 0.5 * base;
    y -= 0.5 * height;

    let v1 = Vec2::new(x + 0.5 * base, y - 0.5 * height);
    let v2 = Vec2::new(x + base, y - 0.5 * height);
    let v3 = Vec2::new(x + 0.75 * base, y - height);
    
    draw_rectangle(x + 0.5 * base, y - 0.5 * height, 0.5 * base, 0.5 * height, color);
    draw_rectangle_lines(x + 0.5 * base, y - 0.5 * height, 0.5 * base, 1.5 * height, thickness, BLACK);

    draw_rectangle(x, y, base, height, color);
    draw_rectangle_lines(x, y, base, height, thickness, BLACK);

    draw_triangle_lines(v1, v2, v3, thickness, BLACK);
    draw_triangle(v1, v2, v3, color);
}

fn render_road(center: [f32; 2], edge: usize, color: Color, scale: f32) {
    let outline_thickness = scale / 7.0;
    let thickness = scale / 10.0;
    let gap = 0.1;

    let [c_x1, c_y1] = get_corner(center, edge, scale);
    let [c_x2, c_y2] = get_corner(center, (edge + 5) % 6, scale);
    let x1 = c_x1 * (1.0 - gap) + gap * c_x2;
    let x2 = c_x2 * (1.0 - gap) + gap * c_x1;
    let y1 = c_y1 * (1.0 - gap) + gap * c_y2;
    let y2 = c_y2 * (1.0 - gap) + gap * c_y1;

    draw_line(x1, y1, x2, y2, outline_thickness,BLACK);
    draw_line(x1, y1, x2, y2, thickness, color);
}

fn render_roads(board: &Board, centers: &[[f32; 2]; 19], scale: f32) {
    for idx in 0..BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[idx];
        for n in 0..6 {
            if let Some(pc) = board.roads[r][q][n] {
                render_road(centers[idx], n, pc.into(), scale)
            }
        }
    }
}

fn render_structures(board: &Board, centers: &[[f32; 2]; 19], scale: f32) {
    for idx in 0..BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[idx];
        for n in 0..6 {
            if let Some(s) = board.structures[r][q][n] {
                if s.structure_type == StructureType::Settlement {
                    render_settlement(centers[idx], n, s.color.into(), scale);
                } else {
                    render_city(centers[idx], n, s.color.into(), scale);
                }
            }
        }
    }
}

fn render_robber(hex: [usize; 2], centers: &[[f32; 2]; 19], scale: f32) {
    let thickness = scale / 20.0;
    let [x, y] = centers[BOARD_COORDS.iter().position(|coord| *coord == hex).unwrap()];

    let w1 = 0.4 * scale;
    let h1 = 0.8 * scale;
    let w2 = 0.6 * scale;
    let h2 = 0.25 * scale;

    let x1 = x - 0.5 * w1;
    let y1 = y - h1 + h2;
    let x2 = x - 0.5 * w2;
    let y2 = y + 0.5 * h2;

    draw_rectangle(x1, y1, w1, h1, GRAY);
    draw_rectangle_lines(x1, y1, w1, h1, thickness, BLACK);
    draw_rectangle(x2, y2, w2, h2, GRAY);
    draw_rectangle_lines(x2, y2, w2, h2, thickness, BLACK);
}

fn render_board(zone: Zone, board: &Board) -> BoardPoints {
    let Zone { x, y, width, height } = zone;
    let scale = 0.1 * if width > height {height} else {width};
    let centers = get_centers(x, y, width, height, scale);
    let corners = get_corners(&centers, scale);
    let edges = get_edges(&centers, scale);
    let ports = get_ports(x, y, width, height, scale);

    render_hexes(board, &centers, scale);
    render_ports(board, &ports, scale);
    render_roads(board, &centers, scale);
    render_structures(board, &centers, scale);
    render_robber(board.robber, &centers, scale);

    BoardPoints {
        centers,
        corners,
        edges,
        board_scale: scale
    }
}

fn get_cards(x: f32, y: f32, _width: f32, height: f32, scale: f32) -> [[f32; 2]; 10] {
    let shift = scale;

    let start_x = x + shift - 0.7 * scale;
    let y = y + 0.5 * height - 0.5 * scale;
    
    let mut cards = [[0.0, y]; 10];
    for i in 0..cards.len() {
        cards[i] = [start_x + i as f32 * shift, y];
    }
    cards
}

fn render_count(pos: [f32; 2], _width: f32, height: f32, count: usize) {
    let size = height / 3.0;
    let thickness = height / 20.0;
    let font_size = height / 3.0;

    let [x, y] = pos;
    let text_x = x + 0.07 * height;
    let text_y = y + 0.25 * height;
    draw_rectangle(x, y, size, size, WHITE);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text(count.to_string().as_str(), text_x, text_y, font_size, BLACK);
}

fn render_resource(pos: [f32; 2], width: f32, height: f32, resource: Resource, count: usize) {
    let thickness = height / 20.0;

    let [x, y] = pos;
    let color = resource.into();

    draw_rectangle(x, y, width, height, color);
    draw_rectangle_lines(x, y, width, height, thickness, BLACK);
    render_count(pos, width, height, count);
}

fn render_dv(pos: [f32; 2], width: f32, height: f32, dv: DVCard, count: usize) {
    let thickness = height / 20.0;
    let font_size = height / 3.0;

    let [x, y] = pos;
    let text_x = x + 0.2 * height;
    let text_y = y + 0.75 * height;

    draw_rectangle(x, y, width, height, WHITE);
    draw_rectangle_lines(x, y, width, height, thickness, BLACK);
    draw_text(dv.into_label().as_str(), text_x, text_y, font_size, BLACK);
    render_count(pos, width, height, count);
}

fn render_hand(zone: Zone, hand: &ResHand, dvs: &DVHand, new_dvs: &DVHand) -> HandPoints {
    let Zone { x, y, width, height } = zone;
    let scale = if 0.9 * height < width / 10.2 { 0.9 * height } else { width / 10.2 };
    let card_width = 0.7 * scale;
    let card_height = scale;
    let cards = get_cards(x, y, width, height, scale);

    let mut num_cards = 0;
    let mut card_idx = 0;
    draw_rectangle(x, y, width, height, BEIGE);
    for res in RESOURCES {
        if hand[res] > 0 {
            render_resource(cards[card_idx], card_width, card_height, res, hand[res]);
            num_cards += 1;
            card_idx += 1;
        }
    }
    for dv in DV_CARDS {
        if dvs[dv] + new_dvs[dv] > 0 {
            render_dv(cards[card_idx], card_width, card_height, dv, dvs[dv] + new_dvs[dv]);
            if dv != DVCard::VictoryPoint {
                num_cards += 1;
            }
            card_idx += 1;
        }
    }

    HandPoints {
        cards,
        card_size: [card_width, card_height],
        num_cards,
    }
}

fn get_buttons(x: f32, y: f32, width: f32, height: f32, scale: f32) -> [[f32; 2]; 5] {
    let shift = if scale < height {
        scale
    } else {
        scale + (width - 5.0 * scale) / 5.0
    };

    let start_x = x + shift - scale;
    let y = y + 0.5 * height - 0.5 * scale;

    let mut buttons = [[0.0; 2]; 5];
    for i in 0..buttons.len() {
        buttons[i] = [start_x + i as f32 * shift, y]
    }
    buttons
}

fn get_clickable_buttons(board: &Board, hand: &ResHand, state: &GameState) -> [bool; 5] {
    if state.roll.is_some() {
        match state.action {
            Action::Idling => [
                hand.can_disc(DV_CARD_HAND),
                hand.can_disc(ROAD_HAND) && EDGE_COORDS.iter().any(|&[r, q, e]| board.can_place_road(r, q, e, state.get_current_color())),
                hand.can_disc(SETTLEMENT_HAND) && CORNER_COORDS.iter().any(|&[r, q, c]| board.can_place_settlement(r, q, c, state.get_current_color())),
                hand.can_disc(CITY_HAND) && CORNER_COORDS.iter().any(|&[r, q, c]| board.can_upgrade_to_city(r, q, c, state.get_current_color())),
                true
            ],
            Action::BuildingRoad => [false, true, false, false, false],
            Action::BuildingSettlement => [false, false, true, false, false],
            Action::UpgradingToCity => [false, false, false, true, false],
            _ => [false, false, false, false, false]
        }
    } else {[
        false, false, false, false, false
    ]}
}

fn render_button(pos: [f32; 2], size: f32, can_click: bool, label: &str) {
    let thickness = size / 20.0;
    let font_size = size / 3.0;

    let [x, y] = pos;
    let color = if can_click {WHITE} else {LIGHTGRAY};
    let text_x = x + 0.15 * size;
    let text_y = y + 0.6 * size;

    draw_rectangle(x, y, size, size, color);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text(label, text_x, text_y, font_size, BLACK);
}

fn render_menu(zone: Zone, board: &Board, hand: &ResHand, state: &GameState) -> MenuPoints {
    let Zone { x, y, width, height } = zone;
    let scale = if height < width / 5.0 {height} else {width / 5.0};

    let buttons = get_buttons(x, y, width, height, scale);
    let can_click = get_clickable_buttons(board, hand, state);
    let labels = [
        "Devel",
        "Road",
        "Settl",
        "City",
        "Pass"
    ];

    draw_rectangle(x, y, width, height, BEIGE);
    for i in 0..buttons.len() {
        render_button(buttons[i], scale, can_click[i], labels[i]);
    }

    MenuPoints {
        buttons,
        button_size: scale
    }
}

fn get_dice(x: f32, y: f32, width: f32, height: f32, scale: f32) -> [[f32; 2]; 2] {
    let y = y + height - 1.1 * scale;
    let x1 = x + width - 2.2 * scale;
    let x2 = x + width - 1.1 * scale;
    [[x1, y], [x2, y]]
}

fn render_die(pos: [f32; 2], size: f32, roll: Option<usize>) {
    let thickness = size / 20.0;

    let [x, y] = pos;
    let color = if roll.is_none() {WHITE} else {LIGHTGRAY};
    let text_x = x + 0.25 * size;
    let text_y = y + 0.75 * size;
    let font_size = size;

    draw_rectangle(x, y, size, size, color);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);

    let label = match roll {
        Some(roll) => roll.to_string(),
        None => "?".to_owned()
    };
    draw_text(label.as_str(), text_x, text_y, font_size, BLACK);
}

fn render_dice(zone: Zone, state: &GameState) -> DicePoints {
    let Zone { x, y, width, height } = zone;
    let scale = 0.8 * if width / 2.1 < height {width / 2.1} else {height};

    let dice = get_dice(x, y, width, height, scale);
    let rolls = match state.roll {
        Some([r1, r2]) => [Some(r1), Some(r2)],
        None => [None, None]
    };

    render_die(dice[0], scale, rolls[0]);
    render_die(dice[1], scale, rolls[1]);

    DicePoints {
        dice,
        dice_size: scale
    }
}

fn render_turn_view(zone: Zone, state: &GameState) {
    let Zone { x, y, width, height } = zone;
    draw_rectangle(x, y, width, height, state.get_current_color().into());
    draw_text(state.get_current_player().vps.to_string().as_str(), x, y + height, 40.0, BLACK);
}

fn render_discarding(hand: &ResHand) {

}

fn render_clickable(pos: [f32; 2], radius: f32, alpha: u8) {
    let thickness = radius / 5.0;
    let color = Color::from_rgba(192, 192, 192, alpha);
    draw_circle(pos[0], pos[1], radius, color);
    draw_circle_lines(pos[0], pos[1], radius, thickness, DARKGRAY);
}

fn render_moving_robber(centers: &[[f32; 2]; 19], board: &Board, radius: f32) {
    let radius = 2.3 * radius;
    let alpha = 0;
    for (_, pos) in centers.iter().copied().enumerate()
        .filter(|(idx, _)| { BOARD_COORDS[*idx] != board.robber }
    ) {
        render_clickable(pos, radius, alpha);
    }
}

fn render_building_road(edges: &[[f32; 2]; 72], board: &Board, player: PlayerColor, radius: f32) {
    let alpha = 192;
    for (_, pos) in edges.iter().copied().enumerate()
        .filter(|(idx, _)| {
            let [r, q, e] = EDGE_COORDS[*idx];
            board.can_place_road(r, q, e, player)
        }
    ) {
        render_clickable(pos, radius, alpha);
    }
}

fn render_building_settlement(corners: &[[f32; 2]; 54], board: &Board, player: PlayerColor, radius: f32) {
    let alpha = 192;
    for (_, pos) in corners.iter().copied().enumerate()
        .filter(|(idx, _)| {
            let [r, q, c] = CORNER_COORDS[*idx];
            board.can_place_settlement(r, q, c, player)
        }
    ) {
        render_clickable(pos, radius, alpha);
    }
}

fn render_upgrading_to_city(corners: &[[f32; 2]; 54], board: &Board, player: PlayerColor, radius: f32) {
    let radius = 0.3 * radius;
    let alpha = 128;
    for (_, [x, y]) in corners.iter().copied().enumerate()
        .filter(|(idx, _)| {
            let [r, q, c] = CORNER_COORDS[*idx];
            board.can_upgrade_to_city(r, q, c, player)
        }
    ) {
        draw_circle(x, y, radius, DARKGRAY);
    }
}

fn render_state_dependents(screen_width: f32, screen_height: f32, board_points: &BoardPoints, state: &GameState, board: &Board) {
    let BoardPoints { centers, corners, edges, board_scale } = board_points;
    let radius = 0.2 * board_scale;
    match state.action {
        Action::Idling => (),
        Action::Discarding(hand) => render_discarding(&hand),
        Action::MovingRobber => render_moving_robber(centers, board, radius),
        Action::BuildingRoad => render_building_road(edges, board, state.get_current_color(), radius),
        Action::BuildingSettlement => render_building_settlement(corners, board, state.get_current_color(), radius),
        Action::UpgradingToCity => render_upgrading_to_city(corners, board, state.get_current_color(), radius)
    }
}

pub fn render_screen(state: &GameState) -> ClickablePoints {
    let board = &state.board;
    let player = state.get_current_player();
    let hand = &player.hand;
    let dvs = &player.dvs;
    let new_dvs = &player.new_dvs;

    let screen_width = screen_width();
    let screen_height = screen_height();

    let board_zone = Zone::new(screen_width, screen_height, 0.0, 0.0, 1.0, 0.85);
    let hand_zone = Zone::new(screen_width, screen_height, 0.0, 0.85, 0.6, 0.15);
    let menu_zone = Zone::new(screen_width, screen_height, 0.6, 0.85, 0.4, 0.15);
    let dice_zone = Zone::new(screen_width, screen_height, 0.8, 0.70, 0.2, 0.15);
    let turn_zone = Zone::new(screen_width, screen_height, 0.0, 0.0, 0.2, 0.1);

    clear_background(BLUE);
    let board_points = render_board(board_zone, board);
    let hand_points = render_hand(hand_zone, hand, dvs, new_dvs);
    let menu_points = render_menu(menu_zone, board, hand, state);
    let dice_points = render_dice(dice_zone, state);
    render_turn_view(turn_zone, state);
    render_state_dependents(screen_width, screen_height, &board_points, state, board);

    ClickablePoints::new(board_points, hand_points, menu_points, dice_points)
}