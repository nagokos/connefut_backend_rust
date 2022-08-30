CREATE TABLE IF NOT EXISTS "user_play_sports"(
  "id" BIGSERIAL UNIQUE, 
  "user_id" BIGINT NOT NULL,
  "sport_id" BIGINT NOT NULL,
  FOREIGN KEY("user_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("sport_id")
    REFERENCES "sports"("id")
    ON DELETE CASCADE,
  PRIMARY KEY("id"),
  UNIQUE("user_id", "sport_id")
);
CREATE INDEX ON "user_play_sports"("user_id");
CREATE INDEX ON "user_play_sports"("sport_id");

