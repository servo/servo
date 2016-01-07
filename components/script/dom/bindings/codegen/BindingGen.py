# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
import os
sys.path.append(os.path.join(".", "parser"))
sys.path.append(os.path.join(".", "ply"))
import cPickle
from Configuration import Configuration
from CodegenRust import CGBindingRoot, replaceFileIfChanged


def generate_binding_rs(config, outputprefix, webidlfile):
    """
    |config| Is the configuration object.
    |outputprefix| is a prefix to use for the header guards and filename.
    """

    filename = outputprefix + ".rs"
    module = CGBindingRoot(config, outputprefix, webidlfile).define()
    if not module:
        print "Skipping empty module: %s" % (filename)
    elif replaceFileIfChanged(filename, module):
        print "Generating binding implementation: %s" % (filename)


def main():
    # Parse arguments.
    from optparse import OptionParser
    usagestring = "usage: %prog configFile outputdir outputPrefix webIDLFile"
    o = OptionParser(usage=usagestring)
    (options, args) = o.parse_args()

    if len(args) != 4:
        o.error(usagestring)
    configFile = os.path.normpath(args[0])
    outputdir = args[1]
    outputPrefix = args[2]
    webIDLFile = os.path.normpath(args[3])

    # Load the parsing results
    resultsPath = os.path.join(outputdir, 'ParserResults.pkl')
    with open(resultsPath, 'rb') as f:
        parserData = cPickle.load(f)

    # Create the configuration data.
    config = Configuration(configFile, parserData)

    # Generate the prototype classes.
    generate_binding_rs(config, outputPrefix, webIDLFile)

if __name__ == '__main__':
    main()
