package lwon.data;

import easyIO.BacktrackScanner.Location;

import java.util.*;

/** A mapping from string keys to (possibly multiple) DataObject values.
 *  The same key may appear multiple times in a dictionary, and is bound to
 *  a list of each of the values it is associated with, in the same order.
 */
public class Dictionary extends DataObject {
    private Map<String, List<DataObject>> mapping;
    private List<Entry> entries;

    /** A key-value pair */
    public record Entry(String key, DataObject value) {}

    /** A builder class for {@code Dictionary}. */
    public static class Builder {
        private final List<Entry> entries = new ArrayList<>();

        /** Create a new empty dictionary builder. */
        public Builder() {}

        /** Add a key-value entry to the stored data. */
        public void put(String key, DataObject value) {
            entries.add(new Entry(key, value));
        }

        /** Create a new dictionary from the stored data. */
        public Dictionary build(Location where) {
            return new Dictionary(entries, where);
        }
    }

    /** Create a {@code Dictionary} from the specified entries, associated
     *  with the input location {@code where}. */
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

    /** Exception meaning that a key was not found. This is a fast exception
     *  that does not support debugging. */
    public static class NotFound extends Exception {
        @Override public NotFound fillInStackTrace() { return this; }
    }

    /** A list of all values associated with this key, in the order in which
     *  they appeared in the input.
     */
    public List<DataObject> get(Object key) {
        List<DataObject> o = mapping.get(key);
        return o == null ? List.of() : o;
    }

    /** Iterator for all key-value entries. */
    public Iterator<Entry> iterator() {
        return entries.iterator();
    }

    /** The number of distinct key-value entries in the dictionary. */
    public int size() {
        return entries.size();
    }

    /** Whether the dictionary is empty. */
    public boolean isEmpty() {
        return mapping.isEmpty();
    }

    /** Whether the dictionary has any entry associated with this key. */
    public boolean containsKey(Object key) {
        return mapping.containsKey(key);
    }

    /** The set of distinct keys in the dictionary. */
    public Iterable<String> keySet() {
        return mapping.keySet();
    }

    @Override public String toString() {
        StringBuilder b = new StringBuilder();
        this.unparse(b, 0);
        return b.toString();
    }

    @Override
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