macro_rules! function_handler {
    ( $handler_name:ident ($($arg:ident:$typ:ty),*) -> $body:expr) => {
        pub fn $handler_name(user: LoggedUser, pool: web::Data<PgPool>, $($arg:$typ,)*) 
            -> impl Future<Item = HttpResponse, Error = actix_web::Error>
        {
            web::block(move || {
                let pg_pool = pool
                    .get()
                    .map_err(|_| {
                        crate::errors::MyStoreError::PGConnectionError
                    })?;
                $body(user, pg_pool)
            })
            .then(|res| match res {
                Ok(data) => Ok(HttpResponse::Ok().json(data)),
                Err(error) => Err(actix_web::error::ErrorInternalServerError(error)),
            })
        }
    };
}