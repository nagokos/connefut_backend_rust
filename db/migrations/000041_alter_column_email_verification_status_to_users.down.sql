ALTER TABLE "users" 
  ALTER COLUMN "email_verification_status" DROP DEFAULT,
  ALTER COLUMN "email_verification_status" TYPE VARCHAR,
  ALTER COLUMN "email_verification_status" SET DEFAULT 'pending';