use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use p256::ecdsa::{SigningKey, signature::{Signer, Signature}};
use p256::pkcs8::DecodePrivateKey;
use sec1::DecodeEcPrivateKey;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::json;
use url::Url;
use rand::Rng;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    nbf: u64,
    exp: u64,
    sub: String,
    uri: String,
}

fn main() -> anyhow::Result<()> {
    // 1. Load Env
    let mut env_path = std::env::current_dir()?;
    loop {
        let candidate = env_path.join(".env");
        if candidate.exists() {
            let contents = std::fs::read_to_string(candidate)?;
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim();
                    let mut val = val.trim();
                    if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                        val = &val[1..val.len()-1];
                    }
                    if env::var(key).is_err() {
                        env::set_var(key, val);
                    }
                }
            }
            break;
        }
        if !env_path.pop() {
            anyhow::bail!("Could not find .env file");
        }
    }

    let api_key_name = env::var("COINBASE_API_KEY_NAME")?;
    let private_key_raw = env::var("COINBASE_PRIVATE_KEY")?;
    
    // Logic from coinbase.rs
    let private_key_pem = private_key_raw.replace("\\n", "\n");
    let private_key_pem = private_key_pem.trim();

    println!("Key Name: {}", api_key_name);
    println!("PEM Start: {:.20}...", private_key_pem);

    let method = "GET";
    let path = "/accounts";
    let base_url = "https://api.coinbase.com/api/v3/brokerage";
    
    // URI Construction
    let base_url_parsed = Url::parse(base_url)?;
    let host = base_url_parsed.host_str().unwrap_or("api.coinbase.com");
    let full_path = format!("{}{}", base_url_parsed.path().trim_end_matches('/'), path);
    let jwt_uri = format!("{} {}{}", method, host, full_path);

    println!("URI: {}", jwt_uri);

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    // 1. Header
    let nonce = format!("{:016x}", rand::thread_rng().gen::<u64>());
    let header = json!({
        "alg": "ES256",
        "kid": api_key_name,
        "nonce": nonce,
        "typ": "JWT"
    });

    // 2. Claims
    let claims = json!({
        "iss": "cdp",
        "nbf": now - 5,
        "exp": now + 120,
        "sub": api_key_name,
        "uri": jwt_uri,
    });

    println!("Header: {}", serde_json::to_string_pretty(&header)?);
    println!("Claims: {}", serde_json::to_string_pretty(&claims)?);

    // 3. Encode
    let header_json = serde_json::to_string(&header)?;
    let claims_json = serde_json::to_string(&claims)?;
    
    let header_b64 = URL_SAFE_NO_PAD.encode(header_json);
    let claims_b64 = URL_SAFE_NO_PAD.encode(claims_json);
    
    let message = format!("{}.{}", header_b64, claims_b64);

    // 4. Sign
    let signing_key = SigningKey::from_sec1_pem(private_key_pem)
        .or_else(|_| SigningKey::from_pkcs8_pem(private_key_pem))
        .expect("Failed to parse key");
        
    let signature: p256::ecdsa::Signature = signing_key.sign(message.as_bytes());
    let sig_bytes = signature.as_ref();
    println!("Signature Length: {}", sig_bytes.len());
    let signature_b64 = URL_SAFE_NO_PAD.encode(sig_bytes);
    
    let jwt = format!("{}.{}", message, signature_b64);
    
    println!("Generated JWT: {}", jwt);

    println!("Attempting CURL request...");
    let output = std::process::Command::new("curl")
        .arg("https://api.coinbase.com/api/v3/brokerage/accounts")
        .arg("-H")
        .arg(format!("Authorization: Bearer {}", jwt))
        .arg("-v")
        .output()?;
    
    println!("CURL Status: {}", output.status);
    println!("CURL Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("CURL Stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}
