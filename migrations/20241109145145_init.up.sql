-- Add up migration script here
create TABLE leagues(
  id serial not null PRIMARY KEY,
  name text not null,
  roleid text,
  channelid text
);

CREATE TABLE races(
  name text not null,
  number int not null,
  time timestamptz not null,
  league_id serial not null,
  FOREIGN KEY(league_id) REFERENCES leagues(id)
);

