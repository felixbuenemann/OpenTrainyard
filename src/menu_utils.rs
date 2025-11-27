use std::time::Duration;

use crate::loading::FontAssets;
use crate::menu_game_screen::{MainGameBotton, NextLevelButton};

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_tweening::*;
use bevy_tweening::lens::TransformScaleLens;


use crate::loading::TileAssets;

/////////////////////////////////////////////////////////////////////////////////////
// COMPONENTS
/////////////////////////////////////////////////////////////////////////////////////

#[derive(Resource)]
pub struct ButtonColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::srgb(55./255., 65./255., 64./255.),
            hovered: Color::srgb(72./255., 79./255., 80./255.),
            pressed: Color::srgb(95./255., 105./255., 106./255.),
        }
    }
}

#[derive(Debug, Component, Clone, PartialEq, Copy, Default)]
pub struct ScrollBarLimits {
    pub max: f32,
    pub min: f32,
    pub current: f32,
    pub step: f32,
}

// Wrapper event for ScrollBarLimits since it can't be both Component and Event
#[derive(Debug, Clone, PartialEq, Copy, Default, Message)]
pub struct ScrollBarLimitsEvent {
    pub limits: ScrollBarLimits,
}
#[derive(Debug, Component, Clone, PartialEq, Copy, Default)]
pub struct ScrollBarPosition {
    max_x: f32,
    min_x: f32,
    current_x: f32,
    step_x: f32,
}
#[derive(Debug, Component, Clone, PartialEq, Copy, Default)]
pub struct ScrollBarStatus {
    dragging: bool,
}

#[derive(Bundle, Clone, Debug, Default)]
pub struct ScrollBarHandleBundle {
    pub scroll_bar_limits: ScrollBarLimits,
    pub scroll_bar_position: ScrollBarPosition,
    pub scroll_bar_status: ScrollBarStatus,

    // UI components for Bevy 0.15
    pub node: Node,
    pub button: Button,
    pub interaction: Interaction,
    pub focus_policy: FocusPolicy,
    pub background_color: BackgroundColor,
    pub z_index: ZIndex,
}

#[derive(Component)]
pub struct BorderElem;

#[derive(Component)]
pub struct ButtonData {
    pub text: String
}

