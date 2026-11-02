# Read-only design checks

Python 3 standard library only. Run from the ui-ux-designer directory, or resolve these paths against its location:

```sh
python3 scripts/design_checks.py tokens assets/tokens.json
python3 scripts/design_checks.py layout assets/layout.json
python3 scripts/design_checks.py handoff assets/handoff.json
```

Each prints JSON. Exit 0 means declared checks pass, 1 means a check failed, 2 means invalid input. Files are never changed. Copy examples to your project and replace them with actual design values; passing the examples does not validate your design.

## Tokens input
`colors` maps semantic names to opaque #RRGGBB. `contrast_pairs` is a nonempty list with `name`, `foreground` and `background` color keys, and numeric `minimum` (default 4.5). Include every actual text/surface and applicable UI pair. The checker uses unrounded WCAG ratios, rejects alpha colors, and cannot discover missing pairs from screenshots.

## Layout input
`viewport_width` is CSS px. `fixed_regions` is a list of side-by-side ancillary regions with `width`; each adds one `region_gap` to the main region. `content_padding` is per side of the main region. `columns` is a nonempty list with `min_width` for content plus optional per-side `padding`; `column_gap` separates columns. All numbers must be finite and nonnegative, with positive viewport/column width.

Available = viewport − sum(fixed widths) − 2×content padding − region gap×fixed-region count.
Required = sum(column minimum + 2×column padding) + column gap×(column count−1).

Submit a separate input for each responsive layout after its intended pane collapse. This checks horizontal arithmetic only: no automatic collapse choice, font measurement, vertical fit or line-length checks. Inspect actual rendering afterward.

## Handoff input
`screens` is a nonempty list of unique `id`, textual `scope`, nonempty unique `required_states` and `designed_states`. `interactions` is a nonempty list of unique `id`, source `screen`, optional destination `target` screen ID, and nonempty `trigger`, `result`, `keyboard` descriptions.

Checks detect unknown destinations, missing declared states and incomplete descriptions. They do not verify truth, execute routes, enforce backend preconditions or measure usability. Keep a human-readable action/state table and test it in the product. Do not treat arbitrary placeholder descriptions as a successful design review.
