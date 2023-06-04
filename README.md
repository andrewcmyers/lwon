# LWON
LightWeight Object Notation

This is a notation for representing structured data that is intended to be even lighter-weight than JSON.

- Self-descriptive without an external schema.
- Supports strings, dictionaries, and multidimensional arrays
- Mostly backward compatible with CSV and JSON
- Mostly looks like JSON, but with less syntactic overhead:
    - key names do not need quoting
    - single-line string values do not need quoting
    - multi-line string values allowed without special escaping

Parsing LWON is harder than parsing JSON, but the intention is to make the data easier to read and
maintain by hand.

## Syntax

* `{` introduces a dictionary. Elements are key: value pairs, where keys
  are string values. Keys may be repeated.

* `[` introduces an array. Elements are one per line but delimited by commas. Elements may be quoted to escape
  commas, with same rules as string values.

* `"` introduces an (explicit) string value. It extends until the closing unescaped ", and may contain multiple lines.
    Leading whitespace on lines is ignored up to the column of the first non-whitespace character.
    Closing whitespace on the first line is ignored, if there is no non-whitespace text following the ".
    Escape sequences are supported, including "\\ " to denote a literal
    space character. A \\ at the end of the line means to ignore the newline.
    A newline is interpreted as the standard newline sequence on the current machine.

* Other non-reserved non-whitespace characters introduce implicit string values that extend until the
  next natural delimiter. Trailing whitespace up to the delimiter is removed. Delimiters other than
  newline and colon *must* be preceded by some whitespace.

### Natural delimiters

Natural delimiters depend on context.
- For dictionary keys, it is a colon (`:`), `\[`, `{`, or `"`. It is an error
  to have a newline delimiter. 
- For dictionary values, it is a newline or closing brace.
- For array elements, it is a comma, closing bracket (`\]`), or newline.

### Arrays

Arrays are automatically multidimensional, with rows as the major
dimension if there are rows. Thus, a CSV file can be interpreted
largely as is. Double, triple, etc. newlines can be used to introduce
even higher dimensions.

### Reserved characters

Some reserved characters cannot begin an implicit string value: |, $, +, \

### Implicit outer objects:

The top-level object need not be explicitly delimited; it may be
implicitly understood to be a dictionary or array, which case the
initial { or \[ is not used. CSV files can therefore be read as
implicit top-level arrays.

## Examples

Personal information as a dictionary:
```
{
  hair: brown
  eyes: blue
  height: 69
  friends: [ alice, bob ]
  credo: "Do unto others as you
          would have them do unto you."
  pets [ { name: fido, species: dog },
         { name: tom, species: cat } ]
}
```

A 2-D array (implicit outer object):
```
Country, Population, GDP
USA, 338, 23.3
China, 1411, 12.2
Germany, 84, 3.7
```
