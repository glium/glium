/*!

This crate allows you to easily write text.

Usage:

```no_run
# extern crate glium_core;
# extern crate glium_text;
# fn main() {
# let display: glium_core::Display = unsafe { std::mem::uninitialized() };
// The `TextSystem` contains the shaders and elements used for text display.
let system = glium_text::TextSystem::new(&display);

// Creating a `FontTexture`, which a regular `Texture` which contains the font.
// Note that loading the systems fonts is not covered by this library.
let font = glium_text::FontTexture::new(&display, std::io::File::open(&Path::new("my_font.ttf")), 24).unwrap();

// Creating a `TextDisplay` which contains the elements required to draw a specific sentence.
let text = glium_text::TextDisplay::new(&system, &font, "Hello world!");

// Finally, drawing the text is done with a `DrawCommand`.
// This draw command contains the matrix and color to use for the text.
display.draw().draw(glium_text::DrawCommand(&text,
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ], [1.0, 1.0, 0.0, 1.0]));
# }
```

*/

#![feature(phase)]
#![feature(tuple_indexing)]
#![deny(warnings)]
#![deny(missing_doc)]

#[phase(plugin)]
extern crate glium_core_macros;
extern crate freetype;
extern crate glium_core;

/// Texture which contains the characters of the font.
pub struct FontTexture<'d> {
    texture: glium_core::Texture,
    character_infos: Vec<(char, CharacterInfos)>,
}

/// Object that contains the elements shared by all `TextDisplay` objects.
///
/// Required to create a `TextDisplay`.
pub struct TextSystem<'d> {
    display: &'d glium_core::Display,
    program: glium_core::Program,
}

/// Object that will allow you to draw a text.
pub struct TextDisplay<'s, 'd, 't> {
    display: &'d glium_core::Display,
    texture: &'t FontTexture<'d>,
    vertex_buffer: Option<glium_core::VertexBuffer<VertexFormat>>,
    index_buffer: Option<glium_core::IndexBuffer>,
    total_text_width: f32,
    is_empty: bool,
}

/// Commands to draw a text.
///
/// ## About the matrix
///
/// One unit in height corresponds to a line of text, but the text can go above or under.
/// The bottom of the line is 0, the top is 1.
/// You need to adapt your matrix by taking these into consideration.
pub struct DrawCommand<'s:'td, 'd:'td, 't:'td, 'td, 'a, 'b: 'a>(pub &'td TextDisplay<'s, 'd, 't>, pub &'a TextSystem<'b>, pub [[f32, ..4], ..4], pub [f32, ..4]);

// structure containing informations about a character of a font
struct CharacterInfos {
    // coordinates of the character top-left hand corner on the font's texture
    coords: (f32, f32),

    // width and height of character in texture units
    size: (f32, f32),

    // number of texture units between the bottom of the character and the base line of text
    height_over_line: f32,
    // number of texture units at the left of the character
    left_padding: f32,
    // number of texture units at the right of the character
    right_padding: f32,
}

#[vertex_format]
#[allow(non_snake_case)]
struct VertexFormat {
    #[allow(dead_code)]
    iPosition: [f32, ..2],
    #[allow(dead_code)]
    iTexCoords: [f32, ..2],
}

#[uniforms]
#[allow(non_snake_case)]
struct Uniforms<'a, 'b> {
    uColor: [f32, ..4],
    uMatrix: [[f32, ..4], ..4],
    uTexture: &'a FontTexture<'b>,
}

