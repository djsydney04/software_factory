import importlib.util
from pathlib import Path
import unittest

MODULE = Path(__file__).parents[1] / 'skills/ui-ux-designer/scripts/design_checks.py'
spec = importlib.util.spec_from_file_location('design_checks', MODULE)
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


class DesignChecks(unittest.TestCase):
    def test_contrast_known_endpoints(self):
        self.assertEqual(d.contrast('#000000', '#FFFFFF'), 21)
        self.assertEqual(d.contrast('#ABCDEF', '#ABCDEF'), 1)
        self.assertAlmostEqual(d.contrast('#777777', '#FFFFFF'), 4.478089453577214, places=8)

    def test_no_rounding_into_pass(self):
        result = d.tokens({'colors': {'text': '#777777', 'bg': '#FFFFFF'}, 'contrast_pairs': [{'name': 'body', 'foreground': 'text', 'background': 'bg'}]})
        self.assertFalse(result['pass'])

    def test_invalid_alpha_and_color(self):
        for value in ('#00000080', 'red', '#GGGGGG', None):
            with self.assertRaises(ValueError): d.luminance(value)

    def test_layout_exact_fit_and_overflow(self):
        data = {'viewport_width': 800, 'fixed_regions': [{'width': 200}], 'content_padding': 20, 'region_gap': 10, 'columns': [{'min_width': 250, 'padding': 10}, {'min_width': 270, 'padding': 5}]}
        self.assertTrue(d.layout(data)['pass'])
        data['viewport_width'] = 799
        self.assertFalse(d.layout(data)['pass'])

    def test_invalid_dimensions(self):
        for value in (-1, float('inf'), True, '1440'):
            with self.assertRaises(ValueError): d.layout({'viewport_width': value, 'columns': [{'min_width': 100}]})

    def contract(self):
        return {'screens': [{'id': 'list', 'scope': 'project', 'required_states': ['ready','empty'], 'designed_states': ['ready','empty']}, {'id': 'detail', 'scope': 'run', 'required_states': ['running'], 'designed_states': ['running']}], 'interactions': [{'id': 'open', 'screen': 'list', 'target': 'detail', 'trigger': 'Activate run link', 'result': 'Open selected run', 'keyboard': 'Tab then Enter'}]}

    def test_handoff_links_and_states(self):
        data = self.contract()
        self.assertTrue(d.handoff(data)['pass'])
        data['screens'][0]['designed_states'] = ['ready']
        data['interactions'][0]['target'] = 'missing'
        self.assertEqual(len(d.handoff(data)['errors']), 2)

    def test_duplicate_ids(self):
        data = self.contract()
        data['screens'].append(data['screens'][0])
        with self.assertRaises(ValueError): d.handoff(data)

    def test_empty_contract_cannot_pass(self):
        with self.assertRaises(ValueError): d.handoff({'screens': [], 'interactions': []})
        with self.assertRaises(ValueError): d.tokens({'colors': {}, 'contrast_pairs': []})

    def test_missing_keyboard(self):
        data = self.contract(); del data['interactions'][0]['keyboard']
        self.assertFalse(d.handoff(data)['pass'])


if __name__ == '__main__': unittest.main()
