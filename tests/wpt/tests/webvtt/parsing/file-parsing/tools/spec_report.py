import os
import sys
import glob
import html
import fnmatch
from os import path

import coverage

OUTPUT_TEMPLATE = """
<!DOCTYPE html>
<html>
<head>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
    <title>Spec Coverage</title>
    <link rel="stylesheet" href="style.css" type="text/css">
    <style>
        .covered {
        }

        .missed {
            background-color: lightcoral;
        }
        code {
            margin: 0;
            padding: 0;
            display:block;
            white-space:pre-wrap;
        }
    </style>
</head>
<body>
    %head
    <div><pre>
        %body
    </pre></div>
</body>
</html>
"""

LINE_TEMPLATE = "<code class=\"%class\">%lineno| %source</code>"

def write_report(data, source_file, output_file):
    module_name, executable_lines, excluded_lines, missing_lines, _ = data
    missing_lines = set(missing_lines)

    with open(output_file, "w") as output, open(source_file, "r") as source:
        lines = source.readlines()

        file_report = []
        padding = len(str(len(lines)))

        for index, line in enumerate(lines):
            line = line[0:-1]
            lineno = index + 1
            line_number = str(lineno).rjust(padding)

            covered = lineno not in missing_lines
            line_class = 'covered' if covered else 'missed'

            formatted_line = (LINE_TEMPLATE.replace('%class', line_class)
                                           .replace('%lineno', line_number)
                                           .replace('%source', html.escape(line)))
            file_report.append(formatted_line)

        report_body = ''.join(file_report)

        report_header = ''

        report = (OUTPUT_TEMPLATE.replace('%head', report_header)
                                 .replace('%body', report_body))
        output.write(report)

def main(argv):
    parsing_path = path.normpath(path.join(path.dirname(__file__), ".."))

    files = argv[1:]
    if not files:
        files = [os.path.join(root, file) for root, _, files in os.walk(parsing_path)
                                          for file in fnmatch.filter(files, '*.vtt')]

    cov = coverage.Coverage()
    cov.start()

    for file_path in files:
        with open(file_path, "r") as file:
            source = file.read()

            import parser
            p = parser.VTTParser(source)
            p.parse()

    cov.stop()

    data = cov.analysis2(parser.__file__)
    write_report(data, parser.__file__, "report.html")

if __name__ == '__main__':
    main(sys.argv)
