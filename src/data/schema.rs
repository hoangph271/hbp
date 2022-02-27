table! {
    tbl_posts (id) {
        id -> Text,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}

table! {
    tbl_users (id) {
        id -> Text,
        username -> Text,
        hashed_password -> Text,
        title -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    tbl_posts,
    tbl_users,
);
