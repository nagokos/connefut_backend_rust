CREATE TABLE IF NOT EXISTS "room_read_managements"(
  "id" BIGSERIAL UNIQUE,
  "last_read_at" TIMESTAMP WITH TIME ZONE NULL,
  "room_id" BIGINT NOT NULL,
  "user_id" BIGINT NULL,
  PRIMARY KEY("id"),
  FOREIGN KEY("room_id") 
    REFERENCES "rooms"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("user_id")
    REFERENCES "users"("id")
    ON DELETE SET NULL,
  UNIQUE("room_id", "user_id")
);
CREATE INDEX ON "room_read_managements"("room_id");
CREATE INDEX ON "room_read_managements"("user_id");