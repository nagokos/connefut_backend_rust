ALTER TABLE "users"
  RENAME COLUMN "email_verification_pin" TO "email_verification_token";
ALTER TABLE "users"
  RENAME COLUMN "email_verification_pin_expires_at" TO "email_verification_token_expires_at";