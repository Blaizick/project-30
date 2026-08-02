
pub mod interop_functions {
    use std::ffi::OsString;

use bevy_ecs::entity::Entity;
use glam::Vec2;
use num::FromPrimitive;

use crate::{log::log::{LogChannel, error, log, verbose_error, verbose_log, verbose_warning, warning}, *};

    pub extern "C" fn u_test_call() {
        log(LogChannel::Default, "Unmanaged Test Call!".to_string());
    }
    pub extern "C" fn m_test_call_dummy() {
        panic!("Callback not initialized!");
    }
    pub extern "C" fn m_on_object_clicked_dummy(_: ObjectIndex) {
        panic!("Callback not initialized!");
    }
    pub extern "C" fn u_create_canvas() -> ObjectIndex {
        let state = get_mut_state();
        let mut array = state.object_manager.get_mut_array_for_type::<Entity>();
        let obj_id = create_object(&mut state.ecs_world, &mut array, &mut state.entity_map);
        let entity_id = array.get_ref_by_obj_id(obj_id).unwrap().clone();
        state.ecs_world.entity_mut(entity_id.clone()).insert((PosComp { pos: Vec2::new(0.0, 0.0), }, SizeComp { size: Vec2::new(100.0, 100.0) } ));
        return obj_id;
    }
    pub extern "C" fn u_create_image() -> ObjectIndex {
        let state = get_mut_state();
        let mut array = state.object_manager.get_mut_array_for_type::<Entity>();
        let obj_id = create_object(&mut state.ecs_world, &mut array, &mut state.entity_map);
        let entity_id = array.get_ref_by_obj_id(obj_id).unwrap().clone();
        state.ecs_world.entity_mut(entity_id).insert((
            PosComp { pos: Vec2::new(0.0, 0.0), }, 
            SizeComp { size: Vec2::new(100.0, 100.0), },
            TextureComp { texture_index: state._1px_texture },
            ColorComp { color: Color::WHITE, },
        ));
        return obj_id;
    }
    pub extern "C" fn u_load_texture(path: NativeString) -> ObjectIndex {
        let path = PathBuf::from_str(&path.to_string()).unwrap();
        let texture = load_texture(&path, &mut get_mut_state().object_manager);
        return texture;    
    }
    pub extern "C" fn u_set_texture(object_index: ObjectIndex, texture_index: ObjectIndex) {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut texture_comp = entity.get_mut::<TextureComp>().unwrap();
        texture_comp.texture_index = texture_index;
    }
    pub extern "C" fn u_get_executing_directory() -> NativeString {
        let state = unsafe { &mut *GLOBAL_STATE.get().unwrap().0.get() };
        return NativeString::from_string(&state.exe_dir.clone().into_os_string().into_string().unwrap());
    }

    pub extern "C" fn u_set_position(object_index: ObjectIndex, position: Vec2) {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut pos_comp = entity.get_mut::<PosComp>().unwrap();
        pos_comp.pos = position;
    }
    pub extern "C" fn u_set_size(object_index: ObjectIndex, size: Vec2) {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut size_comp = entity.get_mut::<SizeComp>().unwrap();
        size_comp.size = size;
    }
    pub extern "C" fn u_set_color(object_index: ObjectIndex, color: Color) {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut color_comp = entity.get_mut::<ColorComp>().unwrap();
        color_comp.color = color;
    }

