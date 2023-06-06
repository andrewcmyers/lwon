package lwon.data;

import easyIO.BacktrackScanner.Location;

/** A data object produced by parsing LWON input.
 */
abstract public class DataObject {
    Location where;

    /** Create a data object. */
    DataObject(Location where) {
        this.where = where;
    }

    /** The input location of this object. */
    public Location location() {
        return where;
    }

    /** Output this data object to the specified StringBuilder in LWON
     *  format, with reasonably nice formatting and the given indent level.
     */
    public abstract void unparse(StringBuilder b, int indent);
}