impl<'d> FontTexture<'d> {
    /// Creates a new texture representing a font stored in a `FontTexture`.
    pub fn new<R: Reader>(display: &'d glium_core::Display, mut font: R, font_size: u32) -> Result<FontTexture, ()> {
        let library = try!(freetype::Library::init().map_err(|_| {}));
        
        // building the freetype face object
        let font = try!(font.read_to_end().map_err(|_| {}));
        let face = try!(library.new_memory_face(font.as_slice(), 0).map_err(|_| {}));
        let face: freetype::ffi::FT_Face = unsafe { std::mem::transmute(face.raw()) };

        // computing the list of characters in the font
        let characters_list = unsafe {
            // TODO: unresolved symbol
            /*if freetype::ffi::FT_Select_CharMap(face, freetype::ffi::FT_ENCODING_UNICODE) != 0 {
                return Err(());
            }*/

            let mut result = Vec::new();

            let mut g: freetype::ffi::FT_UInt = std::mem::uninitialized();
            let mut c = freetype::ffi::FT_Get_First_Char(face, &mut g);

            while g != 0 {
                result.push(std::mem::transmute(c as u32));     // TODO: better solution?
                c = freetype::ffi::FT_Get_Next_Char(face, c, &mut g);
            }

            result
        };

        // building the infos
        let (texture_data, (texture_width, texture_height), chr_infos) = unsafe {
            build_font_image(face, characters_list, font_size)
        };

        // we load the texture in the display
        let texture = glium_core::Texture::new(display, texture_data.as_slice(), texture_width as uint, texture_height as uint, 1, 1);

        Ok(FontTexture {
            texture: texture,
            character_infos: chr_infos,
        })
    }
}

impl<'a, 'd> glium_core::uniforms::UniformValue for &'a FontTexture<'d> {
    fn to_binder(&self) -> glium_core::uniforms::UniformValueBinder {
        use glium_core::uniforms::UniformValue;
        (&self.texture).to_binder()
    }
}

