use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub enum PictureType {
    PROFILE_PIC { user_id: i32 },
}

pub fn get_picture_path(picture_type: PictureType) -> String {
    let mut path = String::from("static/pictures/");

    match picture_type {
        PictureType::PROFILE_PIC { user_id } => {
            path.push_str("profile_pics/");
            path.push_str(&user_id.to_string());
            path.push_str(".png");
        }
    }

    path
}

pub fn get_picture(picture_type: PictureType) -> Option<Vec<u8>> {
    let path = get_picture_path(picture_type);

    if !Path::new(&path).exists() {
        return None;
    }

    let mut file = File::open(path).unwrap();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).unwrap();

    Some(buffer)
}

pub fn save_picture(picture_type: PictureType, picture: &[u8]) -> Result<(), ()> {
    let path = get_picture_path(picture_type);

    let parent_path = Path::new(&path).parent().unwrap();

    if !parent_path.exists() {
        std::fs::create_dir(parent_path).unwrap();
    }

    if Path::new(&path).exists() {
        std::fs::remove_file(&path).unwrap();
    }

    let mut file = File::create(path).unwrap();

    file.write_all(picture).unwrap();

    Ok(())
}
