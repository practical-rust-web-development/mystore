table! {
    use diesel_full_text_search::TsVector;
    use diesel::sql_types::Int4;
    use diesel::sql_types::VarChar;
    use diesel::sql_types::Float8;
    use diesel::sql_types::Nullable;
    products (id) {
        id -> Int4,
        name -> VarChar,
        stock -> Float8,
        price -> Nullable<Int4>,
        description -> Nullable<VarChar>,
        text_searchable_product_col -> TsVector,
        product_rank -> Nullable<Float8>,
        user_id -> Int4,
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

joinable!(products -> users (user_id));

allow_tables_to_appear_in_same_query!(
    products,
    users,
);
