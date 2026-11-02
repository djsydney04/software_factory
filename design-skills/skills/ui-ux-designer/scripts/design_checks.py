#!/usr/bin/env python3
"""Read-only checks for opaque sRGB tokens, layout budgets and design handoffs."""
import argparse
import json
import math
from pathlib import Path
import re
import sys


def number(value, name, minimum=0):
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < minimum:
        raise ValueError(f'{name} must be a finite number >= {minimum}')
    return value


def luminance(color):
    if not isinstance(color, str) or not re.fullmatch(r'#[0-9a-fA-F]{6}', color):
        raise ValueError(f'Expected opaque #RRGGBB, got {color!r}; composite alpha before checking')
    rgb = [int(color[i:i + 2], 16) / 255 for i in (1, 3, 5)]
    linear = [v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4 for v in rgb]
    return sum(v * w for v, w in zip(linear, (0.2126, 0.7152, 0.0722)))


def contrast(foreground, background):
    a, b = sorted((luminance(foreground), luminance(background)))
    return (b + 0.05) / (a + 0.05)


def tokens(data):
    colors = data['colors']
    for value in colors.values():
        luminance(value)
    pairs = data['contrast_pairs']
    if not pairs:
        raise ValueError('Declare at least one actual foreground/background pair')
    results = []
    for pair in pairs:
        minimum = number(pair.get('minimum', 4.5), 'minimum', 1)
        if minimum > 21:
            raise ValueError('minimum cannot exceed 21')
        ratio = contrast(colors[pair['foreground']], colors[pair['background']])
        results.append({'name': pair['name'], 'ratio': round(ratio, 4), 'minimum': minimum, 'pass': ratio >= minimum})
    return {'pass': all(r['pass'] for r in results), 'pairs': results,
            'limit': 'Opaque declared pairs only; not a rendered accessibility audit. Threshold comparison uses unrounded values.'}


def layout(data):
    viewport = number(data['viewport_width'], 'viewport_width', 1)
    fixed = data.get('fixed_regions', [])
    fixed_width = sum(number(r['width'], 'region width') for r in fixed)
    padding = number(data.get('content_padding', 0), 'content_padding') * 2
    region_gaps = number(data.get('region_gap', 0), 'region_gap') * len(fixed)
    available = viewport - fixed_width - padding - region_gaps
    columns = data['columns']
    if not columns:
        raise ValueError('Declare at least one column')
    needed = sum(number(c['min_width'], 'column min_width', 1) + 2 * number(c.get('padding', 0), 'column padding') for c in columns)
    needed += number(data.get('column_gap', 0), 'column_gap') * max(0, len(columns) - 1)
    return {'pass': available >= needed, 'available': available, 'required': needed, 'remaining': available - needed,
            'limit': 'Arithmetic budget, not font measurement. On overflow collapse secondary regions or columns before shrinking text.'}


def identifiers(items, label):
    values = [item['id'] for item in items]
    if any(not isinstance(v, str) or not v.strip() for v in values) or len(set(values)) != len(values):
        raise ValueError(f'{label} IDs must be nonempty unique strings')
    return set(values)


def handoff(data):
    screens = data['screens']
    if not screens:
        raise ValueError('At least one screen is required')
    ids = identifiers(screens, 'screen')
    interactions = data['interactions']
    if not interactions:
        raise ValueError('At least one interaction is required')
    identifiers(interactions, 'interaction')
    errors = []
    for screen in screens:
        if not isinstance(screen.get('scope'), str) or not screen['scope'].strip():
            errors.append(f"{screen['id']}: missing scope")
        for field in ('required_states', 'designed_states'):
            values = screen[field]
            if not isinstance(values, list) or not values or any(not isinstance(v, str) or not v.strip() for v in values) or len(set(values)) != len(values):
                raise ValueError(f'{field} must contain unique nonempty state names')
        missing = set(screen['required_states']) - set(screen['designed_states'])
        if missing:
            errors.append(f"{screen['id']}: undesigned states {', '.join(sorted(missing))}")
    for item in interactions:
        if item['screen'] not in ids:
            errors.append(f"{item['id']}: unknown source screen")
        if 'target' in item and item['target'] not in ids:
            errors.append(f"{item['id']}: unknown target screen")
        for field in ('trigger', 'result', 'keyboard'):
            if not isinstance(item.get(field), str) or not item[field].strip():
                errors.append(f"{item['id']}: missing {field}")
    return {'pass': not errors, 'errors': errors,
            'limit': 'Completeness of declared contracts only; does not execute controls or prove usability.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('check', choices=('tokens', 'layout', 'handoff'))
    parser.add_argument('file', type=Path)
    args = parser.parse_args()
    try:
        data = json.loads(args.file.read_text())
        report = {'tokens': tokens, 'layout': layout, 'handoff': handoff}[args.check](data)
    except (OSError, ValueError, TypeError, KeyError, AttributeError) as exc:
        print(json.dumps({'pass': False, 'error': str(exc)}))
        return 2
    print(json.dumps(report, indent=2))
    return 0 if report['pass'] else 1


if __name__ == '__main__':
    sys.exit(main())
