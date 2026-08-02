use core::slice;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::{CString};
use std::os::raw::c_void;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs};
use std::sync::{OnceLock};
use bevy_ecs::component::Component;
use bevy_ecs::entity::{Entity, EntityIndex};
use bevy_ecs::schedule::Schedule;
use bevy_ecs::system::Query;
use bevy_ecs::world::World;
use glam::{Mat4, UVec2, Vec2};
use netcorehost::nethost::load_hostfxr;
use netcorehost::pdcstr;
use netcorehost::pdcstring::{PdCString};
use sdl3::EventPump;
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseState;
use sdl3::video::{GLProfile, Window};

use crate::font::font::{BatchItem2, Batcher2, Font, Rect, Sprite, SpriteAtlas, load_font};
use crate::function_map::function_map::FunctionMap;
use crate::interop_functions::interop_functions::{*};
use crate::log::log::{GLOBAL_LOGGER, LogChannel, init_logger, log};
// use crate::interop_functions::{}

mod font;
mod gl_macro;
mod interop_functions;
mod log;
mod function_map;

#[repr(C)]
pub struct UnmanagedCallbacks {
    pub u_test_call: extern "C" fn(),
    pub u_create_canvas: extern "C" fn() -> ObjectIndex,
    pub u_create_image: extern "C" fn() -> ObjectIndex,
    pub u_load_texture: extern "C" fn(path: NativeString) -> ObjectIndex,
    pub u_set_texture: extern "C" fn(object_index: ObjectIndex, texture_index: ObjectIndex),
    pub u_get_executing_directory: extern "C" fn() -> NativeString,
    pub u_set_position: extern "C" fn(object_index: ObjectIndex, position: Vec2),
    pub u_set_size: extern "C" fn(object_index: ObjectIndex, size: Vec2),
    pub u_set_color: extern "C" fn(object_index: ObjectIndex, color: Color),
    pub u_get_texture: extern "C" fn(object_index: ObjectIndex) -> ObjectIndex,
    pub u_get_position: extern "C" fn(object_index: ObjectIndex) -> Vec2,
    pub u_get_size: extern "C" fn(object_index: ObjectIndex) -> Vec2,
    pub u_get_color: extern "C" fn(object_index: ObjectIndex) -> Color,
    pub u_get_texture_size: extern "C" fn(texture_index: ObjectIndex) -> UVec2,
    pub u_create_label: extern "C" fn() -> ObjectIndex,
    pub u_set_text: extern "C" fn(object_index: ObjectIndex, text: NativeString),
    pub u_get_text: extern "C" fn(object_index: ObjectIndex) -> NativeString,
    pub u_set_font: extern "C" fn(object_index: ObjectIndex, font: ObjectIndex),
    pub u_get_font: extern "C" fn(object_index: ObjectIndex) -> ObjectIndex,
    pub u_set_font_size: extern "C" fn(object_index: ObjectIndex, font_size: f32),
    pub u_get_font_size: extern "C" fn(object_index: ObjectIndex) -> f32,
    pub u_load_font: extern "C" fn(path: NativeString) -> ObjectIndex,
    pub u_get_mouse_position: extern "C" fn() -> Vec2,
    pub u_create_button: extern "C" fn() -> ObjectIndex,
    pub u_get_function_pointer: extern "C" fn(function_name: NativeString) -> *const c_void,
}

#[repr(C)]
pub struct NativeString {
    pub ptr: *const u8,
    pub len: u32,
}

impl NativeString {
    pub fn from_string(s: &String) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len() as u32,
        }
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len() as u32,
        }
    }

    pub fn to_string(&self) -> String {
        unsafe {
            let bytes = slice::from_raw_parts(self.ptr, self.len as usize);
            str::from_utf8(bytes).unwrap().to_owned()
        }
    }
}

#[repr(C)]
pub struct ManagedCallbacks {
    pub m_test_call: extern "C" fn(),
    pub m_on_element_clicked: extern "C" fn(element: ObjectIndex),
}
impl Default for ManagedCallbacks {
    fn default() -> Self {
        ManagedCallbacks { 
            m_test_call: m_test_call_dummy, 
            m_on_element_clicked: m_on_object_clicked_dummy, 
        }
    }
}

