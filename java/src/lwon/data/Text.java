package lwon.data;

import easyIO.BacktrackScanner.Location;
import org.apache.commons.text.StringEscapeUtils;

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
        return "\"" + StringEscapeUtils.escapeJson(data) + "\"";
    }

    /** Return the contained string value. */
    public String value() {
        return data;
    }

    @Override
    public void unparse(StringBuilder b, int indent) {
        b.append("\"");
        b.append(StringEscapeUtils.escapeJson(data));
        b.append("\"");
    }
}