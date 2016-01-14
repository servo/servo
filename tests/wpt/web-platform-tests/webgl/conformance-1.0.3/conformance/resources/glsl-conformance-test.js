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

function log(msg) {
  if (window.console && window.console.log) {
    window.console.log(msg);
  }
}

var vShaderDB = {};
var fShaderDB = {};

/**
 * vShaderSource: the source code for vertex shader
 * vShaderSuccess: true if vertex shader compilation should
 *   succeed.
 * fShaderSource: the source code for fragment shader
 * fShaderSuccess: true if fragment shader compilation should
 *   succeed.
 * linkSuccess: true of link should succeed
 * passMsg: msg to describe success condition.
 * render: if true render to unit quad. Green = success
 *
 */
function runOneTest(gl, info) {
  var passMsg = info.passMsg
  debug("");
  debug("test: " + passMsg);

  var consoleDiv = document.getElementById("console");

  if (info.vShaderSource === undefined) {
    if (info.vShaderId) {
      info.vShaderSource = document.getElementById(info.vShaderId).text;
    } else {
      info.vShader = 'defaultVertexShader';
      info.vShaderSource = defaultVertexShader;
    }
  }
  if (info.fShaderSource === undefined) {
    if (info.fShaderId) {
      info.fShaderSource = document.getElementById(info.fShaderId).text;
    } else {
      info.fShader = 'defaultFragmentShader';
      info.fShaderSource = defaultFragmentShader;
    }
  }

  var vLabel = (info.vShaderSource == defaultVertexShader ? "default" : "test") + " vertex shader";
  var fLabel = (info.fShaderSource == defaultFragmentShader ? "default" : "test") + " fragment shader";

  var vSource = info.vShaderPrep ? info.vShaderPrep(info.vShaderSource) :
    info.vShaderSource;

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
  if (debugShaders && vShader) {
    wtu.addShaderSource(consoleDiv, vLabel + " translated for driver",
                        debugShaders.getTranslatedShaderSource(vShader));
  }

  var fSource = info.fShaderPrep ? info.fShaderPrep(info.fShaderSource) :
    info.fShaderSource;

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

  if (debugShaders && fShader) {
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
  wtu.setupUnitQuad(gl);
  wtu.clearAndDrawUnitQuad(gl);

  var div = document.createElement("div");
  div.className = "testimages";
  wtu.insertImage(div, "result", wtu.makeImageFromCanvas(gl.canvas));
  div.appendChild(document.createElement('br'));
  consoleDiv.appendChild(div);
  wtu.checkCanvas(gl, [0, 255, 0, 255], "should be green", 0);
}

function runTests(shaderInfos) {
  var wtu = WebGLTestUtils;
  var canvas = document.createElement('canvas');
  canvas.width = 32;
  canvas.height = 32;
  var gl = wtu.create3DContext(canvas);
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

function loadExternalShaders(filename, passMsg) {
  var shaderInfos = [];
  var lines = wtu.readFileList(filename);
  for (var ii = 0; ii < lines.length; ++ii) {
    var info = {
      vShaderSource:  defaultVertexShader,
      vShaderSuccess: true,
      fShaderSource:  defaultFragmentShader,
      fShaderSuccess: true,
      linkSuccess:    true,
    };

    var line = lines[ii];
    var files = line.split(/ +/);
    var passMsg = "";
    for (var jj = 0; jj < files.length; ++jj) {
      var file = files[jj];
      var shaderSource = wtu.readFile(file);
      var firstLine = shaderSource.split("\n")[0];
      var success = undefined;
      if (firstLine.indexOf("fail") >= 0) {
        success = false;
      } else if (firstLine.indexOf("succeed") >= 0) {
        success = true;
      }
      if (success === undefined) {
        testFailed("bad first line in " + file + ":" + firstLine);
        continue;
      }
      if (!wtu.startsWith(firstLine, "// ")) {
        testFailed("bad first line in " + file + ":" + firstLine);
        continue;
      }
      passMsg = passMsg + (passMsg.length ? ", " : "") + firstLine.substr(3);
      if (wtu.endsWith(file, ".vert")) {
        info.vShaderSource = shaderSource;
        info.vShaderSuccess = success;
      } else if (wtu.endsWith(file, ".frag")) {
        info.fShaderSource = shaderSource;
        info.fShaderSuccess = success;
      }
    }
    info.linkSuccess = info.vShaderSuccess && info.fShaderSuccess;
    info.passMsg = passMsg;
    shaderInfos.push(info);
  }
  return shaderInfos;
}

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
  var vShaderElem = document.getElementById('vertexShader');
  var vShaderSource = defaultVertexShader;
  var vShaderSuccess = true;

  var fShaderElem = document.getElementById('fragmentShader');
  var fShaderSource = defaultFragmentShader;
  var fShaderSuccess = true;

  var passMsg = undefined;

  if (vShaderElem) {
    vShaderSource = getSource(vShaderElem);
    passMsg = getPassMessage(vShaderSource);
    vShaderSuccess = getSuccess(passMsg);
  }

  if (fShaderElem) {
    fShaderSource = getSource(fShaderElem);
    passMsg = getPassMessage(fShaderSource);
    fShaderSuccess = getSuccess(passMsg);
  }

  var linkSuccess = vShaderSuccess && fShaderSuccess;

  if (passMsg === undefined) {
    testFailed("no test shader found.");
    finishTest();
    return;
  }

  var info = {
    vShaderSource: vShaderSource,
    vShaderSuccess: vShaderSuccess,
    fShaderSource: fShaderSource,
    fShaderSuccess: fShaderSuccess,
    linkSuccess: linkSuccess,
    passMsg: passMsg
  };

  return info;
}

function runTest() {
  var info = setupTest();
  description(info.passMsg);
  runTests([info]);
}

function runRenderTests(tests) {
  for (var ii = 0; ii < tests.length; ++ii) {
    tests[ii].render = true
  }
  runTests(tests);
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
  runRenderTests: runRenderTests,
  loadExternalShaders: loadExternalShaders,

  none: false,
};
}());
