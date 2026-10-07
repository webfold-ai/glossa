# Hyperframes Composition Brief: Glossa

## Objective
Create a short, restrained launch video for Glossa in the ptyktos-website monochrome editorial style.

## Output
- Composition directory: `brag-output/composition/`
- Rendered video: `brag-output/brag.mp4`
- Format: landscape — 1920x1080
- Duration: 20s

## Source Material
- Project root: /home/ctown/projects/glossa
- Primary files read: README.md, spbench-results.json
- Product name: Glossa
- Tagline / strongest claim: "4.8× faster, ~45× lower memory drop-in replacement for Lingo.dev, Inlang, Tolgee, and Crowdin (measured). Built in pure, zero-dependency Rust."
- Key moment to recreate: the 3-liner quickstart, as a live-typed terminal session
- Copy that must appear verbatim:
  - `curl -fsSL https://ptyktos.com/install.sh | bash`
  - `glossa init --locales es,fr,de,ja`
  - `glossa extract && glossa check --ci`
  - "56,887 files/sec" (AST extraction, 17.58 µs/file, measured — spbench-results.json)
  - "8.3 MB RSS" (vs a measured 375 MB for lingo.dev's own CLI)
  - "0.0% regressions" across 61 test suites

## Creative Direction
- Tone preset: polished
- Creative direction: quiet, research-paper restraint — matches ptyktos-website's editorial identity, not a SaaS hype reel
- Interpretation: slower holds, soft entrances, no aggressive cuts, one single accent-color use in the entire video (spent on "4.8×")
- Angle: the terminal is the product — no invented UI, no generic SaaS language, every number sourced from glossa's own README/benchmarks
- Hook: "zero dependencies." → "4.8× faster."
- Outro / punchline: "Glossa." / "Zero-dependency Rust. Drop-in for Lingo.dev, Tolgee, Crowdin."
- Avoid:
  - Generic SaaS language
  - Abstract filler visuals
  - Any competitor/brand reference not already named in glossa's own README
  - More than one accent-color use

## Visual Identity
- Background: #f8f8f8 (paper)
- Text: #1b1b1f (ink) / #333 (body) / #666 (muted)
- Accent: #0044ff (used once only)
- Display font: Space Grotesk (local woff2, assets/fonts/space-grotesk-latin.woff2)
- Body/label font: Space Mono (local woff2, assets/fonts/space-mono-latin.woff2 + space-mono-bold.woff2)
- Visual references from the project: ptyktos-website's globals.css (paper/ink/muted palette, hairline dividers, uppercase mono labels, ink-card treatment for hero/outro moments)

## Storyboard
Use `brag-plan.md`'s storyboard as the creative contract.

Scene summary:
1. Hook — 4s (0–4s) — "zero dependencies." / "4.8× faster." (accent on "×")
2. Terminal quickstart — 7s (4–11s) — 3-line CLI session, typed reveal
3. Benchmark cards — 6.5s (11–17.5s) — 3 real SPBench numbers, beat-locked arrivals
4. Outro — 2.5s (17.5–20s) — ink card wordmark + tagline, beat-locked to strong cue at 17.52s

## Audio
- Audio role: sparse professional accents over a low restrained bed
- Audio arc: bed fades in under Scene 1, holds low throughout, fades out under the outro hit
- Music: happy-beats-business-moves-vol-1-by-ende-dot-app.mp3 (bundled, first 20s only), tempo 120.19 BPM
- Music treatment: data-volume driven by a fromTo tween 0→0.28 over 0–0.6s, held, then tweened 0.28→0 over 19–20s
- Music cue guidance: bundled preset (`happy-beats-business-moves-vol-1-by-ende-dot-app.music-cues.json`). Strong cue at 17.5195s used for the outro. Beat grid (~0.5s spacing from 3.02s) used for card-arrival spacing at every-other-beat (~1s) per the readable-text rule.
- Audio-reactive treatment: none
- Audio-coupled moments:
  - Scene 1 — soft reveal hit under "4.8×" (~1.7s)
  - Scene 2 — one soft keypress tick per terminal line landing (3 total)
  - Scene 3 — one soft click per benchmark card arrival, beat-grid locked
  - Scene 4 — one logo-payoff bell hit, beat-locked to 17.5195s
- SFX selection guidance: low-HF-risk picks per sfx-analysis.md — impactSoft_medium_001.ogg (major reveal), keypress-00{1,2,3}.wav (typing), click_003.ogg (low-risk selection/arrival), impactBell_heavy_000.ogg (logo payoff, used once)
- Exact SFX choice: as listed above — already selected and copied into `assets/sfx/`, no further Hyperframes discretion needed
- Audio files: copied into `brag-output/composition/assets/music/` and `brag-output/composition/assets/sfx/`

## Hyperframes Instructions
Build per hyperframes-core (composition contract), hyperframes-animation (GSAP motion), hyperframes-keyframes (n/a — no camera moves in this brief). Single standalone `index.html`, one root, one paused GSAP timeline registered at `window.__timelines["glossa-brag"]`. All local assets (fonts, music, sfx) — no CDN except the GSAP runtime script itself.

Requirements:
- Show the real CLI quickstart verbatim (the "show the thing" law for a CLI product).
- Keep all text readable — hold every line at least ~0.8-1.2s settled before it may exit.
- 20s total, matches root `data-duration`.
- Run `npx hyperframes check` and fix every reported error including WCAG contrast before rendering.
- Render `--quality delivery` (or `looks` if delivery is unavailable in this CLI version) for the final file; verify with `ffprobe` that duration is ~20s and file size is consistent with a real 1080p encode (several MB, not under 1MB).
