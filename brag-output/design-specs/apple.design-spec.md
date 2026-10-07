# Apple Product-Marketing Motion — Design Spec

Status: reference incumbent spec. Format: RFC 2119 keywords (MUST, MUST NOT,
SHOULD, SHOULD NOT, MAY), written in ASD-STE100 style (short sentences, one
instruction per sentence, active voice, present tense, controlled vocabulary).
Scope: the motion and typography language of Apple's product pages and
keynote reveals (apple.com product pages, WWDC/event keynote type reveals).

## 1. Typography

1. A headline MUST use a single grotesque sans display face at heavy or
   semibold weight.
2. A headline MUST use tight tracking. Tracking SHOULD sit between -0.02em
   and -0.04em at display sizes.
3. A headline MUST NOT mix more than two weights in one line.
4. Body copy MUST use a lighter weight than the headline above it.
5. A headline SHOULD span no more than two lines. Three lines MAY appear only
   in a dense feature-grid caption.

## 2. Color and Surface

6. A hero frame MUST use one dominant surface color. The surface MUST be
   near-black or near-white. It MUST NOT be a mid-gray.
7. A product render MUST sit on a surface with soft, wide, low-contrast
   gradient falloff. The gradient MUST NOT introduce a visible hard edge.
8. Accent color MUST be reserved for one product-defining detail per scene.
   Accent color MUST NOT appear as decoration.
9. Depth MUST come from light and shadow, not from added texture or noise.

## 3. Motion and Timing

10. An entrance MUST use a decelerating ease. The ease SHOULD approximate a
    custom cubic-bezier close to (0.16, 1, 0.3, 1) — a fast start and a long,
    soft settle.
11. An entrance MUST NOT overshoot. A headline MUST NOT bounce.
12. A multi-word headline SHOULD reveal word-by-word or line-by-line, not
    character-by-character.
13. A word reveal MUST use a small stagger. Stagger per word SHOULD be
    40–90ms.
14. A camera or product move MUST read as physically continuous. A cut MUST
    NOT interrupt a move already in progress.
15. A scale change MUST be subtle. A hero element SHOULD scale by no more
    than 1.05–1.15× during a settle-in.
16. Parallax MAY separate a background layer from a foreground subject by
    2–6% of frame width during a slow pan.
17. A blur-to-focus reveal MAY introduce a subject. Blur amount MUST resolve
    to zero before the subject becomes a reading target.

## 4. Composition and Pacing

18. A scene MUST hold its final state before the next scene begins. The hold
    MUST last at least 0.8 seconds for one short headline.
19. A transition between scenes SHOULD be a crossfade or a match-cut on a
    shared element. A transition MUST NOT be a hard color-flash cut.
20. Negative space MUST occupy the majority of a hero frame. A hero frame
    MUST NOT exceed roughly 30% type/subject coverage.
21. Pacing SHOULD feel unhurried. A scene MUST NOT force a reveal faster than
    the reading-time floor for its copy.

## 5. What this style is NOT

22. This style MUST NOT use bright saturated color fields.
23. This style MUST NOT use comic or playful bounce easing (`back.out`,
    elastic).
24. This style MUST NOT use dense information-per-frame layouts.
