CREATE TABLE files (
  path text PRIMARY KEY,
  size bigint NOT NULL,
  content_type text NOT NULL,
  last_modified bigint NOT NULL
);
