use crate::error::{Error, Result};
use png::{Decoder, Encoder, OutputInfo, text_metadata::ZTXtChunk};
use std::{
    fs::{create_dir_all, File},
    path::Path,
};

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
    pub fn new<'a>(metadata: String) -> IconMetaData<'a> {
        let icon_data: (f32, u32, u32, Vec<IconState<'_>>) = IconMetaData::parse_metadata(metadata);
        //let icon_state_data: Vec<IconState> = IconState::
        IconMetaData {
            version: icon_data.0,
            width: icon_data.1,
            height: icon_data.2,
            icon_states: icon_data.3
        }
    }

    fn parse_metadata<'a>(metadata: String) -> (f32, u32, u32, Vec<IconState<'a>>) {
        let lines: std::str::Lines = metadata.lines();

        let mut dmi_version: f32 = 0.0;
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        let mut icon_meta_data: String = "".to_string();
        let mut icon_states: Vec<IconState<'a>> = vec![];

        for current_line in lines {
            let keywords: std::str::SplitWhitespace = current_line.split_whitespace();
            let keyword: &str = keywords.next().unwrap();

            match keyword {
                "Description" => continue, //png zTxt metadata keyword
                "#" => continue, //# BEGIN DMI, metadata header
                "version" => dmi_version = keywords.skip(2).next().unwrap().parse::<f32>().unwrap(),
                "width" => width = keywords.skip(2).next().unwrap().parse::<u32>().unwrap(),
                "height" => height = keywords.skip(2).next().unwrap().parse::<u32>().unwrap(),

                //icon_state metadata that fills the vector with IconState structs
                "state" => {match IconState::parse_icon_state(keyword, &mut keywords, &mut lines) {
                    Ok(IconState) => (),
                    Err(IconState) => ()
                }},
            }


        }

        (dmi_version, width, height, icon_states)
    }

}

#[derive(Debug)]
pub struct IconState<'a> {
    ///the name of the icon_state
    state_name: &'a str,
    ///number of directional states we have, should always be 1, 4, or 8
    number_of_dirs: u32,
    ///array of frame delays
    delays: &'a [u32],
    ///total number of animation frames
    number_of_frames: u32,

    moving: bool,
}


impl IconState<'_> {
    pub fn parse_icon_state<'a>(starting_keyword: &str, current_line: &mut std::str::SplitWhitespace<'a>, lines: &mut std::str::Lines,) -> Option<IconState<'a>> {
        //we're called when the IconMetaData struct finds a "state" line, which is of the form: "state = "state_name""

        let mut keywords: std::str::SplitWhitespace = *current_line;
        let mut lines_left: std::str::Lines = *lines;
        let mut new_icon_state: IconState<'a> = IconState{state_name: "", number_of_dirs: 1, delays: &[0], number_of_frames: 1, moving: false};

        if starting_keyword != "value" {
            return None //TODOKYLER: figure out how errors work
        }
        match keywords.skip(2).next() {
            Some(new_value) => new_icon_state.state_name = new_value,
            None => return None //TODOKYLER: figure out how errors work
        };
        loop {
            match lines_left.next() {
                Some(next_line) => {
                    let current_line_iterator = next_line.split_whitespace();
                    match current_line_iterator.next() {
                        Some("dirs") => new_icon_state.parse_dir(current_line_iterator.skip(2).next()),
                        Some("frames") => new_icon_state.parse_frames(current_line_iterator.skip(2).next()),
                        Some("delay") => new_icon_state.parse_delay(current_line_iterator),
                    }

                }
                _ => return Some(new_icon_state),
            }
        }



        Some(new_icon_state)
    }

    fn parse_dir(&self, value: Option<&str>) -> () {
        match value {
            Some(dir_num) => self.number_of_dirs = dir_num.parse::<u32>().unwrap(),
            _ => return //TODOKYLER: make this return an error state
        }
    }

    fn parse_frames(&self, value: Option<&str>) -> () {
        match value {
            Some(dir_num) => self.number_of_frames = dir_num.parse::<u32>().unwrap(),
            _ => return //TODOKYLER: make this return an error state
        }
    }

    fn parse_delay(&self, current_line: &mut std::str::SplitWhitespace) -> () {
        match value {
            Some(dir_num) => self.number_of_dirs = dir_num.parse::<u32>().unwrap(),
            _ => return //TODOKYLER: make this return an error state
        }
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
        let icon: IconMetaData<'_> = IconMetaData::new(uncompressed_chunk);
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
    encoder.set_color(png::ColorType::Rgb);
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
