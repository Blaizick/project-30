pub mod font {
use std::{collections::HashMap, ffi::c_char, fmt::format, fs, ops::Add, path::PathBuf};
use ab_glyph_rasterizer::{ Point, Rasterizer, point};
use glam::{Mat4, UVec2, Vec2, Vec3};
use image::ColorType;
use ttf_parser::{Face, OutlineBuilder};
use crate::{log::log::{LogChannel, log}, *};

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Font {
        pub sprite_atlas: ObjectIndex,
    }
    
    impl Default for Font {
        fn default() -> Self {
            Self {
                sprite_atlas: ObjectIndex::NULL,
            }
        }
    }

    pub fn get_font_face(data: &Box<[u8]>) -> Face<'_> {
        return Face::parse(&data, 0).unwrap();
    }

    pub fn load_font(path: &PathBuf, object_manager: &mut ObjectManager) -> ObjectIndex {
        if !path.exists() {
            panic!("Path not exists: {}", path.display())
        }
        let data = fs::read(path).unwrap().into_boxed_slice();
        let face = get_font_face(&data);
        // let glyph_count = face.number_of_glyphs();
        let mut sprite_atlas_builder = SpriteAtlasBuilder::new(UVec2::new(512, 512));

        let pixel_height = 64.0;
        let upem = face.units_per_em() as f32;
        let scale = pixel_height / upem;

        // println!("scale: {}", scale.to_string());

        for char in ' '..='~' {
            if let Some(glyph) = face.glyph_index(char) {
                if let Some(bbox) = face.glyph_bounding_box(glyph) {
                    let width = ((bbox.x_max - bbox.x_min) as f32 * scale).ceil() as u32;
                    let height = ((bbox.y_max - bbox.y_min) as f32 * scale).ceil() as u32;

                    // println!("character: {}, width: {}, height: {}", char.to_string(), width.to_string(), height.to_string());

                    let mut rasterizer = Rasterizer::new(width as usize, height as usize);

                    let mut builder = Builder {
                        rasterizer: &mut rasterizer,
                        offset_x: bbox.x_min as f32 * scale,
                        offset_y: bbox.y_max as f32 * scale,
                        prev_x: 0.0,
                        prev_y: 0.0,
                        start_x: 0.0,
                        start_y: 0.0,
                        scale,
                    };
                    face.outline_glyph(glyph, &mut builder).unwrap();
                    let mut pixels = vec![0u8; (width * height) as usize];
                    rasterizer.for_each_pixel_2d(|x, y, alpha| {
                        pixels[(y * width + x) as usize] = (alpha * 255.0) as u8;
                    });
                    sprite_atlas_builder.push(
                        &pixels, 
                        &UVec2::new(width, height), 
                        &face.glyph_name(glyph).unwrap().to_string(), 
                        object_manager
                    );
                }
            }
        }
        let sprite_atlas_index = sprite_atlas_builder.build(object_manager);

        log(LogChannel::Default, format!("loaded font by path: {}", path.display()));

        object_manager.get_mut_array_for_type::<Font>().create_item_to_obj(Font {
            sprite_atlas: sprite_atlas_index,
        })
    }

struct Builder<'a> {
    pub rasterizer: &'a mut Rasterizer,

    pub offset_x: f32,
    pub offset_y: f32,

    pub prev_x: f32,
    pub prev_y: f32,

    pub start_x: f32,
    pub start_y: f32,

    pub scale: f32,
}

impl Builder<'_> {
    fn transform(&self, _point: Point) -> Point {
        point(_point.x * self.scale - self.offset_x, 
            self.offset_y - _point.y * self.scale)
    } 
}

