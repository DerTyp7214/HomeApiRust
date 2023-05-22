#![allow(dead_code)]

use diesel::prelude::*;

use diesel::{Connection, SqliteConnection};

use super::{
    models::{NewWledItem, UpdateWledItem, WledItem},
    schema::wleditems,
};

impl WledItem {
    pub fn create_wleditem<'a>(
        conn: &mut SqliteConnection,
        new_wleditem: &NewWledItem<'a>,
    ) -> Result<WledItem, diesel::result::Error> {
        conn.transaction(|conn| {
            let response = diesel::insert_into(wleditems::table)
                .values(new_wleditem)
                .execute(conn);

            if response.is_err() {
                return Err(response.unwrap_err());
            }

            let inserted_wleditem = wleditems::table.order(wleditems::_id.desc()).first(conn);

            if inserted_wleditem.is_err() {
                return inserted_wleditem;
            }

            Ok(inserted_wleditem.unwrap())
        })
    }

    pub fn get_wleditems(
        conn: &mut SqliteConnection,
    ) -> Result<Vec<WledItem>, diesel::result::Error> {
        conn.transaction(|conn| wleditems::table.load::<WledItem>(conn))
    }

    pub fn get_wleditem(
        conn: &mut SqliteConnection,
        id: i32,
    ) -> Result<WledItem, diesel::result::Error> {
        conn.transaction(|conn| wleditems::table.find(id).first(conn))
    }

    pub fn get_wleditems_by_user_id(
        conn: &mut SqliteConnection,
        user_id: i32,
    ) -> Result<Vec<WledItem>, diesel::result::Error> {
        conn.transaction(|conn| {
            wleditems::table
                .filter(wleditems::user_settings_id.eq(user_id))
                .load::<WledItem>(conn)
        })
    }

    pub fn update(
        &self,
        conn: &mut SqliteConnection,
        update_wleditem: &UpdateWledItem,
    ) -> Result<WledItem, diesel::result::Error> {
        conn.transaction(|conn| {
            let result = diesel::update(self).set(update_wleditem).execute(conn);

            if result.is_err() {
                return Err(result.unwrap_err());
            }

            let updated_wleditem = wleditems::table.find(self._id).first(conn);

            if updated_wleditem.is_err() {
                return updated_wleditem;
            }

            Ok(updated_wleditem.unwrap())
        })
    }

    pub fn delete(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        conn.transaction(|conn| diesel::delete(self).execute(conn))
    }
}
