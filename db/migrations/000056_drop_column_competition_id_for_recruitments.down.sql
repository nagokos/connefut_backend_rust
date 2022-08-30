ALTER TABLE "recruitments"
  ADD COLUMN "competition_id" BIGINT NOT NULL;
ALTER TABLE "recruitments"
  ADD FOREIGN KEY("competition_id")
    REFERENCES "competitions"("id")
    ON DELETE RESTRICT;
CREATE INDEX ON "recruitments"("competition_id");