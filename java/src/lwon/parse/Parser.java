package lwon.parse;

import easyIO.EOF;
import easyIO.Scanner;
import lwon.data.*;

/**
 *  A Parser for data in LWON format
 */
public class Parser {
    private Scanner scanner;

    public class SyntaxError extends Exception {
        SyntaxError(String msg) {
            super(msg);
        }
    }

    private static final String UNEXPECTED_EOF = "Unexpected end of input";

    public DataObject parse() throws SyntaxError {
        try {
            skipWS();
            var c = scanner.next();
            switch (c) {
                case '{': return parseDictionary();
                case '[': return parseArray();
                case '"': return parseText();
                default: {
                    if (reservedCharacter(c)) {
                        throw new SyntaxError("Unexpected reserved character " + unparseChar(c));
                    }
                }
            }
        } catch (EOF e) {
            throw new SyntaxError(UNEXPECTED_EOF);
        }
    }

    private String unparseChar(char c) {
    }

    private boolean reservedCharacter(char c) {
        return false;
    }

    private void skipWS() throws easyIO.EOF {
        while (Character.isWhitespace(scanner.peek())) scanner.next();
    }

    public Array parseArray() {

    }

    public Dictionary parseDictionary() {

    }

    public Text parseText() {

    }


}
