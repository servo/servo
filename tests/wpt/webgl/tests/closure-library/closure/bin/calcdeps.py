#!/usr/bin/env python
#
# Copyright 2006 The Closure Library Authors. All Rights Reserved.
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


"""Calculates JavaScript dependencies without requiring Google's build system.

This tool is deprecated and is provided for legacy users.
See build/closurebuilder.py and build/depswriter.py for the current tools.

It iterates over a number of search paths and builds a dependency tree.  With
the inputs provided, it walks the dependency tree and outputs all the files
required for compilation.
"""





try:
  import distutils.version
except ImportError:
  # distutils is not available in all environments
  distutils = None

import logging
import optparse
import os
import re
import subprocess
import sys


_BASE_REGEX_STRING = '^\s*goog\.%s\(\s*[\'"](.+)[\'"]\s*\)'
req_regex = re.compile(_BASE_REGEX_STRING % 'require')
prov_regex = re.compile(_BASE_REGEX_STRING % 'provide')
ns_regex = re.compile('^ns:((\w+\.)*(\w+))$')
version_regex = re.compile('[\.0-9]+')


def IsValidFile(ref):
  """Returns true if the provided reference is a file and exists."""
  return os.path.isfile(ref)


def IsJsFile(ref):
  """Returns true if the provided reference is a Javascript file."""
  return ref.endswith('.js')


def IsNamespace(ref):
  """Returns true if the provided reference is a namespace."""
  return re.match(ns_regex, ref) is not None


def IsDirectory(ref):
  """Returns true if the provided reference is a directory."""
  return os.path.isdir(ref)


def ExpandDirectories(refs):
  """Expands any directory references into inputs.

  Description:
    Looks for any directories in the provided references.  Found directories
    are recursively searched for .js files, which are then added to the result
    list.

  Args:
    refs: a list of references such as files, directories, and namespaces

  Returns:
    A list of references with directories removed and replaced by any
    .js files that are found in them. Also, the paths will be normalized.
  """
  result = []
  for ref in refs:
    if IsDirectory(ref):
      # Disable 'Unused variable' for subdirs
      # pylint: disable=unused-variable
      for (directory, subdirs, filenames) in os.walk(ref):
        for filename in filenames:
          if IsJsFile(filename):
            result.append(os.path.join(directory, filename))
    else:
      result.append(ref)
  return map(os.path.normpath, result)


class DependencyInfo(object):
  """Represents a dependency that is used to build and walk a tree."""

  def __init__(self, filename):
    self.filename = filename
    self.provides = []
    self.requires = []

  def __str__(self):
    return '%s Provides: %s Requires: %s' % (self.filename,
                                             repr(self.provides),
                                             repr(self.requires))


def BuildDependenciesFromFiles(files):
  """Build a list of dependencies from a list of files.

  Description:
    Takes a list of files, extracts their provides and requires, and builds
    out a list of dependency objects.

  Args:
    files: a list of files to be parsed for goog.provides and goog.requires.

  Returns:
    A list of dependency objects, one for each file in the files argument.
  """
  result = []
  filenames = set()
  for filename in files:
    if filename in filenames:
      continue

    # Python 3 requires the file encoding to be specified
    if (sys.version_info[0] < 3):
      file_handle = open(filename, 'r')
    else:
      file_handle = open(filename, 'r', encoding='utf8')

    try:
      dep = CreateDependencyInfo(filename, file_handle)
      result.append(dep)
    finally:
      file_handle.close()

    filenames.add(filename)

  return result


def CreateDependencyInfo(filename, source):
  """Create dependency info.

  Args:
    filename: Filename for source.
    source: File-like object containing source.

  Returns:
    A DependencyInfo object with provides and requires filled.
  """
  dep = DependencyInfo(filename)
  for line in source:
    if re.match(req_regex, line):
      dep.requires.append(re.search(req_regex, line).group(1))
    if re.match(prov_regex, line):
      dep.provides.append(re.search(prov_regex, line).group(1))
  return dep


