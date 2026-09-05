use crate::easyuser::*;
use anyhow::{Error, Result, anyhow};
use rss::*;
use serde_json::Value;
use std::collections::HashMap;

const API: &str = "https://api.github.com/advisories";

use crate::router::github_common::rest_get;
fn render_markdown(s: &str) -> String {
    let mut html = String::new();
    let mut in_list = false;
    for line in s.lines() {
        let line = line.trim_end();
        let t = line.trim();
        if t.is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            continue;
        }
        if let Some(rest) = t.strip_prefix("### ") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<h3>{}</h3>", rest));
        } else if let Some(rest) = t.strip_prefix("## ") {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<h2>{}</h2>", rest));
        } else if let Some(rest) = t.strip_prefix("- ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>", rest));
        } else {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&format!("<p>{}</p>", t));
        }
    }
    if in_list {
        html.push_str("</ul>");
    }
    html
}

///GitHub Advisory Database RSS via REST API `GET /advisories`.
///Params: type (reviewed/unreviewed, default reviewed), ecosystem (composer/go/maven/npm/nuget/pip/pub/rubygems/rust/erlang/actions/swift, default all), limit (default 20, max 50)
pub async fn get(para: HashMap<String, String>) -> Result<String, Error> {
    let route_type = para
        .get("type")
        .cloned()
        .unwrap_or_else(|| "reviewed".to_string());
    let ecosystem = para.get("ecosystem").cloned().unwrap_or_default();
    let limit = para
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20)
        .min(50);

    let mut url = format!("{}?type={}&per_page={}", API, route_type, limit);
    if !ecosystem.is_empty() {
        url.push_str(&format!("&ecosystem={}", ecosystem));
    }

    let json: Value = rest_get(&url).await?;
    let advisories = json
        .as_array()
        .ok_or_else(|| anyhow!("GitHub API returned unexpected response"))?;

    let mut item_vec = Vec::new();
    for adv in advisories {
        let ghsa = adv["ghsa_id"].as_str().unwrap_or("");
        let summary = adv["summary"].as_str().unwrap_or("").to_string();
        let cve = adv["cve_id"].as_str().unwrap_or("");
        let severity = adv["severity"].as_str().unwrap_or("unknown");
        let html_url = adv["html_url"].as_str().unwrap_or("");
        let published = adv["published_at"].as_str().unwrap_or("");
        let description = adv["description"].as_str().unwrap_or("").to_string();
        let cvss = adv["cvss"]["score"].as_f64().unwrap_or(0.0);
        let mut references: Vec<String> = Vec::new();
        if let Some(refs) = adv["references"].as_array() {
            for r in refs {
                if let Some(u) = r.as_str() {
                    references.push(u.to_string());
                }
            }
        }
        let mut pkgs: Vec<String> = Vec::new();
        if let Some(vulns) = adv["vulnerabilities"].as_array() {
            for v in vulns {
                let pkg = v["package"]["name"].as_str().unwrap_or("");
                let range = v["vulnerable_version_range"].as_str().unwrap_or("");
                if !pkg.is_empty() {
                    pkgs.push(format!("{} ({})", pkg, range));
                }
            }
        }

        let mut desc = format!(
            "GHSA: {}<br>CVE: {}<br>Severity: {}<br>CVSS: {:.1}<br>Published: {}<br>",
            ghsa, cve, severity, cvss, published
        );
        if !pkgs.is_empty() {
            desc.push_str(&format!("Affected: {}<br>", pkgs.join(", ")));
        }
        if !description.is_empty() {
            desc.push_str(&render_markdown(&description));
        }
        if !references.is_empty() {
            desc.push_str("<br>References:");
            for r in &references {
                desc.push_str(&format!("<br><a href=\"{}\">{}</a>", r, r));
            }
        }

        let title = if summary.is_empty() {
            ghsa.to_string()
        } else {
            format!("[{}] {}", severity.to_uppercase(), summary)
        };

        let pub_date = rfc3339_to_rss(published);

        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(html_url.to_string())
                .description(Some(desc))
                .pub_date(pub_date)
                .guid(if ghsa.is_empty() {
                    rss::Guid {
                        value: html_url.to_string(),
                        permalink: false,
                    }
                } else {
                    rss::Guid {
                        value: ghsa.to_string(),
                        permalink: false,
                    }
                })
                .author(Some("GitHub Advisory Database".to_string()))
                .build(),
        );
    }

    if item_vec.is_empty() {
        return Err(anyhow!("No advisories found"));
    }

    let channel = ChannelBuilder::default()
        .title(format!(
            "GitHub Advisory Database - {}{}",
            route_type,
            if ecosystem.is_empty() {
                String::new()
            } else {
                format!(" - {}", ecosystem)
            }
        ))
        .link("https://github.com/advisories")
        .description("Security advisories from the GitHub Advisory Database")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
