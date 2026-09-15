# Glossa for Vibe Coders, Non-Technical Builders & AI Web Developers

> **Target Audience:** Solopreneurs, Non-Technical Founders, "Vibe Coders", No-Code / Low-Code Creators using v0.dev, Bolt.new, Lovable.dev, Replit, or Cursor.  
> **Objective:** Turn your English-only AI-generated web app into an 8-language global SaaS in 5 minutes without writing a single line of localization code.

---

## 1. The Vibe Coder's Localization Dilemma

You used **v0.dev**, **Bolt.new**, **Lovable**, or **Cursor** to build an incredible web app in an afternoon by simply typing prompts. It works, it looks gorgeous, and you have your first English-speaking users.

Then you realize: **75% of internet users do not speak English as their native language.** You want to expand into Latin America, Europe, Japan, and Southeast Asia.

### The Trap: Asking AI to "Translate My App"
When you prompt an AI generator: *"Translate my entire website into Spanish and Japanese"*, disaster strikes:
1. **Broken Layouts:** The AI rewrites your components, breaking flexboxes, misplacing buttons, and scrambling styling.
2. **Hallucinated Logic:** The AI forgets variables like `{userName}` or `{itemPrice}`, causing runtime crashes.
3. **Expensive Prompt Loops:** You spend hours arguing with the AI across dozens of files, burning through your token credits.
4. **Developer Jargon Overload:** Traditional tutorials tell you to install `i18next`, configure JSON namespaces, build custom routing hooks, and edit 4,000-line dictionary files.

**Glossa eliminates all of this. You don't need to know how i18n works. Glossa does it for you.**

---

## 2. The 3-Step No-Code Workflow

```
[1. Build in English]          [2. Run Glossa]                 [3. In-Context Visual Studio]
Prompt v0 / Bolt.new    ──►    One-click extract &     ──►     Alt+Click any button in browser
as you normally do             translate into 8 langs          to adjust phrasing live
```

### Step 1: Tell Your AI Builder to use `t(...)`
When generating pages in Bolt.new, v0, Lovable, or Cursor, give it this one golden sentence:

> *"Whenever you write user-facing text, wrap it in a standard translation helper like `t('Button Text')` or `<button>Button Text</button>`."*

*(Even if you forget, Glossa's compiler automatically scans raw HTML tags and JSX text nodes!)*

### Step 2: Run Glossa in 1 Click
Open your terminal (or WebContainer terminal in Bolt/Replit) and run:

```bash
# 1. Install Glossa (takes 2 seconds)
curl -fsSL https://ptyktos.com/install.sh | bash

# 2. Translate your entire project into Spanish, French, German, and Japanese
glossa init --locales es,fr,de,ja
glossa translate --provider deepseek --locales es,fr,de,ja
```

*In less than 10 seconds, your entire website is translated with zero code breakage.*

### Step 3: Launch In-Context Visual Studio (`glossa visual`)
Want to see and tweak your translations without looking at code?

```bash
glossa visual --port 3920
```

Open `http://localhost:3920` in your web browser:
1. You see your web app live.
2. Hold **Alt (Option on Mac)** and click on any headline, card, or button.
3. A sleek floating card pops up showing the translation in Spanish, French, German, and Japanese.
4. Edit the copy directly in the browser and hit **Save**.
5. Glossa updates your local files instantly.

---

## 3. Copy-Paste AI Prompts for Bolt.new, v0, and Lovable

Add these prompts to your AI generation instructions to make your apps 100% localization-ready from day one:

### Master System Prompt for New Projects
```text
You are building this web application with internationalization in mind using Glossa.
Guidelines:
- Wrap all button labels, headings, modal text, and form placeholders with clean, descriptive English text.
- Do not concatenate dynamic text in string templates (e.g. use t("Welcome {name}", { name }) instead of "Welcome " + name).
- Glossa handles all extraction, translation, and catalog synchronization automatically.
```

### Prompt to Clean Up an Existing English Project
```text
Refactor this page to prepare it for Glossa localization:
- Ensure all hardcoded button labels, navbar links, and form errors are translatable text nodes.
- Keep all Tailwind CSS styling and component layout identical.
```

---

## 4. Why Vibe Coders Choose Glossa Over Legacy Tools

| Feature | Legacy Tools (Crowdin / i18next) | Glossa for Vibe Coders |
|---|---|---|
| **Coding Knowledge Required** | High (JSON trees, routing, hooks) | **Zero (One command or Visual Click)** |
| **Setup Time** | 3–5 hours of manual wiring | **60 seconds** |
| **Cost** | $50/month per seat subscription | **100% Free / Open Source ($0)** |
| **Visual In-Browser Editing** | Requires enterprise SaaS + SDK | **Built-in (`localhost:3920`)** |
| **Risk of Breaking UI** | High (Layout and syntax errors) | **Zero (Integrity Gate prevents broken tags)** |

---

## 5. Instant Global TAM Expansion

By adding Glossa to your vibe coding workflow, you instantly expand your Total Addressable Market (TAM) from English-only (~400M native speakers) to global markets:

* 🇪🇸 **Spanish:** 500M+ native speakers (Latin America, Spain, US Hispanic market)
* 🇫🇷 **French:** 300M+ speakers (Europe, Canada, West Africa)
* 🇩🇪 **German:** 130M+ high-GDP European consumers
* 🇯🇵 **Japanese:** 125M+ premium e-commerce and B2B buyers
* 🇧🇷 **Portuguese:** 250M+ tech-savvy Brazilian users

**Launch once, translate in 10 seconds, and sell to the entire planet.**
