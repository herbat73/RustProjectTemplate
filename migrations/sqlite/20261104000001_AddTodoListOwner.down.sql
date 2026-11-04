-- Add down migration script here
DROP INDEX "IX_TodoLists_user_id";
ALTER TABLE "TodoLists" DROP COLUMN user_id;
