CREATE TABLE files (
  path text PRIMARY KEY,
  size bigint NOT NULL,
  content_type text NOT NULL,
  last_modified bigint NOT NULL
);

CREATE TABLE tmp (
  path text NOT NULL,
  size bigint NOT NULL,
  content_type text NOT NULL,
  last_modified bigint NOT NULL
);

CREATE TABLE users (name text PRIMARY KEY, password text NOT NULL);
