use core::num;

use macroquad::{
    input::{is_mouse_button_pressed, mouse_position, MouseButton}, window
};
use rand::{seq::IndexedRandom, Rng};

mod game;
mod render;
mod screen_coords;

use crate::game::*;
use crate::render::*;
use crate::screen_coords::ScreenCoords;

pub struct SetupState {
    num_players: usize,
    board: Board,
    players: Vec<Player>,
    current_player: usize,
    all_placed_once: bool,
    settlement: Option<[usize; 3]>,
    finished: bool,
}

impl SetupState {
    fn new<R: Rng + ?Sized>(num_players: usize, rng: &mut R) -> SetupState {
        let board = Board::new(num_players, rng);
        let players = PLAYER_COLORS.choose_multiple(rng, num_players)
            .map(|pc| Player::new(*pc))
            .collect();

        SetupState {
            num_players,
            board,
            players,
            current_player: 0,
            all_placed_once: false,
            settlement: None,
            finished: false
        }
    }

    fn get_current_color(&self) -> PlayerColor {
        self.players[self.current_player].get_color()
    }

    fn get_current_player(&self) -> &Player {
        &self.players[self.current_player]
    }

    fn get_current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.current_player]
    }

    fn is_players_turn(&self, color: PlayerColor) -> bool {
        self.get_current_color() == color
    }

    // fn can_place_road(&self, edge: [usize; 3]) -> bool {
    //     let settlement_coord = self.settlement.unwrap();
    //     self.board.can_place_setup_road(edge, settlement_coord)
    // }

    // fn can_place_settlement(&self, corner: [usize; 3]) -> bool {
    //     self.board.can_place_setup_settlement(corner)
    // }

    fn place_road(&mut self, edge: [usize; 3]) {
        let color = self.get_current_color();
        self.board.place_setup_road(edge, color);
        self.get_current_player_mut().place_setup_road();
    }

    fn place_settlement(&mut self, corner: [usize; 3]) {
        let color = self.get_current_color();
        self.board.place_setup_settlement(corner, color);
        self.get_current_player_mut().place_setup_settlement();
        self.settlement = Some(corner);

        if self.all_placed_once {
            let start_hand = self.board.get_starting_resources(corner);
            self.get_current_player_mut().get_cards(start_hand);
        }
    }

    fn advance_turn(&mut self) {
        if self.all_placed_once {
            if self.current_player == 0 {
                self.finished = true;
            }
            else {
                self.current_player -= 1;
            }
        }
        else {
            if self.current_player == self.num_players - 1 {
                self.all_placed_once = true;
            }
            else {
                self.current_player += 1;
            }
        }
        self.settlement = None;
    }
}

enum Action {
    Idling,
    Discarding,
    MovingRobber,
    BuildingRoad,
    BuildingSettlement,
    BuildingCity,
}

enum RngAction {
    RollingDice,
    Stealing(PlayerColor),
    BuyingDV,
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
    roll: Option<[usize; 2]>,
    played_dv: bool,
    offered_trades: Vec<(ResHand, ResHand)>,
    rng_action: Option<RngAction>,
}

impl From<SetupState> for GameState {
    fn from(setup_state: SetupState) -> Self {
        GameState {
            num_players: setup_state.num_players,
            board: setup_state.board,
            players: setup_state.players,
            largest_army: None,
            largest_army_size: 2,
            longest_road: None,
            longest_road_size: 4,
            current_player: 0,
            action: Action::Idling,
            roll: None,
            played_dv: false,
            offered_trades: Vec::new(),
            rng_action: None,
        }
    }
}

impl GameState {
    // fn new<R: Rng + ?Sized>(num_players: usize, rng: &mut R) -> GameState {
    //     let board = Board::new(num_players, rng);
    //     let players = PLAYER_COLORS.choose_multiple(rng, num_players)
    //         .map(|pc| Player::new(*pc))
    //         .collect();

    //     GameState {
    //         num_players,
    //         board,
    //         players,
    //         largest_army: None,
    //         largest_army_size: 2,
    //         longest_road: None,
    //         longest_road_size: 4,
    //         current_player: 0,
    //         action: Action::Idling,
    //         rolling_dice: false,
    //         roll: None,
    //         played_dv: false,
    //         discarding: None,
    //         stealing_from: None,
    //         yopping: None,
    //         monopolizing: None,
    //         offering_trade: false,
    //         offered_trades: Vec::new(),
    //         passing_turn: false,
    //     }
    // }

    fn get_current_color(&self) -> PlayerColor {
        self.get_current_player().get_color()
    }

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

    fn get_order(&self) -> Vec<PlayerColor> {
        self.players.iter().map(|player| player.get_color()).collect()
    }

    fn is_players_turn(&self, color: PlayerColor) -> bool {
        self.get_current_color() == color
    }

    fn can_buy_dv(&self) -> bool {
        self.get_current_player().can_buy_dv() && self.board.can_draw_dv_card()
    }

