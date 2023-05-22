#![allow(dead_code)]

use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};

use super::models::{HueBridge, WledItem};
use super::{
    models::{UpdateUserSettings, UserSettings},
    schema::usersettings,
};

impl UserSettings {
    pub fn get_usersettings(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<UserSettings>, diesel::result::Error> {
        conn.transaction(|conn| usersettings::table.load::<UserSettings>(conn))
    }

    pub fn get_usersettings_by_user_id(
        conn: &mut SqliteConnection,
        user_id: i32,
    ) -> Result<UserSettings, diesel::result::Error> {
        conn.transaction(|conn| {
            usersettings::table
                .filter(usersettings::user_id.eq(user_id))
                .first(conn)
        })
    }

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        update_usersettings: &UpdateUserSettings,
    ) -> Result<UserSettings, diesel::result::Error> {
        conn.transaction(|conn| {
            let result = diesel::update(self).set(update_usersettings).execute(conn);

            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let updated_usersettings = usersettings::table.find(self.id).first(conn);

            if updated_usersettings.is_err() {
                return updated_usersettings;
            }

            Ok(updated_usersettings.unwrap())
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        conn.transaction(|conn| diesel::delete(self).execute(conn))
    }

    pub fn get_huebridges(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        HueBridge::get_huebridges_by_user_id(conn, self.user_id)
    }

    pub fn get_wleditems(
        &self,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<WledItem>, diesel::result::Error> {
        WledItem::get_wleditems_by_user_id(conn, self.user_id)
    }
}
