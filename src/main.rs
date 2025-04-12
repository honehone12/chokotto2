use salvo::Router;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new()
        .push(Router::with_path("upload").post())
        .push(
            Router::with_path("download/{file}")
                .get()
        );
    
    Ok(())
}