type InitializeFromEngineFn = unsafe extern "C" fn(callbacks: UnmanagedCallbacks, 
    callbacks_size: i32, 
    managed_callbacks: *mut ManagedCallbacks, 
    managed_callbacks_size: *mut i32);


#[derive(Component)]
pub struct FontComp {
    pub font: ObjectIndex,
    pub font_size: f32,
}
#[derive(Component)]
pub struct TextComp {
    pub text: String,
}

pub fn get_mut_entity_array() -> &'static mut Array32<Entity> {
    get_array::<Entity>()
}
pub fn get_mut_texture_array() -> &'static mut Array32<Texture> {
    get_array::<Texture>()
}
pub fn get_mut_sprite_array() -> &'static mut Array32<Sprite> {
    get_array::<Sprite>()
}
pub fn get_mut_sprite_atlas_array() -> &'static mut Array32<SpriteAtlas> {
    get_array::<SpriteAtlas>()
}
pub fn get_mut_font_array() -> &'static mut Array32<Font> {
    get_array::<Font>()
}
pub fn get_mut_shader_array() -> &'static mut Array32<Shader> {
    get_array::<Shader>()
}
pub fn get_mut_state() -> &'static mut State {
    unsafe {&mut *GLOBAL_STATE.get().unwrap().0.get()}
}
pub fn get_ref_state() -> &'static State {
    unsafe {&*GLOBAL_STATE.get().unwrap().0.get()}
}
pub fn get_array<T>() -> &'static mut Array32<T> {
    get_mut_state().object_manager.get_mut_array_for_type::<T>()
}

pub fn get_mouse_position(mouse_state: MouseState) -> Vec2 {
    Vec2::new(mouse_state.x(), mouse_state.y())
}

pub fn load_texture(path: &PathBuf, object_manager: &mut ObjectManager) -> ObjectIndex {
    let img = image::open(path).expect("Failed to open image");
    let img  = img.into_rgba8();
    let (width, height) = img.dimensions();
    let pixels = img.into_raw();

    let wrapped_texture_id = create_texture(&pixels, UVec2::new(width, height), object_manager, gl::RGBA);

    log(LogChannel::Default, format!("Loaded texture by path: {}, texture index: {}", path.clone().into_os_string().into_string().unwrap(), wrapped_texture_id.index.to_string()));

    return wrapped_texture_id;
}
pub fn create_texture(data: &Vec<u8>, size: UVec2, object_manager: &mut ObjectManager, format: u32) -> ObjectIndex {
    let texture = Texture::new(&data, UVec2::new(size.x, size.y), format); 
    let texture_id = object_manager.get_mut_array_for_type::<Texture>().create_item(texture);
    let wrapped_texture_id = ObjectIndex::new(texture_id);
    return wrapped_texture_id;
}