impl<'d> TextSystem<'d> {
    /// Builds a new text system that must be used to build `TextDisplay` objects.
    pub fn new(display: &'d glium_core::Display) -> TextSystem<'d> {
        TextSystem {
            display: display,
            program:
                glium_core::Program::new(display, r"
                    #version 110

                    attribute vec2 iPosition;
                    attribute vec2 iTexCoords;
                    varying vec2 vTexCoords;
                    uniform mat4 uMatrix;
                    
                    void main() {
                        gl_Position = vec4(iPosition, 0.0, 1.0) * uMatrix;
                        vTexCoords = iTexCoords;
                    }
                ", r"
                    #version 110

                    varying vec2 vTexCoords;
                    uniform vec4 uColor;
                    uniform sampler2D uTexture;
                    
                    void main() {
                        gl_FragColor = vec4(uColor.rgb, uColor.a * texture2D(uTexture, vTexCoords));
                        if (gl_FragColor.a <= 0.01)
                            discard;
                    }
                ", None).unwrap()
        }
    }
}

impl<'s, 'd, 't> TextDisplay<'s, 'd, 't> {
    /// Builds a new text display that allows you to draw text.
    pub fn new(system: &'s TextSystem<'d>, texture: &'t FontTexture<'d>, text: &str) -> TextDisplay<'s, 'd, 't> {
        let mut text_display = TextDisplay {
            display: system.display,
            texture: texture,
            vertex_buffer: None,
            index_buffer: None,
            total_text_width: 0.0,
            is_empty: true,
        };

        text_display.set_text(text);

        text_display
    }

    /// Returns the width in GL units of the text.
    pub fn get_width(&self) -> f32 {
        self.total_text_width
    }

    /// Modifies the text on this display.
    pub fn set_text(&mut self, text: &str) {
        self.is_empty = true;
        self.total_text_width = 0.0;
        self.vertex_buffer = None;
        self.index_buffer = None;

        // returning if no text
        if text.len() == 0 {
            return;
        }

        // these arrays will contain the vertex buffer and index buffer data
        let mut vertex_buffer_data = Vec::with_capacity(text.len() * 4 * 4);
        let mut index_buffer_data = Vec::with_capacity(text.len() * 6);

        // iterating over the characters of the string
        for character in text.nfc_chars() {
            let infos = match self.texture.character_infos
                .iter().find(|&&(chr, _)| chr == character)
            {
                Some(infos) => infos,
                None => continue        // character not found in the font, ignoring it
            };
            let infos = infos.1;

            self.is_empty = false;

            // adding the quad in the index buffer
            {
                let first_vertex_offset = vertex_buffer_data.len() as u16;
                index_buffer_data.push(first_vertex_offset);
                index_buffer_data.push(first_vertex_offset + 1);
                index_buffer_data.push(first_vertex_offset + 2);
                index_buffer_data.push(first_vertex_offset + 2);
                index_buffer_data.push(first_vertex_offset + 1);
                index_buffer_data.push(first_vertex_offset + 3);
            }

            // 
            self.total_text_width += infos.left_padding;

            // calculating coords
            let left_coord = self.total_text_width;
            let right_coord = left_coord + infos.size.0;
            let top_coord = infos.height_over_line;
            let bottom_coord = infos.height_over_line - infos.size.1;

            // top-left vertex
            vertex_buffer_data.push(VertexFormat {
                iPosition: [left_coord, top_coord],
                iTexCoords: [infos.coords.0, infos.coords.1],
            });
            
            // top-right vertex
            vertex_buffer_data.push(VertexFormat {
                iPosition: [right_coord, top_coord],
                iTexCoords: [infos.coords.0 + infos.size.0, infos.coords.1],
            });

            // bottom-left vertex
            vertex_buffer_data.push(VertexFormat {
                iPosition: [left_coord, bottom_coord],
                iTexCoords: [infos.coords.0, infos.coords.1 + infos.size.1],
            });
            
            // bottom-right vertex
            vertex_buffer_data.push(VertexFormat {
                iPosition: [right_coord, bottom_coord],
                iTexCoords: [infos.coords.0 + infos.size.0, infos.coords.1 + infos.size.1],
            });
            
            // going to next char
            self.total_text_width = right_coord + infos.right_padding;
        }

        if !vertex_buffer_data.len() != 0 {
            // building the vertex buffer
            self.vertex_buffer = Some(glium_core::VertexBuffer::new(self.display, vertex_buffer_data));
            
            // building the index buffer
            self.index_buffer = Some(glium_core::IndexBuffer::new(self.display, glium_core::TrianglesList, index_buffer_data.as_slice()));
        }
    }
}

impl<'s, 'd, 't, 'td, 'a, 'b> glium_core::DrawCommand for DrawCommand<'s, 'd, 't, 'td, 'a, 'b> {
    fn draw(self, target: &mut glium_core::Target) {
        let DrawCommand(&TextDisplay { ref vertex_buffer, ref index_buffer, ref texture, is_empty, .. }, ref system, ref matrix, ref color) = self;

        // returning if nothing to draw
        if is_empty || vertex_buffer.is_none() || index_buffer.is_none() {
            return;
        }

        let vertex_buffer = vertex_buffer.as_ref().unwrap();
        let index_buffer = index_buffer.as_ref().unwrap();

        let uniforms = Uniforms {
            uMatrix: *matrix,
            uColor: *color,
            uTexture: *texture,
        };

        target.draw(glium_core::BasicDraw(vertex_buffer, index_buffer, &system.program, &uniforms, &std::default::Default::default()));
    }
}

unsafe fn build_font_image(face: freetype::ffi::FT_Face, characters_list: Vec<char>, font_size: u32) -> (Vec<f32>, (u32, u32), Vec<(char, CharacterInfos)>) {
    // setting the right pixel size
    if freetype::ffi::FT_Set_Pixel_Sizes(face, font_size, font_size) != 0 {
        fail!();
    }
    
    // this variable will store the texture data
    // we set an arbitrary capacity that we think will match what we will need
    let mut texture_data: Vec<f32> = Vec::with_capacity(characters_list.len() * font_size as uint * font_size as uint);

    // the width is chosen more or less arbitrarily, because we can store everything as long as the texture is at least as wide as the widest character
    // we just try to estimate a width so that width ~= height
    let texture_width = get_nearest_po2(std::cmp::max(font_size * 2 as u32, ((((characters_list.len() as u32) * font_size * font_size) as f32).sqrt()) as u32));

    // we store the position of the "cursor" in the destination texture
    // this cursor points to the top-left pixel of the next character to write on the texture
    let mut cursor_offset = (0u32, 0u32);

    // number of rows to skip at next carriage return
    let mut rows_to_skip = 0u32;

    // now looping through the list of characters, filling the texture and returning the informations
    let mut characters_infos: Vec<(char, CharacterInfos)> = characters_list.into_iter().filter_map(|character| {
        // loading wanted glyph in the font face
        if freetype::ffi::FT_Load_Glyph(face, freetype::ffi::FT_Get_Char_Index(face, character as freetype::ffi::FT_ULong), freetype::ffi::FT_LOAD_RENDER) != 0 {
            return None;
        }
        let bitmap = &(*(*face).glyph).bitmap;

        // carriage return our cursor if we don't have enough room to write the next caracter
        if cursor_offset.0 + (bitmap.width as u32) >= texture_width {
            assert!(bitmap.width as u32 <= texture_width);       // if this fails, we should increase texture_width
            cursor_offset.0 = 0;
            cursor_offset.1 += rows_to_skip;
            rows_to_skip = 0;
        }

        // if the texture data buffer has not enough lines, adding some
        if rows_to_skip < bitmap.rows as u32 {
            let diff = (bitmap.rows as u32) - rows_to_skip;
            rows_to_skip = bitmap.rows as u32;
            texture_data.grow((diff * texture_width) as uint, 0.0);
        }

        // copying the data to the texture
        let offset_x_before_copy = cursor_offset.0;
        if bitmap.rows >= 1 {
            let destination = texture_data.slice_from_mut((cursor_offset.0 + cursor_offset.1 * texture_width) as uint);
            let source = std::c_vec::CVec::new(std::mem::transmute(bitmap.buffer), destination.len());
            let source = source.as_slice();

            for y in range(0, bitmap.rows as u32) {
                let source = source.slice_from((y * bitmap.width as u32) as uint);
                let destination = destination.slice_from_mut((y * texture_width) as uint);

                for x in range(0, bitmap.width) {
                    // the values in source are bytes between 0 and 255, but we want floats between 0 and 1
                    let val: u8 = *source.get(x as uint).unwrap();
                    let max: u8 = std::num::Bounded::max_value();
                    let val = (val as f32) / (max as f32);
                    let dest = destination.get_mut(x as uint).unwrap();
                    *dest = val;
                }
            }

            cursor_offset.0 += bitmap.width as u32;
            debug_assert!(cursor_offset.0 <= texture_width);
        }

        // filling infos about that character
        // all informations are in pixels for the moment
        // when the texture dimensions will be determined, we will divide those by it
        let left_padding = (*(*face).glyph).bitmap_left;

        Some((character, CharacterInfos {
            left_padding: left_padding as f32,
            right_padding: (((*(*face).glyph).advance.x >> 6) as i32 - bitmap.width - left_padding) as f32,
            height_over_line: (*(*face).glyph).bitmap_top as f32,
            size: (bitmap.width as f32, bitmap.rows as f32),
            coords: (offset_x_before_copy as f32, cursor_offset.1 as f32),
        }))
    }).collect();

    // adding blank lines at the end until the height of the texture is a power of two
    {
        let current_height = texture_data.len() as u32 / texture_width;
        let requested_height = get_nearest_po2(current_height);
        texture_data.grow((texture_width * (requested_height - current_height)) as uint, 0.0);
    }

    // now our texture is finished
    // we know its final dimensions, so we can divide all the pixels values into (0,1) range
    assert!((texture_data.len() as u32 % texture_width) == 0);
    let texture_height = (texture_data.len() as u32 / texture_width) as f32;
    let float_texture_width = texture_width as f32;
    for mut chr in characters_infos.iter_mut() {
        chr.1.left_padding /= float_texture_width;
        chr.1.right_padding /= float_texture_width;
        chr.1.height_over_line /= texture_height;
        chr.1.size.0 /= float_texture_width;
        chr.1.size.1 /= texture_height;
        chr.1.coords.0 /= float_texture_width;
        chr.1.coords.1 /= texture_height;
    }

    // returning
    (texture_data, (texture_width, texture_height as u32), characters_infos)
}

/// Function that will calculate the nearest power of two.
fn get_nearest_po2(mut x: u32) -> u32 {
    assert!(x > 0);
    x -= 1;
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);
    x + 1
}
