extern crate iron;
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;

use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};

use std::error::Error;
use std::sync::Arc;
use diesel::pg::PgConnection;

/// The type of the pool stored in `DieselMiddleware`.
pub type DieselPool = Arc<r2d2::Pool<r2d2_diesel::ConnectionManager<PgConnection>>>;

/// Iron middleware that allows for diesel postgres connections within requests.
pub struct DieselMiddleware {
  /// A pool of diesel postgres connections that are shared between requests.
  pub pool: DieselPool,
}

pub struct Value(DieselPool);

impl typemap::Key for DieselMiddleware { type Value = Value; }

impl DieselMiddleware {

    /// Creates a new pooled connection to the given postgresql server. The URL is in the format:
    ///
    /// ```{none}
    /// postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]
    /// ```
    ///
    /// Returns `Err(err)` if there are any errors connecting to the postgresql database.
    pub fn new(pg_connection_str: &str) -> Result<DieselMiddleware, Box<Error>> {
        let config = r2d2::Config::default();
        let manager = r2d2_diesel::ConnectionManager::<PgConnection>::new(pg_connection_str);
        let pool = try!(r2d2::Pool::new(config, manager));

        Ok(DieselMiddleware {
          pool: Arc::new(pool),
        })
    }
}

impl BeforeMiddleware for DieselMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<DieselMiddleware>(Value(self.pool.clone()));
        Ok(())
    }
}

/// Adds a method to requests to get a database connection.
///
/// ## Example
///
/// ```ignore
/// use iron_diesel_middleware::DieselReqExt;
///
/// fn handler(req: &mut Request) -> IronResult<Response> {
///   let connection = req.db_conn();
///
///   let new_user = NewUser::new("John Smith", 25);
///   diesel::insert(&new_user).into(users::table).execute(&*connection);
///
///   Ok(Response::with((status::Ok, "Added User")))
/// }
/// ```
pub trait DieselReqExt {
  /// Returns a pooled connection to the postgresql database. The connection is returned to
  /// the pool when the pooled connection is dropped.
  ///
  /// **Panics** if a `DieselMiddleware` has not been registered with Iron, or if retrieving
  /// a connection to the database times out.
  fn db_conn(&self) -> r2d2::PooledConnection<r2d2_diesel::ConnectionManager<PgConnection>>;
}

impl<'a, 'b> DieselReqExt for Request<'a, 'b> {
  fn db_conn(&self) -> r2d2::PooledConnection<r2d2_diesel::ConnectionManager<PgConnection>> {
    let poll_value = self.extensions.get::<DieselMiddleware>().unwrap();
    let &Value(ref poll) = poll_value;

    return poll.get().unwrap();
  }
}
