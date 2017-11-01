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
OpenGLESTestRunner = (function(){
var wtu = WebGLTestUtils;
var gl;

var HALF_GRID_MAX_SIZE = 32;
var KNOWN_ATTRIBS = [
  "gtf_Vertex",
  "gtf_Color"
];

var GTFPIXELTOLERANCE = 24;
var GTFACCEPTABLEFAILURECONT = 10;
var GTFAMDPIXELTOLERANCE = 12;
var GTFSCORETOLERANCE = 0.65;
var GTFNCCTOLARANCEZERO = 0.25;
var GTFKERNALSIZE = 5;

function log(msg) {
  // debug(msg);
}

function compareImages(refData, tstData, width, height, diff) {
  function isPixelSame(offset) {
    // First do simple check
    if (Math.abs(refData[offset + 0] - tstData[offset + 0]) <= GTFPIXELTOLERANCE &&
        Math.abs(refData[offset + 1] - tstData[offset + 1]) <= GTFPIXELTOLERANCE &&
        Math.abs(refData[offset + 2] - tstData[offset + 2]) <= GTFPIXELTOLERANCE) {
      return true;
    }

    // TODO: Implement crazy check that's used in OpenGL ES 2.0 conformance tests.
    // NOTE: on Desktop things seem to be working. Maybe the more complex check
    // is needed for embedded systems?
    return false;
  }

  var same = true;
  for (var yy = 0; yy < height; ++yy) {
    for (var xx = 0; xx < width; ++xx) {
      var offset = (yy * width + xx) * 4;
      var diffOffset = ((height - yy - 1) * width + xx) * 4;
      diff[diffOffset + 0] = 0;
      diff[diffOffset + 1] = 0;
      diff[diffOffset + 2] = 0;
      diff[diffOffset + 3] = 255;
      if (!isPixelSame(offset)) {
        diff[diffOffset] = 255;
        if (same) {
          same = false;
          testFailed("pixel @ (" + xx + ", " + yy + " was [" +
                     tstData[offset + 0] + "," +
                     tstData[offset + 1] + "," +
                     tstData[offset + 2] + "," +
                     tstData[offset + 3] + "] expected [" +
                     refData[offset + 0] + "," +
                     refData[offset + 1] + "," +
                     refData[offset + 2] + "," +
                     refData[offset + 3] + "]")
        }
      }
    }
  }
  return same;
}

function persp(fovy, aspect, n, f) {
  var dz = f - n;
  var rad = fovy / 2.0 * 3.14159265 / 180;

  var s = Math.sin(rad);
  if (dz == 0 || s == 0 || aspect == 0)
    return;

  var cot = Math.cos(rad) / s;

  return [
    cot / aspect,
    0.0,
    0.0,
    0.0,

    0.0,
    cot,
    0.0,
    0.0,

    0.0,
    0.0,
    -(f + n) / dz,
    -1.0,

    0.0,
    0.0,
    -2.0 * f * n / dz,
    0.0
  ];
}

function setAttribs(attribs, buffers) {
  for (var name in attribs) {
    var buffer = buffers[name];
    if (!buffer) {
      testFailed("no buffer for attrib:" + name);
      continue;
    }
    var loc = attribs[name];
    log("setup attrib: " + loc + " as " + name);
    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(buffer.data), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, buffer.numComponents, gl.FLOAT, false, 0, 0);
  }
}

function drawSquare(attribs) {
  var buffers = {
    "gtf_Vertex": {
      data: [
        1.0, -1.0, -2.0,
        1.0, 1.0, -2.0,
        -1.0, -1.0, -2.0,
        -1.0, 1.0, -2.0
      ],
      numComponents: 3
    },
    "gtf_Color": {
      data: [
        0.5, 1.0, 0.0,
        0.0, 1.0, 1.0,
        1.0, 0.0, 0.0,
        0.5, 0.0, 1.0
      ],
      numComponents: 3,
    },
    "gtf_SecondaryColor": {
      data: [
        0.5, 0.0, 1.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 1.0,
        0.5, 1.0, 0.0
      ],
      numComponents: 3,
    },
    "gtf_Normal": {
      data: [
        0.5, 0.0, 1.0,
        1.0, 0.0, 0.0,
        0.0, 1.0, 1.0,
        0.5, 1.0, 0.0
      ],
      numComponents: 3,
    },
    "gtf_MultiTexCoord0": {
      data: [
        1.0, 0.0,
        1.0, 1.0,
        0.0, 0.0,
        0.0, 1.0
      ],
      numComponents: 2,
    },
    "gtf_FogCoord": {
      data: [
        0.0,
        1.0,
        0.0,
        1.0
      ],
      numComponents: 1,
    }
  };
  setAttribs(attribs, buffers);
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
}

function drawFrontBackSquare(attribs) {
  var front = {
    "gtf_Vertex": {
      data: [
        1.0, -1.0, -2.0,
        1.0, 0.0, -2.0,
        -1.0, -1.0, -2.0,
        -1.0, 0.0, -2.0
      ],
      numComponents: 3
    },
    "gtf_Color": {
      data: [
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 1.0, 0.0
      ],
      numComponents: 3,
    },
    "gtf_MultiTexCoord0": {
      data: [
        1.0, 0.0,
        1.0, 0.5,
        0.0, 0.0,
        0.0, 0.5
      ],
      numComponents: 2,
    }
  };
  setAttribs(attribs, front);
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);

  var back = {
    "gtf_Vertex": {
      data: [
        1.0, 1.0, -2.0,
        1.0, 0.0, -2.0,
        -1.0, 1.0, -2.0,
        -1.0, 0.0, -2.0
      ],
      numComponents: 3
    },
    "gtf_Color": {
      data: [
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0,
        1.0, 0.0, 0.0
      ],
      numComponents: 3,
    },
    "gtf_MultiTexCoord0": {
      data: [
        1.0, 0.1,
        1.0, 0.5,
        0.0, 0.1,
        0.0, 0.5
      ],
      numComponents: 2,
    }
  };
  setAttribs(attribs, back);
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
}

