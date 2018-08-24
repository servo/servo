#!/usr/bin/python

"""generates tests from OpenGL ES 2.0 .run/.test files."""

import os
import os.path
import sys
import re
import json
import shutil
from optparse import OptionParser
from xml.dom.minidom import parse

if sys.version < '2.6':
   print 'Wrong Python Version !!!: Need >= 2.6'
   sys.exit(1)

# each shader test generates up to 3 512x512 images.
# a 512x512 image takes 1meg of memory so set this
# number apporpriate for the platform with
# the smallest memory issue. At 8 that means
# at least 24 meg is needed to run the test.
MAX_TESTS_PER_SET = 8

VERBOSE = False

FILTERS = [
  re.compile("GL/"),
]

LICENSE = """
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/
"""

COMMENT_RE = re.compile("/\*\n\*\*\s+Copyright.*?\*/",
                        re.IGNORECASE | re.DOTALL)
REMOVE_COPYRIGHT_RE = re.compile("\/\/\s+Copyright.*?\n",
                                 re.IGNORECASE | re.DOTALL)
MATRIX_RE = re.compile("Matrix(\\d)")

VALID_UNIFORM_TYPES = [
  "uniform1f",
  "uniform1fv",
  "uniform1fv",
  "uniform1i",
  "uniform1iv",
  "uniform1iv",
  "uniform2f",
  "uniform2fv",
  "uniform2fv",
  "uniform2i",
  "uniform2iv",
  "uniform2iv",
  "uniform3f",
  "uniform3fv",
  "uniform3fv",
  "uniform3i",
  "uniform3iv",
  "uniform3iv",
  "uniform4f",
  "uniform4fv",
  "uniform4fv",
  "uniform4i",
  "uniform4iv",
  "uniform4ivy",
  "uniformMatrix2fv",
  "uniformMatrix2fv",
  "uniformMatrix3fv",
  "uniformMatrix3fv",
  "uniformMatrix4fv",
  "uniformMatrix4fv",
]

SUBSTITUTIONS = [
  ("uniformmat3fv", "uniformMatrix3fv"),
  ("uniformmat4fv", "uniformMatrix4fv"),
]


def Log(msg):
  global VERBOSE
  if VERBOSE:
    print msg


def TransposeMatrix(values, dim):
  size = dim * dim
  count = len(values) / size
  for m in range(0, count):
    offset = m * size
    for i in range(0, dim):
      for j in range(i + 1, dim):
        t = values[offset + i * dim + j]
        values[offset + i * dim + j] = values[offset + j * dim + i]
        values[offset + j * dim + i] = t


def GetValidTypeName(type_name):
  global VALID_UNIFORM_TYPES
  global SUBSTITUTIONS
  for subst in SUBSTITUTIONS:
    type_name = type_name.replace(subst[0], subst[1])
  if not type_name in VALID_UNIFORM_TYPES:
    print "unknown type name: ", type_name
    raise SyntaxError
  return type_name


def WriteOpen(filename):
  dirname = os.path.dirname(filename)
  if len(dirname) > 0 and not os.path.exists(dirname):
    os.makedirs(dirname)
  return open(filename, "wb")


class TxtWriter():
  def __init__(self, filename):
    self.filename = filename
    self.lines = []

  def Write(self, line):
    self.lines.append(line)

  def Close(self):
    if len(self.lines) > 0:
      Log("Writing: %s" % self.filename)
      f = WriteOpen(self.filename)
      f.write("# this file is auto-generated. DO NOT EDIT.\n")
      f.write("".join(self.lines))
      f.close()


def ReadFileAsLines(filename):
  f = open(filename, "r")
  lines = f.readlines()
  f.close()
  return [line.strip() for line in lines]


def ReadFile(filename):
  f = open(filename, "r")
  content = f.read()
  f.close()
  return content.replace("\r\n", "\n")


def Chunkify(list, chunk_size):
  """divides an array into chunks of chunk_size"""
  return [list[i:i + chunk_size] for i in range(0, len(list), chunk_size)]


def GetText(nodelist):
  """Gets the text of from a list of nodes"""
  rc = []
  for node in nodelist:
    if node.nodeType == node.TEXT_NODE:
      rc.append(node.data)
  return ''.join(rc)


