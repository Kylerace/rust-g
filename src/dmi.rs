use crate::error::{Error, Result};
use png::{Decoder, Encoder, OutputInfo, text_metadata::ZTXtChunk};
use std::{
    fs::{create_dir_all, File},
    path::Path,
};
/*
use inflate::inflate_bytes;
use std::str::from_utf8;
*/

byond_fn!(fn dmi_strip_metadata(path) {
    strip_metadata(path).err()
});

byond_fn!(fn dmi_create_png(path, width, height, data) {
    create_png(path, width, height, data).err()
});

byond_fn!(fn dmi_resize_png(path, width, height, resizetype) {
    let resizetype = match resizetype {
        "catmull" => image::imageops::CatmullRom,
        "gaussian" => image::imageops::Gaussian,
        "lanczos3" => image::imageops::Lanczos3,
        "nearest" => image::imageops::Nearest,
        "triangle" => image::imageops::Triangle,
        _ => image::imageops::Nearest,
    };
    resize_png(path, width, height, resizetype).err()
});

byond_fn!(fn rustg_icon_states(icon_path, icon_state, dir, frame, moving) {
    icon_states(icon_path, icon_state, dir, frame, moving).err()
});

pub struct IconMetaData<'a> {
    ///DMI format version: this should never be anything other than 4.0
    pub version: f32,
    ///width in pixels of every icon_state in this icon
    pub width: u32,
    ///height in pixels of every icon_state in this icon
    pub height: u32,
    ///list of icon_state metadata structs
    pub icon_states: Vec<IconState<'a>>

}

impl IconMetaData<'_> {
    pub fn new(metadata: String) -> IconMetaData<'static> {//TODOKYLER: i dont think i actually want 'static
        let icon_data = IconMetaData::parse_metadata_until_icon_states(metadata);
        //let icon_state_data: Vec<IconState> = IconState::
        /*IconMetaData {
            version: icon_data.0,
            width: icon_data.1,
            height: icon_data.2,
        }*/
    }

    fn parse_metadata_until_icon_states(metadata: String) -> (f32, u32, u32, String) {
        let mut lines = metadata.lines();

        let mut dmi_version: f32 = 0;
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut icon_meta_data: String = "".to_string();

        for current_line in lines {
            let mut keywords = current_line.split_whitespace();
            let mut last_found_keyword: &str = None;
            let mut last_found_value: &str = None;

            match keywords.next() {
                Some("version") => last_found_keyword = &dmi_version,
                Some("width") => last_found_keyword = &width,
                Some("height") => last_found_keyword = &height,
                Some("state") => break, //we got to the icon_state metadata, early out
                _ => Err("improper line data! expected an icon metadata keyword, got {}", _)
            }
            match keywords.next() {
                Some("=") => {
                    match keywords.next() {
                        Some(n @ _) => *last_found_keyword = n,
                        None => Err("improper line data! expected a value")
                    }
                }
                _ => Err("improper line data! expected = as second line element")
            }
        }

        (dmi_version, width, height, icon_meta_data)
    }
}

pub struct IconState<'a> {
    ///the name of the icon_state
    state_name: String,
    ///number of directional states we have, should always be 1, 4, or 8
    number_of_dirs: u32,
    ///array of frame delays
    delays: &'a [u32],
    ///total number of animation frames
    number_of_frames: u32,

    moving: bool,
}

impl IconState<'_> {
    pub fn parse_state_meta_data(remaining_metadata: String) -> Option<(String, IconState)> {

    }
}

fn icon_states(icon_path: &str, icon_state: &str, dir: &str, frame: &str, moving: &str) -> Result<String> {
    let decoder = png::Decoder::new(File::open(icon_path)?);
    let mut reader = decoder.read_info()?;
    let mut return_string: String = "".to_string();

    for text_chunk in &reader.info().compressed_latin1_text {
        if text_chunk.keyword != "Description" {
            continue
        }

        let uncompressed_chunk: String = text_chunk.get_text()?;
        let icon: IconMetaData = parse_icon_metadata(uncompressed_chunk)?;
    }

    Ok(return_string)
}

fn strip_metadata(path: &str) -> Result<()> {
    let (info, image) = read_png(path)?;
    write_png(path, info, image)
}

fn read_png(path: &str) -> Result<(OutputInfo, Vec<u8>)> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;

    let mut buf = vec![0; reader.output_buffer_size()];

    let info: OutputInfo = reader.next_frame(&mut buf)?;
    Ok((info, buf))
}

fn write_png(path: &str, info: OutputInfo, image: Vec<u8>) -> Result<()> {
    let mut encoder = Encoder::new(File::create(path)?, info.width, info.height);
    encoder.set_color(info.color_type);
    encoder.set_depth(info.bit_depth);

    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&image)?)
}

fn create_png(path: &str, width: &str, height: &str, data: &str) -> Result<()> {
    let width = width.parse::<u32>()?;
    let height = height.parse::<u32>()?;

    let bytes = data.as_bytes();
    if bytes.len() % 7 != 0 {
        return Err(Error::InvalidPngData);
    }

    let mut result: Vec<u8> = Vec::new();
    for pixel in bytes.chunks_exact(7) {
        for channel in pixel[1..].chunks_exact(2) {
            result.push(u8::from_str_radix(std::str::from_utf8(channel)?, 16)?);
        }
    }

    if let Some(fdir) = Path::new(path).parent() {
        if !fdir.is_dir() {
            create_dir_all(fdir)?;
        }
    }

    let mut encoder = Encoder::new(File::create(path)?, width, height);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    Ok(writer.write_image_data(&result)?)
}

fn resize_png<P: AsRef<Path>>(
    path: P,
    width: &str,
    height: &str,
    resizetype: image::imageops::FilterType,
) -> Result<()> {
    let width = width.parse::<u32>()?;
    let height = height.parse::<u32>()?;

    let img = image::open(path.as_ref())?;

    let newimg = img.resize(width, height, resizetype);

    Ok(newimg.save_with_format(path.as_ref(), image::ImageFormat::Png)?)
}
