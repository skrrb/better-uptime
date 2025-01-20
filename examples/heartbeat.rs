#[tokio::main]
async fn main() -> better_uptime::Result<()> {
    // https://uptime.betterstack.com/api/v1/heartbeat/XXX, the XXX part is the identifier
    let identifier = "XXX";
    better_uptime::Uptime::heartbeat(identifier).await?;

    Ok(())
}
