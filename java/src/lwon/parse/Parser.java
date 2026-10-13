package lwon.parse;

import easyIO.EOF;
import easyIO.LineTerminators;
import easyIO.Scanner;
import easyIO.BacktrackScanner.Location;
import easyIO.UnexpectedInput;
import lwon.data.*;
import org.apache.commons.text.StringEscapeUtils;

/**
 *  A Parser for data in LWON format
 */
public class Parser {
    private Scanner scanner;

    /** Create a Parser reading from the specified input, which is set to accept
     *  all LWON line terminators.
     *  Requires: no input has been read from input.
     */
    public Parser(Scanner input) {
        scanner = input;
        scanner.setLineTerminators(LineTerminators.LF_CRLF_CR);
    }

    /** Exception meaning that the input contained a violation of LWON syntax rules.
     */
    public static class SyntaxError extends Exception {
        SyntaxError(String msg, Location location) {
            super(location + ": " + msg);
        }
    }

    private static final String UNEXPECTED_EOF = "Unexpected end of input";

    /** Parse an object from the input. All prior positions on the current line,
     *  if any, are treated as if they were whitespace.
     */
    public DataObject parse() throws SyntaxError, EOF {
        skipComments();
        return parse("");
    }

    /** Parse an object from the input. The characters in delimiters should be
     *  treated as delimiters terminating this object.
     *  Required: the current line is not a comment.
     */
    private DataObject parse(String delimiters) throws EOF, SyntaxError {
            DataObject result;
            skipWhitespace();
            Location first = scanner.location();
            int c = scanner.peek();
            switch (c) {
                case -1: throw new EOF();
                case '{':
                    first = expect('{');
                    result = parseDictionary(first);
                    expectClosing('}', first);
                    return result;
                case '[':
                    first = expect('[');
                    result = parseArray(first);
                    expectClosing(']', first);
                    return result;
                case '"':
                    first = expect('"');
                    result = parseText(true, delimiters, first);
                    expect('"');
                    return result;
                default: {
                    char ch = (char) c;
                    if (reservedCharacter(ch)) {
                        throw new SyntaxError("Unexpected reserved character " + unparseChar(ch), first);
                    }
                    result = parseText(false, delimiters, first);
                    return result;
                }
            }
    }

    /** Read the expected character from the scanner or throw SyntaxError
     *  (or EOF, as appropriate) if something else is found.
     */
    private Location expect(char expected) throws EOF, SyntaxError {
        Location here = scanner.location();
        char c = scanner.next();
        if (c != expected) {
            throw new SyntaxError("Expected " + unparseChar(expected), here);
        }
        return here;
    }

    /** Read the expected closing character, reporting end of input as a SyntaxError
     *  at the location of the matching opening character.
     */
    private void expectClosing(char closing, Location opening) throws SyntaxError {
        try {
            expect(closing);
        } catch (EOF exc) {
            throw new SyntaxError("Missing closing " + unparseChar(closing), opening);
        }
    }

    /** Throw SyntaxError if the next character is the closing character
     *  {@code closing}, which has no matching opening character.
     */
    private void rejectUnmatched(char closing) throws SyntaxError, EOF {
        if (scanner.peek() == closing) {
            throw new SyntaxError("Unmatched " + unparseChar(closing), scanner.location());
        }
    }

    private String unparseChar(int c) {
        return StringEscapeUtils.escapeJava(Character.toString(c));
    }

    private boolean reservedCharacter(char c) {
        return (-1 != reservedChars.indexOf(c));
    }

    /** Parse an implicit dictionary data object from the scanner. Throw EOF if there is no
     *  data there and SyntaxError if there is a syntax error. */
    public DataObject parseDictionary() throws SyntaxError, EOF {
        skipComments();
        Dictionary result = parseDictionary(scanner.location());
        rejectUnmatched('}');
        return result;
    }

    /** Parse a dictionary from the scanner. The scanner position is required to be after
     *  the opening brace and on either a real character of dictionary content (i.e., not
     *  whitespace or a comment), or the closing brace.
     */
    public Dictionary parseDictionary(Location start) throws SyntaxError {
        try {
            Location here = start;
            Dictionary.Builder b = new Dictionary.Builder();
            for (;;) {
                skipWhitespace();
                if (scanner.peek() == '}' || scanner.peek() == -1) break;
                here = scanner.location();
                String key = parseKey(scanner.location());
                if (isNewline(scanner.peek())) {
                    throw new SyntaxError("keys must have a non-newline delimiter",  here);
                }
                if (scanner.peek() == ':')  {
                    scanner.next();
                    skipWhitespace();
                }
                DataObject value = parse("}");
                b.put(key, value);
                skipWhitespace();
                if (scanner.peek() == ',') scanner.next();
            }
            return b.build(start);
        } catch (EOF exc) {
            throw new SyntaxError(UNEXPECTED_EOF, start);
        }
    }

