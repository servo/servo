#!/usr/bin/python

# This file is licensed under CC Zero

import os
from os.path import join
import shutil
import filecmp

# Files to not sync across support/ directories
fileExcludes = ('README')
dirExcludes = ('.svn', '.hg', 'CVS')
# To prevent support files from being propagated into a particular support/
# directory, add a file named LOCK

def propagate(source, dest, errors):
  """Compare each file and copy from source to destination.
     Do nothing and flag an error if the destination already exists
     but is different. Recurse.
     source and dest are both directory paths.
     errors is a list of 2-element tuples, the first being a
       source filepath and the second a destination filepath,
       of file pairs where the destination isdifferent from
  """

  # This directory is locked; don't propagate
  if os.path.exists(join(dest, 'LOCK')): return

  # If the source directory doesn't exist return
  if not os.path.exists(source): return

  # Get the file and directory lists for source
  children = os.listdir(source)
  for name in children:
    origin = join(source, name)
    copy = join(dest, name)
    if os.path.isfile(origin):
      if name in fileExcludes: continue
      # Copy over the file if it needs copying
      if not os.path.exists(copy): # file missing
        shutil.copy2(origin, copy) # copy it over
      elif not filecmp.cmp(origin, copy): # files differ
        if not filecmp.cmp(origin, copy, True): # contents equal, stats differ
          shutil.copystat(origin, copy) # update stats so they match for next time
        else: # contents differ
          errors.append((origin, copy))
    elif os.path.isdir(origin):
      if name in dirExcludes: continue
      # Duplicate the directory structure and propagate the subtree
      if not os.path.exists(copy):
        os.makedirs(copy)
      propagate(origin, copy, errors)
  if len(children) == 0:
    print "Warn: " + source + " is empty.\n"

def waterfall(parentDir, childDir, errors):
  """Copy down support files from parent support to child.
     parentDir is the parent of the parent support directory.
     childDir is the parent of the current support directory,
     that we should copy into.
     waterfall recurses into childDir's children."""
  assert os.path.exists(join(parentDir, 'support')), join(parentDir, 'support') + " doesn't exist\n"
  if os.path.exists(join(childDir, 'support')):
    propagate(join(parentDir, 'support'), join(childDir, 'support'), errors)
    dirs = os.walk(childDir).next()[1]
    for name in dirs:
      if name == 'support':
        pass
      elif name not in dirExcludes:
        waterfall(childDir, join(childDir, name), errors)

def outline(source, dest, errors):
  """Copy over directory structure and all files under any support/ directories
     source and dest are both directory paths.
     errors is a list of 2-element tuples, the first being a
       source filepath and the second a destination filepath,
       of support file pairs where the destination copy is
       different from the source
  """
  # Get the directory list for source
  dirs = os.walk(source).next()[1]
  # Copy directory structure
  for name in dirs:
    if name in dirExcludes: continue
    origin = join(source, name)
    copy = join(dest, name)
    if not os.path.exists(copy):
      os.mkdirs(copy)
    if name == 'support':
      # Copy support files
      propagate(origin, copy, errors)
    else:
      outline(origin, copy, errors)

def syncSupport(source, dest, errors):
  """For each support directory in dest, propagate the corresponding support
     files from source.
     source and dest are both directory paths.
     errors is a list of 2-element tuples, the first being a
       source filepath and the second a destination filepath,
       of support file pairs where the destination copy is
       different from the source
  """
  # Get the directory list for est
  dirs = os.walk(dest).next()[1]
  # Scan directory structure, building support dirs as necessary
  for name in dirs:
    if name in dirExcludes: continue
    master = join(source, name)
    slave  = join(dest, name)
    if name == 'support':
      # Copy support files
      propagate(master, slave, errors)
    else:
      syncSupport(master, slave, errors)

def main():
  # Part I: Propagate support files through approved/

  errors = []
  root, dirs, _ = os.walk('.').next()
  if 'approved' in dirs:
    root = join(root, 'approved')
    suites = os.walk(root).next()[1]
    suites = filter(lambda name: name not in dirExcludes, suites)
    for suite in suites:
      waterfall(root, join(root, suite, 'src'), errors)
  else:
    print "Failed to find approved/ directory.\n"
    exit();

  # Part II: Propagate test suite support files into contributors/

  if 'contributors' in dirs:
    _, contribs, _ = os.walk('contributors').next()
    for contributor in contribs:
      contribRoot = join('contributors', contributor, 'submitted')
      if not os.path.exists(contribRoot): continue # missing submitted folder
      dirs = os.walk(contribRoot).next()[1]
      for dir in dirs:
        # Check if contributor has a top-level directory name matching
        # one of our suites; if so, sync any matching support directories
        if dir in suites:
          suiteRoot = join(root, dir, 'src')
          if os.path.exists(suiteRoot):
            syncSupport(suiteRoot, join(contribRoot, dir), errors)
  else:
    print "Failed to find contributors/ directory.\n"

  # Print all errors

  for error in errors:
    print "Mismatch: " + error[0] + " vs " + error [1] + " Copy failed.\n"

if __name__ == "__main__":
  main()
