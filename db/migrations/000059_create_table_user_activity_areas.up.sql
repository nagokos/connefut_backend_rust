CREATE TABLE IF NOT EXISTS "user_activity_areas"(
  "id" BIGSERIAL UNIQUE, 
  "user_id" BIGINT NOT NULL,
  "prefecture_id" BIGINT NOT NULL,
  FOREIGN KEY("user_id") 
    REFERENCES "users"("id")
    ON DELETE CASCADE,
  FOREIGN KEY("prefecture_id") 
    REFERENCES "prefectures"("id")
    ON DELETE CASCADE,
  PRIMARY KEY("id"),
  UNIQUE("user_id", "prefecture_id")
);
CREATE INDEX ON "user_activity_areas"("user_id");
CREATE INDEX ON "user_activity_areas"("prefecture_id");

