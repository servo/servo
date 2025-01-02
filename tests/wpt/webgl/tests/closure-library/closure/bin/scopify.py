#!/usr/bin/python
#
# Copyright 2010 The Closure Library Authors. All Rights Reserved.
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


"""Automatically converts codebases over to goog.scope.

Usage:
cd path/to/my/dir;
../../../../javascript/closure/bin/scopify.py

Scans every file in this directory, recursively. Looks for existing
goog.scope calls, and goog.require'd symbols. If it makes sense to
generate a goog.scope call for the file, then we will do so, and
try to auto-generate some aliases based on the goog.require'd symbols.

Known Issues:

  When a file is goog.scope'd, the file contents will be indented +2.
  This may put some lines over 80 chars. These will need to be fixed manually.

  We will only try to create aliases for capitalized names. We do not check
  to see if those names will conflict with any existing locals.

  This creates merge conflicts for every line of every outstanding change.
  If you intend to run this on your codebase, make sure your team members
  know. Better yet, send them this script so that they can scopify their
  outstanding changes and "accept theirs".

  When an alias is "captured", it can no longer be stubbed out for testing.
  Run your tests.

"""

__author__ = 'nicksantos@google.com (Nick Santos)'

import os.path
import re
import sys

REQUIRES_RE = re.compile(r"goog.require\('([^']*)'\)")

# Edit this manually if you want something to "always" be aliased.
# TODO(nicksantos): Add a flag for this.
DEFAULT_ALIASES = {}

def Transform(lines):
  """Converts the contents of a file into javascript that uses goog.scope.

  Arguments:
    lines: A list of strings, corresponding to each line of the file.
  Returns:
    A new list of strings, or None if the file was not modified.
  """
  requires = []

  # Do an initial scan to be sure that this file can be processed.
  for line in lines:
    # Skip this file if it has already been scopified.
    if line.find('goog.scope') != -1:
      return None

    # If there are any global vars or functions, then we also have
    # to skip the whole file. We might be able to deal with this
    # more elegantly.
    if line.find('var ') == 0 or line.find('function ') == 0:
      return None

    for match in REQUIRES_RE.finditer(line):
      requires.append(match.group(1))

  if len(requires) == 0:
    return None

  # Backwards-sort the requires, so that when one is a substring of another,
  # we match the longer one first.
  for val in DEFAULT_ALIASES.values():
    if requires.count(val) == 0:
      requires.append(val)

  requires.sort()
  requires.reverse()

  # Generate a map of requires to their aliases
  aliases_to_globals = DEFAULT_ALIASES.copy()
  for req in requires:
    index = req.rfind('.')
    if index == -1:
      alias = req
    else:
      alias = req[(index + 1):]

    # Don't scopify lowercase namespaces, because they may conflict with
    # local variables.
    if alias[0].isupper():
      aliases_to_globals[alias] = req

  aliases_to_matchers = {}
  globals_to_aliases = {}
  for alias, symbol in aliases_to_globals.items():
    globals_to_aliases[symbol] = alias
    aliases_to_matchers[alias] = re.compile('\\b%s\\b' % symbol)

  # Insert a goog.scope that aliases all required symbols.
  result = []

  START = 0
  SEEN_REQUIRES = 1
  IN_SCOPE = 2

  mode = START
  aliases_used = set()
  insertion_index = None
  num_blank_lines = 0
  for line in lines:
    if mode == START:
      result.append(line)

      if re.search(REQUIRES_RE, line):
        mode = SEEN_REQUIRES

    elif mode == SEEN_REQUIRES:
      if (line and
          not re.search(REQUIRES_RE, line) and
          not line.isspace()):
        # There should be two blank lines before goog.scope
        result += ['\n'] * 2
        result.append('goog.scope(function() {\n')
        insertion_index = len(result)
        result += ['\n'] * num_blank_lines
        mode = IN_SCOPE
      elif line.isspace():
        # Keep track of the number of blank lines before each block of code so
        # that we can move them after the goog.scope line if necessary.
        num_blank_lines += 1
      else:
        # Print the blank lines we saw before this code block
        result += ['\n'] * num_blank_lines
        num_blank_lines = 0
        result.append(line)

    if mode == IN_SCOPE:
      for symbol in requires:
        if not symbol in globals_to_aliases:
          continue

        alias = globals_to_aliases[symbol]
        matcher = aliases_to_matchers[alias]
        for match in matcher.finditer(line):
          # Check to make sure we're not in a string.
          # We do this by being as conservative as possible:
          # if there are any quote or double quote characters
          # before the symbol on this line, then bail out.
          before_symbol = line[:match.start(0)]
          if before_symbol.count('"') > 0 or before_symbol.count("'") > 0:
            continue

          line = line.replace(match.group(0), alias)
          aliases_used.add(alias)

      if line.isspace():
        # Truncate all-whitespace lines
        result.append('\n')
      else:
        result.append(line)

  if len(aliases_used):
    aliases_used = [alias for alias in aliases_used]
    aliases_used.sort()
    aliases_used.reverse()
    for alias in aliases_used:
      symbol = aliases_to_globals[alias]
      result.insert(insertion_index,
                    'var %s = %s;\n' % (alias, symbol))
    result.append('});  // goog.scope\n')
    return result
  else:
    return None

def TransformFileAt(path):
  """Converts a file into javascript that uses goog.scope.

  Arguments:
    path: A path to a file.
  """
  f = open(path)
  lines = Transform(f.readlines())
  if lines:
    f = open(path, 'w')
    for l in lines:
      f.write(l)
    f.close()

if __name__ == '__main__':
  args = sys.argv[1:]
  if not len(args):
    args = '.'

  for file_name in args:
    if os.path.isdir(file_name):
      for root, dirs, files in os.walk(file_name):
        for name in files:
          if name.endswith('.js') and \
              not os.path.islink(os.path.join(root, name)):
            TransformFileAt(os.path.join(root, name))
    else:
      if file_name.endswith('.js') and \
          not os.path.islink(file_name):
        TransformFileAt(file_name)
