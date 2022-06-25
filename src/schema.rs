table! {
    events (id) {
        id -> Integer,
        evt_type -> Integer,
        timestamp -> Timestamp,
        message -> Text,
    }
}

table! {
    evt_type (id) {
        id -> Integer,
        name -> Text,
    }
}

joinable!(events -> evt_type (evt_type));

allow_tables_to_appear_in_same_query!(
    events,
    evt_type,
);
