
#[macro_use]
extern crate diesel;
extern crate iron_diesel_middleware;
extern crate iron;

use iron::prelude::*;
use diesel::prelude::*;
use iron_diesel_middleware::{DieselMiddleware, DieselReqExt};

table! {
    visitors {
        id -> Serial,
        page -> VarChar,
    }
}

fn my_page(r: &mut Request) -> IronResult<Response> {
    let con = r.db_conn();
    diesel::insert(( "Hi".to_owned(),)).into(visitors::table).execute(&con).unwrap();
    Ok(Response::with((iron::status::Ok, "Hello World")))
}


fn main() {
    let diesel_middleware = DieselMiddleware::new("postgres://localhost/diesel_iron").unwrap();

    let mut chain = Chain::new(my_page);
    chain.link_before(diesel_middleware);
    println!("Running at localhost:3000...");
    Iron::new(chain).http("localhost:3000").unwrap();
}
