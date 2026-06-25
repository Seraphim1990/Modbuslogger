use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use crate::api::router;
use crate::messages::main_msg::MainMsg;
use crate::api::web_sockets::{live_socket_unit::CoordUnitWebSocketData, live_socket::live_router, live_socket_unit, web_sock_coord};
use tower_http::cors::{CorsLayer, Any};


use crate::api::sockets::socket_addr;
use crate::logger;

#[derive(Clone)]
pub struct AppState {
    pub from_api: mpsc::Sender<MainMsg>, // TODO додати канал для координатора вебсокетів
    pub to_ws_coord: mpsc::Sender<CoordUnitWebSocketData>
}

pub async fn init_axum(from_api:  mpsc::Sender<MainMsg>, to_api: mpsc::Receiver<MainMsg>) {

        let addr = socket_addr::init_socket();
        let listener = TcpListener::bind(addr).await.expect("Помилка читання TcpListener");

        let (to_ws_coord, rx_ws_coord) = mpsc::channel(32);

        let app_state = AppState{
            from_api,
            to_ws_coord,
        };

        tokio::spawn(web_sock_coord::web_sock_coord(to_api, rx_ws_coord));

        let app = Router::new()
            .merge(router::node::node_router())
            .merge(router::devices::devices_router())
            .merge(router::value::values_router())
            .merge(router::measures::measures_router())
            //.merge(router::root::root())
            .merge(live_router()) // "/live_data"
            .with_state(app_state)
            .layer(CorsLayer::permissive());;

        logger::printers::event(format!("Сервер запущено: {}", addr));

        axum::serve(listener, app).await.unwrap();
}