//Resource "ClickPosition", with a (x,y) tuple:
#[derive(Default, Resource, Debug, Copy, Clone, PartialEq)]
pub struct ClickPosition { // This is EXCLUDIVELY used because we want to record a click as a click-RELERASE in the SAME POSITION, so that a Touch-Swipe DON't trigger a click.
    pub clicked_pos: Option<Vec2>,
    pub last_hovered_pos: Option<Vec2>,

}
// Impl euclidean distance:
pub fn distance(one: &Vec2, other: &Vec2) -> f32 {
    let x = one.x - other.x;
    let y = one.y - other.y;
    (x * x + y * y).sqrt()
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ClickState {JustClicked, Hovering, JustReleased}

#[derive(Component)]
pub struct TextElem;  // Use this to qury text elems


#[derive(Component)]
pub struct Popup;  // Use this to qury text elems

#[derive(Component)]
pub struct ClosePopupButton;  // Use this to qury text elems

#[derive(Component)]
pub struct NextLevelPopupButton;


#[derive(Debug)]
pub enum PopupType {
    Tutorial,
    Victory
}
// Default Tutorial:
impl Default for PopupType {
    fn default() -> Self {
        PopupType::Tutorial
    }
}

// Resource CarouselState:
#[derive(Debug, Component, Default, Resource)]
pub struct PopupTimer {
    pub timer: Option<Timer>,
    pub popup_text: String,
    pub popup_text_2: Option<String>,
    pub popup_type: PopupType,
}


/////////////////////////////////////////////////////////////////////////////////////
// EVENTS
/////////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Copy, Clone, Message)]
pub struct FullClickHappened {
    pub pos: Vec2
}

#[derive(Debug, Copy, Clone, Message)]
pub struct ScrollHappened {
    pub vx: f32,
    pub vy: f32
}



/////////////////////////////////////////////////////////////////////////////////////
// SYSTEMS
/////////////////////////////////////////////////////////////////////////////////////

pub fn button_color_handler(
    button_colors: Res<ButtonColors>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *color = button_colors.pressed.into();
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

pub fn scrollbar_input_handler(
    // Listen to mouse inputs:
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut ScrollBarPosition,
            &mut ScrollBarLimits,
            &mut ScrollBarStatus,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, _color, _sbpos, _sblimits, mut sbstatus) in
        interaction_query.iter_mut()
    {
        match *interaction {
            Interaction::Pressed => {
                sbstatus.dragging = true;
            }
            Interaction::Hovered => {}
            Interaction::None => {
                sbstatus.dragging = false;
            }
        }
        // If mouse just release left button:
        if mouse_button_input.just_released(MouseButton::Left) {
            sbstatus.dragging = false;
        }
    }
}

pub fn scrollbar_dragging_handler(
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    touches: Res<Touches>,
    mut interaction_query: Query<(
        &mut Node,
        &mut ScrollBarPosition,
        &mut ScrollBarLimits,
        &mut ScrollBarStatus,
    )>,
    mut dragged_event_writer: MessageWriter<ScrollBarLimitsEvent>,
) {
    for (mut node, mut sbpos, mut sblimits, sbstatus) in
        interaction_query.iter_mut()
    {
        if sbstatus.dragging {
            let window = window_query.single().unwrap();
            // Use cursor position for mouse, fall back to touch position
            let pos = window.cursor_position()
                .or_else(|| touches.iter().next().map(|t| t.position()));
            if let Some(pos) = pos {
                let handle_x = (sbpos.max_x - sbpos.min_x) * 0.30;

                let relposx = pos.x - sbpos.min_x - handle_x / 2.;
                let relposx = relposx.clamp(0., sbpos.max_x - sbpos.min_x - handle_x);
                // let window_size = Vec2::new(window.width(), window.height());
                // let position = pos - window_size / 2.;
                let fraction = relposx / (sbpos.max_x - sbpos.min_x - handle_x);
                // Fraction is now in [0,1].
                // Tranform it by (1-x)^3:
                let newval = _get_scrollbar_value(fraction, &sblimits);
                // println!("THANKSSS, {:?}", newval);
                // Update ScrollBarPosition:
                sbpos.current_x = relposx;
                node.left = Val::Px(relposx);
                // launch event:
                if newval != sblimits.current {
                    sblimits.current = newval;
                    dragged_event_writer.write(ScrollBarLimitsEvent { limits: *sblimits });
                }
            }
        }
    }
}




pub fn handle_gesture_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut click_position: Local<ClickPosition>,
    mut full_click_happened_writer: MessageWriter<FullClickHappened>,
    mut scroll_happened_writer: MessageWriter<ScrollHappened>,
) {
    let window = window_query.single().unwrap();
    if mouse_input.any_just_released([MouseButton::Left, MouseButton::Right]) {
        _touch_event_handler(window, &mut click_position, ClickState::JustReleased, &mut full_click_happened_writer, &mut scroll_happened_writer);
    }
    else if mouse_input.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        _touch_event_handler(window, &mut click_position, ClickState::JustClicked, &mut full_click_happened_writer, &mut scroll_happened_writer);
    }
    else if mouse_input.any_pressed([MouseButton::Left, MouseButton::Right]) {
        _touch_event_handler(window, &mut click_position, ClickState::Hovering, &mut full_click_happened_writer, &mut scroll_happened_writer);
    }
}


