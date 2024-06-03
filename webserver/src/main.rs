mod routes;
mod server;
mod static_values;
mod users;

const DB_URL: &str = "sqlite://sqlite.db";

#[tokio::main]
async fn main() {
    let server = server::start_server("127.0.0.1:3000", DB_URL);

    /*let wv_task = tokio::task::spawn_blocking(|| {
        web_view::builder()
            .title("Test")
            .content(web_view::Content::Url("http://localhost:3000"))
            .size(1024, 600)
            .user_data(())
            .invoke_handler(|_wv, _arg| Ok(()))
            .run()
            .unwrap();
    });

    tokio::select! {
        _ = wv_task => {
            println!("WebView finished");
        }
        e = server => {
            println!("Axum finished with result {e:?}");
        }
    }*/
    server.await.unwrap()
}
