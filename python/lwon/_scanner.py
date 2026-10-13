"""A character scanner with backtracking and line/column tracking."""

from __future__ import annotations


class EOF(Exception):
    """The end of the input was reached."""


class Location:
    """The position of a character in the input: its line number (starting
    at 1) and its column (starting at 1, with a line terminator at column 0
    of the line it begins)."""

    def __init__(self, name: str, line: int, column: int, char: str):
        self.name = name
        self.line = line
        self.column = column
        self.char = char

    def __str__(self) -> str:
        return f'"{self.name}", line {self.line}, char {self.column} ({_display(self.char)})'


_DISPLAY = {'\n': '\\n', '\t': '\\t', '\r': '\\r', '\b': '\\b'}


def _display(c: str) -> str:
    if c in _DISPLAY:
        return _DISPLAY[c]
    if ord(c) < 0x20:
        return '\\x{%02x}' % ord(c)
    return c


class Scanner:
    """Scans the characters of a string. Line terminators are "\\n", "\\r\\n",
    and "\\r"; "\\r\\n" is a single terminator.

    The scanner keeps a stack of marks: mark() pushes the current position,
    accept() pops the top mark, and abort() pops it and returns to it.
    """

    def __init__(self, text: str, name: str):
        self.text = text
        self.name = name
        self.pos = 0
        self.marks: list[int] = []
        self.lines: list[int] = []
        self.columns: list[int] = []
        line, column, after_cr = 1, 0, False
        for c in text:
            if after_cr and c == '\n':
                pass
            elif c == '\n' or c == '\r':
                line, column = line + 1, 0
            else:
                column += 1
            after_cr = c == '\r'
            self.lines.append(line)
            self.columns.append(column)

    def peek(self) -> str:
        """The next character, or "" at the end of the input."""
        return self.text[self.pos] if self.pos < len(self.text) else ''

    def has_next(self) -> bool:
        """Whether there is a next character."""
        return self.pos < len(self.text)

    def next(self) -> str:
        """Scan past and return the next character.
        Raises EOF if there is none."""
        if self.pos >= len(self.text):
            raise EOF()
        self.pos += 1
        return self.text[self.pos - 1]

    def location(self) -> Location:
        """The location of the next character. Raises EOF if there is none."""
        if self.pos >= len(self.text):
            raise EOF()
        return Location(self.name, self.lines[self.pos], self.columns[self.pos], self.text[self.pos])

    def column(self) -> int:
        """The column of the next character, or at the end of the input, the
        column of the last character (0 if the input is empty)."""
        if self.pos < len(self.text):
            return self.columns[self.pos]
        return self.columns[-1] if self.columns else 0

    def newline(self) -> bool:
        """If the input is at a line terminator, scan past it and return True.
        Otherwise, return False."""
        c = self.peek()
        if c == '\n':
            self.pos += 1
            return True
        if c == '\r':
            self.pos += 1
            if self.peek() == '\n':
                self.pos += 1
            return True
        return False

    def mark(self) -> None:
        """Push the current position onto the stack of marks."""
        self.marks.append(self.pos)

    def accept(self) -> None:
        """Pop the most recent mark, keeping the current position."""
        self.marks.pop()

    def abort(self) -> None:
        """Pop the most recent mark and return to its position."""
        self.pos = self.marks.pop()
