use {
    dotenv::dotenv,
    reqwest::Method,
    reqwest::blocking::Client,
    reqwest::Proxy,
    serde::Serialize,
    serde_qs::to_string,
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
pub fn send(to: String, from: String, body: String) -> bool {
    const SID_KEY: &'static str = "TWILIO_SEND_SID";
    const TOKEN_KEY: &'static str = "TWILIO_SEND_TOKEN";
    const PROXY_KEY: &'static str = "TWILIO_SEND_PROXY";
    match dotenv() {
        Err(_) => return false,
        _ => {},
    }
    let msg = MessageRequest { to, from , body };
    let req_body = match to_string(&msg) {
        Ok(r) => r,
        Err(_) => return false,
    };
    let sid = match env_var(SID_KEY) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let token = match env_var(TOKEN_KEY) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages",
        sid
    );
    let proxy = match env_var(PROXY_KEY) {
        Ok(p) => Some(p),
        Err(_) => None,
    };
    let mut bld = Client::builder();
    match proxy {
        Some(p) => {
            let prox = match Proxy::all(p) {
                Ok(pr) => pr,
                _ => return false,
            };
            bld = bld.proxy(prox);
        },
        _ => {},
    }
    let req = match bld.build() {
        Ok(r) => r,
        Err(_) => return false,
    };
    match req.request(Method::POST, url)
        .basic_auth(sid, Some(token))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
    {
        Ok(_) => true,
        Err(_) => false,
    }
}
