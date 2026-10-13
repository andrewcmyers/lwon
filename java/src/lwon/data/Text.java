package lwon.data;

import easyIO.BacktrackScanner.Location;

/** A text data object parsed by LWON. It may represent either a short
 *  strong or a long (multiline) string.
 **/
public class Text extends DataObject {
    private String data;

    /** Create a object containing the specified string data, at the specified location. */
    public Text(String s, Location where) {
        super(where);
        data = s;
    }

    @Override
    public String toString() {
        return "\"" + escape(data) + "\"";
    }

    /** Return the contained string value. */
    public String value() {
        return data;
    }

    @Override
    public void unparse(StringBuilder b, int indent) {
        b.append("\"");
        b.append(escape(data));
        b.append("\"");
    }

    /** The text of {@code s} as written between the quotation marks of an LWON
     *  string. The characters {@code "}, <code>&#92;</code>, {@code /}, and
     *  control characters are written as escape sequences, as are characters
     *  outside ASCII: <code>&#92;uXXXX</code> for those in the Basic Multilingual
     *  Plane and <code>&#92;u{XXXXX}</code> for supplementary characters. An
     *  unpaired surrogate in {@code s} is written as <code>&#92;uXXXX</code>,
     *  which is not valid LWON.
     */
    public static String escape(String s) {
        StringBuilder b = new StringBuilder();
        s.codePoints().forEach(c -> {
            switch (c) {
                case '"' -> b.append("\\\"");
                case '\\' -> b.append("\\\\");
                case '/' -> b.append("\\/");
                case '\b' -> b.append("\\b");
                case '\n' -> b.append("\\n");
                case '\t' -> b.append("\\t");
                case '\f' -> b.append("\\f");
                case '\r' -> b.append("\\r");
                default -> {
                    if (c >= 0x20 && c <= 0x7f) {
                        b.appendCodePoint(c);
                    } else if (c <= 0xFFFF) {
                        b.append(String.format("\\u%04X", c));
                    } else {
                        b.append(String.format("\\u{%X}", c));
                    }
                }
            }
        });
        return b.toString();
    }
}
