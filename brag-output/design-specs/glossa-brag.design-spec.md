# Glossa Brag Video — Synthesis Design Spec

Status: normative build contract for `composition/index.html`. Format: RFC
2119 keywords, ASD-STE100 sentence discipline. This spec resolves the three
incumbent specs (`apple.design-spec.md`, `openai.design-spec.md`,
`higgsfield.design-spec.md`) into one buildable direction, then layers the
one non-negotiable project constraint on top: the ptyktos-website monochrome
palette. Where an incumbent rule conflicts with the palette constraint, the
palette constraint MUST win.

## 1. What each incumbent contributes

1. Apple contributes ease curves and restraint: entrances MUST decelerate
   smoothly, camera moves MUST read as continuous, and a scene MUST NOT
   overcrowd its frame.
2. OpenAI contributes calm staging: a claim MUST center, holds MUST be long
   enough to read, and color MUST stay restrained even where a gradient or
   glow appears.
3. Higgsfield contributes pace and proof-first structure: the video MUST
   open with a concrete claim within two seconds, and a stat reveal MAY use
   a small confident overshoot rather than a purely calm settle.

## 2. Non-negotiable project constraint

4. The palette MUST stay paper `#f8f8f8`, ink `#1b1b1f`, body `#333`, muted
   `#666`, hairline `#e2e2e2`, quiet `#f0f0f0`, and exactly one accent
   `#0044ff`.
5. The accent MUST appear in exactly one scene of the video. The accent MUST
   NOT decorate more than one moment.
6. Typography MUST stay Space Grotesk (display) and Space Mono (label/mono),
   the real ptyktos-website font files. No other family MAY appear.
7. The video MUST NOT introduce color gradients, glows, or lens-flare-style
   ornament. Depth MUST come from motion (parallax, scale, blur-focus), not
   from added color.

## 3. Motion requirements (this is the fix for "word slop")

8. A headline MUST reveal as kinetic type, not as a single opacity-and-y
   fade on the whole line. A headline SHOULD reveal word-by-word with a
   40–90ms stagger per word (Apple rule 12–13), or by a masked line-wipe.
9. Every scene-to-scene cut MUST use a real transition (crossfade, wipe, or
   shared-element match-cut). A hard visibility-swap cut MUST NOT remain
   between any two of the four scenes.
10. At least one scene MUST include a camera-style move (a slow scale/pan on
    a `.world` wrapper, per `coordinate-target-zoom` or `viewport-change`).
    A move MUST read as continuous, not as a snap.
11. A stat-card or proof-point entrance MAY use a small `back.out` overshoot
    (Higgsfield rule 10). Overshoot MUST stay small enough to avoid a
    cartoon wobble (`OVERSHOOT ≤ ~1.5`).
12. A proof number or claim MUST be the visually loudest element in its
    scene (Higgsfield rule 16) — largest type, first to settle, longest
    hold.
13. Background texture MAY carry subtle continuous motion (a slow parallax
    grid, a drifting hairline) during a hold, so no frame reads as a static
    slide. Motion MUST stay slow enough to read as ambient, not as an event
    (OpenAI rule 12).
14. The hook MUST land its claim within two seconds of scene start
    (Higgsfield rule 9), even while using a smooth, non-bouncy ease for that
    first reveal (Apple rule 11).

## 4. Verification

15. The final render MUST pass `npx hyperframes check` with 0 errors.
16. The final render MUST keep total duration between 15 and 25 seconds.
17. Every scene MUST be reviewed via `npx hyperframes snapshot` at each
    scene's settle point before the delivery render runs.
