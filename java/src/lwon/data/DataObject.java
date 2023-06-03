package lwon.data;

import easyIO.BacktrackScanner.Location;

abstract public class DataObject {
    Location where;

    DataObject(Location where) {
        this.where = where;
    }

    public Location location() {
        return where;
    }

    public abstract void unparse(StringBuilder b, int indent);
}