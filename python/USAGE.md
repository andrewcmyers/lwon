# Python LWON library

## Examples

Here are some simple examples of how to use LWON from Python:

```python
import lwon

value = lwon.loads(text)                     # a single LWON value
table = lwon.loads(text, implicit="array")   # an implicit array, e.g., CSV
config = lwon.loads(text, implicit="dict")   # an implicit dictionary
for value in lwon.loads_all(text):           # a sequence of LWON values
    ...
with open("data.lwon", encoding="utf-8", newline="") as f:
    value = lwon.load(f)                     # also lwon.load_all(f)
print(lwon.dumps(value))                     # also lwon.dump(value, f)
```

Open files with `newline=""` so that line terminators reach the parser
unchanged. Invalid input raises `lwon.LWONSyntaxError`, whose message
begins with the location of the error.

## Data representations

LWON values are represented by built-in types, some of which
are subclassed to add information:

- Text is a `str`.
- A one-dimensional array is a `list`. Its elements may themselves be
  arrays.
- An array of two or more dimensions is an `lwon.Grid`, a `list` of
  nested lists with the first dimension outermost, and with its number of
  dimensions in its `ndim` attribute. For example, the rows `a, b` and
  `c, d` become `[["a", "b"], ["c", "d"]]`. To write nested lists as a
  multidimensional array rather than as an array of arrays, wrap them in
  a `Grid`: `lwon.Grid(rows)`.
- A dictionary is an `lwon.Dictionary`, a `dict` mapping each key to the
  list of its values, so that `{name: Alice}` becomes
  `{"name": ["Alice"]}`. It also records the order of all its entries,
  which its `entries()` method yields as (key, value) pairs.

When keys are not repeated, `lwon.flatten(value)` converts every
dictionary contained in `value` to a `dict` mapping each key to its
single value, raising `ValueError` if a key has more than one. Its
optional `reduce` parameter specifies how to combine multiple values; for
example, `lwon.flatten(value, reduce=lambda values: values[-1])` keeps
the last value of a key.

#### Tools and Tests

The command `lwon-dump [-a] [-d] file...`, installed by `pip`, works
like `bin/dump`; so do `python3 -m lwon` and, without installation,
`python/bin/dump`.

Python tests can be run using `bin/run-tests python/bin/dump` (the
shared tests in `tests/`) and
`cd python && python3 -m unittest discover -s tests`. 

