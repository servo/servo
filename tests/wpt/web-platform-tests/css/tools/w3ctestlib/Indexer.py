#!/usr/bin/python
# CSS Test Suite Manipulation Library
# Initial code by fantasai, joint copyright 2010 W3C and Microsoft
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

# Define contains vmethod for Template Toolkit
from template.stash import list_op
@list_op("contains")
def list_contains(l, x):
  return x in l

import sys
import re
import os
import codecs
from os.path import join, exists, abspath
from template import Template
import w3ctestlib
from Utils import listfiles, escapeToNamedASCII
from OutputFormats import ExtensionMap
import shutil

class Section:
  def __init__(self, uri, title, numstr):
    self.uri = uri
    self.title = title
    self.numstr = numstr
    self.tests = []
  def __cmp__(self, other):
    return cmp(self.natsortkey(), other.natsortkey())
  def chapterNum(self):
    return self.numstr.partition('.')[0]
  def natsortkey(self):
    chunks = self.numstr.partition('.#')[0].split('.')
    for index in range(len(chunks)):
      if chunks[index].isdigit():
        # wrap in tuple with '0' to explicitly specify numbers come first
        chunks[index] = (0, int(chunks[index]))
      else:
        chunks[index] = (1, chunks[index])
    return (chunks, self.numstr)

