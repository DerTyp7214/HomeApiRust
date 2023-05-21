/*CREATE TABLE huebridges (
 _id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
 id VARCHAR NOT NULL,
 ip VARCHAR NOT NULL,
 user VARCHAR NOT NULL,
 user_settings_id INTEGER REFERENCES usersettings(id) NOT NULL
 );*/
CREATE TABLE "huebridges" (
    "_id" INTEGER NOT NULL,
    "id" TEXT NOT NULL,
    "ip" TEXT NOT NULL,
    "user" TEXT NOT NULL,
    "user_settings_id" INTEGER NOT NULL,
    FOREIGN KEY("user_settings_id") REFERENCES "usersettings"("id"),
    PRIMARY KEY("_id" AUTOINCREMENT)
);