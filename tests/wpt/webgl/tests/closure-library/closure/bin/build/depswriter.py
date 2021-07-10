#!/usr/bin/env python
#
# Copyright 2009 The Closure Library Authors. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS-IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


"""Generates out a Closure deps.js file given a list of JavaScript sources.

Paths can be specified as arguments or (more commonly) specifying trees
with the flags (call with --help for descriptions).

Usage: depswriter.py [path/to/js1.js [path/to/js2.js] ...]
"""

import logging
import optparse
import os
import posixpath
import shlex
import sys

import source
import treescan


__author__ = 'nnaze@google.com (Nathan Naze)'


def MakeDepsFile(source_map):
  """Make a generated deps file.

  Args:
    source_map: A dict map of the source path to source.Source object.

  Returns:
    str, A generated deps file source.
  """

  # Write in path alphabetical order
  paths = sorted(source_map.keys())

  lines = []

  for path in paths:
    js_source = source_map[path]

    # We don't need to add entries that don't provide anything.
    if js_source.provides:
      lines.append(_GetDepsLine(path, js_source))

  return ''.join(lines)


def _GetDepsLine(path, js_source):
  """Get a deps.js file string for a source."""

  provides = sorted(js_source.provides)
  requires = sorted(js_source.requires)
  module = 'true' if js_source.is_goog_module else 'false'

  return 'goog.addDependency(\'%s\', %s, %s, %s);\n' % (
      path, provides, requires, module)


def _GetOptionsParser():
  """Get the options parser."""

  parser = optparse.OptionParser(__doc__)

  parser.add_option('--output_file',
                    dest='output_file',
                    action='store',
                    help=('If specified, write output to this path instead of '
                          'writing to standard output.'))
  parser.add_option('--root',
                    dest='roots',
                    default=[],
                    action='append',
                    help='A root directory to scan for JS source files. '
                    'Paths of JS files in generated deps file will be '
                    'relative to this path.  This flag may be specified '
                    'multiple times.')
  parser.add_option('--root_with_prefix',
                    dest='roots_with_prefix',
                    default=[],
                    action='append',
                    help='A root directory to scan for JS source files, plus '
                    'a prefix (if either contains a space, surround with '
                    'quotes).  Paths in generated deps file will be relative '
                    'to the root, but preceded by the prefix.  This flag '
                    'may be specified multiple times.')
  parser.add_option('--path_with_depspath',
                    dest='paths_with_depspath',
                    default=[],
                    action='append',
                    help='A path to a source file and an alternate path to '
                    'the file in the generated deps file (if either contains '
                    'a space, surround with whitespace). This flag may be '
                    'specified multiple times.')
  return parser


def _NormalizePathSeparators(path):
  """Replaces OS-specific path separators with POSIX-style slashes.

  Args:
    path: str, A file path.

  Returns:
    str, The path with any OS-specific path separators (such as backslash on
      Windows) replaced with URL-compatible forward slashes. A no-op on systems
      that use POSIX paths.
  """
  return path.replace(os.sep, posixpath.sep)


def _GetRelativePathToSourceDict(root, prefix=''):
  """Scans a top root directory for .js sources.

  Args:
    root: str, Root directory.
    prefix: str, Prefix for returned paths.

  Returns:
    dict, A map of relative paths (with prefix, if given), to source.Source
      objects.
  """
  # Remember and restore the cwd when we're done. We work from the root so
  # that paths are relative from the root.
  start_wd = os.getcwd()
  os.chdir(root)

  path_to_source = {}
  for path in treescan.ScanTreeForJsFiles('.'):
    prefixed_path = _NormalizePathSeparators(os.path.join(prefix, path))
    path_to_source[prefixed_path] = source.Source(source.GetFileContents(path))

  os.chdir(start_wd)

  return path_to_source


def _GetPair(s):
  """Return a string as a shell-parsed tuple.  Two values expected."""
  try:
    # shlex uses '\' as an escape character, so they must be escaped.
    s = s.replace('\\', '\\\\')
    first, second = shlex.split(s)
    return (first, second)
  except:
    raise Exception('Unable to parse input line as a pair: %s' % s)


def main():
  """CLI frontend to MakeDepsFile."""
  logging.basicConfig(format=(sys.argv[0] + ': %(message)s'),
                      level=logging.INFO)
  options, args = _GetOptionsParser().parse_args()

  path_to_source = {}

  # Roots without prefixes
  for root in options.roots:
    path_to_source.update(_GetRelativePathToSourceDict(root))

  # Roots with prefixes
  for root_and_prefix in options.roots_with_prefix:
    root, prefix = _GetPair(root_and_prefix)
    path_to_source.update(_GetRelativePathToSourceDict(root, prefix=prefix))

  # Source paths
  for path in args:
    path_to_source[path] = source.Source(source.GetFileContents(path))

  # Source paths with alternate deps paths
  for path_with_depspath in options.paths_with_depspath:
    srcpath, depspath = _GetPair(path_with_depspath)
    path_to_source[depspath] = source.Source(source.GetFileContents(srcpath))

  # Make our output pipe.
  if options.output_file:
    out = open(options.output_file, 'w')
  else:
    out = sys.stdout

  out.write('// This file was autogenerated by %s.\n' % sys.argv[0])
  out.write('// Please do not edit.\n')

  out.write(MakeDepsFile(path_to_source))


if __name__ == '__main__':
  main()