def GetElementText(node, name):
  """Gets the text of an element"""
  elements = node.getElementsByTagName(name)
  if len(elements) > 0:
    return GetText(elements[0].childNodes)
  else:
    return None


def GetBoolElement(node, name):
  text = GetElementText(node, name)
  return text.lower() == "true"


def GetModel(node):
  """Gets the model"""
  model = GetElementText(node, "model")
  if model and len(model.strip()) == 0:
    elements = node.getElementsByTagName("model")
    if len(elements) > 0:
      model = GetElementText(elements[0], "filename")
  return model


def RelativizePaths(base, paths, template):
  """converts paths to relative paths"""
  rels = []
  for p in paths:
    #print "---"
    #print "base: ", os.path.abspath(base)
    #print "path: ", os.path.abspath(p)
    relpath = os.path.relpath(os.path.abspath(p), os.path.dirname(os.path.abspath(base))).replace("\\", "/")
    #print "rel : ", relpath
    rels.append(template % relpath)
  return "\n".join(rels)


def CopyFile(filename, src, dst):
  s = os.path.abspath(os.path.join(os.path.dirname(src), filename))
  d = os.path.abspath(os.path.join(os.path.dirname(dst), filename))
  dst_dir = os.path.dirname(d)
  if not os.path.exists(dst_dir):
    os.makedirs(dst_dir)
  shutil.copyfile(s, d)


def CopyShader(filename, src, dst):
  s = os.path.abspath(os.path.join(os.path.dirname(src), filename))
  d = os.path.abspath(os.path.join(os.path.dirname(dst), filename))
  text = ReadFile(s)
  # By agreement with the Khronos OpenGL working group we are allowed
  # to open source only the .vert and .frag files from the OpenGL ES 2.0
  # conformance tests. All other files from the OpenGL ES 2.0 conformance
  # tests are not included.
  marker = "insert-copyright-here"
  new_text = COMMENT_RE.sub(marker, text)
  if new_text == text:
    print "no matching license found:", s
    raise RuntimeError
  new_text = REMOVE_COPYRIGHT_RE.sub("", new_text)
  new_text = new_text.replace(marker, LICENSE)
  f = WriteOpen(d)
  f.write(new_text)
  f.close()


def IsOneOf(string, regexs):
  for regex in regexs:
    if re.match(regex, string):
      return True
  return False


def CheckForUnknownTags(valid_tags, node, depth=1):
  """do a hacky check to make sure we're not missing something."""
  for child in node.childNodes:
    if child.localName and not IsOneOf(child.localName, valid_tags[0]):
      print "unsupported tag:", child.localName
      print "depth:", depth
      raise SyntaxError
    else:
      if len(valid_tags) > 1:
        CheckForUnknownTags(valid_tags[1:], child, depth + 1)


def IsFileWeWant(filename):
  for f in FILTERS:
    if f.search(filename):
      return True
  return False


