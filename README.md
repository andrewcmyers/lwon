# LWON

LightWeight Object Notation

Applications commonly read configuration files. These configuration files are
often structurally complex, leading to use of expressive standards like XML and
JSON.  Unfortunately the syntax of these standards is cumbersome, making them
inconvenient to work with.

LWON is a lighter-weight notation for representing structured data.  Parsing
LWON is somewhat more complex than parsing JSON or XML, but the data is easier
to read and maintain by hand, without special tools. It is particularly handy
as a syntax for configuration files, but it can be used for other purposes.

*Goals:*

- Self-descriptive without an external schema.

- Supports strings, dictionaries, and multidimensional arrays

- Mostly backward compatible with CSV and JSON

- Mostly looks like JSON, but with less syntactic overhead:

    - key names do not need quoting.
    - single-line string values ("short strings") do not need quoting.
    - multi-line string values ("long strings") are allowed without special escaping.
    - multidimensional arrays are supported directly.

## Examples

Personal information as a dictionary:
```
{
  hair: brown
  eyes: blue
  height: 69
  friends: [ alice, bob ]
  state: New Mexico
  local address: 742 Evergreen Terrace, Springfield
  # Note: No leading whitespace in the second line of the credo.
  credo: "Do unto others as you
          would have them do unto you."
  # Note: values can be complex data with nested arrays and dictionaries.
  pets [ { name: fido, species: dog },
         { name: tom, species: cat } ]
}
```

A 2-D array (implicit outer object):
```
# Header row follows (not special in LWON)
Country, Population, GDP
# Now, some data
USA, 338, 23.3
China, 1411, 12.2
Germany, 84, 3.7
```

See the directory `tests/` for more short examples.

## Syntax

Although all the mandated syntax sticks to ASCII, LWON input is a sequence of
Unicode characters, normally using UTF-8 encoding.

* `{` introduces a dictionary, which is a sequence of key-value pairs,
  where keys are string values. Keys may be repeated. Keys may be
  separated from values by a colon (`:`). Key/value pairs may optionally
  be separated by commas, but note that commas are not a natural
  delimiter for short strings.

* `[` introduces an array. Elements are delimited by commas. Elements
  may be quoted to escape commas, with same rules as string values. Arrays
  are multidimensional, as described below.

* `"` introduces an (explicit, long) string value. The string extends
  until the closing unescaped `"`, and may contain multiple lines.

* Other non-reserved non-whitespace characters introduce implicit "short
  strings" that extend until the next natural delimiter. Trailing whitespace up
  to the delimiter is removed, as is leading whitespace.  Unquoted numbers and
  booleans are treated as short strings.

### Comments

A comment line is one whose first non-whitespace character is a number sign
(`#`). Any such line is ignored completely. There are no inline comments: a
line with a comment cannot include any object data. The character # is
otherwise treated as an ordinary character. It may also be escaped in strings
(`\#`).

### Strings

Short strings are not surrounded by quotation marks, and they must be on a
single line.  Some reserved characters may not begin a short string. In
addition to the above special characters, reserved characters include `|`, `$`,
`+`, `\`. However, these characters may be used inside a short string.
Whitespace other than newlines may also occur inside a short string.

Long strings begin with a double quotation mark (`"`). They may span multiple
lines. Leading whitespace in long strings is handled in a better way than in
most formats.  The leading whitespace on each line is ignored up to the column
of the first non-whitespace character on any previous line, so long strings can
remain indented in a visually attractive way.

For long strings, closing whitespace on the first line is ignored, if there is no
non-whitespace text following the `"`. Standard JSON escape sequences
are supported. In addition, escape sequences like `\]` and `\ ` may
be used to indicate that the next character, otherwise special to
LWON, is to be taken literally.  A backslash `\` at the end of the
line means to ignore the newline.  A newline is interpreted as the
standard newline sequence on the current machine.

### Dictionaries

Dictionaries are internally an ordered list of key-value pairs, with the
ordering as specified in the input. Keys may be any value, not just identifiers.

Querying a dictionary key with a given key returns a list of associated values,
in the input order. Repeated keys do not have to occur sequentially in the
input.

Empty values are allowed but must be specified as long strings: `""`.

### Arrays

Arrays are automatically multidimensional, with rows as the major
dimension if there are rows. Thus, a CSV file can be interpreted
largely as is. Double, triple, etc. newlines can be used to introduce
even higher dimensions. Arrays have uniform dimensions: they are not
ragged. To represent such structures, empty text elements are created
as missing positions.

### Natural delimiters

Natural delimiters depend on context.
- For dictionary keys, it is a colon (`:`), `[`, `{`, or `"`. It is an error
  to have a newline delimiter. 
- For dictionary values, it is a newline or closing brace.
- For array elements, it is a comma, closing bracket (`]`), or newline.

### Implicit outer objects:

The top-level object need not be explicitly delimited; it may be
implicitly understood to be a dictionary or array, which case the
initial `{` or `[` is not used. CSV files can therefore be read as
implicit top-level arrays. To have an implicit outer object, the parser
must be told externally what to expect.

## Implementations

Currently the only LWON implementation is in Java. With luck, implementations
for more language will be provided by other authors.

### Building the code

The Java implementation can be built as a runnable JAR file using the command
`gradle shadowJar`. The script `bin/dump` is an example using this API. It
reads in a file as a sequence of LWON objects and prints each of them to
standard output.

Javadoc documentation for the Java implementation can be found in the `docs/`
directory or [at the web site](https://andrewcmyers.github.io/lwon/).


## Contributing to LWON

There are many things that could be done to improve this project, and help from
outside collaborators is welcome.

- More thorough testing would be good.
   * especially, negative test cases should be added
   * Also, testing should be set up as a Github action.

- Currently the focus is on reading information from external user input. Outputting
  data in human-readable format is also useful if one wants to use LWON as a data store
  for an application. The current output format is verbose and does not preserve
  comments and stylistic choices from the original input. What is wanted is a
  lens "put" operation that converts data objects back to LWON while using the
  original input to provide the missing information.

- LWON data objects actually have *two* other layers to synchronize: not just the
  human-readable data it is parsed from and unparsed to, but also, for many applications,
  application data representations that are more convenient for expressing application
  business logic. Support for all of the arrows in the following diagram would be very
  powerful!

```
diagram:                             app-specific "get"
                    parser               translation
  human-readable  ------→  LWON data object -----→ application data rep
  UTF-8 data  \                  |         \____           |
        |      \___              |              \          | app
external|          \             | LWON update←-Є----------Є-business
editing |           ↓            | operations   |          | logic
        ↓         unparse        ↓              |          ↓
  updated human   ←------- updated LWON data ←--↓-- updated app data
  readable data                          app-specific "put"
                                            translation

```
