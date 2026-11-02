# UI/UX design skill suite

Seven reusable skills for reference analysis, compact workspaces, agent workflows, visual systems, editorial pages and review. Start with `$ui-ux-designer`; it routes to the relevant modules. Each module can also be invoked directly. The suite is source-brand-neutral and includes an optional soft-charcoal operating profile.

## Install

Requires Python 3 for the helper checks. Skill instructions are ordinary Markdown and can be used by compatible agents. Install the complete suite to preserve sibling links:

```sh
python3 design-skills/install.py
```

This creates links in `~/.codex/skills` to this checkout. Existing conflicting skills are left untouched and reported; matching links are safe to rerun. Keep the checkout in place. New skills may require a fresh agent session to appear in its catalog. Other agents can read `design-skills/skills/ui-ux-designer/SKILL.md` directly or install these seven directories in their own supported skill location.

## Use

“Use $ui-ux-designer to refine our project dashboard, retaining its existing functionality.”

“Use $ui-ux-reference-study to extract transferable patterns from these references.”

“Use $ui-ux-review to review this implementation and separate verified issues from untested behavior.”

The main skill includes a read-only Python checker with `tokens`, `layout` and `handoff` commands. See its `references/checks.md` for schemas and limitations. Checks validate declared inputs; they are not browser tests or an accessibility certification. Source captures and external-agent session logs are not bundled.

## Validation

```sh
python3 -m unittest discover -s design-skills/tests -v
python3 design-skills/skills/ui-ux-designer/scripts/design_checks.py tokens design-skills/skills/ui-ux-designer/assets/tokens.json
python3 design-skills/skills/ui-ux-designer/scripts/design_checks.py layout design-skills/skills/ui-ux-designer/assets/layout.json
python3 design-skills/skills/ui-ux-designer/scripts/design_checks.py handoff design-skills/skills/ui-ux-designer/assets/handoff.json
```

Behavioral acceptance prompts are in `references/behavioral-tests.md`. They cover run drilldown, a light-theme counterexample, editorial launch and flawed reference captures.
