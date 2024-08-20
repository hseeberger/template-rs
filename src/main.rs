#[tokio::main]
async fn main() {
    // Error logging already happens in `{{ crate_name }}::main`.
    let _ = {{ crate_name }}::main().await;
}
