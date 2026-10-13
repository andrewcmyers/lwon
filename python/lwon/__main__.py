"""Usage: python -m lwon [-a] [-d] [file ...]

Read each file (or standard input) as a sequence of LWON values and print each
value in LWON format. With -a, the values are implicit arrays; with -d,
implicit dictionaries. Exits with status 1 after printing a message to
standard error if the input is not valid LWON or a file cannot be read."""

import sys
from typing import Optional

from . import LWONSyntaxError, dumps, loads_all


def main(args: Optional[list] = None) -> int:
    """Run the program with the command-line arguments args (by default,
    sys.argv[1:]), and return its exit status."""
    if args is None:
        args = sys.argv[1:]
    implicit = None
    optind = 0
    while optind < len(args) and args[optind].startswith('-'):
        if args[optind] == '-a':
            implicit = 'array'
        elif args[optind] == '-d':
            implicit = 'dict'
        optind += 1
    files = args[optind:] or ['-']
    try:
        for filename in files:
            if filename == '-':
                text, name = sys.stdin.read(), 'stdin'
            else:
                with open(filename, encoding='utf-8', newline='') as f:
                    text, name = f.read(), filename
            for value in loads_all(text, implicit, name):
                print(dumps(value))
    except LWONSyntaxError as e:
        sys.stdout.flush()
        print(e, file=sys.stderr)
        return 1
    except OSError as e:
        print(f'File not found: {e.filename}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
