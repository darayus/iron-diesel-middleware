# Iron Diesel Middleware
[![Build Status](https://travis-ci.org/darayus/iron-diesel-middleware.svg?branch=master)](https://travis-ci.org/darayus/iron-diesel-middleware)

Middleware that provides diesel database connections (currently
PostgreSQL **only**) within iron requests. This is a port of
[iron-postgres-middleware](https://github.com/martinsp/iron-postgres-middleware).

Documentation can be found [here](https://docs.darayus.com/iron_diesel_middleware/iron_diesel_middleware/).

## Usage

1. Add the following to `Cargo.toml`:

   ```toml
   [dependencies.iron_diesel_middleware]
   git = "https://github.com/darayus/iron-diesel-middleware"
   ```
2. Include the crate and import the middleware:

   ```rust
   extern crate iron_diesel_middleware;
   use iron_diesel_middleware::{DieselMiddleware, DieselReqExt};
   ```
3. Setup and add the middleware to iron:

   ```rust
   let diesel_middleware = DieselMiddleware::new("postgresql://localhost/example_middleware").unwrap();
   let mut chain = Chain::new(example_handler);
   chain.link_before(diesel_middleware);
   Iron::new(chain).http("127.0.0.1:8000").unwrap();
   ```
4. Use the diesel connection in requests:
   
   ```rust
   fn example_handler(req: &mut Request) -> IronResult<Response> {
       let con = req.db_conn();
       let response_str = do_something_with(&*con);
       return Ok(Response::with((status::Ok, response_str)));
   }
   ```

A working example can be found in [examples/basic.rs](examples/basic.rs).