pub fn handle_gesture_touch(
    touches: Res<Touches>,
    mut click_position: Local<ClickPosition>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut full_click_happened_writer: MessageWriter<FullClickHappened>,
    mut scroll_happened_writer: MessageWriter<ScrollHappened>,
) {
    let window = window_query.single().unwrap();

    // Handle just released touches first (they're not in iter(), only in iter_just_released())
    for finger in touches.iter_just_released() {
        let touch_pos = Some(finger.position());
        _touch_event_handler_with_pos(window, &mut click_position, ClickState::JustReleased, touch_pos, &mut full_click_happened_writer, &mut scroll_happened_writer);
        return;
    }

    // Handle active touches (pressed or hovering)
    for finger in touches.iter() {
        let touch_pos = Some(finger.position());
        if touches.just_pressed(finger.id()) {
            _touch_event_handler_with_pos(window, &mut click_position, ClickState::JustClicked, touch_pos, &mut full_click_happened_writer, &mut scroll_happened_writer);
        }
        else {
            _touch_event_handler_with_pos(window, &mut click_position, ClickState::Hovering, touch_pos, &mut full_click_happened_writer, &mut scroll_happened_writer);
        }
        return;
    }
}

pub fn advance_tick(
    mut popup_state: ResMut<PopupTimer>,
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    time: Res<Time>,
    button_colors: Res<ButtonColors>,
    // tile assets:
    tile_assets: Res<TileAssets>,
) {
    if let Some(timer) = popup_state.timer.as_mut() {
        timer.tick(time.delta());
        // If is finished:
        if timer.is_finished() {
            // Spawn a tutorial popup:
            match popup_state.popup_type {
                PopupType::Tutorial => {
                    make_tutorial_popup(popup_state.popup_text.clone(), popup_state.popup_text_2.clone(), &mut commands, &font_assets, &button_colors);
                }
                PopupType::Victory => {
                    make_victory_popup(popup_state.popup_text.clone(), popup_state.popup_text_2.clone(),  &mut commands, &font_assets, &button_colors, &tile_assets);
                    // make_tutorial_popup(popup_state.popup_text.clone(), &mut commands, &font_assets, &button_colors);
                }
            }
            // Remove the timer:
            popup_state.timer = None;
        }
    }
}


// Cleanup tutorial system: every time a button with component TutorialPopupButton is clicked, every entity with TutorialPopup is despawned
pub fn cleanup_popup(
    mut commands: Commands,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<ClosePopupButton>)>,
    tutorial_query: Query<Entity, With<Popup>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            for tutorial_id in tutorial_query.iter() {
                if let Ok(mut entity) = commands.get_entity(tutorial_id) {
                    entity.despawn();
                }
            }
            // commands.remove_resource::<PopupTimer>();
        }
    }
}




/////////////////////////////////////////////////////////////////////////////////////
// HELPER FUNCTIONS
/////////////////////////////////////////////////////////////////////////////////////


// ClickState {JustClicked, Hovering, JustReleased}

fn _touch_event_handler(
    window: &Window,
    click_position: &mut ClickPosition,
    state: ClickState,
    full_click_happened_writer: &mut MessageWriter<FullClickHappened>,
    scroll_happened_writer: &mut MessageWriter<ScrollHappened>
) {
    let pos = window.cursor_position();
    _touch_event_handler_with_pos(window, click_position, state, pos, full_click_happened_writer, scroll_happened_writer);
}

