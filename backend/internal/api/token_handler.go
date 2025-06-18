package api

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/mptwd/stratmaker/internal/store"
	"github.com/mptwd/stratmaker/internal/tokens"
	"github.com/mptwd/stratmaker/internal/utils"
)

type TokenHandler struct {
  tokenStore store.TokenStore
  userStore store.UserStore
  logger *log.Logger
}

type createTokenRequest struct {
  Email string `json:"email"` 
  Password string `json:"password"`
}

func NewTokenHandler(tokenStore store.TokenStore, userStore store.UserStore, logger *log.Logger) *TokenHandler {
  return &TokenHandler {
    tokenStore: tokenStore,
    userStore: userStore,
    logger: logger,
  }
}

func (h *TokenHandler) HandleCreateToken(w http.ResponseWriter, r *http.Request) {
  var req createTokenRequest
  err := json.NewDecoder(r.Body).Decode(&req)

  if err != nil {
    h.logger.Printf("ERROR: createTokenRequest: %v", err)
    utils.WriteJSON(w, http.StatusBadRequest, utils.Envelope{"error": "invalid request payload"})
    return
  }

  // lets get the user
  user, err := h.userStore.GetUserByEmail(req.Email)
  if err != nil || user == nil {
    h.logger.Printf("ERROR: GetUserByEmail: %v", err)
    utils.WriteJSON(w, http.StatusInternalServerError, utils.Envelope{"error": "internal server error"})
    return
  }

  passwordsDoMatch, err := user.PasswordHash.Matches(req.Password)
  if err != nil {
    h.logger.Printf("ERROR: PasswordHash.Matches %v", err)
    utils.WriteJSON(w, http.StatusInternalServerError, utils.Envelope{"error": "internal server error"})
    return
  }

  if !passwordsDoMatch {
    utils.WriteJSON(w, http.StatusUnauthorized, utils.Envelope{"error": "invalid credentials"})
    return
  }

  token, err := h.tokenStore.CreateNewToken(user.ID, 24*time.Hour, tokens.ScopeAuth)
  if err != nil {
    h.logger.Printf("ERROR: Creating Token %v", err)
    utils.WriteJSON(w, http.StatusInternalServerError, utils.Envelope{"error": "internal server error"})
    return
  }

  utils.WriteJSON(w, http.StatusCreated, utils.Envelope{"auth_token": token})
}
