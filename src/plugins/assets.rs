pub enum PictureType {
    ProfilePic { user_id: i32 },
}

pub fn get_picture_path(picture_type: PictureType) -> String {
    let mut path = String::from("static/pictures/");

    match picture_type {
        PictureType::ProfilePic { user_id } => {
            path.push_str("profile_pics/");
            path.push_str(&user_id.to_string());
            path.push_str(".png");
        }
    }

    path
}
