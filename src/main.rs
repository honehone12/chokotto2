use std::{
    fs, 
    net::IpAddr, 
    path::{Path, PathBuf}, 
    sync::OnceLock
};
use salvo::{prelude::*, routing::PathFilter};
use clap::Parser;
use regex::Regex;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    home: PathBuf,
    #[arg(short, long = "listen-at", default_value = "127.0.0.1")]
    listen_addr: String,
    #[arg(long, short, default_value_t = 80)]
    port: u16
}

#[derive(Debug)]
struct Properties {
    pub_dir: PathBuf
}

static PROPS: OnceLock<Properties> = OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let Args { mut home, listen_addr, port } = Args::parse();
    
    home.push("Public");
    if !fs::exists(&home)? {
        fs::create_dir_all(&home)?
    }

    let listen_addr = listen_addr.parse::<IpAddr>()?;
    let tcp_listener = TcpListener::new((listen_addr, port)).try_bind().await?;

    PROPS.set(Properties { pub_dir: home.clone() })
        .expect("static properties are already initialized");

    let r = Regex::new(r"^(?:[a-zA-Z0-9_]+|(?:\.[a-zA-Z0-9_]+)+)(?:\.[a-zA-Z0-9_]+)*$")?;
    PathFilter::register_wisp_regex("file", r);
    let router = Router::new()
        // .push(Router::with_path("upload").post())
        .push(
            Router::with_path("download/{file}")
                .get(StaticDir::new(home))
        );

    Server::new(tcp_listener).try_serve(router).await?;

    Ok(())
}
