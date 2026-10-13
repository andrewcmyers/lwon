"""A parser for LWON, following the Java implementation's lwon.parse.Parser."""

from __future__ import annotations

import os
from typing import Optional

from ._scanner import EOF, Location, Scanner
from ._types import Dictionary, Grid, LWONSyntaxError
from ._unparse import escape

UNEXPECTED_EOF = 'Unexpected end of input'
RESERVED_CHARS = '{}[]:'
BLANKS = ' \t'
LINE_TERMINATORS = '\n\r\f'
WHITESPACE = BLANKS + LINE_TERMINATORS
HEX_DIGITS = '0123456789abcdef'
ESCAPES = {
    'r': '\r', 'n': '\n', ' ': ' ', 'f': '\f', 'b': '\b', 't': '\t',
    '\\': '\\', '/': '/', '"': '"', "'": "'",
    '}': '}', ']': ']', '#': '#', ':': ':', '\n': '',
}


def _is_in(c: str, chars: str) -> bool:
    """Whether c is a character (not end of input) in chars."""
    return c != '' and c in chars


def _is_newline(c: str) -> bool:
    """Whether c is the first character of a line terminator."""
    return c == '\n' or c == '\r'


def _unparse_char(c: str) -> str:
    """c, escaped as in an LWON string except for "/", for use in error messages."""
    return c if c == '/' else escape(c)


