/*
** Copyright (c) 2014 The Khronos Group Inc.
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

'use strict';

// ESSL 1.00 spec section 5.8 (also ESSL 3.00 spec section 5.8):
// "The l-value and the expression must satisfy the semantic requirements of both op and equals (=)"
// In the semantic requirements of assignment (=):
// "The lvalue-expression and rvalue-expression must have the same type"

var runTest = function(contextVersion) {
  var vertexTemplateESSL1 = [
    'precision mediump float;',

    'uniform $(rtype) ur;',
    'uniform $(ltype) ul;',

    'void main() {',
    '    $(ltype) a = ul;',
    '    a $(op) ur;',
    '    gl_Position = vec4(float(a$(ltypeToScalar)));',
    '}'
  ].join('\n');

  var vertexTemplateESSL3 = [
    '#version 300 es',
    vertexTemplateESSL1
  ].join('\n');

  var fragmentTemplateESSL1 = [
    'precision mediump float;',

    'uniform $(rtype) ur;',
    'uniform $(ltype) ul;',

    'void main() {',
    '    $(ltype) a = ul;',
    '    a $(op) ur;',
    '    gl_FragColor = vec4(float(a$(ltypeToScalar)));',
    '}'
  ].join('\n');

  var fragmentTemplateESSL3 = [
    '#version 300 es',
    'out mediump vec4 my_FragColor;',
    fragmentTemplateESSL1
  ].join('\n').replace('gl_FragColor', 'my_FragColor');

  var isNonSquareMatrix = function(typeStr) {
    return typeStr.substring(0, 3) == 'mat' &&
           typeStr.length > 5 &&
           typeStr[3] != typeStr[5];
  }

  var vsTemplate = contextVersion < 2 ? vertexTemplateESSL1 : vertexTemplateESSL3;
  var fsTemplate = contextVersion < 2 ? fragmentTemplateESSL1 : fragmentTemplateESSL3;

  var wtu = WebGLTestUtils;

  var tests = [];

  var baseTypes = ['float', 'int'];
  var vecTypes = [['vec2', 'vec3', 'vec4', 'mat2', 'mat3', 'mat4'], ['ivec2', 'ivec3', 'ivec4']];
  if (contextVersion >= 2) {
    vecTypes[0] = ['vec2', 'vec3', 'vec4', 'mat2x2', 'mat3x3', 'mat4x4', 'mat2x3', 'mat2x4', 'mat3x2', 'mat3x4', 'mat4x2', 'mat4x3'];
  }
  var ops = ['+=', '-=', '*=', '/='];

  var fs, vs;
  for (var k = 0; k < ops.length; ++k) {
    var op = ops[k];
    for (var i = 0; i < baseTypes.length; ++i) {
      var baseType = baseTypes[i];
      for (var j = 0; j < vecTypes[i].length; ++j) {
        var vecType = vecTypes[i][j];
        var vecTypeToScalar = vecType.substring(0, 3) == 'mat' ? '[0].x' : '.x';

        var pushTest = function(ltype, rtype, ltypeToScalar, expectSuccess) {
            vs = wtu.replaceParams(vsTemplate, {ltype: ltype, rtype: rtype, ltypeToScalar: ltypeToScalar, op: op});
            fs = wtu.replaceParams(fsTemplate, {ltype: ltype, rtype: rtype, ltypeToScalar: ltypeToScalar, op: op});
            tests.push({
                vShaderSource: vs,
                vShaderSuccess: expectSuccess,
                linkSuccess: expectSuccess,
                passMsg: ltype + " " + op + " " + rtype + " in a vertex shader should " + (expectSuccess ? "succeed." : "fail.")
            });
            tests.push({
                fShaderSource: fs,
                fShaderSuccess: expectSuccess,
                linkSuccess: expectSuccess,
                passMsg: ltype + " " + op + " " + rtype + " in a fragment shader should " + (expectSuccess ? "succeed." : "fail.")
            });
        }

        // "scalar op= vector" is not okay, since the result of op is a vector,
        // which can't be assigned to a scalar.
        pushTest(baseType, vecType, '', false);

        if (j > 0) {
            var vecType2 = vecTypes[i][j - 1];
            // "vector1 op= vector2" is not okay when vector1 and vector2 have
            // non-matching dimensions.
            pushTest(vecType, vecType2, vecTypeToScalar, false);
        }

        // "vector op= scalar" is okay.
        pushTest(vecType, baseType, vecTypeToScalar, true);

        // vecX *= matX is okay (effectively, this treats vector as a row vector).
        if (vecType.substring(0, 3) == 'vec' && op == '*=') {
            pushTest(vecType, 'mat' + vecType[3], vecTypeToScalar, true);
        }

        if (op != '*=' || !isNonSquareMatrix(vecType)) {
            // "vector1 op= vector2" is okay when vector1 and vector2 have the same
            // type (does a component-wise operation or matrix multiplication).
            pushTest(vecType, vecType, vecTypeToScalar, true);
        } else {
            // non-square matrices can only be compound multiplied with a square matrix.
            pushTest(vecType, vecType, vecTypeToScalar, false);
            pushTest(vecType, 'mat' + vecType[3], vecTypeToScalar, true);
        }
      }
    }
  }

  GLSLConformanceTester.runTests(tests, contextVersion);
}

var successfullyParsed = true;
