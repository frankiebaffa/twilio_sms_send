use {
    chrono::Utc,
    reqwest::Method,
    reqwest::blocking::Client,
    reqwest::Proxy,
    serde::Serialize,
    serde_qs::to_string,
    std::{
        env::var as env_var,
        fs::OpenOptions,
        io::Write,
        path::PathBuf,
    },
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
enum LogType {
    Log,
    Error,
}
impl LogType {
    fn to_str(&self) -> String {
        match self {
            Self::Log => {
                format!("LOG  ")
            },
            Self::Error => {
                format!("ERROR")
            },
        }
    }
}
fn write_to_log(log_type: LogType, msg: impl AsRef<str>) {
    const LOG_DIR: &'static str = "TWILIO_SEND_LOG_DIR";
    const BASE_NAME: &'static str = "twilio_sms_send";
    let now = Utc::now();
    let now_short_fmt = now.format("%Y%m%d");
    let now_long_fmt = now.format("%+");
    let log_dir = match env_var(LOG_DIR) {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to find environment variable for log: {}", e);
            return;
        },
    };
    let mut path = PathBuf::from(log_dir);
    let file_name = format!("{}.{}.log", BASE_NAME, now_short_fmt);
    path.push(file_name);
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .read(false)
        .append(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open {} for writing: {}", path.to_str().unwrap(), e);
            return;
        },
    };
    match file.write_all(
        format!("{} {}: {}", log_type.to_str(), now_long_fmt, msg.as_ref())
            .as_bytes()
    ) {
        Ok(_) => {},
        Err(e) => {
            println!("Failed to write to {}: {}", path.to_str().unwrap(), e);
            return;
        },
    }
}
fn log(msg: impl AsRef<str>) {
    write_to_log(LogType::Log, msg);
}
fn error(msg: impl AsRef<str>) {
    write_to_log(LogType::Error, msg);
}
pub fn send(to: String, from: String, body: String) -> bool {
    log(format!("Beginning send"));
    const SID_KEY: &'static str = "TWILIO_SEND_SID";
    const TOKEN_KEY: &'static str = "TWILIO_SEND_TOKEN";
    const PROXY_KEY: &'static str = "TWILIO_SEND_PROXY";
    let msg = MessageRequest { to, from , body };
    let req_body = match to_string(&msg) {
        Ok(r) => r,
        Err(e) => {
            error(format!("Failed to serialize MessageRequest: {}", e));
            return false;
        },
    };
    let sid = match env_var(SID_KEY) {
        Ok(s) => s,
        Err(e) => {
            error(format!(
                "Failed to retrieve environment variable {}: {}",
                SID_KEY, e
            ));
            return false;
        },
    };
    let token = match env_var(TOKEN_KEY) {
        Ok(t) => t,
        Err(e) => {
            error(format!(
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
            log(format!(
                "No proxy defined in {} (not an error): {}", PROXY_KEY, e
            ));
            None
        },
    };
    log(format!("Building client"));
    let mut bld = Client::builder();
    match proxy {
        Some(p) => {
            log("Constructing proxy");
            let prox = match Proxy::all(p) {
                Ok(pr) => pr,
                Err(e) => {
                    error(format!("Failed to construct proxy: {}", e));
                    return false;
                },
            };
            bld = bld.proxy(prox);
        },
        _ => {},
    }
    log("Building request");
    let req = match bld.build() {
        Ok(r) => r,
        Err(e) => {
            error(format!("Failed to build request: {}", e));
            return false;
        },
    };
    log(format!("Sending request to {}", url));
    match req.request(Method::POST, url)
        .basic_auth(sid, Some(token))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(req_body)
        .send()
    {
        Ok(_) => {
            log("Request successful");
            true
        },
        Err(e) => {
            error(format!("Request failed: {}", e));
            false
        },
    }
}