fn _touch_event_handler_with_pos(
    window: &Window,
    click_position: &mut ClickPosition,
    state: ClickState,
    pos: Option<Vec2>,
    full_click_happened_writer: &mut MessageWriter<FullClickHappened>,
    scroll_happened_writer: &mut MessageWriter<ScrollHappened>
) {
    let window_size = Vec2::new(window.width(), window.height());
    // If Some(Vec2), substract Window size:
    let clicked_pos = match pos {
        Some(pos) => Some(pos - window_size / 2.),
        None => None,
    };

    match state {
        ClickState::JustClicked => {
            click_position.clicked_pos = clicked_pos;
            click_position.last_hovered_pos = clicked_pos;
        }
        ClickState::Hovering => {
            if click_position.last_hovered_pos.is_some() && clicked_pos.is_some() {
                let last_pos = click_position.last_hovered_pos.unwrap();
                let new_pos = clicked_pos.unwrap();
                let delta = new_pos - last_pos;
                scroll_happened_writer.write(ScrollHappened{vx: delta.x, vy: delta.y});
            }
            click_position.last_hovered_pos = clicked_pos;
        }
        ClickState::JustReleased => {
            if click_position.clicked_pos.is_some() && click_position.last_hovered_pos.is_some() && distance(&click_position.clicked_pos.unwrap(), &click_position.last_hovered_pos.unwrap()) < 5.
            {
                info!("YEEEE Successfull Click!!! : pos{:?}", clicked_pos);
                full_click_happened_writer.write(FullClickHappened{pos: click_position.clicked_pos.unwrap()});
            }
            else{
                info!("NOO Aborted Click!!! : pos{:?}", clicked_pos);
            }
            click_position.clicked_pos = None;
            click_position.last_hovered_pos = None;
        }
    }
}


pub fn make_scrollbar(
    commands: &mut Commands,
    _assets: &TileAssets,
    font_assets: &FontAssets,
    font_size: f32,
    scroll_bar_limits: ScrollBarLimits,
    _button_colors: &ButtonColors,
    pleft: f32,
    pright: f32,
    ptop: f32,
    pbottom: f32,
    position_fraction: f32,
    type_: impl Bundle,
) -> Entity {
    // let arrow = assets.s_arrow_elem_rigth.clone();
    let back_id = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(pright - pleft),
            height: Val::Px(ptop - pbottom),
            margin: UiRect::all(Val::Auto),
            top: Val::Px(ptop),
            left: Val::Px(pleft),
            ..default()
        },
        ZIndex(5),
        BackgroundColor(Color::srgb(147. / 255.,  170. / 255.,  180. / 255.)),
    ))
    .insert(type_)
    .id();
    let text_id = commands.spawn((
        Text::new("SPEED"),
        TextFont {
            font: font_assets.fira_sans.clone(),
            font_size: font_size,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            margin: UiRect{left: Val::Percent(5.), ..default()},
            ..default()
        },
        ZIndex(20),
    )).id();

    // get the fraction from (scroll_bar_limits.current - min) / (scroll_bar_limits.max- min)
    // and apply it to ScrollBarPosition{ max_x: pright, min_x: pleft} to get the current_x:
    let _fraction = (scroll_bar_limits.current - scroll_bar_limits.min)
        / (scroll_bar_limits.max - scroll_bar_limits.min);
    let current_x = position_fraction * (pright - pleft);
    let current_val = _get_scrollbar_value(position_fraction, &scroll_bar_limits);
    let scroll_bar_limits = ScrollBarLimits {
        min: scroll_bar_limits.min,
        max: scroll_bar_limits.max,
        current: current_val,
        step: scroll_bar_limits.step,
    };
    let handle_id = commands.spawn(ScrollBarHandleBundle {
        node: Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(30.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            top: Val::Px(0.),
            left: Val::Px(current_x),
            ..default()
        },
        // texture: UiImage(arrow),
        background_color: BackgroundColor(Color::srgb(0.9, 0.90, 0.90)),
        scroll_bar_limits: scroll_bar_limits,
        scroll_bar_position: ScrollBarPosition {
            max_x: pright,
            min_x: pleft,
            current_x: current_x,
            step_x: 1.,
        },
        z_index: ZIndex(10),
        ..default()
    })
    .id();
    commands.entity(handle_id).add_children(&[text_id]);// add the child to the parent
    commands.entity(back_id).add_children(&[handle_id]); // add the child to the parent
    return back_id;
}

