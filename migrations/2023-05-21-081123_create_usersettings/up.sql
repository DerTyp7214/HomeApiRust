/*CREATE TABLE usersettings (
 id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
 hue_index INTEGER NOT NULL DEFAULT 0,
 user_id INTEGER REFERENCES users(id) NOT NULL
 );*/
CREATE TABLE "usersettings" (
    "id" INTEGER NOT NULL,
    "hue_index" INTEGER NOT NULL DEFAULT 0,
    "user_id" INTEGER NOT NULL,
    FOREIGN KEY("user_id") REFERENCES "users"("id"),
    PRIMARY KEY("id" AUTOINCREMENT)
)