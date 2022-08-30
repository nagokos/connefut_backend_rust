ALTER TABLE "users"
  RENAME COLUMN "email_verification_pin" to "email_verification_code";
ALTER TABLE "users"
  RENAME COLUMN "email_verification_pin_expires_at" to "email_verification_code_expires_at";
ALTER TABLE "users"
  RENAME COLUMN "password_reset_token" to "password_reset_code";
ALTER TABLE "users"
  RENAME COLUMN "password_reset_token_expires_at" to "password_reset_code_expires_at";
