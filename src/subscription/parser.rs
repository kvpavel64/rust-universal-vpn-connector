//! Default parser for common share links and sing-box JSON outbounds.
use std::collections::HashMap;

use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD},
    Engine,
};
use serde_json::{json, Map, Value};
use url::Url;

use super::{Node, Protocol};
use crate::prelude::contracts_prelude::*;
use crate::prelude::error_prelude::*;

/// Parser for VLESS, VMess, Trojan, Shadowsocks, SOCKS5, HTTP, and Hysteria2 links.
///
/// Also accepts a sing-box JSON configuration containing an `outbounds` array. This
/// parser intentionally rejects malformed lines instead of silently dropping them.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultSubscriptionParser;

impl SubscriptionParser for DefaultSubscriptionParser {
    fn parse(&self, document: &str) -> Result<Vec<Node>> {
        parse_subscription(document)
    }
}

fn parse_subscription(input: &str) -> Result<Vec<Node>> {
    let text = decode_base64(input.trim()).unwrap_or_else(|| input.trim().to_owned());
    if let Ok(value) = serde_json::from_str::<Value>(&text) {
        if let Some(outbounds) = value.get("outbounds").and_then(Value::as_array) {
            let mut nodes = Vec::new();
            for outbound in outbounds {
                let Some(kind) = outbound.get("type").and_then(Value::as_str) else {
                    continue;
                };
                if let Some(protocol) = protocol_from_str(kind) {
                    let mut normalized = outbound.clone();
                    if protocol == Protocol::Socks5 {
                        normalized["type"] = json!("socks");
                    }
                    nodes.push(Node {
                        name: outbound
                            .get("tag")
                            .and_then(Value::as_str)
                            .unwrap_or(kind)
                            .to_owned(),
                        protocol,
                        outbound: normalized,
                    });
                }
            }
            return if nodes.is_empty() {
                Err(Error::EmptySubscription)
            } else {
                Ok(nodes)
            };
        }
    }

    let mut nodes = Vec::new();
    let mut invalid = Vec::new();
    for (line_no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match parse_uri(line) {
            Ok(node) => nodes.push(node),
            Err(_) => invalid.push(line_no + 1),
        }
    }
    if !nodes.is_empty() {
        Ok(nodes)
    } else if !invalid.is_empty() {
        Err(Error::InvalidNode(format!(
            "invalid subscription entries at lines {invalid:?}"
        )))
    } else {
        Err(Error::EmptySubscription)
    }
}

fn protocol_from_str(s: &str) -> Option<Protocol> {
    match s.to_ascii_lowercase().as_str() {
        "vless" => Some(Protocol::Vless),
        "vmess" => Some(Protocol::Vmess),
        "trojan" => Some(Protocol::Trojan),
        "shadowsocks" | "ss" => Some(Protocol::Shadowsocks),
        "socks" | "socks5" => Some(Protocol::Socks5),
        "http" | "https" => Some(Protocol::Http),
        "hysteria2" | "hy2" => Some(Protocol::Hysteria2),
        _ => None,
    }
}

