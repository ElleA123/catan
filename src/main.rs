use macroquad::window;
use rand::{seq::{IndexedRandom, SliceRandom}, Rng};

mod game;
mod render;
mod screen_coords;

use crate::game::*;
use crate::render::*;
use crate::screen_coords::ScreenCoords;

enum Action {
    Idling,
    Discarding,
    MovingRobber,
    BuildingRoad,
    BuildingSettlement,
    UpgradingToCity,
}

pub struct GameState {
    num_players: usize,
    board: Board,
    players: Vec<Player>,
    largest_army: Option<PlayerColor>,
    largest_army_size: usize,
    longest_road: Option<PlayerColor>,
    longest_road_size: usize,
    current_player: usize,
    action: Action,
    rolling_dice: bool,
    roll: Option<[usize; 2]>,
    played_dv: bool,
    stealing_from: Option<PlayerColor>,
    discarding: Option<ResHand>,
    yopping: Option<ResHand>,
    monopolizing: Option<ResHand>,
    offering_trade: bool,
    offered_trades: Vec<(ResHand, ResHand)>,
    passing_turn: bool,
}

impl GameState {
    fn new<R: Rng + ?Sized>(num_players: usize, rng: &mut R) -> GameState {
        let board = Board::new(num_players, rng);
        let players = PLAYER_COLORS.choose_multiple(rng, num_players)
            .map(|pc| Player::new(*pc))
            .collect();

        GameState {
            num_players,
            board,
            players,
            largest_army: None,
            largest_army_size: 2,
            longest_road: None,
            longest_road_size: 4,
            current_player: 0,
            action: Action::Idling,
            rolling_dice: false,
            roll: None,
            played_dv: false,
            discarding: None,
            stealing_from: None,
            yopping: None,
            monopolizing: None,
            offering_trade: false,
            offered_trades: Vec::new(),
            passing_turn: false,
        }
    }

    /*
    fn get_current_player(&self) -> &Player {
        &self.players[self.current_player]
    }
    
    fn get_current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player]
    }

    fn get_player(&self, color: PlayerColor) -> Option<&Player> {
        self.players.iter().find(|player| player.is_color(color))
    }

    fn get_player_mut(&mut self, color: PlayerColor) -> Option<&mut Player> {
        self.players.iter_mut().find(|player| player.is_color(color))
    }

    fn handle_setup_settlement_click(&mut self, mouse_pos: (f32, f32), clickables: &ClickablePoints) -> Option<[usize; 3]> {
        let color = self.get_current_player().get_color();
        let radius = 0.2 * clickables.board_scale;
        let maybe_idx = clickables.corners.iter().position(
            |pos| mouse_is_on_circle(mouse_pos, pos, radius)
        );
        if let Some(idx) = maybe_idx {
            let corner = CORNER_COORDS[idx];
            if self.board.can_place_setup_settlement(corner) {
                self.board.place_settlement(corner, color);
                self.get_current_player_mut().gain_vp();
                return Some(corner);
            }
        }
        return None;
    }

    fn handle_setup_road_click(&mut self, settlement_coord: [usize; 3], mouse_pos: (f32, f32), clickables: &ClickablePoints) -> bool {
        let color = self.get_current_player().get_color();
        let radius = 0.2 * clickables.board_scale;
        let maybe_idx = clickables.edges.iter().position(
            |pos| mouse_is_on_circle(mouse_pos, pos, radius)
        );
        if let Some(idx) = maybe_idx {
            let edge = EDGE_COORDS[idx];
            if self.board.can_place_setup_road(edge, settlement_coord) {
                self.board.place_road(edge, color);
                return true;
            }
        }
        return false;
    }

    async fn setup(&mut self) {
        let mut clickables;
        for player in 0..self.num_players {
            self.current_player = player;
            let mut settlement_coord = None;
            'turn: loop {
                clickables = render_screen(&self);
                if macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    let mouse_pos = macroquad::input::mouse_position();
                    if let Some(coord) = settlement_coord {
                        if self.handle_setup_road_click(coord, mouse_pos, &clickables) {
                            break 'turn;
                        };
                    }
                    else {
                        settlement_coord = self.handle_setup_settlement_click(mouse_pos, &clickables);
                    };
                }
                macroquad::window::next_frame().await
            }
        }
        for player in (0..self.num_players).rev() {
            self.current_player = player;
            let mut settlement_coord = None;
            'turn: loop {
                clickables = render_screen(&self);
                if macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    let mouse_pos = macroquad::input::mouse_position();
                    if let Some(coord) = settlement_coord {
                        if self.handle_setup_road_click(coord, mouse_pos, &clickables) {
                            break 'turn;
                        };
                    }
                    else {
                        settlement_coord = self.handle_setup_settlement_click(mouse_pos, &clickables);
                        if let Some(coord) = settlement_coord {
                            self.players[player].add_cards(self.board.get_starting_resources(coord));
                            // for [r, q] in hexes_touched(coord[0], coord[1], coord[2]) {
                            //     if let Some(hex) = self.board.hexes[r][q] {
                            //         self.players[player].hand.add(ResHand::from(hex.resource));
                            //     }
                            // }
                        }
                    };
                }
                macroquad::window::next_frame().await
            }
        }
    }

    async fn handle_seven<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        for (player, _) in self.players.iter().enumerate()
            .filter(|(_, p)| p.must_discard())
        {
            self.current_player = player;
            self.action = Action::Discarding;
            self.discarding = Some(ResHand::new());

            loop {
                let clickables = render_screen(&self);
                if macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                    handle_click(&clickables, self, rng);
                }
                macroquad::window::next_frame().await
            }
        }
    }

    async fn roll_dice<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        self.roll = Some([rng.random_range(1..=6), rng.random_range(1..=6)]);
        let sum = self.roll.unwrap()[0] + self.roll.unwrap()[1];
        if sum == 7 {
            self.handle_seven(rng).await // Why the fuck did I do this, lmao
        } else {
            self.board.give_resources(&mut self.players, sum);
        }
    }

    fn pass_turn(&mut self) {
        let player = self.get_current_player_mut();
        player.cycle_dvs();
        
        self.current_player = (self.current_player + 1) % self.num_players;
        self.action = Action::Idling;
        self.roll = None;
        self.played_dv = false;
        self.offering_trade = false;
    }

    fn winner(&self) -> Option<PlayerColor> {
        self.players.iter().find(|player| player.has_won()).map(|player| player.get_color())
    }
    */
}

