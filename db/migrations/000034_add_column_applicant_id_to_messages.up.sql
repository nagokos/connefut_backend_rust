ALTER TABLE "messages"
  ADD COLUMN "applicant_id" BIGINT NOT NULL,
  ADD FOREIGN KEY("applicant_id") 
    REFERENCES "applicants"("id") 
    ON DELETE CASCADE;