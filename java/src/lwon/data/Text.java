package lwon.data;

import easyIO.BacktrackScanner.Location;
import org.apache.commons.text.StringEscapeUtils;

public class Text extends DataObject {
    private String data;

    public Text(String s, Location where) {
        super(where);
        data = s;
    }

    @Override
    public String toString() {
        return "\"" + StringEscapeUtils.escapeJson(data) + "\"";
    }

    @Override
    public void unparse(StringBuilder b, int indent) {
        b.append("\"");
        b.append(StringEscapeUtils.escapeJson(data));
        b.append("\"");
    }
}