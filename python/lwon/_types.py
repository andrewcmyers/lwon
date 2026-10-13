"""Public types of the lwon package."""

from __future__ import annotations

from typing import Iterable, Iterator, Optional

from ._scanner import Location


class Grid(list):
    """A multidimensional LWON array of ndim dimensions (at least 2), stored as
    nested lists with the first dimension outermost. For example, the rows
    "a, b" and "c, d" form Grid([["a", "b"], ["c", "d"]]).

    A plain list is a one-dimensional array, whose elements may themselves be
    arrays. A Grid is a list, and is indexed like nested lists.

    Requires: rows is nested to depth ndim, and the lists at each depth all
    have the same length.
    """

    __module__ = 'lwon'

    def __init__(self, rows: Iterable = (), ndim: int = 2):
        super().__init__(rows)
        self.ndim = ndim

    def __repr__(self) -> str:
        return f'Grid({list.__repr__(self)}, ndim={self.ndim})'


class Dictionary(dict):
    """An LWON dictionary: a dict mapping each key to the list of its values, which
    also records the order of its entries (key-value pairs), including entries
    with repeated keys. For example, the entries a: 1, b: 2, a: 3 form the dict
    {"a": ["1", "3"], "b": ["2"]}, with entries in that order.

    The recorded order is that of the pairs given to the constructor and add().
    Changes made through the dict interface are not recorded; entries()
    accounts for them.
    """

    __module__ = 'lwon'

    def __init__(self, pairs: Iterable[tuple[str, object]] = ()):
        super().__init__()
        self._order: list[str] = []
        for key, value in pairs:
            self.add(key, value)

    def add(self, key: str, value) -> None:
        """Add the entry key: value after all recorded entries."""
        self.setdefault(key, []).append(value)
        self._order.append(key)

    def entries(self) -> Iterator[tuple[str, object]]:
        """Yield each entry as a (key, value) pair. The n-th recorded entry for a
        key is yielded in recorded order, as the key's n-th value, if the key
        still has an n-th value. Then any values not yet yielded follow, grouped
        by key in dict order."""
        counts: dict[str, int] = {}
        for key in self._order:
            n = counts.get(key, 0)
            values = self.get(key, [])
            if n < len(values):
                counts[key] = n + 1
                yield key, values[n]
        for key, values in self.items():
            for value in values[counts.get(key, 0):]:
                yield key, value

    def __repr__(self) -> str:
        return f'Dictionary({list(self.entries())!r})'


class LWONSyntaxError(ValueError):
    """The input is not valid LWON. The message begins with the location of the
    error, as '"name", line L, char C'."""

    __module__ = 'lwon'

    def __init__(self, msg: str, location: Optional[Location] = None):
        super().__init__(f'{location}: {msg}' if location else msg)
        self.location = location
