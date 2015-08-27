# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
sys.path.append("./parser/")
sys.path.append("./ply/")
import os
import cPickle
from Configuration import Configuration
from CodegenRust import CGBindingRoot, replaceFileIfChanged


def generate_binding_rs(config, outputprefix, webidlfile):
    """
    |config| Is the configuration object.
    |outputprefix| is a prefix to use for the header guards and filename.
    """

    filename = outputprefix + ".rs"
    root = CGBindingRoot(config, outputprefix, webidlfile)
    if replaceFileIfChanged(filename, root.define()):
        print "Generating binding implementation: %s" % (filename)


def main():
    # Parse arguments.
    from optparse import OptionParser
    usagestring = "usage: %prog configFile outputPrefix webIDLFile"
    o = OptionParser(usage=usagestring)
    o.add_option("--verbose-errors", action='store_true', default=False,
                 help="When an error happens, display the Python traceback.")
    (options, args) = o.parse_args()

    if len(args) != 3:
        o.error(usagestring)
    configFile = os.path.normpath(args[0])
    outputPrefix = args[1]
    webIDLFile = os.path.normpath(args[2])

    # Load the parsing results
    with open('ParserResults.pkl', 'rb') as f:
        parserData = cPickle.load(f)

    # Create the configuration data.
    config = Configuration(configFile, parserData)

    # Generate the prototype classes.
    generate_binding_rs(config, outputPrefix, webIDLFile)

if __name__ == '__main__':
    main()
