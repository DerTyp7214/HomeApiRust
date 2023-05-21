/*CREATE TABLE users (
 id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
 username VARCHAR NOT NULL,
 email VARCHAR NOT NULL UNIQUE,
 hashed_password VARCHAR NOT NULL
 );*/
CREATE TABLE "users" (
    "id" INTEGER NOT NULL,
    "username" TEXT NOT NULL,
    "email" TEXT NOT NULL UNIQUE,
    "hashed_password" TEXT NOT NULL,
    PRIMARY KEY("id" AUTOINCREMENT)
);