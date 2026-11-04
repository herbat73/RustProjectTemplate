CREATE TABLE "TodoLists" (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    colour VARCHAR(7) NOT NULL DEFAULT '#FFFFFF'
);

CREATE TABLE "TodoItems" (
    id SERIAL PRIMARY KEY,
    list_id INTEGER NOT NULL REFERENCES "TodoLists" (id) ON DELETE CASCADE,
    title VARCHAR(200) NOT NULL,
    done BOOLEAN NOT NULL DEFAULT FALSE,
    priority INTEGER NOT NULL DEFAULT 0,
    note TEXT
);

CREATE INDEX "IX_TodoItems_list_id" ON "TodoItems" (list_id);
