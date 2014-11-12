use std::os;
use std::io::File;

mod textures;

fn main() {
    let dest = Path::new(os::getenv("OUT_DIR").unwrap());

    textures::build_texture_file(&mut File::create(&dest.join("textures.rs")).unwrap());
}
