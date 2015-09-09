# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# We do one global pass over all the WebIDL to generate our prototype enum
# and generate information for subsequent phases.

import sys
import os
sys.path.append(os.path.join(".", "parser"))
sys.path.append(os.path.join(".", "ply"))
import WebIDL
import cPickle
from Configuration import Configuration
from CodegenRust import GlobalGenRoots, replaceFileIfChanged


def generate_file(config, name, filename):
    root = getattr(GlobalGenRoots, name)(config)
    code = root.define()

    if replaceFileIfChanged(filename, code):
        print "Generating %s" % (filename)
    else:
        print "%s hasn't changed - not touching it" % (filename)


def main():
    # Parse arguments.
    from optparse import OptionParser
    usageString = "usage: %prog [options] configFile outputdir webidldir [files]"
    o = OptionParser(usage=usageString)
    o.add_option("--cachedir", dest='cachedir', default=None,
                 help="Directory in which to cache lex/parse tables.")
    o.add_option("--verbose-errors", action='store_true', default=False,
                 help="When an error happens, display the Python traceback.")
    (options, args) = o.parse_args()

    if len(args) < 2:
        o.error(usageString)

    configFile = args[0]
    outputdir = args[1]
    baseDir = args[2]
    fileList = args[3:]

    # Parse the WebIDL.
    parser = WebIDL.Parser(options.cachedir)
    for filename in fileList:
        fullPath = os.path.normpath(os.path.join(baseDir, filename))
        with open(fullPath, 'rb') as f:
            lines = f.readlines()
        parser.parse(''.join(lines), fullPath)
    parserResults = parser.finish()

    # Write the parser results out to a pickle.
    resultsPath = os.path.join(outputdir, 'ParserResults.pkl')
    with open(resultsPath, 'wb') as resultsFile:
        cPickle.dump(parserResults, resultsFile, -1)

    # Load the configuration.
    config = Configuration(configFile, parserResults)

    to_generate = [
        ('PrototypeList', 'PrototypeList.rs'),
        ('RegisterBindings', 'RegisterBindings.rs'),
        ('InterfaceTypes', 'InterfaceTypes.rs'),
        ('InheritTypes', 'InheritTypes.rs'),
        ('Bindings', os.path.join('Bindings', 'mod.rs')),
        ('UnionTypes', 'UnionTypes.rs'),
    ]

    for name, filename in to_generate:
        generate_file(config, name, os.path.join(outputdir, filename))

if __name__ == '__main__':
    main()
