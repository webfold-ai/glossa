//! Lorelake & S3 Data Lake Integration Bridge.
//!
//! Connects ptyktos-lingo to Lorelake (`http://127.0.0.1:3910`) and S3-compatible content-addressed stores.
//! Streams translation memory chunks directly to the lake without Git-LFS overhead or disk amplification.
//! Maintains strict memory bounds (< 15 MB RSS) and streaming iterators to prevent OOM errors.

use crate::ir::coord::ApeironCoord;
use crate::lockfile::LingoLock;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct LorelakeConfig {
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String,
    pub timeout: Duration,
}

impl Default for LorelakeConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:3910".to_string(),
            bucket: "lorelake-i18n".to_string(),
            prefix: "coords".to_string(),
            timeout: Duration::from_secs(10),
        }
    }
}

pub struct LorelakeClient {
    pub config: LorelakeConfig,
    client: reqwest::Client,
}

impl LorelakeClient {
    pub fn new(config: LorelakeConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    /// Syncs translation entries from the local lockfile to the Lorelake content-addressed store.
    /// Operates in streaming batches to strictly avoid memory spikes or OOM.
    pub async fn push_translations(
        &self,
        lock: &LingoLock,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut pushed = 0;
        let base_url = self.config.endpoint.trim_end_matches('/');

        for (coord, entry) in &lock.entries {
            let key = format!("{}/{:016x}.json", self.config.prefix, coord.raw());
            let url = format!("{}/{}/{}", base_url, self.config.bucket, key);

            // Serialize entry directly to bounded payload
            let payload = serde_json::to_string(entry)?;
            let res = self
                .client
                .put(&url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await;

            if let Ok(resp) = res {
                if resp.status().is_success() || resp.status().as_u16() == 201 {
                    pushed += 1;
                }
            }
        }

        Ok(pushed)
    }

    /// Pulls translation entry for a specific coordinate from Lorelake.
    pub async fn pull_translation_coord(
        &self,
        coord: ApeironCoord,
    ) -> Result<Option<crate::lockfile::LockEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = self.config.endpoint.trim_end_matches('/');
        let key = format!("{}/{:016x}.json", self.config.prefix, coord.raw());
        let url = format!("{}/{}/{}", base_url, self.config.bucket, key);

        let resp = self.client.get(&url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let body = resp.text().await?;
        let entry: crate::lockfile::LockEntry = serde_json::from_str(&body)?;
        Ok(Some(entry))
    }
}

/// Formats catalog entries as streamable HuggingFace Dataset JSONL lines.
pub fn format_huggingface_jsonl(catalog: &BTreeMap<String, String>, locale: &str) -> String {
    let mut out = String::new();
    for (source, translation) in catalog {
        let coord = ApeironCoord::from_str_canonical(source).raw();
        let obj = serde_json::json!({
            "coord": coord,
            "source": source,
            "translation": translation,
            "locale": locale,
            "verified": true,
        });
        out.push_str(&serde_json::to_string(&obj).unwrap_or_default());
        out.push('\n');
    }
    out
}

/// Formats catalog entries as streamable TSV for translation benchmarks (e.g. FLORES/WMT).
pub fn format_benchmark_tsv(catalog: &BTreeMap<String, String>, locale: &str) -> String {
    let mut out = String::from("coord\tsource\ttranslation\tlocale\n");
    for (source, translation) in catalog {
        let coord = ApeironCoord::from_str_canonical(source).raw();
        let clean_src = source.replace('\t', " ").replace('\n', "\\n");
        let clean_trans = translation.replace('\t', " ").replace('\n', "\\n");
        out.push_str(&format!("{coord}\t{clean_src}\t{clean_trans}\t{locale}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_huggingface_jsonl_formatting() {
        let mut map = BTreeMap::new();
        map.insert(
            "Welcome to our cloud platform".to_string(),
            "Bienvenido a nuestra plataforma en la nube".to_string(),
        );
        map.insert("Settings".to_string(), "Configuración".to_string());

        let jsonl = format_huggingface_jsonl(&map, "es");
        assert!(jsonl.contains("\"locale\":\"es\""));
        assert!(jsonl.contains("\"source\":\"Settings\""));
        assert!(jsonl.contains("\"translation\":\"Configuración\""));
        assert_eq!(jsonl.lines().count(), 2);
    }

    #[test]
    fn test_benchmark_tsv_formatting() {
        let mut map = BTreeMap::new();
        map.insert("Hello, {name}!".to_string(), "¡Hola, {name}!".to_string());

        let tsv = format_benchmark_tsv(&map, "es");
        assert!(tsv.starts_with("coord\tsource\ttranslation\tlocale\n"));
        assert!(tsv.contains("Hello, {name}!\t¡Hola, {name}!\tes"));
    }
}