    fn can_build_road(&self) -> bool {
        self.get_current_player().can_build_road()
        && self.board.can_place_any_road(self.get_current_color()) 
    }

    fn can_build_settlement(&self) -> bool {
        self.get_current_player().can_build_settlement()
        && self.board.can_place_any_settlement(self.get_current_color())
    }

    fn can_build_city(&self) -> bool {
        self.get_current_player().can_build_city()
        && self.board.can_place_any_city(self.get_current_color())
    }

    fn get_available_actions(&self, color: PlayerColor) -> [bool; 5] {
        if color != self.get_current_color() || self.roll.is_none() {
            return [false; 5];
        }
        match self.action {
            Action::Idling => [
                self.can_buy_dv(),
                self.can_build_road(),
                self.can_build_settlement(),
                self.can_build_city(),
                true
            ],
            Action::BuildingRoad => [false, true, false, false, false],
            Action::BuildingSettlement => [false, false, true, false, false],
            Action::BuildingCity => [false, false, false, true, false],
            _ => [false; 5]
        }
    }

    fn roll_dice<R: Rng + ?Sized>(&mut self, rng: &mut R) -> usize {
        self.roll = Some([rng.random_range(1..=6), rng.random_range(1..=6)]);
        return self.roll.unwrap()[0] + self.roll.unwrap()[1];
    }

    fn give_resources(&mut self, roll: usize) {
        let resources = self.board.get_new_resources(self.get_order(), roll);
        for idx in 0..self.num_players {
            self.players[idx].get_cards(resources[idx]);
        }
    }

    fn buy_dv_card<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let dv = self.board.draw_dv_card(rng);
        self.get_current_player_mut().buy_dv(dv);
    }

    fn build_road(&mut self, edge: [usize; 3]) {
        let color = self.get_current_color();
        self.board.place_road(edge, color);
        self.get_current_player_mut().build_road();
    }

    fn build_settlement(&mut self, corner: [usize; 3]) {
        let color = self.get_current_color();
        self.board.place_settlement(corner, color);
        self.get_current_player_mut().build_settlement();
    }

    fn build_city(&mut self, corner: [usize; 3]) {
        let color = self.get_current_color();
        self.board.place_city(corner, color);
        self.get_current_player_mut().build_city();
    }

    fn play_dv_card(&mut self, card: DVCard) {
        match card {
            DVCard::Knight => (),
            DVCard::RoadBuilding => (),
            DVCard::YearOfPlenty => (),
            DVCard::Monopoly => (),
            DVCard::VictoryPoint => (),
        }
    }

    fn pass_turn(&mut self) {
        self.get_current_player_mut().cycle_dvs();

        self.current_player = (self.current_player + 1) % self.num_players;
        self.roll = None;
        self.played_dv = false;
        self.offered_trades.clear();
        self.action = Action::Idling;
    }
}

fn mouse_is_on_circle(mouse_pos: (f32, f32), center: [f32; 2], radius: f32) -> bool {
    (mouse_pos.0 - center[0]).powi(2) + (mouse_pos.1 - center[1]).powi(2) <= radius.powi(2)
}

fn mouse_is_on_rect(mouse_pos: (f32, f32), pos: [f32; 2], width: f32, height: f32) -> bool {
    mouse_pos.0 > pos[0] && mouse_pos.0 < pos[0] + width
    && mouse_pos.1 > pos[1] && mouse_pos.1 < pos[1] + height
}

fn handle_setup_road_click(state: &mut SetupState, coords: &ScreenCoords, settlement: [usize; 3], mouse_pos: (f32, f32)) {
    let radius = coords.build_clickable_radius;
    let maybe_idx = coords.edges.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let edge = EDGE_COORDS[idx];
        if state.board.can_place_setup_road(edge, settlement) {
            state.place_road(edge);
            state.advance_turn();
        }
    }
}

fn handle_setup_settlement_click(state: &mut SetupState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    let radius = coords.build_clickable_radius;
    let maybe_idx = coords.corners.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let corner = CORNER_COORDS[idx];
        if state.board.can_place_setup_settlement(corner) {
            state.place_settlement(corner);
        }
    }
}

fn handle_setup_click(state: &mut SetupState, coords: &ScreenCoords) {
    let mouse_pos = mouse_position();
    match state.settlement {
        Some(settlement) => handle_setup_road_click(state, coords, settlement, mouse_pos),
        None => handle_setup_settlement_click(state, coords, mouse_pos)
    }
}

async fn setup_game(mut state: SetupState) -> GameState {
    let mut coords = ScreenCoords::new();
    loop {
        coords.update();

        if is_mouse_button_pressed(MouseButton::Left) {
            handle_setup_click(&mut state, &coords);
        }

        if state.finished {
            return state.into()
        }

        render_setup_screen(&coords, &state, state.get_current_color());

        window::next_frame().await
    }
}

