package store

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/jackc/pgx/v4/stdlib"
)


func TestCreateAndGetUser(t *testing.T) {
  db := SetupTestDB(t)
  defer db.Close()

  store := NewPostgresUserStore(db)

  var good_password password
  good_password.Set("secure123")
  var still_good_password password
  still_good_password.Set("notSoSecure123")

  tests := []struct {
    name string
    user *User
    wantErr bool
  }{
    {
      name: "valid user",
      user: &User{
        Email: "testemail@gmail.com",
        PasswordHash: good_password,
      },
      wantErr: false,
    },
    {
      name: "no password",
      user: &User{
        Email: "goodemail@gmail.com",
      },
      wantErr: true,
    },
    /*
    {
      name: "no email",
      user: &User{
        PasswordHash: good_password,
      },
      wantErr: true,
    },
    */
    {
      name: "different email, same password",
      user: &User{
        Email: "goodemail@gmail.com",
        PasswordHash: good_password,
      },
      wantErr: false,
    },
    {
      name: "same email, different password",
      user: &User{
        Email: "testemail@gmail.com",
        PasswordHash: still_good_password,
      },
      wantErr: true,
    },
    {
      name: "same email, same password",
      user: &User{
        Email: "testemail@gmail.com",
        PasswordHash: good_password,
      },
      wantErr: true,
    },
  }

  for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
      err := store.CreateUser(tt.user)
      if tt.wantErr {
        assert.Error(t, err)
        return
      }

      u, err := store.GetUserByEmail(tt.user.Email)
      if tt.wantErr {
        assert.NoError(t, err)
        assert.Nil(t, u)
      }

      require.NotNil(t, u)
      require.NoError(t, err)

      assert.Equal(t, tt.user.Email, u.Email)
    })
  }
}

func TestModifiyUser(t *testing.T) {
  db := SetupTestDB(t)
  defer db.Close()

  store := NewPostgresUserStore(db)

  var pwd1 password
  pwd1.Set("pwd1")
  var pwd2 password
  pwd2.Set("pwd2")

  var modified_pwd1 password
  modified_pwd1.Set("m_pwd1")
  var modified_pwd2 password
  modified_pwd2.Set("m_pwd2")

  tests := []struct {
    name string
    user *User
    wantErrCrea bool
    modified_user *User
    wantErrMod bool
  }{
    {
      name: "valid user, modifiy email",
      user: &User{
        Email: "email1@gmail.com",
        PasswordHash: pwd1,
      },
      wantErrCrea: false,
      modified_user: &User{
        Email: "email1_modified@gmail.com",
      },
      wantErrMod: false,
    },
    {
      name: "valid user, modifiy to email that used to exists",
      user: &User{
        Email: "email3@gmail.com",
        PasswordHash: pwd1,
      },
      wantErrCrea: false,
      modified_user: &User{
        Email: "email1@gmail.com",
      },
      wantErrMod: false,
    },
    {
      name: "valid user, modifiy to email that exists",
      user: &User{
        Email: "email4@gmail.com",
        PasswordHash: pwd1,
      },
      wantErrCrea: false,
      modified_user: &User{
        Email: "email1_modified@gmail.com",
      },
      wantErrMod: true,
    },
  }

  for _, tt := range tests {
    t.Run(tt.name, func(t *testing.T) {
      // Creating user
      err := store.CreateUser(tt.user)
      if tt.wantErrCrea {
        assert.Error(t, err)
      }

      // Getting it back
      u, err := store.GetUserByEmail(tt.user.Email)
      if tt.wantErrCrea {
        assert.NoError(t, err)
        assert.Nil(t, u)
        return
      }

      require.NotNil(t, u)
      require.NoError(t, err)

      assert.Equal(t, tt.user.Email, u.Email)

      // Modifing the user
      u.Email = tt.modified_user.Email
      err = store.UpdateUser(u)
      if tt.wantErrMod {
        assert.Error(t, err)
        return
      }
      require.NoError(t, err)

      // Getting it back
      m, err := store.GetUserByEmail(tt.modified_user.Email)
      if tt.wantErrCrea {
        assert.NoError(t, err)
        assert.Nil(t, m)
        return
      }

      require.NotNil(t, m)
      require.NoError(t, err)
      assert.NotEqual(t, tt.user.Email, m.Email)
      assert.Equal(t, m.Email, u.Email)
    })
  }
}
func IntPtr(i int) *int {
  return &i
}

func FloatPtr(i float64) *float64 {
  return &i
}
