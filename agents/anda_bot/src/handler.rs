use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};
use candid::Principal;
use ic_cose_types::to_cbor_bytes;
use ic_tee_agent::{
    http::{Content, ANONYMOUS_PRINCIPAL, HEADER_IC_TEE_CALLER},
    RPCRequest, RPCResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub x_status: Arc<RwLock<ServiceStatus>>,
    pub info: Arc<AppInformation>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ServiceStatus {
    #[default]
    Stopped,
    Running,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppInformation {
    pub id: Principal,
    pub name: String, // engine name
    pub start_time_ms: u64,
    pub default_agent: String,
    pub object_store_client: Option<Principal>,
    pub object_store_canister: Option<Principal>,
    pub caller: Principal,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppInformationJSON {
    pub id: String,   // TEE service id
    pub name: String, // engine name
    pub start_time_ms: u64,
    pub default_agent: String,
    pub object_store_client: Option<String>,
    pub object_store_canister: Option<String>,
    pub caller: String,
}

/// GET /.well-known/app
pub async fn get_information(State(app): State<AppState>, req: Request) -> impl IntoResponse {
    let mut info = app.info.as_ref().clone();
    let headers = req.headers();
    info.caller = if let Some(caller) = headers.get(&HEADER_IC_TEE_CALLER) {
        if let Ok(caller) = Principal::from_text(caller.to_str().unwrap_or_default()) {
            caller
        } else {
            ANONYMOUS_PRINCIPAL
        }
    } else {
        ANONYMOUS_PRINCIPAL
    };

    match Content::from(req.headers()) {
        Content::CBOR(_, _) => Content::CBOR(info, None).into_response(),
        _ => Content::JSON(
            AppInformationJSON {
                id: info.id.to_string(),
                name: info.name,
                start_time_ms: info.start_time_ms,
                default_agent: info.default_agent.clone(),
                object_store_client: info.object_store_client.as_ref().map(|p| p.to_string()),
                object_store_canister: info.object_store_canister.as_ref().map(|p| p.to_string()),
                caller: info.caller.to_string(),
            },
            None,
        )
        .into_response(),
    }
}

pub async fn add_proposal(
    State(app): State<AppState>,
    ct: Content<RPCRequest>,
) -> impl IntoResponse {
    match ct {
        Content::CBOR(req, _) => {
            // TODO: add access control
            let res = handle_proposal(&req, &app).await;
            Content::CBOR(res, None).into_response()
        }
        _ => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
    }
}

async fn handle_proposal(req: &RPCRequest, app: &AppState) -> RPCResponse {
    match req.method.as_str() {
        "start_x_bot" => {
            let mut x_status = app.x_status.write().await;
            *x_status = ServiceStatus::Running;
            Ok(to_cbor_bytes(&"Ok").into())
        }
        "stop_x_bot" => {
            let mut x_status = app.x_status.write().await;
            *x_status = ServiceStatus::Stopped;
            Ok(to_cbor_bytes(&"Ok").into())
        }
        _ => Err(format!("unsupported method {}", req.method)),
    }
}
