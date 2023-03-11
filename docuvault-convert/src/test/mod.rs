#[cfg(test)]
mod tests {
    use std::{sync::Once, env, net::ToSocketAddrs, time::Duration};

    use tokio::time::sleep;
    use tonic::{transport::Server, Request};

    use crate::{AppState, db, apis::convert::{ConvertService, convert::{convert_server::ConvertServer, convert_client::ConvertClient, ConvertRequest}}};
    static INIT: Once = Once::new();
    fn initialize() {
        INIT.call_once(|| {
            tokio::spawn(async {
                println!("initialize!");
                dotenvy::dotenv().ok();
                let state = AppState {
                    db_conn: db::postgres_connect().await,
                    redis_conn: db::redis_connect().await,
                };
                let convert_service = ConvertService {
                    state: state.clone(),
                };
                let server_addr = env::var("SERVER_ADDR").expect("server addr is not set");
                let server_addr = format!("{}:{}",server_addr,7000).to_socket_addrs().unwrap().next().unwrap();
                Server::builder()
                    .add_service(ConvertServer::new(convert_service))
                    .serve(server_addr)
                    .await
                    .unwrap();
                dbg!(&server_addr);
            });
        });
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test(){
            tokio::spawn(async {
                println!("initialize!");
                dotenvy::dotenv().ok();
                let state = AppState {
                    db_conn: db::postgres_connect().await,
                    redis_conn: db::redis_connect().await,
                };
                let convert_service = ConvertService {
                    state: state.clone(),
                };
                let server_addr = env::var("SERVER_ADDR").expect("server addr is not set");
                let server_addr = format!("{}:{}",server_addr,7000).to_socket_addrs().unwrap().next().unwrap();
                Server::builder()
                    .add_service(ConvertServer::new(convert_service))
                    .serve(server_addr)
                    .await
                    .unwrap();
                dbg!(&server_addr);
            });
        sleep(Duration::from_secs(5)).await;
        let mut convert_client = ConvertClient::connect("http://127.0.0.1:7000").await.unwrap();
        convert_client.convert(Request::new(ConvertRequest{
            title: "abc".to_owned(),
            doc_id: 1,
            docuser_id: 2,
            data: "hello there?".to_owned(),
            c_type: 3,
        })).await.unwrap();
    }
    #[tokio::test(flavor = "multi_thread")]
    async fn test2(){
        //initialize();     
    }

}
