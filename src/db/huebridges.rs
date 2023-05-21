#![allow(dead_code)]

use diesel::prelude::*;

use super::{
    models::{HueBridge, NewHueBridge, UpdateHueBridge},
    schema::huebridges,
};

impl HueBridge {
    pub fn create_huebridge<'a>(
        conn: &mut SqliteConnection,
        new_huebridge: &NewHueBridge<'a>,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::insert_into(huebridges::table)
                .values(new_huebridge)
                .execute(conn)
                .expect("Error saving new huebridge");

            let inserted_huebridge: HueBridge = huebridges::table
                .order(huebridges::id.desc())
                .first(conn)
                .expect("Error saving new huebridge");

            Ok(inserted_huebridge)
        })
    }

    pub fn get_huebridge(
        conn: &mut SqliteConnection,
        id: i32,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            let huebridge: HueBridge = huebridges::table
                .find(id)
                .first(conn)
                .expect("Error getting huebridge");

            Ok(huebridge)
        })
    }

    pub fn get_huebridge_by_bridge_id(
        conn: &mut SqliteConnection,
        bridge_id: &str,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            let huebridge: HueBridge = huebridges::table
                .filter(huebridges::id.eq(bridge_id))
                .first(conn)
                .expect("Error getting huebridge");

            Ok(huebridge)
        })
    }

    pub fn get_huebridges(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        conn.transaction(|conn| {
            let huebridges: Vec<HueBridge> = huebridges::table
                .load::<HueBridge>(conn)
                .expect("Error getting huebridges");

            Ok(huebridges)
        })
    }

    pub fn get_huebridges_by_user_id(
        conn: &mut SqliteConnection,
        user_id: i32,
    ) -> Result<Vec<HueBridge>, diesel::result::Error> {
        conn.transaction(|conn| {
            let huebridges: Vec<HueBridge> = huebridges::table
                .filter(huebridges::user_settings_id.eq(user_id))
                .load::<HueBridge>(conn)
                .expect("Error getting huebridges");

            Ok(huebridges)
        })
    }

    pub fn update<'a>(
        &self,
        conn: &mut SqliteConnection,
        update_huebridge: &UpdateHueBridge<'a>,
    ) -> Result<HueBridge, diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::update(self)
                .set(update_huebridge)
                .execute(conn)
                .expect("Error updating huebridge");

            let updated_huebridge: HueBridge = huebridges::table
                .find(self._id)
                .first(conn)
                .expect("Error updating huebridge");

            Ok(updated_huebridge)
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::delete(self)
                .execute(conn)
                .expect("Error deleting huebridge");

            Ok(())
        })
    }
}
