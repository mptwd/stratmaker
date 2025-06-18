package app

import (
	"database/sql"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/mptwd/stratmaker/internal/api"
	"github.com/mptwd/stratmaker/internal/middleware"
	"github.com/mptwd/stratmaker/internal/store"
	"github.com/mptwd/stratmaker/migrations"
)

type Application struct {
  Logger *log.Logger
  //WorkoutHandler *api.WorkoutHandler
  UserHandler *api.UserHandler
  TokenHandler *api.TokenHandler
  Middleware middleware.UserMiddleware
  DB *sql.DB
}
// logger is to avoid using print statements

func NewApplication() (*Application, error) {
  pgDB, err := store.Open()
  if err != nil {
    return nil, err
  }

  err = store.MigrateFS(pgDB, migrations.FS, ".")
  if err != nil {
    panic(err)
  }

  logger := log.New(os.Stdout, "", log.Ldate|log.Ltime)

  // our stores will go here
  //workoutStore := store.NewPostgresWorkoutStore(pgDB)
  userStore := store.NewPostgresUserStore(pgDB)
  tokenStore := store.NewPostgresTokenStore(pgDB)

  // our handlers will go here
  //workoutHandler := api.NewWorkoutHandler(workoutStore, logger)
  userHandler := api.NewUserHandler(userStore, logger)
  tokenHandler := api.NewTokenHandler(tokenStore, userStore, logger)
  middlewareHandler := middleware.UserMiddleware{UserStore: userStore}

  app := &Application{
    Logger: logger,
    //WorkoutHandler: workoutHandler,
    UserHandler: userHandler,
    TokenHandler: tokenHandler,
    Middleware: middlewareHandler,
    DB: pgDB,
  }

  return app, nil
}

func (a *Application) HealthCheck(w http.ResponseWriter, r *http.Request) {
  fmt.Fprint(w, "Status is available\n")
}
