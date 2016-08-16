# Copyright (c) 2016 W3C
# Released under the W3C Test Suite License: see LICENSE.txt

# This tool creates .html test files for the WPT harness from corresponding .text
# files that it finds in the tree for this test collection.


import re
import time
import json
import fnmatch
import os
import shutil
import sys
import argparse

TESTTREE = '..'
DEFDIR   = '../definitions'
TEMPLATE = 'template'


parser = argparse.ArgumentParser()

parser.add_argument('--examples', action="store_const", const=1)

args = parser.parse_args()

# pull in the template

template = open(TEMPLATE, "r").read()

defList = []
defnames = ""

# find all of the definitions
for curdir, subdirList, fileList in os.walk(DEFDIR, topdown=True):
  for file in fnmatch.filter(fileList, "*.json"):
    theFile = os.path.join(curdir, file)
    try:
      testJSON = json.load(open(theFile, "r"))
    except ValueError as e:
      print "parse of " + theFile + " failed: " + e[0]
    else:
      theFile = re.sub("\.\./", "", theFile)
      defList.append(theFile)

if (len(defList)):
    defNames = '"' + '",\n  "'.join(defList) + '"'


# iterate over the folders looking for .test files

for curdir, subdirList, fileList in os.walk(TESTTREE, topdown=True):
  # sjip the definitions directory
  subdirList[:] = [d for d in subdirList if d != "definitions"]
  # skip the examples directory
  if args.examples != 1:
    subdirList[:] = [d for d in subdirList if d != "examples"]

  for file in fnmatch.filter(fileList, "*.test"):
# for each .test file, create a corresponding .html file using template
    theFile = os.path.join(curdir, file)
    try:
      testJSON = json.load(open(theFile, "r"))
    except ValueError as e:
      print "parse of " + theFile + " failed: " + e[0]
    else:
      rfile = re.sub("\.\./", "", file)
      # interesting pattern is {{TESTFILE}}
      tcopy = re.sub("{{TESTFILE}}", rfile, template)

      tcopy = re.sub("{{SCHEMADEFS}}", defNames, tcopy)

      if testJSON['name']:
        tcopy = re.sub("{{TESTTITLE}}", testJSON['name'], tcopy)

      # target file is basename of theFile + '-manual.html'
      target = re.sub("\.test","-manual.html", theFile)

      try:
        out = open(target, "w")
        out.write(tcopy)
        out.close()
      except:
        print("Failed to create "+target)
      else:
        print("Created " + target)
