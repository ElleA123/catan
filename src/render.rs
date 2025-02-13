use macroquad::prelude::*;

use crate::{get_dup_corners, get_dup_edges, Board, Hex, Port, StructureType};
use crate::{BOARD_COORDS, PORT_COORDS};

const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

pub struct ActionPoints {
    centers: [[f32; 2]; 19],
    corners: [[f32; 2]; 54],
    edges: [[f32; 2]; 72]
}

fn get_centers(width: f32, height: f32, scale: f32) -> [[f32; 2]; 19] {
    let q_shift: f32 = scale * SQRT_3;
    let r_shift_x: f32 = scale * 0.5 * SQRT_3;
    let r_shift_y: f32 = scale * 1.5;

    let start_x: f32 = 0.5 * width - (2.0 * q_shift + 2.0 * r_shift_x);
    let start_y: f32 = 0.5 * height - (2.0 * r_shift_y);

    let mut centers = [[0.0; 2]; 19];
    for idx in 0..BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[idx];
        let x = start_x + q_shift * q as f32 + r_shift_x * r as f32;
        let y = start_y + r_shift_y * r as f32;
        centers[idx] = [x, y];
    }
    centers
}

fn get_corners(centers: &[[f32; 2]; 19], scale: f32) -> [[f32; 2]; 54] {
    let mut seen_corners = Vec::with_capacity(114);
    let mut corners = [[0.0; 2]; 54];
    let mut corner_idx = 0;
    for center_idx in 0..BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[center_idx];
        for n in 0..6 {
            if !seen_corners.contains(&[r, q, n]) {
                corners[corner_idx] = get_corner_coords(centers[center_idx], n, scale);
                seen_corners.extend(get_dup_corners(r, q, n));
                corner_idx += 1;
            }
        }
    }
    corners
}

fn get_edges(centers: &[[f32; 2]; 19], scale: f32) -> [[f32; 2]; 72] {
    let mut seen_edges = Vec::with_capacity(114);
    let mut edges = [[0.0; 2]; 72];
    let mut edge_idx = 0;
    for center_idx in 0..BOARD_COORDS.len() {
        let [r, q] = BOARD_COORDS[center_idx];
        for n in 0..6 {
            if !seen_edges.contains(&[r, q, n]) {
                let [x1, y1] = get_corner_coords(centers[center_idx], n, scale);
                let [x2, y2] = get_corner_coords(centers[center_idx], (n + 5) % 6, scale);
                edges[edge_idx] = [(x1 + x2) / 2.0, (y1 + y2) / 2.0];

                seen_edges.extend(get_dup_edges(r, q, n));
                edge_idx += 1;
            }
        }
    }
    edges
}

fn get_ports(width: f32, height: f32, scale: f32) -> [[f32; 3]; 9] {
    let stretch_factor = 1.5;
    let q_shift: f32 = scale * SQRT_3;
    let r_shift_x: f32 = scale * 0.5 * SQRT_3;
    let r_shift_y: f32 = scale * 1.5;

    let start_x: f32 = 0.5 * width - (2.0 * q_shift + 2.0 * r_shift_x);
    let start_y: f32 = 0.5 * height - (2.0 * r_shift_y);

    let mut coords = [[0.0; 3]; 9];
    for idx in 0..PORT_COORDS.len() {
        let [r, q, e] = PORT_COORDS[idx];
        let x = start_x + q_shift * q as f32 + r_shift_x * r as f32
        + stretch_factor * match e {
            0 => -0.25 * SQRT_3 * scale,
            1 => 0.25 * SQRT_3 * scale,
            2 => 0.5 * SQRT_3 * scale,
            3 => 0.25 * SQRT_3 * scale,
            4 => -0.25 * SQRT_3 * scale,
            5 => -0.5 * SQRT_3 * scale,
            _ => panic!("render::get_ports(): invalid edge")
        };
        let y = start_y + r_shift_y * r as f32
        + stretch_factor * match e {
            0 => -0.75 * scale,
            1 => -0.75 * scale,
            2 => 0.0,
            3 => 0.75 * scale,
            4 => 0.75 * scale,
            5 => 0.0,
            _ => panic!("render::get_ports(): invalid edge")
        };
        coords[idx] = [x, y, (e * 60 + 15) as f32];
    }
    coords
}

