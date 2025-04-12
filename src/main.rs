use std::{
    fs, 
    net::IpAddr, 
    path::PathBuf, 
    sync::OnceLock
};
use salvo::{prelude::*, routing::PathFilter};
use clap::Parser;
use regex::Regex;
use tracing::info;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    home: PathBuf,
    #[arg(short, long = "listen-at", default_value = "127.0.0.1")]
    listen_addr: String,
    #[arg(long, short, default_value_t = 8080)]
    port: u16
}

#[derive(Debug)]
struct Properties {
    home: PathBuf
}

const R: &str = r"^(?:[a-zA-Z0-9_-]+|(?:\.[a-zA-Z0-9_-]+)+)(?:\.[a-zA-Z0-9]+)*$";

static PROPS: OnceLock<Properties> = OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let Args { home, listen_addr, port } = Args::parse();
    
    if !fs::exists(&home)? {
        fs::create_dir_all(&home)?
    }
    info!("home is: {home:?}");

    let listen_addr = listen_addr.parse::<IpAddr>()?;
    let tcp_listener = TcpListener::new((listen_addr, port)).try_bind().await?;

    PROPS.set(Properties { home: home.clone() })
        .expect("static properties are already initialized");

    let r = Regex::new(R)?;
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

#[cfg(test)]
mod test {
    use regex::Regex;
    use super::R;

    #[test]
    fn test_regex() {
        let regex = match Regex::new(R) {
            Ok(r) => r,
            Err(e) => panic!("{e}")
        };
        assert!(regex.is_match("my_av_1-9.mp4"));
        assert!(regex.is_match("my_av_1-9.mp4.jpg"));
        assert!(regex.is_match(".anan"));
        assert!(regex.is_match(".tonight.fun.av"));
        assert!(regex.is_match("oh---good____"));
        assert!("!#$%&'()=~^|@`{[]}:*;+<>,/?\"\\ あいアイ愛¥".chars().all(|c| {
            !regex.is_match(&c.to_string())            
        }));
        assert!(!regex.is_match("..oh...good"));
        assert!(!regex.is_match("."));        
        assert!(!regex.is_match(""));
        assert!(!regex.is_match("my_av_1-9.mp_4"));
        assert!(!regex.is_match("my_av_1-9.mp-4"));
        assert!(!regex.is_match("my_av_1-9."));
    }
}