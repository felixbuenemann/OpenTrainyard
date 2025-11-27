use bevy::prelude::*;
use bevy_tweening::*;
use bevy_tweening::lens::*;

use crate::simulator::*;


use crate::board::*;
use crate::logic::*;
use crate::loading::TrainAssets;


////////////////////////////////////////////////////////////////////////////////////
// COMPONENTS
/////////////////////////////////////////////////////////////////////////////////////


#[derive(Bundle)]
pub struct TrainBundle{
    pub train: Train,

    // In Bevy 0.15, Sprite contains the image handle directly
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility, // User indication of whether an entity is visible
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
impl Default for TrainBundle {
    fn default() -> Self {
        Self {
            train: Train { c: Colorz::BROWN_, pos: Pos { px: 0, py: 0, side: Side::T_, going_in: true, towards_side: Some(Side::B_) } },
            sprite: default(),
            transform: default(),
            global_transform: default(),
            visibility: default(),
            inherited_visibility: default(),
            view_visibility: default(),
        }
    }
}

#[derive(Component)]
pub struct CosmeticTrain {} // For vfx of train disappearing


/////////////////////////////////////////////////////////////////////////////////////
// EVENTS
/////////////////////////////////////////////////////////////////////////////////////

#[derive(Message)]
pub struct SpawnCosmeticTrainEvent {
    pub train: Train,
    pub board_id: Entity,
}


/////////////////////////////////////////////////////////////////////////////////////
// SYSTEMS
/////////////////////////////////////////////////////////////////////////////////////


pub fn respawn_trains(
    mut commands: Commands,
    trains_q: Query<Entity, (With<Train>, Without<CosmeticTrain>)>,
    train_assets: Res<TrainAssets>,
    tick_params: Res<TicksInATick>,
    board_q: Query<(Entity, &BoardDimensions, &BoardTileMap, &BoardTickStatus), (With<Board>, Changed<BoardTileMap>)>,
) {
    for (board_id, board_dimensions, board_tilemap, board_tick_status) in board_q.iter() {
        // Despawn existing trains
        for train_entity in trains_q.iter() {
            if let Ok(mut train) = commands.get_entity(train_entity) {
                train.despawn();
            }
        }
        // Spawn new trains
        for train in board_tilemap.current_trains.iter() {
            let child_id = make_train(*train, &mut commands, &train_assets, &board_dimensions, board_tick_status.current_tick_in_a_tick as f32 / tick_params.ticks as f32);
            commands.entity(board_id).add_children(&[child_id]);
        }
    }
}

pub fn move_trains(
    mut trains_q: Query<(&Train, &mut Transform), Without<CosmeticTrain>>,
    tick_params: Res<TicksInATick>,
    board_q: Query<(&BoardDimensions, &BoardTickStatus), With<Board>>,
) {
    for (board_dimensions, board_tick_status) in board_q.iter() {
        for (train, mut transform) in trains_q.iter_mut() {
            *transform = get_train_transform(*train, board_dimensions, (board_tick_status.current_tick_in_a_tick as f32) / (tick_params.ticks as f32));
        }
    }
}

// >> With more than 1 board, you would need this:
// `children` is a collection of Entity IDs
// for &child in children.iter() {
//     if let Ok((train_entity, train)) = trains_q.get(child)
//     {
//         let mut board_entity = commands.entity(board_id);  // Get entity by id:
//         board_entity.remove_children(&[train_entity]);
//     }
// }




pub fn spawn_cosmetic_trains_event(
    mut commands: Commands,
    mut spawn_cosmetic_train_event_reader: MessageReader<SpawnCosmeticTrainEvent>,
    train_assets: Res<TrainAssets>,
    tick_params: Res<TicksInATick>,
    board_q: Query<(Entity, &BoardDimensions, &BoardTickStatus), With<Board>>,
) {
    for event in spawn_cosmetic_train_event_reader.read() {
        // Get the board data using ev.board_id on the query:
        let (_board_entity, board_dimensions, board_tick_status) = board_q.get(event.board_id).unwrap();

        let train = event.train;
        let board_id = event.board_id;
        let child_id = make_train_cosmetic(train, &mut commands, &train_assets, &board_dimensions, board_tick_status.current_tick_in_a_tick as f32 / tick_params.ticks as f32);
        commands.entity(board_id).add_children(&[child_id]);// add the child to the parent
    }
}


pub fn delete_cosmetic_trains_with_finished_animations(
    mut commands: Commands,
    trains_q: Query<(Entity, &Train, &TweenAnim), With<CosmeticTrain>>,
) {
    for (train_entity, _, animator) in trains_q.iter() {
        // Check if animation is completed by comparing elapsed time with total duration
        let tweenable = animator.tweenable();
        if let bevy_tweening::TotalDuration::Finite(total) = tweenable.total_duration() {
            if tweenable.elapsed() >= total {
                if let Ok(mut train) = commands.get_entity(train_entity) {train.despawn();}
            }
        }
    }
}







/////////////////////////////////////////////////////////////////////////////////////
// HELPER FUNCTIONS
/////////////////////////////////////////////////////////////////////////////////////





fn get_train_image(train_assets: &TrainAssets, color: Colorz) -> Handle<Image> {
    match color {
        Colorz::RED_ => train_assets.train_red.clone(),
        Colorz::BLUE_ => train_assets.train_blue.clone(),
        Colorz::YELLOW_ => train_assets.train_yellow.clone(),
        Colorz::ORANGE_ => train_assets.train_orange.clone(),
        Colorz::GREEN_ => train_assets.train_green.clone(),
        Colorz::PURPLE_ => train_assets.train_purple.clone(),
        Colorz::BROWN_ => train_assets.train_brown.clone(),
    }
}


fn get_train_transform(t:Train, board: &BoardDimensions, tick_rateo: f32) -> Transform {
    let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 3.0));
    let in_side: Side = t.pos.side;
    let out_side: Side = match t.pos.towards_side {
        Some(side) => side,
        None => flip_side(in_side),
    };

    let angle = tick_rateo * 0.5 * std::f32::consts::PI;

    let (x, y, train_angle) =  match (in_side, out_side) {
        // For STRAIGHT tracks (in/out is right/left or top/bottom), the train should go from left to rigth or right to left. Use tick_rateo to get the fraction of the way.
        // ASSUME THAT the tile has side 1, and the origin is in the TOP LEFT corner ( x=0 is leftmost, Y=0 is topmost)
        (Side::L_, Side::R_) => {(tick_rateo, 0.5, 0.)},
        (Side::R_, Side::L_) => {(1. - tick_rateo, 0.5, - std::f32::consts::PI)},
        (Side::T_, Side::B_) => {(0.5, 1. - tick_rateo, - std::f32::consts::PI / 2.)},
        (Side::B_, Side::T_) => {(0.5, tick_rateo, std::f32::consts::PI / 2.)},
        // For CURVED tracks, the train should do a CURVED arc from one side of the tile to the other, PIVOTING AROUND THE CONER, that is a CONCAVE arc towards the center of the tile.
        (Side::R_, Side::T_) => {( 1.- 0.5*angle.sin(), 1. - 0.5 * angle.cos(), - angle + std::f32::consts::PI)},
        (Side::T_, Side::L_) => {(0.5* angle.cos(), 1. - 0.5 * angle.sin(), -angle - std::f32::consts::PI /2.)},
        (Side::L_, Side::B_) => {( 0.5*angle.sin(), 0.5 * angle.cos(), - angle)},
        (Side::B_, Side::R_) => {(1. - 0.5* angle.cos(), 0.5*angle.sin(), - angle + std::f32::consts::PI /2.)},
        (Side::T_, Side::R_) => {(1. - 0.5* angle.cos(), 1.- 0.5 * angle.sin(), angle - std::f32::consts::PI /2.)},
        (Side::R_, Side::B_) => {(1. - 0.5*angle.sin(), 0.5 * angle.cos(), angle + std::f32::consts::PI)},
        (Side::B_, Side::L_) => {(0.5* angle.cos(), 0.5 * angle.sin(), angle + std::f32::consts::PI /2.)},
        (Side::L_, Side::T_) => {( 0.5*angle.sin(), 1. - 0.5 * angle.cos(), angle)},
        _ => {panic!("WTF")}
        };
        transform.translation.x = (x + t.pos.px as f32) * board.tile_size - t.pos.px as f32; // This last term is because of the fact that tiles are overlapped by 1px!
        transform.translation.y = (y + (6 - t.pos.py)as f32) * board.tile_size - (6 - t.pos.py) as f32; // This last term is because of the fact that tiles are overlapped by 1px!
        transform.rotation = Quat::from_rotation_z( train_angle);
        
        transform.scale = Vec3{x: 0.68, y: 0.68, z: 3.};
        
        return transform;
}