    pub extern "C" fn u_get_texture(object_index: ObjectIndex) -> ObjectIndex {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let comp = entity.get::<TextureComp>().unwrap();
        return comp.texture_index;
    }
    pub extern "C" fn u_get_position(object_index: ObjectIndex) -> Vec2 {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let comp = entity.get::<PosComp>().unwrap();
        return comp.pos;
    }
    pub extern "C" fn u_get_size(object_index: ObjectIndex) -> Vec2 {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let comp = entity.get::<SizeComp>().unwrap();
        return comp.size;
    }
    pub extern "C" fn u_get_color(object_index: ObjectIndex) -> Color {
        let state = get_mut_state();
        let entity_index = get_mut_entity_array().buffer[object_index.index as usize].unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let comp = entity.get::<ColorComp>().unwrap();
        return comp.color;
    }
    pub extern "C" fn u_get_texture_size(texture_index: ObjectIndex) -> UVec2 {
        let texture = get_mut_texture_array().buffer[texture_index.index as usize].as_ref().unwrap();
        return texture.size;
    }
    pub extern "C" fn u_create_label() -> ObjectIndex {
        let state = get_mut_state();
        let object_index = create_object(&mut state.ecs_world, get_mut_entity_array(), &mut state.entity_map);
        let entity_index = get_mut_entity_array().get_mut_by_obj_id(object_index).unwrap();
        let mut entity = state.ecs_world.entity_mut(*entity_index);
        entity.insert((
            PosComp { pos: Vec2::new(0.0, 0.0), }, 
            FontComp { font: state.default_font_index, font_size: 18.0, },
            TextComp { text: "Lorem Impsum".to_string(), },
            ColorComp { color: Color::WHITE, },
        ));
        object_index
    }
    pub extern "C" fn u_set_text(object_index: ObjectIndex, text: NativeString) {
        let state = get_mut_state();
        let entity_index = state.object_manager.get_ref_array_for_type_unsafe::<Entity>().get_by_obj_id(object_index).unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut text_comp = entity.get_mut::<TextComp>().unwrap();
        text_comp.text = text.to_string();
    }
    pub extern "C" fn u_get_text(object_index: ObjectIndex) -> NativeString {
        let state = get_ref_state();
        let entity_index = state.object_manager.get_ref_array_for_type_unsafe::<Entity>().get_by_obj_id(object_index).unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let text_comp = entity.get::<TextComp>().unwrap();
        NativeString::from_string(&text_comp.text)
    }
    pub extern "C" fn u_set_font(object_index: ObjectIndex, font: ObjectIndex) {
        let state = get_mut_state();
        let entity_index = state.object_manager.get_ref_array_for_type_unsafe::<Entity>().get_by_obj_id(object_index).unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut font_comp = entity.get_mut::<FontComp>().unwrap();
        font_comp.font = font;
    }
    pub extern "C" fn u_get_font(object_index: ObjectIndex) -> ObjectIndex {
        let state = get_ref_state();
        let entity_index = state.object_manager.get_ref_array_for_type_unsafe::<Entity>().get_by_obj_id(object_index).unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let font_comp = entity.get::<FontComp>().unwrap();
        font_comp.font
    }
    pub extern "C" fn u_set_font_size(object_index: ObjectIndex, font_size: f32) {
        let state = get_mut_state();
        let entity_index = state.object_manager.get_ref_array_for_type_unsafe::<Entity>().get_by_obj_id(object_index).unwrap();
        let mut entity = state.ecs_world.entity_mut(entity_index);
        let mut font_comp = entity.get_mut::<FontComp>().unwrap();
        font_comp.font_size = font_size;
    }
    pub extern "C" fn u_get_font_size(object_index: ObjectIndex) -> f32 {
        let state = get_ref_state();
        let entity_index = state.object_manager.get_ref_array_for_type_unsafe::<Entity>().get_by_obj_id(object_index).unwrap();
        let entity = state.ecs_world.entity(entity_index);
        let font_comp = entity.get::<FontComp>().unwrap();
        font_comp.font_size
    }
    pub extern "C" fn u_load_font(path: NativeString) -> ObjectIndex {
        let path_buf = PathBuf::from(OsString::from(path.to_string()));
        load_font(&path_buf, &mut get_mut_state().object_manager)
    }
    pub extern "C" fn u_get_mouse_position() -> Vec2 {
        Vec2::new(0.0, 0.0)
    }
    pub extern "C" fn u_create_button() -> ObjectIndex {
        let state = get_mut_state();
        let mut array = state.object_manager.get_mut_array_for_type::<Entity>();
        let obj_id = create_object(&mut state.ecs_world, &mut array, &mut state.entity_map);
        let entity_id = array.get_ref_by_obj_id(obj_id).unwrap().clone();
        state.ecs_world.entity_mut(entity_id).insert((
            PosComp { pos: Vec2::new(0.0, 0.0), }, 
            SizeComp { size: Vec2::new(100.0, 100.0), },
            TextureComp { texture_index: state._1px_texture },
            ColorComp { color: Color::WHITE, },
            OnClickComp {},
        ));
        obj_id
    }

    pub extern "C" fn u_get_function_pointer(name: NativeString) -> *const c_void {
        get_mut_state().function_map.get_function(&name.to_string()).unwrap()
    }

    pub extern "C" fn u_log(channel: u64, message: NativeString){
        log(FromPrimitive::from_u64(channel).unwrap(), message.to_string());
    }
    pub extern "C" fn u_verbose_log(channel: u64, message: NativeString){
        verbose_log(FromPrimitive::from_u64(channel).unwrap(), message.to_string());
    }
    pub extern "C" fn u_warning(channel: u64, message: NativeString){
        warning(FromPrimitive::from_u64(channel).unwrap(), message.to_string());
    }
    pub extern "C" fn u_verbose_warning(channel: u64, message: NativeString){
        verbose_warning(FromPrimitive::from_u64(channel).unwrap(), message.to_string());
    }
    pub extern "C" fn u_error(channel: u64, message: NativeString){
        error(FromPrimitive::from_u64(channel).unwrap(), message.to_string());
    }
    pub extern "C" fn u_verbose_error(channel: u64, message: NativeString){
        verbose_error(FromPrimitive::from_u64(channel).unwrap(), message.to_string());
    }

    
}