    private String parseKey(Location start) throws SyntaxError, EOF {
        if (scanner.peek() == '"') {
            Location first = expect('"');
            Text t = parseText(true, ":", first);
            expect('"');
            skipBlanks();
            return t.value();
        }
        Text t = parseText(false, ":", start);
        return t.value();
    }

    private static final String reservedChars = "{}[]:";

    /** parse text that is either single-line or multi-line text. In the former
     *  case, a newline is a delimiter.
     */
    public Text parseText(boolean multiline, String delimiters, Location start) throws SyntaxError {
      try {
          StringBuilder b = new StringBuilder();
          StringBuilder ws = new StringBuilder();
          int leftColumn = scanner.column();
          if (multiline) {
              scanner.mark();
              skipBlanks();
              if (isNewline(scanner.peek())) {
                  scanner.accept();
                  newline();
                  leftColumn = skipLeadingSpace(leftColumn);
              } else {
                  scanner.abort();
              }
          }
          if (!multiline && -1 != reservedChars.indexOf(scanner.peek())) {
              throw new SyntaxError("Unexpected reserved character " +
                  Character.toString(scanner.peek()) + " at start of short string", start);
          }
          for (;;) {
              int ch = scanner.peek();
              switch (ch) {
                  case -1:
                      // end of input is a natural delimiter for short strings
                      if (!multiline) return new Text(b.toString(), start);
                      throw new SyntaxError(UNEXPECTED_EOF, start);
                  case '\\':
                      if (!multiline && !ws.isEmpty()) {
                          b.append(ws);
                          ws = new StringBuilder();
                      }
                      try {
                          scanner.newline();
                          skipComments();
                          leftColumn = skipLeadingSpace(leftColumn);
                      } catch (UnexpectedInput exc) {
                          b.append(parseEscapeSequence());
                      }
                      break;
                  case '"':
                      if (multiline) {
                          b.append(ws);
                          scanner.mark();
                          scanner.next();
                          if (scanner.peek() == '"') {
                              b.append(scanner.next()); // handle "" notation
                          } else {
                              scanner.abort();
                              return new Text(b.toString(), start);
                          }
                      } else {
                          if (ws.isEmpty()) {
                              b.append(scanner.next());
                          } else {
                              return new Text(b.toString(), start);
                          }
                      }
                      break;
                  case '{':
                  case '[':
                      if (multiline) {
                          b.append(ws);
                          b.append(scanner.next());
                      } else {
                          if (ws.isEmpty()) {
                              b.append(scanner.next());
                          } else {
                              return new Text(b.toString(), start);
                          }
                      }
                      break;
                  case ' ':
                      if (multiline) {
                          b.append(Character.toChars(scanner.next()));
                      } else {
                          ws.append(scanner.next());
                      }
                      break;
                  case '\r':
                  case '\n':
                      if (!multiline) return new Text(b.toString(), start);
                      newline();
                      skipComments();
                      b.append(System.lineSeparator());
                      leftColumn = skipLeadingSpace(leftColumn);
                      break;
                  default:
                      if (!multiline && -1 != delimiters.indexOf(ch)) {
                          return new Text(b.toString(), start);
                      }
                      if (!ws.isEmpty()) {
                          b.append(ws);
                          ws = new StringBuilder();
                      }
                      b.append(Character.toChars(scanner.next()));
                      break;
              }
          }
      } catch (EOF e) {
          throw new SyntaxError(UNEXPECTED_EOF, start);
      }
    }

    private static final String blanks = " \t";
    private static final String lineTerminators = "\n\r\f";
    private static final String whitespace = blanks + lineTerminators;

    /**
     *  Skip past any comment lines to the beginning
     *  of the next line that is not a comment.
     *  Requires: scanner is at the beginning of a line.
     */
    public void skipComments() throws EOF {
        for (;;) { // loop over lines
            scanner.mark();
            charloop: for (;;) { // loop over initial characters
                int ch = scanner.peek();
                switch (ch) {
                    case -1:
                        scanner.abort();
                        throw new EOF();
                    case '\n':
                    case '\r':
                        scanner.abort(); // blank line
                        return;
                    case '#':
                        scanner.accept();
                        while (scanner.hasNext() && -1 == lineTerminators.indexOf(scanner.peek())) {
                            scanner.next();
                        }
                        if (!scanner.hasNext()) throw new EOF();
                        if (!newline()) scanner.next(); // form feed
                        break charloop;
                    default:
                        if (-1 != blanks.indexOf(ch)) {
                            scanner.next();
                        } else {
                            scanner.abort(); // non-comment line: rewind and return
                            return;
                        }
                }
            }
        }
    }

    private int skipLeadingSpace(int leftColumn) throws EOF {
        while (scanner.peek() == ' ' && scanner.column() < leftColumn)
            scanner.next();
        return scanner.column();
    }

