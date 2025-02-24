use macroquad::prelude::*;

use crate::game::{
    Board, DVCard, DVHand, Hex, Player, PlayerColor, Port, ResHand, Resource, StructureType,
    CITY_HAND, CORNER_COORDS, DV_CARDS, DV_CARD_HAND, EDGE_COORDS, HEX_COORDS, PORT_COORDS, RESOURCES, ROAD_HAND, SETTLEMENT_HAND
};
use crate::screen_coords::ScreenCoords;
use crate::{GameState, Action};

const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

fn render_hex(center: &[f32; 2], hex: Hex, scale: f32) {
    let radius = scale;
    let hex_thickness = scale / 20.0;
    let circle_radius = scale / 2.0;
    let circle_thickness = scale / 30.0;
    let num_offset = 0.24 * scale;
    let font_size = scale;

    let &[x, y] = center;
    let color = hex.resource.into();
    let num_color = if hex.number == 6 || hex.number == 8 {MAROON} else {BLACK};
    let digit_offset = if hex.number >= 10 {num_offset * 0.8} else {0.0};

    draw_poly(x, y, 6, radius, 30.0, color);
    draw_poly_lines(x, y, 6, radius, 30.0, hex_thickness, BLACK);
    draw_circle(x, y, circle_radius, BEIGE);
    draw_circle_lines(x, y, circle_radius, circle_thickness, BLACK);
    draw_text(hex.number.to_string().as_str(), x - num_offset - digit_offset, y + num_offset, font_size, num_color);
}

fn render_desert(center: &[f32; 2], hex_size: f32) {
    let radius = hex_size;
    let thickness = hex_size / 20.0;

    let &[x, y] = center;
    draw_poly(x, y, 6, radius, 30.0, YELLOW);
    draw_poly_lines(x, y, 6, radius, 30.0, thickness, BLACK);
}

fn render_hexes(board: &Board, centers: &[[f32; 2]; 19], hex_size: f32) {
    for idx in 0..HEX_COORDS.len() {
        let [r, q] = HEX_COORDS[idx];
        match board.hexes[r][q] {
            Some(hex) => render_hex(&centers[idx], hex, hex_size),
            None => render_desert(&centers[idx], hex_size)
        }
    }
}

fn render_port(coord: &[f32; 3], port: &Port, hex_size: f32) {
    let radius = hex_size / 3.0;
    let thickness = hex_size / 30.0;
    let stretch_factor = 0.35 * hex_size;

    let color: Color = match port {
        &Port::ThreeForOne => WHITE,
        &Port::TwoForOne(res) => res.into()
    };

    let &[mut x, mut y, rotation] = coord;
    x -= stretch_factor * f32::cos((rotation + 45.0).to_radians());
    y -= stretch_factor * f32::sin((rotation + 45.0).to_radians());

    draw_poly(x, y, 4, radius, rotation, color);
    draw_poly_lines(x, y, 4, radius, rotation, thickness, BLACK);
}

fn render_ports(board: &Board, ports: &[[f32; 3]; 9], hex_size: f32) {
    for idx in 0..ports.len() {
        render_port(&ports[idx], &board.ports[idx], hex_size);
    }
}

fn render_road(edge: &[f32; 2], e: usize, color: Color, hex_size: f32) {
    let outline_thickness = hex_size / 7.0;
    let thickness = hex_size / 10.0;
    let edge_coverage = 0.9;

    let &[x, y] = edge;
    let x1 = x + edge_coverage * if e == 2 || e == 5 {
        0.0
    } else if e == 0 || e == 3 {
        -0.25 * SQRT_3 * hex_size
    } else {
        0.25 * SQRT_3 * hex_size
    };
    let y1 = y + edge_coverage * if e == 2 || e == 5 {
        0.5 * hex_size
    } else if e == 0 || e == 3 {
        0.25 * hex_size
    } else {
        -0.25 * hex_size
    };
    
    let x2 = x - x1;
    let y2 = y - y1;

    draw_line(x1, y1, x2, y2, outline_thickness,BLACK);
    draw_line(x1, y1, x2, y2, thickness, color);
}

fn render_roads(board: &Board, edges: &[[f32; 2]; 72], hex_size: f32) {
    for idx in 0..EDGE_COORDS.len() {
        let [r, q, e] = EDGE_COORDS[idx];
        if let Some(road) = board.roads[r][q][e] {
            render_road(&edges[idx],  e, road.into(), hex_size)
        }
    }
}

