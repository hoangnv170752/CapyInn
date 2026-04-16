# CapyInn README Refresh Design

Date: 2026-04-16
Owner: Codex
Status: Draft approved for spec write-up

## Goal

Refresh the public-facing Vietnamese README for CapyInn before wider open-source launch, using `readme-ai` as a draft-generation tool while keeping the final README intentionally edited by hand.

The result should feel polished and trustworthy for a public repo:

- clear value proposition in the first screenful
- accurate setup and verification instructions
- honest limitations
- stronger visual structure than the current README
- no generic AI-template voice

This pass only targets the Vietnamese README at the repository root:

- target file: `README.md`
- non-goal for this pass: substantial rewrite of `README.en.md`

## Constraints

- `readme-ai` must not be installed or run inside the CapyInn repository.
- Tooling should live in a separate local directory: `/Users/binhan/readme-ai`
- Any generated output must go to a temporary draft file first, not directly overwrite `README.md`
- Final content should be manually curated before commit
- The repo should not gain persistent generator-specific clutter unless it has clear long-term value

## Chosen Approach

Use `readme-ai` as a structure-and-scaffolding engine, then rewrite the final Vietnamese README manually.

Why this approach:

- `readme-ai` is useful for fast section scaffolding, repository tree extraction, and setup/testing structure
- raw generated output is likely too generic for a niche offline-first hotel PMS targeting Vietnam
- CapyInn needs a README with product voice and project honesty, not a generic auto-generated template

This is explicitly not a “generate and commit as-is” workflow.

## Scope

### In scope

- install or prepare `readme-ai` in `/Users/binhan/readme-ai`
- generate one Vietnamese-oriented draft from the local CapyInn repo
- review generated content against current README
- rewrite `README.md` into a more polished public-facing Vietnamese README
- verify repository links, setup steps, test commands, and repo layout text
- preserve consistency with current app naming: `CapyInn`

### Out of scope

- broad rewrite of `README.en.md`
- screenshots, GIFs, or demo media production
- adding CI automation for README generation
- introducing a permanent README generation pipeline
- changing product behavior or app code

## Recommended `readme-ai` Presentation Settings

The draft generation should target this style direction:

- header style: `compact`
- navigation style: `accordion`
- badge style: `for-the-badge`
- visual tone: clean, modern, restrained
- emoji usage: limited and deliberate, not decorative overload

Reasoning:

- `compact` is a better fit for a desktop app repo than a more flamboyant “showcase” header
- `accordion` keeps long README navigation readable on GitHub without feeling too noisy
- `for-the-badge` gives enough visual separation for stack badges without requiring custom assets

## Final README Direction

The final Vietnamese README should be optimized for a GitHub visitor who lands on the repo with little prior context.

Recommended content order:

1. Project identity and one-line value proposition
2. Short “why this exists” framing
3. Core features, compressed into a readable overview
4. Tech stack and system requirements
5. Local setup and development commands
6. Verification commands
7. Project structure
8. Known limitations
9. Contribution, security, and license links

## Content Principles

The rewritten README should:

- lead with “offline-first PMS for mini hotels in Vietnam”
- stay concrete and operational, not aspirational fluff
- prefer short sections over long promotional paragraphs
- keep commands copy-pasteable and current
- avoid over-claiming platform support
- remove stale naming such as `Hotel-Manager/`
- keep the note about clean-slate rename from `MHM` when it materially helps operators

## Generation Workflow

1. Prepare `readme-ai` in `/Users/binhan/readme-ai`
2. Run it against the local repository path `/Users/binhan/HotelManager`
3. Write output to a temporary draft file, for example `README.generated.vi.md`
4. Compare generated structure against current `README.md`
5. Rewrite and polish the real `README.md`
6. Verify links, clone path, commands, and headings
7. Commit only the intended README-related changes

## Success Criteria

This pass is successful if:

- `README.md` is clearly stronger than the current version on first read
- the README reads like CapyInn, not like a generic generated template
- clone/setup/test commands are correct for the current repo
- public repo references use `CapyInn`
- there is no generator detritus left in the project repo unless explicitly chosen

## Risks And Mitigations

### Risk: raw generated content feels generic

Mitigation:
- treat generator output as draft-only
- manually compress or replace generic sections

### Risk: generated structure includes stale repo assumptions

Mitigation:
- manually validate clone URL, repo path, runtime notes, and project tree

### Risk: README becomes too long

Mitigation:
- favor concise sections
- cut low-signal repetition before shipping

## Deliverables

- refreshed `README.md` in Vietnamese
- optional temporary generated draft during execution, not required to remain in repo
- no required changes to app code
