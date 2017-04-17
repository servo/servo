#!/usr/bin/python
# CSS Test Suite Manipulation Library Utilities
# Initial code by fantasai, joint copyright 2010 W3C and Microsoft
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

###### XML Parsing ######

import os
import w3ctestlib
os.environ['XML_CATALOG_FILES'] = os.path.join(w3ctestlib.__path__[0], 'catalog/catalog.xml')

###### File path manipulation ######

import os.path
from os.path import sep, pardir

def assetName(path):
  return intern(os.path.splitext(os.path.basename(path))[0].lower().encode('ascii'))
  
def basepath(path):
  """ Returns the path part of os.path.split.
  """
  return os.path.split(path)[0]

def isPathInsideBase(path, base=''):
  path = os.path.normpath(path)
  if base:
    base = os.path.normpath(base)
    pathlist = path.split(os.path.sep)
    baselist = base.split(os.path.sep)
    while baselist:
      p = pathlist.pop(0)
      b = baselist.pop(0)
      if p != b:
        return False
    return not pathlist[0].startswith(os.path.pardir)
  return not path.startswith(os.path.pardir)

def relpath(path, start):
  """Return relative path from start to end. WARNING: this is not the
     same as a relative URL; see relativeURL()."""
  try:
    return os.path.relpath(path, start)
  except AttributeError:
    # This function is copied directly from the Python 2.6 source
    # code, and is therefore under a different license.

    if not path:
        raise ValueError("no path specified")
    start_list = os.path.abspath(start).split(sep)
    path_list = os.path.abspath(path).split(sep)
    if start_list[0].lower() != path_list[0].lower():
        unc_path, rest = os.path.splitunc(path)
        unc_start, rest = os.path.splitunc(start)
        if bool(unc_path) ^ bool(unc_start):
            raise ValueError("Cannot mix UNC and non-UNC paths (%s and %s)"
                                                                % (path, start))
        else:
            raise ValueError("path is on drive %s, start on drive %s"
                                                % (path_list[0], start_list[0]))
    # Work out how much of the filepath is shared by start and path.
    for i in range(min(len(start_list), len(path_list))):
        if start_list[i].lower() != path_list[i].lower():
            break
    else:
        i += 1

    rel_list = [pardir] * (len(start_list)-i) + path_list[i:]
    if not rel_list:
        return os.path.curdir
    return os.path.join(*rel_list)

def relativeURL(start, end):
  """ Returns relative URL from `start` to `end`.
  """
#  if isPathInsideBase(end, start):
#    return relpath(end, start)
#  else:
  return relpath(end, basepath(start))

def listfiles(path, ext = None):
  """ Returns a list of all files in a directory.
      Optionally lists only files with a given extension.
  """
  try:
    _,_,files = os.walk(path).next()
    if (ext):
      files = [fileName for fileName in files if fileName.endswith(ext)]
  except StopIteration, e:
    files = []
  return files

def listdirs(path):
  """ Returns a list of all subdirectories in a directory.
  """
  try:
    _,dirs,_ = os.walk(path).next()
  except StopIteration, e:
    dirs = []
  return dirs

###### MIME types and file extensions ######

extensionMap = { None     : 'application/octet-stream', # default
                 '.xht'   : 'application/xhtml+xml',
                 '.xhtml' : 'application/xhtml+xml',
                 '.xml'   : 'application/xml',
                 '.htm'   : 'text/html',
                 '.html'  : 'text/html',
                 '.txt'   : 'text/plain',
                 '.jpg'   : 'image/jpeg',
                 '.png'   : 'image/png',
                 '.svg'   : 'image/svg+xml',
               }

def getMimeFromExt(filepath):
  """Convenience function: equal to extenionMap.get(ext, extensionMap[None]).
  """
  if filepath.endswith('.htaccess'):
    return 'config/htaccess'
  ext = os.path.splitext(filepath)[1]
  return extensionMap.get(ext, extensionMap[None])

###### Escaping ######

import types
from htmlentitydefs import entitydefs

entityify = dict([c,e] for e,c in entitydefs.iteritems())

def escapeMarkup(data):
  """Escape markup characters (&, >, <). Copied from xml.sax.saxutils.
  """
  # must do ampersand first
  data = data.replace("&", "&amp;")
  data = data.replace(">", "&gt;")
  data = data.replace("<", "&lt;")
  return data

def escapeToNamedASCII(text):
  """Escapes to named entities where possible and numeric-escapes non-ASCII
  """
  return escapeToNamed(text).encode('ascii', 'xmlcharrefreplace')

def escapeToNamed(text):
  """Escape characters with named entities.
  """
  escapable = set()

  for c in text:
    if ord(c) > 127:
      escapable.add(c)
  if type(text) == types.UnicodeType:
    for c in escapable:
      cLatin = c.encode('Latin-1', 'ignore')
      if (cLatin in entityify):
        text = text.replace(c, "&%s;" % entityify[cLatin])
  else:
    for c in escapable:
      text = text.replace(c, "&%s;" % entityify[c])
  return text