class Indexer:

  def __init__(self, suite, sections, suites, flags, splitChapter=False, templatePathList=None,
               extraData=None, overviewTmplNames=None, overviewCopyExts=('.css', 'htaccess')):
    """Initialize indexer with TestSuite `suite` toc data file
       `tocDataPath` and additional template paths in list `templatePathList`.

       The toc data file should be list of tab-separated records, one
       per line, of each spec section's uri, number/letter, and title.
       `splitChapter` selects a single page index if False, chapter 
       indicies if True.
       `extraData` can be a dictionary whose data gets passed to the templates.
       `overviewCopyExts` lists file extensions that should be found
       and copied from the template path into the main build directory.
       The default value is ['.css', 'htaccess'].
       `overviewTemplateNames` lists template names that should be
       processed from the template path into the main build directory.
       The '.tmpl' extension, if any, is stripped from the output filename.
       The default value is ['index.htm.tmpl', 'index.xht.tmpl', 'testinfo.data.tmpl']
    """
    self.suite        = suite
    self.splitChapter = splitChapter
    self.extraData    = extraData
    self.overviewCopyExtPat = re.compile('.*(%s)$' % '|'.join(overviewCopyExts))
    self.overviewTmplNames = overviewTmplNames if overviewTmplNames is not None \
      else ['index.htm.tmpl', 'index.xht.tmpl', 'testinfo.data.tmpl',
            'implementation-report-TEMPLATE.data.tmpl']

    # Initialize template engine
    self.templatePath = [join(w3ctestlib.__path__[0], 'templates')]
    if templatePathList:
      self.templatePath.extend(templatePathList)
    self.templatePath = [abspath(path) for path in self.templatePath]
    self.tt = Template({
       'INCLUDE_PATH': self.templatePath,
       'ENCODING'    : 'utf-8',
       'PRE_CHOMP'   : 1,
       'POST_CHOMP'  : 0,
    })

    # Load toc data
    self.sections = {}
    for uri, numstr, title in sections:
      uri = intern(uri.encode('utf-8'))
      uriKey = intern(self._normalizeScheme(uri))
      numstr = escapeToNamedASCII(numstr)
      title = escapeToNamedASCII(title) if title else None
      self.sections[uriKey] = Section(uri, title, numstr)
    
    self.suites = suites
    self.flags = flags

    # Initialize storage
    self.errors = {}
    self.contributors = {}
    self.alltests = []

  def _normalizeScheme(self, uri):
    if (uri and uri.startswith('http:')):
      return 'https:' + uri[5:]
    return uri

  def indexGroup(self, group):
    for test in group.iterTests():
      data = test.getMetadata()
      if data: # Shallow copy for template output
        data = dict(data)
        data['file'] = '/'.join((group.name, test.relpath)) \
                       if group.name else test.relpath
        if (data['scripttest']):
            data['flags'].append(intern('script'))
        self.alltests.append(data)
        for uri in data['links']:
          uri = self._normalizeScheme(uri)
          uri = uri.replace(self._normalizeScheme(self.suite.draftroot), self._normalizeScheme(self.suite.specroot))
          if self.sections.has_key(uri):
            testlist = self.sections[uri].tests.append(data)
        for credit in data['credits']:
          self.contributors[credit[0]] = credit[1]
      else:
        self.errors[test.sourcepath] = test.errors

  def __writeTemplate(self, template, data, outfile):
    o = self.tt.process(template, data)
    with open(outfile, 'w') as f:
      f.write(o.encode('utf-8'))

  def writeOverview(self, destDir, errorOut=sys.stderr, addTests=[]):
    """Write format-agnostic pages such as test suite overview pages,
       test data files, and error reports.

       Indexed errors are reported to errorOut, which must be either
       an output handle such as sys.stderr, a tuple of
       (template filename string, output filename string)
       or None to suppress error output.

       `addTests` is a list of additional test paths, relative to the
       overview root; it is intended for indexing raw tests
    """

    # Set common values
    data = self.extraData.copy()
    data['suitetitle']   = self.suite.title
    data['suite']        = self.suite.name
    data['specroot']     = self.suite.specroot
    data['draftroot']    = self.suite.draftroot
    data['contributors'] = self.contributors
    data['tests']        = self.alltests
    data['extmap']       = ExtensionMap({'.xht':'', '.html':'', '.htm':'', '.svg':''})
    data['formats']      = self.suite.formats
    data['addtests']     = addTests
    data['suites']       = self.suites
    data['flagInfo']     = self.flags
    data['formatInfo']   = { 'html4': { 'report': True, 'path': 'html4', 'ext': 'htm', 'filter': 'nonHTML'},
                             'html5': { 'report': True, 'path': 'html', 'ext': 'htm', 'filter': 'nonHTML' },
                             'xhtml1': { 'report': True, 'path': 'xhtml1', 'ext': 'xht', 'filter': 'HTMLonly' },
                             'xhtml1print': { 'report': False, 'path': 'xhtml1print', 'ext': 'xht', 'filter': 'HTMLonly' },
                             'svg': { 'report': True, 'path': 'svg', 'ext': 'svg', 'filter': 'HTMLonly' }
                           }

    # Copy simple copy files
    for tmplDir in reversed(self.templatePath):
      files = listfiles(tmplDir)
      for file in files:
        if self.overviewCopyExtPat.match(file):
          shutil.copy(join(tmplDir, file), join(destDir, file))

    # Generate indexes
    for tmpl in self.overviewTmplNames:
      out = tmpl[0:-5] if tmpl.endswith('.tmpl') else tmpl
      self.__writeTemplate(tmpl, data, join(destDir, out))

    # Report errors
    if (self.errors):
        if type(errorOut) is type(('tmpl','out')):
            data['errors'] = errors
            self.__writeTemplate(errorOut[0], data, join(destDir, errorOut[1]))
        else:
            sys.stdout.flush()
            for errorLocation in self.errors:
                print >> errorOut, "Error in %s: %s" % \
                               (errorLocation, ' '.join([str(error) for error in self.errors[errorLocation]]))

  def writeIndex(self, format):
    """Write indices into test suite build output through format `format`.
    """

    # Set common values
    data = self.extraData.copy()
    data['suitetitle'] = self.suite.title
    data['suite']      = self.suite.name
    data['specroot']   = self.suite.specroot
    data['draftroot']  = self.suite.draftroot
    
    data['indexext']   = format.indexExt
    data['isXML']      = format.indexExt.startswith('.x')
    data['formatdir']  = format.formatDirName
    data['extmap']     = format.extMap
    data['tests']      = self.alltests
    data['suites']     = self.suites
    data['flagInfo']   = self.flags

    # Generate indices:

    # Reftest indices
    self.__writeTemplate('reftest-toc.tmpl', data,
                         format.dest('reftest-toc%s' % format.indexExt))
    self.__writeTemplate('reftest.tmpl', data,
                         format.dest('reftest.list'))

    # Table of Contents
    sectionlist = sorted(self.sections.values())
    if self.splitChapter:
      # Split sectionlist into chapters
      chapters = []
      lastChapNum = '$' # some nonmatching initial char
      chap = None
      for section in sectionlist:
        if (section.title and (section.chapterNum() != lastChapNum)):
          lastChapNum = section.chapterNum()
          chap = section
          chap.sections = []
          chap.testcount = 0
          chap.testnames = set()
          chapters.append(chap)
        chap.testnames.update([test['name'] for test in section.tests])
        chap.testcount = len(chap.testnames)
        chap.sections.append(section)

      # Generate main toc
      data['chapters'] = chapters
      self.__writeTemplate('chapter-toc.tmpl', data,
                           format.dest('toc%s' % format.indexExt))
      del data['chapters']

      # Generate chapter tocs
      for chap in chapters:
        data['chaptertitle'] = chap.title
        data['testcount']    = chap.testcount
        data['sections']     = chap.sections
        self.__writeTemplate('test-toc.tmpl', data, format.dest('chapter-%s%s' \
                             % (chap.numstr, format.indexExt)))

    else: # not splitChapter
      data['chapters'] = sectionlist
      self.__writeTemplate('test-toc.tmpl', data,
                           format.dest('toc%s' % format.indexExt))
      del data['chapters']
