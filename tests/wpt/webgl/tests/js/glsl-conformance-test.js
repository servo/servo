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
GLSLConformanceTester = (function(){

var wtu = WebGLTestUtils;
var defaultVertexShader = [
  "attribute vec4 vPosition;",
  "void main()",
  "{",
  "    gl_Position = vPosition;",
  "}"
].join('\n');

var defaultFragmentShader = [
  "precision mediump float;",
  "void main()",
  "{",
  "    gl_FragColor = vec4(1.0,0.0,0.0,1.0);",
  "}"
].join('\n');

var defaultESSL3VertexShader = [
  "#version 300 es",
  "in vec4 vPosition;",
  "void main()",
  "{",
  "    gl_Position = vPosition;",
  "}"
].join('\n');

var defaultESSL3FragmentShader = [
  "#version 300 es",
  "precision mediump float;",
  "out vec4 my_FragColor;",
  "void main()",
  "{",
  "    my_FragColor = vec4(1.0,0.0,0.0,1.0);",
  "}"
].join('\n');

function log(msg) {
  bufferedLogToConsole(msg);
}

var vShaderDB = {};
var fShaderDB = {};

/**
 * The info parameter should contain the following keys. Note that you may leave
 * the parameters for one shader out, in which case the default shader will be
 * used.
 * vShaderSource: the source code for vertex shader
 * vShaderId: id of an element containing vertex shader source code. Used if
 *   vShaderSource is not specified.
 * vShaderSuccess: true if vertex shader compilation should
 *   succeed.
 * fShaderSource: the source code for fragment shader
 * fShaderId: id of an element containing fragment shader source code. Used if
 *   fShaderSource is not specified.
 * fShaderSuccess: true if fragment shader compilation should
 *   succeed.
 * linkSuccess: true if link should succeed
 * passMsg: msg to describe success condition.
 * render: if true render to unit quad. Green = success
 * uniforms: an array of objects specifying uniforms to set prior to rendering.
 *   Each object should have the following keys:
 *     name: uniform variable name in the shader source. Uniform location will
 *       be queried based on its name.
 *     functionName: name of the function used to set the uniform. For example:
 *       'uniform1i'
 *     value: value of the uniform to set.
 */
function runOneTest(gl, info) {
  var passMsg = info.passMsg
  debug("");
  debug("test: " + passMsg);

  var consoleDiv = document.getElementById("console");

  var vIsDefault = false;
  var fIsDefault = false;

  if (info.vShaderSource === undefined) {
    if (info.vShaderId) {
      info.vShaderSource = document.getElementById(info.vShaderId).text;
    } else {
      vIsDefault = true;
    }
  }
  if (info.fShaderSource === undefined) {
    if (info.fShaderId) {
      info.fShaderSource = document.getElementById(info.fShaderId).text;
    } else {
      fIsDefault = true;
    }
  }

  var vLabel = (vIsDefault ? "default" : "test") + " vertex shader";
  var fLabel = (fIsDefault ? "default" : "test") + " fragment shader";
  if (vIsDefault) {
    info.vShaderSource = defaultVertexShader;
    info.vShaderSuccess = true;
  }
  if (fIsDefault) {
    info.fShaderSource = defaultFragmentShader;
    info.fShaderSuccess = true;
  }

  if (vIsDefault != fIsDefault) {
    // The language version of the default shader is chosen
    // according to the language version of the other shader.
    // We rely on "#version 300 es" being in this usual format.
    // It must be on the first line of the shader according to the spec.
    if (fIsDefault) {
      // If we're using the default fragment shader, we need to make sure that
      // it's language version matches with the vertex shader.
      if (info.vShaderSource.split('\n')[0] == '#version 300 es') {
        info.fShaderSource = defaultESSL3FragmentShader;
      }
    } else {
      // If we're using the default vertex shader, we need to make sure that
      // it's language version matches with the fragment shader.
      if (info.fShaderSource.split('\n')[0] == '#version 300 es') {
        info.vShaderSource = defaultESSL3VertexShader;
      }
    }
  }

  var vSource = info.vShaderPrep ? info.vShaderPrep(info.vShaderSource) :
    info.vShaderSource;

  if (!quietMode())
    wtu.addShaderSource(consoleDiv, vLabel, vSource);

  // Reuse identical shaders so we test shared shader.
  var vShader = vShaderDB[vSource];
  if (!vShader) {
    vShader = wtu.loadShader(gl, vSource, gl.VERTEX_SHADER);
    if (info.vShaderTest) {
      if (!info.vShaderTest(vShader)) {
        testFailed("[vertex shader test] " + passMsg);
        return;
      }
    }
    // As per GLSL 1.0.17 10.27 we can only check for success on
    // compileShader, not failure.
    if (!info.ignoreResults && info.vShaderSuccess && !vShader) {
      testFailed("[unexpected vertex shader compile status] (expected: " +
                 info.vShaderSuccess + ") " + passMsg);
    }
    // Save the shaders so we test shared shader.
    if (vShader) {
      vShaderDB[vSource] = vShader;
    }
  }

  var debugShaders = gl.getExtension('WEBGL_debug_shaders');
  if (debugShaders && vShader && !quietMode()) {
    wtu.addShaderSource(consoleDiv, vLabel + " translated for driver",
                        debugShaders.getTranslatedShaderSource(vShader));
  }

  var fSource = info.fShaderPrep ? info.fShaderPrep(info.fShaderSource) :
    info.fShaderSource;

  if (!quietMode())
    wtu.addShaderSource(consoleDiv, fLabel, fSource);

  // Reuse identical shaders so we test shared shader.
  var fShader = fShaderDB[fSource];
  if (!fShader) {
    fShader = wtu.loadShader(gl, fSource, gl.FRAGMENT_SHADER);
    if (info.fShaderTest) {
      if (!info.fShaderTest(fShader)) {
        testFailed("[fragment shader test] " + passMsg);
        return;
      }
    }
    //debug(fShader == null ? "fail" : "succeed");
    // As per GLSL 1.0.17 10.27 we can only check for success on
    // compileShader, not failure.
    if (!info.ignoreResults && info.fShaderSuccess && !fShader) {
      testFailed("[unexpected fragment shader compile status] (expected: " +
                info.fShaderSuccess + ") " + passMsg);
      return;
    }

    // Safe the shaders so we test shared shader.
    if (fShader) {
      fShaderDB[fSource] = fShader;
    }
  }

  if (debugShaders && fShader && !quietMode()) {
    wtu.addShaderSource(consoleDiv, fLabel + " translated for driver",
                        debugShaders.getTranslatedShaderSource(fShader));
  }

  if (vShader && fShader) {
    var program = gl.createProgram();
    gl.attachShader(program, vShader);
    gl.attachShader(program, fShader);

    if (vSource.indexOf("vPosition") >= 0) {
      gl.bindAttribLocation(program, 0, "vPosition");
    }
    if (vSource.indexOf("texCoord0") >= 0) {
      gl.bindAttribLocation(program, 1, "texCoord0");
    }
    gl.linkProgram(program);
    var linked = (gl.getProgramParameter(program, gl.LINK_STATUS) != 0);
    if (!linked) {
      var error = gl.getProgramInfoLog(program);
      log("*** Error linking program '"+program+"':"+error);
    }
    if (!info.ignoreResults && linked != info.linkSuccess) {
      testFailed("[unexpected link status] " + passMsg);
      return;
    }
  } else {
    if (!info.ignoreResults && info.linkSuccess) {
      testFailed("[link failed] " + passMsg);
      return;
    }
  }

  if (parseInt(wtu.getUrlOptions().dumpShaders)) {
    var vInfo = {
      shader: vShader,
      shaderSuccess: info.vShaderSuccess,
      label: vLabel,
      source: vSource
    };
    var fInfo = {
      shader: fShader,
      shaderSuccess: info.fShaderSuccess,
      label: fLabel,
      source: fSource
    };
    wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vInfo, fInfo);
  }

  if (!info.render) {
    testPassed(passMsg);
    return;
  }

  gl.useProgram(program);

  if (info.uniforms !== undefined) {
    for (var i = 0; i < info.uniforms.length; ++i) {
      var uniformLocation = gl.getUniformLocation(program, info.uniforms[i].name);
      gl[info.uniforms[i].functionName](uniformLocation, info.uniforms[i].value);
      debug(info.uniforms[i].name + ' set to ' + info.uniforms[i].value);
    }
  }

  wtu.setupUnitQuad(gl);
  wtu.clearAndDrawUnitQuad(gl);

  var div = document.createElement("div");
  div.className = "testimages";
  wtu.insertImage(div, "result", wtu.makeImageFromCanvas(gl.canvas));
  div.appendChild(document.createElement('br'));
  consoleDiv.appendChild(div);

  var tolerance = 0;
  if (info.renderTolerance !== undefined) {
    tolerance = info.renderTolerance;
  }
  wtu.checkCanvas(gl, [0, 255, 0, 255], "should be green", tolerance);
}

function runTests(shaderInfos, opt_contextVersion) {
  var wtu = WebGLTestUtils;
  var canvas = document.createElement('canvas');
  canvas.width = 32;
  canvas.height = 32;
  var gl = wtu.create3DContext(canvas, undefined, opt_contextVersion);
  if (!gl) {
    testFailed("context does not exist");
    finishTest();
    return;
  }

  var testIndex = 0;
  var runNextTest = function() {
    if (testIndex == shaderInfos.length) {
      finishTest();
      return;
    }

    runOneTest(gl, shaderInfos[testIndex++]);
    setTimeout(runNextTest, 1);
  }
  runNextTest();
};

function getSource(elem) {
  var str = elem.text;
  return str.replace(/^\s*/, '').replace(/\s*$/, '');
}

function getPassMessage(source) {
  var lines = source.split('\n');
  return lines[0].substring(3);
}

function getSuccess(msg) {
  if (msg.indexOf("fail") >= 0) {
    return false;
  }
  if (msg.indexOf("succeed") >= 0) {
    return true;
  }
  testFailed("bad test description. Must have 'fail' or 'succeed'");
}

function setupTest() {
  var info = {};

  var vShaderElem = document.getElementById('vertexShader');
  if (vShaderElem) {
    info.vShaderSource = getSource(vShaderElem);
    info.passMsg = getPassMessage(info.vShaderSource);
    info.vShaderSuccess = getSuccess(info.passMsg);
  }

  var fShaderElem = document.getElementById('fragmentShader');
  if (fShaderElem) {
    info.fShaderSource = getSource(fShaderElem);
    info.passMsg = getPassMessage(info.fShaderSource);
    info.fShaderSuccess = getSuccess(info.passMsg);
  }

  // linkSuccess should be true if shader success value is undefined or true for both shaders.
  info.linkSuccess = info.vShaderSuccess !== false && info.fShaderSuccess !== false;

  if (info.passMsg === undefined) {
    testFailed("no test shader found.");
    finishTest();
    return;
  }

  return info;
}

function runTest() {
  var info = setupTest();
  description(info.passMsg);
  runTests([info]);
}

function runRenderTests(tests, opt_contextVersion) {
  for (var ii = 0; ii < tests.length; ++ii) {
    tests[ii].render = true
  }
  runTests(tests, opt_contextVersion);
}

function runRenderTest() {
  var info = setupTest();
  description(info.passMsg);
  runRenderTests([info]);
}

return {
  runTest: runTest,
  runTests: runTests,
  runRenderTest: runRenderTest,
  runRenderTests: runRenderTests
};
}());
