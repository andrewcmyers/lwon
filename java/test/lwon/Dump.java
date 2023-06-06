package lwon;

import easyIO.EOF;
import easyIO.Scanner;
import lwon.data.DataObject;
import lwon.parse.Parser;

import java.io.FileNotFoundException;
import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

/** A simple LWON demo that reads input from a file or files and outputs formatted
 *  versions of the same data in LWON format.
 */
public class Dump {

    private Dump() {}

    /** Usage: {@code dump [-a] [-d] <file> ...} */
    public static void main(String[] args) {
        int optind = 0;
        boolean dictionary = false, array = false;
        List<String> inputFiles = new ArrayList<>();
        for (; optind < args.length; optind++) {
            String opt = args[optind];
            if (opt.codePointAt(0) != '-') break;
            if (opt.equals("-d")) {
                dictionary = true;
            } else if (opt.equals("-a")) {
                array = true;
            }
        }
        for (int i = optind; i < args.length; i++) inputFiles.add(args[i]);
        if (inputFiles.isEmpty()) inputFiles.add("-");
        try {
            for (String filename : inputFiles) {
                Scanner scanner = filename.equals("-")
                    ? new Scanner(new InputStreamReader(System.in), "stdin")
                    : new Scanner(filename);
                Parser parser = new Parser(scanner);
                while (true) {
                    try {
                        DataObject obj;
                        if (dictionary) obj = parser.parseDictionary();
                        else if (array) obj = parser.parseArray();
                        else obj = parser.parse();
                        StringBuilder b = new StringBuilder();
                        obj.unparse(b, 0);
                        System.out.println(b);
                    } catch (EOF e) {
                        break;
                    }
                }
            }
        } catch (Parser.SyntaxError e) {
            System.err.println(e.getMessage());
        } catch (FileNotFoundException e) {
            System.err.println("File not found: " + e.getMessage());
        }
    }
}