#[derive(Component)]
pub struct PosComp {
    pub pos: Vec2,
}
#[derive(Component)]
pub struct SizeComp {
    pub size: Vec2,
}
#[derive(Component)]
pub struct TextureComp {
    pub texture_index: ObjectIndex,
}
#[derive(Component)]
pub struct ColorComp {
    pub color: Color,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectIndex{
    pub index: u32,
}

impl ObjectIndex {
    pub fn new(index: u32) -> ObjectIndex {
        return ObjectIndex { index:  index };
    }
    pub const NULL: ObjectIndex = ObjectIndex { index: 0 };
}
impl Default for ObjectIndex {
    fn default() -> Self {
        Self::NULL
    }
}

pub struct Array32<T> {
    pub buffer: Vec<Option<T>>,
    pub recycle: Vec<u32>,
}
impl<T> Array32<T> {
    pub fn new(size: u32) -> Array32<T> {
        let mut arr = Array32::<T> { 
            buffer: Vec::<Option<T>>::new(), 
            recycle: Vec::<u32>::new(), 
        };
        arr.buffer.reserve(size as usize);
        return arr;
    }
    pub fn create_item(&mut self, item: T) -> u32 {
        let id = self.buffer.len() as u32;
        self.buffer.push(Option::Some(item));
        return id;
    }
    pub fn create_item_to_obj(&mut self, item: T) -> ObjectIndex {
        return ObjectIndex::new(self.create_item(item));
    }
    pub fn get_by_obj_id(&self, index: ObjectIndex) -> Option<T> where T : Copy {
        self.get(index.index as usize)
    }
    pub fn get(&self, index: usize) -> Option<T> where T : Copy {
        self.buffer[index]
    }
    pub fn get_ref_by_obj_id(&self, index: ObjectIndex) -> Option<&T> {
        self.get_ref(index.index as usize)
    }
    pub fn get_ref(&self, index: usize) -> Option<&T> {
        self.buffer[index].as_ref()
    }
    pub fn get_mut_by_obj_id(&mut self, index: ObjectIndex) -> Option<&mut T> {
        self.get_mut(index.index as usize)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.buffer[index].as_mut()
    }
}

pub struct State {
    pub ecs_world: World,
    pub object_manager: ObjectManager,
    pub buffers: Buffers,
    pub default_shader_index: ObjectIndex,
    pub window: Window,
    pub _1px_texture: ObjectIndex, 
    pub exe_dir: PathBuf,
    pub default_font_index: ObjectIndex,
    pub batcher: Batcher2,
    pub text_shader_index: ObjectIndex,
    pub event_pump: EventPump,
    pub entity_map: HashMap<Entity, ObjectIndex>,
    pub managed_callbacks: ManagedCallbacks,
    pub function_map: FunctionMap,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        return Color { r: r, g: g, b: b, a: a };
    }

    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0, };
}
impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

#[repr(C)]
pub struct Vertex{
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(position: Vec2, uv: Vec2, color: Color) -> Vertex {
        Vertex { position, uv: uv, color: color, }
    }
    pub fn from_position(position: Vec2) -> Vertex {
        Vertex::new(position, Vec2::ZERO, Color::WHITE)
    }
}

pub struct Batch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture: ObjectIndex,
    pub proj_mat: Mat4,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Texture {
    pub handle: u32,
    pub size: UVec2,
}
impl Default for Texture {
    fn default() -> Self {
        Self::NONE
    }
}

impl Texture {
    pub const NONE: Texture = Texture { handle: 0, size: UVec2::ZERO, };

    pub fn new(data: &Vec<u8>, size: UVec2, format: u32) -> Self {
        let mut handle: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut handle);
            gl::BindTexture(gl::TEXTURE_2D, handle);
            
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::REPEAT as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::REPEAT as i32,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                size.x as i32,
                size.y as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const _,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
        return Texture { handle: handle, size: size, }
    }
}

struct GlobalState(UnsafeCell<State>);

unsafe impl Sync for GlobalState {}

static GLOBAL_STATE: OnceLock<GlobalState> = OnceLock::new();

pub fn create_object(ecs_world: &mut World, obj_arr: &mut Array32<Entity>, entity_map: &mut HashMap<Entity, ObjectIndex>) -> ObjectIndex {
    let entity_id = ecs_world.spawn(()).id();
    let object_id = obj_arr.create_item(entity_id);
    let object_index = ObjectIndex::new(object_id);
    entity_map.insert(entity_id, object_index);
    return object_index;
}

unsafe impl Sync for State {}
unsafe impl Send for State {}