impl OutlineBuilder for Builder<'_> {

    fn move_to(&mut self, x: f32, y: f32) {
        self.prev_x = x;
        self.prev_y = y;

        self.start_x = x;
        self.start_y = y;
    }


    fn line_to(&mut self, x: f32, y: f32) {
        let pos1 = self.transform(point(self.prev_x, self.prev_y));
        let pos2 = self.transform(point(x, y));
        let _x1 = pos1.x;
        let _y1 = pos1.y;
        let _x2 = pos2.x;
        let _y2 = pos2.y;
        // println!("offset_x: {}, offset_y: {}", self.offset_x, self.offset_y);
        // println!("1) x: {}, y: {}, _x: {}, _y: {}", self.prev_x, self.prev_y, _x1.to_string(), _y1.to_string());
        // println!("2) x: {}, y: {}, _x: {}, _y: {}", x, y, _x2.to_string(), _y2.to_string());
        self.rasterizer.draw_line(
            point(_x1, _y1),
            point(_x2, _y2),
        );

        self.prev_x = x;
        self.prev_y = y;
    }


    fn quad_to(
        &mut self,
        x1: f32,
        y1: f32,
        x: f32,
        y: f32,
    ) {
        self.rasterizer.draw_quad(
            self.transform(point(self.prev_x, self.prev_y)),
            self.transform(point(x1, y1)),
            self.transform(point(x, y)),
        );

        self.prev_x = x;
        self.prev_y = y;
    }


    fn curve_to(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x: f32,
        y: f32,
    ) {
        self.rasterizer.draw_cubic(
            self.transform(point(self.prev_x, self.prev_y)),
            self.transform(point(x1, y1)),
            self.transform(point(x2, y2)),
            self.transform(point(x, y)),
        );

        self.prev_x = x;
        self.prev_y = y;
    }


    fn close(&mut self) {
        self.prev_x = self.start_x;
        self.prev_y = self.start_y;
    }
}
    pub struct Batcher2 {
        pub batch_items: Vec<BatchItem2>,
        pub texture_index: ObjectIndex,
        pub projection: Mat4,
        pub shader: ObjectIndex,
    }

    impl Batcher2 {
        pub fn new() -> Self {
            Self {
                batch_items: Vec::new(),
                texture_index: ObjectIndex::NULL,
                projection: Mat4::default(),
                shader: ObjectIndex::NULL,
            }
        }

        pub fn set_texture(&mut self, texture_index: ObjectIndex){
            self.texture_index = texture_index;
            if self.batch_items.is_empty() {
                self.push_new_batch();
            }
            let last_item_different; 
            let last_item_empty;
            {
                let last_item = self.batch_items.last().unwrap();
                last_item_different = last_item.texture_index != texture_index;
                last_item_empty = last_item.is_empty();
            }
            if last_item_different {
                if !last_item_empty {
                    self.push_new_batch();
                }
            }
            self.batch_items.last_mut().unwrap().texture_index = texture_index;
        }
        pub fn set_projection(&mut self, projection: Mat4){
            self.projection = projection;
            if self.batch_items.is_empty() {
                self.push_new_batch();
            }
            let last_item_different; 
            let last_item_empty;
            {
                let last_item = self.batch_items.last().unwrap();
                last_item_different = last_item.projection != projection;
                last_item_empty = last_item.is_empty();
            }
            if last_item_different {
                if !last_item_empty {
                    self.push_new_batch();
                }
            }
            self.batch_items.last_mut().unwrap().projection = projection;
        }
        pub fn set_shader(&mut self, shader_index: ObjectIndex){
            self.shader = shader_index;
            if self.batch_items.is_empty() {
                self.push_new_batch();
            }
            let last_item_different; 
            let last_item_empty;
            {
                let last_item = self.batch_items.last().unwrap();
                last_item_different = last_item.shader_index != shader_index;
                last_item_empty = last_item.is_empty();
            }
            if last_item_different {
                if !last_item_empty {
                    self.push_new_batch();
                }
            }
            self.batch_items.last_mut().unwrap().shader_index = shader_index;
        }

        pub fn get_new_batch(&self) -> BatchItem2 {
            BatchItem2::new(self.texture_index.clone(), self.projection.clone(), self.shader.clone())
        }

        pub fn push_new_batch(&mut self) {
            self.batch_items.push(self.get_new_batch());
        }

        pub fn push_quad_with_uv(&mut self, position: &Vec2, size: &Vec2, color: &Color, rect: Rect) {
            if self.batch_items.is_empty() {
                self.push_new_batch();
            }
            let base = self.batch_items.last().unwrap().vertices.len() as u32;
            let mut vertex_vec =  vec![
                Vertex::new(Vec2::new(0.0, 0.0), Vec2::new(rect.min_x(), rect.min_y()), *color),
                Vertex::new(Vec2::new(1.0, 0.0), Vec2::new(rect.max_x(), rect.min_y()), *color),
                Vertex::new(Vec2::new(1.0, 1.0), Vec2::new(rect.max_x(), rect.max_y()), *color),
                Vertex::new(Vec2::new(0.0, 1.0), Vec2::new(rect.min_x(), rect.max_y()), *color),
            ];
            let model_matrix = 
                Mat4::from_scale(Vec3::new(size.x, size.y, 1.0)) *
                Mat4::from_translation(Vec3::new(position.x, position.y, 0.0))
                ; 
            for vertex in vertex_vec.iter_mut() {
                let pos = model_matrix.transform_point3(Vec3::new(vertex.position.x, vertex.position.y, 0.0));
                vertex.position.x = pos.x;
                vertex.position.y = pos.y;
            }
            let last_batch_item = self.batch_items.last_mut().unwrap();
            last_batch_item.vertices.extend(vertex_vec);
            last_batch_item.indices.extend([
                base,
                base + 1,
                base + 2,

                base,
                base + 2,
                base + 3,
            ]);
        }
        pub fn push_quad(&mut self, position: &Vec2, size: &Vec2, color: &Color) {
            self.push_quad_with_uv(position, size, color, Rect::new(0.0, 0.0, 1.0, 1.0));
        }
        pub fn push_text(&mut self, position: &Vec2, font_size: f32, text: &String, font_index: &ObjectIndex, color: &Color, object_manager: &ObjectManager) {
            let font = object_manager.get_ref_array_for_type_unsafe::<Font>().get_by_obj_id(*font_index).unwrap();
            self.set_texture(object_manager.get_ref_array_for_type_unsafe::<SpriteAtlas>().get_ref_by_obj_id(font.sprite_atlas).unwrap().texture_index);
            let mut current_position = position.clone();
            for char in text.chars() {
                let sprite_rect = {
                    let sprite_atlas = object_manager.get_ref_array_for_type_unsafe::<SpriteAtlas>().get_ref_by_obj_id(font.sprite_atlas).unwrap();
                    sprite_atlas.get_rect_for_sprite(&char.to_string(), object_manager)
                };
                let char_size = Vec2::new(
                    sprite_rect.width * font_size,
                    sprite_rect.height * font_size,
                );
                current_position.x += char_size.x;
                self.push_quad_with_uv(&current_position, &char_size, color, sprite_rect);
                // println!("character: {}, sprite_rect: {}", char.to_string(), sprite_rect.to_string());
            }
        }
        pub fn flush_batch(&self, batch: &BatchItem2, object_manager: &ObjectManager, buffers: &Buffers) {
            let shader = object_manager.get_ref_array_for_type_unsafe::<Shader>().get_ref_by_obj_id(batch.shader_index).unwrap();
            let texture = object_manager.get_ref_array_for_type_unsafe::<Texture>().get_ref_by_obj_id(batch.texture_index).unwrap();
            unsafe {
                gl_call!(gl::ActiveTexture(gl::TEXTURE0));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, texture.handle));

                // println!("shader_index 2: {}", shader.program.to_string());
                gl_call!(gl::UseProgram(shader.program));

                gl_call!(gl::BindVertexArray(buffers.vao));
                gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, buffers.vbo));
                gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffers.ebo));

                gl_call!(gl::BufferData(gl::ARRAY_BUFFER, (batch.vertices.len() * std::mem::size_of::<Vertex>()) as isize, batch.vertices.as_ptr() as *const _, gl::STATIC_DRAW));
                gl_call!(gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (batch.indices.len() * std::mem::size_of::<u32>()) as isize, batch.indices.as_ptr() as *const _, gl::STATIC_DRAW));

                let stride = std::mem::size_of::<Vertex>() as i32;

                gl_call!(gl::EnableVertexAttribArray(0));
                gl_call!(gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, std::ptr::null()));

                gl_call!(gl::EnableVertexAttribArray(1));
                gl_call!(gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (2 * std::mem::size_of::<f32>()) as *const _));

                gl_call!(gl::EnableVertexAttribArray(2));
                gl_call!(gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (4 * std::mem::size_of::<f32>()) as *const _));

                let location = gl::GetUniformLocation(shader.program, c"uProj".as_ptr());
                gl_call!(gl::UniformMatrix4fv(location, 1, gl::FALSE, batch.projection.to_cols_array().as_ptr()));

                let location = gl::GetUniformLocation(shader.program, c"uTex".as_ptr());
                gl_call!(gl::Uniform1i(location, 0 as i32));

                gl_call!(gl::DrawElements(gl::TRIANGLES, batch.indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null()));
            }
        }
        pub fn flush(&mut self, object_manager: &ObjectManager, ) {
            let state = get_ref_state();
            for item in self.batch_items.iter() {
                self.flush_batch(item, object_manager, &state.buffers);
            }
            self.batch_items.clear();
        }
    }

    pub struct BatchItem2 {
        pub vertices: Vec<Vertex>,
        pub indices: Vec<u32>,
        pub texture_index: ObjectIndex, 
        pub projection: Mat4,
        pub shader_index: ObjectIndex,
    }

    impl BatchItem2 {
        pub fn new(texture_index: ObjectIndex, projection: Mat4, shader_index: ObjectIndex) -> BatchItem2 {
            BatchItem2 {
                vertices: Vec::new(),
                indices: Vec::new(),
                texture_index: texture_index,
                projection: projection,
                shader_index: shader_index,
            }
        }
        pub fn is_empty(&self) -> bool {
            self.vertices.len() <= 0
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Sprite {
        sprite_atlas_index: ObjectIndex,
        region: URect,
    }

    impl Default for Sprite {
        fn default() -> Self {
            Self { sprite_atlas_index: ObjectIndex::NULL, region: URect::default(), }
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct RectBase<T> where T : Copy + Add<Output = T> + Default {
        x: T,
        y: T,
        width: T,
        height: T,
    }
    impl<T> Default for RectBase<T> where T : Copy + Add<Output = T> + Default {
        fn default() -> Self {
            Self { x: T::default(), y: T::default(), width: T::default(), height: T::default(), }            
        }
    }
    impl<T> RectBase<T> where T : Copy + Add<Output = T> + Default + PartialOrd {
        pub fn new(x: T, y: T, width: T, height: T) -> Self {
            Self { x, y, width, height, }
        }
        pub fn min_x(&self) -> T {
            return self.x;
        }
        pub fn min_y(&self) -> T {
            return self.y;
        }
        pub fn max_x(&self) -> T {
            return self.x + self.width;
        }
        pub fn max_y(&self) -> T {
            return self.y + self.height;
        }
        pub fn is_position_in_bounds(&self, x: T, y: T) -> bool {
            x >= self.x && y >= self.y && x <= self.x + self.width && y <= self.height
        } 
    }

    impl<T> ToString for RectBase<T> where T : Copy + Add<Output = T> + Default + ToString {
        fn to_string(&self) -> String {
            format!("x: {}, y: {}, size_x: {}, size_y: {}", self.x.to_string(), self.y.to_string(), self.width.to_string(), self.height.to_string())
        }
    }

    pub type Rect = RectBase<f32>;
    pub type URect = RectBase<u32>; 

    #[repr(C)]
    pub struct SpriteAtlas {
        texture_index: ObjectIndex,
        sprite_map: HashMap<String, ObjectIndex>,
    }

    impl Default for SpriteAtlas {
        fn default() -> Self {
            Self { texture_index: ObjectIndex::NULL, sprite_map: HashMap::new(), }
        }
    }

    impl SpriteAtlas {
        pub fn get_rect_for_sprite(&self, sprite_id: &String, object_manager: &ObjectManager) -> Rect {
            // println!("sprite_map_len: {}", self.sprite_map.len().to_string());
            let sprite_index = self.sprite_map[sprite_id];
            let sprite_size = {
                let sprite = object_manager.get_ref_array_for_type_unsafe::<Sprite>().get_ref_by_obj_id(sprite_index).unwrap();
                Vec2::new(sprite.region.width as f32, sprite.region.height as f32)
            };
            let texture_size = { 
                let texture = object_manager.get_ref_array_for_type_unsafe::<Texture>().get_ref_by_obj_id(self.texture_index).unwrap();
                texture.size.as_vec2()
            };
            let sprite_position = {
                let sprite = object_manager.get_ref_array_for_type_unsafe::<Sprite>().get_ref_by_obj_id(sprite_index).unwrap();
                Vec2::new(sprite.region.x as f32, sprite.region.y as f32)
            };
            get_rect_for_region(&texture_size, &sprite_size, &sprite_position)
        }
    }
    pub fn get_rect_for_region(texture_size: &Vec2, sprite_size: &Vec2, sprite_position: &Vec2) -> Rect {
        return Rect::new(sprite_position.x / texture_size.x,
            sprite_position.y / texture_size.y,
            sprite_size.x / texture_size.x,
            sprite_size.y / texture_size.y,
        );
    }

    #[repr(C)]
    pub struct SpriteAtlasBuilder {
        pub data: Vec<u8>,
        pub size: UVec2,
        pub prev: UVec2,
        pub cur_height: u32,
        pub sprite_map: HashMap<String, ObjectIndex>,
    }

    impl SpriteAtlasBuilder {
        pub fn new(size: UVec2) -> Self {
            return SpriteAtlasBuilder { 
                size: size, 
                data: vec![0u8; (size.x * size.y) as usize],
                prev: UVec2::ZERO,
                cur_height: 0, 
                sprite_map: HashMap::new(),
            }
        }
        pub fn build(mut self, object_manager: &mut ObjectManager) -> ObjectIndex {
            // image::save_buffer("C:\\Users\\Blaizi\\Desktop\\saved_img.png", &self.data, self.size.x, self.size.y, ColorType::L8).unwrap();
            
            let texture_index = create_texture(&self.data, self.size, object_manager, gl::RED);
            let sprite_atlas = SpriteAtlas { texture_index, sprite_map: HashMap::new(), };
            let sprite_atlas_index = object_manager.get_mut_array_for_type::<SpriteAtlas>().create_item_to_obj(sprite_atlas);
            for (name, sprite_index) in self.sprite_map.iter_mut() {
                let sprite = object_manager.get_mut_array_for_type::<Sprite>().get_mut_by_obj_id(*sprite_index).unwrap();
                sprite.sprite_atlas_index = sprite_atlas_index;
            }
            let sprite_atlas = object_manager.get_mut_array_for_type::<SpriteAtlas>().get_mut_by_obj_id(sprite_atlas_index).unwrap();
            sprite_atlas.sprite_map = self.sprite_map;
            return sprite_atlas_index;
        }
        pub fn push(&mut self, data: &Vec<u8>, size: &UVec2, name: &String, object_manager: &mut ObjectManager) -> ObjectIndex {
            if self.prev.x + size.x >= self.size.x {
                self.prev.x = 0;
                self.prev.y += self.cur_height;
                self.cur_height = 0;
            }
            if size.y > self.cur_height {
                self.cur_height = size.y;
            }
            let region = URect::new(self.prev.x, self.prev.y, size.x, size.y);
            // println!("data_size: {}, size_x: {}, size_y: {}, prev_x: {}, prev_y: {}, name: {}, source_size: {}", 
            //     self.data.len().to_string(), 
            //     self.size.x.to_string(), 
            //     self.size.y.to_string(),
            //     self.prev.x.to_string(),
            //     self.prev.y.to_string(),
            //     name.to_string(),
            //     size.to_string(),
            // );
            for x in 0..size.x {
                for y in 0..size.y {
                    let cx = self.prev.x + x;
                    let cy = self.prev.y + y;
                    let atlas_i = cx + cy * self.size.x;
                    let source_i = x + y * size.x;
                    assert!(atlas_i < self.data.len() as u32, "atlas_i: {}, self.data.len(): {}", atlas_i.to_string(), self.data.len().to_string());
                    assert!(source_i < data.len() as u32, "source_i: {}, data.len(): {}", source_i.to_string(), data.len().to_string());
                    self.data[atlas_i as usize] = data[source_i as usize];
                }
            }
            self.prev.x += size.x;
            let sprite = Sprite {
                sprite_atlas_index: ObjectIndex::NULL,
                region: region,
            };
            let sprite_index = object_manager.get_mut_array_for_type::<Sprite>().create_item_to_obj(sprite);
            self.sprite_map.insert(name.clone(), sprite_index);
            return sprite_index;
        }
    }
}