#![allow(dead_code)]

use diesel::prelude::*;

use super::{
    models::{HueBridge, NewHueBridge, UpdateHueBridge, UserSettings},
    schema::huebridges,
};

impl HueBridge {
    pub fn create_huebridge<'a>(
        conn: &mut SqliteConnection,
        new_huebridge: &NewHueBridge<'a>,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            let result = diesel::insert_into(huebridges::table)
                .values(new_huebridge)
                .execute(conn);

            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let inserted_huebridge = huebridges::table.order(huebridges::id.desc()).first(conn);

            if inserted_huebridge.is_err() {
                return inserted_huebridge;
            }

            Ok(inserted_huebridge.unwrap())
        })
    }

    pub fn get_huebridge(
        conn: &mut SqliteConnection,
        id: i32,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| huebridges::table.find(id).first(conn))
    }

    pub fn get_huebridge_by_bridge_id(
        conn: &mut SqliteConnection,
        user_id: i32,
        bridge_id: &str,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            let user_settings_result = UserSettings::get_usersettings_by_user_id(conn, user_id);

            if user_settings_result.is_err() {
                return Err(diesel::result::Error::NotFound);
            }

            let user_settings = user_settings_result.unwrap();

            let huebridge = huebridges::table
                .filter(huebridges::id.eq(bridge_id))
                .filter(huebridges::user_settings_id.eq(user_settings.id))
                .first(conn);

            if huebridge.is_err() {
                return Err(diesel::result::Error::NotFound);
            }

            Ok(huebridge.unwrap())
        })
    }

    pub fn get_huebridges(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        conn.transaction(|conn| huebridges::table.load::<HueBridge>(conn))
    }

    pub fn get_huebridges_by_user_id(
        conn: &mut SqliteConnection,
        user_id: i32,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        conn.transaction(|conn| {
            huebridges::table
                .filter(huebridges::user_settings_id.eq(user_id))
                .load::<HueBridge>(conn)
        })
    }

    pub fn update<'a>(
        &self,
        conn: &mut SqliteConnection,
        update_huebridge: &UpdateHueBridge<'a>,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            let result = diesel::update(self).set(update_huebridge).execute(conn);

            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let updated_huebridge = huebridges::table.find(self._id).first(conn);

            if updated_huebridge.is_err() {
                return updated_huebridge;
            }

            Ok(updated_huebridge.unwrap())
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        conn.transaction(|conn| diesel::delete(self).execute(conn))
    }
}
