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
    fn new<R: Rng + ?Sized>(num_humans: usize, num_cpus: usize, rng: &mut R) -> SetupState {
        let num_players = num_humans + num_cpus;

        let board = Board::new(num_players, rng);
        let players = PLAYER_COLORS
            .iter().copied()
            .enumerate().collect::<Vec<(usize, PlayerColor)>>()
            .choose_multiple(rng, num_players)
            .map(|&(i, pc)| Player::new(pc, i < num_humans))
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

pub enum Action {
    Idling,
    Discarding,
    MovingRobber,
    ChoosingVictim,
    BuildingRoad,
    BuildingSettlement,
    BuildingCity,
    RoadBuilding(bool),
}

pub enum RngAction {
    RollingDice,
    Stealing(PlayerColor),
    BuyingDV,
}

pub enum Selector {
    Discarding(ResHand),
    Trading(ResHand, ResHand),
    Yopping(ResHand),
    Monopolizing(ResHand),
}

impl Selector {
    pub fn get_bottom(&self) -> ResHand {
        match self {
            Selector::Discarding(hand) => *hand,
            Selector::Trading(give, _) => *give,
            Selector::Yopping(hand) => *hand,
            Selector::Monopolizing(hand) => *hand,
        }
    }

    pub fn get_top(&self) -> Option<ResHand> {
        match self {
            Selector::Trading(_, get) => Some(*get),
            _ => None
        }
    }

    pub fn get_bottom_mut(&mut self) -> &mut ResHand {
        match self {
            Selector::Discarding(hand) => hand,
            Selector::Trading(give, _) => give,
            Selector::Yopping(hand) => hand,
            Selector::Monopolizing(hand) => hand,
        }
    }

    pub fn get_top_mut(&mut self) -> &mut ResHand {
        match self {
            Selector::Trading(_, get) => get,
            _ => panic!("Selector::get_top_mut(): no top")
        }
    }

    pub fn add_top_card(&mut self, card: Resource) {
        self.get_top_mut()[card] += 1;
    }

    pub fn add_bottom_card(&mut self, card: Resource) {
        self.get_bottom_mut()[card] += 1;
    }

    pub fn discard_top_card(&mut self, card: Resource) {
        self.get_top_mut()[card] -= 1;
    }

    pub fn discard_bottom_card(&mut self, card: Resource) {
        self.get_bottom_mut()[card] -= 1;
    }
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
    turn_player: usize,
    roll: Option<[usize; 2]>,
    played_dv: bool,
    selector: Option<Selector>,
    offered_trades: Vec<(ResHand, ResHand)>,
    trade_responses: Vec<Vec<bool>>,
    action: Action,
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
            turn_player: 0,
            roll: None,
            played_dv: false,
            selector: None,
            offered_trades: Vec::with_capacity(3),
            trade_responses: Vec::with_capacity(3),
            action: Action::Idling,
            rng_action: None,
        }
    }
}

impl GameState {
    // fn new<R: Rng + ?Sized>(num_humans: usize, num_cpus: usize, rng: &mut R) -> GameState {
    //     let num_players = num_humans + num_cpus;

    //     let board = Board::new(num_players, rng);
    //     let players = PLAYER_COLORS
    //         .iter().copied()
    //         .enumerate().collect::<Vec<(usize, PlayerColor)>>()
    //         .choose_multiple(rng, num_players)
    //         .map(|&(i, pc)| Player::new(pc, i < num_humans))
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
    //         turn_player: 0,
    //         action: Action::Idling,
    //         roll: None,
    //         played_dv: false,
    //         selector: None,
    //         offered_trades: Vec::with_capacity(3),
    //         trade_responses: Vec::with_capacity(3),
    //         rng_action: None
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