pub fn main() {
    init_logger();
    GLOBAL_LOGGER.lock().unwrap().enable_all_log_channels();
    GLOBAL_LOGGER.lock().unwrap().show_verbose = true;

    let mut object_manager = ObjectManager::new();

    object_manager.get_mut_array_for_type::<Entity>().create_item(Entity::from_index(EntityIndex::from_raw_u32(0 as u32).unwrap()));
    object_manager.get_mut_array_for_type::<Texture>().create_item(Texture::default());
    object_manager.get_mut_array_for_type::<Sprite>().create_item(Sprite::default());
    object_manager.get_mut_array_for_type::<SpriteAtlas>().create_item(SpriteAtlas::default());
    object_manager.get_mut_array_for_type::<Font>().create_item(Font::default());
    object_manager.get_mut_array_for_type::<Shader>().create_item(Shader::default());

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(4, 6);

    let window = video_subsystem.window("rust-sdl3 demo", 800, 600)
        .opengl()
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    gl_call!(gl::Enable(gl::BLEND));
    gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

    let mut buffers = Buffers { vao: 0, vbo: 0, ebo: 0 };
    gl_call!(gl::GenVertexArrays(1, &mut buffers.vao));
    gl_call!(gl::GenBuffers(1, &mut buffers.vbo));
    gl_call!(gl::GenBuffers(1, &mut buffers.ebo));

    let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let assets_dir = exe_dir.join("assets");

    let default_shader_index = load_shader(
        &assets_dir.join("shaders").join("default.vert"), 
        &assets_dir.join("shaders").join("default.frag"),
        &mut object_manager,
    );
    let text_shader_index = load_shader(
        &assets_dir.join("shaders").join("text.vert"), 
        &assets_dir.join("shaders").join("text.frag"),
        &mut object_manager,
    );

    let _1px_texture = load_texture(&assets_dir.join("sprites").join("1px.png"), &mut object_manager);
    let default_font_index = load_font(&assets_dir.join("fonts").join("default-font.ttf"), &mut object_manager);

    let batcher = Batcher2::new();

    let event_pump = sdl_context.event_pump().unwrap();

    let mut function_map: FunctionMap = FunctionMap::new();

    register_function!(function_map, u_log);
    register_function!(function_map, u_verbose_log);
    register_function!(function_map, u_warning);
    register_function!(function_map, u_verbose_warning);
    register_function!(function_map, u_error);
    register_function!(function_map, u_verbose_error);

    let _ = GLOBAL_STATE.set(GlobalState(UnsafeCell::new(
        State { 
            ecs_world: World::new(), 
            buffers, 
            default_shader_index,
            window,
            _1px_texture,
            exe_dir,
            object_manager,
            default_font_index,
            batcher,
            text_shader_index,
            event_pump,
            entity_map: HashMap::new(),
            managed_callbacks: ManagedCallbacks::default(),
            function_map,
        })));

    let state = get_mut_state();

    let mut update_schedule = Schedule::default();
    update_schedule.add_systems(check_element_overlap);

    let mut render_schedule = Schedule::default();
    render_schedule.add_systems((render_textures, render_text));
    
    let unmanaged_callbacks = UnmanagedCallbacks {
        u_test_call,
        u_create_canvas,
        u_create_image,
        u_load_texture,
        u_set_texture,
        u_get_executing_directory,
        u_set_position,
        u_set_size,
        u_set_color,
        u_get_texture,
        u_get_position,
        u_get_size,
        u_get_color,
        u_get_texture_size,
        u_create_label,
        u_set_text,
        u_get_text,
        u_set_font,
        u_get_font,
        u_set_font_size,
        u_get_font_size,
        u_load_font,
        u_get_mouse_position,
        u_create_button,
        u_get_function_pointer,
    };
    let unmanaged_callbacks_size = std::mem::size_of::<UnmanagedCallbacks>() as i32;
    
    let mut managed_callbacks_size: i32 = 0;
    
    let dll_path = PdCString::from_os_str(state.exe_dir.join("bindings").join("binding-lib.dll")).unwrap();
    let runtime_config_path = PdCString::from_os_str(state.exe_dir.join("bindings").join("binding-lib.runtimeconfig.json")).unwrap(); 

    let hostfxr = load_hostfxr().unwrap();
    let hostfxr_context = hostfxr.initialize_for_runtime_config(runtime_config_path).unwrap();
    let hostfxr_fn_loader = hostfxr_context.get_delegate_loader_for_assembly(dll_path).unwrap();
    
    let initialize_from_engine = hostfxr_fn_loader.get_function_with_unmanaged_callers_only::<InitializeFromEngineFn>(pdcstr!("CsBindings.Engine, binding-lib"), pdcstr!("InitializeFromEngine")).unwrap();
    unsafe {
        initialize_from_engine(unmanaged_callbacks, unmanaged_callbacks_size, &mut state.managed_callbacks, &mut managed_callbacks_size);
    }
    if managed_callbacks_size != std::mem::size_of::<ManagedCallbacks>() as i32 {
        panic!("Managed callbacks size mismatches unmanaged: {}, managed: {}", (std::mem::size_of::<ManagedCallbacks>() as i32).to_string(),  managed_callbacks_size.to_string());
    }

    'running: loop {
        for event in state.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window { win_event: WindowEvent::Resized(w, h), .. } => unsafe {
                    gl::Viewport(0, 0, w, h);
                }
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.3, 0.5, 0.9, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);   
        }

        // println!("mouse_pos: {}", Vec2::new(state.event_pump.mouse_state().x(), state.event_pump.mouse_state().y()).to_string());

        let state = get_mut_state();

        update_schedule.run(&mut state.ecs_world);

        render_schedule.run(&mut state.ecs_world);
        state.batcher.flush(&state.object_manager);

        state.window.gl_swap_window();
    }
}

