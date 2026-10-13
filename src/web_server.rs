use defmt::info;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use picoserve::{
    AppBuilder, AppRouter, Server, make_static,
    routing::{PathRouter, get_service},
};

use crate::config;

const WEB_TASK_POOL_SIZE: usize = config::WEB_TASK_POOL_SIZE;

pub struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new()
            .route(
                "/",
                get_service(picoserve::response::File::html(include_str!(
                    "web/index.html"
                ))),
            )
            .route(
                "/style.css",
                get_service(picoserve::response::File::html(include_str!(
                    "web/style.css"
                ))),
            )
            .route(
                "/main.js",
                get_service(picoserve::response::File::html(include_str!("web/main.js"))),
            )
    }
}

pub async fn start_web_server(spawner: Spawner, stack: Stack<'static>) {
    info!("Starting web server with {} tasks...", WEB_TASK_POOL_SIZE);

    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());

    let config = make_static!(
        picoserve::Config,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Duration::from_secs(5),
            persistent_start_read_request: Duration::from_secs(1),
            read_request: Duration::from_secs(1),
            write: Duration::from_secs(1),
        })
        .keep_connection_alive()
    );

    for id in 0..WEB_TASK_POOL_SIZE {
        spawner.spawn(web_task(id, stack, app, config).unwrap());
    }
}

#[embassy_executor::task(pool_size=WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    stack: Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config,
) {
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 4096];

    info!("Web server listening on port {}", config::HTTP_SERVER_PORT);

    let server = Server::new(app, config, &mut http_buffer);

    server
        .listen_and_serve(
            id,
            stack,
            config::HTTP_SERVER_PORT,
            &mut tcp_rx_buffer,
            &mut tcp_tx_buffer,
        )
        .await;
}