def BuildDependencyHashFromDependencies(deps):
  """Builds a hash for searching dependencies by the namespaces they provide.

  Description:
    Dependency objects can provide multiple namespaces.  This method enumerates
    the provides of each dependency and adds them to a hash that can be used
    to easily resolve a given dependency by a namespace it provides.

  Args:
    deps: a list of dependency objects used to build the hash.

  Raises:
    Exception: If a multiple files try to provide the same namepace.

  Returns:
    A hash table { namespace: dependency } that can be used to resolve a
    dependency by a namespace it provides.
  """
  dep_hash = {}
  for dep in deps:
    for provide in dep.provides:
      if provide in dep_hash:
        raise Exception('Duplicate provide (%s) in (%s, %s)' % (
            provide,
            dep_hash[provide].filename,
            dep.filename))
      dep_hash[provide] = dep
  return dep_hash


def CalculateDependencies(paths, inputs):
  """Calculates the dependencies for given inputs.

  Description:
    This method takes a list of paths (files, directories) and builds a
    searchable data structure based on the namespaces that each .js file
    provides.  It then parses through each input, resolving dependencies
    against this data structure.  The final output is a list of files,
    including the inputs, that represent all of the code that is needed to
    compile the given inputs.

  Args:
    paths: the references (files, directories) that are used to build the
      dependency hash.
    inputs: the inputs (files, directories, namespaces) that have dependencies
      that need to be calculated.

  Raises:
    Exception: if a provided input is invalid.

  Returns:
    A list of all files, including inputs, that are needed to compile the given
    inputs.
  """
  deps = BuildDependenciesFromFiles(paths + inputs)
  search_hash = BuildDependencyHashFromDependencies(deps)
  result_list = []
  seen_list = []
  for input_file in inputs:
    if IsNamespace(input_file):
      namespace = re.search(ns_regex, input_file).group(1)
      if namespace not in search_hash:
        raise Exception('Invalid namespace (%s)' % namespace)
      input_file = search_hash[namespace].filename
    if not IsValidFile(input_file) or not IsJsFile(input_file):
      raise Exception('Invalid file (%s)' % input_file)
    seen_list.append(input_file)
    file_handle = open(input_file, 'r')
    try:
      for line in file_handle:
        if re.match(req_regex, line):
          require = re.search(req_regex, line).group(1)
          ResolveDependencies(require, search_hash, result_list, seen_list)
    finally:
      file_handle.close()
    result_list.append(input_file)

  # All files depend on base.js, so put it first.
  base_js_path = FindClosureBasePath(paths)
  if base_js_path:
    result_list.insert(0, base_js_path)
  else:
    logging.warning('Closure Library base.js not found.')

  return result_list


def FindClosureBasePath(paths):
  """Given a list of file paths, return Closure base.js path, if any.

  Args:
    paths: A list of paths.

  Returns:
    The path to Closure's base.js file including filename, if found.
  """

  for path in paths:
    pathname, filename = os.path.split(path)

    if filename == 'base.js':
      f = open(path)

      is_base = False

      # Sanity check that this is the Closure base file.  Check that this
      # is where goog is defined.  This is determined by the @provideGoog
      # flag.
      for line in f:
        if '@provideGoog' in line:
          is_base = True
          break

      f.close()

      if is_base:
        return path

def ResolveDependencies(require, search_hash, result_list, seen_list):
  """Takes a given requirement and resolves all of the dependencies for it.

  Description:
    A given requirement may require other dependencies.  This method
    recursively resolves all dependencies for the given requirement.

  Raises:
    Exception: when require does not exist in the search_hash.

  Args:
    require: the namespace to resolve dependencies for.
    search_hash: the data structure used for resolving dependencies.
    result_list: a list of filenames that have been calculated as dependencies.
      This variable is the output for this function.
    seen_list: a list of filenames that have been 'seen'.  This is required
      for the dependency->dependant ordering.
  """
  if require not in search_hash:
    raise Exception('Missing provider for (%s)' % require)

  dep = search_hash[require]
  if not dep.filename in seen_list:
    seen_list.append(dep.filename)
    for sub_require in dep.requires:
      ResolveDependencies(sub_require, search_hash, result_list, seen_list)
    result_list.append(dep.filename)


