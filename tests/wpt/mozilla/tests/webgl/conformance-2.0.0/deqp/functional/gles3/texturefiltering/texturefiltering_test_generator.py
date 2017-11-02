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
  Generator for texturefilter* tests.
  This file needs to be run in its folder.
"""

import sys

_DO_NOT_EDIT_WARNING = """<!--

This file is auto-generated from texturefiltering_test_generator.py
DO NOT EDIT!

-->

"""

_HTML_TEMPLATE = """<html>
<head>
<meta http-equiv="Content-Type" content="text/html; charset=utf-8">
<title>WebGL Texture Filtering Tests</title>
<link rel="stylesheet" href="../../../../resources/js-test-style.css"/>
<script src="../../../../js/js-test-pre.js"></script>
<script src="../../../../js/webgl-test-utils.js"></script>

<script src="../../../../closure-library/closure/goog/base.js"></script>
<script src="../../../deqp-deps.js"></script>
<script>goog.require('functional.gles3.es3fTextureFilteringTests');</script>
</head>
<body>
<div id="description"></div>
<div id="console"></div>
<canvas id="canvas" width="300" height="300"> </canvas>
<script>
var wtu = WebGLTestUtils;
var gl = wtu.create3DContext('canvas', null, 2);

functional.gles3.es3fTextureFilteringTests.run(gl, [%(start)s, %(end)s]);
</script>
</body>
</html>
"""

_FILTERABLE_FORMAT_COUNT = 10
_SIZE_2D_COUNT = 6
_SIZE_CUBE_COUNT = 5
_SIZE_2D_ARRAY_COUNT = 5
_SIZE_3D_COUNT = 5
_MIN_FILTER_MODE_COUNT = 6
_MAG_FILTER_MODE_COUNT = 2
_WRAP_MODE_COUNT = 3

_GROUPS = [
    '2d_formats',
    '2d_sizes',
    '2d_combinations',
    'cube_formats',
    'cube_sizes',
    'cube_combinations',
    'cube_no_edges_visible',
    '2d_array_formats',
    '2d_array_sizes',
    '2d_array_combinations',
    '3d_formats',
    '3d_sizes',
    '3d_combinations'
]

_GROUP_TEST_COUNTS = [
    _FILTERABLE_FORMAT_COUNT, # 2d_formats
    _SIZE_2D_COUNT, # 2d_sizes
    _MIN_FILTER_MODE_COUNT, # 2d_combinations
    _FILTERABLE_FORMAT_COUNT, # cube_formats
    _SIZE_CUBE_COUNT, # cube_sizes
    _MIN_FILTER_MODE_COUNT, # cube_combinations
    1, # cube_no_edges_visible
    _FILTERABLE_FORMAT_COUNT, # 2d_array_formats
    _SIZE_2D_ARRAY_COUNT, # 2d_array_sizes
    _MIN_FILTER_MODE_COUNT, # 2d_array_combinations
    _FILTERABLE_FORMAT_COUNT, # 3d_formats
    _SIZE_3D_COUNT, # 3d_sizes,
    _MIN_FILTER_MODE_COUNT * _MAG_FILTER_MODE_COUNT * _WRAP_MODE_COUNT, # 3d_combinations
]

def GenerateFilename(group, count, index):
  """Generate test filename."""
  assert index >= 0 and index < count
  filename = group
  if count > 1:
    index_str = str(index)
    if index < 10:
      index_str = "0" + index_str
    filename += "_" + index_str
  filename += ".html"
  return filename

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
  assert len(_GROUPS) == len(_GROUP_TEST_COUNTS)
  test_index = 0
  filelist = []
  for ii in range(len(_GROUPS)):
    group = _GROUPS[ii]
    count = _GROUP_TEST_COUNTS[ii]
    for index in range(count):
      filename = GenerateFilename(group, count, index)
      filelist.append(filename)
      WriteTest(filename, test_index, test_index + 1)
      test_index += 1
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
