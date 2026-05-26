mod pds;
mod sessions;

use std::{env, net::SocketAddr, sync::Arc};

use askama::Template;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::pds::{CreateAccountInput, UserProfile, PDS_URL};
use crate::sessions::{PendingSignup, SignupSessions};

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    pds_admin_password: String,
    app_base_url: String,
    sessions: Arc<SignupSessions>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    user_count: usize,
    version: String,
    users: Vec<UserProfile>,
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTemplate {
    email: String,
    handle: String,
    err: Option<String>,
    success: Option<String>,
}

#[derive(Template)]
#[template(path = "success.html")]
struct SuccessTemplate {
    handle: String,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,pds_dash=debug")),
        )
        .init();

    let state = AppState {
        http: reqwest::Client::builder()
            .user_agent("pds-dash/0.1")
            .build()
            .expect("reqwest client"),
        pds_admin_password: env::var("PDS_ADMIN_PASSWORD").unwrap_or_default(),
        app_base_url: env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
        sessions: Arc::new(SignupSessions::default()),
    };

    if state.pds_admin_password.is_empty() {
        tracing::warn!("PDS_ADMIN_PASSWORD is not set; creating invite codes will fail");
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/signup", get(signup_get).post(signup_post))
        .route("/signup/callback", get(signup_callback))
        .route("/signup/success", get(signup_success))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, app).await.expect("serve");
}

async fn index(State(state): State<Arc<AppState>>) -> Response {
    let (version, users) = tokio::join!(
        pds::get_pds_version(&state.http),
        pds::get_pds_users(&state.http),
    );

    let version = version.unwrap_or_else(|e| {
        tracing::error!(error = %e, "failed to load PDS version");
        "0".to_string()
    });
    let users = users.unwrap_or_else(|e| {
        tracing::error!(error = %e, "failed to load PDS users");
        Vec::new()
    });

    render(IndexTemplate {
        user_count: users.len(),
        version,
        users,
    })
}

async fn signup_get() -> Response {
    render(SignupTemplate {
        email: String::new(),
        handle: String::new(),
        err: None,
        success: None,
    })
}

#[derive(Deserialize)]
struct SignupForm {
    email: String,
    handle: String,
    password: String,
}

async fn signup_post(State(state): State<Arc<AppState>>, Form(form): Form<SignupForm>) -> Response {
    let email = form.email.trim().to_string();
    let handle = form.handle.trim().to_string();
    let password = form.password;

    if email.is_empty() || handle.is_empty() || password.is_empty() {
        return render(SignupTemplate {
            email,
            handle,
            err: Some("All fields are required.".into()),
            success: None,
        });
    }

    let resolved_handle = if handle.contains('.') {
        handle.clone()
    } else {
        format!("{handle}.tgirl.beauty")
    };

    let invite_code = match pds::create_invite_code(&state.http, &state.pds_admin_password).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "create_invite_code failed");
            return render(SignupTemplate {
                email,
                handle,
                err: Some(format!("Failed to create invite code: {e}")),
                success: None,
            });
        }
    };

    let state_token = uuid::Uuid::new_v4().simple().to_string();
    state.sessions.save(
        state_token.clone(),
        PendingSignup::new(resolved_handle.clone(), email.clone(), password, invite_code),
    );

    let callback = format!("{}/signup/callback", state.app_base_url.trim_end_matches('/'));
    let mut gate_url = match reqwest::Url::parse(&format!("{PDS_URL}/gate/signup")) {
        Ok(u) => u,
        Err(e) => {
            tracing::error!(error = %e, "gate URL parse failed");
            return render(SignupTemplate {
                email,
                handle,
                err: Some("Internal error preparing captcha redirect.".into()),
                success: None,
            });
        }
    };
    gate_url
        .query_pairs_mut()
        .append_pair("handle", &resolved_handle)
        .append_pair("state", &state_token)
        .append_pair("redirect_url", &callback);

    Redirect::to(gate_url.as_str()).into_response()
}

#[derive(Deserialize)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

async fn signup_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CallbackParams>,
) -> Response {
    if let Some(err) = params.error {
        return render(SignupTemplate {
            email: String::new(),
            handle: String::new(),
            err: Some(format!("Captcha failed: {err}")),
            success: None,
        });
    }

    let (Some(code), Some(state_token)) = (params.code, params.state) else {
        return render(SignupTemplate {
            email: String::new(),
            handle: String::new(),
            err: Some("Missing captcha response.".into()),
            success: None,
        });
    };

    let Some(pending) = state.sessions.take(&state_token) else {
        return render(SignupTemplate {
            email: String::new(),
            handle: String::new(),
            err: Some("Your signup session expired. Please start over.".into()),
            success: None,
        });
    };

    match pds::create_account(
        &state.http,
        CreateAccountInput {
            handle: &pending.handle,
            email: &pending.email,
            password: &pending.password,
            invite_code: &pending.invite_code,
            verification_code: &code,
        },
    )
    .await
    {
        Ok(()) => {
            let query = serde_urlencoded::to_string([("handle", &pending.handle)])
                .unwrap_or_default();
            Redirect::to(&format!("/signup/success?{query}")).into_response()
        }
        Err(msg) => render(SignupTemplate {
            email: pending.email,
            handle: pending.handle,
            err: Some(msg),
            success: None,
        }),
    }
}

#[derive(Deserialize)]
struct SuccessParams {
    handle: Option<String>,
}

async fn signup_success(Query(params): Query<SuccessParams>) -> Response {
    render(SuccessTemplate {
        handle: params.handle.unwrap_or_default(),
    })
}

fn render<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(body) => Html(body).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "template render failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "template error").into_response()
        }
    }
}