fn parse_uri(line: &str) -> Result<Node> {
    if line.to_ascii_lowercase().starts_with("vmess://") {
        return parse_vmess(line);
    }
    let url = Url::parse(line).map_err(|_| Error::InvalidNode(line.to_owned()))?;
    let scheme = url.scheme().to_ascii_lowercase();
    let protocol = protocol_from_str(&scheme).ok_or_else(|| Error::InvalidNode(line.to_owned()))?;
    let host = url
        .host_str()
        .ok_or_else(|| Error::InvalidNode(line.to_owned()))?;
    let default_port = match protocol {
        Protocol::Socks5 => 1080,
        Protocol::Http if scheme == "https" => 443,
        Protocol::Http => 80,
        _ => 443,
    };
    let port = url.port().unwrap_or(default_port);
    let name = url
        .fragment()
        .map(percent_decode)
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("{scheme}://{host}:{port}"));
    let q: HashMap<String, String> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    let mut out = Map::new();
    out.insert("type".into(), json!(protocol.as_str()));
    out.insert("tag".into(), json!(name));
    out.insert("server".into(), json!(host));
    out.insert("server_port".into(), json!(port));

    match protocol {
        Protocol::Vless => {
            if url.username().is_empty() {
                return Err(Error::InvalidNode(line.to_owned()));
            }
            out.insert("uuid".into(), json!(url.username()));
            if let Some(flow) = q.get("flow") {
                out.insert("flow".into(), json!(flow));
            }
            if q.get("encryption").is_some_and(|v| v != "none") {
                return Err(Error::InvalidNode("VLESS encryption must be none".into()));
            }
            add_tls(&mut out, &q, false);
            add_transport(&mut out, &q);
        }
        Protocol::Trojan => {
            let password = url.password().unwrap_or(url.username());
            if password.is_empty() {
                return Err(Error::InvalidNode(line.to_owned()));
            }
            out.insert("password".into(), json!(password));
            add_tls(&mut out, &q, true);
            add_transport(&mut out, &q);
        }
        Protocol::Shadowsocks => {
            let userinfo = if !url.username().is_empty() {
                decode_base64(url.username()).unwrap_or_else(|| {
                    format!("{}:{}", url.username(), url.password().unwrap_or(""))
                })
            } else {
                decode_base64(url.path().trim_start_matches('/'))
                    .ok_or_else(|| Error::InvalidNode(line.to_owned()))?
            };
            let credentials = userinfo.split('@').next().unwrap_or(&userinfo);
            let (method, password) = credentials
                .split_once(':')
                .ok_or_else(|| Error::InvalidNode(line.to_owned()))?;
            out.insert("method".into(), json!(method));
            out.insert("password".into(), json!(password));
        }
        Protocol::Hysteria2 => {
            let password = if url.username().is_empty() {
                url.password().unwrap_or("")
            } else {
                url.username()
            };
            if password.is_empty() {
                return Err(Error::InvalidNode(line.to_owned()));
            }
            out.insert("password".into(), json!(password));
            add_tls(&mut out, &q, true);
            if let Some(obfs) = q.get("obfs") {
                out.insert("obfs".into(), json!({"type":obfs,"password":q.get("obfs-password").cloned().unwrap_or_default()}));
            }
            for (source, target) in [("upmbps", "up_mbps"), ("downmbps", "down_mbps")] {
                if let Some(value) = q.get(source).and_then(|v| v.parse::<u64>().ok()) {
                    out.insert(target.into(), json!(value));
                }
            }
        }
        Protocol::Socks5 | Protocol::Http => {
            if !url.username().is_empty() {
                out.insert("username".into(), json!(url.username()));
            }
            if let Some(password) = url.password() {
                out.insert("password".into(), json!(password));
            }
            if scheme == "https" || q.get("security").is_some_and(|v| v == "tls") {
                add_tls(&mut out, &q, true);
            }
        }
        Protocol::Vmess => unreachable!(),
    }
    Ok(Node {
        name,
        protocol,
        outbound: Value::Object(out),
    })
}

fn parse_vmess(line: &str) -> Result<Node> {
    let payload = line
        .strip_prefix("vmess://")
        .ok_or_else(|| Error::InvalidNode(line.into()))?;
    let decoded = decode_base64(payload).ok_or_else(|| Error::InvalidNode(line.into()))?;
    let v: Value = serde_json::from_str(&decoded)?;
    let get = |keys: &[&str]| -> String {
        keys.iter()
            .find_map(|k| v.get(*k).and_then(Value::as_str))
            .unwrap_or("")
            .to_owned()
    };
    let host = get(&["add", "address"]);
    let port = get(&["port"])
        .parse::<u16>()
        .map_err(|_| Error::InvalidNode(line.into()))?;
    let uuid = get(&["id", "uuid"]);
    if host.is_empty() || uuid.is_empty() || port == 0 {
        return Err(Error::InvalidNode(line.into()));
    }
    let mut out = json!({"type":"vmess","tag":"vmess","server":host,"server_port":port,"uuid":uuid,"security":get(&["scy","security"])});
    if let Some(obj) = out.as_object_mut() {
        obj.insert(
            "alter_id".into(),
            json!(get(&["aid"]).parse::<u32>().unwrap_or(0)),
        );
        let mut q = HashMap::new();
        q.insert("security".into(), get(&["tls"]));
        q.insert("sni".into(), get(&["sni", "host"]));
        q.insert("type".into(), get(&["net", "type"]));
        q.insert("path".into(), get(&["path"]));
        q.insert("host".into(), get(&["host"]));
        add_tls(obj, &q, false);
        add_transport(obj, &q);
    }
    let name = get(&["ps", "name"]);
    Ok(Node {
        name: if name.is_empty() {
            format!("vmess://{host}:{port}")
        } else {
            name
        },
        protocol: Protocol::Vmess,
        outbound: out,
    })
}

fn add_tls(out: &mut Map<String, Value>, q: &HashMap<String, String>, default_on: bool) {
    let security = q.get("security").map(String::as_str).unwrap_or("");
    let enabled =
        matches!(security, "tls" | "reality" | "1") || (default_on && security.is_empty());
    if !enabled {
        return;
    }
    let mut tls = json!({"enabled":true});
    if let Some(sni) = q.get("sni").or_else(|| q.get("peer")) {
        tls["server_name"] = json!(sni);
    }
    if q.get("allowInsecure")
        .is_some_and(|v| v == "1" || v == "true")
    {
        tls["insecure"] = json!(true);
    }
    if let Some(fp) = q.get("fp") {
        tls["utls"] = json!({"enabled":true,"fingerprint":fp});
    }
    if security == "reality" {
        let mut reality = json!({"enabled":true});
        if let Some(key) = q.get("pbk") {
            reality["public_key"] = json!(key);
        }
        if let Some(id) = q.get("sid") {
            reality["short_id"] = json!(id);
        }
        tls["reality"] = reality;
    }
    if let Some(alpn) = q.get("alpn") {
        tls["alpn"] = json!(alpn.split(',').collect::<Vec<_>>());
    }
    out.insert("tls".into(), tls);
}

