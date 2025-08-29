// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        public_id -> Int4,
        #[max_length = 128]
        name -> Varchar,
        #[max_length = 64]
        email -> Varchar,
        #[max_length = 32]
        document -> Varchar,
        #[max_length = 128]
        password -> Varchar,
        birthdate -> Date,
        #[max_length = 16]
        login_type -> Varchar,
        #[max_length = 16]
        user_type -> Varchar,
        is_active -> Bool,
        create_date -> Timestamp,
        update_date -> Timestamp,
        deletion_date -> Nullable<Timestamp>,
    }
}
