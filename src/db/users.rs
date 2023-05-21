#![allow(dead_code)]

use super::huebridges::get_huebridges_by_user_id;
use super::models::{
    HueBridge, NewUser, NewUserSettings, UpdateUser, User, UserSettings, WledItem,
};
use super::schema::{users, usersettings};
use super::usersettings::get_usersettings_by_user_id;
use super::wleditems::get_wleditems_by_user_id;
use diesel::prelude::*;

pub fn create_user<'a>(
    conn: &mut SqliteConnection,
    new_user: &NewUser<'a>,
) -> Result<User, diesel::result::Error> {
    conn.transaction(|conn| {
        diesel::insert_into(users::table)
            .values(new_user)
            .execute(conn)
            .expect("Error saving new user");

        let inserted_user: User = users::table
            .order(users::id.desc())
            .first(conn)
            .expect("Error saving new user");

        diesel::insert_into(usersettings::table)
            .values(NewUserSettings {
                hue_index: &0,
                user_id: &inserted_user.id,
            })
            .execute(conn)
            .expect("Error saving new user settings");

        Ok(inserted_user)
    })
}

pub fn get_users(conn: &mut SqliteConnection) -> Result<Vec<User>, diesel::result::Error> {
    conn.transaction(|conn| {
        let users: Vec<User> = users::table
            .load::<User>(conn)
            .expect("Error getting users");

        Ok(users)
    })
}

pub fn get_user(conn: &mut SqliteConnection, user_id: i32) -> Result<User, diesel::result::Error> {
    conn.transaction(|conn| {
        let user: User = users::table
            .find(user_id)
            .first(conn)
            .expect("Error getting user");

        Ok(user)
    })
}

pub fn get_user_by_mail(
    conn: &mut SqliteConnection,
    user_mail: &str,
) -> Result<User, diesel::result::Error> {
    conn.transaction(|conn| {
        let user: User = users::table
            .filter(users::email.eq(user_mail))
            .first(conn)
            .expect("Error getting user");

        Ok(user)
    })
}

impl User {
    pub fn update<'a>(
        &self,
        conn: &mut SqliteConnection,
        update_user: &UpdateUser<'a>,
    ) -> Result<User, diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::update(self)
                .set(update_user)
                .execute(conn)
                .expect("Error updating user");

            let updated_user: User = users::table
                .find(self.id)
                .first(conn)
                .expect("Error updating user");

            Ok(updated_user)
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::delete(self)
                .execute(conn)
                .expect("Error deleting user");

            Ok(())
        })
    }

    pub fn get_usersettings(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<UserSettings, diesel::result::Error> {
        get_usersettings_by_user_id(conn, self.id)
    }

    pub fn get_huebridges(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        get_huebridges_by_user_id(conn, self.id)
    }

    pub fn get_wleditems(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<WledItem>, diesel::result::Error> {
        get_wleditems_by_user_id(conn, self.id)
    }
}
