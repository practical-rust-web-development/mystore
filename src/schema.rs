table! {
    prices (id) {
        id -> Int4,
        name -> Varchar,
        user_id -> Int4,
    }
}

table! {
    prices_products (id) {
        id -> Int4,
        price_id -> Int4,
        product_id -> Int4,
        user_id -> Int4,
        amount -> Nullable<Int4>,
    }
}

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
        cost -> Nullable<Int4>,
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

joinable!(prices -> users (user_id));
joinable!(prices_products -> prices (price_id));
joinable!(prices_products -> products (product_id));
joinable!(prices_products -> users (user_id));
joinable!(products -> users (user_id));

allow_tables_to_appear_in_same_query!(
    prices,
    prices_products,
    products,
    users,
);
