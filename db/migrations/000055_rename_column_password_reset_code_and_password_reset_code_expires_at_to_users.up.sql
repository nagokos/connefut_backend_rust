ALTER TABLE "users"
  RENAME COLUMN "password_reset_code" TO "password_reset_token";
ALTER TABLE "users"
  RENAME COLUMN "password_reset_code_expires_at" TO "password_reset_token_expires_at";