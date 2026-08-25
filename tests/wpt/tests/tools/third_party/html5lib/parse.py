#!/usr/bin/env python
"""
Parse a document to a tree, with optional profiling
"""

import argparse
import sys
import traceback

from html5lib import html5parser
from html5lib import treebuilders, serializer, treewalkers
from html5lib import constants
from html5lib import _utils


def parse():
    parser = get_parser()
    opts = parser.parse_args()
    encoding = "utf8"

    try:
        f = opts.filename
        # Try opening from the internet
        if f.startswith('http://'):
            try:
                import urllib.request
                import urllib.parse
                import urllib.error
                import cgi
                f = urllib.request.urlopen(f)
                contentType = f.headers.get('content-type')
                if contentType:
                    (mediaType, params) = cgi.parse_header(contentType)
                    encoding = params.get('charset')
            except Exception:
                pass
        elif f == '-':
            f = sys.stdin
            if sys.version_info[0] >= 3:
                encoding = None
        else:
            try:
                # Try opening from file system
                f = open(f, "rb")
            except IOError as e:
                sys.stderr.write("Unable to open file: %s\n" % e)
                sys.exit(1)
    except IndexError:
        sys.stderr.write("No filename provided. Use -h for help\n")
        sys.exit(1)

    treebuilder = treebuilders.getTreeBuilder(opts.treebuilder)

    p = html5parser.HTMLParser(tree=treebuilder, debug=opts.log)

    if opts.fragment:
        parseMethod = p.parseFragment
    else:
        parseMethod = p.parse

    if opts.profile:
        import cProfile
        import pstats
        cProfile.runctx("run(parseMethod, f, encoding, scripting)", None,
                        {"run": run,
                         "parseMethod": parseMethod,
                         "f": f,
                         "encoding": encoding,
                         "scripting": opts.scripting},
                        "stats.prof")
        # XXX - We should use a temp file here
        stats = pstats.Stats('stats.prof')
        stats.strip_dirs()
        stats.sort_stats('time')
        stats.print_stats()
    elif opts.time:
        import time
        t0 = time.time()
        document = run(parseMethod, f, encoding, opts.scripting)
        t1 = time.time()
        if document:
            printOutput(p, document, opts)
            t2 = time.time()
            sys.stderr.write("\n\nRun took: %fs (plus %fs to print the output)" % (t1 - t0, t2 - t1))
        else:
            sys.stderr.write("\n\nRun took: %fs" % (t1 - t0))
    else:
        document = run(parseMethod, f, encoding, opts.scripting)
        if document:
            printOutput(p, document, opts)


def run(parseMethod, f, encoding, scripting):
    try:
        document = parseMethod(f, override_encoding=encoding, scripting=scripting)
    except Exception:
        document = None
        traceback.print_exc()
    return document


def printOutput(parser, document, opts):
    if opts.encoding:
        print("Encoding:", parser.tokenizer.stream.charEncoding)

    for item in parser.log:
        print(item)

    if document is not None:
        if opts.xml:
            tb = opts.treebuilder.lower()
            if tb == "dom":
                document.writexml(sys.stdout, encoding="utf-8")
            elif tb == "lxml":
                import lxml.etree
                sys.stdout.write(lxml.etree.tostring(document, encoding="unicode"))
            elif tb == "etree":
                sys.stdout.write(_utils.default_etree.tostring(document, encoding="unicode"))
        elif opts.tree:
            if not hasattr(document, '__getitem__'):
                document = [document]
            for fragment in document:
                print(parser.tree.testSerializer(fragment))
        elif opts.html:
            kwargs = {}
            for opt in serializer.HTMLSerializer.options:
                try:
                    kwargs[opt] = getattr(opts, opt)
                except Exception:
                    pass
            if not kwargs['quote_char']:
                del kwargs['quote_char']

            if opts.sanitize:
                kwargs["sanitize"] = True

            tokens = treewalkers.getTreeWalker(opts.treebuilder)(document)
            if sys.version_info[0] >= 3:
                encoding = None
            else:
                encoding = "utf-8"
            for text in serializer.HTMLSerializer(**kwargs).serialize(tokens, encoding=encoding):
                sys.stdout.write(text)
            if not text.endswith('\n'):
                sys.stdout.write('\n')
    if opts.error:
        errList = []
        for pos, errorcode, datavars in parser.errors:
            errList.append("Line %i Col %i" % pos + " " + constants.E.get(errorcode, 'Unknown error "%s"' % errorcode) % datavars)
        sys.stdout.write("\nParse errors:\n" + "\n".join(errList) + "\n")


def get_parser():
    parser = argparse.ArgumentParser(description=__doc__)

    parser.add_argument("-p", "--profile", action="store_true",
                        help="Use the hotshot profiler to "
                        "produce a detailed log of the run")

    parser.add_argument("-t", "--time",
                        action="store_true",
                        help="Time the run using time.time (may not be accurate on all platforms, especially for short runs)")

    parser.add_argument("-b", "--treebuilder",
                        default="etree")

    parser.add_argument("-e", "--error", action="store_true",
                        help="Print a list of parse errors")

    parser.add_argument("-f", "--fragment", action="store_true",
                        help="Parse as a fragment")

    parser.add_argument("-s", "--scripting", action="store_true",
                        help="Handle noscript tags as if scripting was enabled")

    parser.add_argument("--tree", action="store_true",
                        help="Output as debug tree")

    parser.add_argument("-x", "--xml", action="store_true",
                        help="Output as xml")

    parser.add_argument("--no-html", action="store_false",
                        dest="html", help="Don't output html")

    parser.add_argument("-c", "--encoding", action="store_true",
                        help="Print character encoding used")

    parser.add_argument("--inject-meta-charset", action="store_true",
                        help="inject <meta charset>")

    parser.add_argument("--strip-whitespace", action="store_true",
                        help="strip whitespace")

    parser.add_argument("--omit-optional-tags", action="store_true",
                        help="omit optional tags")

    parser.add_argument("--quote-attr-values", action="store_true",
                        help="quote attribute values")

    parser.add_argument("--use-best-quote-char", action="store_true",
                        help="use best quote character")

    parser.add_argument("--quote-char",
                        help="quote character")

    parser.add_argument("--no-minimize-boolean-attributes",
                        action="store_false",
                        dest="minimize_boolean_attributes",
                        help="minimize boolean attributes")

    parser.add_argument("--use-trailing-solidus", action="store_true",
                        help="use trailing solidus")

    parser.add_argument("--space-before-trailing-solidus",
                        action="store_true",
                        help="add space before trailing solidus")

    parser.add_argument("--escape-lt-in-attrs", action="store_true",
                        help="escape less than signs in attribute values")

    parser.add_argument("--escape-rcdata", action="store_true",
                        help="escape rcdata element values")

    parser.add_argument("--sanitize", action="store_true",
                        help="sanitize")

    parser.add_argument("-l", "--log", action="store_true",
                        help="log state transitions")

    parser.add_argument("filename")

    return parser


if __name__ == "__main__":
    parse()
