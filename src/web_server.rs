use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use picoserve::futures::Either as WsEither;
use picoserve::response::WebSocketUpgrade;
use picoserve::response::ws::{Message, SocketRx, SocketTx, WebSocketCallback};
use picoserve::{
    AppBuilder, AppRouter, Server, make_static,
    routing::{PathRouter, get, get_service},
};

use crate::config;
use crate::shared::*;
use crate::ui::funcui::REDRAW_UI;

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
                "/ws",
                get(|upgrade: WebSocketUpgrade| async move { upgrade.on_upgrade(EchoWebSocket) }),
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

pub struct EchoWebSocket;

impl WebSocketCallback for EchoWebSocket {
    async fn run<R: embedded_io_async::Read, W: picoserve::io::Write<Error = R::Error>>(
        self,
        mut rx: SocketRx<R>,
        mut tx: SocketTx<W>,
    ) -> Result<(), W::Error> {
        let mut buffer = [0u8; 1024];
        let tx_packages_receiver = PACKET_CHANNEL.receiver();

        // This code runs for every WS connection
        info!("WS Connected");
        REDRAW_UI.signal(());

        loop {
            // `core::future::pending()` is a future that never resolves.
            // It's used here because we don't need a cancellation signal.

            // TODO it looks messy - it should be rewritten but no idea how
            match rx
                .next_message(&mut buffer, tx_packages_receiver.receive())
                .await
            {
                Ok(WsEither::First(Ok(message))) => match message {
                    Message::Text(text) => {
                        info!("Text Message Received {}", text);
                        REDRAW_UI.signal(());
                    }
                    Message::Binary(_) => {}
                    Message::Close(reason) => {
                        info!("WS Connection closed: {}", reason);
                        break;
                    }
                    Message::Ping(data) => {
                        tx.send_pong(data).await?;
                        info!("WS Ping");
                    }
                    Message::Pong(_) => todo!(),
                },
                Ok(WsEither::First(Err(e))) => {
                    error!("WE: {:?}", defmt::Debug2Format(&e));
                    break;
                }
                Ok(WsEither::Second(packet)) => {
                    tx.send_text(core::str::from_utf8(&packet).unwrap_or("[]"))
                        .await?;
                }
                Err(e) => {
                    error!("WE: {:?}", defmt::Debug2Format(&e));
                    break;
                }
            };
        }

        Ok(())
    }
}
