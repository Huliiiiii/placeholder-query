diesel::table! {
    actor (actor_id) {
        actor_id -> Integer,
        first_name -> Text,
        last_name -> Text,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    film (film_id) {
        film_id -> Integer,
        title -> Text,
        description -> Nullable<Text>,
        release_year -> Nullable<Integer>,
        language_id -> Integer,
        original_language_id -> Nullable<Integer>,
        rental_duration -> SmallInt,
        rental_rate -> Numeric,
        length -> Nullable<SmallInt>,
        replacement_cost -> Numeric,
        last_update -> Timestamptz,
        special_features -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    language (language_id) {
        language_id -> Integer,
        name -> Bpchar,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    film_actor (actor_id, film_id) {
        actor_id -> Integer,
        film_id -> Integer,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    category (category_id) {
        category_id -> Integer,
        name -> Text,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    film_category (film_id, category_id) {
        film_id -> Integer,
        category_id -> Integer,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    address (address_id) {
        address_id -> Integer,
        #[sql_name = "address"]
        address_ -> Text,
        address2 -> Nullable<Text>,
        district -> Text,
        city_id -> Integer,
        postal_code -> Nullable<Text>,
        phone -> Text,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    city (city_id) {
        city_id -> Integer,
        #[sql_name = "city"]
        city_ -> Text,
        country_id -> Integer,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    country (country_id) {
        country_id -> Integer,
        #[sql_name = "country"]
        country_ -> Text,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    customer (customer_id) {
        customer_id -> Integer,
        store_id -> Integer,
        first_name -> Text,
        last_name -> Text,
        email -> Nullable<Text>,
        address_id -> Integer,
        activebool -> Bool,
        create_date -> Date,
        last_update -> Nullable<Timestamptz>,
        active -> Nullable<Integer>,
    }
}

diesel::table! {
    inventory (inventory_id) {
        inventory_id -> Integer,
        film_id -> Integer,
        store_id -> Integer,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    rental (rental_id) {
        rental_id -> Integer,
        rental_date -> Timestamptz,
        inventory_id -> Integer,
        customer_id -> Integer,
        return_date -> Nullable<Timestamptz>,
        staff_id -> Integer,
        last_update -> Timestamptz,
    }
}

diesel::table! {
    payment (payment_date, payment_id) {
        payment_id -> Integer,
        customer_id -> Integer,
        staff_id -> Integer,
        rental_id -> Integer,
        amount -> Numeric,
        payment_date -> Timestamptz,
    }
}

diesel::table! {
    staff (staff_id) {
        staff_id -> Integer,
        first_name -> Text,
        last_name -> Text,
        address_id -> Integer,
        email -> Nullable<Text>,
        store_id -> Integer,
        active -> Bool,
        username -> Text,
        password -> Nullable<Text>,
        last_update -> Timestamptz,
        picture -> Nullable<Bytea>,
    }
}

diesel::table! {
    store (store_id) {
        store_id -> Integer,
        manager_staff_id -> Integer,
        address_id -> Integer,
        last_update -> Timestamptz,
    }
}

diesel::joinable!(film -> language (language_id));
diesel::joinable!(film_actor -> film (film_id));
diesel::joinable!(film_actor -> actor (actor_id));
diesel::joinable!(film_category -> film (film_id));
diesel::joinable!(film_category -> category (category_id));
diesel::joinable!(address -> city (city_id));
diesel::joinable!(city -> country (country_id));
diesel::joinable!(customer -> address (address_id));
diesel::joinable!(customer -> store (store_id));
diesel::joinable!(inventory -> film (film_id));
diesel::joinable!(inventory -> store (store_id));
diesel::joinable!(rental -> inventory (inventory_id));
diesel::joinable!(rental -> customer (customer_id));
diesel::joinable!(rental -> staff (staff_id));
diesel::joinable!(payment -> customer (customer_id));
diesel::joinable!(payment -> staff (staff_id));
diesel::joinable!(payment -> rental (rental_id));
diesel::joinable!(staff -> address (address_id));
diesel::joinable!(store -> staff (manager_staff_id));
diesel::joinable!(store -> address (address_id));

diesel::allow_tables_to_appear_in_same_query!(
    actor,
    film,
    language,
    film_actor,
    category,
    film_category,
    address,
    city,
    country,
    customer,
    inventory,
    rental,
    payment,
    staff,
    store,
);
