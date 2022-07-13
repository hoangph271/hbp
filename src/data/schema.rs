table! {
    tbl_posts (id) {
        id -> Text,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    tbl_posts,
);
