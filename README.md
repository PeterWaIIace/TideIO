# TideIO - Tide based EngineIO/SocketIO implementation

23.09.2024

Disclaimer: This is independent project, not (yet) connected to tide. It uses tide libraries. Tide is amazing rust framework for webapps and you should check it here: https://github.com/http-rs/tide/tree/main 

Project was done as internal need for Tide SocketIO implementaion

## Current state of the project **[Experimental - WiP]**:

- 👍 Long Polling: Works/WiP
- 👎 WebSockets: NYI/Not Yet Implemented

## How to test:

First:
```
cargo run 
```

Second got to `127.0.0.1:8000` <- you should see a lot of logs from echo messages

## How to use:

```Rust
use tide::prelude::*;
use async_std::prelude::*;
use std::sync::Arc;
use serde_json::Value;
use tide::{Request, Response};
use tide_websockets::{Message, WebSocket};

use futures_util::FutureExt;
mod tideio;

#[tokio::main] // Ensure there is no space in `async_std`
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/").serve_file("templates/index.html");

    let io = Arc::new(tideio::TideIO::new());
    let io_clone = Arc::clone(&io);
    io.on("bind",move |data,namespace,sid| {
        println!("Bind received data: {:?}", data);
        io_clone.emit("onBind", vec![Value::String(String::from("Echo"))], namespace, sid);
    });

    // Define routes
    let io_clone = Arc::clone(&io);
    app.at("/socket.io/").get(move |req: Request<()>| {
        let io: Arc<tideio::TideIO> = Arc::clone(&io_clone);
        async move {
            io.handle(req).await
        }
    });

    // Define routes
    let io_clone = Arc::clone(&io);
    app.at("/socket.io/").post(move |req: Request<()>| {
        let io: Arc<tideio::TideIO> = Arc::clone(&io_clone);
        async move {
            io.handle(req).await
        }
    });

    app.listen("127.0.0.1:8000").await?;
    Ok(())
}

```

You can use for now:

```Rust
io.on("bind",move |data,namespace,sid| {});
```

and 

```Rust
io.emit("onBind", vec![Value::String(String::from("Echo"))], namespace, sid);
```
