ALTER TABLE "recruitments" 
  ALTER COLUMN "status" DROP DEFAULT,
  ALTER COLUMN "status"
    SET DATA TYPE recruitment_status
    USING status::varchar::recruitment_status,
  ALTER COLUMN "status" SET DEFAULT 'draft';