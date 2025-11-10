use chrono::{DateTime, Utc};
use std::{collections::HashMap, fs::File};
//use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
//use nix::sys::socket::{sendmsg, ControlMessage, MsgFlags};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

#[derive(Clone, Serialize, Deserialize)]
struct DatasetInfo {
    asset: String,
    timeframe: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    count: usize,
    version: i32,
    path: String,
    ta: Vec<String>, // technical indicators
}

#[derive(Clone)]
struct AppState {
    arg_path: String,
    datasets: Arc<RwLock<HashMap<String, DatasetInfo>>>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Unix socket, doing only http listener for now
    /*
    let listener = UnixListener::bind("/tmp/dataset_manager.sock")?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        handle_client(&mut stream, &datasets)?;
    }
    */
    let arg_path = env::args().nth(1).expect("No argument provided");

    let datasets = Arc::new(RwLock::new(load_datasets(&arg_path)?));
    let state = AppState { arg_path, datasets };

    let app = Router::new()
        .route("/datasets", get(list_datasets))
        .route("/datasets/:name", get(get_dataset))
        .route("/datasets/reload", post(reload_all))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8081));
    println!("DatasetManager running at {}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/*
fn handle_client(stream: &mut UnixStream, datasets: &HashMap<String, Dataset>) -> std::io::Result<()> {

    let mut buf = [0u8; 128];
    let n = stream.read(&mut buf)?;
    let name = std::str::from_utf8(&buf[..n]).unwrap().trim();

    if let Some(ds) = datasets.get(name) {
        println!("{}", "../datasets/".to_string() + &ds.meta.path);
        let file = File::open("../datasets/".to_string() + &ds.meta.path)?;
        let fd = file.as_raw_fd();
        let meta_json = serde_json::to_string(&ds.meta)?;
        let iov = [IoSlice::new(meta_json.as_bytes())];
        sendmsg::<()>(
            stream.as_raw_fd(),
            &iov,
            &[ControlMessage::ScmRights(&[fd])],
            MsgFlags::empty(),
            None,
        )?;
    }
    Ok(())
}
*/

fn load_datasets(path: &str) -> Result<HashMap<String, DatasetInfo>, std::io::Error> {
    // Normally you’d read all .meta.json files from your datasets directory
    let mut map = HashMap::new();
    //for entry in std::fs::read_dir("../datasets")? {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        // skip directories
        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) == Some("bin") {
            let meta_path = path.with_extension("meta.json");

            if !meta_path.exists() {
                eprintln!("Warning: No meta.json found for {:?}", path);
                continue;
            }

            match File::open(&meta_path) {
                Ok(file) => match serde_json::from_reader::<_, DatasetInfo>(file) {
                    Ok(meta) => {
                        let key = format!("{}-{}", meta.asset, meta.timeframe);
                        println!("Loaded dataset: {}", key);
                        map.insert(key, meta);
                    }
                    Err(e) => {
                        eprintln!("Error parsing meta.json for {:?}: {}", meta_path, e);
                    }
                },
                Err(e) => {
                    eprintln!("Error opening {:?}: {}", meta_path, e);
                }
            }

            // These two lines are for keeping the file opened in memory, but for now, not doing that
            //let file = File::open(&path)?;
            //let mmap = unsafe { Mmap::map(&file)? };
        }
    }

    println!("Loaded {} datasets", map.len());
    Ok(map)
}

async fn list_datasets(State(state): State<AppState>) -> Json<Vec<DatasetInfo>> {
    let lock = state.datasets.read().unwrap();
    Json(lock.values().cloned().collect())
}

async fn get_dataset(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<DatasetInfo>, StatusCode> {
    let lock = state.datasets.read().unwrap();
    lock.get(&name)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn reload_all(State(state): State<AppState>) -> Result<Json<String>, (StatusCode, String)> {
    match load_datasets(&state.arg_path) {
        Ok(new_data) => {
            let mut lock = state.datasets.write().unwrap();
            let count = new_data.len();
            *lock = new_data;
            Ok(Json(format!("Reloaded {} datasets", count)))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to reload: {}", e),
        )),
    }
}
