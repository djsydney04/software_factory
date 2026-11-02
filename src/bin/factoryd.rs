use software_factory::{
    client::default_data_dir,
    provider::discover_capabilities,
    runtime::Supervisor,
    service::{self, DataDirectory, ServiceState},
    store::Store,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data_dir = default_data_dir();
    let mut executable = std::env::var_os("FACTORY_CODEX_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/bin/codex")
        });
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => data_dir = args.next().ok_or("--data-dir requires a path")?.into(),
            "--provider-executable" => {
                executable = args
                    .next()
                    .ok_or("--provider-executable requires an absolute path")?
                    .into()
            }
            "--help" => {
                println!("factoryd [--data-dir PATH] [--provider-executable PATH]\nIndependent authenticated local Software Factory service");
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    let directory = DataDirectory::acquire(data_dir)?;
    let store = Store::open(directory.path.join("factory.sqlite3"))?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let connection = directory.publish_connection(address)?;
    let state =
        ServiceState::new(store, connection.token, address).with_data_dir(directory.path.clone());
    let capability_state = state.clone();
    let capability_executable = executable.clone();
    let capability_cwd = directory.path.clone();
    let discovery = tokio::spawn(async move {
        let value = match discover_capabilities(capability_executable, capability_cwd).await {
            Ok(caps) => serde_json::to_value(caps).unwrap(),
            Err(error) => {
                serde_json::json!({"available":false,"authenticated":false,"models":[],"limitations":[format!("Codex metadata unavailable: {error}")]})
            }
        };
        *capability_state
            .capabilities
            .write()
            .unwrap_or_else(|e| e.into_inner()) = value;
    });
    let (shutdown_send, shutdown_receive) = tokio::sync::watch::channel(false);
    let supervisor = tokio::spawn(
        Supervisor::new(state.clone(), directory.path.clone(), executable).serve(shutdown_receive),
    );
    println!("Software Factory service listening at http://{address}. Native client authentication is configured.");
    axum::serve(listener, service::router(state))
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = shutdown_send.send(true);
        })
        .await?;
    discovery.abort();
    let _ = discovery.await;
    supervisor
        .await?
        .map_err(|e| e as Box<dyn std::error::Error>)?;
    Ok(())
}
