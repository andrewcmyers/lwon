"""Reading and writing LWON (Light-Weight Object Notation).

LWON values are represented by built-in Python types and subclasses of them:

- text is a str;
- a one-dimensional array is a list, whose elements may themselves be arrays;
- an array of two or more dimensions is a Grid, a list of nested lists;
- a dictionary is a Dictionary, a dict mapping each key to the list of its
  values in input order, which also records the order of all entries. A key
  that is not repeated maps to a list of one value. flatten() converts
  dictionaries to dicts mapping each key to a single value.
"""

from __future__ import annotations

from typing import Callable, Iterator, Optional, TextIO

from ._parser import Parser
from ._scanner import EOF, Scanner
from ._types import Dictionary, Grid, LWONSyntaxError
from ._unparse import NEWLINE, unparse

__all__ = ['Dictionary', 'Grid', 'LWONSyntaxError', 'dump', 'dumps', 'flatten', 'load', 'load_all',
           'loads', 'loads_all']


def loads_all(text: str, implicit: Optional[str] = None, name: str = '<string>') -> Iterator:
    """Yield each LWON value in text, in order. If implicit is "array" or "dict",
    each value is an implicit array or dictionary, without enclosing brackets
    or braces. name identifies text in error messages.
    Raises LWONSyntaxError if text is not valid LWON."""
    parser = Parser(Scanner(text, name))
    parse = _parse_method(parser, implicit)
    while True:
        try:
            value = parse()
        except EOF:
            return
        yield value


def _parse_method(parser: Parser, implicit: Optional[str]) -> Callable:
    return {None: parser.parse, 'array': parser.parse_implicit_array,
            'dict': parser.parse_implicit_dictionary}[implicit]


def loads(text: str, implicit: Optional[str] = None, name: str = '<string>'):
    """The single LWON value in text. implicit and name are as for loads_all.
    Raises LWONSyntaxError if text is not valid LWON or does not contain
    exactly one value."""
    parser = Parser(Scanner(text, name))
    parse = _parse_method(parser, implicit)
    try:
        value = parse()
    except EOF:
        raise LWONSyntaxError(f'"{name}": No LWON value') from None
    extra = parser.next_location()
    if extra is not None:
        raise LWONSyntaxError('Unexpected data after the value', extra)
    return value


def load_all(fp: TextIO, implicit: Optional[str] = None, name: Optional[str] = None) -> Iterator:
    """Yield each LWON value read from fp, as for loads_all. name defaults to
    fp.name if it exists. fp should be opened with newline='' so that line
    terminators are not translated."""
    return loads_all(fp.read(), implicit, _name(fp, name))


def load(fp: TextIO, implicit: Optional[str] = None, name: Optional[str] = None):
    """The single LWON value read from fp, as for loads. name and fp are as
    for load_all."""
    return loads(fp.read(), implicit, _name(fp, name))


def _name(fp: TextIO, name: Optional[str]) -> str:
    if name is not None:
        return name
    return str(getattr(fp, 'name', '<stream>'))


def dumps(value) -> str:
    """The LWON text for value, a str, list, Grid, or dict mapping str keys to
    lists of values, such as a Dictionary. The entries of a Dictionary are
    written in the order given by Dictionary.entries(); those of another dict,
    grouped by key in dict order.
    Raises TypeError if value or a value within it is of another type."""
    b: list[str] = []
    unparse(value, b, 0)
    return ''.join(b)


def dump(value, fp: TextIO) -> None:
    """Write the LWON text for value to fp, followed by a newline, as for dumps."""
    fp.write(dumps(value) + NEWLINE)


def flatten(value, reduce: Optional[Callable[[list], object]] = None):
    """A copy of value in which each dictionary, at any depth, maps each key to a
    single value: reduce applied to the list of the key's (flattened) values.
    For example, reduce=lambda values: values[-1] keeps the last value. If
    reduce is None, each key must have exactly one value, and ValueError is
    raised otherwise."""
    if isinstance(value, dict):
        result = {}
        for key, values in value.items():
            flat = [flatten(v, reduce) for v in values]
            if reduce is not None:
                result[key] = reduce(flat)
            elif len(flat) == 1:
                result[key] = flat[0]
            else:
                raise ValueError(f'key {key!r} has {len(flat)} values')
        return result
    if isinstance(value, Grid):
        return Grid((flatten(v, reduce) for v in value), value.ndim)
    if isinstance(value, list):
        return [flatten(v, reduce) for v in value]
    return value
