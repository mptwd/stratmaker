package store

import (
	"testing"
	"time"

	"github.com/mptwd/stratmaker/internal/tokens"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/jackc/pgx/v4/stdlib"
)

/*
  Insert(token *tokens.Token) error
  CreateNewToken(userID int, ttl time.Duration, scope string) (*tokens.Token, error)
  DeleteAllTokensForUser(userID int, scope string) error
*/

func TestCreateNewTokenAndInsert(t *testing.T) {
  db := SetupTestDB(t)
  defer db.Close()

  us := NewPostgresUserStore(db)
  ts := NewPostgresTokenStore(db)

  var pwd1 password
  pwd1.Set("secure123")
  var pwd2 password
  pwd2.Set("notSoSecure123")

  tests := []struct {
    name string
    user *User
    numTokens int
    tokenDuration time.Duration
    tokenScope string
    wantErr bool
  }{
    {
      name: "valid user 1",
      user: &User{
        Email: "email1@gmail.com",
        PasswordHash: pwd1,
      },
      numTokens: 1,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "valid user 2",
      user: &User{
        Email: "email2@gmail.com",
        PasswordHash: pwd2,
      },
      numTokens: 1,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "valid user 3 with 2 tokens",
      user: &User{
        Email: "email3@gmail.com",
        PasswordHash: pwd2,
      },
      numTokens: 2,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "valid user 4 with 10 tokens",
      user: &User{
        Email: "email4@gmail.com",
        PasswordHash: pwd2,
      },
      numTokens: 10,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "valid user 4 with 10 tokens",
      user: &User{
        Email: "email5@gmail.com",
        PasswordHash: pwd1,
      },
      numTokens: 1,
      tokenDuration: time.Second,
      tokenScope: tokens.ScopeAuth,
      wantErr: true,
    },
  }

  for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
      // Creating the user
      err := us.CreateUser(tt.user)
      require.NoError(t, err)

      // Getting user back and checking if all is good
      u, err := us.GetUserByEmail(tt.user.Email)
      require.NotNil(t, u)
      require.NoError(t, err)
      require.Equal(t, tt.user.Email, u.Email)

      // Creating a Token for the user
      for range tt.numTokens {
        token, err := ts.CreateNewToken(u.ID, tt.tokenDuration, tt.tokenScope)
        require.NoError(t, err)
        require.NotNil(t, token)
      }
      if (tt.wantErr) {
        time.Sleep(2 * tt.tokenDuration)
      }
      
      // Getting it back to see if all is good
      var count int = 0
      err = db.QueryRow(`SELECT COUNT(*) FROM tokens WHERE user_id = $1 AND scope = $2`,
        u.ID,
        tokens.ScopeAuth).Scan(&count)
      require.NoError(t, err)
      assert.Equal(t, tt.numTokens, count)

      var expiry time.Time
      err = db.QueryRow(`SELECT expiry FROM tokens WHERE user_id = $1 AND scope = $2`,
        u.ID,
        tokens.ScopeAuth).Scan(&expiry)
      require.NoError(t, err)
      if tt.wantErr {
        assert.Greater(t, time.Now(), expiry)
        return
      }
      assert.Less(t, time.Now(), expiry)
    })
  }
}

func TestDeleteAllTokensForUser(t *testing.T) {
  db := SetupTestDB(t)
  defer db.Close()

  us := NewPostgresUserStore(db)
  ts := NewPostgresTokenStore(db)

  var pwd1 password
  pwd1.Set("pwd1")
  var pwd2 password
  pwd2.Set("pwd2")

  tests := []struct {
    name string
    user *User
    numTokens int
    tokenDuration time.Duration
    tokenScope string
    wantErr bool
  }{
    {
      name: "1 token",
      user: &User{
        Email: "email1@gmail.com",
        PasswordHash: pwd1,
      },
      numTokens: 1,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "2 tokens",
      user: &User{
        Email: "email2@gmail.com",
        PasswordHash: pwd2,
      },
      numTokens: 2,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "10 tokens",
      user: &User{
        Email: "email3@gmail.com",
        PasswordHash: pwd2,
      },
      numTokens: 10,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
    {
      name: "no tokens",
      user: &User{
        Email: "email4@gmail.com",
        PasswordHash: pwd2,
      },
      numTokens: 0,
      tokenDuration: time.Hour,
      tokenScope: tokens.ScopeAuth,
      wantErr: false,
    },
  }

  for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
      // Creating the user
      err := us.CreateUser(tt.user)
      require.NoError(t, err)

      // Getting user back and checking if all is good
      u, err := us.GetUserByEmail(tt.user.Email)
      require.NotNil(t, u)
      require.NoError(t, err)
      require.Equal(t, tt.user.Email, u.Email)

      // Creating a Token for the user
      for range tt.numTokens {
        token, err := ts.CreateNewToken(u.ID, tt.tokenDuration, tt.tokenScope)
        require.NoError(t, err)
        require.NotNil(t, token)
      }
      
      // Getting it back to see if all is good
      var count int = 0
      err = db.QueryRow(`SELECT COUNT(*) FROM tokens WHERE user_id = $1 AND scope = $2`,
        u.ID,
        tokens.ScopeAuth).Scan(&count)
      require.NoError(t, err)
      assert.Equal(t, tt.numTokens, count)

      err = ts.DeleteAllTokensForUser(u.ID, tt.tokenScope)
      assert.NoError(t, err)
    })
  }
}
