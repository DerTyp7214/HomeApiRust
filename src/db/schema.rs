// @generated automatically by Diesel CLI.

diesel::table! {
    huebridges (_id) {
        _id -> Integer,
        id -> Integer,
        ip -> Text,
        user -> Text,
        user_settings_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        hashed_password -> Text,
    }
}

diesel::table! {
    usersettings (id) {
        id -> Integer,
        hue_index -> Integer,
        user_id -> Integer,
    }
}

diesel::table! {
    wleditems (_id) {
        _id -> Integer,
        ip -> Text,
        name -> Text,
        user_settings_id -> Integer,
    }
}

diesel::joinable!(huebridges -> usersettings (user_settings_id));
diesel::joinable!(usersettings -> users (user_id));
diesel::joinable!(wleditems -> usersettings (user_settings_id));

diesel::allow_tables_to_appear_in_same_query!(
    huebridges,
    users,
    usersettings,
    wleditems,
);
