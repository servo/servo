#!/usr/bin/env python2.7

# Copyright (c) 2016 PowerMapper Software
#
# Permission is hereby granted, free of charge, to any person obtaining a
# copy of this software and associated documentation files (the "Software"),
# to deal in the Software without restriction, including without limitation
# the rights to use, copy, modify, merge, publish, distribute, sublicense,
# and/or sell copies of the Software, and to permit persons to whom the
# Software is furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
# THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
# FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
# DEALINGS IN THE SOFTWARE.

"""build_svg_tests.py.

This script builds a set of SVG-in-HTML test files for the Nu Html Checker
based on the SVG 1.1 Second Edition Test Suite
http://www.w3.org/Graphics/SVG/Test/20110816/archives/W3C_SVG_11_TestSuite.tar.gz

"""

import logging
import os
import sys, getopt
import urllib2

# some files in the SVG 1.1 test suite don't validate against the SVG 1.1 DTD
# but are valid against the HTML 5 spec

valid_svg_files = dict([
    # these entries manually added after cross checking behaviour with spec

    # VNU warns about text not in Unicode Normalization Form C, but it's not an error
    ('struct-cond-02-t-manual.svg', 'Source text is not in Unicode Normalization Form C'),
    # FiLl, fill and FILL are all valid in case-insensitive HTML (but SVG DTD is case-sensitive)
    ('styling-css-10-f-manual.svg', 'Attribute FiLl not allowed on SVG element circle at this point')
])

# some files in the SVG 1.1 test suite don't validate against the SVG 1.1 DTD
# and some files are marked as version='SVG 1.2'.
# this is used to toggle between -isvalid.html and -novalid.html output

invalid_svg_files = dict([
    # 'DTD Invalid' entries are produced by calling validate_svg_dtd (see below)
    ('animate-elem-24-t-manual.svg', 'DTD Invalid'),
    ('animate-elem-77-t-manual.svg', 'DTD Invalid'),
    ('animate-pservers-grad-01-b-manual.svg', 'DTD Invalid'),
    ('conform-viewers-03-f-manual.svg', 'DTD Invalid'),
    ('coords-dom-01-f-manual.svg', 'DTD Invalid'),
    ('coords-dom-02-f-manual.svg', 'DTD Invalid'),
    ('extend-namespace-01-f-manual.svg', 'DTD Invalid'),
    ('filters-color-02-b-manual.svg', 'DTD Invalid'),
    ('filters-conv-02-f-manual.svg', 'DTD Invalid'),
    ('filters-conv-04-f-manual.svg', 'DTD Invalid'),
    ('filters-conv-05-f-manual.svg', 'DTD Invalid'),
    ('filters-light-05-f-manual.svg', 'DTD Invalid'),
    ('fonts-glyph-04-t-manual.svg', 'DTD Invalid'),
    ('interact-pointer-02-t-manual.svg', 'DTD Invalid'),
    ('linking-a-09-b-manual.svg', 'DTD Invalid'),
    ('linking-a-10-f-manual.svg', 'DTD Invalid'),
    ('masking-filter-01-f-manual.svg', 'DTD Invalid'),
    ('masking-intro-01-f-manual.svg', 'DTD Invalid'),
    ('painting-marker-04-f-manual.svg', 'DTD Invalid'),
    ('paths-data-18-f-manual.svg', 'DTD Invalid'),
    ('paths-data-20-f-manual.svg', 'DTD Invalid'),
    ('pservers-grad-23-f-manual.svg', 'DTD Invalid'),
    ('render-elems-03-t-manual.svg', 'DTD Invalid'),
    ('shapes-rect-03-t-manual.svg', 'DTD Invalid'),
    ('struct-cond-02-t-manual.svg', 'DTD Invalid'),
    ('struct-dom-17-f-manual.svg', 'DTD Invalid'),
    ('struct-dom-19-f-manual.svg', 'DTD Invalid'),
    ('struct-frag-05-t-manual.svg', 'DTD Invalid'),
    ('struct-image-12-b-manual.svg', 'DTD Invalid'),
    ('struct-use-11-f-manual.svg', 'DTD Invalid'),
    ('struct-use-12-f-manual.svg', 'DTD Invalid'),
    ('styling-css-10-f-manual.svg', 'DTD Invalid'),
    ('styling-pres-02-f-manual.svg', 'DTD Invalid'),
    ('svgdom-over-01-f-manual.svg', 'DTD Invalid'),
    ('text-dom-03-f-manual.svg', 'DTD Invalid'),
    ('text-fonts-03-t-manual.svg', 'DTD Invalid'),
    ('text-fonts-05-f-manual.svg', 'DTD Invalid'),
    ('text-tref-02-b-manual.svg', 'DTD Invalid'),
    ('types-dom-04-b-manual.svg', 'DTD Invalid'),

    # these entries manually added after cross checking behaviour with spec
    # note there are some confusing differences between w:iri-ref (used in HTML for img/@src)
    # and xsd:anyURI (used in SVG for image/@xlink:href)
    ('conform-viewers-02-f-manual.svg', 'Newlines in data: URI - not allowed by URL Standard or RFC 2397.'),
    ('coords-transformattr-01-f-manual.svg', 'Numeric character reference expanded to carriage return - not allowed in HTML5 - see 8.1.4'),
    ('fonts-overview-201-t-manual.svg', 'Unsupported SVG version specified - specifies SVG 1.2'),
    ('script-specify-01-f-manual.svg', 'Attribute contentscripttype not allowed on element svg at this point - not allowed in HTML5 - see 4.8.18 SVG'),
    ('types-dom-04-b-manual.svg', 'Attribute externalresourcesrequired not allowed on element svg at this point - not allowed in HTML5 - see 4.8.18 SVG'),
    ('metadata-example-01-t-manual.svg', 'Element rdf:rdf not allowed as child of element metadata in this context - namespaced XML not allowed in HTML5')
])