pub fn render_textures(query: Query<(Entity, &PosComp, &SizeComp, &TextureComp, &ColorComp)>){
    let state = unsafe { &mut *GLOBAL_STATE.get().unwrap().0.get() };
    
    let window_size = state.window.size();
    let proj = glam::camera::lh::proj::directx::orthographic(0.0, window_size.0 as f32, window_size.1 as f32, 0.0, -1.0, 1.0);

    state.batcher.set_projection(proj);
    state.batcher.set_shader(state.default_shader_index);

    for (_, pos_comp, size_comp, texture_comp, color_comp) in query {
        state.batcher.set_texture(texture_comp.texture_index);
        state.batcher.push_quad(&pos_comp.pos, &size_comp.size, &color_comp.color);
    }
}

pub fn render_text(query: Query<(Entity, &PosComp, &FontComp, &TextComp, &ColorComp)>) {
    let state = get_mut_state();
    let window_size = state.window.size();
    let proj = glam::camera::lh::proj::directx::orthographic(0.0, window_size.0 as f32, window_size.1 as f32, 0.0, -1.0, 1.0);
    state.batcher.set_projection(proj);
    state.batcher.set_shader(state.text_shader_index);
    for (_, pos_comp, font_comp, text_comp, color_comp) in query {
        state.batcher.push_text(&pos_comp.pos, font_comp.font_size, &text_comp.text, &font_comp.font, &color_comp.color, &mut state.object_manager);
    }
}

pub struct Buffers {
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Shader {
    pub program: u32,
}

impl Default for Shader {
    fn default() -> Self {
        Self::NONE
    }
}

impl Shader {
    pub fn new(program: u32) -> Self {
        Self { program: program, }
    }
    pub const NONE: Self = Self { program: 0 };
}

pub fn create_shader(vert_source: &String, frag_source: &String) -> Shader {
    let program;

    let vs;
    let fs;
    unsafe {
        vs = gl::CreateShader(gl::VERTEX_SHADER);
        fs = gl::CreateShader(gl::FRAGMENT_SHADER);
    }

    let vs_cstr = CString::new(vert_source.as_str()).unwrap();
    let fs_cstr = CString::new(frag_source.as_str()).unwrap();

    let vs_src_argv = vs_cstr.as_ptr();
    let fs_src_argv = fs_cstr.as_ptr();

    // println!("{}", vert_source);
    // println!("{}", frag_source);

    gl_call!(gl::ShaderSource(vs, 1, &vs_src_argv, std::ptr::null()));
    gl_call!(gl::ShaderSource(fs, 1, &fs_src_argv, std::ptr::null()));

    gl_call!(gl::CompileShader(vs));
    gl_call!(gl::CompileShader(fs));

    check_shader_compile(vs);
    check_shader_compile(fs);

    unsafe {
        program = gl::CreateProgram();
    }

    gl_call!(gl::AttachShader(program, vs));
    gl_call!(gl::AttachShader(program, fs));

    gl_call!(gl::LinkProgram(program));

    check_program_link(program);

    gl_call!(gl::DeleteShader(vs));
    gl_call!(gl::DeleteShader(fs));
    
    Shader::new(program)
}

fn check_shader_compile(shader: u32) {
    unsafe {
        let mut success = 0;

        gl::GetShaderiv(
            shader,
            gl::COMPILE_STATUS,
            &mut success
        );

        if success == 0 {
            let mut len = 0;

            gl::GetShaderiv(
                shader,
                gl::INFO_LOG_LENGTH,
                &mut len
            );

            let mut buffer = vec![0u8; len as usize];

            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buffer.as_mut_ptr() as *mut _
            );

            panic!(
                "Shader compile error:\n{}",
                String::from_utf8_lossy(&buffer)
            );
        }
    }
}

