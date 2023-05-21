#![allow(dead_code)]

use diesel::prelude::*;

use diesel::{Connection, SqliteConnection};

use super::{
    models::{NewWledItem, UpdateWledItem, WledItem},
    schema::wleditems,
};

pub fn create_wleditem<'a>(
    conn: &mut SqliteConnection,
    new_wleditem: &NewWledItem<'a>,
) -> Result<WledItem, diesel::result::Error> {
    conn.transaction(|conn| {
        diesel::insert_into(wleditems::table)
            .values(new_wleditem)
            .execute(conn)
            .expect("Error saving new wleditem");

        let inserted_wleditem: WledItem = wleditems::table
            .order(wleditems::_id.desc())
            .first(conn)
            .expect("Error saving new wleditem");

        Ok(inserted_wleditem)
    })
}

pub fn get_wleditems(conn: &mut SqliteConnection) -> Result<Vec<WledItem>, diesel::result::Error> {
    conn.transaction(|conn| {
        let wleditems: Vec<WledItem> = wleditems::table
            .load::<WledItem>(conn)
            .expect("Error getting wleditems");

        Ok(wleditems)
    })
}

pub fn get_wleditem(conn: &mut SqliteConnection, id: i32) -> Result<WledItem, diesel::result::Error> {
    conn.transaction(|conn| {
        let wleditem: WledItem = wleditems::table
            .find(id)
            .first(conn)
            .expect("Error getting wleditem");

        Ok(wleditem)
    })
}

pub fn get_wleditems_by_user_id(
    conn: &mut SqliteConnection,
    user_id: i32,
) -> Result<Vec<WledItem>, diesel::result::Error> {
    conn.transaction(|conn| {
        let wleditems: Vec<WledItem> = wleditems::table
            .filter(wleditems::user_settings_id.eq(user_id))
            .load::<WledItem>(conn)
            .expect("Error getting wleditems");

        Ok(wleditems)
    })
}

impl WledItem {
    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        update_wleditem: &UpdateWledItem,
    ) -> Result<WledItem, diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::update(self)
                .set(update_wleditem)
                .execute(conn)
                .expect("Error updating wleditem");

            let updated_wleditem: WledItem = wleditems::table
                .find(self._id)
                .first(conn)
                .expect("Error updating wleditem");

            Ok(updated_wleditem)
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
        conn.transaction(|conn| {
            diesel::delete(self)
                .execute(conn)
                .expect("Error deleting wleditem");

            Ok(())
        })
    }
}
