#!/usr/bin/env python

# Copyright (c) 2015 The Khronos Group Inc.
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
  Generator for tex-2d* and tex-3d* tests.
  This file needs to be run in its folder.
"""

import os
import os.path
import sys

_LICENSE = """<!--

Copyright (c) 2015 The Khronos Group Inc.

Permission is hereby granted, free of charge, to any person obtaining a
copy of this software and/or associated documentation files (the
"Materials"), to deal in the Materials without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Materials, and to
permit persons to whom the Materials are furnished to do so, subject to
the following conditions:

The above copyright notice and this permission notice shall be included
in all copies or substantial portions of the Materials.

THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.

-->

"""

_DO_NOT_EDIT_WARNING = """<!--

This file is auto-generated from py/tex_image_test_generator.py
DO NOT EDIT!

-->

"""

_ELEMENT_TYPES = [
  'canvas',
  'canvas-sub-rectangle',
  'image',
  'image-data',
  'svg-image',
  'video',
  'webgl-canvas',
  'image-bitmap-from-image-data',
  'image-bitmap-from-image',
  'image-bitmap-from-video',
  'image-bitmap-from-canvas',
  'image-bitmap-from-blob',
  'image-bitmap-from-image-bitmap'
]

_FORMATS_TYPES_WEBGL1 = [
  {'internal_format': 'RGB', 'format': 'RGB', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGB', 'format': 'RGB', 'type': 'UNSIGNED_SHORT_5_6_5' },
  {'internal_format': 'RGBA', 'format': 'RGBA', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGBA', 'format': 'RGBA', 'type': 'UNSIGNED_SHORT_4_4_4_4' },
  {'internal_format': 'RGBA', 'format': 'RGBA', 'type': 'UNSIGNED_SHORT_5_5_5_1' },
]

_FORMATS_TYPES_WEBGL2 = [
  {'internal_format': 'R8', 'format': 'RED', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'R16F', 'format': 'RED', 'type': 'HALF_FLOAT' },
  {'internal_format': 'R16F', 'format': 'RED', 'type': 'FLOAT' },
  {'internal_format': 'R32F', 'format': 'RED', 'type': 'FLOAT' },
  {'internal_format': 'R8UI', 'format': 'RED_INTEGER', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RG8', 'format': 'RG', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RG16F', 'format': 'RG', 'type': 'HALF_FLOAT' },
  {'internal_format': 'RG16F', 'format': 'RG', 'type': 'FLOAT' },
  {'internal_format': 'RG32F', 'format': 'RG', 'type': 'FLOAT' },
  {'internal_format': 'RG8UI', 'format': 'RG_INTEGER', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGB8', 'format': 'RGB', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'SRGB8', 'format': 'RGB', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGB565', 'format': 'RGB', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGB565', 'format': 'RGB', 'type': 'UNSIGNED_SHORT_5_6_5' },
  {'internal_format': 'R11F_G11F_B10F', 'format': 'RGB', 'type': 'UNSIGNED_INT_10F_11F_11F_REV' },
  {'internal_format': 'R11F_G11F_B10F', 'format': 'RGB', 'type': 'HALF_FLOAT' },
  {'internal_format': 'R11F_G11F_B10F', 'format': 'RGB', 'type': 'FLOAT' },
  {'internal_format': 'RGB9_E5', 'format': 'RGB', 'type': 'HALF_FLOAT' },
  {'internal_format': 'RGB9_E5', 'format': 'RGB', 'type': 'FLOAT' },
  {'internal_format': 'RGB16F', 'format': 'RGB', 'type': 'HALF_FLOAT' },
  {'internal_format': 'RGB16F', 'format': 'RGB', 'type': 'FLOAT' },
  {'internal_format': 'RGB32F', 'format': 'RGB', 'type': 'FLOAT' },
  {'internal_format': 'RGB8UI', 'format': 'RGB_INTEGER', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGBA8', 'format': 'RGBA', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'SRGB8_ALPHA8', 'format': 'RGBA', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGB5_A1', 'format': 'RGBA', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGB5_A1', 'format': 'RGBA', 'type': 'UNSIGNED_SHORT_5_5_5_1' },
  {'internal_format': 'RGBA4', 'format': 'RGBA', 'type': 'UNSIGNED_BYTE' },
  {'internal_format': 'RGBA4', 'format': 'RGBA', 'type': 'UNSIGNED_SHORT_4_4_4_4' },
  {'internal_format': 'RGBA16F', 'format': 'RGBA', 'type': 'HALF_FLOAT' },
  {'internal_format': 'RGBA16F', 'format': 'RGBA', 'type': 'FLOAT' },
  {'internal_format': 'RGBA32F', 'format': 'RGBA', 'type': 'FLOAT' },
  {'internal_format': 'RGBA8UI', 'format': 'RGBA_INTEGER', 'type': 'UNSIGNED_BYTE' },
]

def GenerateFilename(dimension, element_type, internal_format, format, type):
  """Generate test filename."""
  filename = ("tex-" + dimension + "d-" +
              internal_format + "-" + format + "-" + type + ".html")
  return filename.lower()

def WriteTest(filename, dimension, element_type, internal_format, format, type, default_context_version):
  """Write one test."""
  file = open(filename, "wb")
  file.write(_LICENSE)
  file.write(_DO_NOT_EDIT_WARNING)
  code = """
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<link rel="stylesheet" href="../../../resources/js-test-style.css"/>
<script src="../../../js/js-test-pre.js"></script>
<script src="../../../js/webgl-test-utils.js"></script>
<script src="../../../js/tests/tex-image-and-sub-image-utils.js"></script>"""
  if element_type == 'image-bitmap-from-image-data' or element_type == 'image-bitmap-from-image' or \
     element_type == 'image-bitmap-from-video' or element_type == 'image-bitmap-from-canvas' or \
     element_type == 'image-bitmap-from-blob' or element_type == 'image-bitmap-from-image-bitmap':
    code += """
