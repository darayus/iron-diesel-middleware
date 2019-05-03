extern crate iron;
extern crate diesel;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};
use diesel::r2d2::{Pool, ConnectionManager, PooledConnection};

use std::error::Error;

/// The type of the pool stored in `DieselMiddleware`.
pub type DieselPool<T: diesel::Connection> = Pool<ConnectionManager<T>>;

pub type DieselPooledConnection<T: diesel::Connection> = PooledConnection<ConnectionManager<T>>;

/// Iron middleware that allows for diesel connections within requests.
pub struct DieselMiddleware<T: 'static + diesel::Connection> {
  /// A pool of diesel connections that are shared between requests.
  pub pool: DieselPool<T>,
}

pub struct Value<T: 'static + diesel::Connection>(DieselPool<T>);

impl<T: diesel::Connection> typemap::Key for DieselMiddleware<T> { type Value = Value<T>; }

impl<T: diesel::Connection> DieselMiddleware<T> {

    /// Creates a new pooled connection to the given sql server. The URL is in the format:
    ///
    /// ```{none}
    /// postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]
    /// ```
    ///
    /// Returns `Err(err)` if there are any errors connecting to the sql database.
    pub fn new(connection_str: &str) -> Result<DieselMiddleware<T>, Box<Error>> {
        let manager = ConnectionManager::<T>::new(connection_str); 
        Ok(Self::new_with_pool(Pool::builder().build(manager)?))
    }
    /// Creates a instance of the middleware with the ability to provide a preconfigured pool.
    pub fn new_with_pool(pool: Pool<ConnectionManager<T>>) -> DieselMiddleware<T> {
        DieselMiddleware {pool: pool}
    }
}

impl<T: diesel::Connection> BeforeMiddleware for DieselMiddleware<T> {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<DieselMiddleware<T>>(Value(self.pool.clone()));
        Ok(())
    }
}

/// Adds a method to requests to get a database connection.
///
/// ## Example
///
/// ```ignore
/// use iron_diesel_middleware::{DieselPooledConnection, DieselReqExt};
///
/// fn handler(req: &mut Request) -> IronResult<Response> {
///   let connection: DieselPooledConnection<diesel::pg::PgConnection> = req.db_conn();
///
///   let new_user = NewUser::new("John Smith", 25);
///   diesel::insert(&new_user).into(users::table).execute(&*connection);
///
///   Ok(Response::with((status::Ok, "Added User")))
/// }
/// ```
pub trait DieselReqExt<T: 'static + diesel::Connection> {
  /// Returns a pooled connection to the sql database. The connection is returned to
  /// the pool when the pooled connection is dropped.
  ///
  /// **Panics** if a `DieselMiddleware` has not been registered with Iron, or if retrieving
  /// a connection to the database times out.
  fn db_conn(&self) -> PooledConnection<ConnectionManager<T>>;
}

impl<'a, 'b, T: 'static + diesel::Connection> DieselReqExt<T> for Request<'a, 'b> {
  fn db_conn(&self) -> PooledConnection<ConnectionManager<T>> {
    let poll_value = self.extensions.get::<DieselMiddleware<T>>().unwrap();
    let &Value(ref poll) = poll_value;

    return poll.get().unwrap();
  }
}
