// Based on https://github.com/Erkaman/image-load-store-demo
// License included at the end of the file.
//
// This example showcases image load/store functionality in compute shaders, which
// allows you to read and write to arbitrary textures in a shader.
//
// Three compute shaders are launched. The first one draws the mandelbrot fractal. The second
// one applies a box filter on the output of the first. Finally, the third one copies data
// the original unsigned integer texture to a signed normalized integer texture.
// Finally, this last texture is blitted to the display framebuffer.

#[macro_use]
extern crate glium;

use std::time::Instant;

use glium::{
    program::ComputeShader, texture::UnsignedTexture2d, uniform, Display, Surface, Texture2d,
};
use glutin::surface::WindowSurface;
use support::{ApplicationContext, State};

mod support;

struct Application {
    start_time: Instant,
    fract_texture: UnsignedTexture2d,
    final_texture: Texture2d,
    fractal_shader: ComputeShader,
    box_blur_shader: ComputeShader,
    copy_shader: ComputeShader,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE:&'static str = "Glium compute_image example";

    fn new(display: &Display<WindowSurface>) -> Self {
        let start_time = Instant::now();

        let fract_texture = UnsignedTexture2d::empty_with_format(
            display,
            glium::texture::UncompressedUintFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            1024,
            1024,
        )
        .unwrap();

        let final_texture = Texture2d::empty_with_format(
            display,
            glium::texture::UncompressedFloatFormat::U8U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            1024,
            1024,
        )
        .unwrap();

        let fractal_shader = ComputeShader::from_source(display, r#"\
    #version 430
    layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;



    uniform uint uWidth;
    uniform uint uHeight;
    uniform float uTime;
    uniform layout(binding=3, rgba8ui) writeonly uimage2D uFractalTexture;

    void main() {
    ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
    vec2 uv = vec2(i) * vec2(1.0 / float(uWidth), 1.0 / float(uHeight));


    float n = 0.0;
    vec2 c = vec2(-.745, .186) +  (uv - 0.5)*(2.0+ 1.7*cos(1.8*uTime)  ),
        z = vec2(0.0);
    const int M =128;
    for (int i = 0; i<M; i++)
        {
        z = vec2(z.x*z.x - z.y*z.y, 2.*z.x*z.y) + c;
        if (dot(z, z) > 2) break;

        n++;
        }
    vec3 bla = vec3(0,0,0.0);
    vec3 blu = vec3(0,0,0.8);
    vec4 color;
    if( n >= 0 && n <= M/2-1 ) { color = vec4( mix( vec3(0.2, 0.1, 0.4), blu, n / float(M/2-1) ), 1.0) ;  }
    if( n >= M/2 && n <= M ) { color = vec4( mix( blu, bla, float(n - M/2 ) / float(M/2) ), 1.0) ;  }

    imageStore(uFractalTexture, i , uvec4(color * 255.0f));
    }
        "#).unwrap();

        let box_blur_shader = glium::program::ComputeShader::from_source(
            display,
            r#"\
    #version 430
    layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

    uniform uint uWidth;
    uniform uint uHeight;
    uniform layout(binding=3, rgba8ui) uimage2D uFractalTexture;

    // sample with clamping from the texture.
    vec4 csample(ivec2 i) {
    i = ivec2(clamp(i.x, 0, uWidth-1), clamp(i.y, 0, uHeight-1));
    return imageLoad(uFractalTexture, i);
    }

    #define R 8
    #define W (1.0 / ((1.0+2.0*float(R)) * (1.0+2.0*float(R))))
    void main() {
    ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);

    vec4 sum = vec4(0.0);
    // first compute the blurred color.
        for(int x = -R; x <= +R; x++ )
        for(int y = -R; y <= +R; y++ )
        sum += W * csample(i + ivec2(x,y));

    // now store the blurred color.
    imageStore(uFractalTexture,  i, uvec4(sum) );
    }

        "#,
        )
        .unwrap();

        let copy_shader = glium::program::ComputeShader::from_source(
            display,
            r#"\
    #version 430
    layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

    uniform layout(binding=3, rgba8ui) readonly uimage2D uFractalTexture;
    uniform layout(binding=4, rgba8) writeonly image2D destTexture;

    void main() {
    ivec2 i = ivec2(gl_GlobalInvocationID.x, gl_GlobalInvocationID.y);
    vec3 c = vec3(imageLoad(uFractalTexture, i).xyz);
    vec3 cnorm = c/255.0;
    imageStore(destTexture, i, vec4(cnorm,1.0));
    }
        "#,
        )
        .unwrap();

        Self {
            start_time,
            fract_texture,
            final_texture,
            fractal_shader,
            box_blur_shader,
            copy_shader,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let frame = display.draw();
        let image_unit = self
            .fract_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8UI)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::Write);
        self.fractal_shader.execute(
            uniform! {
                uWidth: self.fract_texture.width(),
                uHeight: self.fract_texture.height(),
                uFractalTexture: image_unit,
                uTime: Instant::now().duration_since(self.start_time.clone()).as_secs_f32(),
            },
            self.fract_texture.width(),
            self.fract_texture.height(),
            1,
        );

        let image_unit = self
            .fract_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8UI)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::ReadWrite);
        self.box_blur_shader.execute(
            uniform! {
                uWidth: self.fract_texture.width(),
                uHeight: self.fract_texture.height(),
                uFractalTexture: image_unit,
            },
            self.fract_texture.width(),
            self.fract_texture.height(),
            1,
        );

        let fract_unit = self
            .fract_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8UI)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::Read);
        let final_unit = self
            .final_texture
            .image_unit(glium::uniforms::ImageUnitFormat::RGBA8)
            .unwrap()
            .set_access(glium::uniforms::ImageUnitAccess::Write);

        self.copy_shader.execute(
            uniform! {
                uFractalTexture: fract_unit,
                destTexture: final_unit,
            },
            self.fract_texture.width(),
            self.fract_texture.height(),
            1,
        );

        // drawing a frame
        self.final_texture
            .as_surface()
            .fill(&frame, glium::uniforms::MagnifySamplerFilter::Nearest);

        frame.finish().unwrap();
    }
}

fn main() {
    State::<Application>::run_loop();
}

// The MIT License (MIT)

// Copyright (c) 2016 Eric Arnebäck

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
