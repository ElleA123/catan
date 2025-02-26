use macroquad::prelude::*;

use crate::game::{
    Board, DVCard, Hex, Player, PlayerColor, Port, Resource, StructureType,
    CORNER_COORDS, DV_CARDS, EDGE_COORDS, HEX_COORDS, RESOURCES
};
use crate::screen_coords::ScreenCoords;
use crate::{Action, GameState, Selector, SetupState};

const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

fn render_background(coords: &ScreenCoords) {
    let [hand_x, hand_y, hand_w, hand_h] = coords.hand_zone;
    let [menu_x, menu_y, menu_w, menu_h] = coords.menu_zone;
    let [info_x, info_y, info_w, info_h] = coords.info_zone;

    clear_background(BLUE);
    draw_rectangle(hand_x, hand_y, hand_w, hand_h, BEIGE);
    draw_rectangle(menu_x, menu_y, menu_w, menu_h, BEIGE);
    draw_rectangle(info_x, info_y, info_w, info_h, WHITE);
}

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
    } else {
        0.25 * hex_size
    };
    
    let x2 = x + (x - x1);
    let y2 = y + (y - y1);

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
    let cards = &coords.cards;
    let size = &coords.card_size;
    
    let hand = player.get_hand();
    let all_dvs = player.get_combined_dvs();

    let mut card_idx = 0;
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

fn render_menu(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    let buttons = &coords.buttons;
    let size = coords.button_size;

    let can_click = state.get_available_actions(color);
    let labels = [
        "Devel",
        "Road",
        "Settl",
        "City",
        "Pass"
    ];
    for i in 0..buttons.len() {
        render_button(buttons[i], size, can_click[i], labels[i]);
    }
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
        Some(roll) => &roll.to_string(),
        None => "?"
    };
    draw_text(label, text_x, text_y, font_size, BLACK);
}

fn render_dice(coords: &ScreenCoords, state: &GameState) {
    let dice = &coords.dice;
    let size = coords.dice_size;
    let rolls = match state.roll {
        Some([r1, r2]) => [Some(r1), Some(r2)],
        None => [None, None]
    };

    render_die(dice[0], size, rolls[0]);
    render_die(dice[1], size, rolls[1]);
}

fn render_info_box(coords: &ScreenCoords, player: &Player) {
    let &[x, y, width, height] = &coords.info_zone;
    let vps = player.get_vps().to_string();

    draw_rectangle(x, y, width, height, player.get_color().into());
    draw_text(vps.as_str(), x, y + height, 40.0, BLACK);
}

fn render_clickable(pos: [f32; 2], radius: f32, alpha: u8) {
    let thickness = radius / 5.0;
    let color = Color::from_rgba(192, 192, 192, alpha);
    draw_circle(pos[0], pos[1], radius, color);
    draw_circle_lines(pos[0], pos[1], radius, thickness, DARKGRAY);
}

fn render_choosing_victim(coords: &ScreenCoords, state: &GameState) {
    let corners = &coords.corners;
    let radius = coords.build_clickable_radius;
    let alpha = 0;
    for idx in 0..CORNER_COORDS.len() {
        let corner = CORNER_COORDS[idx];
        if state.board.is_robbable(corner, state.get_current_color()) {
            let pos = corners[idx];
            render_clickable(pos, radius, alpha);
        }
    }
}

fn render_moving_robber(coords: &ScreenCoords, state: &GameState) {
    let centers = &coords.centers;
    let robber = state.board.robber;

    let radius = coords.robber_clickable_radius;
    let alpha = 0;
    for idx in 0..HEX_COORDS.len() {
        if HEX_COORDS[idx] == robber {
            continue;
        }
        let pos = centers[idx];
        render_clickable(pos, radius, alpha);
    }
}

fn render_building_road(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    let edges = &coords.edges;
    let board = &state.board;

    let radius = coords.build_clickable_radius;
    let alpha = 192;

    for idx in 0..EDGE_COORDS.len() {
        if board.can_place_road(EDGE_COORDS[idx], color) {
            let pos = edges[idx];
            render_clickable(pos, radius, alpha);
        }
    }
}

fn render_building_settlement(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    let corners = &coords.corners;
    let board = &state.board;

    let radius = coords.build_clickable_radius;
    let alpha = 192;

    for idx in 0..CORNER_COORDS.len() {
        if board.can_place_settlement(CORNER_COORDS[idx], color) {
            let pos = corners[idx];
            render_clickable(pos, radius, alpha);
        }
    }
}

fn render_building_city(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    let corners = &coords.corners;
    let board = &state.board;

    let radius = coords.city_clickable_radius;

    for idx in 0..CORNER_COORDS.len() {
        if board.can_place_city(CORNER_COORDS[idx], color) {
            let [x, y] = corners[idx];
            draw_circle(x, y, radius, DARKGRAY);
        }
    }
}

fn render_selector_bg(coords: &ScreenCoords) {
    let [x, y, w, h] = coords.selector_zone;
    draw_rectangle(x, y, w, h, BEIGE);
}

fn render_selector_bottom(coords: &ScreenCoords, selector: &Selector) {
    let cards = &coords.selector_bottom_cards;
    let selectors = &coords.selector_bottom_selectors;
    let size = &coords.selector_card_size;
    let selector_size = coords.selector_selector_size;
    let hand = selector.get_bottom();

    for idx in 0..RESOURCES.len() {
        let res = RESOURCES[idx];
        if hand[res] > 0 {
            render_resource(&cards[idx], size, res, hand[res].to_string().as_str());
        }
        render_selector_selector(&selectors[idx], selector_size, res);
    }
}