def GetDepsLine(dep, base_path):
  """Returns a JS string for a dependency statement in the deps.js file.

  Args:
    dep: The dependency that we're printing.
    base_path: The path to Closure's base.js including filename.
  """
  return 'goog.addDependency("%s", %s, %s);' % (
      GetRelpath(dep.filename, base_path), dep.provides, dep.requires)


def GetRelpath(path, start):
  """Return a relative path to |path| from |start|."""
  # NOTE: Python 2.6 provides os.path.relpath, which has almost the same
  # functionality as this function. Since we want to support 2.4, we have
  # to implement it manually. :(
  path_list = os.path.abspath(os.path.normpath(path)).split(os.sep)
  start_list = os.path.abspath(
      os.path.normpath(os.path.dirname(start))).split(os.sep)

  common_prefix_count = 0
  for i in range(0, min(len(path_list), len(start_list))):
    if path_list[i] != start_list[i]:
      break
    common_prefix_count += 1

  # Always use forward slashes, because this will get expanded to a url,
  # not a file path.
  return '/'.join(['..'] * (len(start_list) - common_prefix_count) +
                  path_list[common_prefix_count:])


def PrintLine(msg, out):
  out.write(msg)
  out.write('\n')


def PrintDeps(source_paths, deps, out):
  """Print out a deps.js file from a list of source paths.

  Args:
    source_paths: Paths that we should generate dependency info for.
    deps: Paths that provide dependency info. Their dependency info should
        not appear in the deps file.
    out: The output file.

  Returns:
    True on success, false if it was unable to find the base path
    to generate deps relative to.
  """
  base_path = FindClosureBasePath(source_paths + deps)
  if not base_path:
    return False

  PrintLine('// This file was autogenerated by calcdeps.py', out)
  excludesSet = set(deps)

  for dep in BuildDependenciesFromFiles(source_paths + deps):
    if not dep.filename in excludesSet:
      PrintLine(GetDepsLine(dep, base_path), out)

  return True


def PrintScript(source_paths, out):
  for index, dep in enumerate(source_paths):
    PrintLine('// Input %d' % index, out)
    f = open(dep, 'r')
    PrintLine(f.read(), out)
    f.close()


def GetJavaVersion():
  """Returns the string for the current version of Java installed."""
  proc = subprocess.Popen(['java', '-version'], stderr=subprocess.PIPE)
  proc.wait()
  version_line = proc.stderr.read().splitlines()[0]
  return version_regex.search(version_line).group()


def FilterByExcludes(options, files):
  """Filters the given files by the exlusions specified at the command line.

  Args:
    options: The flags to calcdeps.
    files: The files to filter.
  Returns:
    A list of files.
  """
  excludes = []
  if options.excludes:
    excludes = ExpandDirectories(options.excludes)

  excludesSet = set(excludes)
  return [i for i in files if not i in excludesSet]


def GetPathsFromOptions(options):
  """Generates the path files from flag options.

  Args:
    options: The flags to calcdeps.
  Returns:
    A list of files in the specified paths. (strings).
  """

  search_paths = options.paths
  if not search_paths:
    search_paths = ['.']  # Add default folder if no path is specified.

  search_paths = ExpandDirectories(search_paths)
  return FilterByExcludes(options, search_paths)


def GetInputsFromOptions(options):
  """Generates the inputs from flag options.

  Args:
    options: The flags to calcdeps.
  Returns:
    A list of inputs (strings).
  """
  inputs = options.inputs
  if not inputs:  # Parse stdin
    logging.info('No inputs specified. Reading from stdin...')
    inputs = filter(None, [line.strip('\n') for line in sys.stdin.readlines()])

  logging.info('Scanning files...')
  inputs = ExpandDirectories(inputs)

  return FilterByExcludes(options, inputs)