# TODO Github Issue #216 MathML and SVG uses xsd:anyURI, HTML URLs use URL Standard
# TODO Github Issue #217 NU has script/@type optional for HTML, but not SVG-in-HTML

def build_html_testfiles(svgdirectory,htmldirectory):
    """Builds HTML test files from SVG test suite folder."""

    logging.debug('build_html_testfiles: IN')

    testfiles = []

    for filename in os.listdir(svgdirectory):
        #logging.debug(filename)
        if filename.endswith(".svg"):
            htmlpathname = build_html_test_file(filename, svgdirectory, htmldirectory)
            if htmlpathname:
                testfiles.append(htmlpathname)
        pass
    pass


def build_html_test_file(filename, svgdirectory, htmldirectory):
    """Builds HTML test file by wrapping input SVG in boilerplate HTML."""

    svgpathname = svgdirectory + "/" + filename

    # valid_svg_file overrides invalid_svg_files (may invalid in case-sensitive XML but valid in case-insensitive HTML)
    if invalid_svg_files.has_key(filename) and not valid_svg_files.has_key(filename):
        htmlpathname = htmldirectory + "/" + filename.replace( "-manual.svg", "-novalid.html")
    else:
        htmlpathname = htmldirectory + "/" + filename.replace( "-manual.svg", "-isvalid.html")

    logging.debug(svgpathname)
    logging.debug(htmlpathname)

    # read SVG data
    svgfile = open(svgpathname, "rU")
    svg = svgfile.read()
    svgfile.close()

    # but remove <d:SVGTestCase> from file (not valid in HTML or SVG DTD)
    svg = svg.replace('<?xml version="1.0" encoding="UTF-8"?>', '')
    svgbefore = svg.split("<d:SVGTestCase")[0];
    svgafter = svg.split("</d:SVGTestCase>")[1];
    svg = svgbefore + svgafter

    # ignore files with SVG DOCTYPE and !ENTITY declarations (unsupported in HTML)
    if svg.find( "<!DOCTYPE" ) != -1:
        return

    # uncomment these 2 lines to generate 'DTD Invalid' entries for invalid_svg_files dict above
    # very slow operation - only needs done if the SVG test suite ever changes
    # when uncommented expect to see AttributeError: 'NoneType' object has no attribute 'find'
    #validate_svg_dtd(filename, svg)
    #return

    htmlfile = open(htmlpathname, "w")

    htmlfile.write("<!DOCTYPE html>\n")
    htmlfile.write("<html lang='en'>\n")

    htmlfile.write("<head>\n")
    htmlfile.write(" <title>%s</title>\n" % os.path.basename(svgpathname) )
    htmlfile.write(" <meta charset='utf-8'>\n")
    htmlfile.write("</head>\n")

    htmlfile.write("<body>\n")
    htmlfile.write(" <h1>Source SVG: %s</h1>\n" % os.path.basename(svgpathname) )

    # insert SVG without any XML processing to avoid unexpected transformations on
    # encoding and entity refs, but remove <d:SVGTestCase> from file (not valid in HTML)
    htmlfile.write(svgbefore)
    htmlfile.write(svgafter)

    htmlfile.write("</body>\n")

    htmlfile.write("</html>\n")
    htmlfile.close()

    return htmlpathname

def create_dir_if_missing(directory):
    """Create the given directory if it doesn't exist"""

    d = os.path.dirname(directory)
    if not os.path.exists(directory):
        os.makedirs(directory)


def validate_svg_dtd(filename,svg):
    """Prints legacy DTD markup validation status to stdout in a format suitable for pasting into invalid_svg_files dict above."""

    # setup multipart/form-data POST body
    body = ''
    body = body + '--AaB03x\r\n'
    body = body + 'Content-Disposition: form-data; name="fieldname"\r\n'
    body = body + '\r\n'
    body = body + 'value\r\n'
    body = body + '--AaB03x\r\n'
    body = body + 'Content-Disposition: form-data; name="uploaded_file"; filename="test.svg"\r\n'
    body = body + 'Content-Type: image/svg+xml\r\n'
    body = body + '\r\n'
    body = body + svg
    body = body + '\r\n'
    body = body + '--AaB03x--\r\n'

    # send request to W3 DTD validator for SVG 1.1 validation
    headers = { "Content-type" : "multipart/form-data; boundary=AaB03x", "Content-length" : len(body) }
    request = urllib2.Request("http://validator.w3.org/check?charset=utf-8&doctype=SVG+1.1&output=json", data=body, headers=headers)
    response = urllib2.urlopen(request, timeout=60)

    status = response.info().getheader('X-W3C-Validator-Status')
    logging.debug(status)

    if status == "Valid":
        return True

    print "    ('%s', 'DTD %s')," % (filename, status)
    return False


def main():

    #logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(message)s')
    logging.debug('main: IN')

    ccdir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    svgdirectory = os.path.join(os.path.dirname(ccdir), "svg", "import")
    htmldirectory = os.path.join(ccdir, "html-svg")

    try:
        opts, args = getopt.getopt(sys.argv[1:],"",["svgdir=","outdir="])
    except getopt.GetoptError:
        print 'build-svg-tests.py --svgdir <indir> --outdir <outdir>'
        sys.exit(2)

    for opt, arg in opts:
        print opt, arg
        if opt in ("-s", "--svgdir"):
            svgdirectory = arg
        elif opt in ("-o", "--outdir"):
            htmldirectory = arg


    create_dir_if_missing(htmldirectory)
    build_html_testfiles(svgdirectory, htmldirectory)


main()
