//! CLI Commands and Execution Pipelines.
//!
//! Command-line interface: extraction, translation, checks, catalog export, and helpers.

use crate::config::LingoConfig;
use crate::emitters::{resource, TargetFramework};
use crate::engine::providers::anthropic::AnthropicProvider;
use crate::engine::providers::offline::OfflineProvider;
use crate::engine::providers::ollama::OllamaProvider;
use crate::engine::providers::openai::OpenAiProvider;
use crate::engine::pseudo::pseudo_localize;
use crate::engine::{TranslationEngine, TranslationProvider};
use crate::lockfile::LingoLock;
use crate::parsers::extract_directory;
use clap::{Parser, Subcommand};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    name = "glossa",
    author,
    version,
    about = "Multi-framework localization tool: extract strings, translate missing entries, verify translations, and write locale catalogs."
)]
pub struct Cli {
    #[arg(short, long, default_value = ".")]
    pub dir: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize the project configuration (glossa.json, or an existing lingo.json) with framework detection
    Init {
        #[arg(short, long)]
        framework: Option<String>,
        #[arg(short, long)]
        locales: Option<String>,
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Extract translatable strings across any framework into the unified catalog
    Extract {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Translate missing entries with the configured provider and check each result with the integrity gate
    Translate {
        #[arg(short, long)]
        locales: Option<String>,
        #[arg(short, long)]
        provider: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },

    /// Not implemented yet (accepts a target and exits with an error); use `codegen` for generated accessors
    Compile {
        #[arg(short, long)]
        target: String,
        #[arg(short, long)]
        in_place: bool,
    },

    /// Check localization coverage and validate placeholders for CI/CD
    Check {
        /// Enable CI mode (outputs GitHub Actions annotations and fails on missing keys)
        #[arg(long)]
        ci: bool,
        /// Output format: "text", "github", "json"
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show translation deltas and pending strings
    Diff,

    /// Generate pseudo-localized strings to test frontend UI clipping and UTF-8 support
    Pseudo {
        #[arg(short, long, default_value = "pseudo")]
        locale: String,
    },

    /// Sync lockfile translations into locale catalog files (JSON, YAML, PO, Fluent, Android XML, iOS, ARB, XLIFF)
    Sync,

    /// Run micro-benchmarks of the extraction, lockfile, gate and emitter code and print Glossa's own timings
    Bench {
        #[arg(short, long, default_value_t = 5000)]
        count: usize,
        /// Also print the report as JSON (with percentiles)
        #[arg(long)]
        spbench: bool,
        /// Output path for structured JSON benchmark report
        #[arg(long)]
        json_output: Option<PathBuf>,
    },

    /// Start the Model Context Protocol (MCP) server on stdio
    Mcp,

    /// Push lockfile entries to an HTTP object store (default endpoint http://127.0.0.1:3910); only `push` is implemented
    Lorelake {
        /// Action: "push" (other values are rejected)
        #[arg(short, long, default_value = "push")]
        action: String,
        /// Object store endpoint URL
        #[arg(short, long, default_value = "http://127.0.0.1:3910")]
        endpoint: String,
        /// Bucket name
        #[arg(short, long, default_value = "lorelake-i18n")]
        bucket: String,
    },

    /// Export translation dataset for HuggingFace Hub (JSONL) or FLORES/WMT benchmarks (TSV)
    Export {
        /// Output format: "hf" / "jsonl" (HuggingFace Dataset), or "tsv" (Translation Benchmark)
        #[arg(short, long, default_value = "hf")]
        format: String,
        /// Locale code to export
        #[arg(short, long)]
        locale: String,
        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate message accessor code with typed slot parameters (TypeScript or Rust)
    Codegen {
        /// Target language: "ts" / "typescript" or "rust"
        #[arg(short, long, default_value = "ts")]
        target: String,
        /// Output file path (defaults to src/i18n/messages.ts or src/i18n/messages.rs)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Launch the in-context visual editor (default port 3920)
    Visual {
        /// Local port to bind
        #[arg(short, long, default_value_t = 3920)]
        port: u16,
    },

    /// Detect and import locale settings from Lingo.dev, Inlang, Tolgee, Crowdin, Lokalise or i18next config files
    Import {
        /// Only import from one system: "lingo", "inlang", "tolgee", "crowdin", "lokalise", "i18next"
        #[arg(short, long)]
        from: Option<String>,
    },
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let root = cli.dir.canonicalize().unwrap_or(cli.dir);

    match cli.command {
        Commands::Init {
            framework,
            locales,
            format,
        } => {
            let mut cfg = LingoConfig::load_or_default(&root);

            // Auto-detect and import incumbent configurations if present
            let detected_imports = crate::importer::detect_incumbent_systems(&root);
            for imp in &detected_imports {
                println!("Detected existing {} config at {}; importing its settings.", imp.system.name(), imp.config_file.display());
                crate::importer::import_to_glossa_config(imp, &mut cfg);
            }

            if let Some(fw) = framework {
                cfg.framework = Some(fw);
            } else if let Some(detected) = LingoConfig::detect_framework_name(&root) {
                println!("🔍 Auto-detected framework/ecosystem: {detected}");
                cfg.framework = Some(detected);
            }

            if let Some(locs) = locales {
                cfg.target_locales = locs.split(',').map(|s| s.trim().to_string()).collect();
            }

            if let Some(fmt) = format {
                cfg.output_format = fmt;
            }

            let path = cfg.save_to_file(&root)?;
            println!("✨ Created config file at {}", path.display());
            println!("   Source locale:  {}", cfg.source_locale);
            println!("   Target locales: {:?}", cfg.target_locales);
            println!("   Output format:  {}", cfg.output_format);
            println!("   Framework:      {:?}", cfg.framework);
        }

        Commands::Extract { output: _ } => {
            let config = LingoConfig::load_or_default(&root);
            println!("Scanning project sources...");
            let start = Instant::now();

            let catalog = extract_directory(&root, &[])?;
            let elapsed = start.elapsed();

            println!("✓ Extracted {} translatable units in {:?}", catalog.len(), elapsed);

            let lock_path = LingoLock::resolve_path(&root);
            let mut lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();
            lock.sync_catalog(&catalog);
            lock.save_to_file(&lock_path)?;

            println!("🔒 Synced to lockfile at {}", lock_path.display());
            let deltas = lock.compute_deltas(&config.target_locales);
            println!(
                "📊 Pending translations across {:?}: {}",
                config.target_locales,
                deltas.len()
            );
        }

        Commands::Translate {
            locales,
            provider,
            dry_run,
        } => {
            let config = LingoConfig::load_or_default(&root);
            let target_locales: Vec<String> = if let Some(l) = locales {
                l.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                config.target_locales.clone()
            };

            let provider_name = provider.unwrap_or_else(|| config.provider.clone());
            println!("🌐 Initializing translation engine: {provider_name}");

            let prov: Arc<dyn TranslationProvider> = match provider_name.as_str() {
                "openai" => Arc::new(OpenAiProvider::new(None, None, None)),
                "anthropic" | "claude" => Arc::new(AnthropicProvider::new(None, None)),
                "ollama" | "local" => Arc::new(OllamaProvider::new(None, None)),
                "deepseek" => Arc::new(OpenAiProvider::new(
                    None,
                    Some("deepseek-chat".to_string()),
                    Some("https://api.deepseek.com".to_string()),
                )),
                _ => Arc::new(OfflineProvider::new()),
            };

            let engine = TranslationEngine::new(prov);
            let lock_path = LingoLock::resolve_path(&root);
            let mut lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();

            let deltas = lock.compute_deltas(&target_locales);
            if deltas.is_empty() {
                println!("All translations are up to date; nothing to translate.");
                return Ok(());
            }

            if dry_run {
                println!("🔎 [Dry Run] {} items would be translated:", deltas.len());
                for d in &deltas {
                    println!("   - {:?} for {:?}", d.canonical_text, d.missing_locales);
                }
                return Ok(());
            }

            println!("🚀 Translating {} pending items...", deltas.len());
            let start = Instant::now();

            for delta in &deltas {
                for loc in &delta.missing_locales {
                    if !target_locales.contains(loc) {
                        continue;
                    }

                    match engine
                        .translate_and_verify(&[delta.canonical_text.clone()], loc)
                        .await
                    {
                        Ok(results) => {
                            if let Some(trans) = results.first() {
                                lock.record_translation(
                                    delta.coord,
                                    loc,
                                    trans.clone(),
                                    engine.provider_name(),
                                    1.0,
                                );
                                println!("  ✓ [{loc}] \"{}\" -> \"{}\"", delta.canonical_text, trans);
                            }
                        }
                        Err(err) => {
                            eprintln!(
                                "  ✗ [{loc}] Null-gate error on \"{}\": {err}",
                                delta.canonical_text
                            );
                        }
                    }
                }
            }

            lock.save_to_file(&lock_path)?;
            let elapsed = start.elapsed();
            println!("✅ Finished translation pass in {:?}", elapsed);

            // Export to locale files
            export_locales(&root, &config, &lock)?;
        }

        Commands::Compile { target, in_place } => {
            let target_fw = TargetFramework::parse(&target).unwrap_or(TargetFramework::React);
            return Err(format!(
                "`compile` is not implemented yet (target {target_fw:?}, in-place: {in_place}); use `glossa codegen` to generate accessors"
            )
            .into());
        }

        Commands::Check { ci, format } => {
            let config = LingoConfig::load_or_default(&root);
            let lock_path = LingoLock::resolve_path(&root);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();

            let deltas = lock.compute_deltas(&config.target_locales);
            let is_ci = ci || std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok();

            if !deltas.is_empty() {
                if format == "github" || is_ci {
                    for d in &deltas {
                        for loc in &d.missing_locales {
                            println!(
                                "::error title=Missing Translation [{loc}]::Missing translation for \"{}\"",
                                d.canonical_text
                            );
                        }
                    }
                }

                eprintln!(
                    "❌ CI Check Failed: {} translations are missing across {:?}!",
                    deltas.len(),
                    config.target_locales
                );
                for d in deltas.iter().take(10) {
                    eprintln!("   - [{:?}] {}", d.missing_locales, d.canonical_text);
                }
                if deltas.len() > 10 {
                    eprintln!("   ... and {} more", deltas.len() - 10);
                }

                // Write GitHub Step Summary if in GitHub Actions
                if let Ok(summary_path) = std::env::var("GITHUB_STEP_SUMMARY") {
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(summary_path) {
                        let _ = writeln!(file, "## Glossa CI Check: FAILED");
                        let _ = writeln!(
                            file,
                            "| Target Locales | Missing Items | Total Entries |"
                        );
                        let _ = writeln!(file, "| :--- | :--- | :--- |");
                        let _ = writeln!(
                            file,
                            "| {:?} | **{}** | {} |",
                            config.target_locales,
                            deltas.len(),
                            lock.entries.len()
                        );
                    }
                }

                std::process::exit(1);
            }

            if let Ok(summary_path) = std::env::var("GITHUB_STEP_SUMMARY") {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(summary_path) {
                    let _ = writeln!(file, "## Glossa CI Check: PASSED");
                    let _ = writeln!(file, "All {} entries have translations for {:?}.", lock.entries.len(), config.target_locales);
                }
            }

            println!("CI Check Passed: all {} entries have translations for {:?}.", lock.entries.len(), config.target_locales);
        }

        Commands::Diff => {
            let config = LingoConfig::load_or_default(&root);
            let lock_path = LingoLock::resolve_path(&root);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();
            let deltas = lock.compute_deltas(&config.target_locales);

            println!("📊 Localization Diff & Status:");
            println!("   Total registered keys: {}", lock.entries.len());
            println!("   Configured locales:    {:?}", config.target_locales);
            println!("   Pending translations:  {}", deltas.len());

            if !deltas.is_empty() {
                println!("\nMissing Translations:");
                for d in deltas.iter().take(15) {
                    println!("   + [{}] \"{}\"", d.missing_locales.join(", "), d.canonical_text);
                }
                if deltas.len() > 15 {
                    println!("   ... and {} more", deltas.len() - 15);
                }
            } else {
                println!("✨ All registered keys have complete translations.");
            }
        }

        Commands::Pseudo { locale } => {
            let config = LingoConfig::load_or_default(&root);
            let lock_path = LingoLock::resolve_path(&root);
            let mut lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();

            println!("🧪 Generating pseudo-localized strings for [{locale}]...");
            let mut pseudo_map = BTreeMap::new();
            let coords_and_texts: Vec<(crate::ir::coord::ApeironCoord, String)> = lock
                .entries
                .iter()
                .map(|(coord, entry)| (*coord, entry.canonical_text.clone()))
                .collect();

            for (coord, text) in coords_and_texts {
                let pseudo_val = pseudo_localize(&text);
                lock.record_translation(coord, &locale, pseudo_val.clone(), "pseudo-engine", 1.0);
                pseudo_map.insert(text, pseudo_val);
            }
            lock.save_to_file(&lock_path)?;

            let locales_dir = root.join(&config.locales_dir);
            fs::create_dir_all(&locales_dir)?;
            let out_file = locales_dir.join(format!("{locale}.json"));
            fs::write(&out_file, resource::emit_json(&pseudo_map))?;

            println!("✓ Pseudo-localization generated at {}", out_file.display());
            println!("  Example: \"{}\"", pseudo_map.values().next().unwrap_or(&String::new()));
        }

        Commands::Sync => {
            let config = LingoConfig::load_or_default(&root);
            let lock_path = LingoLock::resolve_path(&root);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();
            export_locales(&root, &config, &lock)?;
            println!("Synchronized all locale files from lockfile.");
        }

        Commands::Bench {
            count,
            spbench,
            json_output,
        } => {
            run_spbench_suite(count, spbench, json_output)?;
        }

        Commands::Mcp => {
            crate::mcp::run_mcp_server().await?;
        }

        Commands::Lorelake { action, endpoint, bucket } => {
            use crate::engine::lorelake::{LorelakeClient, LorelakeConfig};
            let config = LorelakeConfig {
                endpoint,
                bucket,
                prefix: "coords".to_string(),
                timeout: std::time::Duration::from_secs(10),
            };
            let client = LorelakeClient::new(config);
            let lock_path = LingoLock::resolve_path(&root);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();

            match action.as_str() {
                "push" => {
                    println!("Pushing {} entries to {}...", lock.entries.len(), client.config.endpoint);
                    match client.push_translations(&lock).await {
                        Ok(pushed) => println!("Pushed {pushed} of {} entries.", lock.entries.len()),
                        Err(e) => return Err(format!("push failed: {e}").into()),
                    }
                }
                other => {
                    return Err(format!("unsupported action \"{other}\"; only \"push\" is implemented").into());
                }
            }
        }

        Commands::Export { format, locale, output } => {
            use crate::engine::lorelake::{format_benchmark_tsv, format_huggingface_jsonl};
            let lock_path = LingoLock::resolve_path(&root);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();

            let mut map = BTreeMap::new();
            for (_, entry) in &lock.entries {
                if let Some(loc_entry) = entry.locales.get(&locale) {
                    map.insert(entry.canonical_text.clone(), loc_entry.translated_text.clone());
                }
            }

            let rendered = match format.to_ascii_lowercase().as_str() {
                "tsv" | "benchmark" | "flores" => format_benchmark_tsv(&map, &locale),
                _ => format_huggingface_jsonl(&map, &locale),
            };

            if let Some(out_file) = output {
                fs::write(&out_file, &rendered)?;
                println!("✓ Exported {} translations to {}", map.len(), out_file.display());
            } else {
                print!("{rendered}");
            }
        }

        Commands::Codegen { target, output } => {
            use crate::codegen::{generate_code, CodegenOptions, CodegenTarget};
            let config = LingoConfig::load_or_default(&root);
            let lock_path = LingoLock::resolve_path(&root);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();

            let codegen_target = match target.to_ascii_lowercase().as_str() {
                "rust" | "rs" => CodegenTarget::Rust,
                _ => CodegenTarget::TypeScript,
            };

            let opts = CodegenOptions {
                target: codegen_target,
                source_locale: config.source_locale.clone(),
                target_locales: config.target_locales.clone(),
            };

            let code = generate_code(&lock, &opts);
            let out_file = output.unwrap_or_else(|| match codegen_target {
                CodegenTarget::TypeScript => root.join("src").join("i18n").join("messages.ts"),
                CodegenTarget::Rust => root.join("src").join("i18n").join("messages.rs"),
            });

            if let Some(parent) = out_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&out_file, code)?;
            println!("Generated message accessors at {}", out_file.display());
            println!("   Total catalog functions: {}", lock.entries.len());
        }

        Commands::Visual { port } => {
            use crate::visual::{run_visual_server, VisualServerConfig};
            let config = VisualServerConfig {
                port,
                root_dir: root.clone(),
            };
            run_visual_server(config)?;
        }

        Commands::Import { from } => {
            let mut config = LingoConfig::load_or_default(&root);
            let imports = crate::importer::detect_incumbent_systems(&root);
            if imports.is_empty() {
                println!("! No incumbent i18n configuration files detected in {}", root.display());
            } else {
                for imp in &imports {
                    if let Some(ref selected) = from {
                        if !imp.system.name().to_lowercase().contains(&selected.to_lowercase()) {
                            continue;
                        }
                    }
                    println!("🔄 Importing from {} ({})", imp.system.name(), imp.config_file.display());
                    crate::importer::import_to_glossa_config(imp, &mut config);
                }
                let saved = config.save_to_file(&root)?;
                println!("Wrote configuration to {}", saved.display());
            }
        }
    }

    Ok(())
}

fn export_locales(root: &Path, config: &LingoConfig, lock: &LingoLock) -> std::io::Result<()> {
    let locales_dir = root.join(&config.locales_dir);
    fs::create_dir_all(&locales_dir)?;

    for loc in &config.target_locales {
        let mut map = BTreeMap::new();
        for (_, entry) in &lock.entries {
            if let Some(loc_entry) = entry.locales.get(loc) {
                map.insert(entry.canonical_text.clone(), loc_entry.translated_text.clone());
            }
        }

        match config.output_format.as_str() {
            "po" => {
                let content = resource::emit_po(&map, loc);
                fs::write(locales_dir.join(format!("{loc}.po")), content)?;
            }
            "fluent" | "ftl" => {
                let content = resource::emit_fluent(&map);
                fs::write(locales_dir.join(format!("{loc}.ftl")), content)?;
            }
            "yaml" | "yml" => {
                let content = resource::emit_yaml(&map, loc);
                fs::write(locales_dir.join(format!("{loc}.yml")), content)?;
            }
            "android" | "xml" => {
                let res_dir = locales_dir.join(format!("values-{loc}"));
                fs::create_dir_all(&res_dir)?;
                let content = resource::emit_android_xml(&map);
                fs::write(res_dir.join("strings.xml"), content)?;
            }
            "ios" | "strings" => {
                let lproj_dir = locales_dir.join(format!("{loc}.lproj"));
                fs::create_dir_all(&lproj_dir)?;
                let content = resource::emit_ios_strings(&map);
                fs::write(lproj_dir.join("Localizable.strings"), content)?;
            }
            "arb" => {
                let content = resource::emit_flutter_arb(&map, loc);
                fs::write(locales_dir.join(format!("intl_{loc}.arb")), content)?;
            }
            "properties" => {
                let content = resource::emit_properties(&map);
                fs::write(locales_dir.join(format!("messages_{loc}.properties")), content)?;
            }
            "xliff" | "xlf" => {
                let content = resource::emit_xliff(&map, &config.source_locale, loc);
                fs::write(locales_dir.join(format!("messages_{loc}.xlf")), content)?;
            }
            "csv" => {
                let content = resource::emit_csv(&map);
                fs::write(locales_dir.join(format!("{loc}.csv")), content)?;
            }
            _ => {
                let content = resource::emit_json(&map);
                fs::write(locales_dir.join(format!("{loc}.json")), content)?;
            }
        }
    }
    Ok(())
}

#[derive(serde::Serialize, Debug)]
pub struct SpbenchReport {
    pub suite: String,
    pub timestamp: String,
    pub environment: SpbenchEnv,
    pub benchmarks: Vec<BenchmarkResult>,
}

#[derive(serde::Serialize, Debug)]
pub struct SpbenchEnv {
    pub os: String,
    pub arch: String,
    /// Resident set size of this process in MB when the benchmarks finished
    /// (read from `/proc/self/statm`; `null` where that file is unavailable).
    pub rss_mb_at_end: Option<f64>,
}

#[derive(serde::Serialize, Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub count: usize,
    pub total_ms: f64,
    pub throughput_per_sec: f64,
    pub mean_us: f64,
    pub p50_us: f64,
    pub p90_us: f64,
    pub p99_us: f64,
}

/// Current resident set size in MB, from `/proc/self/statm` (assumes 4 KiB pages).
fn current_rss_mb() -> Option<f64> {
    let content = fs::read_to_string("/proc/self/statm").ok()?;
    let pages = content.split_whitespace().nth(1)?.parse::<f64>().ok()?;
    Some((pages * 4096.0) / (1024.0 * 1024.0))
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 * pct) / 100.0) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Builds a result from per-operation latencies (microseconds) and the wall time of the loop.
/// `ops_per_sample` is how many logical operations one latency sample covers.
fn summarize(
    name: &str,
    mut samples_us: Vec<f64>,
    ops_per_sample: usize,
    elapsed: std::time::Duration,
) -> BenchmarkResult {
    samples_us.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let ops = (samples_us.len() * ops_per_sample).max(1);
    let per_op = ops_per_sample as f64;
    BenchmarkResult {
        name: name.to_string(),
        count: ops,
        total_ms: elapsed.as_secs_f64() * 1000.0,
        throughput_per_sec: ops as f64 / elapsed.as_secs_f64().max(f64::MIN_POSITIVE),
        mean_us: elapsed.as_secs_f64() * 1_000_000.0 / ops as f64,
        p50_us: percentile(&samples_us, 50.0) / per_op,
        p90_us: percentile(&samples_us, 90.0) / per_op,
        p99_us: percentile(&samples_us, 99.0) / per_op,
    }
}

/// Times Glossa's own code on fixed in-process samples and prints the results. It does not run
/// or compare against any other tool, and the numbers depend on the machine it runs on.
fn run_spbench_suite(
    count: usize,
    spbench: bool,
    json_output: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running Glossa micro-benchmarks ({count} iterations)...");

    let sample_jsx = r#"
    export default function Card({ title, count, user }) {
        return (
            <div className="card">
                <h1>{title}</h1>
                <p>Welcome back, {user.name}! You have {count} notifications.</p>
                <button placeholder="Filter notifications">Click <a href="/settings">here</a> to configure</button>
            </div>
        );
    }
    "#;

    let sample_vue = r#"
    <template>
        <div class="user-panel">
            <h2>{{ $t('User Account') }}</h2>
            <p>Hello {{ user.name }}, status: {{ status }}</p>
            <button @click="refresh">Refresh Telemetry</button>
        </div>
    </template>
    "#;

    // 1. Extraction: alternates the JSX and Vue samples.
    let mut samples = Vec::with_capacity(count);
    let mut total_extracted = 0;
    let start = Instant::now();
    for i in 0..count {
        let t0 = Instant::now();
        let items = if i % 2 == 0 {
            crate::parsers::jsx::extract_jsx(sample_jsx, &format!("Card{i}.tsx"))
        } else {
            crate::parsers::vue::extract_vue(sample_vue, &format!("User{i}.vue"))
        };
        samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
        total_extracted += items.len();
    }
    let extraction = summarize("Extraction (JSX and Vue samples, alternating)", samples, 1, start.elapsed());

    // 2. Coordinate hashing plus lockfile insertion.
    let b2_count = count * 2;
    let mut lock = crate::lockfile::LingoLock::new();
    let mut samples = Vec::with_capacity(b2_count);
    let start = Instant::now();
    for i in 0..b2_count {
        let t0 = Instant::now();
        let coord = crate::ir::coord::ApeironCoord::from_str_canonical(&format!("Term #{i} in application context"));
        lock.record_translation(coord, "es", format!("Término #{i}"), "spbench", 1.0);
        samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
    }
    let lockfile = summarize("Coordinate hash and lockfile insert", samples, 1, start.elapsed());

    // 3. Integrity gate on one valid pair (slots and numbered tags preserved).
    let valid_src = "Welcome {user}, click <0>here</0> to proceed.";
    let valid_trans = "Bienvenido {user}, haz clic <0>aquí</0> para continuar.";
    let mut samples = Vec::with_capacity(count);
    let mut gate_ok = 0usize;
    let start = Instant::now();
    for _ in 0..count {
        let t0 = Instant::now();
        if crate::engine::gate::ApeironNullGate::validate(valid_src, valid_trans).is_ok() {
            gate_ok += 1;
        }
        samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
    }
    let gate = summarize("Integrity gate (one valid pair)", samples, 1, start.elapsed());

    // 4. Catalog emitters: JSON, YAML, PO and Fluent over a 500-key catalog, repeated.
    let mut catalog = BTreeMap::new();
    for i in 0..500 {
        catalog.insert(format!("key_{i}"), format!("Translated text #{i} for user {{user}}"));
    }
    const B4_ITERATIONS: usize = 200;
    let mut samples = Vec::with_capacity(B4_ITERATIONS);
    let start = Instant::now();
    for _ in 0..B4_ITERATIONS {
        let t0 = Instant::now();
        let _json = crate::emitters::resource::emit_json(&catalog);
        let _yaml = crate::emitters::resource::emit_yaml(&catalog, "es");
        let _po = crate::emitters::resource::emit_po(&catalog, "es");
        let _ftl = crate::emitters::resource::emit_fluent(&catalog);
        samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
    }
    let emitters = summarize(
        "Emitters (JSON, YAML, PO, Fluent; per unit per format)",
        samples,
        500 * 4,
        start.elapsed(),
    );

    let report = SpbenchReport {
        suite: "glossa bench".to_string(),
        timestamp: crate::lockfile::now_rfc3339(),
        environment: SpbenchEnv {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rss_mb_at_end: current_rss_mb(),
        },
        benchmarks: vec![extraction, lockfile, gate, emitters],
    };

    println!("OS: {} | Arch: {} | Timestamp: {}", report.environment.os, report.environment.arch, report.timestamp);
    println!("Extraction produced {total_extracted} terms; the gate accepted {gate_ok} of {count} checks.");
    println!("{:<58} {:>10} {:>12} {:>10} {:>10} {:>10}", "Benchmark", "ops", "ops/sec", "mean us", "p50 us", "p99 us");
    for b in &report.benchmarks {
        println!(
            "{:<58} {:>10} {:>12.0} {:>10.3} {:>10.3} {:>10.3}",
            b.name, b.count, b.throughput_per_sec, b.mean_us, b.p50_us, b.p99_us
        );
    }
    if let Some(rss) = report.environment.rss_mb_at_end {
        println!("RSS at end of run: {rss:.1} MB");
    }

    if let Some(out_path) = json_output {
        let json_data = serde_json::to_string_pretty(&report)?;
        fs::write(&out_path, json_data)?;
        println!("Wrote JSON report to {}", out_path.display());
    } else if spbench {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    Ok(())
}
