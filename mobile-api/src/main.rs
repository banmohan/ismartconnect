#[tokio::main]
async fn main() {
    if let Err(err) = mobile_api::run().await {
        eprintln!("mobile-api failed: {err:#}");
        std::process::exit(1);
    }
}
