use std::{
    net::IpAddr, 
    path::PathBuf, 
    sync::OnceLock
};
use anyhow::bail;
use salvo::{prelude::*, routing::PathFilter};
use clap::Parser;
use regex::Regex;
use tracing::info;
use tokio::fs;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    home: PathBuf,
    #[arg(short, long = "listen-at", default_value = "127.0.0.1")]
    listen_addr: String,
    #[arg(long, short, default_value_t = 4545)]
    port: u16
}

#[derive(Debug)]
struct Properties {
    home: PathBuf,
    validator: Regex
}

const R: &str = r"^(?:[a-zA-Z0-9_-]+|(?:\.[a-zA-Z0-9_-]+)+)(?:\.[a-zA-Z0-9]+)*$";

static PROPS: OnceLock<Properties> = OnceLock::new();

async fn make_dst_name(file_path: &PathBuf) -> anyhow::Result<Option<PathBuf>> {
    if !fs::try_exists(&file_path).await? {
        return Ok(None);
    }

    let original = match file_path.file_name() {
        Some(inner) => match inner.to_str() {
            Some(o) => o,
            None => bail!("could not convert file name as not valid utf-8")
        }
        None => bail!("validation is not work")
    };

    let mut path = file_path.clone();
    let mut n = 0u32;
    loop {
        let mut indexed = original.to_string();
        let number = format!("_copy{n}");
        match indexed.find('.') {
            Some(idx) => indexed.insert_str(idx, &number),
            None => indexed.push_str(&number)
        }

        path.set_file_name(indexed);
        if !fs::try_exists(&path).await? {
            return Ok(Some(path));
        }

        let (m, overflow) = n.overflowing_add(1);
        if overflow {
            bail!("could not make destination file");
        }
        n = m;
    }
}

#[handler]
async fn upload(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {
    let Some(Properties{ home, validator }) = PROPS.get() else {
        bail!("props are not initialized");
    };
    
    let Some(file) = req.file("file").await else {
        res.render(StatusError::bad_request());
        return Ok(())
    };

    let Some(file_name) = file.name() else {
        res.render(StatusError::bad_request());
        return Ok(())
    };
    if !validator.is_match(file_name) {
        res.render(StatusError::bad_request());
        return Ok(())
    }

    let tmp_path = file.path();
    let mut dst_path = home.clone();
    dst_path.push(file_name);

    if let Some(new_path) = make_dst_name(&dst_path).await? {
        dst_path = new_path
    }
    
    match fs::copy(tmp_path, &dst_path).await {
        Ok(n) => {
            info!("uploaded {n}bytes: {dst_path:?}");
            res.render("ok");
            Ok(())
        }
        Err(e) => bail!(e)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    
    let Args { home, listen_addr, port } = Args::parse();
    
    if !fs::try_exists(&home).await? {
        fs::create_dir_all(&home).await?
    }
    info!("home is: {home:?}");

    let listen_addr = listen_addr.parse::<IpAddr>()?;
    let tcp_listener = TcpListener::new((listen_addr, port)).try_bind().await?;

    PROPS.set(Properties { 
        home: home.clone(),
        validator: Regex::new(R)? 
    }).expect("static properties are already initialized");

    let r = Regex::new(R)?;
    PathFilter::register_wisp_regex("validation", r);
    let router = Router::new()
        .push(Router::with_path("upload").post(upload))
        .push(
            Router::with_path("download/{**file:validation}").get(
                StaticDir::new(home).auto_list(true).include_dot_files(true)
            )
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