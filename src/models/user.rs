use crate::{
    controllers::utils::{format_document, password_hash, random_public_id},
    models::error::ApiError,
    schema::users,
};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{
    ExpressionMethods, QueryDsl,
    prelude::{AsChangeset, Insertable, Queryable},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::ValidateEmail;

#[derive(Queryable, Insertable, AsChangeset, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub public_id: i32,
    pub name: String,
    pub email: String,
    pub document: String,
    pub password: String,
    pub birthdate: NaiveDate,
    pub login_type: String,
    pub user_type: String,
    pub is_active: bool,
    pub create_date: NaiveDateTime,
    pub update_date: NaiveDateTime,
    pub deletion_date: Option<NaiveDateTime>,
}

pub struct UserAuthInfo {
    pub id: Uuid,
    pub public_id: i32,
    pub email: String,
    pub user_type: String,
}

impl From<User> for UserAuthInfo {
    fn from(input: User) -> Self {
        Self {
            id: input.id,
            public_id: input.public_id,
            email: input.email,
            user_type: input.user_type,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
    pub document: String,
    pub password: String,
    pub birthdate: String,
    pub login_type: String,
    pub user_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateUser {
    pub name: String,
    pub email: String,
    pub document: String,
    pub birthdate: NaiveDate,
}

impl RegisterUser {
    pub fn validate_fields(self: &Self) -> bool {
        if self.document.trim().is_empty()
            || self.name.trim().is_empty()
            || self.email.trim().is_empty()
            || self.password.trim().is_empty()
            || self.birthdate.to_string().trim().is_empty()
            || self.login_type.trim().is_empty()
            || self.user_type.trim().is_empty()
            || !self.email.validate_email()
        {
            return false;
        }
        true
    }

    pub fn parse_fields(self: &mut Self) -> Result<(), String> {
        self.email = self.email.trim().to_string();
        self.name = self.name.trim().to_string();
        self.password = password_hash(self.password.trim());
        self.user_type = self.user_type.trim().to_string();
        self.login_type = self.login_type.trim().to_string();
        self.document = match format_document(&self.document) {
            Ok(document) => document,
            Err(e) => return Err(e),
        };
        self.birthdate = self.birthdate.trim().to_string();

        Ok(())
    }
}

impl From<RegisterUser> for User {
    fn from(input: RegisterUser) -> Self {
        let birthdate = NaiveDate::from_str(&input.birthdate)
            .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2001, 1, 1).unwrap());

        Self {
            id: Uuid::new_v4(),
            public_id: random_public_id(),
            name: input.name,
            email: input.email,
            document: input.document,
            password: input.password,
            birthdate,
            login_type: input.login_type,
            user_type: input.user_type,
            is_active: true,
            create_date: chrono::Utc::now().naive_utc(),
            update_date: chrono::Utc::now().naive_utc(),
            deletion_date: None,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

impl LoginUser {
    pub fn validate_fields(self: &Self) -> Result<(), String> {
        if self.email.trim().is_empty() || self.password.trim().is_empty() {
            return Err(ApiError::InvalidData.to_string());
        }
        if !self.email.validate_email() {
            return Err(ApiError::InvalidEmail.to_string());
        }
        Ok(())
    }

    pub fn parse_fields(self: &mut Self) {
        self.email = self.email.trim().to_string();
        self.password = self.password.trim().to_string();
    }
}

pub async fn register_user(conn: &mut AsyncPgConnection, user: &User) -> Result<(), String> {
    use crate::schema::users::dsl::*;

    match diesel::insert_into(users).values(user).execute(conn).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn find_user_by_email(conn: &mut AsyncPgConnection, param: &str) -> Result<User, String> {
    use crate::schema::users::dsl::*;

    match users.filter(email.eq(param)).get_result(conn).await {
        Ok(user) => Ok(user),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn find_user_by_document(
    conn: &mut AsyncPgConnection,
    param: &str,
) -> Result<User, String> {
    use crate::schema::users::dsl::*;

    match users.filter(document.eq(param)).get_result(conn).await {
        Ok(user) => Ok(user),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn find_user_by_id(conn: &mut AsyncPgConnection, param: &Uuid) -> Result<User, String> {
    use crate::schema::users::dsl::*;

    match users.filter(id.eq(param)).get_result(conn).await {
        Ok(user) => Ok(user),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn find_user_by_public_id(
    conn: &mut AsyncPgConnection,
    param: i32,
) -> Result<User, String> {
    use crate::schema::users::dsl::*;

    match users.filter(public_id.eq(param)).get_result(conn).await {
        Ok(user) => Ok(user),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn update_user_data(
    conn: &mut AsyncPgConnection,
    id_param: &Uuid,
    data: &UpdateUser,
) -> Result<(), ApiError> {
    use crate::schema::users::dsl::*;

    match diesel::update(users)
        .filter(id.eq(id_param))
        .set((
            name.eq(&data.name),
            email.eq(&data.email),
            document.eq(&data.document),
            birthdate.eq(data.birthdate),
        ))
        .execute(conn)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(ApiError::Database(e.to_string())),
    }
}
