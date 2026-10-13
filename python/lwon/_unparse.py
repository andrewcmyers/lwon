"""Conversion of Python values to LWON text, in the same format as the Java
implementation."""

from __future__ import annotations

import unicodedata

from ._types import Dictionary, Grid

NEWLINE = '\n'
_JSON_ESCAPES = {
    '"': '\\"', '\\': '\\\\', '/': '\\/',
    '\b': '\\b', '\n': '\\n', '\t': '\\t', '\f': '\\f', '\r': '\\r',
}
_RESERVED_START = '{}[]:"|$+\\#'
_RESERVED_ANYWHERE = '{}[]:"\\'


def unparse(value, b: list[str], indent: int) -> None:
    """Append the LWON text for value to b, with nested lines indented by
    indent levels. value is a str, a list (a one-dimensional array), a Grid,
    or a dict mapping str keys to lists of values. The entries of a Dictionary
    are written in the order given by its entries() method; those of another
    dict, grouped by key in dict order.
    Raises TypeError if value or a value within it is of another type."""
    if isinstance(value, str):
        b.append('"' + _escape_json(value) + '"')
    elif isinstance(value, Grid):
        _unparse_array(value, _dimensions(value, value.ndim), b, indent)
    elif isinstance(value, list):
        _unparse_array(value, [len(value)], b, indent)
    elif isinstance(value, dict):
        _unparse_dictionary(value, b, indent)
    else:
        raise TypeError(f'Cannot convert {type(value).__name__} to LWON')


def _escape_json(s: str) -> str:
    out = []
    for c in s:
        if c in _JSON_ESCAPES:
            out.append(_JSON_ESCAPES[c])
        elif ord(c) < 0x20 or ord(c) > 0x7f:
            out.append(_unicode_escape(c))
        else:
            out.append(c)
    return ''.join(out)


def _unicode_escape(c: str) -> str:
    """c as \\uXXXX escapes of its UTF-16 code units."""
    data = c.encode('utf-16-be', 'surrogatepass')
    return ''.join('\\u%04X' % int.from_bytes(data[i:i + 2], 'big')
                   for i in range(0, len(data), 2))


def _dimensions(grid: list, ndim: int) -> list[int]:
    dims = []
    level = grid
    for _ in range(ndim):
        dims.append(len(level))
        level = level[0] if level else []
    return dims


def _unparse_array(array: list, dims: list[int], b: list[str], indent: int) -> None:
    ndim = len(dims)
    b.append('[' + NEWLINE)
    if ndim == 1 and dims[0] == 0:
        b.append('  ' * indent + ']')
        return
    indices = [0] * ndim
    first = True
    while True:
        b.append('  ' * (indent + 1) if first else ' ')
        first = False
        element = array
        for i in indices:
            element = element[i]
        unparse(element, b, indent + 1)
        newlines = 0
        i = ndim - 1
        while i >= 0:
            indices[i] += 1
            if indices[i] < dims[i]:
                if i == ndim - 1:
                    b.append(',')
                break
            newlines += 1
            first = True
            for j in range(i, ndim):
                indices[j] = 0
            i -= 1
        if i < 0:
            break
        b.append(NEWLINE * newlines)
    b.append(NEWLINE + '  ' * indent + ']')


def _unparse_dictionary(d: dict, b: list[str], indent: int) -> None:
    b.append('{')
    first = True
    entries = d.entries() if isinstance(d, Dictionary) else (
        (key, value) for key, values in d.items() for value in values)
    for key, value in entries:
        if not first:
            b.append(',')
        first = False
        b.append(NEWLINE + '  ' * (indent + 1))
        b.append('"' + _escape_json(key) + '"' if _needs_quotes(key) else key)
        b.append(': ')
        unparse(value, b, indent + 1)
    b.append(NEWLINE + '  ' * indent + '}')


def _needs_quotes(key: str) -> bool:
    """Whether key must be quoted to be read back unchanged."""
    if key == '':
        return True
    if _is_java_whitespace(key[0]) or _is_java_whitespace(key[-1]):
        return True
    if key[0] in _RESERVED_START:
        return True
    return any(c in _RESERVED_ANYWHERE or _is_iso_control(c) for c in key)


def _is_java_whitespace(c: str) -> bool:
    """Whether c is whitespace as defined by Java's Character.isWhitespace."""
    if c in ' \t\n\x0b\f\r\x1c\x1d\x1e\x1f':
        return True
    return unicodedata.category(c) in ('Zs', 'Zl', 'Zp') and c not in '   '


def _is_iso_control(c: str) -> bool:
    return ord(c) <= 0x1f or 0x7f <= ord(c) <= 0x9f