    private static final String hexdigits = "0123456789abcdef";
    private String parseEscapeSequence() throws SyntaxError, EOF {
        Location where = scanner.location();
        expect('\\');
        int first = scanner.next();
        switch (first) {
            case 'r': return "\r";
            case 'n': return "\n";
            case ' ': return " ";
            case 'f': return "\f";
            case 'b': return "\b";
            case 't': return "\t";
            case '\\': return "\\";
            case '/': return "/";
            case '"': return "\"";
            case '\'': return "'";
            case 'u': {
                int c = 0;
                boolean braces = false;
                if (scanner.peek() == '{') {
                    braces = true;
                    expect('{');
                }
                // \\uXXXX takes exactly 4 hex digits; \\u{X...} takes 1 to 6.
                int maxDigits = braces ? 6 : 4;
                int digits = 0;
                while (digits < maxDigits && scanner.hasNext()) {
                    int i = hexdigits.indexOf(Character.toLowerCase(scanner.peek()));
                    if (i == -1) break;
                    scanner.next();
                    c = c * 16 + i;
                    digits++;
                }
                if (digits == 0 || !braces && digits < 4) {
                    throw new SyntaxError("Expected hex digits in character escape", where);
                }
                if (braces) expect('}');
                if (Character.isValidCodePoint(c)) {
                    return Character.toString(c);
                } else {
                    throw new SyntaxError("Invalid codepoint in character escape", where);
                }
            }
            case '}': return "}";
            case ']': return "]";
            case '#': return "#";
            case ':': return ":";
            case '\r':
                if (scanner.peek() == '\n') scanner.next();
                return "";
            case '\n': return "";
            default:
                throw new SyntaxError("Illegal escape sequence", where);
        }
    }

    /** Whether ch is the first character of a line break: \n, \r\n, or \r. */
    private static boolean isNewline(int ch) {
        return ch == '\n' || ch == '\r';
    }

    /** If the input is at a line break (\n, \r\n, or \r), consume it and return true.
     *  Otherwise, return false. */
    private boolean newline() {
        try {
            scanner.newline();
            return true;
        } catch (UnexpectedInput exc) {
            return false;
        }
    }

    /** Skip past characters considered blanks (which do not include newlines) */
    public void skipBlanks() {
        try {
            while (scanner.hasNext() && -1 != blanks.indexOf(scanner.peek())) {
                scanner.next();
            }
        } catch (EOF exc) {
            // nothing left
        }
    }
    /** Skip past characters considered whitespace (including newlines) and comments. */
    public void skipWhitespace() {
        try {
            while (scanner.hasNext() && -1 != whitespace.indexOf(scanner.peek())) {
                if (newline()) {
                    skipComments();
                } else {
                    scanner.next();
                }
            }
        } catch (EOF exc) {
            // nothing left
        }
    }

    /** Parse an implicit array data object from the scanner. Throw EOF if there is no
     *  data there and SyntaxError if there is a syntax error. */
    public DataObject parseArray() throws SyntaxError, EOF {
        skipComments();
        Array result = parseArray(scanner.location());
        rejectUnmatched(']');
        return result;
    }
    /** Parse an array from the scanner. The scanner position is required to be after
     *  the opening bracket and on either a real character of array content or the
     *  closing bracket.
     */
    public Array parseArray(Location where) throws SyntaxError {
        Array.Builder builder = new Array.Builder();
        int[] indices = new int[0];
        Location here = where;

        try {
            for (;;) {
                skipWhitespace();
                if (!scanner.hasNext() || scanner.peek() == ']') return builder.build(where);
                here = scanner.location();
                DataObject obj = parse(",]");
                builder.set(indices, obj);
                skipBlanks();
                int ch = scanner.peek();
                switch (ch) {
                    case ',':
                        if (indices.length == 0) {
                            indices = new int[] { 1 };
                        } else {
                            indices[indices.length - 1]++;
                        }
                        scanner.next();
                        skipWhitespace();
                        break;
                    case -1:
                    case ']':
                        return builder.build(where);
                    case '\r':
                    case '\n':
                        int dimout = 0;
                        try {
                            for (;;) {
                                here = scanner.location();
                                if (!newline()) break;
                                skipComments();
                                dimout++;
                            }
                        } catch (EOF exc) {
                            // done
                        }
                        int index = indices.length - 1 - dimout;
                        if (index < 0) {
                            indices = new int[indices.length - index];
                            index = 0;
                        } else {
                            for (int j = index + 1; j < indices.length; j++) {
                                indices[j] = 0;
                            }
                        }
                        indices[index]++;
                        break;
                    default:
                        throw new SyntaxError("Unexpected character " + unparseChar(ch), scanner.location());
                }
            }
        } catch (EOF e) {
            throw new SyntaxError(UNEXPECTED_EOF, here);
        }
    }
}