    fn move_robber(&mut self, hex: [usize; 2]) {
        self.board.robber = hex;
        let robbable = self.board.get_colors_on_hex(hex);
        if robbable.len() == 0 {
            self.action = Action::Idling;
            return;
        } else if robbable.len() == 1 {
            self.rng_action = Some(RngAction::Stealing(robbable.into_iter().nth(0).unwrap()));
            self.action = Action::Idling;
        } else {
            self.action = Action::ChoosingVictim;
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

    fn update_largest_army(&mut self) {
        let old = self.largest_army.replace(self.get_current_color());
        self.largest_army_size = self.get_current_player().get_knights();
        
        self.get_current_player_mut().set_largest_army(true);
        if let Some(prev) = old {
            self.get_player_mut(prev).unwrap().set_largest_army(false);
        }
    }

    fn advance_road_building(&mut self) {
        let placed_once = match self.action {
            Action::RoadBuilding(placed_once) => placed_once,
            _ => panic!("advance_road_building(): not road building")
        };

        if placed_once {
            self.action = Action::Idling;
        } else {
            self.action = Action::RoadBuilding(true);
        }
    }

    fn play_dv_card(&mut self, card: DVCard) {
        match card {
            DVCard::Knight => {
                self.get_current_player_mut().play_dv_card(DVCard::Knight);
                if self.get_current_player().get_knights() > self.largest_army_size {
                    self.update_largest_army();
                }
                self.action = Action::MovingRobber;
            },
            DVCard::RoadBuilding => {
                self.get_current_player_mut().play_dv_card(DVCard::RoadBuilding);
                self.action = Action::RoadBuilding(false);
            },
            DVCard::YearOfPlenty => {
                self.selector = Some(Selector::Yopping(ResHand::new()));
            },
            DVCard::Monopoly => {
                self.selector = Some(Selector::Monopolizing(ResHand::new()));
            },
            DVCard::VictoryPoint => (),
        }
    }

    fn someone_must_discard(&self) -> bool {
        self.players.iter().any(|p| p.must_discard())
    }

    fn initiate_discarding(&mut self) {
        self.action = Action::Discarding;
        self.selector = Some(Selector::Discarding(ResHand::new()));
        while !self.players[self.current_player].must_discard() {
            self.current_player = (self.current_player + 1) % self.num_players;
        }
    }

    fn can_add_to_bottom(&self, card: Resource) -> bool {
        let pool = self.get_current_player().get_hand()[card];
        match self.selector.as_ref().unwrap() {
            Selector::Discarding(hand) => hand[card] < pool,
            Selector::Trading(give, _) => give[card] < pool,
            Selector::Yopping(hand) => hand.size() < 2,
            Selector::Monopolizing(hand) => hand.size() < 1
        }
    }

    fn can_add_to_top(&self, card: Resource) -> bool {
        match self.selector.as_ref().unwrap() {
            Selector::Trading(_, get) => get[card] < 19,
            _ => false
        }
    }

    fn can_discard_from_bottom(&self, card: Resource) -> bool {
        let hand = match self.selector.as_ref().unwrap() {
            Selector::Discarding(hand) => hand,
            Selector::Trading(give, _) => give,
            Selector::Yopping(hand) => hand,
            Selector::Monopolizing(hand) => hand,
        };
        hand[card] > 0
    }

    fn can_discard_from_top(&self, card: Resource) -> bool {
        match self.selector.as_ref().unwrap() {
            Selector::Trading(_, get) => get[card] > 0,
            _ => false
        }
    }

    fn get_selector(&self) -> &Selector {
        self.selector.as_ref().unwrap()
    }

    fn get_selector_mut(&mut self) -> &mut Selector {
        self.selector.as_mut().unwrap()
    }

    fn can_cancel_selector(&self) -> bool {
        match self.selector.as_ref().unwrap() {
            Selector::Discarding(_) => false,
            _ => true
        }
    }

    fn can_trade_with_bank(&self, give: ResHand, get: ResHand) -> bool {
        if !self.board.bank.can_discard(get) || give.count_nonzero() != 1 {
            return false;
        }

        let item_given = give.nth_nonzero(0).unwrap();
        let color = self.get_current_color();
        let ports = PORT_COORDS.iter().enumerate().filter(|(_, [r, q, e])|
            self.board.structure_is_color([*r, *q, *e], color)
            || self.board.structure_is_color([*r, *q, (*e + 5) % 6], color)
        ).map(|(idx, _)| self.board.ports[idx]);

        let mut rate = 4;
        for port in ports {
            match port {
                Port::ThreeForOne => {
                    if rate == 4 {
                        rate = 3;
                    }
                }
                Port::TwoForOne(res) => {
                    if res == item_given {
                        rate = 2;
                        break;
                    }
                }
            }
        }
        give.size() == rate * get.size()
    }

    fn can_execute_selector(&self) -> bool {
        match self.selector.as_ref().unwrap() {
            Selector::Discarding(hand) =>
                hand.size() == self.get_current_player().get_hand().size() / 2,
            Selector::Trading(give, get) => trade_is_reasonable(*give, *get),
            Selector::Yopping(hand) => hand.size() == 2,
            Selector::Monopolizing(hand) => hand.size() == 1,
        }
    }

    fn execute_discard(&mut self, hand: ResHand) {
        self.get_current_player_mut().discard_cards(hand);
        while !self.players[self.current_player].must_discard() {
            self.current_player = (self.current_player + 1) % self.num_players;
            if self.current_player == self.turn_player {
                self.action = Action::MovingRobber;
                return;
            }
        }
        self.selector = Some(Selector::Discarding(ResHand::new()));
    }

    fn execute_trade(&mut self, give: ResHand, get: ResHand) {
        if self.can_trade_with_bank(give, get) {
            self.board.bank.discard(get);
            self.board.bank.add(give);
            self.get_current_player_mut().discard_cards(give);
            self.get_current_player_mut().get_cards(get);
        } else {
            self.offered_trades.push((give, get));
        }
        self.selector = Some(Selector::Trading(ResHand::new(), ResHand::new()))
    }

    fn execute_yop(&mut self, hand: ResHand) {
        self.get_current_player_mut().play_dv_card(DVCard::YearOfPlenty);
        self.get_current_player_mut().get_cards(hand);
    }

    fn execute_monopoly(&mut self, card: Resource) {
        self.get_current_player_mut().play_dv_card(DVCard::Monopoly);

        let monopolizer = self.get_current_color();
        let mut gained = 0;
        for player in self.players.iter_mut() {
            if !player.is_color(monopolizer) {
                gained += player.get_hand()[card];
                player.discard_all(card);
            }
        }
        let monopolied = ResHand::from_monopoly(card, gained);
        self.get_current_player_mut().get_cards(monopolied);
    }

    fn execute_selector(&mut self) {
        let Some(selector) = self.selector.take() else { panic!("execute_selector(): selector to execute!") };
        match selector {
            Selector::Discarding(hand) =>
                self.execute_discard(hand),
            Selector::Trading(give, get) =>
                self.execute_trade(give, get),
            Selector::Yopping(hand) =>
                self.execute_yop(hand),
            Selector::Monopolizing(hand) =>
                self.execute_monopoly(hand.nth_nonzero(0).unwrap())
        }
    }

    fn cancel_selector(&mut self) {
        self.selector = None;
    }

    fn open_trade_menu(&mut self) {
        self.selector = Some(Selector::Trading(ResHand::new(), ResHand::new()));
    }

    fn pass_turn(&mut self) {
        self.get_current_player_mut().cycle_dvs();

        self.turn_player = (self.turn_player + 1) % self.num_players;
        self.current_player = self.turn_player;
        self.roll = None;
        self.played_dv = false;
        self.offered_trades.clear();
        self.action = Action::Idling;
    }
}

fn trade_is_reasonable(give: ResHand, get: ResHand) -> bool {
    give.size() > 0 && get.size() > 0 && RESOURCES.iter().all(|&res| give[res] == 0 || get[res] == 0)
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
            return state.into();
        }

        render_setup_screen(&coords, &state, state.get_current_color());

        window::next_frame().await
    }
}

