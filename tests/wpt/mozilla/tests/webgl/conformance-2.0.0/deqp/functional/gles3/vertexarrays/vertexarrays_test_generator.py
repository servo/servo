#!/usr/bin/env python

# Copyright (c) 2016 The Khronos Group Inc.
#
# Permission is hereby granted, free of charge, to any person obtaining a
# copy of this software and/or associated documentation files (the
# "Materials"), to deal in the Materials without restriction, including
# without limitation the rights to use, copy, modify, merge, publish,
# distribute, sublicense, and/or sell copies of the Materials, and to
# permit persons to whom the Materials are furnished to do so, subject to
# the following conditions:
#
# The above copyright notice and this permission notice shall be included
# in all copies or substantial portions of the Materials.
#
# THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
# IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
# CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
# TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
# MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.

"""
  Generator for vertexarrays* tests.
  This file needs to be run in its folder.
"""

import sys

_DO_NOT_EDIT_WARNING = """<!--

This file is auto-generated from vertexarrays_test_generator.py
DO NOT EDIT!

-->

"""

_HTML_TEMPLATE = """<html>
<head>
<meta http-equiv="Content-Type" content="text/html; charset=utf-8">
<title>WebGL Vertex Arrays Conformance Tests</title>
<link rel="stylesheet" href="../../../../resources/js-test-style.css"/>
<script src="../../../../js/js-test-pre.js"></script>
<script src="../../../../js/webgl-test-utils.js"></script>

<script src="../../../../closure-library/closure/goog/base.js"></script>
<script src="../../../deqp-deps.js"></script>
<script>goog.require('functional.gles3.es3fVertexArrayTests');</script>
</head>
<body>
<div id="description"></div>
<div id="console"></div>
<canvas id="canvas" width="200" height="100"> </canvas>
<script>
var wtu = WebGLTestUtils;
var gl = wtu.create3DContext('canvas', null, 2);

functional.gles3.es3fVertexArrayTests.run(gl, [%(start)s, %(end)s]);
</script>
</body>
</html>
"""

_GROUPS = [
    'single_attribute.stride',
    'single_attribute.normalize',
    'single_attribute.output_type.float',
    'single_attribute.output_type.short',
    'single_attribute.output_type.byte',
    'single_attribute.output_type.unsigned_short',
    'single_attribute.output_type.unsigned_byte',
    'single_attribute.output_type.unsigned_int',
    'single_attribute.output_type.int',
    'single_attribute.output_type.half',
    'single_attribute.output_type.unsigned_int_2_10_10_10',
    'single_attribute.output_type.int_2_10_10_10',
    'single_attribute.usage.static_draw',
    'single_attribute.usage.stream_draw',
    'single_attribute.usage.dynamic_draw',
    'single_attribute.usage.static_copy',
    'single_attribute.usage.stream_copy',
    'single_attribute.usage.dynamic_copy',
    'single_attribute.usage.static_read',
    'single_attribute.usage.stream_read',
    'single_attribute.usage.dynamic_read',
    'single_attribute.offset',
    'single_attribute.first',
    'multiple_attributes.count',
    'multiple_attributes.storage',
    'multiple_attributes.stride',
    'multiple_attributes.output',
]

def WriteTest(filename, start, end):
  """Write one test."""
  file = open(filename, "wb")
  file.write(_DO_NOT_EDIT_WARNING)
  file.write(_HTML_TEMPLATE % {
    'start': start,
    'end': end
  })
  file.close

def GenerateTests():
  """Generate all tests."""
  filelist = []
  ii = 0
  for ii in range(len(_GROUPS)):
    filename = _GROUPS[ii] + ".html"
    filelist.append(filename)
    WriteTest(filename, ii, ii + 1)
  return filelist

def GenerateTestList(filelist):
  file = open("00_test_list.txt", "wb")
  file.write('\n'.join(filelist))
  file.close

def main(argv):
  """This is the main function."""
  filelist = GenerateTests()
  GenerateTestList(filelist)

if __name__ == '__main__':
  sys.exit(main(sys.argv[1:]))
