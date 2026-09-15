# Enterprise Implementation Guide: Multi-Repo, Governance & Air-Gapped Workflows

> **Target Audience:** Enterprise Architects, Chief Information Security Officers (CISO), Engineering Directors.  
> **Scope:** Multi-repo coordination, brand tone governance, air-gapped sovereign VPC deployment, and SOC2/HIPAA compliance.

---

## 1. Enterprise Architecture Overview

Global enterprises often operate dozens of frontend applications, mobile apps, and microservices across disparate tech stacks. In this environment, traditional localization platforms create severe friction:
* **Duplicative Translation Spend:** If the Billing Team translates `"Cancel Subscription"`, the Mobile Team and Checkout Team independently re-translate the exact same string, wasting tens of thousands in agency and LLM fees.
* **Security & IP Exfiltration:** Sending proprietary source code and unannounced product copy to third-party multi-tenant SaaS platforms violates corporate data policies and triggers security review delays.
* **Inconsistent Terminology:** Different teams translate key corporate terms, legal disclaimers, or trademarked product names inconsistently across platforms.

**Glossa delivers a unified, sovereign localization architecture:**

```
                  ┌──────────────────────────────────────────────┐
                  │    CENTRAL LORELAKE TRANSLATION LAKEHOUSE    │
                  │        (S3 / Private VPC Object Store)       │
                  │   Shared Invariant Coordinate Deduplication  │
                  └──────────────────────┬───────────────────────┘
                                         │
        ┌────────────────────────────────┼────────────────────────────────┐
        ▼                                ▼                                ▼
┌──────────────────────┐      ┌──────────────────────┐      ┌──────────────────────┐
│   Web App (Next.js)  │      │  Mobile (iOS/Android)│      │  Backend (FastAPI)   │
│   glossa extract     │      │   glossa extract     │      │   glossa extract     │
│   glossa lorelake    │      │   glossa lorelake    │      │   glossa lorelake    │
└──────────────────────┘      └──────────────────────┘      └──────────────────────┘
```

---

## 2. Multi-Repo & Monorepo Deduplication

Enterprise teams synchronize local lockfiles with a centralized content-addressed store (`Lorelake S3`) to achieve organizational deduplication:

```bash
# Push newly translated terms to organizational lakehouse
glossa lorelake push --bucket enterprise-translations-prod

# Pull verified translations across all company repositories
glossa lorelake pull --bucket enterprise-translations-prod
```

### How It Saves Budget:
When any service in your company translates a button label, disclaimer, or error message, Glossa maps it to an invariant 64-bit coordinate (`ContentCoord`). Every other service in your enterprise consumes that translation instantly with **0 new translation tokens or agency fees**.

---

## 3. Brand Glossaries & Tone Governance

Enterprise brand safety requires that legal disclaimers, product names, and trademarked terms are never translated or modified by AI models.

Glossa enforces brand glossaries directly inside the **Integrity Gate**. Define protected terms in `glossa.json`:

```json
{
  "source_locale": "en",
  "target_locales": ["es", "fr", "de", "ja", "zh"],
  "glossary": {
    "Ptyktos": "Ptyktos",
    "SmartVault Enterprise": "SmartVault Enterprise",
    "Zero-Knowledge Encryption": "Chiffrement à divulgation nulle de connaissance"
  },
  "rules": {
    "preserve_untranslated_terms": ["BrandName", "API_KEY", "OAuth2"],
    "tone": "formal_enterprise",
    "strict_tag_conservation": true
  }
}
```

If an LLM or translator translates `"SmartVault Enterprise"` into `"Bóveda Inteligente"` or alters a protected legal term, Glossa's Integrity Gate catches the violation in 1.8 µs and blocks the build:

```text
❌ GateViolation::GlossaryViolation:
   Source: "Welcome to SmartVault Enterprise"
   Translation: "Bienvenido a Bóveda Inteligente"
   Reason: Term 'SmartVault Enterprise' must be preserved verbatim.
```

---

## 4. Air-Gapped & Sovereign VPC Deployment

For defense, intelligence, healthcare, and financial institutions requiring total isolation from the public internet, Glossa operates in 100% air-gapped mode.

### Architecture Blueprint

```
 ┌────────────────────────────────────────────────────────────────────────┐
 │                      ENTERPRISE PRIVATE VPC                            │
 │                                                                        │
 │   ┌──────────────────────┐              ┌──────────────────────────┐   │
 │   │  CI/CD Build Cluster │              │ Private GPU Cluster      │   │
 │   │  (GitLab / Jenkins)  │              │ (vLLM / Ollama Node)     │   │
 │   │                      │  HTTP / API  │                          │   │
 │   │  Runs: glossa check  ├─────────────►│ DeepSeek-R1 / Llama-3 70B│   │
 │   │        glossa trans  │              │                          │   │
 │   └──────────┬───────────┘              └──────────────────────────┘   │
 │              │                                                         │
 │              ▼                                                         │
 │   ┌──────────────────────┐                                             │
 │   │ Lorelake S3 / MinIO  │                                             │
 │   │ Local Coordinate DB  │                                             │
 │   └──────────────────────┘                                             │
 └────────────────────────────────────────────────────────────────────────┘
```

### Setup Configuration
Set the local endpoint in your environment:

```bash
export GLOSSA_PROVIDER="ollama"
export OLLAMA_HOST="http://internal-vllm-node.vpc.local:11434"
export GLOSSA_MODEL="llama3:70b"

# Run localization completely offline inside your VPC
glossa translate --locales es,fr,de,ja
```

*No source code, customer data, or internal copy ever leaves your private network.*

---

## 5. Separation of Concerns & Team Workflows

Glossa enforces clean role boundaries so engineering velocity is never blocked by localization reviews:

| Persona | Primary Tool | Responsibilities |
|---|---|---|
| **Software Engineers** | CLI / Git / IDE MCP | Writes UI code naturally; runs `glossa check` in CI; never edits raw translation tables. |
| **Product Managers** | In-Context Studio (`glossa visual`) | Reviews copy live on localhost:3920 or staging; clicks text to adjust tone or phrasing. |
| **Localization Teams** | Glossa Web Console | Manages corporate glossaries, resolves fuzzy translation memory deltas, approves overrides. |
| **SecOps / Compliance** | CI Gating Policies | Enforces zero data leakage, validates audit logs, reviews air-gapped container hashes. |

---

## 6. Security, Compliance & Audit Trail

* **Zero Third-Party Telemetry:** The Glossa binary contains no analytics, phone-home beacons, or external tracking calls.
* **Cryptographically Immutable Audit Trail:** Every translation record in `glossa.lock` stores the provider, timestamp, confidence score, and invariant coordinate hash, providing complete provenance for regulatory audits (SOC2 Type II, HIPAA, ISO 27001).
* **Minimal Blast Radius:** Single static binary (7.9 MB) with no dynamic runtime dependencies, eliminating CVE vulnerabilities associated with JavaScript `node_modules` supply chains.
