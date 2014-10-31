extern crate image;
use glium;

/// 
pub struct Image {
    image: self::image::DynamicImage,
}

impl Image {
    /// Loads an image.
    pub fn load<R: Reader + Seek>(reader: R) -> Result<Image, self::image::ImageError> {
        let image = match image::load(reader, self::image::PNG) {
            Ok(img) => img,
            Err(e) => return Err(e)
        };

        Ok(Image {
            image: image
        })
    }
}

impl glium::texture::Texture2DData<(u8, u8, u8, u8)> for Image {
    fn get_dimensions(&self) -> (u32, u32) {
        use self::image::GenericImage;
        self.image.dimensions()
    }

    fn into_vec(self) -> Vec<(u8, u8, u8, u8)> {
        self.image.to_rgba().into_vec()
            .into_iter()
            .map(|p| p.channels())
            .collect()
    }

    fn from_vec(_: Vec<(u8, u8, u8, u8)>, _: u32) -> Image {
        unimplemented!()
    }
}