pub fn make_train(train: Train, commands: &mut Commands, train_assets: &TrainAssets, board_dimensions: &BoardDimensions, tick_rateo: f32) -> Entity {
    let child = commands.spawn(TrainBundle {
        train: train,
        sprite: Sprite {
            image: get_train_image(train_assets, train.c),
            ..default()
        },
        transform: get_train_transform(train, board_dimensions, tick_rateo),
        ..default()
    });
    return child.id();
}

pub fn make_train_cosmetic(train: Train, commands: &mut Commands, train_assets: &TrainAssets, board_dimensions: &BoardDimensions, tick_rateo: f32) -> Entity {
    let transform = get_train_transform(train, board_dimensions, tick_rateo);
    let scale = transform.scale;

    let t1 = Tween::new(
        EaseFunction::CubicOut, std::time::Duration::from_millis(400 as u64),
        TransformScaleLens {start: scale, end: scale * 1.7},
    );
    let t2 = Tween::new(
        EaseFunction::CubicOut, std::time::Duration::from_millis(400 as u64),
        SpriteColorLens {start: Color::srgba(1., 1., 1., 0.8), end: Color::srgba(1., 1., 1., 0.),},
    );

    let child = commands.spawn((
        TrainBundle {
            train: train,
            sprite: Sprite {
                image: get_train_image(train_assets, train.c),
                ..default()
            },
            transform,
            ..default()
        },
        CosmeticTrain{},
        TweenAnim::new(t1),
        TweenAnim::new(t2),
    ));

    return child.id();
}

