use anyhow::Result;
use axum::{
    extract::{OriginalUri, Request, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use minijinja::Environment;
use percent_encoding::percent_decode;
use serde::Serialize;
use std::{
    net::{IpAddr, SocketAddr},
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use tokio::fs;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{debug, error, info};

struct HttpServeState {
    path: PathBuf,
}

#[derive(Serialize)]
struct DirList {
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

    // Create a router for file service handler.
    // Note that the path must include a '/' and also follow the '/*key' pattern.
    let file_app = Router::new()
        .route("/", get(file_service))
        .route("/*key", get(file_service));

    // Customize the path here and integrate it with file_app.
    // Note that it needs to end with a slash.
    let app = Router::new()
        .nest("/", file_app)
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);
    let addr = SocketAddr::new(*addr, port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on: {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn file_service(State(state): State<Arc<HttpServeState>>, req: Request) -> Response {
    debug!("Start file service handler...");

    // Concatenate local file path
    let req_path = req.uri().path();
    let file_path = match build_and_validate_path(state.path.clone(), req_path) {
        Some(path) => path,
        None => {
            error!("Invalid path: {:?}", req_path);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // check the path, if it is a directory, Generate a directory file list.
    // If it is a file, serve the file.
    if file_path.is_dir() {
        // For directory access, if there is no trailing slash "/", redirect to dir/.
        match check_path_suffix(&req) {
            Some(res) => res,
            None => {
                match get_dir_list(file_path).await {
                    Ok(metadata) => {
                        // Render an HTML page.
                        match render_template(metadata) {
                            Ok(rendered) => Html(rendered).into_response(),
                            Err(e) => {
                                error!("Error rendering template: {:?}", e);
                                StatusCode::INTERNAL_SERVER_ERROR.into_response()
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading directory: {:?}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
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

fn build_and_validate_path(base_path: impl AsRef<Path>, req_path: &str) -> Option<PathBuf> {
    let path = req_path.trim_start_matches('/');
    let path_decoded = percent_decode(path.as_bytes()).decode_utf8().ok()?;
    let path_decoded = Path::new(&*path_decoded);
    let mut path_to_file = base_path.as_ref().to_path_buf();
    for component in path_decoded.components() {
        match component {
            Component::Normal(comp) => {
                if Path::new(comp)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
                {
                    path_to_file.push(comp);
                } else {
                    return None;
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::ParentDir | Component::RootDir => return None,
        }
    }
    Some(path_to_file)
}

fn check_path_suffix(req: &Request) -> Option<Response> {
    let original_uri = if let Some(path) = req.extensions().get::<OriginalUri>() {
        path.0.path()
    } else {
        req.uri().path()
    };
    if !original_uri.ends_with('/') {
        Some(Redirect::permanent(add_root_suffix(original_uri).as_str()).into_response())
    } else {
        None
    }
}

// Generate a directory file list
async fn get_dir_list(local_path: impl AsRef<Path>) -> Result<DirList> {
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
        let icon = format!("{}.gif", etype);
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
        entries: df_entries,
    })
}

fn add_root_suffix(path: &str) -> String {
    if path.is_empty() {
        "/".to_string()
    } else {
        format!("{}/", path)
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
        let path = "src";
        let result = add_root_suffix(path);
        assert_eq!(result, "src/");
    }

    #[tokio::test]
    async fn test_get_dir_list() {
        let local_path = PathBuf::from("src");
        let result = get_dir_list(local_path).await;
        assert!(result.is_ok());
        let mut dir_list = result.unwrap();
        dir_list.entries.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(dir_list.entries.len(), 5);
        assert_eq!(dir_list.entries[0].etype, "folder");
        assert_eq!(dir_list.entries[0].name, "cli/");
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
        let res = file_service(State(state), req).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers().get("content-type").unwrap(), "text/x-rust");
    }

    #[test]
    fn test_build_and_validate_path() {
        let base_path = PathBuf::from("src");
        let req_path = "/lib.rs";
        let result = build_and_validate_path(base_path, req_path);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("src/lib.rs"));
    }

    #[test]
    fn test_build_and_validate_path_invalid() {
        let base_path = PathBuf::from("src");
        let req_path = "/../lib.rs";
        let result = build_and_validate_path(base_path, req_path);
        assert!(result.is_none());
    }

    #[test]
    fn test_check_path_suffix() {
        let req = Request::builder()
            .uri(Uri::from_str("/src").unwrap())
            .body(axum::body::Body::empty())
            .unwrap();
        let result = check_path_suffix(&req);
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
    }

    #[test]
    fn test_check_path_suffix_no_redirect() {
        let req = Request::builder()
            .uri(Uri::from_str("/src/").unwrap())
            .body(axum::body::Body::empty())
            .unwrap();
        let result = check_path_suffix(&req);
        assert!(result.is_none());
    }

    #[test]
    fn test_render_template() {
        let data = DirList {
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
