package lwon.parse;

import easyIO.EOF;
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

    public Parser(Scanner input) {
        scanner = input;
    }

    public static class SyntaxError extends Exception {
        SyntaxError(String msg, Location location) {
            super(location + ": " + msg);
        }
    }

    private static final String UNEXPECTED_EOF = "Unexpected end of input";

    public DataObject parse() throws SyntaxError, EOF {
        Location location = scanner.location();
        return parse(location, "");
    }

    public DataObject parse(Location start, String delimiters) throws EOF, SyntaxError {
            DataObject result;
            scanner.whitespace();
            Location first = scanner.location();
            int c = scanner.peek();
            switch (c) {
                case -1:
                    throw new SyntaxError(UNEXPECTED_EOF, start);
                case '{':
                    expect('{');
                    result = parseDictionary(first);
                    expect('}');
                    return result;
                case '[':
                    expect('[');
                    result = parseArray(first);
                    expect(']');
                    return result;
                case '"':
                    expect('"');
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

    private void expect(char expected) throws EOF, SyntaxError {
        Location here = scanner.location();
        char c = scanner.next();
        if (c != expected) {
            throw new SyntaxError("Expected " + unparseChar(expected), here);
        }
    }

    private String unparseChar(char c) {
        return StringEscapeUtils.escapeJava(String.valueOf(c));
    }

    private boolean reservedCharacter(char c) {
        return (-1 != reservedChars.indexOf(c));
    }

    public Dictionary parseDictionary(Location start) throws SyntaxError {
        try {
            Dictionary.Builder b = new Dictionary.Builder();
            for (;;) {
                scanner.whitespace();
                if (scanner.peek() == '}' || scanner.peek() == -1) break;
                String key = parseKey(start);
                if (scanner.peek() == ':')  {
                    scanner.next();
                    scanner.whitespace();
                }
                DataObject value = parse(scanner.location(), "}");
                b.put(key, value);
                scanner.whitespace();
                if (scanner.peek() == ',') scanner.next();
            }
            return b.build(start);
        } catch (EOF exc) {
            throw new SyntaxError(UNEXPECTED_EOF, start);
        }
    }

    private String parseKey(Location start) throws SyntaxError {
        Text t = parseText(false, ":", start);
        return t.value();
    }

    public static final String reservedChars = "{}[]:";

    /** parse text that is either single-line or multi-line text. In the former
     *  case, a newline is a delimiter.
     */
    public Text parseText(boolean multiline, String delimiters, Location start) throws SyntaxError {
      try {
          StringBuilder b = new StringBuilder();
          StringBuilder ws = new StringBuilder();
          if (!multiline && -1 != reservedChars.indexOf(scanner.peek())) {
              throw new SyntaxError("Unexpected reserved character " +
                  Character.toString(scanner.peek()) + " at start of short string", start);
          }
          int leftColumn = scanner.column();
          for (;;) {
              int ch = scanner.peek();
              switch (ch) {
                  case -1:
                      throw new SyntaxError(UNEXPECTED_EOF, start);
                  case '\\':
                      if (!multiline && !ws.isEmpty()) {
                          b.append(ws);
                          ws = new StringBuilder();
                      }
                      try {
                          scanner.newline();
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
                  case ' ':
                      if (multiline) {
                          b.append(Character.toChars(scanner.next()));
                      } else {
                          ws.append(scanner.next());
                      }
                      break;
                  case '\r':
                  case '\n':
                      try {
                          if (!multiline) return new Text(b.toString(), start);
                          scanner.newline();
                          b.append(System.lineSeparator());
                          while (scanner.peek() == ' ' && scanner.column() <= leftColumn)
                              scanner.next();
                          leftColumn = skipLeadingSpace(leftColumn);
                      } catch (UnexpectedInput e) {
                          b.append(Character.toChars(ch));
                      }
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
          throw new Error("Can't happen");
      }
    }

    private int skipLeadingSpace(int leftColumn) throws EOF {
        while (scanner.peek() == ' ' && scanner.column() < leftColumn)
            scanner.next();
        return scanner.column();
    }

    static final String sep = System.lineSeparator();

    /** If the next characters on the input are */
    private boolean atNewline() {
        try {
            scanner.mark();
            // assume line separator does not contain supplementary characters
            for (int i = 0; i < sep.length(); i++) {
                if (scanner.next() != sep.charAt(i)) {
                    scanner.abort();
                    return false;
                }
            }
            scanner.accept();
            return true;
        } catch (EOF exc) {
            scanner.abort();
            return false;
        }
    }

    String hexdigits = "0123456789abcdef";
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
                int digits = 0;
                while (scanner.hasNext()) {
                    int i = hexdigits.indexOf(Character.toLowerCase(scanner.peek()));
                    if (i == -1 || !braces && ++digits >= 4) break;
                    c = c * 16 + i;
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
            case ':': return ":";
            default:
                throw new SyntaxError("Illegal escape sequence", where);
        }
    }

    private static final String blanks = " \t\f\r";

    public void skipBlanks() {
        try {
            while (scanner.hasNext() && -1 != blanks.indexOf(scanner.peek())) {
                scanner.next();
            }
        } catch (EOF exc) {
            // nothing left
        }
    }

    public Array parseArray(Location where) throws SyntaxError {
        Array.Builder builder = new Array.Builder();
        int[] indices = new int[0];
        Location here = where;

        scanner.whitespace();
        try {
            for (;;) {
                skipBlanks();
                if (!scanner.hasNext()) return builder.build(here);
                here = scanner.location();
                DataObject obj = parse(here, ",]");
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
                        break;
                    case -1:
                    case ']':
                        return builder.build(here);
                    case '\r':
                    case '\n':
                        int dimout = 0;
                        try {
                            for (;;) {
                                scanner.newline();
                                dimout++;
                            }
                        } catch (UnexpectedInput exc) {
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
                        throw new SyntaxError("Unexpected character " + ch, here);
                }
            }
        } catch (EOF e) {
            throw new SyntaxError(UNEXPECTED_EOF, here);
        }
    }
}