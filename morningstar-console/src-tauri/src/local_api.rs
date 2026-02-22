use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::thread;

use tiny_http::{Header, Method, Response, Server};

fn log_line(msg: &str) {
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(r"C:\temp\morningstar-console.log")
    {
        let _ = writeln!(f, "{msg}");
    }
}

fn json_header() -> Header {
    Header::from_bytes("Content-Type", "application/json").unwrap()
}

fn respond_json(request: tiny_http::Request, status: u16, body: String) {
    let mut resp = Response::from_string(body).with_header(json_header());
    if status != 200 {
        resp = resp.with_status_code(status);
    }
    let _ = request.respond(resp);
}

fn call_openai(text: &str) -> Result<String, String> {
    // Load API key from environment
    let key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "missing_openai_key".to_string())?;

    if key.trim().is_empty() {
        return Err("missing_openai_key".to_string());
    }

    // Build request payload for Responses API
    let payload = serde_json::json!({
        "model": "gpt-5.2",
        "input": text
    });

    // Send HTTP request
    let response = ureq::post("https://api.openai.com/v1/responses")
        .set("Authorization", &format!("Bearer {}", key))
        .set("Content-Type", "application/json")
        .send_json(payload);

    match response {
        Ok(resp) => {
            let value: serde_json::Value = resp
                .into_json()
                .map_err(|_| "invalid_openai_response".to_string())?;

            // Extract output_text if present
            let answer = value
                .get("output_text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            Ok(answer)
        }
        Err(e) => Err(format!("openai_request_failed: {}", e)),
    }
}

pub fn start_local_api() {
    log_line("START_LOCAL_API CALLED");

    thread::spawn(|| {
        log_line("LOCAL_API THREAD SPAWNED");

        let server = match Server::http("127.0.0.1:17373") {
            Ok(s) => s,
            Err(e) => {
                log_line(&format!("FAILED TO BIND SERVER: {e}"));
                return;
            }

          

        };

        log_line("LOCAL_API SERVER BOUND OK on 127.0.0.1:17373");

        for mut request in server.incoming_requests() {
            let url = request.url().to_string();
            let method = request.method().clone();

            // --- GET /api/ping ---
            if method == Method::Get && url == "/api/ping" {
                respond_json(request, 200, r#"{"ok":true}"#.to_string());
                continue;
            }

            // --- POST /api/ask ---
            if method == Method::Post && url == "/api/ask" {
                // Read request body (limit to 64KB)
                let mut body_str = String::new();
                let read_result = request
                    .as_reader()
                    .take(64 * 1024)
                    .read_to_string(&mut body_str);

                if read_result.is_err() {
                    respond_json(request, 400, r#"{"error":"bad_request"}"#.to_string());
                    continue;
                }

                // Expect JSON: {"text":"..."}
                let text = match serde_json::from_str::<serde_json::Value>(&body_str)
                    .ok()
                    .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
                {
                    Some(t) if !t.trim().is_empty() => t,
                    _ => {
                        respond_json(request, 400, r#"{"error":"missing_text"}"#.to_string());
                        continue;
                    }
                };

                // Read OpenAI key from env (keep keys OUT of frontend)
                let key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
                if key.trim().is_empty() {
                    respond_json(
                        request,
                        500,
                        r#"{"error":"missing_openai_key"}"#.to_string(),
                    );
                    continue;
                }

                // NOTE: For now this is a stub response so you can confirm the route works.
                // Once this compiles and /api/ask responds, we’ll swap in the OpenAI call cleanly.
                log_line(&format!("ASK: {}", text.replace('\n', "\\n")));
                respond_json(
                    request,
                    200,
                    r#"{"answer":"Octavian online. Stub response working."}"#.to_string(),
                );
                continue;
            }

            // --- Default 404 ---
            respond_json(request, 404, r#"{"error":"not_found"}"#.to_string());
        }
    });
}