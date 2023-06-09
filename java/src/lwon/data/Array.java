package lwon.data;

import easyIO.BacktrackScanner.Location;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Iterator;

/** An N-dimensional array of DataObjects */
public class Array extends DataObject implements Iterable<DataObject> {
    /** The sizes of the dimensions. May be empty to signify a scalar.
     *  Dimensions are in column-major order, so the last dimension is
     *  the one corresponding to a sequential scan through the data array.
     */
    private final int[] dimensions;

    /**
     * Invariant: data.length is at least as large as the product of all elements of
     * dimensions, or 1 if dimensions.length == 0
     */
    private final DataObject[] data;

    private Array(int[] dimensions, DataObject[] data, Location where) {
        super(where);
        this.dimensions = dimensions;
        this.data = data;
    }

    /** The index within the data array where a given data item is stored. */
    private static int dataIndex(int[] indices, int[] dimensions) {
        assert indices.length <= dimensions.length;
        int offset = dimensions.length - indices.length;
        int i = 0;
        for (int j = 0; j < indices.length; j++) {
            i *= dimensions[j + offset];
            i += indices[j];
        }
        return i;
    }

    /** The dimensions of the array. */
    public int[] dimensions() {
        return dimensions.clone();
    }

    /** Get the data item at the specified indices. */
    public DataObject get(int[] indices) {
        assert indices.length == dimensions.length;
        return data[dataIndex(indices, dimensions)];
    }

    /** Produce a new Array in which all dimensions of size 1 are removed. This can be used to
     * convert a single-column array into a single-row array. */
    public Array squeeze() {
        ArrayList<Integer> newDims = new ArrayList<>();
        for (int i : dimensions) {
            if (i > 1) newDims.add(i);
        }
        int[] newDimsArr = new int[newDims.size()];
        for (int i = 0; i < newDims.size(); i++)
            newDimsArr[i] = newDims.get(i);
        if (data.length == 0) newDimsArr = new int[0];
        return new Array(newDimsArr, data.clone(), location());
    }

    @Override
    public void unparse(StringBuilder b, int indent) {
        int[] indices = new int[dimensions.length];
        b.append("[");
        b.append(System.lineSeparator());
        if (indices.length == 1 && dimensions[0] == 0) {
            b.append("  ".repeat(indent));
            b.append("]");
            return;
        }
        boolean first = true;
        for (;;) {
            if (first) {
                b.append("  ".repeat(indent + 1));
            } else {
                b.append(" ");
            }
            first = false;
            DataObject obj = get(indices);
            obj.unparse(b, indent + 1);
            int i = indices.length - 1;
            int newlines = 0;
            for (; i >= 0; i--) {
                int n = dimensions[i];
                indices[i]++;
                if (indices[i] < n) {
                    if (i == indices.length - 1) {
                        b.append(",");
                    }
                    break;
                }
                newlines++;
                first = true;
                for (int j = i; j < indices.length; j++) indices[j] = 0;
            }
            if (i < 0) break;
            if (newlines > 0) b.append(System.lineSeparator().repeat(newlines));
        }
        b.append(System.lineSeparator());
        b.append("  ".repeat(indent));
        b.append("]");
    }

    public Iterator<DataObject> iterator() {
        return Arrays.stream(data).iterator();
    }

    /** A builder class for {@code Array}.
     */
    public static class Builder {
        private record Entry(int[] indices, DataObject value) {}
        private final ArrayList<Entry> entries = new ArrayList<>();

        /** Create a new builder containing no elements. */
        public Builder() {}

        /** Create an array object containing the current data in the builder. */
        public Array build(Location where) {
            int numDims = 0;
            for (Entry e : entries) {
                numDims = Math.max(numDims, e.indices.length);
            }
            int[] dimensions = new int[numDims];
            for (Entry e : entries) {
                int d = e.indices.length;
                int offset = numDims - d;
                for (int i = 0; i < d; i++) {
                    dimensions[i + offset] = Math.max(dimensions[i + offset], e.indices[i] + 1);
                }
            }
            if (numDims == 0) {
                numDims = 1;
                dimensions = new int[]{entries.size()};
            }
            DataObject[] data = new DataObject[size(dimensions)];
            DataObject dummy = new Text("", where);
            for (Entry e : entries) {
                data[dataIndex(e.indices, dimensions)] = e.value;
            }
            for (int j = 0; j < data.length; j++) {
                if (data[j] == null) data[j] = dummy;
            }
            return new Array(dimensions, data, where);
        }
        /** Set the object located at the specified indices to {@code obj}.
         *  If the length of {@code indices} given is less than the
         *  dimensionality of the array, enough zeros are prepended to the index
         *  array to equalize the dimensionality.
         */
        public void set(int[] indices, DataObject obj) {
            entries.add(new Entry(indices.clone(), obj));
        }

        /** The number of entries in the full array.
         */
        private int size(int[] dimensions) {
            int result = 1;
            for (int s : dimensions) {
                result *= s;
            }
            return result;
        }
    }
}
