CREATE TABLE tbl_users (
  id TEXT NOT NULL PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  hashed_password TEXT NOT NULL,
  title TEXT
)
