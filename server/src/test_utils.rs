use actix_web::dev::Service;
use actix_web::{test, web, App};
use sqlx::postgres::PgPoolOptions;
use std::env;

pub async fn test_app() -> impl Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create test pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .service(crate::index)
            .service(crate::health)
            .service(crate::list_threads)
            .service(crate::create_thread)
            .service(crate::list_categories)
            .service(crate::create_category)
            .service(crate::check_username)
            .service(crate::register_user)
            .service(crate::list_comments)
            .service(crate::create_comment)
    ).await
}
