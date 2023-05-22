#![allow(dead_code)]

use super::models::{
    HueBridge, NewUser, NewUserSettings, UpdateUser, User, UserSettings, WledItem,
};
use super::schema::{users, usersettings};
use diesel::prelude::*;

impl User {
    pub fn create_user<'a>(
        conn: &mut SqliteConnection,
        new_user: &NewUser<'a>,
    ) -> Result<User, diesel::result::Error> {
        conn.transaction(|conn| {
            let create_user_result = diesel::insert_into(users::table)
                .values(new_user)
                .execute(conn);

            if create_user_result.is_err() {
                return Err(create_user_result.unwrap_err());
            }

            let inserted_user_result = users::table.order(users::id.desc()).first(conn);

            if inserted_user_result.is_err() {
                return inserted_user_result;
            }

            let inserted_user: User = inserted_user_result.unwrap();

            let create_usersettings_result = diesel::insert_into(usersettings::table)
                .values(NewUserSettings {
                    hue_index: &0,
                    user_id: &inserted_user.id,
                })
                .execute(conn);

            if create_usersettings_result.is_err() {
                return Err(create_usersettings_result.unwrap_err());
            }

            Ok(inserted_user)
        })
    }

    pub fn get_users(conn: &mut SqliteConnection) -> Result<Vec<User>, diesel::result::Error> {
        conn.transaction(|conn| users::table.load::<User>(conn))
    }

    pub fn get_user(
        conn: &mut SqliteConnection,
        user_id: i32,
    ) -> Result<User, diesel::result::Error> {
        conn.transaction(|conn| users::table.find(user_id).first(conn))
    }

    pub fn get_user_by_mail(
        conn: &mut SqliteConnection,
        user_mail: &str,
    ) -> Result<User, diesel::result::Error> {
        conn.transaction(|conn| users::table.filter(users::email.eq(user_mail)).first(conn))
    }

    pub fn get_user_by_username(
        conn: &mut SqliteConnection,
        user_name: &str,
    ) -> Result<User, diesel::result::Error> {
        conn.transaction(|conn| users::table.filter(users::username.eq(user_name)).first(conn))
    }

    pub fn update<'a>(
        &self,
        conn: &mut SqliteConnection,
        update_user: &UpdateUser<'a>,
    ) -> Result<User, diesel::result::Error> {
        conn.transaction(|conn| {
            let result = diesel::update(self).set(update_user).execute(conn);

            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let updated_user = users::table.find(self.id).first(conn);

            if updated_user.is_err() {
                return updated_user;
            }

            Ok(updated_user.unwrap())
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        conn.transaction(|conn| diesel::delete(self).execute(conn))
    }

    pub fn get_usersettings(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<UserSettings, diesel::result::Error> {
        UserSettings::get_usersettings_by_user_id(conn, self.id)
    }

    pub fn get_huebridges(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        HueBridge::get_huebridges_by_user_id(conn, self.id)
    }

    pub fn get_wleditems(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<WledItem>, diesel::result::Error> {
        WledItem::get_wleditems_by_user_id(conn, self.id)
    }
}