fn handle_idling_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    let [card_width, card_height] = coords.card_size;
    if let Some(n) = coords.cards.iter().position(
        |pos| mouse_is_on_rect(mouse_pos, *pos, card_width, card_height)
    ) {
        let num_resources = state.get_current_player().get_hand().count_nonzero();
        if n >= num_resources {
            let card = state.get_current_player().get_combined_dvs().nth_nonzero(n - num_resources).unwrap();
            let playable_dvs = state.get_current_player().get_dvs();
            if playable_dvs[card] > 0 {
                state.play_dv_card(card);
            }
        }
        return;
    }

    if state.roll.is_none() {
        if coords.dice.iter().any(
            |pos| mouse_is_on_rect(mouse_pos, *pos, coords.dice_size, coords.dice_size)
        ) {
            state.rng_action = Some(RngAction::RollingDice);
        }
        return;
    }

    let maybe_menu_id = coords.buttons.iter().position(
        |&pos| mouse_is_on_rect(mouse_pos, pos, coords.button_size, coords.button_size)
    );
    if let Some(id) = maybe_menu_id {
        match id {
            0 => {
                if state.can_buy_dv() {
                    state.rng_action = Some(RngAction::BuyingDV);
                }
            },
            1 => {
                if state.can_build_road() {
                    state.action = Action::BuildingRoad;
                }
            },
            2 => {
                if state.can_build_settlement() {
                    state.action = Action::BuildingSettlement;
                }
            },
            3 => {
                if state.can_build_city() {
                    state.action = Action::BuildingCity;
                }
            },
            4 => {
                state.pass_turn();
            },
            _ => panic!("handle_idling_click(): illegal menu button")
        }
        return;
    }
}

fn handle_discarding_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    
}

fn handle_robber_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    
}

fn handle_road_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    let cancel_button = coords.buttons[1];
    if mouse_is_on_rect(mouse_pos, cancel_button, coords.button_size, coords.button_size) {
        state.action = Action::Idling;
        return
    }
    let radius = coords.build_clickable_radius;
    let color = state.get_current_color();
    let maybe_idx = coords.edges.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
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

fn handle_structure_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32), structure_type: StructureType) {
    let cancel_button = if structure_type == StructureType::Settlement { coords.buttons[2] } else { coords.buttons[3] };
    if mouse_is_on_rect(mouse_pos, cancel_button, coords.button_size, coords.button_size) {
        state.action = Action::Idling;
        return
    }
    let radius = coords.build_clickable_radius;
    let color = state.get_current_color();
    let maybe_idx = coords.corners.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
    );
    if let Some(idx) = maybe_idx {
        let corner = CORNER_COORDS[idx];
        if structure_type == StructureType::Settlement
        && state.board.can_place_settlement(corner, color) {
            state.build_settlement(corner);
            state.action = Action::Idling;
        }
        else if state.board.can_place_city(corner, color) {
            state.build_city(corner);
            state.action = Action::Idling;
        }
    }
}

fn handle_click(state: &mut GameState, coords: &ScreenCoords) {
    let mouse_pos = mouse_position();
    match state.action {
        Action::Idling => handle_idling_click(state, coords, mouse_pos),
        Action::Discarding => handle_discarding_click(state, coords, mouse_pos),
        Action::MovingRobber => handle_robber_click(state, coords, mouse_pos),
        Action::BuildingRoad => handle_road_click(state, coords, mouse_pos),
        Action::BuildingSettlement => handle_structure_click(state, coords, mouse_pos, StructureType::Settlement),
        Action::BuildingCity => handle_structure_click(state, coords, mouse_pos, StructureType::City),
    }
}

#[macroquad::main("Catan")]
async fn main() {
    let mut rng = rand::rng();
    let num_players = 4;

    let state = SetupState::new(num_players, &mut rng);
    let mut state = setup_game(state).await;

    let mut coords = ScreenCoords::new();

    loop {
        coords.update();

        if is_mouse_button_pressed(MouseButton::Left) {
            handle_click(&mut state, &coords);
        }

        if let Some(rng_action) = &state.rng_action {
            match rng_action {
                RngAction::RollingDice => {
                    let mut sum = 7;
                    while sum == 7 {
                        sum = state.roll_dice(&mut rng);
                    }
                    state.give_resources(sum);

                    state.rng_action = None;
                },
                RngAction::Stealing(color) => {
                    let stolen = state.get_player_mut(*color).unwrap().discard_random_card(&mut rng);
                    if let Some(res) = stolen {
                        state.get_current_player_mut().get_card(res);
                    }
                    state.rng_action = None;
                },
                RngAction::BuyingDV => {
                    state.buy_dv_card(&mut rng);
                    state.rng_action = None;
                },
            }
        }

        render_screen(&coords, &state, state.get_current_color());

        window::next_frame().await
    }
}