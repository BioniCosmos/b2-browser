CREATE TYPE entry AS ENUM ('file', 'directory');
CREATE TABLE files (
  path varchar PRIMARY KEY,
  type entry NOT NULL,
  name varchar NOT NULL,
  parent_path varchar REFERENCES files,
  size bigint NOT NULL,
  content_type varchar NOT NULL,
  last_modified bigint NOT NULL
);
