use actix_web::dev::{Server, ServerHandle};
use actix_web::{web, App, HttpServer};
use api::v1::{employee, inventory, order};
use config::db::Db;
use config::meilisearch::Meilisearch;
use db::mysql::init_db_pool;
use search::meilisearch::init_meilisearch;
use sea_orm::{ConnectionTrait, Statement};
use std::future::Future;
use std::net::TcpListener;

pub struct TestApp {
    pub server_url: String,
}

pub async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(TestApp) -> Fut,
    Fut: Future<Output = ()>,
{
    dotenv::dotenv().ok();
    let _ = env_logger::try_init();

    // Setup Database dan Meilisearch
    let db_pool = init_db_pool(&Db::new("test").url)
        .await
        .expect("Gagal inisialisasi pool database untuk tes");

    db_pool
        .execute(Statement::from_string(
            db_pool.get_database_backend(),
            "DELETE FROM `inventory`;",
        ))
        .await
        .expect("Gagal membersihkan tabel inventory");
    db_pool
        .execute(Statement::from_string(
            db_pool.get_database_backend(),
            "DELETE FROM `employee`;",
        ))
        .await
        .expect("Gagal membersihkan tabel employee");
    db_pool
        .execute(Statement::from_string(
            db_pool.get_database_backend(),
            "DELETE FROM `order`;",
        ))
        .await
        .expect("Gagal membersihkan tabel order");

    let meili_client =
        init_meilisearch(&Meilisearch::new("test").host, &Meilisearch::new("test").api_key)
            .await
            .expect("Gagal inisialisasi Meilisearch untuk tes");
    
    let index = meili_client.index("inventory");
    let _ = index.delete().await;

    // Setup Server
    let listener = TcpListener::bind("127.0.0.1:0").expect("Gagal bind ke port acak");
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    
    let db_pool_for_server = init_db_pool(&Db::new("test").url)
        .await
        .expect("Gagal inisialisasi pool database untuk server");
    let meili_client_for_server = meili_client.clone();

    let server: Server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool_for_server.clone()))
            .app_data(web::Data::new(meili_client_for_server.clone()))
            .configure(inventory::routes::init_routes)
            .configure(employee::routes::init_routes)
            .configure(order::routes::init_routes)
    })
    .listen(listener)
    .expect("Gagal listen pada listener")
    .run();

    let server_handle: ServerHandle = server.handle();
    let server_task = tokio::spawn(server);

    let app = TestApp { server_url };

    // Eksekusi badan tes
    test_body(app).await;

    // Teardown
    server_handle.stop(true).await;
    // Menunggu task server selesai dan mengabaikan hasilnya.
    let _ = server_task.await;
}