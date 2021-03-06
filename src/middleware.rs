use nickel::{Request, Response, Middleware, Action, Continue, NickelError};
use postgres::{Connection, SslMode};
use r2d2_postgres::{PostgresPoolManager, Error};
use r2d2::{Pool, ErrorHandler, Config, NoopErrorHandler, PooledConnection};
use std::default::Default;
use std::sync::Arc;

pub struct PostgresMiddleware<H: ErrorHandler<Error>> {
    pub pool: Arc<Pool<Connection, Error, PostgresPoolManager, H>>
}

impl<H> PostgresMiddleware<H> where H: ErrorHandler<Error> {
    pub fn new(connect_str: &str, ssl_mode: SslMode, num_connections: uint, handler: H)
            -> PostgresMiddleware<H> {
        let manager = PostgresPoolManager::new(connect_str, ssl_mode);
        let config = Config {
            pool_size: num_connections,
            ..Default::default()
        };

        PostgresMiddleware {
            pool: Arc::new(Pool::new(config, manager, handler).unwrap())
        }
    }
}

impl<H> Middleware for PostgresMiddleware<H> where H: ErrorHandler<Error> {
    fn invoke (&self, req: &mut Request, _resp: &mut Response) -> Result<Action, NickelError> {
        req.map.insert(self.pool.clone());
        Ok(Continue)
    }
}

pub trait PostgresRequestExtensions {
    fn db_conn(&self) -> PooledConnection<Connection, Error, PostgresPoolManager, NoopErrorHandler>;
}

impl<'a, 'b> PostgresRequestExtensions for Request<'a, 'b> {
    fn db_conn(&self) -> PooledConnection<Connection, Error, PostgresPoolManager, NoopErrorHandler> {
        self.map.find::<Arc<Pool<Connection,
                             Error,
                             PostgresPoolManager,
                             NoopErrorHandler>>>().unwrap()
                                                 .get()
                                                 .unwrap()
    }
}
