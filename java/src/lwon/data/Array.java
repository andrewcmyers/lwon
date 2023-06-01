package lwon.data;

/** An N-dimensional array of DataObjects */
public class Array implements DataObject {
    /** The sizes of the dimensions */
    private int[] dimensions;

    /** Invariant: data.length is at least as large as the product of all elements of
     *  dimensions, or 1 if dimensions.length == 0
     */
    private DataObject[] data;

    public DataObject get(int[] indices) {
        int i = 0;
        for (int j = 0; j < indices.length; j++) {
            i *= dimensions[j];
            i += indices[j];
        }
        return data[i];
    }

    public class Builder {
        Array build() {
            
        }
    }
}