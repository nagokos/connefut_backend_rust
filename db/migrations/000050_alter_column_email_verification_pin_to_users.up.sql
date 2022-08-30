ALTER TABLE "users"
  ALTER COLUMN "email_verification_pin" TYPE INT USING("email_verification_pin"::INTEGER)