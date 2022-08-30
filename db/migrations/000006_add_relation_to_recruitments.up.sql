ALTER TABLE "recruitments"
  ADD FOREIGN KEY("competition_id")
    REFERENCES "competitions"("id")
    ON DELETE RESTRICT,
  ADD FOREIGN KEY("prefecture_id")
    REFERENCES "prefectures"("id")
    ON DELETE RESTRICT,
  ADD FOREIGN KEY("user_id")
    REFERENCES "users"("id")
    ON DELETE CASCADE;
