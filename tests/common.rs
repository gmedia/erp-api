use actix_web::dev::{Server, ServerHandle};
use actix_web::{web, App, HttpServer};
use api::v1::{employee, inventory, order};
use config::meilisearch::Meilisearch;
use db::mysql::init_db_pool;
use migration::{Migrator, MigratorTrait};
use once_cell::sync::Lazy;
use search::meilisearch::init_meilisearch;
use sea_orm::{ConnectionTrait, Database, Statement};
use std::future::Future;
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::env;

pub struct TestApp {
    pub server_url: String,
}

static TEST_DBS: Lazy<Vec<String>> = Lazy::new(|| {
    (1..=11)
        .map(|i| format!("mysql://root:root_password@127.0.0.1:3306/erp_db_test_{}", i))
        .collect()
});

static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

async fn setup_databases() {
    let db_url = env::var("DATABASE_URL_ROOT")
        .unwrap_or_else(|_| "mysql://root:root_password@127.0.0.1:3306".to_string());
    let conn = Database::connect(&db_url).await.expect("Gagal terhubung ke server database");

    for db_url in TEST_DBS.iter() {
        let db_name = db_url.split('/').last().unwrap();
        conn.execute(Statement::from_string(
            conn.get_database_backend(),
            format!("DROP DATABASE IF EXISTS `{}`;", db_name),
        ))
        .await
        .unwrap();
        conn.execute(Statement::from_string(
            conn.get_database_backend(),
            format!("CREATE DATABASE `{}`;", db_name),
        ))
        .await
        .unwrap();

        let db_conn = Database::connect(db_url).await.expect("Gagal terhubung ke database test");
        Migrator::up(&db_conn, None).await.unwrap();
    }
}

pub async fn run_test<F, Fut>(test_body: F)
where
    F: FnOnce(TestApp) -> Fut,
    Fut: Future<Output = ()>,
{
    dotenv::dotenv().ok();
    let _ = env_logger::try_init();

    static SETUP_ONCE: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
    SETUP_ONCE.get_or_init(setup_databases).await;

    let db_index = DB_COUNTER.fetch_add(1, Ordering::SeqCst) % TEST_DBS.len();
    let db_url = &TEST_DBS[db_index];
    let db_pool = init_db_pool(db_url)
        .await
        .expect("Gagal inisialisasi pool database untuk tes");

    let meili_client =
        init_meilisearch(&Meilisearch::new("test").host, &Meilisearch::new("test").api_key)
            .await
            .expect("Gagal inisialisasi Meilisearch untuk tes");
    
    let index = meili_client.index("inventory");
    let _ = index.delete().await;
    meili_client.create_index("inventory", Some("id")).await.expect("Gagal membuat index inventory");

    let listener = TcpListener::bind("127.0.0.1:0").expect("Gagal bind ke port acak");
    let server_url = format!("http://{}", listener.local_addr().unwrap());
    
    let db_pool_for_server = db_pool.clone();
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

    test_body(app).await;

    server_handle.stop(true).await;
    let _ = server_task.await;
}