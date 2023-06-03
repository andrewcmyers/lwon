import easyIO.BacktrackScanner;
import easyIO.EOF;
import easyIO.Scanner;
import easyIO.UnexpectedInput;
import lwon.data.DataObject;
import lwon.parse.Parser;

import java.io.InputStreamReader;
import java.util.ArrayList;
import java.util.List;

public class Main {
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
        try {
            Scanner scanner = new Scanner(new InputStreamReader(System.in), "stdin");
            Parser parser = new Parser(scanner);
            while (true) {
                try {
                    DataObject obj;
                    if (dictionary) obj = parser.parseDictionary(scanner.location());
                    else if (array) obj = parser.parseArray(scanner.location());
                    else obj = parser.parse();
                    StringBuilder b = new StringBuilder();
                    obj.unparse(b, 0);
                    System.out.println(b);
                } catch (EOF e) {
                    // exit
                }
            }
        } catch (Parser.SyntaxError e) {
            System.err.println(e.getMessage());
        }
    }
}