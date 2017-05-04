#!/usr/bin/python
# CSS Test Source Manipulation Library
# Initial code by fantasai, joint copyright 2010 W3C and Microsoft
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

from os.path import basename, exists, join
import os
import filecmp
import shutil
import re
import codecs
import collections
from xml import dom
import html5lib
from html5lib import treebuilders, inputstream
from lxml import etree
from lxml.etree import ParseError
from Utils import getMimeFromExt, escapeToNamedASCII, basepath, isPathInsideBase, relativeURL, assetName
import HTMLSerializer
import warnings
import hashlib

class SourceTree(object):
  """Class that manages structure of test repository source.
     Temporarily hard-coded path and filename rules, this should be configurable.
  """

  def __init__(self, repository = None):
    self.mTestExtensions = ['.xht', '.html', '.xhtml', '.htm', '.xml', '.svg']
    self.mReferenceExtensions = ['.xht', '.html', '.xhtml', '.htm', '.xml', '.png', '.svg']
    self.mRepository = repository

  def _splitDirs(self, dir):
    if ('' == dir):
      pathList = []
    elif ('/' in dir):
      pathList = dir.split('/')
    else:
      pathList = dir.split(os.path.sep)
    return pathList

  def _splitPath(self, filePath):
    """split a path into a list of directory names and the file name
       paths may come form the os or mercurial, which always uses '/' as the
       directory separator
    """
    dir, fileName = os.path.split(filePath.lower())
    return (self._splitDirs(dir), fileName)

  def isTracked(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return (not self._isIgnored(pathList, fileName))

  def _isApprovedPath(self, pathList):
    return ((1 < len(pathList)) and ('approved' == pathList[0]) and (('support' == pathList[1]) or ('src' in pathList)))

  def isApprovedPath(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return (not self._isIgnored(pathList, fileName)) and self._isApprovedPath(pathList)

  def _isIgnoredPath(self, pathList):
      return (('.hg' in pathList) or ('.git' in pathList) or
              ('.svn' in pathList) or ('cvs' in pathList) or
              ('incoming' in pathList) or ('work-in-progress' in pathList) or
              ('data' in pathList) or ('archive' in pathList) or
              ('reports' in pathList) or ('tools' == pathList[0]) or
              ('test-plan' in pathList) or ('test-plans' in pathList))

  def _isIgnored(self, pathList, fileName):
    if (pathList):  # ignore files in root
      return (self._isIgnoredPath(pathList) or
              fileName.startswith('.directory') or ('lock' == fileName) or
              ('.ds_store' == fileName) or
              fileName.startswith('.hg') or fileName.startswith('.git') or
              ('sections.dat' == fileName) or ('get-spec-sections.pl' == fileName))
    return True

  def isIgnored(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return self._isIgnored(pathList, fileName)

  def isIgnoredDir(self, dir):
    pathList = self._splitDirs(dir)
    return self._isIgnoredPath(pathList)

  def _isToolPath(self, pathList):
    return ('tools' in pathList)

  def _isTool(self, pathList, fileName):
    return self._isToolPath(pathList)

  def isTool(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return (not self._isIgnored(pathList, fileName)) and self._isTool(pathList, fileName)

  def _isSupportPath(self, pathList):
    return ('support' in pathList)

  def _isSupport(self, pathList, fileName):
    return (self._isSupportPath(pathList) or
            ((not self._isTool(pathList, fileName)) and
             (not self._isReference(pathList, fileName)) and
             (not self._isTestCase(pathList, fileName))))

  def isSupport(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return (not self._isIgnored(pathList, fileName)) and self._isSupport(pathList, fileName)

  def _isReferencePath(self, pathList):
    return (('reftest' in pathList) or ('reference' in pathList))

  def _isReference(self, pathList, fileName):
    if ((not self._isSupportPath(pathList)) and (not self._isToolPath(pathList))):
      baseName, fileExt = os.path.splitext(fileName)[:2]
      if (bool(re.search('(^ref-|^notref-).+', baseName)) or
          bool(re.search('.+(-ref[0-9]*$|-notref[0-9]*$)', baseName)) or
          ('-ref-' in baseName) or ('-notref-' in baseName)):
        return (fileExt in self.mReferenceExtensions)
      if (self._isReferencePath(pathList)):
        return (fileExt in self.mReferenceExtensions)
    return False

  def isReference(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return (not self._isIgnored(pathList, fileName)) and self._isReference(pathList, fileName)

  def isReferenceAnywhere(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return self._isReference(pathList, fileName)

  def _isTestCase(self, pathList, fileName):
    if ((not self._isToolPath(pathList)) and (not self._isSupportPath(pathList)) and (not self._isReference(pathList, fileName))):
      fileExt = os.path.splitext(fileName)[1]
      return (fileExt in self.mTestExtensions)
    return False

  def isTestCase(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    return (not self._isIgnored(pathList, fileName)) and self._isTestCase(pathList, fileName)

  def getAssetName(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    if (self._isReference(pathList, fileName) or self._isTestCase(pathList, fileName)):
      return assetName(fileName)
    return fileName.lower() # support files keep full name

  def getAssetType(self, filePath):
    pathList, fileName = self._splitPath(filePath)
    if (self._isReference(pathList, fileName)):
      return intern('reference')
    if (self._isTestCase(pathList, fileName)):
      return intern('testcase')
    if (self._isTool(pathList, fileName)):
      return intern('tool')
    return intern('support')


class SourceCache:
  """Cache for FileSource objects. Supports one FileSource object
     per sourcepath.
  """
  def __init__(self, sourceTree):
    self.__cache = {}
    self.sourceTree = sourceTree

  def generateSource(self, sourcepath, relpath, data = None):
    """Return a FileSource or derivative based on the extensionMap.

       Uses a cache to avoid creating more than one of the same object:
       does not support creating two FileSources with the same sourcepath;
       asserts if this is tried. (.htaccess files are not cached.)

       Cache is bypassed if loading form a change context
    """
    if ((None == data) and self.__cache.has_key(sourcepath)):
      source = self.__cache[sourcepath]
      assert relpath == source.relpath
      return source

    if basename(sourcepath) == '.htaccess':
      return ConfigSource(self.sourceTree, sourcepath, relpath, data)
    mime = getMimeFromExt(sourcepath)
    if (mime == 'application/xhtml+xml'):
      source = XHTMLSource(self.sourceTree, sourcepath, relpath, data)
    elif (mime == 'text/html'):
      source = HTMLSource(self.sourceTree, sourcepath, relpath, data)
    elif (mime == 'image/svg+xml'):
      source = SVGSource(self.sourceTree, sourcepath, relpath, data)
    elif (mime == 'application/xml'):
      source = XMLSource(self.sourceTree, sourcepath, relpath, data)
    else:
      source = FileSource(self.sourceTree, sourcepath, relpath, mime, data)
    if (None == data):
      self.__cache[sourcepath] = source
    return source

class SourceSet:
  """Set of FileSource objects. No two FileSources of the same type in the set may
     have the same name (except .htaccess files, which are merged).
  """
  def __init__(self, sourceCache):
    self.sourceCache = sourceCache
    self.pathMap = {} # type/name -> source

  def __len__(self):
    return len(self.pathMap)

  def _keyOf(self, source):
    return source.type() + '/' + source.keyName()

  def __contains__(self, source):
    return self._keyOf(source) in self.pathMap


  def iter(self):
    """Iterate over FileSource objects in SourceSet.
    """
    return self.pathMap.itervalues()

  def addSource(self, source, ui):
    """Add FileSource `source`. Throws exception if we already have
       a FileSource with the same path relpath but different contents.
       (ConfigSources are exempt from this requirement.)
    """
    cachedSource = self.pathMap.get(self._keyOf(source))
    if not cachedSource:
      self.pathMap[self._keyOf(source)] = source
    else:
      if source != cachedSource:
        if isinstance(source, ConfigSource):
          cachedSource.append(source)
        else:
          ui.warn("File merge mismatch %s vs %s for %s\n" % \
                (cachedSource.sourcepath, source.sourcepath, source.name()))

  def add(self, sourcepath, relpath, ui):
    """Generate and add FileSource from sourceCache. Return the resulting
       FileSource.

       Throws exception if we already have a FileSource with the same path
       relpath but different contents.
    """
    source = self.sourceCache.generateSource(sourcepath, relpath)
    self.addSource(source, ui)
    return source

  @staticmethod
  def combine(a, b, ui):
    """Merges a and b, and returns whichever one contains the merger (which
       one is chosen based on merge efficiency). Can accept None as an argument.
    """
    if not (a and b):
      return a or b
    if len(a) < len(b):
      return b.merge(a, ui)
    return a.merge(b, ui)

  def merge(self, other, ui):
    """Merge sourceSet's contents into this SourceSet.

       Throws a RuntimeError if there's a sourceCache mismatch.
       Throws an Exception if two files with the same relpath mismatch.
       Returns merge result (i.e. self)
    """
    if self.sourceCache is not other.sourceCache:
      raise RuntimeError

    for source in other.pathMap.itervalues():
      self.addSource(source, ui)
    return self

  def adjustContentPaths(self, format):
    for source in self.pathMap.itervalues():
      source.adjustContentPaths(format)

  def write(self, format):
    """Write files out through OutputFormat `format`.
    """
    for source in self.pathMap.itervalues():
      format.write(source)


class StringReader(object):
  """Wrapper around a string to give it a file-like api
  """
  def __init__(self, string):
    self.mString = string
    self.mIndex = 0

  def read(self, maxSize = None):
    if (self.mIndex < len(self.mString)):
      if (maxSize and (0 < maxSize)):
        slice = self.mString[self.mIndex:self.mIndex + maxSize]
        self.mIndex += len(slice)
        return slice
      else:
        self.mIndex = len(self.mString)
        return self.mString
    return ''


class NamedDict(object):
    def get(self, key):
        if (key in self):
            return self[key]
        return None

    def __eq__(self, other):
        for key in self.__slots__:
            if (self[key] != other[key]):
                return False
        return True

    def __ne__(self, other):
        for key in self.__slots__:
            if (self[key] != other[key]):
                return True
        return False

    def __len__(self):
        return len(self.__slots__)

    def __iter__(self):
        return iter(self.__slots__)

    def __contains__(self, key):
        return (key in self.__slots__)

    def copy(self):
        clone = self.__class__()
        for key in self.__slots__:
            clone[key] = self[key]
        return clone

    def keys(self):
        return self.__slots__

    def has_key(self, key):
        return (key in self)

    def items(self):
        return [(key, self[key]) for key in self.__slots__]

    def iteritems(self):
        return iter(self.items())

    def iterkeys(self):
        return self.__iter__()

    def itervalues(self):
        return iter(self.items())

    def __str__(self):
        return '{ ' + ', '.join([key + ': ' + str(self[key]) for key in self.__slots__]) + ' }'


class Metadata(NamedDict):
    __slots__ = ('name', 'title', 'asserts', 'credits', 'reviewers', 'flags', 'links', 'references', 'revision', 'selftest', 'scripttest')

    def __init__(self, name = None, title = None, asserts = [], credits = [], reviewers = [], flags = [], links = [],
                 references = [], revision = None, selftest = True, scripttest = False):
        self.name = name
        self.title = title
        self.asserts = asserts
        self.credits = credits
        self.reviewers = reviewers
        self.flags = flags
        self.links = links
        self.references = references
        self.revision = revision
        self.selftest = selftest
        self.scripttest = scripttest

    def __getitem__(self, key):
        if ('name' == key):
            return self.name
        if ('title' == key):
            return self.title
        if ('asserts' == key):
            return self.asserts
        if ('credits' == key):
            return self.credits
        if ('reviewers' == key):
            return self.reviewers
        if ('flags' == key):
            return self.flags
        if ('links' == key):
            return self.links
        if ('references' == key):
            return self.references
        if ('revision' == key):
            return self.revision
        if ('selftest' == key):
            return self.selftest
        if ('scripttest' == key):
            return self.scripttest
        return None

    def __setitem__(self, key, value):
        if ('name' == key):
            self.name = value
        elif ('title' == key):
            self.title = value
        elif ('asserts' == key):
            self.asserts = value
        elif ('credits' == key):
            self.credits = value
        elif ('reviewers' == key):
            self.reviewers = value
        elif ('flags' == key):
            self.flags = value
        elif ('links' == key):
            self.links = value
        elif ('references' == key):
            self.references = value
        elif ('revision' == key):
            self.revision = value
        elif ('selftest' == key):
            self.selftest = value
        elif ('scripttest' == key):
            self.scripttest = value
        else:
            raise KeyError()


class ReferenceData(NamedDict):
    __slots__ = ('name', 'type', 'relpath', 'repopath')

    def __init__(self, name = None, type = None, relpath = None, repopath = None):
        self.name = name
        self.type = type
        self.relpath = relpath
        self.repopath = repopath

    def __getitem__(self, key):
        if ('name' == key):
            return self.name
        if ('type' == key):
            return self.type
        if ('relpath' == key):
            return self.relpath
        if ('repopath' == key):
            return self.repopath
        return None

    def __setitem__(self, key, value):
        if ('name' == key):
            self.name = value
        elif ('type' == key):
            self.type = value
        elif ('relpath' == key):
            self.relpath = value
        elif ('repopath' == key):
            self.repopath = value
        else:
            raise KeyError()

UserData = collections.namedtuple('UserData', ('name', 'link'))

class LineString(str):
    def __new__(cls, value, line):
        self = str.__new__(cls, value)
        self.line = line
        return self

    def lineValue(self):
        return 'Line ' + str(self.line) + ': ' + str.__str__(self) if (self.line) else str.__str__(self)


class FileSource:
  """Object representing a file. Two FileSources are equal if they represent
     the same file contents. It is recommended to use a SourceCache to generate
     FileSources.
  """

  def __init__(self, sourceTree, sourcepath, relpath, mimetype = None, data = None):
    """Init FileSource from source path. Give it relative path relpath.

       `mimetype` should be the canonical MIME type for the file, if known.
        If `mimetype` is None, guess type from file extension, defaulting to
        the None key's value in extensionMap.

       `data` if provided, is a the contents of the file. Otherwise the file is read
        from disk.
    """
    self.sourceTree = sourceTree
    self.sourcepath = sourcepath
    self.relpath    = relpath
    self.mimetype   = mimetype or getMimeFromExt(sourcepath)
    self._data      = data
    self.errors     = None
    self.encoding   = 'utf-8'
    self.refs       = {}
    self.scripts    = {}
    self.metadata   = None
    self.metaSource = None

  def __eq__(self, other):
    if not isinstance(other, FileSource):
      return False
    return self.sourcepath == other.sourcepath or \
           filecmp.cmp(self.sourcepath, other.sourcepath)

  def __ne__(self, other):
    return not self == other

  def __cmp__(self, other):
    return cmp(self.name(), other.name())

  def name(self):
    return self.sourceTree.getAssetName(self.sourcepath)

  def keyName(self):
    if ('support' == self.type()):
      return os.path.relpath(self.relpath, 'support')
    return self.name()

  def type(self):
    return self.sourceTree.getAssetType(self.sourcepath)

  def relativeURL(self, other):
    return relativeURL(self.relpath, other.relpath)

  def data(self):
    """Return file contents as a byte string."""
    if (self._data is None):
      self._data = open(self.sourcepath, 'r').read()
    if (self._data.startswith(codecs.BOM_UTF8)):
      self.encoding = 'utf-8-sig' # XXX look for other unicode BOMs
    return self._data

  def unicode(self):
    try:
      return self.data().decode(self.encoding)
    except UnicodeDecodeError, e:
      return None

  def parse(self):
    """Parses and validates FileSource data from sourcepath."""
    self.loadMetadata()

  def validate(self):
    """Ensure data is loaded from sourcepath."""
    self.parse()

  def adjustContentPaths(self, format):
    """Adjust any paths in file content for output format
       XXX need to account for group paths"""
    if (self.refs):
      seenRefs = {}
      seenRefs[self.sourcepath] = '=='
      def adjustReferences(source):
        newRefs = {}
        for refName in source.refs:
          refType, refPath, refNode, refSource = source.refs[refName]
          if refSource:
            refPath = relativeURL(format.dest(self.relpath), format.dest(refSource.relpath))
            if (refSource.sourcepath not in seenRefs):
              seenRefs[refSource.sourcepath] = refType
              adjustReferences(refSource)
          else:
            refPath = relativeURL(format.dest(self.relpath), format.dest(refPath))
          if (refPath != refNode.get('href')):
            refNode.set('href', refPath)
          newRefs[refName] = (refType, refPath, refNode, refSource) # update path in metadata
        source.refs = newRefs
      adjustReferences(self)

    if (self.scripts):   # force testharness.js scripts to absolute path
      for src in self.scripts:
        if (src.endswith('/resources/testharness.js')):   # accept relative paths to testharness.js
            scriptNode = self.scripts[src]
            scriptNode.set('src', '/resources/testharness.js')
        elif (src.endswith('/resources/testharnessreport.js')):
            scriptNode = self.scripts[src]
            scriptNode.set('src', '/resources/testharnessreport.js')


  def write(self, format):
    """Writes FileSource.data() out to `self.relpath` through Format `format`."""
    data = self.data()
    f = open(format.dest(self.relpath), 'w')
    f.write(data)
    if (self.metaSource):
      self.metaSource.write(format) # XXX need to get output path from format, but not let it choose actual format

  def compact(self):
    """Clears all cached data, preserves computed data."""
    pass

  def revision(self):
    """Returns hash of the contents of this file and any related file, references, support files, etc.
       XXX also needs to account for .meta file
    """
    sha = hashlib.sha1()
    sha.update(self.data())
    seenRefs = set(self.sourcepath)
    def hashReference(source):
        for refName in source.refs:
            refSource = source.refs[refName][3]
            if (refSource and (refSource.sourcepath not in seenRefs)):
                sha.update(refSource.data())
                seenRefs.add(refSource.sourcepath)
                hashReference(refSource)
    hashReference(self)
    return sha.hexdigest()

  def loadMetadata(self):
    """Look for .meta file and load any metadata from it if present
    """
    pass

  def augmentMetadata(self, next=None, prev=None, reference=None, notReference=None):
    if (self.metaSource):
      return self.metaSource.augmentMetadata(next, prev, reference, notReference)
    return None

  # See http://wiki.csswg.org/test/css2.1/format for more info on metadata
  def getMetadata(self, asUnicode = False):
    """Return dictionary of test metadata. Stores list of errors
       in self.errors if there are parse or metadata errors.
       Data fields include:
         - asserts [list of strings]
         - credits [list of (name string, url string) tuples]
         - reviewers [ list of (name string, url string) tuples]
         - flags   [list of token strings]
         - links   [list of url strings]
         - name    [string]
         - title   [string]
         - references [list of ReferenceData per reference; None if not reftest]
         - revision   [revision id of last commit]
         - selftest [bool]
         - scripttest [bool]
       Strings are given in ascii unless asUnicode==True.
    """

    self.validate()

    def encode(str):
        return str if (hasattr(str, 'line')) else intern(str.encode('utf-8'))

    def escape(str, andIntern = True):
      return str.encode('utf-8') if asUnicode else intern(escapeToNamedASCII(str)) if andIntern else escapeToNamedASCII(str)

    def listReferences(source, seen):
        refGroups = []
        for refType, refRelPath, refNode, refSource in source.refs.values():
            if ('==' == refType):
                if (refSource):
                    refSourcePath = refSource.sourcepath
                else:
                    refSourcePath = os.path.normpath(join(basepath(source.sourcepath), refRelPath))
                if (refSourcePath in seen):
                    continue
                seen.add(refSourcePath)
                if (refSource):
                    sourceData = ReferenceData(name = self.sourceTree.getAssetName(refSourcePath), type = refType,
                                               relpath = refRelPath, repopath = refSourcePath)
                    if (refSource.refs):
                        subRefLists = listReferences(refSource, seen.copy())
                        if (subRefLists):
                            for subRefList in subRefLists:
                                refGroups.append([sourceData] + subRefList)
                        else:
                            refGroups.append([sourceData])
                    else:
                        refGroups.append([sourceData])
                else:
                    sourceData = ReferenceData(name = self.sourceTree.getAssetName(refSourcePath), type = refType,
                                               relpath = relativeURL(self.sourcepath, refSourcePath),
                                               repopath = refSourcePath)
                    refGroups.append([sourceData])
        notRefs = {}
        for refType, refRelPath, refNode, refSource in source.refs.values():
            if ('!=' == refType):
                if (refSource):
                    refSourcePath = refSource.sourcepath
                else:
                    refSourcePath = os.path.normpath(join(basepath(source.sourcepath), refRelPath))
                if (refSourcePath in seen):
                    continue
                seen.add(refSourcePath)
                if (refSource):
                    sourceData = ReferenceData(name = self.sourceTree.getAssetName(refSourcePath), type = refType,
                                               relpath = refRelPath, repopath = refSourcePath)
                    notRefs[sourceData.name] = sourceData
                    if (refSource.refs):
                        for subRefList in listReferences(refSource, seen):
                            for subRefData in subRefList:
                                notRefs[subRefData.name] = subRefData
                else:
                    sourceData = ReferenceData(name = self.sourceTree.getAssetName(refSourcePath), type = refType,
                                               relpath = relativeURL(self.sourcepath, refSourcePath),
                                               repopath = refSourcePath)
                    notRefs[sourceData.name] = sourceData
        if (notRefs):
            for refData in notRefs.values():
                refData.type = '!='
            if (refGroups):
                for refGroup in refGroups:
                    for notRef in notRefs.values():
                        for ref in refGroup:
                            if (ref.name == notRef.name):
                                break
                        else:
                            refGroup.append(notRef)
            else:
                refGroups.append(notRefs.values())
        return refGroups

    references = listReferences(self, set([self.sourcepath])) if (self.refs) else None

    if (self.metadata):
      data = Metadata(
              name       = encode(self.name()),
              title      = escape(self.metadata['title'], False),
              asserts    = [escape(assertion, False) for assertion in self.metadata['asserts']],
              credits    = [UserData(escape(name), encode(link)) for name, link in self.metadata['credits']],
              reviewers  = [UserData(escape(name), encode(link)) for name, link in self.metadata['reviewers']],
              flags      = [encode(flag) for flag in self.metadata['flags']],
              links      = [encode(link) for link in self.metadata['links']],
              references = references,
              revision   = self.revision(),
              selftest   = self.isSelftest(),
              scripttest = self.isScripttest()
             )
      return data
    return None

  def addReference(self, referenceSource, match = None):
    """Add reference source."""
    self.validate()
    refName = referenceSource.name()
    refPath = self.relativeURL(referenceSource)
    if refName not in self.refs:
      node = None
      if match == '==':
        node = self.augmentMetadata(reference=referenceSource).reference
      elif match == '!=':
        node = self.augmentMetadata(notReference=referenceSource).notReference
      self.refs[refName] = (match, refPath, node, referenceSource)
    else:
      node = self.refs[refName][2]
      node.set('href', refPath)
      if (match):
        node.set('rel', 'mismatch' if ('!=' == match) else 'match')
      else:
        match = self.refs[refName][0]
      self.refs[refName] = (match, refPath, node, referenceSource)

  def getReferencePaths(self):
    """Get list of paths to references as tuple(path, relPath, refType)."""
    self.validate()
    return [(os.path.join(os.path.dirname(self.sourcepath), ref[1]),
             os.path.join(os.path.dirname(self.relpath), ref[1]),
             ref[0])
            for ref in self.refs.values()]

  def isTest(self):
    self.validate()
    return bool(self.metadata) and bool(self.metadata.get('links'))

  def isReftest(self):
    return self.isTest() and bool(self.refs)

  def isSelftest(self):
    return self.isTest() and (not bool(self.refs))

  def isScripttest(self):
    if (self.isTest() and self.scripts):
        for src in self.scripts:
            if (src.endswith('/resources/testharness.js')):   # accept relative paths to testharness.js
                return True
    return False

  def hasFlag(self, flag):
    data = self.getMetadata()
    if data:
      return flag in data['flags']
    return False



class ConfigSource(FileSource):
  """Object representing a text-based configuration file.
     Capable of merging multiple config-file contents.
  """

  def __init__(self, sourceTree, sourcepath, relpath, mimetype = None, data = None):
    """Init ConfigSource from source path. Give it relative path relpath.
    """
    FileSource.__init__(self, sourceTree, sourcepath, relpath, mimetype, data)
    self.sourcepath = [sourcepath]

  def __eq__(self, other):
    if not isinstance(other, ConfigSource):
      return False
    if self is other or self.sourcepath == other.sourcepath:
      return True
    if len(self.sourcepath) != len(other.sourcepath):
      return False
    for this, that in zip(self.sourcepath, other.sourcepath):
      if not filecmp.cmp(this, that):
        return False
    return True

  def __ne__(self, other):
    return not self == other

  def name(self):
    return '.htaccess'

  def type(self):
    return intern('support')

  def data(self):
    """Merge contents of all config files represented by this source."""
    data = ''
    for src in self.sourcepath:
      data += open(src).read()
      data += '\n'
    return data

  def getMetadata(self, asUnicode = False):
    return None

  def append(self, other):
    """Appends contents of ConfigSource `other` to this source.
       Asserts if self.relpath != other.relpath.
    """
    assert isinstance(other, ConfigSource)
    assert self != other and self.relpath == other.relpath
    self.sourcepath.extend(other.sourcepath)

class ReftestFilepathError(Exception):
  pass

class ReftestManifest(ConfigSource):
  """Object representing a reftest manifest file.
     Iterating the ReftestManifest returns (testpath, refpath) tuples
     with paths relative to the manifest.
  """
  def __init__(self, sourceTree, sourcepath, relpath, data = None):
    """Init ReftestManifest from source path. Give it relative path `relpath`
       and load its .htaccess file.
    """
    ConfigSource.__init__(self, sourceTree, sourcepath, relpath, mimetype = 'config/reftest', data = data)

  def basepath(self):
    """Returns the base relpath of this reftest manifest path, i.e.
       the parent of the manifest file.
    """
    return basepath(self.relpath)

  baseRE = re.compile(r'^#\s*relstrip\s+(\S+)\s*')
  stripRE = re.compile(r'#.*')
  parseRE = re.compile(r'^\s*([=!]=)\s*(\S+)\s+(\S+)')

  def __iter__(self):
    """Parse the reftest manifest files represented by this ReftestManifest
       and return path information about each reftest pair as
         ((test-sourcepath, ref-sourcepath), (test-relpath, ref-relpath), reftype)
       Raises a ReftestFilepathError if any sources file do not exist or
       if any relpaths point higher than the relpath root.
    """
    striplist = []
    for src in self.sourcepath:
      relbase = basepath(self.relpath)
      srcbase = basepath(src)
      for line in open(src):
        strip = self.baseRE.search(line)
        if strip:
          striplist.append(strip.group(1))
        line = self.stripRE.sub('', line)
        m = self.parseRE.search(line)
        if m:
          record = ((join(srcbase, m.group(2)), join(srcbase, m.group(3))), \
                    (join(relbase, m.group(2)), join(relbase, m.group(3))), \
                    m.group(1))
#          for strip in striplist:
            # strip relrecord
          if not exists(record[0][0]):
            raise ReftestFilepathError("Manifest Error in %s: "
                                       "Reftest test file %s does not exist." \
                                        % (src, record[0][0]))
          elif not exists(record[0][1]):
            raise ReftestFilepathError("Manifest Error in %s: "
                                       "Reftest reference file %s does not exist." \
                                       % (src, record[0][1]))
          elif not isPathInsideBase(record[1][0]):
            raise ReftestFilepathError("Manifest Error in %s: "
                                       "Reftest test replath %s not within relpath root." \
                                       % (src, record[1][0]))
          elif not isPathInsideBase(record[1][1]):
            raise ReftestFilepathError("Manifest Error in %s: "
                                       "Reftest test replath %s not within relpath root." \
                                       % (src, record[1][1]))
          yield record

import Utils # set up XML catalog
xhtmlns = '{http://www.w3.org/1999/xhtml}'
svgns = '{http://www.w3.org/2000/svg}'
xmlns = '{http://www.w3.org/XML/1998/namespace}'
xlinkns = '{http://www.w3.org/1999/xlink}'

class XMLSource(FileSource):
  """FileSource object with support reading XML trees."""

  NodeTuple = collections.namedtuple('NodeTuple', ['next', 'prev', 'reference', 'notReference'])

  # Public Data
  syntaxErrorDoc = \
  u"""
  <!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">
  <html xmlns="http://www.w3.org/1999/xhtml">
    <head><title>Syntax Error</title></head>
    <body>
      <p>The XML file <![CDATA[%s]]> contains a syntax error and could not be parsed.
      Please correct it and try again.</p>
      <p>The parser's error report was:</p>
      <pre><![CDATA[%s]]></pre>
    </body>
  </html>
  """

  # Private Data and Methods
  __parser = etree.XMLParser(no_network=True,
  # perf nightmare           dtd_validation=True,
                             remove_comments=False,
                             strip_cdata=False,
                             resolve_entities=False)

  # Public Methods

  def __init__(self, sourceTree, sourcepath, relpath, data = None):
    """Initialize XMLSource by loading from XML file `sourcepath`.
      Parse errors are reported in `self.errors`,
      and the source is replaced with an XHTML error message.
    """
    FileSource.__init__(self, sourceTree, sourcepath, relpath, data = data)
    self.tree = None
    self.injectedTags = {}

  def cacheAsParseError(self, filename, e):
      """Replace document with an error message."""
      errorDoc = self.syntaxErrorDoc % (filename, e)
      from StringIO import StringIO
      self.tree = etree.parse(StringIO(errorDoc), parser=self.__parser)

  def parse(self):
    """Parse file and store any parse errors in self.errors"""
    self.errors = None
    try:
      data = self.data()
      if (data):
        self.tree = etree.parse(StringReader(data), parser=self.__parser)
        self.encoding = self.tree.docinfo.encoding or 'utf-8'
        self.injectedTags = {}
      else:
        self.tree = None
        self.errors = ['Empty source file']
        self.encoding = 'utf-8'

      FileSource.loadMetadata(self)
      if ((not self.metadata) and self.tree and (not self.errors)):
        self.extractMetadata(self.tree)
    except etree.ParseError as e:
      print "PARSE ERROR: " + self.sourcepath
      self.cacheAsParseError(self.sourcepath, e)
      e.W3CTestLibErrorLocation = self.sourcepath
      self.errors = [str(e)]
      self.encoding = 'utf-8'

  def validate(self):
    """Parse file if not parsed, and store any parse errors in self.errors"""
    if self.tree is None:
      self.parse()

  def getMeatdataContainer(self):
    return self.tree.getroot().find(xhtmlns+'head')

  def injectMetadataLink(self, rel, href, tagCode = None):
    """Inject (prepend) <link> with data given inside metadata container.
       Injected element is tagged with `tagCode`, which can be
       used to clear it with clearInjectedTags later.
    """
    self.validate()
    container = self.getMeatdataContainer()
    if (container):
      node = etree.Element(xhtmlns+'link', {'rel': rel, 'href': href})
      node.tail = container.text
      container.insert(0, node)
      self.injectedTags[node] = tagCode or True
      return node
    return None

  def clearInjectedTags(self, tagCode = None):
    """Clears all injected elements from the tree, or clears injected
       elements tagged with `tagCode` if `tagCode` is given.
    """
    if not self.injectedTags or not self.tree: return
    for node in self.injectedTags:
      node.getparent().remove(node)
      del self.injectedTags[node]

  def serializeXML(self):
    self.validate()
    return etree.tounicode(self.tree)

  def data(self):
    if ((not self.tree) or (self.metaSource)):
      return FileSource.data(self)
    return self.serializeXML().encode(self.encoding, 'xmlcharrefreplace')

  def unicode(self):
    if ((not self.tree) or (self.metaSource)):
      return FileSource.unicode(self)
    return self.serializeXML()

  def write(self, format, output=None):
    """Write Source through OutputFormat `format`.
       Write contents as string `output` instead if specified.
    """
    if not output:
      output = self.unicode()

    # write
    f = open(format.dest(self.relpath), 'w')
    f.write(output.encode(self.encoding, 'xmlcharrefreplace'))
    f.close()

  def compact(self):
    self.tree = None

  def getMetadataElements(self, tree):
    container = self.getMeatdataContainer()
    if (None != container):
      return [node for node in container]
    return None

  def extractMetadata(self, tree):
    """Extract metadata from tree."""
    links = []; credits = []; reviewers = []; flags = []; asserts = []; title = ''

    def tokenMatch(token, string):
        return bool(re.search('(^|\s+)%s($|\s+)' % token, string)) if (string) else False

    errors = []
    readFlags = False
    metaElements = self.getMetadataElements(tree)
    if (not metaElements):
        errors.append("Missing <head> element")
    else:
        # Scan and cache metadata
        for node in metaElements:
            if (node.tag == xhtmlns+'link'):
                # help links
                if tokenMatch('help', node.get('rel')):
                    link = node.get('href').strip() if node.get('href') else None
                    if (not link):
                        errors.append(LineString("Help link missing href value.", node.sourceline))
                    elif (not (link.startswith('http://') or link.startswith('https://'))):
                        errors.append(LineString("Help link " + link.encode('utf-8') + " must be absolute URL.", node.sourceline))
                    elif (link in links):
                        errors.append(LineString("Duplicate help link " + link.encode('utf-8') + ".", node.sourceline))
                    else:
                        links.append(LineString(link, node.sourceline))
                # == references
                elif tokenMatch('match', node.get('rel')) or tokenMatch('reference', node.get('rel')):
                    refPath = node.get('href').strip() if node.get('href') else None
                    if (not refPath):
                        errors.append(LineString("Reference link missing href value.", node.sourceline))
                    else:
                        refName = self.sourceTree.getAssetName(join(self.sourcepath, refPath))
                        if (refName in self.refs):
                            errors.append(LineString("Reference " + refName.encode('utf-8') + " already specified.", node.sourceline))
                        else:
                            self.refs[refName] = ('==', refPath, node, None)
                # != references
                elif tokenMatch('mismatch', node.get('rel')) or tokenMatch('not-reference', node.get('rel')):
                    refPath = node.get('href').strip() if node.get('href') else None
                    if (not refPath):
                        errors.append(LineString("Reference link missing href value.", node.sourceline))
                    else:
                        refName = self.sourceTree.getAssetName(join(self.sourcepath, refPath))
                        if (refName in self.refs):
                            errors.append(LineString("Reference " + refName.encode('utf-8') + " already specified.", node.sourceline))
                        else:
                            self.refs[refName] = ('!=', refPath, node, None)
                else: # may have both author and reviewer in the same link
                    # credits
                    if tokenMatch('author', node.get('rel')):
                        name = node.get('title')
                        name = name.strip() if name else name
                        if (not name):
                            errors.append(LineString("Author link missing name (title attribute).", node.sourceline))
                        else:
                            link = node.get('href').strip() if node.get('href') else None
                            if (not link):
                                errors.append(LineString("Author link for \"" + name.encode('utf-8') + "\" missing contact URL (http or mailto).", node.sourceline))
                            else:
                                credits.append((name, link))
                    # reviewers
                    if tokenMatch('reviewer', node.get('rel')):
                        name = node.get('title')
                        name = name.strip() if name else name
                        if (not name):
                            errors.append(LineString("Reviewer link missing name (title attribute).", node.sourceline))
                        else:
                            link = node.get('href').strip() if node.get('href') else None
                            if (not link):
                                errors.append(LineString("Reviewer link for \"" + name.encode('utf-8') + "\" missing contact URL (http or mailto).", node.sourceline))
                            else:
                                reviewers.append((name, link))
            elif (node.tag == xhtmlns+'meta'):
                metatype = node.get('name')
                metatype = metatype.strip() if metatype else metatype
                # requirement flags
                if ('flags' == metatype):
                    if (readFlags):
                        errors.append(LineString("Flags must only be specified once.", node.sourceline))
                    else:
                        readFlags = True
                        if (None == node.get('content')):
                            errors.append(LineString("Flags meta missing content attribute.", node.sourceline))
                        else:
                            for flag in sorted(node.get('content').split()):
                                flags.append(flag)
                # test assertions
                elif ('assert' == metatype):
                    if (None == node.get('content')):
                        errors.append(LineString("Assert meta missing content attribute.", node.sourceline))
                    else:
                        asserts.append(node.get('content').strip().replace('\t', ' '))
            # title
            elif (node.tag == xhtmlns+'title'):
                title = node.text.strip() if node.text else ''
                match = re.match('(?:[^:]*)[tT]est(?:[^:]*):(.*)', title, re.DOTALL)
                if (match):
                    title = match.group(1)
                title = title.strip()
            # script
            elif (node.tag == xhtmlns+'script'):
                src = node.get('src').strip() if node.get('src') else None
                if (src):
                    self.scripts[src] = node

    if (asserts or credits or reviewers or flags or links or title):
        self.metadata = {'asserts'   : asserts,
                         'credits'   : credits,
                         'reviewers' : reviewers,
                         'flags'     : flags,
                         'links'     : links,
                         'title'     : title
                        }

    if (errors):
        if (self.errors):
            self.errors += errors
        else:
            self.errors = errors


  def augmentMetadata(self, next=None, prev=None, reference=None, notReference=None):
     """Add extra useful metadata to the head. All arguments are optional.
          * Adds next/prev links to  next/prev Sources given
          * Adds reference link to reference Source given
     """
     self.validate()
     if next:
       next = self.injectMetadataLink('next', self.relativeURL(next), 'next')
     if prev:
       prev = self.injectMetadataLink('prev', self.relativeURL(prev), 'prev')
     if reference:
       reference = self.injectMetadataLink('match', self.relativeURL(reference), 'ref')
     if notReference:
       notReference = self.injectMetadataLink('mismatch', self.relativeURL(notReference), 'not-ref')
     return self.NodeTuple(next, prev, reference, notReference)


class XHTMLSource(XMLSource):
  """FileSource object with support for XHTML->HTML conversions."""

  # Public Methods

  def __init__(self, sourceTree, sourcepath, relpath, data = None):
    """Initialize XHTMLSource by loading from XHTML file `sourcepath`.
      Parse errors are stored in `self.errors`,
      and the source is replaced with an XHTML error message.
    """
    XMLSource.__init__(self, sourceTree, sourcepath, relpath, data = data)

  def serializeXHTML(self, doctype = None):
    return self.serializeXML()

  def serializeHTML(self, doctype = None):
    self.validate()
    # Serialize
#    print self.relpath
    serializer = HTMLSerializer.HTMLSerializer()
    output = serializer.serializeHTML(self.tree, doctype)
    return output


class SVGSource(XMLSource):
  """FileSource object with support for extracting metadata from SVG."""

  def __init__(self, sourceTree, sourcepath, relpath, data = None):
    """Initialize SVGSource by loading from SVG file `sourcepath`.
      Parse errors are stored in `self.errors`,
      and the source is replaced with an XHTML error message.
    """
    XMLSource.__init__(self, sourceTree, sourcepath, relpath, data = data)

  def getMeatdataContainer(self):
    groups = self.tree.getroot().findall(svgns+'g')
    for group in groups:
      if ('testmeta' == group.get('id')):
        return group
    return None

  def extractMetadata(self, tree):
    """Extract metadata from tree."""
    links = []; credits = []; reviewers = []; flags = []; asserts = []; title = ''

    def tokenMatch(token, string):
        return bool(re.search('(^|\s+)%s($|\s+)' % token, string)) if (string) else False

    errors = []
    readFlags = False
    metaElements = self.getMetadataElements(tree)
    if (not metaElements):
        errors.append("Missing <g id='testmeta'> element")
    else:
        # Scan and cache metadata
        for node in metaElements:
            if (node.tag == xhtmlns+'link'):
                # help links
                if tokenMatch('help', node.get('rel')):
                    link = node.get('href').strip() if node.get('href') else None
                    if (not link):
                        errors.append(LineString("Help link missing href value.", node.sourceline))
                    elif (not (link.startswith('http://') or link.startswith('https://'))):
                        errors.append(LineString("Help link " + link.encode('utf-8') + " must be absolute URL.", node.sourceline))
                    elif (link in links):
                        errors.append(LineString("Duplicate help link " + link.encode('utf-8') + ".", node.sourceline))
                    else:
                        links.append(LineString(link, node.sourceline))
                # == references
                elif tokenMatch('match', node.get('rel')) or tokenMatch('reference', node.get('rel')):
                    refPath = node.get('href').strip() if node.get('href') else None
                    if (not refPath):
                        errors.append(LineString("Reference link missing href value.", node.sourceline))
                    else:
                        refName = self.sourceTree.getAssetName(join(self.sourcepath, refPath))
                        if (refName in self.refs):
                            errors.append(LineString("Reference " + refName.encode('utf-8') + " already specified.", node.sourceline))
                        else:
                            self.refs[refName] = ('==', refPath, node, None)
                # != references
                elif tokenMatch('mismatch', node.get('rel')) or tokenMatch('not-reference', node.get('rel')):
                    refPath = node.get('href').strip() if node.get('href') else None
                    if (not refPath):
                        errors.append(LineString("Reference link missing href value.", node.sourceline))
                    else:
                        refName = self.sourceTree.getAssetName(join(self.sourcepath, refPath))
                        if (refName in self.refs):
                            errors.append(LineString("Reference " + refName.encode('utf-8') + " already specified.", node.sourceline))
                        else:
                            self.refs[refName] = ('!=', refPath, node, None)
                else: # may have both author and reviewer in the same link
                    # credits
                    if tokenMatch('author', node.get('rel')):
                        name = node.get('title')
                        name = name.strip() if name else name
                        if (not name):
                            errors.append(LineString("Author link missing name (title attribute).", node.sourceline))
                        else:
                            link = node.get('href').strip() if node.get('href') else None
                            if (not link):
                                errors.append(LineString("Author link for \"" + name.encode('utf-8') + "\" missing contact URL (http or mailto).", node.sourceline))
                            else:
                                credits.append((name, link))
                    # reviewers
                    if tokenMatch('reviewer', node.get('rel')):
                        name = node.get('title')
                        name = name.strip() if name else name
                        if (not name):
                            errors.append(LineString("Reviewer link missing name (title attribute).", node.sourceline))
                        else:
                            link = node.get('href').strip() if node.get('href') else None
                            if (not link):
                                errors.append(LineString("Reviewer link for \"" + name.encode('utf-8') + "\" missing contact URL (http or mailto).", node.sourceline))
                            else:
                                reviewers.append((name, link))
            elif (node.tag == svgns+'metadata'):
                metatype = node.get('class')
                metatype = metatype.strip() if metatype else metatype
                # requirement flags
                if ('flags' == metatype):
                    if (readFlags):
                        errors.append(LineString("Flags must only be specified once.", node.sourceline))
                    else:
                        readFlags = True
                        text = node.find(svgns+'text')
                        flagString = text.text if (text) else node.text
                        if (flagString):
                            for flag in sorted(flagString.split()):
                                flags.append(flag)
            elif (node.tag == svgns+'desc'):
                metatype = node.get('class')
                metatype = metatype.strip() if metatype else metatype
                # test assertions
                if ('assert' == metatype):
                    asserts.append(node.text.strip().replace('\t', ' '))
            # test title
            elif node.tag == svgns+'title':
                title = node.text.strip() if node.text else ''
                match = re.match('(?:[^:]*)[tT]est(?:[^:]*):(.*)', title, re.DOTALL)
                if (match):
                    title = match.group(1)
                title = title.strip()
            # script tag (XXX restricted to metadata container?)
            elif (node.tag == svgns+'script'):
                src = node.get('src').strip() if node.get('src') else None
                if (src):
                    self.scripts[src] = node

    if (asserts or credits or reviewers or flags or links or title):
        self.metadata = {'asserts'   : asserts,
                         'credits'   : credits,
                         'reviewers' : reviewers,
                         'flags'     : flags,
                         'links'     : links,
                         'title'     : title
                        }
    if (errors):
        if (self.errors):
            self.errors += errors
        else:
            self.errors = errors



class HTMLSource(XMLSource):
  """FileSource object with support for HTML metadata and HTML->XHTML conversions (untested)."""

  # Private Data and Methods
  __parser = html5lib.HTMLParser(tree = treebuilders.getTreeBuilder('lxml'))

  # Public Methods

  def __init__(self, sourceTree, sourcepath, relpath, data = None):
    """Initialize HTMLSource by loading from HTML file `sourcepath`.
    """
    XMLSource.__init__(self, sourceTree, sourcepath, relpath, data = data)

  def parse(self):
    """Parse file and store any parse errors in self.errors"""
    self.errors = None
    try:
      data = self.data()
      if data:
        with warnings.catch_warnings():
          warnings.simplefilter("ignore")
          htmlStream = html5lib.inputstream.HTMLInputStream(data)
          if ('utf-8-sig' != self.encoding):  # if we found a BOM, respect it
            self.encoding = htmlStream.detectEncoding()[0]
          self.tree = self.__parser.parse(data, encoding = self.encoding)
          self.injectedTags = {}
      else:
        self.tree = None
        self.errors = ['Empty source file']
        self.encoding = 'utf-8'

      FileSource.loadMetadata(self)
      if ((not self.metadata) and self.tree and (not self.errors)):
        self.extractMetadata(self.tree)
    except Exception as e:
      print "PARSE ERROR: " + self.sourcepath
      e.W3CTestLibErrorLocation = self.sourcepath
      self.errors = [str(e)]
      self.encoding = 'utf-8'

  def _injectXLinks(self, element, nodeList):
    injected = False

    xlinkAttrs = ['href', 'type', 'role', 'arcrole', 'title', 'show', 'actuate']
    if (element.get('href') or element.get(xlinkns + 'href')):
      for attr in xlinkAttrs:
        if (element.get(xlinkns + attr)):
          injected = True
        if (element.get(attr)):
          injected = True
          value = element.get(attr)
          del element.attrib[attr]
          element.set(xlinkns + attr, value)
          nodeList.append((element, xlinkns + attr, attr))

    for child in element:
        if (type(child.tag) == type('')): # element node
            qName = etree.QName(child.tag)
            if ('foreignobject' != qName.localname.lower()):
                injected |= self._injectXLinks(child, nodeList)
    return injected


  def _findElements(self, namespace, elementName):
      elements = self.tree.findall('.//{' + namespace + '}' + elementName)
      if (self.tree.getroot().tag == '{' + namespace + '}' + elementName):
          elements.insert(0, self.tree.getroot())
      return elements

  def _injectNamespace(self, elementName, prefix, namespace, doXLinks, nodeList):
    attr = xmlns + prefix if (prefix) else 'xmlns'
    elements = self._findElements(namespace, elementName)
    for element in elements:
      if not element.get(attr):
        element.set(attr, namespace)
        nodeList.append((element, attr, None))
        if (doXLinks):
          if (self._injectXLinks(element, nodeList)):
            element.set(xmlns + 'xlink', 'http://www.w3.org/1999/xlink')
            nodeList.append((element, xmlns + 'xlink', None))

  def injectNamespaces(self):
    nodeList = []
    self._injectNamespace('html', None, 'http://www.w3.org/1999/xhtml', False, nodeList)
    self._injectNamespace('svg', None, 'http://www.w3.org/2000/svg', True, nodeList)
    self._injectNamespace('math', None, 'http://www.w3.org/1998/Math/MathML', True, nodeList)
    return nodeList

  def removeNamespaces(self, nodeList):
      if nodeList:
          for element, attr, oldAttr in nodeList:
              if (oldAttr):
                  value = element.get(attr)
                  del element.attrib[attr]
                  element.set(oldAttr, value)
              else:
                  del element.attrib[attr]

  def serializeXHTML(self, doctype = None):
    self.validate()
    # Serialize
    nodeList = self.injectNamespaces()
#    print self.relpath
    serializer = HTMLSerializer.HTMLSerializer()
    o = serializer.serializeXHTML(self.tree, doctype)

    self.removeNamespaces(nodeList)
    return o

  def serializeHTML(self, doctype = None):
    self.validate()
    # Serialize
#    print self.relpath
    serializer = HTMLSerializer.HTMLSerializer()
    o = serializer.serializeHTML(self.tree, doctype)

    return o

  def data(self):
    if ((not self.tree) or (self.metaSource)):
      return FileSource.data(self)
    return self.serializeHTML().encode(self.encoding, 'xmlcharrefreplace')

  def unicode(self):
    if ((not self.tree) or (self.metaSource)):
      return FileSource.unicode(self)
    return self.serializeHTML()

