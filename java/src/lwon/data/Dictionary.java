package lwon.data;

import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;

/** A mapping from string keys to (possibly multiple) DataObject values.
 *  The same key may appear multiple times in a dictionary, and is bound to
 *  a list of each of the values it is associated with, in the same order.
 */
public class Dictionary implements DataObject {
    Map<String, List<DataObject>> mapping;

    public static class Builder {
        private Map<String, List<DataObject>> mapping = new HashMap<>();
        public Builder() {}

        void put(String key, DataObject value) {
            if (!mapping.containsKey(key)) {
                LinkedList<DataObject> values = new LinkedList<>();
                values.add(value);
                mapping.put(key, values);
            } else {
                mapping.get(key).add(value);
            }
        }
        public Dictionary build() {
            Map<String, List<DataObject>> m = mapping;
            mapping = new HashMap<>();
            return new Dictionary(m);
        }
    }
    private Dictionary(Map<String, List<DataObject>> m) {
        mapping = m;
    }
    class NotFound extends Exception {
        @Override public NotFound fillInStackTrace() { return this; }
    }
    List<DataObject> get(String key) throws NotFound {
        List<DataObject> o = mapping.get(key);
        if (o == null) throw new NotFound();
        return o;
    }
}