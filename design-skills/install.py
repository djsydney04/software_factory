#!/usr/bin/env python3
"""Link this complete suite into the personal skill directory without overwrites."""
from pathlib import Path
import sys

def install(source, destination):
    sources = sorted(source.glob('*/SKILL.md'))
    conflicts = []
    for skill in sources:
        target = destination / skill.parent.name
        if (target.exists() or target.is_symlink()) and target.resolve() != skill.parent.resolve():
            conflicts.append(str(target))
    if conflicts:
        print('No changes made. Existing skills conflict:\n' + '\n'.join(conflicts))
        return 1
    destination.mkdir(parents=True, exist_ok=True)
    for skill in sources:
        target = destination / skill.parent.name
        if not target.is_symlink() and not target.exists():
            target.symlink_to(skill.parent.resolve(), target_is_directory=True)
        print(f'Installed {skill.parent.name}: {target}')
    return 0

if __name__ == '__main__':
    sys.exit(install(Path(__file__).resolve().parent / 'skills', Path.home() / '.codex' / 'skills'))