def Compile(compiler_jar_path, source_paths, out, flags=None):
  """Prepares command-line call to Closure compiler.

  Args:
    compiler_jar_path: Path to the Closure compiler .jar file.
    source_paths: Source paths to build, in order.
    flags: A list of additional flags to pass on to Closure compiler.
  """
  args = ['java', '-jar', compiler_jar_path]
  for path in source_paths:
    args += ['--js', path]

  if flags:
    args += flags

  logging.info('Compiling with the following command: %s', ' '.join(args))
  proc = subprocess.Popen(args, stdout=subprocess.PIPE)
  (stdoutdata, stderrdata) = proc.communicate()
  if proc.returncode != 0:
    logging.error('JavaScript compilation failed.')
    sys.exit(1)
  else:
    out.write(stdoutdata)


def main():
  """The entrypoint for this script."""

  logging.basicConfig(format='calcdeps.py: %(message)s', level=logging.INFO)

  usage = 'usage: %prog [options] arg'
  parser = optparse.OptionParser(usage)
  parser.add_option('-i',
                    '--input',
                    dest='inputs',
                    action='append',
                    help='The inputs to calculate dependencies for. Valid '
                    'values can be files, directories, or namespaces '
                    '(ns:goog.net.XhrIo).  Only relevant to "list" and '
                    '"script" output.')
  parser.add_option('-p',
                    '--path',
                    dest='paths',
                    action='append',
                    help='The paths that should be traversed to build the '
                    'dependencies.')
  parser.add_option('-d',
                    '--dep',
                    dest='deps',
                    action='append',
                    help='Directories or files that should be traversed to '
                    'find required dependencies for the deps file. '
                    'Does not generate dependency information for names '
                    'provided by these files. Only useful in "deps" mode.')
  parser.add_option('-e',
                    '--exclude',
                    dest='excludes',
                    action='append',
                    help='Files or directories to exclude from the --path '
                    'and --input flags')
  parser.add_option('-o',
                    '--output_mode',
                    dest='output_mode',
                    action='store',
                    default='list',
                    help='The type of output to generate from this script. '
                    'Options are "list" for a list of filenames, "script" '
                    'for a single script containing the contents of all the '
                    'file, "deps" to generate a deps.js file for all '
                    'paths, or "compiled" to produce compiled output with '
                    'the Closure compiler.')
  parser.add_option('-c',
                    '--compiler_jar',
                    dest='compiler_jar',
                    action='store',
                    help='The location of the Closure compiler .jar file.')
  parser.add_option('-f',
                    '--compiler_flag',
                    '--compiler_flags', # for backwards compatability
                    dest='compiler_flags',
                    action='append',
                    help='Additional flag to pass to the Closure compiler. '
                    'May be specified multiple times to pass multiple flags.')
  parser.add_option('--output_file',
                    dest='output_file',
                    action='store',
                    help=('If specified, write output to this path instead of '
                          'writing to standard output.'))

  (options, args) = parser.parse_args()

  search_paths = GetPathsFromOptions(options)

  if options.output_file:
    out = open(options.output_file, 'w')
  else:
    out = sys.stdout

  if options.output_mode == 'deps':
    result = PrintDeps(search_paths, ExpandDirectories(options.deps or []), out)
    if not result:
      logging.error('Could not find Closure Library in the specified paths')
      sys.exit(1)

    return

  inputs = GetInputsFromOptions(options)

  logging.info('Finding Closure dependencies...')
  deps = CalculateDependencies(search_paths, inputs)
  output_mode = options.output_mode

  if output_mode == 'script':
    PrintScript(deps, out)
  elif output_mode == 'list':
    # Just print out a dep per line
    for dep in deps:
      PrintLine(dep, out)
  elif output_mode == 'compiled':
    # Make sure a .jar is specified.
    if not options.compiler_jar:
      logging.error('--compiler_jar flag must be specified if --output is '
                    '"compiled"')
      sys.exit(1)

    # User friendly version check.
    if distutils and not (distutils.version.LooseVersion(GetJavaVersion()) >
        distutils.version.LooseVersion('1.6')):
      logging.error('Closure Compiler requires Java 1.6 or higher.')
      logging.error('Please visit http://www.java.com/getjava')
      sys.exit(1)

    Compile(options.compiler_jar, deps, out, options.compiler_flags)

  else:
    logging.error('Invalid value for --output flag.')
    sys.exit(1)

if __name__ == '__main__':
  main()
