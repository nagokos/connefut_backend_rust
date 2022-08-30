ALTER TABLE "users" 
  ALTER COLUMN "email_verification_status" DROP DEFAULT,
  ALTER COLUMN "email_verification_status"
    SET DATA TYPE email_verification_status
    USING email_verification_status::varchar::email_verification_status,
  ALTER COLUMN "email_verification_status" SET DEFAULT 'pending';