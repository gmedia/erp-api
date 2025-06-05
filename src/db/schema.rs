diesel::table! {
    inventory (id) {
        id -> Char,
        name -> Varchar,
        quantity -> Integer,
        price -> Double,
    }
}

diesel::table! {
    employees (id) {
        id -> Char,
        name -> Varchar,
        role -> Varchar,
        email -> Varchar,
    }
}

diesel::table! {
    orders (id) {
        id -> Char,
        customer_id -> Char,
        total_amount -> Double,
        created_at -> Timestamp,
    }
}