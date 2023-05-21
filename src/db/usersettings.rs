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
        conn.transaction(|conn| {
            let usersettings: Vec<UserSettings> = usersettings::table
                .load::<UserSettings>(conn)
                .expect("Error getting user settings");
    
            Ok(usersettings)
        })
    }
    
    pub fn get_usersettings_by_user_id(
        conn: &mut SqliteConnection,
        user_id: i32,
    ) -> Result<UserSettings, diesel::result::Error> {
        conn.transaction(|conn| {
            let usersettings: UserSettings = usersettings::table
                .filter(usersettings::user_id.eq(user_id))
                .first(conn)
                .expect("Error getting user settings");
    
            Ok(usersettings)
        })
    }

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        update_usersettings: &UpdateUserSettings,
    ) -> Result<UserSettings, diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::update(self)
                .set(update_usersettings)
                .execute(conn)
                .expect("Error updating user settings");

            let updated_usersettings: UserSettings = usersettings::table
                .find(self.id)
                .first(conn)
                .expect("Error updating user settings");

            Ok(updated_usersettings)
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::delete(self)
                .execute(conn)
                .expect("Error deleting user settings");

            Ok(())
        })
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