pub fn make_button(
    text: String,
    commands: &mut Commands,
    font_assets: &FontAssets,
    button_colors: &ButtonColors,
    font_size: f32,
    pleft: f32,
    pright: f32,
    ptop: f32,
    pbottom: f32,
    type1: impl Bundle,
    type2: Option<impl Bundle>,
) -> Entity {
    let mut ec = commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(pright - pleft),
            height: Val::Px(ptop - pbottom),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            top: Val::Px(ptop),
            left: Val::Px(pleft),
            ..default()
        },
        BackgroundColor(button_colors.normal),
        ButtonData{text: text.clone()},
    ));
    ec.with_children(|parent| {
        parent.spawn((
            Text::new(text.clone()),
            TextFont {
                font: font_assets.fira_sans.clone(),
                font_size: font_size,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
    });
    ec.insert(type1);
    if let Some(type2) = type2 {
        ec.insert(type2);
    }
    return ec.id();
}


pub fn make_rect_with_colored_text(
    text1: String,
    text2: String,
    _textcolor: Color,
    commands: &mut Commands,
    font_assets: &FontAssets,
    button_colors: &ButtonColors,
    font_size: f32,
    pleft: f32,
    pright: f32,
    ptop: f32,
    pbottom: f32,
    type1: impl Bundle,
    type2: Option<impl Bundle>,
) -> Entity {
    let mut ec = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(pright - pleft),
            height: Val::Px(ptop - pbottom),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            top: Val::Px(ptop),
            left: Val::Px(pleft),
            ..default()
        },
        BackgroundColor(button_colors.normal),
    ));
    let combined_text = format!("{}{}", text1, text2);
    ec.with_children(|parent| {
        parent.spawn((
            Text::new(combined_text),
            TextFont {
                font: font_assets.fira_sans.clone(),
                font_size: font_size,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextLayout::new_with_justify(Justify::Center),
        ));
    });
    ec.insert(type1);
    if let Some(type2) = type2 {
        ec.insert(type2);
    }
    return ec.id();
}


pub fn make_text(
    text: String,
    commands: &mut Commands,
    font_assets: &FontAssets,
    _button_colors: &ButtonColors,
    font_size: f32,
    pleft: f32,
    pright: f32,
    ptop: f32,
    pbottom: f32,
    type1: impl Bundle,
    type2: Option<impl Bundle>,
) -> Entity {
    let mut ec = commands.spawn(
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(pright - pleft),
            height: Val::Px(ptop - pbottom),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            top: Val::Px(ptop),
            left: Val::Px(pleft),
            right: Val::Px(pright),
            bottom: Val::Px(pbottom),
            ..default()
        }
    );
    ec.with_children(|parent| {
        parent.spawn((
            Text::new(text),
            TextFont {
                font: font_assets.fira_sans.clone(),
                font_size: font_size,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextLayout::new_with_justify(Justify::Center),
            TextElem{},
        ));
    });
    ec.insert(type1);
    if let Some(type2) = type2 {
        ec.insert(type2);
    }
    return ec.id();
}


