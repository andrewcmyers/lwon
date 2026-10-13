"""Tests of the Python API. The shared tests in tests/ cover parsing and output
through python/bin/dump."""

import io
import unittest

import lwon
from lwon import Dictionary, Grid, LWONSyntaxError


class TypesTests(unittest.TestCase):
    def test_text(self):
        self.assertEqual(lwon.loads('hello world'), 'hello world')
        self.assertEqual(lwon.loads('"a\\tb"'), 'a\tb')

    def test_dictionary_maps_keys_to_lists(self):
        self.assertEqual(lwon.loads('{a: 1, b: 2\nc: 3}'), {'a': ['1, b: 2'], 'c': ['3']})
        self.assertEqual(lwon.loads('k: x\nj: y\nk: z', implicit='dict'),
                         {'k': ['x', 'z'], 'j': ['y']})

    def test_one_dimensional_array_is_list(self):
        value = lwon.loads('[a, b, c]')
        self.assertIs(type(value), list)
        self.assertEqual(value, ['a', 'b', 'c'])

    def test_multidimensional_array_is_grid(self):
        value = lwon.loads('a, b\nc, d', implicit='array')
        self.assertIsInstance(value, Grid)
        self.assertEqual(value.ndim, 2)
        self.assertEqual(value, [['a', 'b'], ['c', 'd']])
        self.assertEqual(value[1][0], 'c')

    def test_three_dimensions(self):
        value = lwon.loads('a, b\nc, d\n\ne, f\ng, h', implicit='array')
        self.assertEqual(value.ndim, 3)
        self.assertEqual(value, [[['a', 'b'], ['c', 'd']], [['e', 'f'], ['g', 'h']]])

    def test_missing_positions_are_empty(self):
        self.assertEqual(lwon.loads('a, b\nc', implicit='array'), [['a', 'b'], ['c', '']])

    def test_array_of_arrays_is_list_of_lists(self):
        value = lwon.loads('[[a, b], [c, d]]')
        self.assertIs(type(value), list)
        self.assertEqual(value, [['a', 'b'], ['c', 'd']])

    def test_empty_array(self):
        self.assertEqual(lwon.loads('[]'), [])


class RoundTripTests(unittest.TestCase):
    def assert_round_trip(self, text):
        value = lwon.loads(text)
        self.assertEqual(lwon.loads(lwon.dumps(value)), value)
        self.assertEqual(type(lwon.loads(lwon.dumps(value))), type(value))

    def test_grid_and_array_of_arrays_differ(self):
        self.assertNotEqual(lwon.dumps(lwon.loads('[[a, b], [c, d]]')),
                            lwon.dumps(lwon.loads('[a, b\nc, d]')))
        self.assert_round_trip('[[a, b], [c, d]]')
        self.assert_round_trip('[a, b\nc, d]')

    def test_grid_of_arrays(self):
        value = lwon.loads('[[a, b]\n[c, d]]')
        self.assertEqual(value.ndim, 2)
        self.assertEqual(value, [[['a', 'b']], [['c', 'd']]])
        self.assert_round_trip('[[a, b]\n[c, d]]')

    def test_nested_values(self):
        self.assert_round_trip('{a: [1, 2\n3, 4]\nb: {"c:d": "x\\ny"}\nb: ""}')


class DumpsTests(unittest.TestCase):
    def test_hand_built_values(self):
        self.assertEqual(lwon.dumps({'name': ['Alice'], 'tags': [['x', 'y']]}),
                         '{\n  name: "Alice",\n  tags: [\n    "x", "y"\n  ]\n}')
        self.assertEqual(lwon.dumps(Grid([['1', '2'], ['3', '4']])),
                         '[\n  "1", "2"\n  "3", "4"\n]')

    def test_rejects_other_types(self):
        with self.assertRaises(TypeError):
            lwon.dumps({'n': [3]})

    def test_dump_writes_newline(self):
        out = io.StringIO()
        lwon.dump('x', out)
        self.assertEqual(out.getvalue(), '"x"\n')


