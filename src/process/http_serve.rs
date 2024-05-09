use anyhow::Result;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use minijinja::Environment;
use serde::Serialize;
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};
use tokio::fs;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{error, info};

struct HttpServeState {
    path: PathBuf,
}

#[derive(Serialize)]
struct DirList {
    parent: String,
    entries: Vec<DirEntry>,
}

#[derive(Serialize)]
struct DirEntry {
    path: String,
    name: String,
    etype: String,
    icon: String,
    update: String,
    size: String,
}

pub async fn process_http_serve(
    path: PathBuf,
    addr: &IpAddr,
    port: u16,
    _daemon: bool,
) -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting http server...");
    let shared_state = Arc::new(HttpServeState { path: path.clone() });
    let app = Router::new()
        .route("/", get(file_service))
        .route("/*key", get(file_service))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);
    let addr = SocketAddr::new(*addr, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on: {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn file_service(
    State(state): State<Arc<HttpServeState>>,
    path: Option<Path<PathBuf>>,
    req: Request,
) -> Response {
    // check the uri path
    let path = path.map(|p| p.0).unwrap_or_else(|| PathBuf::from(""));
    // Concatenate local file path
    let file_path = state.path.join(path.clone());

    // check the path, if it is a directory, Generate a directory file list.
    // If it is a file, serve the file.
    if file_path.is_dir() {
        // For directory access, if there is no trailing slash "/", redirect to dir/.
        let check = if path == PathBuf::from("") {
            true
        } else {
            path.to_string_lossy().ends_with('/')
        };

        if !check {
            Redirect::permanent(add_root_suffix(&path).as_str()).into_response()
        } else {
            match get_dir_list(path, file_path).await {
                Ok(metadata) => {
                    // Render an HTML page.
                    let rendered = render_template(metadata).unwrap();
                    Html(rendered).into_response()
                }
                Err(e) => {
                    error!("Error reading directory: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    } else {
        // use tower-http ServeDir to serve files
        let mut dir_service = ServeDir::new(state.path.clone());
        match dir_service.try_call(req).await {
            Ok(res) => res.into_response(),
            Err(e) => {
                error!("Error serving file: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

// Generate a directory file list
async fn get_dir_list(req_path: PathBuf, local_path: PathBuf) -> Result<DirList> {
    let mut entries = fs::read_dir(local_path).await?;
    let mut df_entries = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        let etype = if entry.path().is_dir() {
            "folder".to_string()
        } else {
            "text".to_string()
        };
        let name = match entry.file_name().to_str() {
            Some(name) => {
                if name.starts_with('.') {
                    continue;
                } else if etype == "folder" {
                    format!("{}/", name)
                } else {
                    name.to_string()
                }
            }
            None => continue,
        };
        let path = name.clone();
        let icon = format!("/{}.gif", etype);
        let date_time: DateTime<Utc> = entry.metadata().await?.modified()?.into();
        let update = date_time.format("%Y-%m-%d %H:%M").to_string();
        let size_bytes = entry.metadata().await?.len();
        let size = match etype.as_str() {
            "folder" => "-".to_string(),
            _ => {
                if size_bytes < 1024 {
                    format!("{}B", size_bytes)
                } else if size_bytes < 1024 * 1024 {
                    format!("{:.1}KB", size_bytes as f64 / 1024.0)
                } else {
                    format!("{:.1}MB", size_bytes as f64 / 1024.0 / 1024.0)
                }
            }
        };
        df_entries.push(DirEntry {
            path,
            name,
            etype,
            icon,
            update,
            size,
        });
    }
    Ok(DirList {
        parent: if let Some(parent) = req_path.parent() {
            add_root_suffix(parent)
        } else {
            "/".to_string()
        },
        entries: df_entries,
    })
}

fn add_root_suffix(path: &std::path::Path) -> String {
    if path == PathBuf::from("") {
        "/".to_string()
    } else {
        format!("/{}/", path.display())
    }
}

const INDEX_HTML: &str = include_str!("../../templates/index.html");

fn render_template(data: impl Serialize) -> Result<String> {
    let mut env = Environment::new();
    let temp_name = "index.html";
    env.add_template(temp_name, INDEX_HTML)?;
    let tmpl = env.get_template(temp_name)?;
    let rendered = tmpl.render(data)?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, Uri};
    use std::str::FromStr;

    #[test]
    fn test_add_root_suffix() {
        let path = PathBuf::from("src");
        let result = add_root_suffix(&path);
        assert_eq!(result, "/src/");
    }

    #[tokio::test]
    async fn test_get_dir_list() {
        let path = PathBuf::from("cli/http.rs");
        let local_path = PathBuf::from("src");
        let result = get_dir_list(path, local_path).await;
        assert!(result.is_ok());
        let mut dir_list = result.unwrap();
        dir_list.entries.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(dir_list.entries.len(), 5);
        assert_eq!(dir_list.entries[0].etype, "folder");
        assert_eq!(dir_list.entries[0].name, "cli/");
        assert_eq!(dir_list.parent, "/cli/")
    }

    #[tokio::test]
    async fn test_file_service() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("src"),
        });
        let req = Request::builder()
            .uri(Uri::from_str("/lib.rs").unwrap())
            .body(axum::body::Body::empty())
            .unwrap();
        let res = file_service(State(state), Some(Path(PathBuf::from("lib.rs"))), req).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "text/x-rust");
    }

    #[test]
    fn test_render_template() {
        let data = DirList {
            parent: "/".to_string(),
            entries: vec![DirEntry {
                path: "src/".to_string(),
                name: "src/".to_string(),
                etype: "folder".to_string(),
                icon: "/folder.gif".to_string(),
                update: "2021-09-01 00:00".to_string(),
                size: "-".to_string(),
            }],
        };
        let result = render_template(data);
        assert!(result.is_ok());
    }
}
