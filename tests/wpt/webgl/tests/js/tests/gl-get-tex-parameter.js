/*
** Copyright (c) 2015 The Khronos Group Inc.
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

// This test relies on the surrounding web page defining a variable
// "contextVersion" which indicates what version of WebGL it's running
// on -- 1 for WebGL 1.0, 2 for WebGL 2.0, etc.

"use strict";
description();
var wtu = WebGLTestUtils;
var gl = wtu.create3DContext("example", undefined, contextVersion);

// NOTE: We explicitly do this in a funky order
// to hopefully find subtle bugs.

var targets = [
  'TEXTURE_2D',
  'TEXTURE_2D',
  'TEXTURE_CUBE_MAP',
  'TEXTURE_CUBE_MAP'
];

if (contextVersion > 1) {
  targets = targets.concat([
    'TEXTURE_2D_ARRAY',
    'TEXTURE_2D_ARRAY',
    'TEXTURE_3D',
    'TEXTURE_3D'
  ]);
}

// Create textures on different active textures.
for (var ii = 0; ii < targets.length; ++ii) {
  var target = targets[ii];
  var tex = gl.createTexture();
  gl.activeTexture(gl.TEXTURE0 + ii);
  gl.bindTexture(gl[target], tex);
}

wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");

var states = [
  { state: 'TEXTURE_WRAP_S',            default: 'REPEAT',                 value1: 'CLAMP_TO_EDGE',           value2: 'REPEAT'          },
  { state: 'TEXTURE_WRAP_T',            default: 'REPEAT',                 value1: 'MIRRORED_REPEAT',         value2: 'REPEAT'          },
  { state: 'TEXTURE_MAG_FILTER',        default: 'LINEAR',                 value1: 'NEAREST',                 value2: 'LINEAR'          },
  { state: 'TEXTURE_MIN_FILTER',        default: 'NEAREST_MIPMAP_LINEAR',  value1: 'LINEAR_MIPMAP_LINEAR',    value2: 'NEAREST'         }
];

if (contextVersion > 1) {
  states = states.concat([
    { state: 'TEXTURE_WRAP_R',            default: 'REPEAT',                 value1: 'CLAMP_TO_EDGE',           value2: 'MIRRORED_REPEAT' },
    { state: 'TEXTURE_COMPARE_FUNC',      default: 'LEQUAL',                 value1: 'GREATER',                 value2: 'LESS'            },
    { state: 'TEXTURE_COMPARE_MODE',      default: 'NONE',                   value1: 'COMPARE_REF_TO_TEXTURE',  value2: 'NONE'            },
    { state: 'TEXTURE_BASE_LEVEL',        default: 0,                        value1: 100,                       value2: 99                },
    { state: 'TEXTURE_MAX_LEVEL',         default: 1000,                     value1: 800,                       value2: 300               },
    { state: 'TEXTURE_MIN_LOD',           default: -1000.0,                  value1: -500.0,                    value2: -999.0            },
    { state: 'TEXTURE_MAX_LOD',           default: 1000.0,                   value1: 500.0,                     value2: 999.0             },
    // Note: For TEXTURE_IMMUTABLE_LEVELS and TEXTURE_IMMUTABLE_FORMAT,
    // these two pname are used by getTexParameter API only, not available in texParameter[fi] in specifications.
    // Thus, these two states store default value only.
    { state: 'TEXTURE_IMMUTABLE_LEVELS',  default: 0,     },
    { state: 'TEXTURE_IMMUTABLE_FORMAT',  default: false, }
  ]);
}

function getStateInfoValue(stateInfo, item, method) {
  switch (stateInfo.state) {
  case 'TEXTURE_WRAP_R':
  case 'TEXTURE_WRAP_S':
  case 'TEXTURE_WRAP_T':
  case 'TEXTURE_MAG_FILTER':
  case 'TEXTURE_MIN_FILTER':
  case 'TEXTURE_COMPARE_FUNC':
  case 'TEXTURE_COMPARE_MODE':
      if (method === 'Get') {
        return 'gl["' + stateInfo[item] + '"]';
      } else if (method === 'Set') {
        return gl[stateInfo[item]];
      }
      break;
  case 'TEXTURE_BASE_LEVEL':
  case 'TEXTURE_MAX_LEVEL':
  case 'TEXTURE_MIN_LOD':
  case 'TEXTURE_MAX_LOD':
      if (method === 'Get') {
        return '' + stateInfo[item];
      } else if (method === 'Set') {
        return stateInfo[item];
      }
      break;
  case 'TEXTURE_IMMUTABLE_LEVELS':
  case 'TEXTURE_IMMUTABLE_FORMAT':
      // Return default value only.
      return '' + stateInfo.default;
  default:
      wtu.error("Not reached!");
      return null;
      break;
  }
}

function applyStates(fn) {
  for (var ss = 0; ss < states.length; ++ss) {
    var stateInfo = states[ss];
    for (var ii = 0; ii < targets.length; ++ii) {
      var target = targets[ii];
      gl.activeTexture(gl.TEXTURE0 + ii);
      fn(target, stateInfo);
    }
  }
}

// test the default state.
applyStates(function(target, stateInfo) {
  var a = 'gl.getTexParameter(gl["' + target + '"], gl["' + stateInfo.state + '"])';
  var b = getStateInfoValue(stateInfo, 'default', 'Get');
  shouldBe(a, b);
});

// test new state
applyStates(function(target, stateInfo) {
  switch (stateInfo.state) {
  case 'TEXTURE_IMMUTABLE_FORMAT':
  case 'TEXTURE_IMMUTABLE_LEVELS':
      // Skip these two pname for texParameterf[fi].
      break;
  case 'TEXTURE_MIN_LOD':
  case 'TEXTURE_MAX_LOD':
      gl.texParameterf(gl[target], gl[stateInfo.state], getStateInfoValue(stateInfo, 'value1', 'Set'));
      break;
  default:
      gl.texParameteri(gl[target], gl[stateInfo.state], getStateInfoValue(stateInfo, 'value1', 'Set'));
      break;
  }
});

applyStates(function(target, stateInfo) {
  var a = 'gl.getTexParameter(gl["' + target + '"], gl["' + stateInfo.state + '"])';
  var b = getStateInfoValue(stateInfo, 'value1', 'Get');
  shouldBe(a, b);
});

// test different states on each target.
function getItem(count) {
  return (count % 2) ? 'value2' : 'value1';
}

applyStates(function() {
  var count = 0;
  return function(target, stateInfo) {
    switch (stateInfo.state) {
    case 'TEXTURE_IMMUTABLE_FORMAT':
    case 'TEXTURE_IMMUTABLE_LEVELS':
        // Skip these two pname for texParameterf[fi].
        break;
    case 'TEXTURE_MIN_LOD':
    case 'TEXTURE_MAX_LOD':
        gl.texParameterf(gl[target], gl[stateInfo.state], getStateInfoValue(stateInfo, getItem(count), 'Set'));
        break;
    default:
        gl.texParameteri(gl[target], gl[stateInfo.state], getStateInfoValue(stateInfo, getItem(count), 'Set'));
        break;
    }
    ++count;
  }
}());

applyStates(function() {
  var count = 0;
  return function(target, stateInfo) {
    var a = 'gl.getTexParameter(gl["' + target + '"], gl["' + stateInfo.state + '"])';
    var b = getStateInfoValue(stateInfo, getItem(count), 'Get');
    shouldBe(a, b);
    ++count;
  };
}());

wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");

var successfullyParsed = true;
