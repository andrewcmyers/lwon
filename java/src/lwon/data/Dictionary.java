package lwon.data;

import easyIO.BacktrackScanner.Location;

import java.util.*;

/** A mapping from string keys to (possibly multiple) DataObject values.
 *  The same key may appear multiple times in a dictionary, and is bound to
 *  a list of each of the values it is associated with, in the same order.
 */
public class Dictionary extends DataObject {
    Map<String, List<DataObject>> mapping;
    List<Entry> entries;

    public record Entry(String key, DataObject value) {}

    public static class Builder {
        private final List<Entry> entries = new ArrayList<>();
        public Builder() {}

        public void put(String key, DataObject value) {
            entries.add(new Entry(key, value));
        }
        public Dictionary build(Location where) {
            return new Dictionary(entries, where);
        }
    }
    private Dictionary(List<Entry> entries, Location where) {
        super(where);
        this.entries = entries;
        mapping = new HashMap<>();
        for (Entry e : entries) {
            if (!mapping.containsKey(e.key)) {
                mapping.put(e.key, new ArrayList<>());
            }
            mapping.get(e.key).add(e.value);
        }
    }
    static class NotFound extends Exception {
        @Override public NotFound fillInStackTrace() { return this; }
    }
    public List<DataObject> get(Object key) throws NotFound {
        List<DataObject> o = mapping.get(key);
        if (o == null) throw new NotFound();
        return o;
    }

    public Iterator<Entry> iterator() {
        return entries.iterator();
    }

    public int size() {
        return mapping.size();
    }

    public boolean isEmpty() {
        return mapping.isEmpty();
    }

    public boolean containsKey(Object key) {
        return mapping.containsKey(key);
    }

    public Iterable<String> keySet() {
        return mapping.keySet();
    }

    @Override public String toString() {
        StringBuilder b = new StringBuilder();
        this.unparse(b, 0);
        return b.toString();
    }

    public void unparse(StringBuilder b, int indent) {
        b.append("{");
        boolean first = true;
        for (Entry e : entries) {
            if (!first) {
                b.append(",");
            }
            first = false;
            b.append(System.lineSeparator());
            b.append("  ".repeat(indent + 1));
            b.append(e.key);
            b.append(": ");
            e.value.unparse(b, indent + 1);
        }
        b.append(System.lineSeparator());
        b.append("  ".repeat(indent));
        b.append("}");
    }
}