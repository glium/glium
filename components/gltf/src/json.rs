#![allow(dead_code)]

use std::collections::HashMap;

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct GLTFDocument {
    pub allExtensions: Option<Vec<String>>,
    pub accessors: Option<Accessor>,
    pub animations: Option<bool>,  // FIXME: 
    pub asset: Option<Asset>,
    pub buffers: Option<HashMap<String, Buffer>>,
    pub bufferViews: Option<HashMap<String, BufferView>>,
    pub cameras: Option<bool>,  // FIXME: 
    pub images: Option<HashMap<String, Image>>,
    pub lights: Option<bool>,  // FIXME: 
    pub materials: Option<bool>,  // FIXME: 
    pub meshes: Option<bool>,  // FIXME: 
    pub nodes: Option<bool>,  // FIXME: 
    pub programs: Option<HashMap<String, Program>>,
    pub samplers: Option<bool>,  // FIXME: 
    pub scene: Option<String>,
    pub scenes: Option<bool>,  // FIXME: 
    pub shaders: Option<HashMap<String, Shader>>,
    pub skins: Option<bool>,  // FIXME: 
    pub techniques: Option<bool>,  // FIXME: 
    pub textures: Option<HashMap<String, Texture>>,
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Accessor {
    pub bufferView: String,
    pub byteOffset: int,
    pub byteStride: Option<u8>,
    pub componentType: int,
    pub count: int,
    pub type_: String,     // FIXME: 
    pub max: Option<Vec<f32>>,
    pub min: Option<Vec<f32>>,
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Asset {
    pub copyright: String,
    pub generator: String,
    pub premultipliedAlpha: Option<bool>,
    pub profile: Option<String>,
    pub version: Option<f32>,
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Buffer {
    pub uri: String,
    pub byteLength: Option<uint>,
    pub type_: String,      // FIXME: 
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct BufferView {
    pub buffer: String,
    pub byteOffset: uint,
    pub byteLength: Option<uint>,
    pub target: uint,
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Image {
    pub uri: String,
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Program {
    pub attributes: Option<Vec<String>>,
    pub fragmentShader: String,
    pub vertexShader: String,
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Shader {
    pub uri: String,
    pub type_: int,     // FIXME: 
}

#[deriving(Decodable, Show, Clone)]
#[allow(non_snake_case)]
pub struct Texture {
    pub format: Option<int>,
    pub internalFormat: Option<int>,
    pub sampler: String, 
    pub source: String,
    pub target: Option<int>,
    pub type_: Option<int>,     // FIXME: 
}