pub fn make_tutorial_popup(
    text: String,
    text_2: Option<String>,
    commands: &mut Commands,
    font_assets: &FontAssets,
    button_colors: &ButtonColors,
)
{
    let font_size = 14.;
// A fixed positioned popup rectangle with sides at 10% to 90& of screen width, and 45% to 55% of screen height. nside, print text.
    let popup_id = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(80.),
            height: Val::Percent(25.),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            top: Val::Percent(35.),
            left: Val::Percent(10.),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        Popup{},
        MainGameBotton{},
    )).id();
    let text_id = commands.spawn((
        Text::new(text),
        TextFont {
            font: font_assets.fira_sans.clone(),
            font_size: font_size,
            ..default()
        },
        TextColor(Color::srgba(0.9, 0.9, 0.9, 0.9)),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            margin: UiRect{top: Val::Percent(-13.), ..default()},
            ..default()
        },
    )).id();
    commands.entity(popup_id).add_children(&[text_id]);// add the child to the parent
    if let Some(text_2_) = text_2 {
        let text2_id = commands.spawn((
            Text::new(text_2_),
            TextFont {
                font: font_assets.fira_sans.clone(),
                font_size: 8.,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            TextLayout::new_with_justify(Justify::Center),
            Node {
                position_type: PositionType::Absolute,
                // Button (centered horizontally, 40% of width., bottom vertically)
                bottom: Val::Auto,
                left: Val::Auto,
                right: Val::Auto,
                top: Val::Percent(60.),
                ..default()
            },
        )).id();
        commands.entity(popup_id).add_children(&[text2_id]);// add the child to the parent
    }

    let but_id = commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            // Button (centered horizontally, 40% of width., bottom vertically)
            bottom: Val::Percent(5.),
            left: Val::Percent(35.),
            right: Val::Percent(35.),
            top: Val::Percent(75.),
            ..default()
        },
        BackgroundColor(button_colors.normal),
        ClosePopupButton{},
    )).id();
    commands.entity(popup_id).add_children(&[but_id]);// add the child to the parent

    let but_text_id = commands.spawn((
        Text::new("GOT IT"),
        TextFont {
            font: font_assets.fira_sans.clone(),
            font_size: font_size * 0.66,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    )).id();
    commands.entity(but_id).add_children(&[but_text_id]);// add the child to the parent
}


pub fn make_victory_popup(
    text: String,
    text_2: Option<String>,
    commands: &mut Commands,
    font_assets: &FontAssets,
    button_colors: &ButtonColors,
    tile_assets: &TileAssets,
)
{
    let font_size = 14.;
// A fixed positioned popup rectangle with sides at 10% to 90& of screen width, and 45% to 55% of screen height. nside, print text.
    let popup_id = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            top: Val::Percent(30.),
            left: Val::Percent(5.),
            right: Val::Percent(5.),
            bottom: Val::Percent(35.),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        ZIndex(10),
        Popup{},
        MainGameBotton{},
    )).id();

    // Build combined text with sections
    let combined_text = if let Some(text2_) = &text_2 {
        format!("YOU SOLVED IT!\n\n{}{}", text, text2_)
    } else {
        format!("YOU SOLVED IT!\n\n{}", text)
    };

    let text_id = commands.spawn((
        Text::new(combined_text),
        TextFont {
            font: font_assets.fira_sans.clone(),
            font_size: font_size,
            ..default()
        },
        TextColor(Color::srgba(0.9, 0.9, 0.9, 0.9)),
        TextLayout::new_with_justify(Justify::Left),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(55.),
            top: Val::Percent(18.),
            ..default()
        },
    )).id();
    commands.entity(popup_id).add_children(&[text_id]);// add the child to the parent

    let but_id_close = commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            bottom: Val::Percent(5.),
            left: Val::Percent(12.),
            right: Val::Percent(55.),
            top: Val::Percent(75.),
            ..default()
        },
        BackgroundColor(button_colors.normal),
        ClosePopupButton{},
    )).id();
    commands.entity(popup_id).add_children(&[but_id_close]);// add the child to the parent

    let but_next_id = commands.spawn((
        Text::new("REPLAY SOLUTION"),
        TextFont {
            font: font_assets.fira_sans.clone(),
            font_size: font_size * 0.9,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
    )).id();
    commands.entity(but_id_close).add_children(&[but_next_id]);// add the child to the parent

    let but_id_nextlevel = commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center, // I have to say, this was cool ....
            bottom: Val::Percent(5.),
            left: Val::Percent(55.),
            right: Val::Percent(12.),
            top: Val::Percent(75.),
            ..default()
        },
        BackgroundColor(button_colors.normal),
        NextLevelButton{},
    )).id();
    commands.entity(popup_id).add_children(&[but_id_nextlevel]);// add the child to the parent

    let but_nexttext_id = commands.spawn((
        Text::new("NEXT LEVEL"),
        TextFont {
            font: font_assets.fira_sans.clone(),
            font_size: font_size * 0.9,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
    )).id();
    commands.entity(but_id_nextlevel).add_children(&[but_nexttext_id]);// add the child to the parent

    let tick_id = commands.spawn((
        ImageNode::new(tile_assets.tick.clone()),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(17.),
            top: Val::Percent(25.),
            right: Val::Auto,
            bottom: Val::Auto,
            ..default()
        },
        Transform::default().with_scale(Vec3 { x: 0., y: 0., z: 10. }),
        TweenAnim::new(Tween::new(
                EaseFunction::CubicIn, Duration::from_millis(400 as u64),
                TransformScaleLens {start: Vec3 { x: 0., y: 0., z: 10. }, end: Vec3 { x: 1.9, y: 1.9, z: 10. },},
            )
        )
    )).id();
    commands.entity(popup_id).add_children(&[tick_id]);// add the child to the parent

}


