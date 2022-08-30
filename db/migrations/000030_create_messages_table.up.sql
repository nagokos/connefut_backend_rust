CREATE TABLE IF NOT EXISTS "messages"(
  "id" BIGSERIAL UNIQUE,
  "content" VARCHAR(1000) NOT NULL,
  "room_id" BIGINT NOT NULL,
  "user_id" BIGINT NULL,
  PRIMARY KEY("id"),
  FOREIGN KEY("room_id") 
    REFERENCES "rooms"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("user_id")
    REFERENCES "users"("id")
    ON DELETE SET NULL
);
CREATE INDEX ON "messages"("room_id");
CREATE INDEX ON "messages"("user_id");