<script src="../../../js/tests/tex-image-and-sub-image-with-image-bitmap-utils.js"></script>"""
  code += """
<script src="../../../js/tests/tex-image-and-sub-image-%(dimension)sd-with-%(element_type)s.js"></script>
</head>
<body>"""
  if element_type == 'image-data':
    code += """
<canvas id="texcanvas" width="2" height="2"></canvas>"""
  code += """
<canvas id="example" width="32" height="32"></canvas>"""
  code += """
<div id="description"></div>
<div id="console"></div>
<script>
"use strict";
function testPrologue(gl) {
    return true;
}

generateTest("%(internal_format)s", "%(format)s", "%(type)s", testPrologue, "../../../resources/", %(default_context_version)s)();
</script>
</body>
</html>
"""
  file.write(code % {
    'dimension': dimension,
    'element_type': element_type,
    'internal_format': internal_format,
    'format': format,
    'type': type,
    'default_context_version': default_context_version,
  })
  file.close()

def GenerateTests(test_dir, test_cases, dimension, default_context_version):
  test_dir_template = test_dir + '/%s'
  for element_type in _ELEMENT_TYPES:
    os.chdir(test_dir_template % element_type.replace('-', '_'))
    if dimension == '3':
      # Assume we write 2D tests first.
      index_file = open("00_test_list.txt", "ab")
    else:
      index_file = open("00_test_list.txt", "wb")
    for tex_info in test_cases:
      internal_format = tex_info['internal_format']
      format = tex_info['format']
      type = tex_info['type']
      filename = GenerateFilename(dimension, element_type, internal_format, format, type)
      index_file.write(filename)
      index_file.write('\n')
      WriteTest(filename, dimension, element_type, internal_format, format, type, default_context_version)
    index_file.close();

def main(argv):
  """This is the main function."""
  py_dir = os.path.dirname(os.path.realpath(__file__))
  GenerateTests(os.path.realpath(py_dir + '/../conformance/textures'), _FORMATS_TYPES_WEBGL1, '2', '1')
  GenerateTests(os.path.realpath(py_dir + '/../conformance2/textures'), _FORMATS_TYPES_WEBGL2, '2', '2')
  GenerateTests(os.path.realpath(py_dir + '/../conformance2/textures'), _FORMATS_TYPES_WEBGL2, '3', '2')

if __name__ == '__main__':
  sys.exit(main(sys.argv[1:]))