fn handle_selector_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) -> bool {
    let buttons = &coords.selector_buttons;
    let button_size = coords.selector_button_size;

    if mouse_is_on_rect(mouse_pos, buttons[0], button_size, button_size) {
        if state.can_cancel_selector() {
            state.cancel_selector();
        }
        return true;
    }
    if mouse_is_on_rect(mouse_pos, buttons[1], button_size, button_size) {
        if state.can_execute_selector() {
            state.execute_selector();
        }
        return true;
    }

    let [selector_card_width, selector_card_height] = coords.selector_card_size;
    let selector_size = coords.selector_selector_size;

    if let Some(idx) = coords.selector_bottom_cards.iter().position(
        |pos| mouse_is_on_rect(mouse_pos, *pos, selector_card_width, selector_card_height)
    ) {
        let card = RESOURCES[idx];
        if state.can_discard_from_bottom(card) {
            state.get_selector_mut().discard_bottom_card(card);
        }
        return true;
    }
    else if let Some(idx) = coords.selector_bottom_selectors.iter().position(
        |pos| mouse_is_on_rect(mouse_pos, *pos, selector_size, selector_size)
    ) {
        let card = RESOURCES[idx];
        if state.can_add_to_bottom(card) {
            state.get_selector_mut().add_bottom_card(card);
        }
    }

    match state.get_selector() {
        Selector::Trading(_, _) => (),
        _ => return false
    };

    if let Some(idx) = coords.selector_top_cards.iter().position(
        |pos| mouse_is_on_rect(mouse_pos, *pos, selector_card_width, selector_card_height)
    ) {
        let card = RESOURCES[idx];
        if state.can_discard_from_top(card) {
            state.get_selector_mut().discard_top_card(card);
        }
        return true;
    }
    else if let Some(idx) = coords.selector_top_selectors.iter().position(
        |pos| mouse_is_on_rect(mouse_pos, *pos, selector_size, selector_size)
    ) {
        let card = RESOURCES[idx];
        if state.can_add_to_top(card) {
            state.get_selector_mut().add_top_card(card);
        }
        return true;
    }
    return false;
}