fn render_hex(center: [f32; 2], hex: Hex, scale: f32, ) {
    let radius = scale;
    let hex_thickness = scale / 20.0;
    let circle_radius = scale / 2.0;
    let circle_thickness = scale / 30.0;
    let num_offset = scale / 4.0;
    let num_font_size = scale;

    let [x, y] = center;
    let color = hex.resource.into();
    let num_color = if hex.number == 6 || hex.number == 8 {MAROON} else {BLACK};
    let digit_offset = if hex.number >= 10 {num_offset * 0.8} else {0.0};

    draw_poly(x, y, 6, radius, 30.0, color);
    draw_poly_lines(x, y, 6, radius, 30.0, hex_thickness, BLACK);
    draw_circle(x, y, circle_radius, BEIGE);
    draw_circle_lines(x, y, circle_radius, circle_thickness, BLACK);
    draw_text(hex.number.to_string().as_str(), x - num_offset - digit_offset, y + num_offset, num_font_size, num_color);
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
        Port::Three => WHITE,
        Port::Two(res) => res.into()
    };

    draw_poly(x, y, 4, radius, rotation, color);
    draw_poly_lines(x, y, 4, radius, rotation, thickness, BLACK);
}

fn render_ports(board: &Board, ports: &[[f32; 3]; 9], scale: f32) {
    for idx in 0..ports.len() {
        render_port(ports[idx], board.ports[idx], scale);
    }
}

fn get_corner_coords(center: [f32; 2], corner: usize, scale: f32) -> [f32; 2] {
    [
        center[0] + match corner {
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
        }
    ]
}

fn render_settlement(center: [f32; 2], corner: usize, color: Color, scale: f32) {
    let base = scale / 2.5;
    let height = scale / 3.0;
    let thickness = scale / 20.0;

    let [mut x, mut y] = get_corner_coords(center, corner, scale);
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

    let [mut x, mut y] = get_corner_coords(center, corner, scale);
    x -= 0.5 * base;
    y -= 0.5 * height;

    let v1 = Vec2::new(x + 0.5 * base, y - 0.5 * height);
    let v2 = Vec2::new(x + base, y - 0.5 * height);
    let v3 = Vec2::new(x + 0.75 * base, y - height);
    
    draw_rectangle(x, y, base, height, color);
    draw_rectangle_lines(x, y, base, height, thickness, BLACK);

    draw_rectangle(x + 0.5 * base, y - 0.5 * height, 0.5 * base, 0.5 * height, color);
    draw_rectangle_lines(x + 0.5 * base, y - 0.5 * height, 0.5 * base, 0.5 * height, thickness, color);

    draw_triangle_lines(v1, v2, v3, thickness, BLACK);
    draw_triangle(v1, v2, v3, color);
}

fn render_road(center: [f32; 2], edge: usize, color: Color, scale: f32) {
    let thickness = scale / 15.0;

    let [x1, y1] = get_corner_coords(center, edge, scale);
    let [x2, y2] = get_corner_coords(center, (edge + 5) % 6, scale);

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

pub fn render_board(board: &Board) -> ActionPoints {
    let width = screen_width();
    let height = screen_height();
    let scale = 0.09 * if width > height {height} else {width};
    let centers = get_centers(width, height, scale);
    let corners = get_corners(&centers, scale);
    let edges = get_edges(&centers, scale);
    let ports = get_ports(width, height, scale);

    clear_background(BLUE);
    render_hexes(board, &centers, scale);
    render_ports(board, &ports, scale);
    render_roads(board, &centers, scale);
    render_structures(board, &centers, scale);

    let action_points = ActionPoints {
        centers,
        corners,
        edges
    };
    return action_points;
}