// If you ever ACTUALLY needed to animate the style > scale property:
//  // In main  // .add_system(component_animator_system::<Style>) 
// // In the tweening: // //MyUiScaleLens{start: Val::Percent(0.), end: Val::Percent(100.),},
// fn _lerp_val(start: &Val, end: &Val, ratio: f32) -> Val {
//     match (start, end) {
//         (Val::Percent(start), Val::Percent(end)) => {
//             Val::Percent((end - start).mul_add(ratio, *start))
//         }
//         (Val::Px(start), Val::Px(end)) => Val::Px((end - start).mul_add(ratio, *start)),
//         _ => *start,
//     }
// }
// struct MyUiScaleLens {
//     start: Val,
//     end: Val,
// }
// impl Lens<Style> for MyUiScaleLens {
//     fn lerp(&mut self, target: &mut Style, ratio: f32) {
//         target.size = Size::new(_lerp_val(&self.start, &self.end, ratio), Val::Auto);
//         println!("CALLED !! lerp: {:?}", target.size);
//     }
// }



pub fn make_border(
    commands: &mut Commands,
    color: Color,
    // an arg implementing both Bundle and Clone:
    component_to_add: impl Bundle + Clone,
) {
    // Draw a UiImage that has only a border, no fill. The border is yellow. It should be AS BIG AS THE SCREEN.
    // Do it by creating 4 different narrow rectangles, at each side of the screen:
    // Left rectangle:
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(4.),
                height: Val::Percent(100.),
                // Left of the screen, using percentages:
                top: Val::Px(0.),
                left: Val::Px(0.),
                ..default()
            },
            // yellow color:
            BackgroundColor(color),
        ))
        .insert(BorderElem)
        .insert(component_to_add.clone());
    // Right rectangle:
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(4.),
                height: Val::Percent(100.),
                // Right of the screen, using percentages:
                top: Val::Px(0.),
                right: Val::Px(0.),
                ..default()
            },
            // yellow color:
            BackgroundColor(color),
        ))
        .insert(BorderElem)
        .insert(component_to_add.clone());
    // Top rectangle:
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Px(4.),
                // Top of the screen, using percentages:
                top: Val::Px(0.),
                left: Val::Px(0.),
                ..default()
            },
            // yellow color:
            BackgroundColor(color),
        ))
        .insert(BorderElem)
        .insert(component_to_add.clone());
    // Bottom rectangle:
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Px(4.),
                // Bottom of the screen, using percentages:
                bottom: Val::Px(0.),
                left: Val::Px(0.),
                ..default()
            },
            // yellow color:
            BackgroundColor(color),
        ))
        .insert(BorderElem)
        .insert(component_to_add.clone());
}


fn _get_scrollbar_value(fraction: f32, sblimits: &ScrollBarLimits) -> f32 {
    let fraction = (1. - fraction).powf(4.);
                
    // New value using the fraction with  ScrollBarLimits { pub max: f32, pub min: f32, pub current: f32, pub step: f32,}:
    let newval = sblimits.min + (sblimits.max - sblimits.min) * fraction;
    // Round newval to the nearest step:
    let newval = (newval / sblimits.step).round() * sblimits.step;
    newval
}
