// @generated automatically by Diesel CLI.

diesel::table! {
    jobs (id) {
        id -> Int4,
        schedule -> Varchar,
        next_run -> Timestamptz,
    }
}