fn check_program_link(program: u32) {
    let mut success = 0; 

    unsafe {
        gl::GetProgramiv(program, 
            gl::LINK_STATUS, 
            &mut success
        ); 
        if success == 0 { 
            let mut len = 0; 
            gl::GetProgramiv(program, 
                gl::INFO_LOG_LENGTH, 
                &mut len
            ); 
            let mut buffer = vec![0u8; len as usize]; 
            gl::GetProgramInfoLog(program, 
                len, 
                std::ptr::null_mut(), 
                buffer.as_mut_ptr() as *mut _
            ); 
            panic!("Shader link error {}", String::from_utf8_lossy(&buffer)); 
        }
    }
}

pub fn create_and_allocate_shader(vert_source: &String, frag_source: &String, object_manager: &mut ObjectManager) -> ObjectIndex {
    let shader = create_shader(vert_source, frag_source);
    // println!("shader_index: {}", shader.program.to_string());
    ObjectIndex::new(object_manager.get_mut_array_for_type_unsafe::<Shader>().create_item(shader))
}
pub fn load_shader(vert_source_path: &PathBuf, frag_source_path: &PathBuf, object_manager: &mut ObjectManager) -> ObjectIndex {
    let vert_source = fs::read_to_string(vert_source_path).unwrap();
    let frag_source = fs::read_to_string(frag_source_path).unwrap();
    // println!("Loaded shader by path: vertex_shader: {}, fragment_shader: {}", vert_source, frag_source);
    let shader_index = create_and_allocate_shader(&vert_source, &frag_source, object_manager);
    log(LogChannel::Default, format!("Loaded shader by path: vertex_shader: {}, fragment_shader: {}", vert_source_path.display(), frag_source_path.display()));
    shader_index
}

pub struct ObjectManager {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn ensure_array_for_type<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.map.contains_key(&type_id) {
            let arr = Array32::<T>::new(u16::MAX as u32);
            self.map.insert(type_id, Box::new(arr));
        }
    }

    pub fn get_mut_array_for_type<T: 'static>(&mut self) -> &mut Array32<T> {
        self.ensure_array_for_type::<T>();
        self.get_mut_array_for_type_unsafe::<T>()
    }
    pub fn get_ref_array_for_type<T: 'static>(&mut self) -> &Array32<T> {
        self.ensure_array_for_type::<T>();
        self.get_ref_array_for_type_unsafe::<T>()
    }

    pub fn get_mut_array_for_type_unsafe<T: 'static>(&mut self) -> &mut Array32<T> {
        let type_id = TypeId::of::<T>();
        self.map.get_mut(&type_id).unwrap().downcast_mut::<Array32<T>>().unwrap()
    }
    pub fn get_ref_array_for_type_unsafe<T: 'static>(&self) -> &Array32<T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id).unwrap().downcast_ref::<Array32<T>>().unwrap()
    }
}

#[derive(Component)]
pub struct OnClickComp {
    
}

pub fn check_element_overlap(query: Query<(Entity, &PosComp, &SizeComp, &OnClickComp, )>) {
    let mouse_position = get_mouse_position(get_ref_state().event_pump.mouse_state());
    for (entity, pos_comp, size_comp, _, ) in query {
        // println!("pos: {}, size: {}", pos_comp.pos, size_comp.size);
        let rect = Rect::new(pos_comp.pos.x, pos_comp.pos.y, size_comp.size.x, size_comp.size.y);
        if rect.is_position_in_bounds(mouse_position.x, mouse_position.y) {
            if get_ref_state().event_pump.mouse_state().left() {
                // println!("mouse_position: {}", mouse_position.to_string());
                (get_ref_state().managed_callbacks.m_on_element_clicked)(get_ref_state().entity_map[&entity]);
            }
        }
    }
}