/*
fn mouse_is_on_circle(mouse_pos: (f32, f32), center: &[f32; 2], radius: f32) -> bool {
    (mouse_pos.0 - center[0]).powi(2) + (mouse_pos.1 - center[1]).powi(2) <= radius.powi(2)
}

fn mouse_is_on_rect(mouse_pos: (f32, f32), pos: [f32; 2], width: f32, height: f32) -> bool {
    mouse_pos.0 > pos[0] && mouse_pos.0 < pos[0] + width
    && mouse_pos.1 > pos[1] && mouse_pos.1 < pos[1] + height
}

fn handle_knight(state: &mut GameState) {
    println!("handling knight");
    state.get_current_player_mut().play_dv_card(DVCard::Knight);
    if state.get_current_player().get_knights() > state.largest_army_size {
        let old_player = state.largest_army.replace(state.get_current_player().get_color());
        state.largest_army_size = state.get_current_player().get_knights();
        if let Some(player) = old_player {
            state.get_player_mut(player).unwrap().set_largest_army(false);
        }
        state.get_current_player_mut().set_largest_army(true);
        if state.winner().is_some() {
            return
        }
    }
    state.action = Action::MovingRobber;
}

fn handle_road_building(state: &mut GameState) {
    // Ughhh
}

fn handle_year_of_plenty(state: &mut GameState) {
    // Hand selector, size of 2
}

fn handle_monopoly(state: &mut GameState) {
    // Reuse hand selector API w/ size of 1?
}

fn handle_dv_card(card: DVCard, state: &mut GameState) {
    match card {
        DVCard::Knight => handle_knight(state),
        DVCard::RoadBuilding => handle_road_building(state),
        DVCard::YearOfPlenty => handle_year_of_plenty(state),
        DVCard::Monopoly => handle_monopoly(state),
        DVCard::VictoryPoint => ()
    }
}

fn handle_idle_click<R: Rng + ?Sized>(clickables: &ClickablePoints, mouse_pos: (f32, f32), state: &mut GameState, rng: &mut R) {
    if clickables.dice.iter().any(|pos| mouse_is_on_rect(mouse_pos, *pos, clickables.dice_size, clickables.dice_size)) {
        state.rolling_dice = true;
    }
    else if let Some(id) = clickables.cards.iter().position(|pos| mouse_is_on_rect(mouse_pos, *pos, clickables.card_size[0], clickables.card_size[1])) {
        println!("{}th card clicked", id);
        if id >= state.get_current_player().get_hand().count_nonzero() {
            println!("its a dv");
            let card = state.get_current_player().get_combined_dvs()
                .nth_nonzero(id - state.get_current_player().get_hand().count_nonzero()).unwrap();
            println!("{:?}", card);
            if state.get_current_player().get_dvs()[card] > 0 {
                handle_dv_card(card, state);
            }
        }
    }
    else if state.roll.is_some() {
        let maybe_menu_id = clickables.buttons.iter().position(|&pos| mouse_is_on_rect(mouse_pos, pos, clickables.button_size, clickables.button_size));
        if let Some(id) = maybe_menu_id {
            match id {
                0 => {
                    if state.get_current_player_mut().can_buy_dv() {
                        let card = state.board.draw_dv_card(rng);
                        state.get_current_player_mut().buy_dv(card);
                    }
                },
                1 => {
                    if state.get_current_player_mut().can_build_road() {
                        state.action = Action::BuildingRoad;
                    }
                },
                2 => {
                    if state.get_current_player_mut().can_build_settlement() {
                        state.action = Action::BuildingSettlement;
                    }
                },
                3 => {
                    if state.get_current_player_mut().can_upgrade_to_city() {
                        state.action = Action::UpgradingToCity;
                    }
                },
                4 => {
                    state.passing_turn = true;
                },
                _ => panic!("handle_idle_click(): illegal menu button")
            }
        }
    }
}

fn handle_discarding_click(clickables: &ClickablePoints, mouse_pos: (f32, f32), state: &mut GameState) {

}

fn handle_steal(state: &mut GameState) {
    let opts = state.board.get_colors_on_hex(state.board.robber);
    if opts.len() == 0 {
        return;
    }
    let target = if opts.len() == 1 {
        opts.get(opts.iter().nth(0).unwrap()).unwrap()
    } else {
        opts.get(opts.iter().nth(0).unwrap()).unwrap() // TODO
    };
    state.stealing_from = Some(*target);
}

fn handle_robber_click(clickables: &ClickablePoints, mouse_pos: (f32, f32), state: &mut GameState) {
    let radius= 0.46 * clickables.board_scale;
    let maybe_idx = clickables.centers.iter().position(
        |center| mouse_is_on_circle(mouse_pos, center, clickables.board_scale)
    );
    if let Some(idx) = maybe_idx {
        let coord = BOARD_COORDS[idx];
        if state.board.robber != coord {
            state.board.robber = coord;
            handle_steal(state);
        }
        state.action = Action::Idling;
    }
}

fn handle_road_click(clickables: &ClickablePoints, mouse_pos: (f32, f32), state: &mut GameState) {
    if mouse_is_on_rect(mouse_pos, clickables.buttons[1], clickables.button_size, clickables.button_size) {
        state.action = Action::Idling;
        return
    }
    let radius = 0.2 * clickables.board_scale;
    let color = state.get_current_player().get_color();
    let maybe_idx = clickables.edges.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let edge = EDGE_COORDS[idx];
        if state.board.can_place_road(edge, color) {
            state.get_current_player_mut().build_road();
            state.board.place_road(edge, color);
            state.action = Action::Idling;
        }
    }
}

fn handle_structure_click(structure_type: StructureType, clickables: &ClickablePoints, mouse_pos: (f32, f32), state: &mut GameState) {
    let cancel_button = if structure_type == StructureType::Settlement {clickables.buttons[2]} else {clickables.buttons[3]};
    if mouse_is_on_rect(mouse_pos, cancel_button, clickables.button_size, clickables.button_size) {
        state.action = Action::Idling;
        return
    }
    let radius = 0.2 * clickables.board_scale;
    let color = state.get_current_player().get_color();
    let maybe_idx = clickables.corners.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let corner = CORNER_COORDS[idx];
        if structure_type == StructureType::Settlement
        && state.board.can_place_settlement(corner, color) {
            state.get_current_player_mut().build_settlement();
            state.board.place_settlement(corner, color);
            state.action = Action::Idling;
        }
        else if state.board.can_upgrade_to_city(corner, color) {
            state.get_current_player_mut().upgrade_to_city();
            state.board.upgrade_to_city(corner, color);
            state.action = Action::Idling;
        }
    }
}

fn handle_click<R: Rng + ?Sized>(clickables: &ClickablePoints, state: &mut GameState, rng: &mut R) {
    let mouse_pos = macroquad::input::mouse_position();
    match state.action {
        Action::Idling => handle_idle_click(clickables, mouse_pos, state, rng),
        Action::Discarding => handle_discarding_click(clickables, mouse_pos, state),
        Action::MovingRobber => handle_robber_click(clickables, mouse_pos, state),
        Action::BuildingRoad => handle_road_click(clickables, mouse_pos, state),
        Action::BuildingSettlement => handle_structure_click(StructureType::Settlement, clickables, mouse_pos, state),
        Action::UpgradingToCity => handle_structure_click(StructureType::City, clickables, mouse_pos, state),
    }
}

#[macroquad::main("Catan")]
async fn main() {
    let mut rng = rand::rng();
    let num_players = 4;

    let mut state = GameState::new(num_players, &mut rng);
    state.setup().await;

    let winner: PlayerColor;
    let mut clickables;
    loop {
        if state.rolling_dice {
            state.roll_dice(&mut rng);
            state.rolling_dice = false;
        }

        if let Some(target) = state.stealing_from {
            let maybe_card = state.get_player_mut(target).unwrap().discard_random_card(&mut rng);
            if let Some(card) = maybe_card {
                state.get_current_player_mut().add_cards(ResHand::from(card));
            }
            state.stealing_from = None;
        }

        if state.passing_turn {
            state.pass_turn();
            state.passing_turn = false;
        }

        clickables = render_screen(&state);
        if macroquad::input::is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
            handle_click(&clickables, &mut state, &mut rng);
            if let Some(w) = state.winner() {
                winner = w;
                break;
            }
        }

        macroquad::window::next_frame().await
    }
    println!("{:?} wins!!!", winner);
}
*/

#[macroquad::main("Catan")]
async fn main() {
    let mut rng = rand::rng();
    let num_players = 4;

    let mut coords = ScreenCoords::new();
    let mut state = GameState::new(num_players, &mut rng);

    loop {
        coords.update();
        render_screen(&coords, &state);

        window::next_frame().await
    }
}