ALTER TABLE "recruitments"
  ADD COLUMN "sport_id" BIGINT NOT NULL;
ALTER TABLE "recruitments"
  ADD FOREIGN KEY("sport_id")
    REFERENCES "sports"("id")
    ON DELETE RESTRICT;
CREATE INDEX ON "recruitments"("sport_id");