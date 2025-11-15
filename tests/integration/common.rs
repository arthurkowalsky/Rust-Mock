use reqwest;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

pub const TEST_PORT: u16 = 18090;
pub const BASE_URL: &str = "http://127.0.0.1:18090";

pub struct TestServer {
    process: Child,
}

impl TestServer {
    pub async fn start() -> Self {
        let build_status = Command::new("cargo")
            .args(&["build", "--release"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to build application");

        assert!(build_status.success(), "Build failed");

        let process = Command::new("./target/release/RustMock")
            .args(&["--port", &TEST_PORT.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        let client = reqwest::Client::new();

        for _ in 0..50 {
            if client.get(format!("{}/__mock/config", BASE_URL))
                .send()
                .await
                .is_ok()
            {
                println!("Server started successfully on port {}", TEST_PORT);
                return TestServer { process };
            }
            sleep(Duration::from_millis(100)).await;
        }

        panic!("Server failed to start within timeout");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        println!("Server stopped");
    }
}