fn add_transport(out: &mut Map<String, Value>, q: &HashMap<String, String>) {
    match q.get("type").map(String::as_str).unwrap_or("tcp") {
        "ws" => {
            let mut t = json!({"type":"ws"});
            if let Some(p) = q.get("path") {
                t["path"] = json!(p);
            }
            if let Some(h) = q.get("host") {
                t["headers"] = json!({"Host":h});
            }
            out.insert("transport".into(), t);
        }
        "grpc" => {
            let mut t = json!({"type":"grpc"});
            if let Some(s) = q.get("serviceName") {
                t["service_name"] = json!(s);
            }
            out.insert("transport".into(), t);
        }
        "http" | "h2" => {
            let mut t = json!({"type":"http"});
            if let Some(p) = q.get("path") {
                t["path"] = json!([p]);
            }
            if let Some(h) = q.get("host") {
                t["host"] = json!([h]);
            }
            out.insert("transport".into(), t);
        }
        "httpupgrade" => {
            let mut t = json!({"type":"httpupgrade"});
            if let Some(p) = q.get("path") {
                t["path"] = json!(p);
            }
            if let Some(h) = q.get("host") {
                t["host"] = json!(h);
            }
            out.insert("transport".into(), t);
        }
        _ => {}
    }
}

fn decode_base64(input: &str) -> Option<String> {
    let value = input.trim();
    STANDARD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .or_else(|_| URL_SAFE_NO_PAD.decode(value.trim_end_matches('=')))
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}
fn percent_decode(value: &str) -> String {
    url::form_urlencoded::parse(format!("v={value}").as_bytes())
        .next()
        .map(|(_, v)| v.into_owned())
        .unwrap_or_else(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_vless_with_tls_and_label() {
        let nodes = parse_subscription("vless://00000000-0000-0000-0000-000000000000@example.com:443?encryption=none&security=tls&sni=example.com#test").unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].protocol, Protocol::Vless);
        assert_eq!(nodes[0].outbound["server"], "example.com");
        assert_eq!(nodes[0].name, "test");
    }

    #[test]
    fn parses_trojan_and_hysteria2() {
        let doc =
            "trojan://secret@example.org:443#t\nhy2://secret@example.net:443?sni=cdn.example.net#h";
        let nodes = parse_subscription(doc).unwrap();
        assert_eq!(nodes[0].outbound["password"], "secret");
        assert_eq!(nodes[1].protocol, Protocol::Hysteria2);
    }

    #[test]
    fn parses_vmess_json_payload() {
        let payload = r#"{"add":"vmess.example","port":"443","id":"00000000-0000-0000-0000-000000000000","ps":"node","net":"ws","path":"/ws","tls":"tls"}"#;
        let link = format!("vmess://{}", STANDARD.encode(payload));
        let nodes = parse_subscription(&link).unwrap();
        assert_eq!(nodes[0].protocol, Protocol::Vmess);
        assert_eq!(nodes[0].outbound["transport"]["type"], "ws");
    }

    #[test]
    fn parses_sing_box_json_outbounds() {
        let json = r#"{"outbounds":[{"type":"socks","tag":"local-upstream","server":"127.0.0.1","server_port":1080}]}"#;
        assert_eq!(
            parse_subscription(json).unwrap()[0].protocol,
            Protocol::Socks5
        );
    }

    #[test]
    fn parses_shadowsocks_socks5_http_and_https() {
        let credentials = URL_SAFE_NO_PAD.encode("aes-128-gcm:secret");
        let doc = format!("ss://{credentials}@ss.example:8388#ss\nsocks5://user:pass@socks.example:1080#socks\nhttp://user:pass@http.example:8080#http\nhttps://user:pass@secure.example:443#https");
        let nodes = parse_subscription(&doc).unwrap();
        assert_eq!(nodes.len(), 4);
        assert_eq!(nodes[0].protocol, Protocol::Shadowsocks);
        assert_eq!(nodes[1].outbound["server_port"], 1080);
        assert_eq!(nodes[2].protocol, Protocol::Http);
        assert_eq!(nodes[3].outbound["tls"]["enabled"], true);
    }

    #[test]
    fn parses_base64_wrapped_subscription() {
        let raw = "trojan://secret@example.org:443#edge";
        let wrapped = STANDARD.encode(raw);
        assert_eq!(
            parse_subscription(&wrapped).unwrap()[0].protocol,
            Protocol::Trojan
        );
    }

    #[test]
    fn rejects_empty_or_invalid_documents() {
        assert!(matches!(
            parse_subscription("\n# comment"),
            Err(Error::EmptySubscription)
        ));
        assert!(parse_subscription("not-a-proxy-uri").is_err());
    }
}
