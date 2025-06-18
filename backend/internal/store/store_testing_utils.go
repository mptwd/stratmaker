package store

import (
	"database/sql"
	"testing"
)

func SetupTestDB(t *testing.T) *sql.DB {
  db, err := sql.Open("pgx", "host=localhost user=postgres password=postgres dbname=postgres port=5434 sslmode=disable")
  if err != nil {
    t.Fatalf("opening test db: %v", err)
  }

  // run the migration for our test db
  err = Migrate(db, "../../migrations/")
  if err != nil {
    t.Fatalf("migrating test db error: %v", err)
  }

  //_, err = db.Exec(`TRUNCATE users, tokens CASCADE`) // whipe the db every time we start the test
  _, err = db.Exec(`TRUNCATE users`) // whipe the db every time we start the test
  if err != nil {
    t.Fatalf("truncating test db error: %v", err)
  }

  return db
}

