package api

import (
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"regexp"

	"github.com/mptwd/stratmaker/internal/store"
	"github.com/mptwd/stratmaker/internal/utils"
)

type registerUserRequest struct {
  Email string `json:"email"`
  Password string `json:"password"`
}

type UserHandler struct {
  userStore store.UserStore
  logger *log.Logger
}

func NewUserHandler(userStore store.UserStore, logger *log.Logger) *UserHandler {
  return &UserHandler{
    userStore: userStore,
    logger: logger,
  }
}

func (h *UserHandler) validateRegisterRequest(req *registerUserRequest) error {
  if req.Email == "" {
    return errors.New("email is required")
  }

  if len(req.Email) > 255 {
    return errors.New("email cannot be greater than 255 characters")
  }


  emailRegex := regexp.MustCompile(`^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`)
  if !emailRegex.MatchString(req.Email) {
    return errors.New("invalid email format")
  }

  // we can have password validation
  if req.Password == "" {
    return errors.New("password is required")
  }

  return nil
}


func (h *UserHandler) HandleRegisterUser(w http.ResponseWriter, r *http.Request) {
  var req registerUserRequest

  err := json.NewDecoder(r.Body).Decode(&req)
  if err != nil {
    h.logger.Printf("ERROR: decoding register request: %v", err)
    utils.WriteJSON(w, http.StatusBadRequest, utils.Envelope{"error": "invalid request payload"})
    return
  }

  err = h.validateRegisterRequest(&req)
  if err != nil {
    utils.WriteJSON(w, http.StatusBadRequest, utils.Envelope{"error": err.Error()})
    return
  }

  user := &store.User {
    Email: req.Email,
  }

  // how do we deal with their password
  err = user.PasswordHash.Set(req.Password)
  if err != nil {
    h.logger.Printf("ERROR: hashing password %v", err)
    utils.WriteJSON(w, http.StatusInternalServerError, utils.Envelope{"error": "internal server error"})
    return
  }

  err = h.userStore.CreateUser(user)
  if err != nil {
    h.logger.Printf("ERROR: registering user %v", err)
    utils.WriteJSON(w, http.StatusInternalServerError, utils.Envelope{"error": "internal server error"})
    return
  }

  utils.WriteJSON(w, http.StatusCreated, utils.Envelope{"user": user})

}




