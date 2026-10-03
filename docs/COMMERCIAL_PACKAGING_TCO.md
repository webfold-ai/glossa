# Glossa Commercial Packaging and TCO Scenarios

> **Status: HYPOTHESIS, not a measurement.** This document contains an illustrative cost model, not customer evidence, a measured savings claim, or a quote. No customer invoices, repository workloads, translation volumes, or staffing costs were used to produce a measured TCO.

## Published competitor prices

Prices below are vendor-published list prices checked on 2026-10-03. They are reference prices, not estimates of a particular customer's bill. Taxes, usage, contract terms, discounts, and currency conversion may change the amount paid.

| Product | Published price | Billing basis / notes |
|---|---|---|
| Lokalise Explorer | $149/month | Per organization, not per seat |
| Lokalise Growth | $379/month | Per organization, not per seat |
| Lokalise Advanced | $1,049/month | Per organization, not per seat |
| Tolgee Translate | €58/month | Annual billing |
| Tolgee Optimise | €133/month | Annual billing |
| Tolgee Manage | €373/month | Annual billing |
| Lingo.dev Sandbox | $0 | Sandbox tier |
| Lingo.dev Production | $99/month + $2/MTok infrastructure + pass-through LLM cost | Usage and model charges vary |
| Crowdin | Quote / unverified | No published price verified for this comparison |
| Phrase | Quote / unverified | No published price verified for this comparison |
| Weblate Cloud | Unverified | No published price verified for this comparison |

## Illustrative scenario (ESTIMATE)

The following scenario inputs are **ESTIMATES** only: a 35-person organization, a codebase of approximately 3,500 files, 8 target languages, and 120 pull requests per week. They are not observed customer data.

| Cost item | Current TMS / workflow | With Glossa in workflow | Status and evidence needed |
|---|---|---|---|
| TMS subscription | Use the relevant vendor's published plan and actual quote; do not multiply by seats for Lokalise | No assumed removal of an existing TMS | **ESTIMATE** until verified against an invoice and selected plan |
| Translation / LLM usage | **ESTIMATE** based on actual monthly volume, provider, and model | **ESTIMATE** based on the same usage and measured reuse | Measure tokens and spend on the same real repository and period |
| CI runtime | **ESTIMATE** | **ESTIMATE** | Time matched CI runs with equivalent checks and runners |
| Setup and maintenance | **ESTIMATE** | **ESTIMATE** | Record implementation and ongoing engineering hours |
| Defect remediation | Not included | Not included | No evidence supports a dollar value or assumed elimination |

No annual total or savings percentage is stated because the input costs and Glossa operating costs have not been measured. Any future scenario total must label each modeled amount **ESTIMATE** and disclose its source or calculation.

The majority of the earlier claimed saving came from assuming the organization stops buying a TMS. That is a workflow and procurement decision, not a Glossa performance property. Glossa does not replace translation hosting, translator collaboration, or vendor management; an organization that keeps its TMS should not count the subscription as a Glossa saving.

## Evidence required before making a savings claim

Run the same defined workflow on a real repository and record the vendor invoice, language and string counts, model and token usage, CI wall time, setup and maintenance time, and defects found. Keep actual charges separate from estimates. Do not assign a monetary value to avoided quality incidents without customer evidence.
