/*CREATE TABLE wleditems (
 _id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
 ip VARCHAR NOT NULL,
 name VARCHAR NOT NULL,
 user_settings_id INTEGER REFERENCES usersettings(id) NOT NULL
 );*/
CREATE TABLE "wleditems" (
    "_id" INTEGER NOT NULL,
    "ip" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "user_settings_id" INTEGER NOT NULL,
    FOREIGN KEY("user_settings_id") REFERENCES "usersettings"("id"),
    PRIMARY KEY("_id" AUTOINCREMENT)
);