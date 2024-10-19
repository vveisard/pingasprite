use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Rgba, RgbaImage, SubImage};
use serde_derive::Deserialize;
use std::{
  collections::HashMap,
  env,
  fs::{self},
  path::Path,
};
use toml::{self};

/// configuration for the rect of a sprite in a sprite sheet.
#[derive(Debug, Deserialize)]
struct SpriteRectConfig {
  top: u32,
  left: u32,
  width: u32,
  height: u32,
}

/// configuration for a sprite in a sprite sheet.
#[derive(Debug, Deserialize)]
struct SpriteConfig(SpriteRectConfig);

/// configuration for io
#[derive(Debug, Deserialize)]
struct IoConfig {
  /// path of the sprite sheet input file, relative to the config file
  input_sprite_sheet_file_path: String,
  /// path of the sprite output directory, relative to the config file
  output_sprite_directory_path: String,
  // extension of the output sprites. used for output path and format
  output_sprite_format: String,
}

/// congiguration for program
#[derive(Debug, Deserialize)]
struct Config {
  io: IoConfig,
  /// key: RGB hex value of input color, without leading #.
  /// value: RGB hex value of output color, with leading #.
  replacements: HashMap<String, String>,
  sprites: Vec<SpriteConfig>,
}

fn main() {
  let program_args: Vec<String> = env::args().collect();
  let config_file_path_arg = &program_args[1];
  let config_file_path = Path::new(config_file_path_arg);
  let config_file_contents = fs::read_to_string(config_file_path).unwrap();
  let config = toml::from_str::<Config>(&config_file_contents).unwrap();

  let input_sprite_sheet_file_path = Path::join(
    config_file_path.parent().unwrap(),
    config.io.input_sprite_sheet_file_path,
  );

  let output_sprite_directory_path = Path::join(
    config_file_path.parent().unwrap(),
    config.io.output_sprite_directory_path,
  );

  let output_sprite_image_extension = config.io.output_sprite_format;
  let output_sprite_image_format: ImageFormat =
    ImageFormat::from_extension(&output_sprite_image_extension).unwrap();

  let input_sprite_sheet_image = ImageReader::open(input_sprite_sheet_file_path)
    .unwrap()
    .decode()
    .unwrap();

  let mut input_sprite_images: Vec<SubImage<&DynamicImage>> = Vec::new();
  for sprite_config in config.sprites.iter() {
    let sprite_image = input_sprite_sheet_image.view(
      sprite_config.0.left,
      sprite_config.0.top,
      sprite_config.0.width,
      sprite_config.0.height,
    );

    input_sprite_images.push(sprite_image);
  }

  let mut output_sprite_images: Vec<RgbaImage> = Vec::new();
  for input_sprite_image in input_sprite_images {
    let mut output_sprite_image =
      RgbaImage::new(input_sprite_image.width(), input_sprite_image.height());

    // iterate each pixel in the input sprite image and replace pixels using config
    for (input_image_pixel_x, input_image_pixel_y, input_sprite_image_pixel_rgba) in
      input_sprite_image.pixels()
    {
      // TODO refactor next pixel using expression syntax or function
      let mut output_pixel_rgba: Rgba<u8> = input_sprite_image_pixel_rgba;

      // without leading # character
      let input_sprite_image_pixel_hex = hex::encode(input_sprite_image_pixel_rgba.0);

      for (replacement_input_hex, replacement_output_hex) in config.replacements.iter() {
        if input_sprite_image_pixel_hex != *replacement_input_hex {
          continue;
        }

        let decoded_replacement_output_hex: Vec<u8> = hex::decode(replacement_output_hex).unwrap();

        if decoded_replacement_output_hex.len() != 4 {
          panic!("invalid length {}", replacement_output_hex);
        }

        output_pixel_rgba = image::Rgba([
          decoded_replacement_output_hex[0],
          decoded_replacement_output_hex[1],
          decoded_replacement_output_hex[2],
          decoded_replacement_output_hex[3],
        ]);
      }

      output_sprite_image.put_pixel(input_image_pixel_x, input_image_pixel_y, output_pixel_rgba);
    }

    output_sprite_images.push(output_sprite_image);
  }

  // write sprites to output
  // TODO parallel
  for (index, output_sprite_image) in output_sprite_images.iter().enumerate() {
    let output_sprite_image_file_name = format!("{}.{}", index, output_sprite_image_extension);

    let output_sprite_output_file_path =
      Path::join(&output_sprite_directory_path, output_sprite_image_file_name);

    output_sprite_image
      .save_with_format(output_sprite_output_file_path, output_sprite_image_format)
      .unwrap();
  }
}
