#!/usr/bin/python
# CSS Test Suite Manipulation Library
# Initial code by fantasai, joint copyright 2010 W3C and Microsoft
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

import OutputFormats
import Utils
from Groups import TestGroup, excludeDirs
from Sources import SourceTree, SourceCache
from shutil import copytree, rmtree
from os.path import join
import os
from mercurial import ui as UserInterface, hg

class TestSuite:
  """Representation of a standard CSS test suite."""

  def __init__(self, name, title, specUri, draftUri, sourceCache = None, ui = None):
    self.name = name
    self.title = title
    self.specroot = specUri
    self.draftroot = draftUri

    self.ui = ui if ui else UserInterface.ui()
    self.defaultReftestRelpath='reftest.list'
    self.groups = {}
    self.sourcecache = sourceCache if sourceCache else SourceCache(SourceTree(hg.repository(self.ui, '.')))
    self.formats = ('html4', 'xhtml1', 'xhtml1print') # XXX FIXME, hardcoded list is lame
    self.rawgroups = {}

  def addTestsByExt(self, dir, ext, groupName='', groupTitle=''):
    """Add tests from directory `dir` by file extension (via `ext`, e.g. ext='.xht').
    """
    group = TestGroup(self.sourcecache, dir, selfTestExt=ext,
                      name=groupName, title=groupTitle, ui = self.ui)
    self.addGroup(group)


  def addTestsByList(self, dir, filenames, groupName='', groupTitle=''):
    """Add tests from directory `dir`, via file name list `filenames`.
    """
    group = TestGroup(self.sourcecache, dir, selfTestList=filenames,
                      name=groupName, title=groupTitle, ui = self.ui)
    self.addGroup(group)

  def addReftests(self, dir, manifestPath, groupName='', groupTitle=''):
    """Add tests by importing context of directory `dir` and importing all
       tests listed in the `reftestManifestName` manifest inside `dir`.
    """
    group = TestGroup(self.sourcecache,
                      dir, manifestPath=manifestPath,
                      manifestDest=self.defaultReftestRelpath,
                      name=groupName, title=groupTitle, ui = self.ui)
    self.addGroup(group)

  def addGroup(self, group):
    """ Add CSSTestGroup `group` to store. """
    master = self.groups.get(group.name)
    if master:
      master.merge(group)
    else:
      self.groups[group.name] = group

  def addRaw(self, dir, relpath):
    """Add the contents of directory `dir` to the test suite by copying
       (not processing). Note this means such tests will not be indexed.
       `relpath` gives the directory's path within the build destination.
    """
    self.rawgroups[dir] = relpath

  def setFormats(self, formats):
    self.formats = formats
    
  def buildInto(self, dest, indexer):
    """Builds test suite through all OutputFormats into directory at path `dest`
       or through OutputFormat destination `dest`, using Indexer `indexer`.
    """
    if isinstance(dest, OutputFormats.BasicFormat):
      formats = (dest,)
      dest = dest.root
    else:
      formats = []
      for format in self.formats:
        if (format == 'html4'):
          formats.append(OutputFormats.HTMLFormat(dest, self.sourcecache.sourceTree))
        elif (format == 'html5'):
          formats.append(OutputFormats.HTML5Format(dest, self.sourcecache.sourceTree))
        elif (format == 'xhtml1'):
          formats.append(OutputFormats.XHTMLFormat(dest, self.sourcecache.sourceTree))
        elif (format == 'xhtml1print'):
          formats.append(OutputFormats.XHTMLPrintFormat(dest, self.sourcecache.sourceTree, self.title))
        elif (format == 'svg'):
          formats.append(OutputFormats.SVGFormat(dest, self.sourcecache.sourceTree))

    for format in formats:
      for group in self.groups.itervalues():
        group.build(format)

    for group in self.groups.itervalues():
      indexer.indexGroup(group)

    for format in formats:
      indexer.writeIndex(format)


    rawtests = []
    for src, relpath in self.rawgroups.items():
      copytree(src, join(dest,relpath))
      for (root, dirs, files) in os.walk(join(dest,relpath)):
        for xdir in excludeDirs:
          if xdir in dirs:
            dirs.remove(xdir)
            rmtree(join(root,xdir))
        rawtests.extend(
          [join(Utils.relpath(root,dest),file)
           for file in files]
        )

    rawtests.sort()
    indexer.writeOverview(dest, addTests=rawtests)
    
