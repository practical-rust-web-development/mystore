table! {
    products (id) {
        id -> Int4,
        name -> Varchar,
        stock -> Float8,
        price -> Nullable<Int4>,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        company -> Varchar,
        password -> Varchar,
        created_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    products,
    users,
);
