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


"""Scans a source JS file for its provided and required namespaces.

Simple class to scan a JavaScript file and express its dependencies.
"""

__author__ = 'nnaze@google.com'


import re

_BASE_REGEX_STRING = r'^\s*goog\.%s\(\s*[\'"](.+)[\'"]\s*\)'
_MODULE_REGEX = re.compile(_BASE_REGEX_STRING % 'module')
_PROVIDE_REGEX = re.compile(_BASE_REGEX_STRING % 'provide')

_REQUIRE_REGEX_STRING = (r'^\s*(?:(?:var|let|const)\s+[a-zA-Z_$][a-zA-Z0-9$_]*'
                         r'\s*=\s*)?goog\.require\(\s*[\'"](.+)[\'"]\s*\)')
_REQUIRES_REGEX = re.compile(_REQUIRE_REGEX_STRING)


class Source(object):
  """Scans a JavaScript source for its provided and required namespaces."""

  # Matches a "/* ... */" comment.
  # Note: We can't definitively distinguish a "/*" in a string literal without a
  # state machine tokenizer. We'll assume that a line starting with whitespace
  # and "/*" is a comment.
  _COMMENT_REGEX = re.compile(
      r"""
      ^\s*   # Start of a new line and whitespace
      /\*    # Opening "/*"
      .*?    # Non greedy match of any characters (including newlines)
      \*/    # Closing "*/""",
      re.MULTILINE | re.DOTALL | re.VERBOSE)

  def __init__(self, source):
    """Initialize a source.

    Args:
      source: str, The JavaScript source.
    """

    self.provides = set()
    self.requires = set()
    self.is_goog_module = False

    self._source = source
    self._ScanSource()

  def GetSource(self):
    """Get the source as a string."""
    return self._source

  @classmethod
  def _StripComments(cls, source):
    return cls._COMMENT_REGEX.sub('', source)

  @classmethod
  def _HasProvideGoogFlag(cls, source):
    """Determines whether the @provideGoog flag is in a comment."""
    for comment_content in cls._COMMENT_REGEX.findall(source):
      if '@provideGoog' in comment_content:
        return True

    return False

  def _ScanSource(self):
    """Fill in provides and requires by scanning the source."""

    stripped_source = self._StripComments(self.GetSource())

    source_lines = stripped_source.splitlines()
    for line in source_lines:
      match = _PROVIDE_REGEX.match(line)
      if match:
        self.provides.add(match.group(1))
      match = _MODULE_REGEX.match(line)
      if match:
        self.provides.add(match.group(1))
        self.is_goog_module = True
      match = _REQUIRES_REGEX.match(line)
      if match:
        self.requires.add(match.group(1))

    # Closure's base file implicitly provides 'goog'.
    # This is indicated with the @provideGoog flag.
    if self._HasProvideGoogFlag(self.GetSource()):

      if len(self.provides) or len(self.requires):
        raise Exception(
            'Base file should not provide or require namespaces.')

      self.provides.add('goog')


def GetFileContents(path):
  """Get a file's contents as a string.

  Args:
    path: str, Path to file.

  Returns:
    str, Contents of file.

  Raises:
    IOError: An error occurred opening or reading the file.

  """
  fileobj = open(path)
  try:
    return fileobj.read()
  finally:
    fileobj.close()
