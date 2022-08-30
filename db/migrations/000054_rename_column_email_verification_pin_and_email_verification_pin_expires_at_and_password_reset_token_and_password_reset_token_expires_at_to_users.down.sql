ALTER TABLE "users"
  RENAME COLUMN "email_verification_code" to "email_verification_pin";
ALTER TABLE "users"
  RENAME COLUMN "email_verification_code_expires_at" to "email_verification_pin_expires_at";
ALTER TABLE "users"
  RENAME COLUMN "password_reset_code" to "password_reset_token";
ALTER TABLE "users"
  RENAME COLUMN "password_reset_code_expires_at" to "password_reset_token_expires_at";
