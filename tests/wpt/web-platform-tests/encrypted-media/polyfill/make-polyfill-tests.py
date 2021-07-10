#!/usr/bin/python

from __future__ import print_function

import os, re, os.path, glob

head = re.compile( r"^(\s*</head>)", re.MULTILINE )
runtest = re.compile( r"runTest\(\s*(\S.*?)\s*\)", re.DOTALL )

scripts = '''
    <!-- Polyfill files (NOTE: These are added by auto-generation script) -->
    <script src=/encrypted-media/polyfill/chrome-polyfill.js></script>
    <script src=/encrypted-media/polyfill/firefox-polyfill.js></script>
    <script src=/encrypted-media/polyfill/edge-persistent-usage-record.js></script>
    <script src=/encrypted-media/polyfill/edge-keystatuses.js></script>
    <script src=/encrypted-media/polyfill/clearkey-polyfill.js></script>'''

def process_file( infile, outfile ) :
    with open( outfile, "w" ) as output :
        with open( infile, "r" ) as input :
            output.write( runtest.sub( r"runTest( \1, 'polyfill: ' )", head.sub( scripts + r"\1", input.read() ) ) )

if __name__ == '__main__' :
    if (not os.getcwd().endswith('polyfill')) :
        print("Please run from polyfill directory")
        exit( 1 )

    for infile in glob.glob( "../*.html" ) :
        process_file( infile, os.path.basename( infile ) )

    for infile in glob.glob( "../resources/*.html" ) :
        process_file( infile, os.path.join( "resources", os.path.basename( infile ) ) )
