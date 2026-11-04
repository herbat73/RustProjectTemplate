-- Todo lists belong to a user. Lists created before this migration have no owner and are not visible through the API.
ALTER TABLE "TodoLists" ADD COLUMN user_id INTEGER REFERENCES "users" (id) ON DELETE CASCADE;
CREATE INDEX "IX_TodoLists_user_id" ON "TodoLists" (user_id);