function drawGrid(attribs, width, height) {
  var n = Math.min(Math.floor(Math.max(width, height) / 4), HALF_GRID_MAX_SIZE);

  var numVertices = (n + n) * (n + n) * 6;

  var gridVertices   = [];
  var gridColors     = [];
  var gridSecColors  = [];
  var gridNormals    = [];
  var gridFogCoords  = [];
  var gridTexCoords0 = [];

  var currentVertex    = 0;
  var currentColor     = 0;
  var currentSecColor  = 0;
  var currentTexCoord0 = 0;
  var currentNormal    = 0;
  var currentFogCoord  = 0;

  var z = -2.0;
  for(var i = -n; i < n; ++i)
  {
      var x1 = i / n;
      var x2 = (i + 1) / n;
      for(var j = -n; j < n; ++j)
      {
          var y1 = j / n;
          var y2 = (j + 1) / n;

          // VERTEX 0
          gridVertices[currentVertex++] = x1;
          gridVertices[currentVertex++] = y1;
          gridVertices[currentVertex++] = z;
          gridColors[currentColor++] = 1.0 - (x1 + y1 + 2.0) / 4.0;
          gridColors[currentColor++] = (x1 + 1.0) / 2.0;
          gridColors[currentColor++] = (y1 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = 1.0 - (x2 + y2 + 2.0) / 4.0;
          gridSecColors[currentSecColor++] = (x2 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = (y2 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (x1 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (y1 + 1.0) / 2.0;
          gridNormals[currentNormal++] = 1.0 - (x2 + y2 + 2.0) / 4.0;
          gridNormals[currentNormal++] = (x2 + 1.0) / 2.0;
          gridNormals[currentNormal++] = (y2 + 1.0) / 2.0;
          gridFogCoords[currentFogCoord++] = (y1 + 1.0) / 2.0;

          // VERTEX 1
          gridVertices[currentVertex++] = x2;
          gridVertices[currentVertex++] = y1;
          gridVertices[currentVertex++] = z;
          gridColors[currentColor++] = 1.0 - (x2 + y1 + 2.0) / 4.0;
          gridColors[currentColor++] = (x2 + 1.0) / 2.0;
          gridColors[currentColor++] = (y1 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = 1.0 - (x1 + y2 + 2.0) / 4.0;
          gridSecColors[currentSecColor++] = (x1 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = (y2 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (x2 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (y1 + 1.0) / 2.0;
          gridNormals[currentNormal++] = 1.0 - (x1 + y2 + 2.0) / 4.0;
          gridNormals[currentNormal++] = (x1 + 1.0) / 2.0;
          gridNormals[currentNormal++] = (y2 + 1.0) / 2.0;
          gridFogCoords[currentFogCoord++] = (y1 + 1.0) / 2.0;

          // VERTEX 2
          gridVertices[currentVertex++] = x2;
          gridVertices[currentVertex++] = y2;
          gridVertices[currentVertex++] = z;
          gridColors[currentColor++] = 1.0 - (x2 + y2 + 2.0) / 4.0;
          gridColors[currentColor++] = (x2 + 1.0) / 2.0;
          gridColors[currentColor++] = (y2 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = 1.0 - (x1 + y1 + 2.0) / 4.0;
          gridSecColors[currentSecColor++] = (x1 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = (y1 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (x2 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (y2 + 1.0) / 2.0;
          gridNormals[currentNormal++] = 1.0 - (x1 + y1 + 2.0) / 4.0;
          gridNormals[currentNormal++] = (x1 + 1.0) / 2.0;
          gridNormals[currentNormal++] = (y1 + 1.0) / 2.0;
          gridFogCoords[currentFogCoord++] = (y2 + 1.0) / 2.0;

          // VERTEX 2
          gridVertices[currentVertex++] = x2;
          gridVertices[currentVertex++] = y2;
          gridVertices[currentVertex++] = z;
          gridColors[currentColor++] = 1.0 - (x2 + y2 + 2.0) / 4.0;
          gridColors[currentColor++] = (x2 + 1.0) / 2.0;
          gridColors[currentColor++] = (y2 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = 1.0 - (x1 + y1 + 2.0) / 4.0;
          gridSecColors[currentSecColor++] = (x1 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = (y1 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (x2 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (y2 + 1.0) / 2.0;
          gridNormals[currentNormal++] = 1.0 - (x1 + y1 + 2.0) / 4.0;
          gridNormals[currentNormal++] = (x1 + 1.0) / 2.0;
          gridNormals[currentNormal++] = (y1 + 1.0) / 2.0;
          gridFogCoords[currentFogCoord++] = (y2 + 1.0) / 2.0;

          // VERTEX 3
          gridVertices[currentVertex++] = x1;
          gridVertices[currentVertex++] = y2;
          gridVertices[currentVertex++] = z;
          gridColors[currentColor++] = 1.0 - (x1 + y2 + 2.0) / 4.0;
          gridColors[currentColor++] = (x1 + 1.0) / 2.0;
          gridColors[currentColor++] = (y2 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = 1.0 - (x2 + y1 + 2.0) / 4.0;
          gridSecColors[currentSecColor++] = (x2 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = (y1 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (x1 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (y2 + 1.0) / 2.0;
          gridNormals[currentNormal++] = 1.0 - (x2 + y1 + 2.0) / 4.0;
          gridNormals[currentNormal++] = (x2 + 1.0) / 2.0;
          gridNormals[currentNormal++] = (y1 + 1.0) / 2.0;
          gridFogCoords[currentFogCoord++] = (y2 + 1.0) / 2.0;

          // VERTEX 0
          gridVertices[currentVertex++] = x1;
          gridVertices[currentVertex++] = y1;
          gridVertices[currentVertex++] = z;
          gridColors[currentColor++] = 1.0 - (x1 + y1 + 2.0) / 4.0;
          gridColors[currentColor++] = (x1 + 1.0) / 2.0;
          gridColors[currentColor++] = (y1 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = 1.0 - (x2 + y2 + 2.0) / 4.0;
          gridSecColors[currentSecColor++] = (x2 + 1.0) / 2.0;
          gridSecColors[currentSecColor++] = (y2 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (x1 + 1.0) / 2.0;
          gridTexCoords0[currentTexCoord0++] = (y1 + 1.0) / 2.0;
          gridNormals[currentNormal++] = 1.0 - (x2 + y2 + 2.0) / 4.0;
          gridNormals[currentNormal++] = (x2 + 1.0) / 2.0;
          gridNormals[currentNormal++] = (y2 + 1.0) / 2.0;
          gridFogCoords[currentFogCoord++] = (y1 + 1.0) / 2.0;
      }
  }

  var buffers = {
    "gtf_Vertex":         { data: gridVertices,   numComponents: 3 },
    "gtf_Color":          { data: gridColors,     numComponents: 3 },
    "gtf_SecondaryColor": { data: gridSecColors,  numComponents: 3 },
    "gtf_Normal":         { data: gridNormals,    numComponents: 3 },
    "gtf_FogCoord":       { data: gridFogCoords,  numComponents: 1 },
    "gtf_MultiTexCoord0": { data: gridTexCoords0, numComponents: 2 }
  };
  setAttribs(attribs, buffers);
  gl.drawArrays(gl.TRIANGLES, 0, numVertices);
}

var MODEL_FUNCS = {
  square:          drawSquare,
  frontbacksquare: drawFrontBackSquare,
  grid:            drawGrid
};

function drawWithProgram(program, programInfo, test) {
  gl.useProgram(program);
  var attribs = { };

  var numAttribs = gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES);
  for (var ii = 0; ii < numAttribs; ++ii) {
    var info = gl.getActiveAttrib(program, ii);
    var name = info.name;
    var location = gl.getAttribLocation(program, name);
    attribs[name] = location;

    if (KNOWN_ATTRIBS.indexOf(name) < 0) {
      testFailed("unknown attrib:" + name)
    }
  }

  var uniforms = { };
  var numUniforms = gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS);
  for (var ii = 0; ii < numUniforms; ++ii) {
    var info = gl.getActiveUniform(program, ii);
    var name = info.name;
    if (name.match(/\[0\]$/)) {
      name = name.substr(0, name.length - 3);
    }
    var location = gl.getUniformLocation(program, name);
    uniforms[name] = {location: location};
  }

  var getUniformLocation = function(name) {
    var uniform = uniforms[name];
    if (uniform) {
      uniform.used = true;
      return uniform.location;
    }
    return null;
  }

  // Set known uniforms
  var loc = getUniformLocation("gtf_ModelViewProjectionMatrix");
  if (loc)  {
    gl.uniformMatrix4fv(
      loc,
      false,
      persp(60, 1, 1, 30));
  }
  var loc = getUniformLocation("viewportwidth");
  if (loc) {
    gl.uniform1f(loc, gl.canvas.width);
  }
  var loc = getUniformLocation("viewportheight");
  if (loc) {
    gl.uniform1f(loc, gl.canvas.height);
  }

  // Set test specific uniforms
  for (var name in programInfo.uniforms) {
    var location = getUniformLocation(name);
    if (!location) {
      continue;
    }
    var uniform = programInfo.uniforms[name];
    var type = uniform.type;
    var value = uniform.value;
    var transpose = uniform.transpose;
    if (transpose !== undefined) {
      log("gl." + type + '("' + name + '", ' + transpose + ", " + value + ")");
      gl[type](location, transpose, value);
    } else if (!type.match("v$")) {
      var args = [location];
      for (var ii = 0; ii < value.length; ++ii) {
        args.push(value[ii]);
      }
      gl[type].apply(gl, args);
      log("gl." + type + '("' + name + '", ' + args.slice(1) + ")");
    } else {
      log("gl." + type + '("' + name + '", ' + value + ")");
      gl[type](location, value);
    }
    var err = gl.getError();
    if (err != gl.NO_ERROR) {
      testFailed(wtu.glEnumToString(gl, err) + " generated setting uniform: " + name);
    }
  }

  // Filter out specified built-in uniforms
  if (programInfo.builtin_uniforms) {
    var num_builtins_found = 0;
    var valid_values = programInfo.builtin_uniforms.valid_values;
    for (var index in valid_values) {
      var uniform = uniforms[valid_values[index]];
      if (uniform) {
        ++num_builtins_found;
        uniform.builtin = true;
      }
    }

    var min_required = programInfo.builtin_uniforms.min_required;
    if (num_builtins_found < min_required) {
      testFailed("only found " + num_builtins_found + " of " + min_required +
                 " required built-in uniforms: " + valid_values);
    }
  }

  // Check for unset uniforms
  for (var name in uniforms) {
    var uniform = uniforms[name];
    if (!uniform.used && !uniform.builtin) {
      testFailed("uniform " + name + " never set");
    }
  }


  for (var state in test.state) {
    var fields = test.state[state];
    switch (state) {
    case 'depthrange':
      gl.depthRange(fields.near, fields.far);
      break;
    default:
      testFailed("unknown state: " + state)
    }
  }

  gl.clearColor(0, 0, 0, 0);
  gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

  var model = test.model || "square";
  var fn = MODEL_FUNCS[model];
  if (!fn) {
    testFailed("unknown model type: " + model)
  } else {
    log("draw as: " + model)
    fn(attribs, gl.canvas.width, gl.canvas.height);
  }

  var pixels = new Uint8Array(gl.canvas.width * gl.canvas.height * 4);
  gl.readPixels(0, 0, gl.canvas.width, gl.canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  return {
    width: gl.canvas.width,
    height: gl.canvas.height,
    pixels: pixels,
    img: wtu.makeImageFromCanvas(gl.canvas)
  };
}

function runProgram(programInfo, test, label, callback) {
  var shaders = [];
  var source = [];
  var count = 0;

  function loadShader(path, type, index) {
    wtu.loadTextFileAsync(path, function(success, text) {
      addShader(success, text, type, path, index);
    });
  }

  function addShader(success, text, type, path, index) {
    ++count;
    if (!success) {
      testFailed("could not load: " + path);
    } else {
      var shader = wtu.loadShader(gl, text, type);
      shaders.push(shader);
      source[index] = text;
    }
    if (count == 2) {
      var result;
      if (shaders.length == 2) {
        debug("");
        if (!quietMode()) {
        var consoleDiv = document.getElementById("console");
          wtu.addShaderSources(
              gl, consoleDiv, label + " vertex shader", shaders[0], source[0],
              programInfo.vertexShader);
          wtu.addShaderSources(
              gl, consoleDiv, label + " fragment shader", shaders[1], source[1],
              programInfo.fragmentShader);
        }
        var program = wtu.createProgram(gl, shaders[0], shaders[1]);
        result = drawWithProgram(program, programInfo, test);
      }
      callback(result);
    }
  }

  loadShader(programInfo.vertexShader, gl.VERTEX_SHADER, 0);
  loadShader(programInfo.fragmentShader, gl.FRAGMENT_SHADER, 1);
}

function compareResults(expected, actual) {
  var width = expected.width;
  var height = expected.height;
  var canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  var ctx = canvas.getContext("2d");
  var imgData = ctx.getImageData(0, 0, width, height);
  var tolerance = 0;

  var expData = expected.pixels;
  var actData = actual.pixels;

  var same = compareImages(expData, actData, width, height, imgData.data);

  var console = document.getElementById("console");
  var diffImg = null;
  if (!same) {
    ctx.putImageData(imgData, 0, 0);
    diffImg = wtu.makeImageFromCanvas(canvas);
  }

  if (!quietMode()) {
    var div = document.createElement("div");
    div.className = "testimages";
    wtu.insertImage(div, "reference", expected.img);
    wtu.insertImage(div, "test", actual.img);
    if (diffImg) {
      wtu.insertImage(div, "diff", diffImg);
    }
    div.appendChild(document.createElement('br'));

    console.appendChild(div);
  }

  if (!same) {
    testFailed("images are different");
  } else {
    testPassed("images are the same");
  }

  if (!quietMode())
    console.appendChild(document.createElement('hr'));
}

function runCompareTest(test, callback) {
  debug("");
  debug("test: " + test.name);
  var results = [];
  var count = 0;

  function storeResults(index) {
    return function(result) {
      results[index] = result;
      ++count;
      if (count == 2) {
        compareResults(results[0], results[1]);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "there should be no errors");
        callback();
      }
    }
  }

  runProgram(test.referenceProgram, test, "reference", storeResults(0));
  runProgram(test.testProgram, test, "test", storeResults(1));
}

function runBuildTest(test, callback) {
  debug("");
  debug("test: " + test.name);

  var shaders = [null, null];
  var source = ["",""];
  var success = [undefined, undefined];
  var count = 0;

  function loadShader(path, type, index) {
    if (path == "empty") {
      shaders[index] = gl.createShader();
      success[index] = true;
      source[index] = "/* empty */";
      attachAndLink();
    } else {
      wtu.loadTextFileAsync(path, function(loadSuccess, text) {
        if (!loadSuccess) {
          success[index] = false;
          source[index] = "/* could not load */";
          testFailed("could not load:" + path);
        } else {
          source[index] = text;
          shaders[index] = wtu.loadShader(gl, text, type, function(index) {
            return function(msg) {
              success[index] = false
            }
          }(index));
          if (success[index] === undefined) {
            success[index] = true;
          }
        }
        attachAndLink();
      });
    }
  }

  function attachAndLink() {
    ++count;
    if (count == 2) {
      if (!quietMode()) {
        debug("");
        var c = document.getElementById("console");
        wtu.addShaderSource(
            c, "vertex shader", source[0], test.testProgram.vertexShader);
        debug("compile: " + (success[0] ? "success" : "fail"));
        wtu.addShaderSource(
            c, "fragment shader", source[1], test.testProgram.fragmentShader);
        debug("compile: " + (success[1] ? "success" : "fail"));
      }
      compileSuccess = (success[0] && success[1]);
      if (!test.compstat) {
        if (compileSuccess) {
          testFailed("expected compile failure but was successful");
        } else {
          testPassed("expected compile failure and it failed");
        }
      } else {
        if (compileSuccess) {
          testPassed("expected compile success and it was successful");
        } else {
          testFailed("expected compile success but it failed");
        }
        var linkSuccess = true;
        var program = wtu.createProgram(gl, shaders[0], shaders[1], function() {
          linkSuccess = false;
        });
        if (linkSuccess !== test.linkstat) {
          testFailed("expected link to " + (test.linkstat ? "succeed" : "fail"));
        } else {
          testPassed("shaders compiled and linked as expected.");
        }
      }
      callback();
    }
  }

  loadShader(test.testProgram.vertexShader, gl.VERTEX_SHADER, 0);
  loadShader(test.testProgram.fragmentShader, gl.FRAGMENT_SHADER, 1);
}

var testPatterns = {
  compare: runCompareTest,
  build: runBuildTest,

  dummy: null  // just here to mark the end
};

function LogGLCall(functionName, args) {
  console.log("gl." + functionName + "(" +
            WebGLDebugUtils.glFunctionArgsToString(functionName, args) + ")");
}

// Runs the tests async since they will load shaders.
function run(obj) {
  description();

  var canvas = document.getElementById("example");
  gl = wtu.create3DContext(canvas);
  if (window.WebGLDebugUtils) {
    gl = WebGLDebugUtils.makeDebugContext(gl, undefined, LogGLCall);
  }
  if (!gl) {
    testFailed("context does not exist");
    finishTest();
    return;
  }

  if (gl.canvas.width != 500 || gl.canvas.height != 500) {
    testFailed("canvas must be 500x500 pixels: Several shaders are hard coded to this size.");
  }

  var tests = obj.tests;
  var ndx = 0;

  function runNextTest() {
    if (ndx < tests.length) {
      var test = tests[ndx++];
      var fn = testPatterns[test.pattern];
      if (!fn) {
        testFailed("test pattern: " + test.pattern + " not supoprted")
        runNextTest();
      } else {
        fn(test, runNextTest);
      }
    } else {
      finishTest();
    }
  }
  runNextTest();
}

return {
  run: run,
};
}());