fn render_selector_selector(pos: &[f32; 2], size: f32, resource: Resource) {
    let &[x, y] = pos;
    let thickness = size / 14.0;

    draw_rectangle(x, y, size, size, resource.into());
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
}

fn render_selector_top(coords: &ScreenCoords, selector: &Selector) {
    let cards = &coords.selector_top_cards;
    let selectors = &coords.selector_top_selectors;
    let card_size = &coords.selector_card_size;
    let selector_size = coords.selector_selector_size;
    let hand = selector.get_top().unwrap();

    for idx in 0..RESOURCES.len() {
        let res = RESOURCES[idx];
        if hand[res] > 0 {
            render_resource(&cards[idx], card_size, res, hand[res].to_string().as_str());
        }
        render_selector_selector(&selectors[idx], selector_size, res);
    }
}

fn render_confirm(coords: &ScreenCoords, state: &GameState) {
    let &[x, y] = &coords.selector_buttons[1];
    let size = coords.selector_button_size;
    let thickness = coords.hex_size / 20.0;

    let text_x = x + 0.25 * size;
    let text_y = y + 0.7 * size;
    let font_size = size;

    let color = if state.can_execute_selector() {WHITE} else {GRAY};

    draw_rectangle(x, y, size, size, color);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text("C", text_x, text_y, font_size, BLACK);
}

fn render_cancel(coords: &ScreenCoords) {
    let &[x, y] = &coords.selector_buttons[0];
    let size = coords.selector_button_size;
    let thickness = coords.hex_size / 20.0;

    let text_x = x + 0.25 * size;
    let text_y = y + 0.7 * size;
    let font_size = size;

    draw_rectangle(x, y, size, size, WHITE);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text("X", text_x, text_y, font_size, BLACK);
}

fn render_selector(coords: &ScreenCoords, state: &GameState) {
    let selector = state.get_selector();
    
    render_selector_bg(coords);
    render_selector_bottom(coords, selector);
    match selector {
        Selector::Trading(_, _) => render_selector_top(coords, selector),
        _ => ()
    };
    
    render_confirm(coords, state);
    match selector {
        Selector::Discarding(_) => (),
        _ => render_cancel(coords)
    };
}

fn render_trade_button(coords: &ScreenCoords) {
    let &[x, y] = &coords.trade_button;
    let size = coords.trade_button_size;

    let thickness = size / 20.0;
    let text_x = x + 0.07 * size;
    let text_y = y + 0.6 * size;
    let font_size = size / 2.5;

    draw_rectangle(x, y, size, size, BEIGE);
    draw_rectangle_lines(x, y, size, size, thickness, BLACK);
    draw_text("Trade", text_x, text_y, font_size, BLACK);
}

fn render_state_dependents(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    if !state.is_players_turn(color) {
        return;
    }

    if state.selector.is_some() {
        render_selector(coords, state);
    } else {
        render_trade_button(coords);
    }

    match state.action {
        Action::ChoosingVictim => render_choosing_victim(coords, state),
        Action::MovingRobber => render_moving_robber(coords, state),
        Action::BuildingRoad => render_building_road(coords, state, color),
        Action::BuildingSettlement => render_building_settlement(coords, state, color),
        Action::BuildingCity => render_building_city(coords, state, color),
        _ => ()
    }
}

pub fn render_screen(coords: &ScreenCoords, state: &GameState, color: PlayerColor) {
    let board = &state.board;
    let player = state.get_player(color).unwrap();

    render_background(coords);
    render_board(coords, board);
    render_hand(coords, player);
    render_menu(coords, state, color);
    render_dice(coords, state);
    render_info_box(coords, player);
    render_state_dependents(coords, state, color);
}

fn render_setup_menu(coords: &ScreenCoords) {
    let buttons = &coords.buttons;
    let size = coords.button_size;

    let labels = [
        "Devel",
        "Road",
        "Settl",
        "City",
        "Pass"
    ];
    for i in 0..buttons.len() {
        render_button(buttons[i], size, false, labels[i]);
    }
}

fn render_building_road_setup(coords: &ScreenCoords, state: &SetupState, settlement: [usize; 3]) {
    let edges = &coords.edges;
    let board = &state.board;

    let radius = coords.build_clickable_radius;
    let alpha = 192;

    for idx in 0..EDGE_COORDS.len() {
        if board.can_place_setup_road(EDGE_COORDS[idx], settlement) {
            let pos = edges[idx];
            render_clickable(pos, radius, alpha);
        }
    }
}

fn render_building_settlement_setup(coords: &ScreenCoords, state: &SetupState) {
    let corners = &coords.corners;
    let board = &state.board;

    let radius = coords.build_clickable_radius;
    let alpha = 192;

    for idx in 0..CORNER_COORDS.len() {
        if board.can_place_setup_settlement(CORNER_COORDS[idx]) {
            let pos = corners[idx];
            render_clickable(pos, radius, alpha);
        }
    }
}

fn render_setup_state_dependents(coords: &ScreenCoords, state: &SetupState, color: PlayerColor) {
    if !state.is_players_turn(color) {
        return;
    }

    match state.settlement {
        Some(settlement) => render_building_road_setup(coords, state, settlement),
        None => render_building_settlement_setup(coords, state)
    }
}

pub fn render_setup_screen(coords: &ScreenCoords, state: &SetupState, color: PlayerColor) {
    render_background(coords);
    render_board(coords, &state.board);
    render_hand(coords, state.get_current_player());
    render_info_box(coords, state.get_current_player());
    render_setup_menu(coords);
    render_setup_state_dependents(coords, state, color);
}