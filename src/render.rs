use macroquad::prelude::*;

use crate::{Board, StructureType, BOARD_COORDS, PORT_COORDS};
use crate::Hex;
use crate::Resource;

const SQRT_3: f32 = 1.732050807568877293527446341505872367_f32;

fn hex_centers(width: f32, height: f32, scale: f32) -> [[f32; 2]; 19] {
    let mut centers = [[0.0; 2]; 19];

    // 300x400

    let q_shift: f32 = scale * SQRT_3;
    let r_shift_x: f32 = scale * 0.5 * SQRT_3;
    let r_shift_y: f32 = scale * 1.5;

    let start_x: f32 = 0.5 * width - (2.0 * q_shift + 2.0 * r_shift_x);
    let start_y: f32 = 0.5 * height - (2.0 * r_shift_y);

    for idx in 0..BOARD_COORDS.len() {
        let (r, q) = BOARD_COORDS[idx];
        let x = start_x + q_shift * q as f32 + r_shift_x * r as f32;
        let y = start_y + r_shift_y * r as f32;
        centers[idx] = [x, y];
    }
    centers
}

// fn port_coords(width: f32, height: f32, scale: f32) -> [[f32; 2]; 9] {
//     let mut coords: [[f32; 2]; 9] = [[0.0; 2]; 9];
//     for idx in 0..PORT_COORDS.len() {
//         let (r, q) = PORT_COORDS[idx];

//     }
// }

fn render_hex(center: [f32; 2], scale: f32, hex: Hex) {
    let radius = scale;
    let hex_thickness = scale / 20.0;
    let circle_radius = scale / 2.0;
    let circle_thickness = scale / 30.0;
    let num_offset = scale / 4.0;
    let num_font_size = scale;

    let [x, y] = center;
    let color = match hex.resource {
        Resource::Wood => DARKGREEN,
        Resource::Brick => RED,
        Resource::Wheat => GOLD,
        Resource::Sheep => GREEN,
        Resource::Ore => GRAY
    };
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

fn render_hexes(board: &Board, hex_centers: &[[f32; 2]; 19], scale: f32) {
    for i in 0..hex_centers.len() {
        let (r, q) = BOARD_COORDS[i];
        match board.hexes[r][q] {
            Some(hex) => render_hex(hex_centers[i], scale, hex),
            None => render_desert(hex_centers[i], scale)
        }
    }
}

fn get_corner_coords(hex_center: [f32; 2], corner: usize, scale: f32) -> [f32; 2] {
    [
        hex_center[0] + match corner {
            0 => 0.0,
            1 => 0.5 * SQRT_3 * scale,
            2 => 0.5 * SQRT_3 * scale,
            3 => 0.0,
            4 => -0.5 * SQRT_3 * scale,
            5 => -0.5 * SQRT_3 * scale,
            _ => panic!("render::render_structure(): invalid corner")
        },
        hex_center[1] + match corner {
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

fn render_settlement(hex_center: [f32; 2], corner: usize, color: Color, scale: f32) {
    let base = scale / 2.5;
    let height = scale / 3.0;
    let thickness = scale / 20.0;

    let [mut x, mut y] = get_corner_coords(hex_center, corner, scale);
    x -= 0.5 * base;
    y -= 0.5 * height;

    let v1 = Vec2::new(x, y);
    let v2 = Vec2::new(x + base, y);
    let v3 = Vec2::new(x + 0.5 * base, y - 0.5 * height);
    
    draw_rectangle(x, y, base, height, color);
    draw_rectangle_lines(x, y, base, height, thickness, BLACK);
    draw_triangle_lines(v1, v2, v3, thickness, BLACK);
    draw_triangle(v1, v2, v3, color);
}

fn render_city(hex_center: [f32; 2], corner: usize, color: Color, scale: f32) {
    let base = scale / 2.0;
    let height = scale / 4.0;
    let thickness = scale / 20.0;

    let [mut x, mut y] = get_corner_coords(hex_center, corner, scale);
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

fn render_road(hex_center: [f32; 2], edge: usize, color: Color, scale: f32) {
    let thickness = scale / 15.0;

    let [x1, y1] = get_corner_coords(hex_center, edge, scale);
    let [x2, y2] = get_corner_coords(hex_center, (edge + 5) % 6, scale);

    draw_line(x1, y1, x2, y2, thickness, color);
}

fn render_roads(board: &Board, hex_centers: &[[f32; 2]; 19], scale: f32) {
    for idx in 0..BOARD_COORDS.len() {
        let (r, q) = BOARD_COORDS[idx];
        for n in 0..6 {
            if let Some(pc) = board.roads[r][q][n] {
                render_road(hex_centers[idx], n, pc.to_color(), scale)
            }
        }
    }
}

fn render_structures(board: &Board, hex_centers: &[[f32; 2]; 19], scale: f32) {
    for idx in 0..BOARD_COORDS.len() {
        let (r, q) = BOARD_COORDS[idx];
        for n in 0..6 {
            if let Some(s) = board.structures[r][q][n] {
                if s.structure_type == StructureType::Settlement {
                    render_settlement(hex_centers[idx], n, s.color.to_color(), scale);
                } else {
                    render_city(hex_centers[idx], n, s.color.to_color(), scale);
                }
            }
        }
    }
}

pub fn render_board(board: &Board) {
    let width = screen_width();
    let height = screen_height();
    let scale = if width > height {height} else {width} / 10.0;
    let hex_centers = hex_centers(width, height, scale);
    // let port_coords = port_coords(width, height, scale);

    clear_background(BLUE);
    render_hexes(board, &hex_centers, scale);
    render_roads(board, &hex_centers, scale);
    render_structures(board, &hex_centers, scale);
}