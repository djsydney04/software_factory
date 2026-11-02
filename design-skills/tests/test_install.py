import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('installer', Path(__file__).parents[1] / 'install.py')
installer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(installer)

class Installation(unittest.TestCase):
    def test_install_and_repeat(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            skill = base / 'source' / 'example'
            skill.mkdir(parents=True)
            (skill / 'SKILL.md').write_text('example')
            for _ in range(2):
                self.assertEqual(installer.install(base / 'source', base / 'destination'), 0)
            self.assertEqual((base / 'destination/example').resolve(), skill.resolve())

    def test_conflict_preserves_existing_and_makes_no_partial_install(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            for name in ['a', 'b']:
                skill = base / 'source' / name
                skill.mkdir(parents=True)
                (skill / 'SKILL.md').write_text(name)
            destination = base / 'destination'
            destination.mkdir()
            (destination / 'b').write_text('existing')
            self.assertEqual(installer.install(base / 'source', destination), 1)
            self.assertEqual((destination / 'b').read_text(), 'existing')
            self.assertFalse((destination / 'a').exists())