fn render_settlement(corner: &[f32; 2], color: Color, hex_size: f32) {
    let base = hex_size / 2.5;
    let height = hex_size / 3.0;
    let thickness = hex_size / 20.0;

    let &[mut x, mut y] = corner;
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

fn render_city(corner: &[f32; 2], color: Color, hex_size: f32) {
    let base = hex_size / 2.0;
    let height = hex_size / 4.0;
    let thickness = hex_size / 20.0;

    let &[mut x, mut y] = corner;
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

fn render_structures(board: &Board, corners: &[[f32; 2]; 54], hex_size: f32) {
    for idx in 0..CORNER_COORDS.len() {
        let [r, q, c] = CORNER_COORDS[idx];
        if let Some(s) = board.structures[r][q][c] {
            if s.structure_type == StructureType::Settlement {
                render_settlement(&corners[idx],s.color.into(), hex_size);
            } else {
                render_city(&corners[idx],s.color.into(), hex_size);
            }
        }
    }
}

fn render_robber(hex: [usize; 2], centers: &[[f32; 2]; 19], hex_size: f32) {
    let thickness = hex_size / 20.0;
    let [x, y] = centers[HEX_COORDS.iter().position(|coord| *coord == hex).unwrap()];

    let w1 = 0.4 * hex_size;
    let h1 = 0.8 * hex_size;
    let w2 = 0.6 * hex_size;
    let h2 = 0.25 * hex_size;

    let x1 = x - 0.5 * w1;
    let y1 = y - h1 + h2;
    let x2 = x - 0.5 * w2;
    let y2 = y + 0.5 * h2;

    draw_rectangle(x1, y1, w1, h1, GRAY);
    draw_rectangle_lines(x1, y1, w1, h1, thickness, BLACK);
    draw_rectangle(x2, y2, w2, h2, GRAY);
    draw_rectangle_lines(x2, y2, w2, h2, thickness, BLACK);
}

fn render_board(coords: &ScreenCoords, board: &Board) {
    let ScreenCoords { centers, corners, edges, ports, hex_size, .. } = coords;

    render_hexes(board, centers, *hex_size);
    render_ports(board, ports, *hex_size);
    render_roads(board, edges, *hex_size);
    render_structures(board, corners, *hex_size);
    render_robber(board.robber, centers, *hex_size);
}

fn render_count(pos: &[f32; 2], _width: f32, height: f32, count: &str) {
    let size = height / 3.0;
    let thickness = height / 20.0;
    let font_size = height / 3.0;

    let &[x, y] = pos;
    let text_x = x + 0.07 * height;
    let text_y = y + 0.25 * height;
    draw_rectangle(x, y, size, size, WHITE);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text(count, text_x, text_y, font_size, BLACK);
}

fn render_resource(pos: &[f32; 2], size: &[f32; 2], resource: Resource, count: &str) {
    let &[x, y] = pos;
    let &[width, height] = size;

    let thickness = height / 20.0;
    let color = resource.into();

    draw_rectangle(x, y, width, height, color);
    draw_rectangle_lines(x, y, width, height, thickness, BLACK);
    render_count(pos, width, height, count);
}

fn render_dv(pos: &[f32; 2], size: &[f32; 2], dv: DVCard, count: &str) {
    let &[x, y] = pos;
    let &[width, height] = size;

    let thickness = height / 20.0;
    let font_size = height / 3.0;
    let text_x = x + 0.2 * height;
    let text_y = y + 0.75 * height;

    draw_rectangle(x, y, width, height, WHITE);
    draw_rectangle_lines(x, y, width, height, thickness, BLACK);
    draw_text(dv.into_label().as_str(), text_x, text_y, font_size, BLACK);
    render_count(pos, width, height, count);
}

fn render_hand(coords: &ScreenCoords, player: &Player) {
    let &[x, y, width, height] = &coords.hand_zone;
    let cards = &coords.cards;
    let size = &coords.card_size;

    let hand = player.get_hand();
    let all_dvs = player.get_combined_dvs();

    let mut card_idx = 0;
    draw_rectangle(x, y, width, height, BEIGE);
    for res in RESOURCES {
        if hand[res] > 0 {
            render_resource(&cards[card_idx], size, res, hand[res].to_string().as_str());
            card_idx += 1;
        }
    }
    for dv in DV_CARDS {
        if all_dvs[dv] > 0 {
            render_dv(&cards[card_idx], size, dv, all_dvs[dv].to_string().as_str());
            card_idx += 1;
        }
    }
}

/*
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
                hand.can_discard(DV_CARD_HAND),
                hand.can_discard(ROAD_HAND) && EDGE_COORDS.iter().any(|&edge| board.can_place_road(edge, state.get_current_player().get_color())),
                hand.can_discard(SETTLEMENT_HAND) && CORNER_COORDS.iter().any(|&corner| board.can_place_settlement(corner, state.get_current_player().get_color())),
                hand.can_discard(CITY_HAND) && CORNER_COORDS.iter().any(|&corner| board.can_upgrade_to_city(corner, state.get_current_player().get_color())),
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
    draw_rectangle(x, y, width, height, state.get_current_player().get_color().into());
    draw_text(state.get_current_player().get_vps().to_string().as_str(), x, y + height, 40.0, BLACK);
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

fn render_selector_button(pos: [f32; 2], size: f32) {
    let thickness = size / 20.0;
    let [x, y] = pos;
    draw_rectangle(x, y, size, size, WHITE);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text("v", x + 0.4 * size, y + 0.6 * size, size, BLACK);
}

fn render_selector_base(zone: Zone, selected: &ResHand, pool: Option<&ResHand>) -> ([[f32; 2]; 10], f32) {
    let Zone { x, y, width, height } = zone;
    let scale = 0.3 * height;
    let button_size = 0.7 * scale;
    let selected_cards = get_selected_cards(x, y, width, height, scale);
    let pool_cards = get_pool_cards(x, y, width, height, scale);
    let buttons = get_selector_buttons(x, y, width, height, scale);

    draw_rectangle(x, y, width, height, BEIGE);
    for resource in RESOURCES {
        if selected[resource] > 0 {
            render_resource(selected_cards[resource as usize], resource, selected[resource].to_string().as_str(), scale);
        }
    }

    if let Some(pool) = pool {
        for resource in RESOURCES {
            let remaining = pool[resource] - selected[resource];
            render_resource(pool_cards[resource as usize], resource, remaining.to_string().as_str(), scale);
        }
    } else {
        for resource in RESOURCES {
            render_resource(pool_cards[resource as usize], resource, "∞", scale);
        }
    }

    for resource in RESOURCES {
        if selected[resource] > 0 {
            render_selector_button(buttons[resource as usize], button_size);
        }
    }

    let mut clickables = [[0.0; 2]; 10];
    for i in 0..5 {
        clickables[i] = pool_cards[i];
    }
    for i in 5..10 {
        clickables[i] = buttons[i];
    }
    (clickables, button_size)
}

fn render_discarding(zone: Zone, state: &GameState) -> SelectorPoints {
    let Zone { height, .. } = zone;
    let selected = state.discarding.as_ref().unwrap();
    let (selector_buttons, size) = render_selector_base(zone, selected, Some(&state.get_current_player().get_hand()));
    SelectorPoints {
        selector_buttons,
        selector_button_size: size,
        conf_buttons: Vec::new(),
        conf_button_size: 0.25 * height
    }
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
        .filter(|(idx, _)| { HEX_COORDS[*idx] != board.robber }
    ) {
        render_clickable(pos, radius, alpha);
    }
}

fn render_building_road(edges: &[[f32; 2]; 72], board: &Board, player: PlayerColor, radius: f32) {
    let alpha = 192;
    for (_, pos) in edges.iter().copied().enumerate()
        .filter(|(idx, _)| {
            board.can_place_road(EDGE_COORDS[*idx], player)
        }
    ) {
        render_clickable(pos, radius, alpha);
    }
}

fn render_building_settlement(corners: &[[f32; 2]; 54], board: &Board, player: PlayerColor, radius: f32) {
    let alpha = 192;
    for (_, pos) in corners.iter().copied().enumerate()
        .filter(|(idx, _)| {
            board.can_place_settlement(CORNER_COORDS[*idx], player)
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
            board.can_upgrade_to_city(CORNER_COORDS[*idx], player)
        }
    ) {
        draw_circle(x, y, radius, DARKGRAY);
    }
}

fn render_state_dependents(_screen_width: f32, _screen_height: f32, board_points: &BoardPoints, state: &GameState, board: &Board, selector_zone: Zone) -> Option<SelectorPoints> {
    let BoardPoints { centers, corners, edges, board_scale } = board_points;
    let radius = 0.2 * board_scale;

    match state.action {
        Action::Idling => (),
        Action::Discarding => return Some(render_discarding(selector_zone, state)),
        Action::MovingRobber => render_moving_robber(centers, board, radius),
        Action::BuildingRoad => render_building_road(edges, board, state.get_current_player().get_color(), radius),
        Action::BuildingSettlement => render_building_settlement(corners, board, state.get_current_player().get_color(), radius),
        Action::UpgradingToCity => render_upgrading_to_city(corners, board, state.get_current_player().get_color(), radius)
    }
    return None;
}
// */

pub fn render_screen(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    let player = state.get_player(color).unwrap();
    let board = &state.board;

    clear_background(BLUE);
    render_board(coords, board);
    render_hand(coords, player);
    // render_menu(coords, state);
    // render_dice(coords, state);
    // render_turn_view(coords, state);
    // render_state_dependents(screen_width, screen_height, &board_points, state, board, selector_zone);
}