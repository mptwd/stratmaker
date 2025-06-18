package main

import (
	"fmt"
	"net/http"
	"time"
  "flag"

	"github.com/mptwd/stratmaker/internal/app"
	"github.com/mptwd/stratmaker/internal/routes"
)

func main() {
  var port int
  flag.IntVar(&port, "port", 8080, "go backend server port")
  flag.Parse()
  app, err := app.NewApplication()
  if err != nil {
    panic(err)
  }
  defer app.DB.Close() // to tell go to run this function once the program ends


  r := routes.SetupRoutes(app)
  server := &http.Server{
    Addr: fmt.Sprintf(":%d", port),
    Handler: r,
    IdleTimeout: time.Minute,
    ReadTimeout: 10 * time.Second,
    WriteTimeout: 30 * time.Second,
  }

  app.Logger.Printf("We are running on port %d", port)

  err = server.ListenAndServe()
  if err != nil {
    app.Logger.Fatal(err)
  }
}
