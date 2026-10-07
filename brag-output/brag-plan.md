# Brag Plan: Glossa

## What is this app?
A pure zero-dependency Rust CLI that replaces Lingo.dev, Tolgee, Inlang, and Crowdin as a universal localization engine and CI/CD verification gate — a measured 4.8× faster extraction with ~45× lower memory, and it mathematically rejects broken translations before they ship.

## The angle
Glossa has no landing page and no app UI to screenshot — it's a CLI. The video treats the terminal itself as the product: real commands, real numbers, real output. No mockups, no invented metaphors. The restraint of the visual system (paper/ink monochrome, one blue accent used exactly once) mirrors the restraint of the product's own pitch: it doesn't need decoration, the numbers do the work.

## Hook (first 2-3 seconds)
Two lines, full-bleed, Space Grotesk: "zero dependencies." then "4.8× faster." — the second line's "×" is the video's single accent-color moment.

## Key moments (the middle)
- The real 3-liner install → init → extract/check quickstart, typed live in a Space Mono terminal card.
- Three SPBench numbers arriving in sequence: 56,887 files/sec (measured), 8.3 MB idle RSS, 0.0% regressions across 61 test suites.

## Outro / punchline
Ink card, white type: "Glossa." — "Zero-dependency Rust. Drop-in for Lingo.dev, Tolgee, Crowdin."

## User flow worth showing
none — landing-page only (glossa is a headless CLI, no app screens). The 3-liner quickstart stands in as the flow: install → init → extract & check.

## Tone
- Preset: polished
- Creative direction: quiet, confident, research-paper restraint — the source site (ptyktos-website) is an evidence-first editorial portfolio, not a SaaS launch page
- Interpretation: slow-ish holds, no aggressive cuts, generous whitespace, type does the work instead of motion tricks. One accent-color use in the whole video, spent deliberately on the strongest number.

## Format: landscape — 1920x1080
## Duration: 20s

## Visual identity (from ptyktos-website, not glossa — glossa has no site of its own)
- Background: #f8f8f8 (paper)
- Accent: #0044ff — used exactly once, the "×" in "4.8×"
- Text: #1b1b1f (ink), #333 (body), #666 (muted)
- Display font: Space Grotesk (300-700), tight letter-spacing on big type, sentence case
- Body/label font: Space Mono, uppercase labels only, 0.05-0.08em tracking
- Strongest visual element: the terminal quickstart card — ink-on-paper, Space Mono, hairline border

## Share copy (draft)
Glossa: a 7.9 MB, zero-dependency Rust binary that replaces Lingo.dev, Tolgee, Inlang, and Crowdin. 4.8× faster extraction (measured), 8.3 MB idle RSS, 0.0% regressions across 61 test suites.

## Audio direction
- Role: sparse professional accents over a low, restrained instrumental bed
- Music: happy-beats-business-moves-vol-1 (bundled), used only for its first 20s
- Music treatment: fade in 0→0.6s, hold at low volume (~0.28) as a bed, fade out 19→20s under the outro
- Music cue guidance: track tempo 120.19 BPM (bundled preset). Strong cue at 17.52s — outro wordmark beat-locked to it. Beat grid available every ~0.5s from 3.02s onward for sequential card timing.
- Audio-reactive treatment: none — restraint over cleverness for this tone
- SFX posture: sparse, low-HF-risk only (per sfx-analysis.md)
- Audio-coupled moments: hook accent line (soft reveal hit), 3 terminal lines (single soft keypress tick each, not per-character), 3 benchmark cards (soft click on arrival), outro wordmark (bell/logo-payoff hit, beat-locked)
- Restraint rule: no waveform/particle visuals, no more than one bell/impact hit in the whole video, no per-character typing sound spam

## Storyboard

### Scene 1 — Hook — 4s
"zero dependencies." settles first (0.2–0.9s in, holds). "4.8× faster." settles second (1.6–2.4s in, holds to 4s), with "×" in accent blue.
Sequential/interaction: yes — two lines arrive in sequence, each holds before the next
Audio intent: quiet confidence, then one small lift on the number
Audio-coupled idea: soft reveal hit under "4.8×" at ~1.7s
Music: bed fading in under this scene
Transition mood: clean → Scene 2

### Scene 2 — Terminal quickstart — 7s (4–11s)
Ink-bordered paper card, "THE 3-LINER QUICKSTART" label in mono. Three lines type-reveal in sequence (clip-path wipe, not per-character): install curl, `glossa init --locales es,fr,de,ja`, `glossa extract && glossa check --ci`. Full block holds after all three land.
Sequential/interaction: yes — 3 lines reveal one after another, each a left-to-right wipe
Audio intent: mechanical, precise, calm
Audio-coupled idea: one soft keypress tick per line landing (3 total, not per-character)
Music: bed continues, low
Transition mood: clean → Scene 3

### Scene 3 — Benchmark cards — 6.5s (11–17.5s)
"SPBENCH · REAL NUMBERS" label. Three cards arrive in sequence then hold together as a set: 56,887/sec (AST extraction, measured), 8.3 MB (idle RSS vs a measured 375 MB for lingo.dev's own CLI), 0.0% (regressions, 61 suites).
Sequential/interaction: yes — 3 cards arrive one by one on the beat grid, then hold as a set for readability
Audio intent: rhythmic but restrained
Audio-coupled idea: one soft click per card, beat-grid locked (~1s spacing, every-other-beat at 120 BPM per the readable-text rule)
Music: bed continues
Transition mood: clean → Scene 4

### Scene 4 — Outro — 2.5s (17.5–20s)
Ink card, white type. "Glossa." then the tagline line beneath it. Holds to end.
Sequential/interaction: none
Audio intent: settled, confident close
Audio-coupled idea: one logo-payoff hit beat-locked to the 17.52s strong cue
Music: fades out 19–20s under the hit

**Music mood for this video:** upbeat but restrained instrumental bed, cut to 20s
**Audio summary:** a quiet, low-volume music bed with sparse, low-HF-risk clicks/ticks on every arrival and one deliberate bell hit at the outro, beat-locked to the track's first strong cue.