fn handle_idling_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    if state.selector.is_none()
    && mouse_is_on_rect(mouse_pos, coords.trade_button, coords.trade_button_size, coords.trade_button_size) {
        state.open_trade_menu();
        return;
    }

    let [card_width, card_height] = coords.card_size;
    if let Some(n) = coords.cards.iter().position(
        |pos| mouse_is_on_rect(mouse_pos, *pos, card_width, card_height)
    ) {
        let num_resources = state.get_current_player().get_hand().count_nonzero();
        if n >= num_resources {
            if let Some(card) = state.get_current_player().get_combined_dvs().nth_nonzero(n - num_resources) {
                let playable_dvs = state.get_current_player().get_dvs();
                if playable_dvs[card] > 0 {
                    state.play_dv_card(card);
                }
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

    if state.selector.is_some() && handle_selector_click(state, coords, mouse_pos) {
        return;
    }
}

fn handle_discarding_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    if state.selector.is_some() {
        handle_selector_click(state, coords, mouse_pos);
    }
}

fn handle_moving_robber_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    let radius = coords.robber_clickable_radius;
    if let Some(idx) = coords.centers.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
    ) {
        state.move_robber(HEX_COORDS[idx]);
    }
}

fn handle_choosing_victim_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    let radius = coords.build_clickable_radius;
    if let Some(idx) = coords.corners.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
    ) {
        let corner = CORNER_COORDS[idx];
        let [r, q, c] = corner;
        if state.board.is_robbable(corner, state.get_current_color()) {
            let color = state.board.structures[r][q][c].as_ref().unwrap().color;
            state.rng_action = Some(RngAction::Stealing(color));
        }
    }
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
            state.build_road(edge);
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

fn handle_road_building_click(state: &mut GameState, coords: &ScreenCoords, mouse_pos: (f32, f32)) {
    let color = state.get_current_color();
    let radius = coords.build_clickable_radius;
    if let Some(idx) = coords.edges.iter().position(
        |pos| mouse_is_on_circle(mouse_pos, *pos, radius)
    ) {
        let edge = EDGE_COORDS[idx];
        if state.board.can_place_road(edge, color) {
            state.board.place_road(edge, color);
            state.advance_road_building();
        }
    }
}

fn handle_click(state: &mut GameState, coords: &ScreenCoords) {
    let mouse_pos = mouse_position();
    match state.action {
        Action::Idling => handle_idling_click(state, coords, mouse_pos),
        Action::Discarding => handle_discarding_click(state, coords, mouse_pos),
        Action::MovingRobber => handle_moving_robber_click(state, coords, mouse_pos),
        Action::ChoosingVictim => handle_choosing_victim_click(state, coords, mouse_pos),
        Action::BuildingRoad => handle_road_click(state, coords, mouse_pos),
        Action::BuildingSettlement => handle_structure_click(state, coords, mouse_pos, StructureType::Settlement),
        Action::BuildingCity => handle_structure_click(state, coords, mouse_pos, StructureType::City),
        Action::RoadBuilding(_) => handle_road_building_click(state, coords, mouse_pos)
    }
}

async fn play_one_player_game(num_cpus: usize) {
    if num_cpus == 0 || num_cpus > 3 { panic!("Error: bad amount of CPUs"); }

    let mut rng = rand::rng();

    let state = SetupState::new(1, num_cpus, &mut rng);
    let mut state = setup_game(state).await;

    // let mut state = GameState::new(1, num_cpus, &mut rng);
    // state.board.place_settlement([2, 2, 3], PlayerColor::Blue);
    // state.board.place_settlement([2, 2, 5], PlayerColor::Red);
    // state.board.place_settlement([2, 2, 1], PlayerColor::Orange);
    // state.board.place_settlement([0, 2, 0], PlayerColor::Red);

    let mut coords = ScreenCoords::new();

    loop {
        coords.update();

        if is_mouse_button_pressed(MouseButton::Left) {
            handle_click(&mut state, &coords);
        }

        if let Some(rng_action) = &state.rng_action {
            match rng_action {
                RngAction::RollingDice => {
                    let sum = state.roll_dice(&mut rng);
                    if sum != 7 {
                        state.give_resources(sum);
                    }
                    else {
                        if state.someone_must_discard() {
                            state.initiate_discarding();
                        } else {
                            state.action = Action::MovingRobber;
                        }
                    }
                },
                RngAction::Stealing(color) => {
                    let stolen = state.get_player_mut(*color).unwrap().discard_random_card(&mut rng);
                    if let Some(res) = stolen {
                        state.get_current_player_mut().get_card(res);
                    }
                    state.action = Action::Idling;
                },
                RngAction::BuyingDV => {
                    state.buy_dv_card(&mut rng);
                },
            }
            state.rng_action = None;
        }

        render_screen(&coords, &state, state.get_current_color());

        window::next_frame().await
    }
}

#[macroquad::main("Catan")]
async fn main() {
    let num_cpus = 3;
    play_one_player_game(num_cpus).await;
}