class Parser:
    """Parses LWON values from a Scanner. Dictionaries are parsed as Dictionary
    objects recording their entries in input order; arrays as lists (one
    dimension) or Grids (more than one); text as str."""

    def __init__(self, scanner: Scanner):
        self.scanner = scanner

    def parse(self):
        """Parse the next value from the input. All prior positions on the current
        line, if any, are treated as whitespace. Raises EOF if there is no value
        and LWONSyntaxError if there is a syntax error."""
        self.skip_comments()
        return self._parse('')

    def parse_implicit_dictionary(self) -> Dictionary:
        """Parse an implicit dictionary, which is not enclosed in braces. Raises EOF
        if there is no data and LWONSyntaxError if there is a syntax error."""
        self.skip_comments()
        result = self._parse_dictionary(self.scanner.location())
        self._reject_unmatched('}')
        return result

    def parse_implicit_array(self) -> list:
        """Parse an implicit array, which is not enclosed in brackets. Raises EOF
        if there is no data and LWONSyntaxError if there is a syntax error."""
        self.skip_comments()
        result = self._parse_array(self.scanner.location())
        self._reject_unmatched(']')
        return result

    def next_location(self) -> Optional[Location]:
        """Skip whitespace and comments, and return the location of the next
        character, or None at the end of the input."""
        self.skip_whitespace()
        return self.scanner.location() if self.scanner.has_next() else None

    def _parse(self, delimiters: str):
        s = self.scanner
        self.skip_whitespace()
        c = s.peek()
        if c == '':
            raise EOF()
        first = s.location()
        if c == '{':
            first = self._expect('{')
            result = self._parse_dictionary(first)
            self._expect_closing('}', first)
            return result
        if c == '[':
            first = self._expect('[')
            result = self._parse_array(first)
            self._expect_closing(']', first)
            return result
        if c == '"':
            first = self._expect('"')
            result = self._parse_text(True, delimiters, first)
            self._expect('"')
            return result
        if c in RESERVED_CHARS:
            raise LWONSyntaxError('Unexpected reserved character ' + _unparse_char(c), first)
        return self._parse_text(False, delimiters, first)

    def _expect(self, expected: str) -> Location:
        here = self.scanner.location()
        if self.scanner.next() != expected:
            raise LWONSyntaxError('Expected ' + _unparse_char(expected), here)
        return here

    def _expect_closing(self, closing: str, opening: Location) -> None:
        try:
            self._expect(closing)
        except EOF:
            raise LWONSyntaxError('Missing closing ' + _unparse_char(closing), opening) from None

    def _reject_unmatched(self, closing: str) -> None:
        if self.scanner.peek() == closing:
            raise LWONSyntaxError('Unmatched ' + _unparse_char(closing), self.scanner.location())

    def _parse_dictionary(self, start: Location) -> Dictionary:
        s = self.scanner
        result = Dictionary()
        try:
            while True:
                self.skip_whitespace()
                if s.peek() in ('}', ''):
                    return result
                here = s.location()
                key = self._parse_key(s.location())
                if _is_newline(s.peek()):
                    raise LWONSyntaxError('keys must have a non-newline delimiter', here)
                if s.peek() == ':':
                    s.next()
                    self.skip_whitespace()
                value = self._parse('}')
                result.add(key, value)
                self.skip_whitespace()
                if s.peek() == ',':
                    s.next()
        except EOF:
            raise LWONSyntaxError(UNEXPECTED_EOF, start) from None

    def _parse_key(self, start: Location) -> str:
        if self.scanner.peek() == '"':
            first = self._expect('"')
            key = self._parse_text(True, ':', first)
            self._expect('"')
            self.skip_blanks()
            return key
        return self._parse_text(False, ':', start)

    def _parse_text(self, multiline: bool, delimiters: str, start: Location) -> str:
        s = self.scanner
        try:
            b: list[str] = []
            ws: list[str] = []
            left_column = s.column()
            if multiline:
                s.mark()
                self.skip_blanks()
                if _is_newline(s.peek()):
                    s.accept()
                    s.newline()
                    left_column = self._skip_leading_space(left_column)
                else:
                    s.abort()
            if not multiline and _is_in(s.peek(), RESERVED_CHARS):
                raise LWONSyntaxError('Unexpected reserved character ' + s.peek()
                                      + ' at start of short string', start)
            while True:
                ch = s.peek()
                if ch == '':
                    if not multiline:
                        return ''.join(b)
                    raise LWONSyntaxError(UNEXPECTED_EOF, start)
                elif ch == '\\':
                    if not multiline and ws:
                        b.extend(ws)
                        ws = []
                    b.append(self._parse_escape_sequence())
                elif ch == '"':
                    if multiline:
                        s.mark()
                        s.next()
                        if s.peek() == '"':
                            b.append(s.next())  # "" stands for "
                            s.accept()
                        else:
                            s.abort()
                            return ''.join(b)
                    elif not ws:
                        b.append(s.next())
                    else:
                        return ''.join(b)
                elif ch in '{[':
                    if multiline or not ws:
                        b.append(s.next())
                    else:
                        return ''.join(b)
                elif ch in BLANKS:
                    if multiline:
                        b.append(s.next())
                    else:
                        ws.append(s.next())
                elif _is_newline(ch):
                    if not multiline:
                        return ''.join(b)
                    s.newline()
                    self.skip_comments()
                    b.append(os.linesep)
                    left_column = self._skip_leading_space(left_column)
                else:
                    if not multiline and ch in delimiters:
                        return ''.join(b)
                    if ws:
                        b.extend(ws)
                        ws = []
                    b.append(s.next())
        except EOF:
            raise LWONSyntaxError(UNEXPECTED_EOF, start) from None

    def skip_comments(self) -> None:
        """Skip past any comment lines to the beginning of the next line that is
        not a comment. Raises EOF if the end of the input is reached.
        Requires: the scanner is at the beginning of a line."""
        s = self.scanner
        while True:
            s.mark()
            while True:
                ch = s.peek()
                if ch == '':
                    s.abort()
                    raise EOF()
                if _is_newline(ch):
                    s.abort()  # blank line
                    return
                if ch == '#':
                    s.accept()
                    while s.has_next() and s.peek() not in LINE_TERMINATORS:
                        s.next()
                    if not s.has_next():
                        raise EOF()
                    if not s.newline():
                        s.next()  # form feed
                    break
                if ch in BLANKS:
                    s.next()
                else:
                    s.abort()  # not a comment line
                    return

    def _skip_leading_space(self, left_column: int) -> int:
        s = self.scanner
        while s.peek() == ' ' and s.column() < left_column:
            s.next()
        return s.column()

    def _parse_escape_sequence(self) -> str:
        s = self.scanner
        where = s.location()
        self._expect('\\')
        first = s.next()
        if first in ESCAPES:
            return ESCAPES[first]
        if first == '\r':
            if s.peek() == '\n':
                s.next()
            return ''
        if first == 'u':
            return self._parse_unicode_escape(where)
        raise LWONSyntaxError('Illegal escape sequence', where)

    def _parse_unicode_escape(self, where: Location) -> str:
        """Parse the rest of a Unicode escape sequence that begins at where, after
        its u, and return the character it denotes. An escape sequence for a high
        surrogate must be immediately followed by one for a low surrogate, and
        together they denote one supplementary character."""
        braces = self.scanner.peek() == '{'
        c = self._parse_code_point(braces, where)
        if not braces and 0xD800 <= c <= 0xDBFF:
            low = self._parse_low_surrogate(where)
            return chr(0x10000 + ((c - 0xD800) << 10) + (low - 0xDC00))
        if not braces and 0xDC00 <= c <= 0xDFFF:
            raise LWONSyntaxError('Unpaired surrogate in character escape', where)
        if c > 0x10FFFF or 0xD800 <= c <= 0xDFFF:
            raise LWONSyntaxError('Invalid codepoint in character escape', where)
        return chr(c)

    def _parse_code_point(self, braces: bool, where: Location) -> int:
        """Parse the hex digits of a Unicode escape sequence that begins at where:
        exactly four digits, or if braces, one to six digits enclosed in braces.
        Return the number they denote."""
        s = self.scanner
        if braces:
            self._expect('{')
        max_digits = 6 if braces else 4
        c = 0
        digits = 0
        while digits < max_digits and s.has_next():
            i = HEX_DIGITS.find(s.peek().lower())
            if i == -1:
                break
            s.next()
            c = c * 16 + i
            digits += 1
        if digits == 0 or not braces and digits < 4:
            raise LWONSyntaxError('Expected hex digits in character escape', where)
        if braces:
            self._expect('}')
        return c

    def _parse_low_surrogate(self, where: Location) -> int:
        """Parse a four-digit Unicode escape sequence for a low surrogate, which
        completes the surrogate pair whose escape sequence begins at where, and
        return the surrogate."""
        s = self.scanner
        if s.peek() == '\\':
            s.next()
            if s.peek() == 'u':
                s.next()
                if s.peek() != '{':
                    c = self._parse_code_point(False, where)
                    if 0xDC00 <= c <= 0xDFFF:
                        return c
        raise LWONSyntaxError('Unpaired surrogate in character escape', where)

    def skip_blanks(self) -> None:
        """Skip past blanks, which do not include line terminators."""
        while _is_in(self.scanner.peek(), BLANKS):
            self.scanner.next()

    def skip_whitespace(self) -> None:
        """Skip past whitespace, including line terminators, and comments."""
        s = self.scanner
        try:
            while _is_in(s.peek(), WHITESPACE):
                if s.newline():
                    self.skip_comments()
                else:
                    s.next()
        except EOF:
            pass

    def _parse_array(self, where: Location) -> list:
        s = self.scanner
        entries: list[tuple[tuple[int, ...], object]] = []
        indices: list[int] = []
        here = where
        try:
            while True:
                self.skip_whitespace()
                if not s.has_next() or s.peek() == ']':
                    return _build_array(entries)
                here = s.location()
                entries.append((tuple(indices), self._parse(',]')))
                self.skip_blanks()
                ch = s.peek()
                if ch == ',':
                    if not indices:
                        indices = [1]
                    else:
                        indices[-1] += 1
                    s.next()
                    self.skip_whitespace()
                elif ch in ('', ']'):
                    return _build_array(entries)
                elif _is_newline(ch):
                    dimout = 0
                    try:
                        while True:
                            here = s.location()
                            if not s.newline():
                                break
                            self.skip_comments()
                            dimout += 1
                    except EOF:
                        pass
                    index = len(indices) - 1 - dimout
                    if index < 0:
                        indices = [0] * (len(indices) - index)
                        index = 0
                    else:
                        for j in range(index + 1, len(indices)):
                            indices[j] = 0
                    indices[index] += 1
                else:
                    raise LWONSyntaxError('Unexpected character ' + _unparse_char(ch), s.location())
        except EOF:
            raise LWONSyntaxError(UNEXPECTED_EOF, here) from None


def _build_array(entries: list) -> list:
    """The array whose elements are given by entries, a list of (indices, value)
    pairs. An index tuple shorter than the number of dimensions is padded with
    leading zeros. Positions with no entry hold ""."""
    ndim = max((len(ix) for ix, _ in entries), default=0)
    if ndim == 0:
        return [value for _, value in entries]
    dims = [0] * ndim
    for ix, _ in entries:
        offset = ndim - len(ix)
        for i, n in enumerate(ix):
            dims[i + offset] = max(dims[i + offset], n + 1)
    nested = _filled(dims)
    for ix, value in entries:
        full = (0,) * (ndim - len(ix)) + ix
        row = nested
        for i in full[:-1]:
            row = row[i]
        row[full[-1]] = value
    return nested if ndim == 1 else Grid(nested, ndim)


def _filled(dims: list[int]) -> list:
    """Nested lists of the given dimensions, filled with ""."""
    if len(dims) == 1:
        return [''] * dims[0]
    return [_filled(dims[1:]) for _ in range(dims[0])]
