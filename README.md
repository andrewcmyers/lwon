# lwon
LightWeight Object Notation

This is a notation for representing structured data that is intended to be even lighter-weight than JSON.

- Self-descriptive without an external schema.
- Supports strings, dictionaries, and multidimensional arrays
- Mostly backward compatible with CSV
- Mostly looks like JSON, but with less syntactic overhead:
    - key names do not need quoting
    - single-line string values do not need quoting
    - multi-line string values allowed without special escaping
