#[macro_use]
extern crate diesel;
extern crate iron;
extern crate iron_diesel_middleware;

use iron::prelude::*;
use iron::status;
use diesel::prelude::*;
use iron_diesel_middleware::{DieselMiddleware, DieselPooledConnection, DieselReqExt};

// Create this table in the `example_middleware` database:
// CREATE TABLE users (
//     "id" serial,
//     "name" text,
//     PRIMARY KEY ("id")
// );
table! {
    users {
        id -> Integer,
        name -> Varchar,
    }
}

pub fn list_users(req: &mut Request) -> IronResult<Response> {
    // Get a diesel connection
    let con: DieselPooledConnection<diesel::pg::PgConnection> = req.db_conn();
    let all_users: Vec<(i32, String)> = users::table.load(&*con).unwrap();

    let mut user_list = String::new();
    for user in all_users {
        // Each line contains a user in the format id: name
        user_list.push_str(&format!("{}: {}\n", user.0, user.1));
    }

    Ok(Response::with((status::Ok, user_list)))
}

pub fn main() {
    let diesel_middleware: DieselMiddleware<diesel::pg::PgConnection> = DieselMiddleware::new("postgresql://localhost/example_middleware").unwrap();

    // Link the middleware before every request so the middleware is
    // accessible to the request handler
    let mut chain = Chain::new(list_users);
    chain.link_before(diesel_middleware);

    // Run the web server
    let address = "127.0.0.1:8000";
    println!("Running webserver on {}...", address);
    Iron::new(chain).http(address).unwrap();
}
