use diesel::prelude::*;

use super::schema::{huebridges, users, usersettings, wleditems};

#[derive(Queryable, PartialEq, Identifiable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub hashed_password: String,
}

#[derive(Insertable, PartialEq, Selectable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub hashed_password: &'a str,
}

#[derive(AsChangeset, PartialEq, Selectable)]
#[diesel(table_name = users)]
pub struct UpdateUser<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub hashed_password: Option<&'a str>,
}

#[derive(Queryable, PartialEq, Identifiable, Selectable, Associations)]
#[diesel(table_name = usersettings)]
#[diesel(belongs_to(User))]
pub struct UserSettings {
    pub id: i32,
    pub hue_index: i32,
    pub user_id: i32,
}

#[derive(Insertable, PartialEq, Associations)]
#[diesel(table_name = usersettings)]
#[diesel(belongs_to(User))]
pub struct NewUserSettings<'a> {
    pub hue_index: &'a i32,
    pub user_id: &'a i32,
}

#[derive(AsChangeset, PartialEq, Associations)]
#[diesel(table_name = usersettings)]
#[diesel(belongs_to(User))]
pub struct UpdateUserSettings<'a> {
    pub hue_index: Option<&'a i32>,
    pub user_id: Option<&'a i32>,
}

#[derive(Queryable, PartialEq, Identifiable, Selectable, Associations)]
#[diesel(table_name = huebridges)]
#[diesel(belongs_to(UserSettings))]
#[diesel(primary_key(_id))]
pub struct HueBridge {
    pub _id: i32,
    pub id: String,
    pub ip: String,
    pub user: String,
    pub user_settings_id: i32,
}

#[derive(Insertable, PartialEq, Associations)]
#[diesel(table_name = huebridges)]
#[diesel(belongs_to(UserSettings))]
pub struct NewHueBridge<'a> {
    pub id: &'a str,
    pub ip: &'a str,
    pub user: &'a str,
    pub user_settings_id: &'a i32,
}

#[derive(AsChangeset, PartialEq, Associations)]
#[diesel(table_name = huebridges)]
#[diesel(belongs_to(UserSettings))]
pub struct UpdateHueBridge<'a> {
    pub id: Option<&'a str>,
    pub ip: Option<&'a str>,
    pub user: Option<&'a str>,
    pub user_settings_id: Option<&'a i32>,
}

#[derive(Queryable, PartialEq, Identifiable, Selectable, Associations)]
#[diesel(table_name = wleditems)]
#[diesel(belongs_to(UserSettings))]
#[diesel(primary_key(_id))]
pub struct WledItem {
    pub _id: i32,
    pub ip: String,
    pub name: String,
    pub user_settings_id: i32,
}

#[derive(Insertable, PartialEq, Associations)]
#[diesel(table_name = wleditems)]
#[diesel(belongs_to(UserSettings))]
pub struct NewWledItem<'a> {
    pub ip: &'a str,
    pub name: &'a str,
    pub user_settings_id: &'a i32,
}

#[derive(AsChangeset, PartialEq, Associations)]
#[diesel(table_name = wleditems)]
#[diesel(belongs_to(UserSettings))]
pub struct UpdateWledItem<'a> {
    pub ip: Option<&'a str>,
    pub name: Option<&'a str>,
    pub user_settings_id: Option<&'a i32>,
}
