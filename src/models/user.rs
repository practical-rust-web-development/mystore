use chrono::NaiveDateTime;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    #[serde(skip)]
    pub id: i32,
    pub email: String,
    pub company: String,
    #[serde(skip)]
    pub password: String,
    pub created_at: NaiveDateTime
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub company: String,
    pub password: String,
    pub created_at: NaiveDateTime
}

use bcrypt::{hash, DEFAULT_COST};
use diesel::PgConnection;
use chrono::Local;
use crate::errors::MyStoreError;

impl User {
    pub fn create(register_user: RegisterUser, connection: &PgConnection) ->
     Result<User, MyStoreError> {
        use diesel::RunQueryDsl;

        Ok(diesel::insert_into(users::table)
            .values(NewUser {
                email: register_user.email,
                company: register_user.company,
                password: Self::hash_password(register_user.password)?,
                created_at: Local::now().naive_local()
            })
            .get_result(connection)?)
    }

    pub fn hash_password(plain: String) -> Result<String, MyStoreError> {
        Ok(hash(plain, DEFAULT_COST)?)
    }
}

#[derive(Deserialize)]
pub struct RegisterUser {
    pub email: String,
    pub company: String,
    pub password: String,
    pub password_confirmation: String
}

impl RegisterUser {
    pub fn validates(self) ->
     Result<RegisterUser, MyStoreError> {
         if self.password == self.password_confirmation {
             Ok(self)
         } else {
             Err(
                 MyStoreError::PasswordNotMatch(
                     "Password and Password Confirmation does not match".to_string()
                 )
             )
         }
    }
}

#[derive(Deserialize)]
pub struct AuthUser {
    pub email: String,
    pub password: String
}

impl AuthUser {
    pub fn login(&self, connection: &PgConnection) ->
     Result<User, MyStoreError> {
        use bcrypt::verify;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use crate::schema::users::dsl::email;

        let mut records =
            users::table
                .filter(email.eq(&self.email))
                .load::<User>(connection)?;

        let user =
            records
                .pop()
                .ok_or(MyStoreError::DBError(diesel::result::Error::NotFound))?;

        let verify_password =
            verify(&self.password, &user.password)
                .map_err( |_error| {
                    MyStoreError::WrongPassword(
                        "Wrong password, check again please".to_string()
                    )
                })?;

        if verify_password {
            Ok(user)
        } else {
            Err(MyStoreError::WrongPassword(
                "Wrong password, check again please".to_string()
            ))
        }
    }
}