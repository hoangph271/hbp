CREATE TABLE tbl_posts (
  id TEXT NOT NULL PRIMARY KEY,
  title VARCHAR NOT NULL,
  body TEXT NOT NULL,
  published BOOLEAN NOT NULL DEFAULT 0
)