class LoadsTests(unittest.TestCase):
    def test_no_value(self):
        with self.assertRaises(LWONSyntaxError):
            lwon.loads('# only a comment\n')

    def test_extra_value(self):
        with self.assertRaises(LWONSyntaxError) as cm:
            lwon.loads('a\nb')
        self.assertIn('line 2, char 1', str(cm.exception))

    def test_loads_all(self):
        self.assertEqual(list(lwon.loads_all('a\n[b]\n{c: d}')), ['a', ['b'], {'c': ['d']}])

    def test_syntax_error_location(self):
        with self.assertRaises(LWONSyntaxError) as cm:
            list(lwon.loads_all('x\n  {a: [1, 2\n', name='f'))
        self.assertTrue(str(cm.exception).startswith('"f", line 2, char 7'))
        self.assertEqual((cm.exception.location.line, cm.exception.location.column), (2, 7))

    def test_load_uses_file_name(self):
        f = io.StringIO('{a')
        f.name = 'data.lwon'
        with self.assertRaises(LWONSyntaxError) as cm:
            lwon.load(f)
        self.assertIn('"data.lwon"', str(cm.exception))


class DictionaryTests(unittest.TestCase):
    def test_parsed_entries_keep_input_order(self):
        d = lwon.loads('{a: 1\nb: 2\na: 3\nc: 4\nb: 5}')
        self.assertIsInstance(d, Dictionary)
        self.assertEqual(d, {'a': ['1', '3'], 'b': ['2', '5'], 'c': ['4']})
        self.assertEqual(list(d.entries()),
                         [('a', '1'), ('b', '2'), ('a', '3'), ('c', '4'), ('b', '5')])
        self.assertEqual(lwon.dumps(d), '{\n  a: "1",\n  b: "2",\n  a: "3",\n  c: "4",\n  b: "5"\n}')

    def test_constructor_and_add(self):
        d = Dictionary([('x', '1'), ('y', '2')])
        d.add('x', '3')
        self.assertEqual(d, {'x': ['1', '3'], 'y': ['2']})
        self.assertEqual(list(d.entries()), [('x', '1'), ('y', '2'), ('x', '3')])

    def test_entries_after_changes_through_dict_interface(self):
        d = Dictionary([('a', '1'), ('b', '2'), ('a', '3')])
        d['a'][0] = 'one'     # a changed value keeps its place
        d['b'].append('4')    # an extra value follows the recorded entries
        d['c'] = ['5']        # so does a new key
        self.assertEqual(list(d.entries()),
                         [('a', 'one'), ('b', '2'), ('a', '3'), ('b', '4'), ('c', '5')])
        d['a'].pop()          # a recorded entry with no value left is skipped
        del d['b']
        self.assertEqual(list(d.entries()), [('a', 'one'), ('c', '5')])

    def test_plain_dict_is_grouped_by_key(self):
        self.assertEqual(lwon.dumps({'a': ['1', '3'], 'b': ['2']}),
                         '{\n  a: "1",\n  a: "3",\n  b: "2"\n}')

    def test_round_trip_keeps_interleaving(self):
        text = 'k: 1\nj: 2\nk: 3'
        d = lwon.loads(text, implicit='dict')
        self.assertEqual(list(lwon.loads(lwon.dumps(d)).entries()), list(d.entries()))


class FlattenTests(unittest.TestCase):
    def test_single_values(self):
        value = lwon.loads('{a: 1\nb: {c: [x, {d: y}]}}')
        self.assertEqual(lwon.flatten(value), {'a': '1', 'b': {'c': ['x', {'d': 'y'}]}})

    def test_repeated_key_rejected_by_default(self):
        with self.assertRaisesRegex(ValueError, "'k' has 2 values"):
            lwon.flatten(lwon.loads('{k: 1\nk: 2}'))

    def test_reduce(self):
        value = lwon.loads('{k: 1\nk: 2\nj: 3}')
        self.assertEqual(lwon.flatten(value, reduce=lambda vs: vs[-1]), {'k': '2', 'j': '3'})
        self.assertEqual(lwon.flatten(value, reduce=list), {'k': ['1', '2'], 'j': ['3']})

    def test_grid_stays_grid(self):
        value = lwon.flatten(lwon.loads('[{a: 1}, b\nc, d]'))
        self.assertIsInstance(value, Grid)
        self.assertEqual(value, [[{'a': '1'}, 'b'], ['c', 'd']])


if __name__ == '__main__':
    unittest.main()
