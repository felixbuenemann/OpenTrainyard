
use std::time::Duration;

use crate::data_saving::SelectedLevelSolvedDataEvent;
use crate::data_saving::SolutionData;
use crate::data_saving::SolutionsSavedData;
use crate::loading::FontAssets;
use crate::GameState;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::TransformPositionLens;

use crate::menu_utils::*;
use crate::board::Rect;

use crate::board::*;
use crate::all_puzzles_clean::*;

use crate::utils::SelectedLevel;


// Defines the amount of time that should elapse between each physics step.

/////////////////////////////////////////////////////////////////////////////////////
// COMPONENTS
/////////////////////////////////////////////////////////////////////////////////////


const ANIMATION_TIME_STEP: f32 = 600.;


pub struct MenuSolutionsPlugin;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::MenuSolutions`
impl Plugin for MenuSolutionsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(get_board_option_default())
            .add_systems(OnEnter(GameState::MenuSolutions), setup_solutions_menu)
            .add_systems(OnExit(GameState::MenuSolutions), cleanup_solutions_menu)
            .add_systems(Update, (
                create_board,
                handle_gesture_mouse,
                handle_gesture_touch,
                handle_full_click_solution,
                click_nextlevel_button_solution,
                click_prevlevel_button_solution,
                click_back_button_solution,
                click_clone_button_solution,
                click_newsolution_button_solution,
                scroll_events_solution_touch,
                scroll_events_solution_mouse,
                click_deletesolution_button_solution,
                make_board_and_title,
                advance_tick,
            ).run_if(in_state(GameState::MenuSolutions)))
            .add_message::<BoardEvent>()
            .add_message::<RedrawCarouselEvent>()
            // add CarouselState resource:
            .insert_resource(CarouselState::default())
            ;
    }
}


// Resource CarouselState:
#[derive(Debug, Component, Default, Resource)]
pub struct CarouselState {
    pub timer: Timer,
    pub position_offset: Vec3,
    pub position_delta: Vec3,
    // Note that current_index (which is the index of the current map) is stored in the SelectedLevel resource !!
}


#[derive(Component)]
pub struct SolutionsMenuBotton;

#[derive(Component)]
pub struct NextLevelButtonSolutions;

#[derive(Component)]
pub struct PrevLevelButton;

#[derive(Component)]
pub struct CopySolutionButton;

#[derive(Component)]
pub struct NewSolutionButton;

#[derive(Component)]
pub struct DeleteSolutionButton;

#[derive(Component)]
pub struct BackButtonSolutions;

#[derive(Component)]
pub struct CloneButton;

#[derive(Component)]
pub struct LevelNameElem;

#[derive(Component)]
pub struct BestScoreElem;

#[derive(Component)]
pub struct CarouselTextNode;




/////////////////////////////////////////////////////////////////////////////////////
// EVENTS
/////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Message)]
pub struct RedrawCarouselEvent {
    pub maps: Option<Vec<SolutionData>>,
    pub level_name: String,
    pub index: Option<usize>,
}


/////////////////////////////////////////////////////////////////////////////////////
// SYSTEMS
/////////////////////////////////////////////////////////////////////////////////////



fn advance_tick(
    mut carousel_state: ResMut<CarouselState>,
    time: Res<Time>,
) {
    carousel_state.timer.tick(time.delta());
}


fn setup_solutions_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    selected_level: ResMut<SelectedLevel>,
    // Resource CarouselState:
    mut redraw_carousel_event_writer: MessageWriter<RedrawCarouselEvent>,
)
{
    let level_name = selected_level.level.clone();
    // Print the game name:
    println!("LAUNCHED: {}", level_name.clone());
    redraw_carousel_event_writer.write(RedrawCarouselEvent { maps: None, level_name: level_name, index: None});
    let font_size = 15.;

    let window = window_query.single().unwrap();
    let (width, margin, _heigh, percent_left_right, left, right, bottom, top) = get_coordinates(window);
    let _prev_id = make_button("PREVIOUS LEVEL".to_string(), &mut commands, &font_assets, &button_colors, font_size, left, right , top, bottom, PrevLevelButton, Some(SolutionsMenuBotton));
    let _next_id = make_button("NEXT LEVEL".to_string(), &mut commands, &font_assets, &button_colors, font_size, width * percent_left_right + margin/2., width - margin , top, bottom, SolutionsMenuBotton, Some(NextLevelButtonSolutions));


    let ((l1, r1, b1, t1), (l2, r2, b2, t2), (l3, r3, b3, t3)) = get_sol_commands_coordinates(window);
    println!("l1: {}, r1: {}, b1: {}, t1: {}", l1, r1, b1, t1);
    println!("l2: {}, r2: {}, b2: {}, t2: {}", l2, r2, b2, t2);
    println!("l3: {}, r3: {}, b3: {}, t3: {}", l3, r3, b3, t3);

    let _new_id = make_button("DELETE".to_string(), &mut commands, &font_assets, &button_colors, font_size, l1, r1, t1, b1, SolutionsMenuBotton, Some(DeleteSolutionButton));
    let _new_id = make_button("NEW SOLUTION".to_string(), &mut commands, &font_assets, &button_colors, font_size, l2, r2, t2, b2, SolutionsMenuBotton, Some(NewSolutionButton));
    let _clone_id = make_button("CLONE".to_string(), &mut commands, &font_assets, &button_colors, font_size, l3, r3, t3, b3, SolutionsMenuBotton, Some(CloneButton));


    // Upper::
    let ((left_, right_, bottom_, top_), _, _) = get_upper_coordinates(window);
    let _back_id = make_button("BACK".to_string(), &mut commands, &font_assets, &button_colors, 22.*0.8, left_, right_, top_, bottom_, SolutionsMenuBotton, Some(BackButtonSolutions));
}






fn cleanup_solutions_menu(
        mut commands: Commands,
        buttons: Query<Entity, With<SolutionsMenuBotton>>,
        board_q: Query<Entity, With<Board>>,
        mut board_event_writer: MessageWriter<BoardEvent>,
) {
    // For button in query:
    for button in buttons.iter() { // It's never more than 1, but can very well be 0
        if let Ok(mut id) = commands.get_entity(button) { id.despawn();};
    }
    // Delete boards:
    for board_id in board_q.iter() {
        if let Ok(mut id) = commands.get_entity(board_id) { id.despawn();}
    }
    board_event_writer.write(BoardEvent::Delete);

}



// Listen to scrollwheenl events:
pub fn scroll_events_solution_mouse(
    mut scroll_evr: MessageReader<MouseWheel>,
    board_q: Query<(Entity, &Transform), With<Board>>,
    textnode_q: Query<(Entity, &Transform, &Node), With<CarouselTextNode>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut carousel_state: ResMut<CarouselState>,
    mut commands: Commands,
    mut selected_level: ResMut<SelectedLevel>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.read() {
        let (vx, vy) = match ev.unit {
            MouseScrollUnit::Line => { (ev.x, ev.y) }
            MouseScrollUnit::Pixel => { (ev.x, ev.y) }
        };
        // v = vy if vx==0 else vx
        let v = if vx == 0. { vy } else { vx };
        let window = window_query.single().unwrap();
        _scroll_event_solution(v, &mut carousel_state, &mut selected_level, &board_q, &textnode_q, window, &mut commands);
    }
}

// Set constan SCROLLWHEEL_SPEED_MULTIPLIER:
const TOUCH_SWIPE_SPEED_DECAY: f32 = 0.04;


// Listen to scrollwheenl events:
pub fn scroll_events_solution_touch(
    board_q: Query<(Entity, &Transform), With<Board>>,
    textnode_q: Query<(Entity, &Transform, &Node), With<CarouselTextNode>>,
    mut scroll_evr: MessageReader<ScrollHappened>,
    // touches: Res<Touches>,
    mut carousel_state: ResMut<CarouselState>,
    mut commands: Commands,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut selected_level: ResMut<SelectedLevel>,
) {
    // for finger in touches.iter() {
    //     *current_vy = Some(finger.delta().y);
    //     let finger_pos = format!("{:?}", finger.position());
    // }
    for ev in scroll_evr.read() {
        let current_vx = Some(ev.vx);
        if let Some(vx) = current_vx.as_ref() {
            let window = window_query.single().unwrap();
            _scroll_event_solution(*vx, &mut carousel_state, &mut selected_level, &board_q, &textnode_q, window, &mut commands);
        }
    }
}



fn click_back_button_solution(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<BackButtonSolutions>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected_level: ResMut<SelectedLevel>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let level_name = selected_level.level.clone();
                *selected_level = SelectedLevel::default();
                selected_level.level = level_name;
                next_state.set(GameState::MenuLevels);
            }
            _ => {}
        }
    }
}


fn click_clone_button_solution(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<CloneButton>, With<SolutionsMenuBotton>)>,
    mut selected_level: ResMut<SelectedLevel>,
    // SelectedLevelSolvedDataEvent event writer:
    mut selected_level_solved_data_event_writer: MessageWriter<SelectedLevelSolvedDataEvent>,
    mut redraw_carousel_event_writer: MessageWriter<RedrawCarouselEvent>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let new_solution_data = selected_level.player_maps[selected_level.current_index as usize].clone();
                let new_index = selected_level.current_index.clone()as usize + 1;
                selected_level.player_maps.insert(new_index, new_solution_data);
                selected_level_solved_data_event_writer.write(SelectedLevelSolvedDataEvent{data: None});
                redraw_carousel_event_writer.write(RedrawCarouselEvent { maps: Some(selected_level.player_maps.clone()), level_name: selected_level.level.clone(), index: Some(new_index)});

            }
            _ => {}
        }
    }
}

fn click_newsolution_button_solution(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<NewSolutionButton>, With<SolutionsMenuBotton>)>,
    mut selected_level: ResMut<SelectedLevel>,
    // SelectedLevelSolvedDataEvent event writer:
    mut selected_level_solved_data_event_writer: MessageWriter<SelectedLevelSolvedDataEvent>,
    mut redraw_carousel_event_writer: MessageWriter<RedrawCarouselEvent>,
    levels: Res<PuzzlesData>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let empty_map = levels.puzzles.iter().find(|p| p.name == selected_level.level.clone()).unwrap().parsed_map.clone();
                let new_solution_data = SolutionData::new_from_string(empty_map, 0);
                let new_index = selected_level.current_index.clone()as usize + 1;
                selected_level.player_maps.insert(new_index, new_solution_data);
                selected_level_solved_data_event_writer.write(SelectedLevelSolvedDataEvent{data: None});
                redraw_carousel_event_writer.write(RedrawCarouselEvent { maps: Some(selected_level.player_maps.clone()), level_name: selected_level.level.clone(), index: Some(new_index)});

            }
            _ => {}
        }
    }
}

fn click_deletesolution_button_solution(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<DeleteSolutionButton>, With<SolutionsMenuBotton>)>,
    mut selected_level: ResMut<SelectedLevel>,
    // SelectedLevelSolvedDataEvent event writer:
    mut selected_level_solved_data_event_writer: MessageWriter<SelectedLevelSolvedDataEvent>,
    mut redraw_carousel_event_writer: MessageWriter<RedrawCarouselEvent>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if selected_level.player_maps.len() > 1 {
                    let index = selected_level.current_index as usize;
                    selected_level.player_maps.remove(index);
                    // let newindex be the min between (index and selected_level.player_maps.len() - 1);
                    let newindex: Option<usize> = if index <= selected_level.player_maps.len() - 1 { Some(index) } else { None};
                    selected_level_solved_data_event_writer.write(SelectedLevelSolvedDataEvent{data: None});
                    redraw_carousel_event_writer.write(RedrawCarouselEvent { maps: Some(selected_level.player_maps.clone()), level_name: selected_level.level.clone(), index: newindex});
                }
            }
            _ => {}
        }
    }
}



fn click_nextlevel_button_solution(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<NextLevelButtonSolutions>, With<SolutionsMenuBotton>)>,
    mut selected_level: ResMut<SelectedLevel>,
    levels: Res<PuzzlesData>,
    mut redraw_carousel_event_writer: MessageWriter<RedrawCarouselEvent>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(next_puzzle) = get_next_puzzle(selected_level.level.clone(), &levels) {
                    let level_name = next_puzzle.name.clone();
                    *selected_level = SelectedLevel::default();
                    selected_level.level = level_name.clone();
                    println!("LAUNCHED: {}", level_name.clone());
                    redraw_carousel_event_writer.write(RedrawCarouselEvent { maps: None, level_name: selected_level.level.clone(), index: None});
                    return
                }
            }
            _ => {}
        }
    }
}




fn click_prevlevel_button_solution(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<PrevLevelButton>, With<SolutionsMenuBotton>)>,
    mut selected_level: ResMut<SelectedLevel>,
    levels: Res<PuzzlesData>,
    mut redraw_carousel_event_writer: MessageWriter<RedrawCarouselEvent>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(prev_puzzle) = get_prev_puzzle(selected_level.level.clone(), &levels) {
                    let level_name = prev_puzzle.name.clone();
                    *selected_level = SelectedLevel::default();
                    selected_level.level = level_name.clone();
                    redraw_carousel_event_writer.write(RedrawCarouselEvent { maps: None, level_name: selected_level.level.clone(), index: None });
                    return
                }
            }
            _ => {}
        }
    }
}

// Listen to event:
fn handle_full_click_solution(
    mut full_click_happened_reader: MessageReader<FullClickHappened>,
    mut next_state: ResMut<NextState<GameState>>,
    mut selected_level: ResMut<SelectedLevel>,
    carousel_state: ResMut<CarouselState>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    for ev in full_click_happened_reader.read() {
        // Get the board:
        // Check if ev.pos is inside the board:
        let window = window_query.single().unwrap();
        let width = window.width() as f32;
        let height = window.height() as f32;
        let rect = Rect{left:0. - width / 2., top:height / 2. - width / 2.+25. - height / 2. , right:width - width / 2.,  bottom:height / 2. +width / 2. + 25. - height / 2. };
        if carousel_state.timer.is_finished() && in_bounds(ev.pos, rect) {
            // Get the map:
            println!("UHHH.. Why in finished??");
            let map = selected_level.player_maps[selected_level.current_index as usize].clone();
            selected_level.current_map = map.map;
            next_state.set(GameState::Playing);
            // Write the event:
            // change_level_writer.send(ChangeLevel);
        }
    }
}


fn make_board_and_title(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut commands: Commands,
    board_q: Query<Entity, With<Board>>,
    mut carousel_state: ResMut<CarouselState>,
    mut selected_level: ResMut<SelectedLevel>,
    levels: Res<PuzzlesData>,
    player_solutions_data: Res<SolutionsSavedData>,
    level_name_query: Query<Entity, With<LevelNameElem>>,
    best_score_text_query: Query<Entity, With<BestScoreElem>>,
    mut board_event_writer: MessageWriter<BoardEvent>,
    mut redraw_carousel_event_reader: MessageReader<RedrawCarouselEvent>,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
    texts: Query<Entity, With<CarouselTextNode>>,
) {
    for ev in redraw_carousel_event_reader.read() {
        // Get the window:
        let w = window_query.single().unwrap();
        // Get width:
        let width = w.width();
        let _height = w.height();
        // Delete board:
        for board_id in board_q.iter() {
            if let Ok(mut id) = commands.get_entity(board_id) { id.despawn();}
        }
        // Despawn the level name:
        for level_name_id in level_name_query.iter() {
            if let Ok(mut level_name_ec) = commands.get_entity(level_name_id) {level_name_ec.despawn();}
        }
        // For button in query:
        for text in texts.iter() { // It's never more than 1, but can very well be 0
            if let Ok(mut id) = commands.get_entity(text) { id.despawn();};
        }
        for best_score_text in best_score_text_query.iter() {
            if let Ok(mut id) = commands.get_entity(best_score_text) { id.despawn();};
        }

        // Get the maps:
        let empty_map = levels.puzzles.iter().find(|p| p.name == ev.level_name.clone()).unwrap().parsed_map.clone();
        let empty_map_data = SolutionData::new_from_string(empty_map.clone(), 0);
        let solved_data = player_solutions_data.get(&ev.level_name);
        let maps = match &ev.maps {
            Some(maps) => maps.clone(),
            None => {
                if selected_level.player_maps.len() != 0 {selected_level.player_maps.clone()} else if solved_data.len() == 0 { vec![empty_map_data] }  else { solved_data }
            }
        };
        let index = match &ev.index {
            Some(index) => *index as u16,
            None => maps.len() as u16 - 1,
        };
        *selected_level = SelectedLevel{
            level: ev.level_name.clone(),
            player_maps: maps.clone(),
            current_index: index,
            current_map: maps[index as usize].map.clone(),
            vanilla_map: empty_map,
            city: "".to_string(),
        };
        // print index:
        println!("Index: {}", selected_level.current_index);

        carousel_state.timer = Timer::new(Duration::from_millis(500), TimerMode::Once);
        carousel_state.position_delta = Vec3::new(width * 0.6, 0., 0.);
        carousel_state.position_offset = Vec3::new((1.-SCALE) * width/2. * 1.5  -width * 0.61, - width * SCALE / 2. + 25., 0.);


        // Spawn the level name BUTTON:
        let (_, (left_, right_, bottom_, top_), _) = get_upper_coordinates(w);
        let _name_id = make_text(ev.level_name.clone(), &mut commands, &font_assets, &button_colors, 20., left_, right_, top_, bottom_, SolutionsMenuBotton, Some(LevelNameElem));

        // Spawn the "pick solution" text:
        let _text_id = make_text("  PICK A SOLUTION".to_string(), &mut commands, &font_assets, &button_colors, 20., left_, right_, top_  + width * SCALE * 1.5 + 45., bottom_  + width * SCALE * 1.5 + 45., SolutionsMenuBotton, Some(BestScoreElem));
        if player_solutions_data.expert_mode() && !player_solutions_data.just_begun_level(&selected_level.level) {
            let best_solution_data = levels.puzzles.iter().find(|p| p.name == selected_level.level.clone()).unwrap().track_count.clone();
            let besttrack_text = " (BEST TRACK COUNT: ".to_string() + &best_solution_data +")";
            let _bestscore_id = make_text(besttrack_text, &mut commands, &font_assets, &button_colors, 17., left_, right_, top_  + width * SCALE * 1.5 + 45. + 25., bottom_  + width * SCALE * 1.5 + 45. + 25., SolutionsMenuBotton, Some(BestScoreElem));
        }


        // Set the name of the game:
        // Get the solved maps, if there are any:
        // let n_maps = maps.len();
        for (i, map_data) in selected_level.player_maps.iter().enumerate() {
            let ii = - (selected_level.current_index as i16) + i as i16;
            let pos = carousel_state.position_offset + carousel_state.position_delta * ii as f32;
            let boardpos = Some(BoardPosition::Custom(pos));
            board_event_writer.write(BoardEvent::Make{map_name: ev.level_name.clone(), map: map_data.map.clone(), scale: SCALE, position: boardpos, index: Some(i as u32)});

            // Make the text:
            let duration = if map_data.time == 0 {String::from("Unsolved")} else {format!("steps: {}", map_data.time)};
            let text = format!("{}+{}  ({})", map_data.tracks, map_data.second_tracks, duration);
            make_text(text, &mut commands, &font_assets, &button_colors, 20., left_ + carousel_state.position_delta.x * ii as f32, right_ + carousel_state.position_delta.x * ii as f32, top_+75., bottom_+75., SolutionsMenuBotton, Some(CarouselTextNode));
        }
    }
}



/////////////////////////////////////////////////////////////////////////////////////
// HELPER FUNCTIONS
/////////////////////////////////////////////////////////////////////////////////////

const SCALE: f32 = 0.5;

fn _scroll_event_solution(
        v: f32,
        carousel_state: &mut ResMut<CarouselState>,
        selected_level: &mut ResMut<SelectedLevel>,
        board_q: &Query<(Entity, &Transform), With<Board>>,
        textnode_q: &Query<(Entity, &Transform, &Node), With<CarouselTextNode>>,
        window: &Window,
        commands: &mut Commands) {
    if v<0. && carousel_state.timer.is_finished() && selected_level.current_index < selected_level.player_maps.len() as u16 - 1 {
        _start_animation(true, board_q, textnode_q, window, commands, carousel_state);
        selected_level.current_index += 1;
        selected_level.current_map = selected_level.player_maps[selected_level.current_index as usize].map.clone();
    } else if v>0. && carousel_state.timer.is_finished() && selected_level.current_index > 0
    {
        _start_animation(false, board_q, textnode_q, window, commands, carousel_state);
        selected_level.current_index -= 1;
        selected_level.current_map = selected_level.player_maps[selected_level.current_index as usize].map.clone();
    }
}

fn _start_animation(
    go_left: bool,
    board_q: &Query<(Entity, &Transform), With<Board>>,
    textnode_q: &Query<(Entity, &Transform, &Node), With<CarouselTextNode>>,
    window: &Window,
    commands: &mut Commands,
    carousel_state: &mut ResMut<CarouselState>,
) {
    // Get the window:
    let _width = window.width() as f32;
    let delta = if go_left { - carousel_state.position_delta.x } else { carousel_state.position_delta.x };
    for (board_id, transform) in board_q.iter() {
        let board_pos = transform.translation;
        let new_transform = Vec3::new(board_pos.x + delta, board_pos.y, board_pos.z);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut, Duration::from_millis(ANIMATION_TIME_STEP as u64),
            TransformPositionLens {start: transform.translation, end: new_transform,},
        );
        commands.entity(board_id).insert(TweenAnim::new(tween),);
    }
    // Same for text nodes - animate using left position change
    for (textnode_id, transform, node) in textnode_q.iter() {
        let current_left = match node.left {
            Val::Px(v) => v,
            _ => 0.0,
        };
        let current_right = match node.right {
            Val::Px(v) => v,
            _ => 0.0,
        };
        let _new_left = current_left + delta;
        let _new_right = current_right - delta;
        // Use transform animation instead of UiPositionLens which uses deprecated position field
        let start_pos = transform.translation;
        let end_pos = Vec3::new(start_pos.x + delta, start_pos.y, start_pos.z);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut, Duration::from_millis(ANIMATION_TIME_STEP as u64),
            TransformPositionLens {start: start_pos, end: end_pos,},
        );
        commands.entity(textnode_id).insert(TweenAnim::new(tween),);
    }

    // Restart the timer in the carousel state:
    carousel_state.timer = Timer::new(Duration::from_millis(ANIMATION_TIME_STEP as u64), TimerMode::Once)
}


fn get_coordinates(window: &Window) -> (f32, f32, f32, f32, f32, f32, f32, f32) {
    let width = window.width();
    let height = window.height();
    // Genius plan: I'll assume THE BOARD IS ALWAYS ABOUT AS WIDE AS THE SCREEN, AND ALSO SQUARE.
    // Boundaries (left right top bottom) of a Rectangle that occupies the LEFT HALF of the screen, minus a 20 pixel wide margin all around:
    let margin = 7.;
    let button_height = 40.;
    let percent_left_right = 0.5;
    let left = margin;
    let right = width * percent_left_right - margin/2.;
    // Make the button 40 px high FROM THE BOTTOM:
    let bottom = height / 2. + width / 2. - 1.5 * margin +3.;
    let top = height / 2. + width / 2. - 1.5 * margin + button_height +3.;
    (width, margin, button_height, percent_left_right, left, right, bottom, top)
}

fn get_upper_coordinates(window: &Window) -> ((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32)) {
    let width = window.width();
    let height = window.height();
    // Genius plan: I'll assume THE BOARD IS ALWAYS ABOUT AS WIDE AS THE SCREEN, AND ALSO SQUARE.
    // Boundaries (left right top bottom) of a Rectangle that occupies the RIGHT HALF of the screen, minus a 20 pixel wide margin all around:
    let margin = 7.;
    let button_height = 30.;
    // Position it at the TOP of the screen:
    let percent_left_right = 0.3;
    let left = margin;
    let right = width * percent_left_right + margin/2.;
    let bottom = height / 2. - width / 2. - 3.5 * margin - 2.* button_height;
    let top = height / 2. - width / 2. - 3.5 * margin - button_height;
    return ((left, right - margin -6., bottom, top), (right, width - right, bottom, top), (width - right + margin + 6., width - left, bottom, top));
}




fn get_sol_commands_coordinates(window: &Window) -> ((f32, f32, f32, f32), (f32, f32, f32, f32), (f32, f32, f32, f32)) {
    let width = window.width();
    let height = window.height();
    // Genius plan: I'll assume THE BOARD IS ALWAYS ABOUT AS WIDE AS THE SCREEN, AND ALSO SQUARE.
    // Boundaries (left right top bottom) of a Rectangle that occupies the RIGHT HALF of the screen, minus a 20 pixel wide margin all around:
    let margin = 7.;
    let button_height = 40.;
    let percent_left_right = 0.3;
    let left = margin;
    let right = width * percent_left_right + margin/2.;
    // Make the button 40 px high FROM THE BOTTOM:
    let bottom = height / 2. + width / 2. - 1.5 * margin  - button_height - margin + 3.;
    let top = height / 2. + width / 2. - 1.5 * margin + button_height - button_height - margin + 3.;
    return ((left, right - margin, bottom, top), (right, width - right, bottom, top), (width - right + margin, width - left, bottom, top));
}

