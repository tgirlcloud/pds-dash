use anyhow::{anyhow, Context, Result};
use base64::Engine;
use jacquard::api::app_bsky::actor::profile::Profile;
use jacquard::api::com_atproto::repo::describe_repo::DescribeRepo;
use jacquard::api::com_atproto::repo::get_record::GetRecord;
use jacquard::api::com_atproto::server::create_account::CreateAccount;
use jacquard::api::com_atproto::server::create_invite_code::CreateInviteCode;
use jacquard::api::com_atproto::sync::list_repos::ListRepos;
use jacquard::deps::fluent_uri::Uri;
use jacquard::types::string::{AtIdentifier, Handle, Nsid, RecordKey};
use jacquard::types::value::from_data;
use jacquard::xrpc::XrpcExt;
use jacquard::CowStr;

pub const PDS_URL: &str = "https://pds.tgirl.cloud";

fn base_uri() -> Uri<String> {
    Uri::parse(PDS_URL.to_string()).expect("valid PDS URL")
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub handle: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
}

pub async fn get_pds_version(client: &reqwest::Client) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct HealthResponse {
        version: String,
    }
    let res = client
        .get(format!("{PDS_URL}/xrpc/_health"))
        .send()
        .await
        .context("failed to call _health")?;
    let body: HealthResponse = res.json().await.context("health response was not JSON")?;
    Ok(body.version)
}

async fn list_repo_dids(client: &reqwest::Client) -> Result<Vec<String>> {
    let mut dids = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let req = ListRepos::new()
            .maybe_cursor(cursor.as_deref().map(CowStr::from))
            .build();
        let resp = client
            .xrpc(base_uri())
            .send(&req)
            .await
            .map_err(|e| anyhow!("listRepos failed: {e}"))?;
        let output = resp
            .into_output()
            .map_err(|e| anyhow!("listRepos decode failed: {e}"))?;

        dids.extend(output.repos.into_iter().map(|r| r.did.as_str().to_string()));

        match output.cursor {
            Some(c) => cursor = Some(c.as_ref().to_string()),
            None => break,
        }
    }
    Ok(dids)
}

async fn get_user_profile(client: &reqwest::Client, did: &str) -> Result<UserProfile> {
    let repo_ident = AtIdentifier::new_owned(did).context("invalid DID for describeRepo")?;
    let describe_req = DescribeRepo::new().repo(repo_ident).build();
    let describe_resp = client
        .xrpc(base_uri())
        .send(&describe_req)
        .await
        .map_err(|e| anyhow!("describeRepo failed: {e}"))?;
    let describe = describe_resp
        .into_output()
        .map_err(|e| anyhow!("describeRepo decode failed: {e}"))?;

    let collection = Nsid::new_owned("app.bsky.actor.profile").expect("valid nsid");
    let rkey = RecordKey::any("self").expect("valid rkey");
    let profile_req = GetRecord::new()
        .repo(AtIdentifier::new_owned(did).expect("did already validated"))
        .collection(collection)
        .rkey(rkey)
        .build();

    let (display_name, avatar) = match client.xrpc(base_uri()).send(&profile_req).await {
        Ok(resp) => match resp.into_output() {
            Ok(record) => {
                let profile: Profile<'_> = from_data(&record.value)
                    .map_err(|e| anyhow!("profile decode failed: {e}"))?;
                let display_name = profile.display_name.map(|s| s.as_ref().to_string());
                let avatar = profile.avatar.map(|blob_ref| {
                    let cid = blob_ref.blob().cid().as_str();
                    format!("https://cdn.bsky.app/img/feed_thumbnail/plain/{did}/{cid}")
                });
                (display_name, avatar)
            }
            Err(e) => {
                tracing::warn!(did = %did, error = %e, "profile record decode failed");
                (None, None)
            }
        },
        Err(e) => {
            tracing::warn!(did = %did, error = %e, "getRecord failed");
            (None, None)
        }
    };

    Ok(UserProfile {
        handle: describe.handle.as_str().to_string(),
        display_name,
        avatar,
    })
}

pub async fn get_pds_users(client: &reqwest::Client) -> Result<Vec<UserProfile>> {
    let dids = list_repo_dids(client).await?;
    let mut profiles = Vec::with_capacity(dids.len());
    for did in dids {
        match get_user_profile(client, &did).await {
            Ok(p) => profiles.push(p),
            Err(e) => tracing::warn!(did = %did, error = %e, "failed to load profile"),
        }
    }
    Ok(profiles)
}

pub async fn create_invite_code(client: &reqwest::Client, admin_password: &str) -> Result<String> {
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("admin:{admin_password}"));
    let auth_value = reqwest::header::HeaderValue::from_str(&format!("Basic {auth}"))
        .context("admin auth header invalid")?;

    let req = CreateInviteCode::new().use_count(1).build();
    let resp = client
        .xrpc(base_uri())
        .header(reqwest::header::AUTHORIZATION, auth_value)
        .send(&req)
        .await
        .map_err(|e| anyhow!("createInviteCode request failed: {e}"))?;
    let output = resp
        .into_output()
        .map_err(|e| anyhow!("createInviteCode decode failed: {e}"))?;
    Ok(output.code.as_ref().to_string())
}

pub struct CreateAccountInput<'a> {
    pub handle: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub invite_code: &'a str,
    pub verification_code: &'a str,
}

pub async fn create_account(
    client: &reqwest::Client,
    input: CreateAccountInput<'_>,
) -> Result<(), String> {
    let resolved_handle = if input.handle.contains('.') {
        input.handle.to_string()
    } else {
        format!("{}.tgirl.beauty", input.handle)
    };

    let handle =
        Handle::new_owned(&resolved_handle).map_err(|e| format!("invalid handle: {e}"))?;

    let req = CreateAccount::new()
        .handle(handle)
        .email(CowStr::from(input.email.to_string()))
        .password(CowStr::from(input.password.to_string()))
        .invite_code(CowStr::from(input.invite_code.to_string()))
        .verification_code(CowStr::from(input.verification_code.to_string()))
        .build();

    let resp = client
        .xrpc(base_uri())
        .send(&req)
        .await
        .map_err(|e| format!("createAccount transport error: {e}"))?;

    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }

    #[derive(serde::Deserialize)]
    struct PdsError {
        error: Option<String>,
        message: Option<String>,
    }

    let body_bytes = resp.buffer();
    let body_text = std::str::from_utf8(body_bytes).unwrap_or("<non-utf8 body>");
    let parsed: Option<PdsError> = serde_json::from_slice(body_bytes).ok();

    tracing::error!(
        status = %status,
        handle = %resolved_handle,
        email = %input.email,
        body = %body_text,
        "createAccount failed",
    );

    let message = match parsed {
        Some(PdsError { error: Some(code), message: Some(msg) }) => format!("{code}: {msg}"),
        Some(PdsError { error: Some(code), message: None }) => code,
        Some(PdsError { error: None, message: Some(msg) }) => msg,
        _ => format!("PDS returned {status}: {body_text}"),
    };
    Err(message)
}
