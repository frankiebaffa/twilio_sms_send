use {
    reqwest::Method,
    reqwest::Client,
    reqwest::Proxy,
    serde::Serialize,
    serde_qs::to_string,
    slog::LogContext,
    std::env::var as env_var,
};
#[derive(Serialize)]
struct MessageRequest {
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "Body")]
    body: String,
}
const LOG_DIR: &'static str = "TWILIO_SEND_LOG_DIR";
const BASE_NAME: &'static str = "twilio_sms_send";
pub async fn send(to: String, from: String, body: String) -> bool {
    let ctx = LogContext::from_env(LOG_DIR, BASE_NAME);
    ctx.log(format!("Beginning send"));
    const SID_KEY: &'static str = "TWILIO_SEND_SID";
    const TOKEN_KEY: &'static str = "TWILIO_SEND_TOKEN";
    const PROXY_KEY: &'static str = "TWILIO_SEND_PROXY";
    let msg = MessageRequest { to, from , body };
    let req_body = match to_string(&msg) {
        Ok(r) => r,
        Err(e) => {
            ctx.error(format!("Failed to serialize MessageRequest: {}", e));
            return false;
        },
    };
    let sid = match env_var(SID_KEY) {
        Ok(s) => s,
        Err(e) => {
            ctx.error(format!(
                "Failed to retrieve environment variable {}: {}",
                SID_KEY, e
            ));
            return false;
        },
    };
    let token = match env_var(TOKEN_KEY) {
        Ok(t) => t,
        Err(e) => {
            ctx.error(format!(
                "Failed to retrieve environment variable {}: {}",
                TOKEN_KEY, e
            ));
            return false;
        },
    };
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages",
        sid
    );
    let proxy = match env_var(PROXY_KEY) {
        Ok(p) => Some(p),
        Err(e) => {
            ctx.log(format!(
                "No proxy defined in {} (not an error): {}", PROXY_KEY, e
            ));
            None
        },
    };
    ctx.log(format!("Building client"));
    let mut bld = Client::builder();
    match proxy {
        Some(p) => {
            ctx.log("Constructing proxy");
            let prox = match Proxy::all(p) {
                Ok(pr) => pr,
                Err(e) => {
                    ctx.error(format!("Failed to construct proxy: {}", e));
                    return false;
                },
            };
            bld = bld.proxy(prox);
        },
        _ => {},
    }
    ctx.log("Building request");
    let req = match bld.build() {
        Ok(r) => r,
        Err(e) => {
            ctx.error(format!("Failed to build request: {}", e));
            return false;
        },
    };
    ctx.log(format!("Sending request to {}", url));
    match req.request(Method::POST, url)
        .basic_auth(sid, Some(token))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
        .await
    {
        Ok(_) => {
            ctx.log("Request successful");
            true
        },
        Err(e) => {
            ctx.error(format!("Request failed: {}", e));
            false
        },
    }
}
