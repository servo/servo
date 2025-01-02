#!/usr/bin/env python

# Copyright (c) 2021 The Khronos Group Inc.
# Use of this source code is governed by an MIT-style license that can be
# found in the LICENSE.txt file.

"""
  Generator for multisample* tests.
  This file needs to be run in its folder.
"""

import sys

_DO_NOT_EDIT_WARNING = """<!--

This file is auto-generated from multisample_test_generator.py
DO NOT EDIT!

-->

"""

_DO_NOT_EDIT_TEST_LIST_WARNING = """# This file is auto-generated from multisample_test_generator.py
# DO NOT EDIT!
"""

_HTML_TEMPLATE = """<html>
<head>
<meta http-equiv="Content-Type" content="text/html; charset=utf-8">
<title>WebGL Multisample Conformance Tests</title>
<link rel="stylesheet" href="../../../../resources/js-test-style.css"/>
<script src="../../../../js/js-test-pre.js"></script>
<script src="../../../../js/webgl-test-utils.js"></script>

<script src="../../../../closure-library/closure/goog/base.js"></script>
<script src="../../../deqp-deps.js"></script>
<script>goog.require('functional.gles3.es3fMultisampleTests');</script>
</head>
<body>
<div id="description"></div>
<div id="console"></div>
<canvas id="canvas" width="256" height="256"> </canvas>
<script>
var wtu = WebGLTestUtils;
var gl = wtu.create3DContext('canvas', null, 2);

functional.gles3.es3fMultisampleTests.run(gl, [%(start)s, %(end)s]);
</script>
</body>
</html>
"""

# Note: must match CaseType in es3fMultisampleTests.js.
_GROUPS = [
  'default_fbo',
  'fbo_4_samples',
  'fbo_8_samples',
  'fbo_max_samples',
]

def GenerateFilename(group):
  """Generate test filename."""
  filename = group
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
  filelist = []
  for ii in range(len(_GROUPS)):
    filename = GenerateFilename(_GROUPS[ii])
    filelist.append(filename)
    WriteTest(filename, ii, ii + 1)
  return filelist

def GenerateTestList(filelist):
  file = open("00_test_list.txt", "wb")
  file.write(_DO_NOT_EDIT_TEST_LIST_WARNING)
  file.write('\n'.join(filelist))
  file.close

def main(argv):
  """This is the main function."""
  filelist = GenerateTests()
  GenerateTestList(filelist)

if __name__ == '__main__':
  sys.exit(main(sys.argv[1:]))