class TestReader():
  """class to read and parse tests"""

  def __init__(self, basepath):
    self.tests = []
    self.modes = {}
    self.patterns = {}
    self.basepath = basepath

  def Print(self, msg):
    if self.verbose:
      print msg

  def MakeOutPath(self, filename):
    relpath = os.path.relpath(os.path.abspath(filename), os.path.dirname(os.path.abspath(self.basepath)))
    return relpath

  def ReadTests(self, filename):
    """reads a .run file and parses."""
    Log("reading %s" % filename)
    outname = self.MakeOutPath(filename + ".txt")
    f = TxtWriter(outname)
    dirname = os.path.dirname(filename)
    lines = ReadFileAsLines(filename)
    count = 0
    tests_data = []
    for line in lines:
      if len(line) > 0 and not line.startswith("#"):
        fname = os.path.join(dirname, line)
        if line.endswith(".run"):
          if self.ReadTests(fname):
            f.Write(line + ".txt\n")
            count += 1
        elif line.endswith(".test"):
          tests_data.extend(self.ReadTest(fname))
        else:
          print "Error in %s:%d:%s" % (filename, count, line)
          raise SyntaxError()
    if len(tests_data):
      global MAX_TESTS_PER_SET
      sets = Chunkify(tests_data, MAX_TESTS_PER_SET)
      id = 1
      for set in sets:
        suffix = "_%03d_to_%03d" % (id, id + len(set) - 1)
        test_outname = self.MakeOutPath(filename + suffix + ".html")
        if os.path.basename(test_outname).startswith("input.run"):
          dname = os.path.dirname(test_outname)
          folder_name = os.path.basename(dname)
          test_outname = os.path.join(dname, folder_name + suffix + ".html")
        self.WriteTests(filename, test_outname, {"tests":set})
        f.Write(os.path.basename(test_outname) + "\n")
        id += len(set)
      count += 1
    f.Close()
    return count

  def ReadTest(self, filename):
    """reads a .test file and parses."""
    Log("reading %s" % filename)
    dom = parse(filename)
    tests = dom.getElementsByTagName("test")
    tests_data = []
    outname = self.MakeOutPath(filename + ".html")
    for test in tests:
      if not IsFileWeWant(filename):
        self.CopyShaders(test, filename, outname)
      else:
        test_data = self.ProcessTest(test, filename, outname, len(tests_data))
        if test_data:
          tests_data.append(test_data)
    return tests_data

  def ProcessTest(self, test, filename, outname, id):
    """Process a test"""
    mode = test.getAttribute("mode")
    pattern = test.getAttribute("pattern")
    self.modes[mode] = 1
    self.patterns[pattern] = 1
    Log ("%d: mode: %s pattern: %s" % (id, mode, pattern))
    method = getattr(self, 'Process_' + pattern)
    test_data = method(test, filename, outname)
    if test_data:
      test_data["pattern"] = pattern
    return test_data

  def WriteTests(self, filename, outname, tests_data):
    Log("Writing %s" % outname)
    template = """<!DOCTYPE html>
<!-- this file is auto-generated. DO NOT EDIT.
%(license)s
-->
<html>
<head>
<meta charset="utf-8">
<title>WebGL GLSL conformance test: %(title)s</title>
%(css)s
%(scripts)s
</head>
<body>
<canvas id="example" width="500" height="500" style="width: 16px; height: 16px;"></canvas>
<div id="description"></div>
<div id="console"></div>
</body>
<script>
"use strict";
OpenGLESTestRunner.run(%(tests_data)s);
var successfullyParsed = true;
</script>
</html>
"""
    css = [
      "../../resources/js-test-style.css",
      "../../resources/ogles-tests.css",
    ]
    scripts = [
      "../../resources/js-test-pre.js",
      "../../resources/webgl-test-utils.js",
      "ogles-utils.js",
    ]
    css_html = RelativizePaths(outname, css, '<link rel="stylesheet" href="%s" />')
    scripts_html = RelativizePaths(outname, scripts, '<script src="%s"></script>')

    f = WriteOpen(outname)
    f.write(template % {
        "license": LICENSE,
        "css": css_html,
        "scripts": scripts_html,
        "title": os.path.basename(outname),
        "tests_data": json.dumps(tests_data, indent=2)
      })
    f.close()


  def CopyShaders(self, test, filename, outname):
    """For tests we don't actually support yet, at least copy the shaders"""
    shaders = test.getElementsByTagName("shader")
    for shader in shaders:
      for name in ["vertshader", "fragshader"]:
        s = GetElementText(shader, name)
        if s and s != "empty":
          CopyShader(s, filename, outname)

  #
  # pattern handlers.
  #

  def Process_compare(self, test, filename, outname):
    global MATRIX_RE

    valid_tags = [
      ["shader", "model", "glstate"],
      ["uniform", "vertshader", "fragshader", "filename", "depthrange"],
      ["name", "count", "transpose", "uniform*", "near", "far"],
    ]
    CheckForUnknownTags(valid_tags, test)

    # parse the test
    shaders = test.getElementsByTagName("shader")
    shaderInfos = []
    for shader in shaders:
      v = GetElementText(shader, "vertshader")
      f = GetElementText(shader, "fragshader")
      CopyShader(v, filename, outname)
      CopyShader(f, filename, outname)
      info = {
        "vertexShader": v,
        "fragmentShader": f,
      }
      shaderInfos.append(info)
      uniformElems = shader.getElementsByTagName("uniform")
      if len(uniformElems) > 0:
        uniforms = {}
        info["uniforms"] = uniforms
        for uniformElem in uniformElems:
          uniform = {"count": 1}
          for child in uniformElem.childNodes:
            if child.localName == None:
              pass
            elif child.localName == "name":
              uniforms[GetText(child.childNodes)] = uniform
            elif child.localName == "count":
              uniform["count"] = int(GetText(child.childNodes))
            elif child.localName == "transpose":
              uniform["transpose"] = (GetText(child.childNodes) == "true")
            else:
              if "type" in uniform:
                print "utype was:", uniform["type"], " found ", child.localName
                raise SyntaxError
              type_name = GetValidTypeName(child.localName)
              uniform["type"] = type_name
              valueText = GetText(child.childNodes).replace(",", " ")
              uniform["value"] = [float(t) for t in valueText.split()]
              m = MATRIX_RE.search(type_name)
              if m:
                # Why are these backward from the API?!?!?
                TransposeMatrix(uniform["value"], int(m.group(1)))
    data = {
      "name": os.path.basename(outname),
      "model": GetModel(test),
      "referenceProgram": shaderInfos[1],
      "testProgram": shaderInfos[0],
    }
    gl_states = test.getElementsByTagName("glstate")
    if len(gl_states) > 0:
      state = {}
      data["state"] = state
      for gl_state in gl_states:
        for state_name in gl_state.childNodes:
          if state_name.localName:
            values = {}
            for field in state_name.childNodes:
              if field.localName:
                values[field.localName] = GetText(field.childNodes)
            state[state_name.localName] = values
    return data

  def Process_shaderload(self, test, filename, outname):
    """no need for shaderload tests"""
    self.CopyShaders(test, filename, outname)

  def Process_extension(self, test, filename, outname):
    """no need for extension tests"""
    self.CopyShaders(test, filename, outname)

  def Process_createtests(self, test, filename, outname):
    Log("createtests Not implemented:  %s" % filename)
    self.CopyShaders(test, filename, outname)

  def Process_GL2Test(self, test, filename, outname):
    Log("GL2Test Not implemented:  %s" % filename)
    self.CopyShaders(test, filename, outname)

  def Process_uniformquery(self, test, filename, outname):
    Log("uniformquery Not implemented:  %s" % filename)
    self.CopyShaders(test, filename, outname)

  def Process_egl_image_external(self, test, filename, outname):
    """no need for egl_image_external tests"""
    self.CopyShaders(test, filename, outname)

  def Process_dismount(self, test, filename, outname):
    Log("dismount Not implemented:  %s" % filename)
    self.CopyShaders(test, filename, outname)

  def Process_build(self, test, filename, outname):
    """don't need build tests"""
    valid_tags = [
      ["shader", "compstat", "linkstat"],
      ["vertshader", "fragshader"],
    ]
    CheckForUnknownTags(valid_tags, test)

    shader = test.getElementsByTagName("shader")
    if not shader:
      return None
    vs = GetElementText(shader[0], "vertshader")
    fs = GetElementText(shader[0], "fragshader")
    if vs and vs != "empty":
      CopyShader(vs, filename, outname)
    if fs and fs != "empty":
      CopyShader(fs, filename, outname)
    data = {
      "name": os.path.basename(outname),
      "compstat": bool(GetBoolElement(test, "compstat")),
      "linkstat": bool(GetBoolElement(test, "linkstat")),
      "testProgram": {
        "vertexShader": vs,
        "fragmentShader": fs,
      },
    }
    attach = test.getElementsByTagName("attach")
    if len(attach) > 0:
      data["attachError"] = GetElementText(attach[0], "attacherror")
    return data

  def Process_coverage(self, test, filename, outname):
    Log("coverage Not implemented:  %s" % filename)
    self.CopyShaders(test, filename, outname)

  def Process_attributes(self, test, filename, outname):
    Log("attributes Not implemented:  %s" % filename)
    self.CopyShaders(test, filename, outname)

  def Process_fixed(self, test, filename, outname):
    """no need for fixed function tests"""
    self.CopyShaders(test, filename, outname)


def main(argv):
  """This is the main function."""
  global VERBOSE

  parser = OptionParser()
  parser.add_option(
      "-v", "--verbose", action="store_true",
      help="prints more output.")

  (options, args) = parser.parse_args(args=argv)

  if len(args) < 1:
    pass # fix me

  os.chdir(os.path.dirname(__file__) or '.')

  VERBOSE = options.verbose

  filename = args[0]
  test_reader = TestReader(filename)
  test_reader.ReadTests(filename)


if __name__ == '__main__':
  sys.exit(main(sys.argv[1:]))
