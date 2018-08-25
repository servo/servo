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
GLSLGenerator = (function() {

var vertexShaderTemplate = [
  "attribute vec4 aPosition;",
  "",
  "varying vec4 vColor;",
  "",
  "$(extra)",
  "$(emu)",
  "",
  "void main()",
  "{",
  "   gl_Position = aPosition;",
  "   vec2 texcoord = vec2(aPosition.xy * 0.5 + vec2(0.5, 0.5));",
  "   vec4 color = vec4(",
  "       texcoord,",
  "       texcoord.x * texcoord.y,",
  "       (1.0 - texcoord.x) * texcoord.y * 0.5 + 0.5);",
  "   $(test)",
  "}"
].join("\n");

var fragmentShaderTemplate = [
  "precision mediump float;",
  "",
  "varying vec4 vColor;",
  "",
  "$(extra)",
  "$(emu)",
  "",
  "void main()",
  "{",
  "   $(test)",
  "}"
].join("\n");

var baseVertexShader = [
  "attribute vec4 aPosition;",
  "",
  "varying vec4 vColor;",
  "",
  "void main()",
  "{",
  "   gl_Position = aPosition;",
  "   vec2 texcoord = vec2(aPosition.xy * 0.5 + vec2(0.5, 0.5));",
  "   vColor = vec4(",
  "       texcoord,",
  "       texcoord.x * texcoord.y,",
  "       (1.0 - texcoord.x) * texcoord.y * 0.5 + 0.5);",
  "}"
].join("\n");

var baseVertexShaderWithColor = [
  "attribute vec4 aPosition;",
  "attribute vec4 aColor;",
  "",
  "varying vec4 vColor;",
  "",
  "void main()",
  "{",
  "   gl_Position = aPosition;",
  "   vColor = aColor;",
  "}"
].join("\n");

var baseFragmentShader = [
  "precision mediump float;",
  "varying vec4 vColor;",
  "",
  "void main()",
  "{",
  "   gl_FragColor = vColor;",
  "}"
].join("\n");

var types = [
  { type: "float",
    code: [
      "float $(func)_emu($(args)) {",
      "  return $(func)_base($(baseArgs));",
      "}"].join("\n")
  },
  { type: "vec2",
    code: [
      "vec2 $(func)_emu($(args)) {",
      "  return vec2(",
      "      $(func)_base($(baseArgsX)),",
      "      $(func)_base($(baseArgsY)));",
      "}"].join("\n")
  },
  { type: "vec3",
    code: [
      "vec3 $(func)_emu($(args)) {",
      "  return vec3(",
      "      $(func)_base($(baseArgsX)),",
      "      $(func)_base($(baseArgsY)),",
      "      $(func)_base($(baseArgsZ)));",
      "}"].join("\n")
  },
  { type: "vec4",
    code: [
      "vec4 $(func)_emu($(args)) {",
      "  return vec4(",
      "      $(func)_base($(baseArgsX)),",
      "      $(func)_base($(baseArgsY)),",
      "      $(func)_base($(baseArgsZ)),",
      "      $(func)_base($(baseArgsW)));",
      "}"].join("\n")
  }
];

var bvecTypes = [
  { type: "bvec2",
    code: [
      "bvec2 $(func)_emu($(args)) {",
      "  return bvec2(",
      "      $(func)_base($(baseArgsX)),",
      "      $(func)_base($(baseArgsY)));",
      "}"].join("\n")
  },
  { type: "bvec3",
    code: [
      "bvec3 $(func)_emu($(args)) {",
      "  return bvec3(",
      "      $(func)_base($(baseArgsX)),",
      "      $(func)_base($(baseArgsY)),",
      "      $(func)_base($(baseArgsZ)));",
      "}"].join("\n")
  },
  { type: "bvec4",
    code: [
      "vec4 $(func)_emu($(args)) {",
      "  return bvec4(",
      "      $(func)_base($(baseArgsX)),",
      "      $(func)_base($(baseArgsY)),",
      "      $(func)_base($(baseArgsZ)),",
      "      $(func)_base($(baseArgsW)));",
      "}"].join("\n")
  }
];

var replaceRE = /\$\((\w+)\)/g;

var replaceParams = function(str) {
  var args = arguments;
  return str.replace(replaceRE, function(str, p1, offset, s) {
    for (var ii = 1; ii < args.length; ++ii) {
      if (args[ii][p1] !== undefined) {
        return args[ii][p1];
      }
    }
    throw "unknown string param '" + p1 + "'";
  });
};

var generateReferenceShader = function(
    shaderInfo, template, params, typeInfo, test) {
  var input = shaderInfo.input;
  var output = shaderInfo.output;
  var feature = params.feature;
  var testFunc = params.testFunc;
  var emuFunc = params.emuFunc || "";
  var extra = params.extra || '';
  var args = params.args || "$(type) value";
  var type = typeInfo.type;
  var typeCode = typeInfo.code;

  var baseArgs = params.baseArgs || "value$(field)";
  var baseArgsX = replaceParams(baseArgs, {field: ".x"});
  var baseArgsY = replaceParams(baseArgs, {field: ".y"});
  var baseArgsZ = replaceParams(baseArgs, {field: ".z"});
  var baseArgsW = replaceParams(baseArgs, {field: ".w"});
  var baseArgs = replaceParams(baseArgs, {field: ""});

  test = replaceParams(test, {
    input: input,
    output: output,
    func: feature + "_emu"
  });
  emuFunc = replaceParams(emuFunc, {
    func: feature
  });
  args = replaceParams(args, {
    type: type
  });
  typeCode = replaceParams(typeCode, {
    func: feature,
    type: type,
    args: args,
    baseArgs: baseArgs,
    baseArgsX: baseArgsX,
    baseArgsY: baseArgsY,
    baseArgsZ: baseArgsZ,
    baseArgsW: baseArgsW
  });
  var shader = replaceParams(template, {
    extra: extra,
    emu: emuFunc + "\n\n" + typeCode,
    test: test
  });
  return shader;
};

var generateTestShader = function(
    shaderInfo, template, params, test) {
  var input = shaderInfo.input;
  var output = shaderInfo.output;
  var feature = params.feature;
  var testFunc = params.testFunc;
  var extra = params.extra || '';

  test = replaceParams(test, {
    input: input,
    output: output,
    func: feature
  });
  var shader = replaceParams(template, {
    extra: extra,
    emu: '',
    test: test
  });
  return shader;
};

function _reportResults(refData, refImg, testData, testImg, tolerance,
                        width, height, ctx, imgData, wtu, canvas2d, consoleDiv) {
  var same = true;
  var firstFailure = null;
  for (var yy = 0; yy < height; ++yy) {
    for (var xx = 0; xx < width; ++xx) {
      var offset = (yy * width + xx) * 4;
      var imgOffset = ((height - yy - 1) * width + xx) * 4;
      imgData.data[imgOffset + 0] = 0;
      imgData.data[imgOffset + 1] = 0;
      imgData.data[imgOffset + 2] = 0;
      imgData.data[imgOffset + 3] = 255;
      if (Math.abs(refData[offset + 0] - testData[offset + 0]) > tolerance ||
          Math.abs(refData[offset + 1] - testData[offset + 1]) > tolerance ||
          Math.abs(refData[offset + 2] - testData[offset + 2]) > tolerance ||
          Math.abs(refData[offset + 3] - testData[offset + 3]) > tolerance) {
        var detail = 'at (' + xx + ',' + yy + '): ref=(' +
            refData[offset + 0] + ',' +
            refData[offset + 1] + ',' +
            refData[offset + 2] + ',' +
            refData[offset + 3] + ')  test=(' +
            testData[offset + 0] + ',' +
            testData[offset + 1] + ',' +
            testData[offset + 2] + ',' +
            testData[offset + 3] + ') tolerance=' + tolerance;
        consoleDiv.appendChild(document.createTextNode(detail));
        consoleDiv.appendChild(document.createElement('br'));
        if (!firstFailure) {
          firstFailure = ": " + detail;
        }
        imgData.data[imgOffset] = 255;
        same = false;
      }
    }
  }

  var diffImg = null;
  if (!same) {
    ctx.putImageData(imgData, 0, 0);
    diffImg = wtu.makeImageFromCanvas(canvas2d);
  }

  var div = document.createElement("div");
  div.className = "testimages";
  wtu.insertImage(div, "ref", refImg);
  wtu.insertImage(div, "test", testImg);
  if (diffImg) {
    wtu.insertImage(div, "diff", diffImg);
  }
  div.appendChild(document.createElement('br'));

  consoleDiv.appendChild(div);

  if (!same) {
    testFailed("images are different" + (firstFailure ? firstFailure : ""));
  } else {
    testPassed("images are the same");
  }

  consoleDiv.appendChild(document.createElement('hr'));
}

var runFeatureTest = function(params) {
  var wtu = WebGLTestUtils;
  var gridRes = params.gridRes;
  var vertexTolerance = params.tolerance || 0;
  var fragmentTolerance = params.tolerance || 1;
  if ('fragmentTolerance' in params)
    fragmentTolerance = params.fragmentTolerance;

  description("Testing GLSL feature: " + params.feature);

  var width = 32;
  var height = 32;

  var consoleDiv = document.getElementById("console");
  var canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  var gl = wtu.create3DContext(canvas, { premultipliedAlpha: false });
  if (!gl) {
    testFailed("context does not exist");
    finishTest();
    return;
  }

  var canvas2d = document.createElement('canvas');
  canvas2d.width = width;
  canvas2d.height = height;
  var ctx = canvas2d.getContext("2d");
  var imgData = ctx.getImageData(0, 0, width, height);

  var shaderInfos = [
    { type: "vertex",
      input: "color",
      output: "vColor",
      vertexShaderTemplate: vertexShaderTemplate,
      fragmentShaderTemplate: baseFragmentShader,
      tolerance: vertexTolerance
    },
    { type: "fragment",
      input: "vColor",
      output: "gl_FragColor",
      vertexShaderTemplate: baseVertexShader,
      fragmentShaderTemplate: fragmentShaderTemplate,
      tolerance: fragmentTolerance
    }
  ];
  for (var ss = 0; ss < shaderInfos.length; ++ss) {
    var shaderInfo = shaderInfos[ss];
    var tests = params.tests;
    var testTypes = params.emuFuncs || (params.bvecTest ? bvecTypes : types);
    // Test vertex shaders
    for (var ii = 0; ii < tests.length; ++ii) {
      var type = testTypes[ii];
      if (params.simpleEmu) {
        type = {
          type: type.type,
          code: params.simpleEmu
        };
      }
      debug("");
      var str = replaceParams(params.testFunc, {
        func: params.feature,
        type: type.type,
        arg0: type.type
      });
      var passMsg = "Testing: " + str + " in " + shaderInfo.type + " shader";
      debug(passMsg);

      var referenceVertexShaderSource = generateReferenceShader(
          shaderInfo,
          shaderInfo.vertexShaderTemplate,
          params,
          type,
          tests[ii]);
      var referenceFragmentShaderSource = generateReferenceShader(
          shaderInfo,
          shaderInfo.fragmentShaderTemplate,
          params,
          type,
          tests[ii]);
      var testVertexShaderSource = generateTestShader(
          shaderInfo,
          shaderInfo.vertexShaderTemplate,
          params,
          tests[ii]);
      var testFragmentShaderSource = generateTestShader(
          shaderInfo,
          shaderInfo.fragmentShaderTemplate,
          params,
          tests[ii]);


      debug("");
      var referenceVertexShader = wtu.loadShader(gl, referenceVertexShaderSource, gl.VERTEX_SHADER, testFailed, true, 'reference');
      var referenceFragmentShader = wtu.loadShader(gl, referenceFragmentShaderSource, gl.FRAGMENT_SHADER, testFailed, true, 'reference');
      var testVertexShader = wtu.loadShader(gl, testVertexShaderSource, gl.VERTEX_SHADER, testFailed, true, 'test');
      var testFragmentShader = wtu.loadShader(gl, testFragmentShaderSource, gl.FRAGMENT_SHADER, testFailed, true, 'test');
      debug("");

      if (parseInt(wtu.getUrlOptions().dumpShaders)) {
        var vRefInfo = {
          shader: referenceVertexShader,
          shaderSuccess: true,
          label: "reference vertex shader",
          source: referenceVertexShaderSource
        };
        var fRefInfo = {
          shader: referenceFragmentShader,
          shaderSuccess: true,
          label: "reference fragment shader",
          source: referenceFragmentShaderSource
        };
        wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vRefInfo, fRefInfo);

        var vTestInfo = {
          shader: testVertexShader,
          shaderSuccess: true,
          label: "test vertex shader",
          source: testVertexShaderSource
        };
        var fTestInfo = {
          shader: testFragmentShader,
          shaderSuccess: true,
          label: "test fragment shader",
          source: testFragmentShaderSource
        };
        wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vTestInfo, fTestInfo);
      }

      var refData = draw(
          referenceVertexShader, referenceFragmentShader);
      var refImg = wtu.makeImageFromCanvas(canvas);
      if (ss == 0) {
        var testData = draw(
            testVertexShader, referenceFragmentShader);
      } else {
        var testData = draw(
            referenceVertexShader, testFragmentShader);
      }
      var testImg = wtu.makeImageFromCanvas(canvas);

      _reportResults(refData, refImg, testData, testImg, shaderInfo.tolerance,
                     width, height, ctx, imgData, wtu, canvas2d, consoleDiv);
    }
  }

  finishTest();

  function draw(vertexShader, fragmentShader) {
    var program = wtu.createProgram(gl, vertexShader, fragmentShader, testFailed);

    var posLoc = gl.getAttribLocation(program, "aPosition");
    wtu.setupIndexedQuad(gl, gridRes, posLoc);

    gl.useProgram(program);
    wtu.clearAndDrawIndexedQuad(gl, gridRes, [0, 0, 255, 255]);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "no errors from draw");

    var img = new Uint8Array(width * height * 4);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, img);
    return img;
  }

};

var runBasicTest = function(params) {
  var wtu = WebGLTestUtils;
  var gridRes = params.gridRes;
  var vertexTolerance = params.tolerance || 0;
  var fragmentTolerance = vertexTolerance;
  if ('fragmentTolerance' in params)
    fragmentTolerance = params.fragmentTolerance || 0;

  description("Testing : " + document.getElementsByTagName("title")[0].innerText);

  var width = 32;
  var height = 32;

  var consoleDiv = document.getElementById("console");
  var canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  var gl = wtu.create3DContext(canvas);
  if (!gl) {
    testFailed("context does not exist");
    finishTest();
    return;
  }

  var canvas2d = document.createElement('canvas');
  canvas2d.width = width;
  canvas2d.height = height;
  var ctx = canvas2d.getContext("2d");
  var imgData = ctx.getImageData(0, 0, width, height);

  var shaderInfos = [
    { type: "vertex",
      input: "color",
      output: "vColor",
      vertexShaderTemplate: vertexShaderTemplate,
      fragmentShaderTemplate: baseFragmentShader,
      tolerance: vertexTolerance
    },
    { type: "fragment",
      input: "vColor",
      output: "gl_FragColor",
      vertexShaderTemplate: baseVertexShader,
      fragmentShaderTemplate: fragmentShaderTemplate,
      tolerance: fragmentTolerance
    }
  ];
  for (var ss = 0; ss < shaderInfos.length; ++ss) {
    var shaderInfo = shaderInfos[ss];
    var tests = params.tests;
//    var testTypes = params.emuFuncs || (params.bvecTest ? bvecTypes : types);
    // Test vertex shaders
    for (var ii = 0; ii < tests.length; ++ii) {
      var test = tests[ii];
      debug("");
      var passMsg = "Testing: " + test.name + " in " + shaderInfo.type + " shader";
      debug(passMsg);

      function genShader(shaderInfo, template, shader, subs) {
        shader = replaceParams(shader, subs, {
            input: shaderInfo.input,
            output: shaderInfo.output
          });
        shader = replaceParams(template, subs, {
            test: shader,
            emu: "",
            extra: ""
          });
        return shader;
      }

      var referenceVertexShaderSource = genShader(
          shaderInfo,
          shaderInfo.vertexShaderTemplate,
          test.reference.shader,
          test.reference.subs);
      var referenceFragmentShaderSource = genShader(
          shaderInfo,
          shaderInfo.fragmentShaderTemplate,
          test.reference.shader,
          test.reference.subs);
      var testVertexShaderSource = genShader(
          shaderInfo,
          shaderInfo.vertexShaderTemplate,
          test.test.shader,
          test.test.subs);
      var testFragmentShaderSource = genShader(
          shaderInfo,
          shaderInfo.fragmentShaderTemplate,
          test.test.shader,
          test.test.subs);

      debug("");
      var referenceVertexShader = wtu.loadShader(gl, referenceVertexShaderSource, gl.VERTEX_SHADER, testFailed, true, 'reference');
      var referenceFragmentShader = wtu.loadShader(gl, referenceFragmentShaderSource, gl.FRAGMENT_SHADER, testFailed, true, 'reference');
      var testVertexShader = wtu.loadShader(gl, testVertexShaderSource, gl.VERTEX_SHADER, testFailed, true, 'test');
      var testFragmentShader = wtu.loadShader(gl, testFragmentShaderSource, gl.FRAGMENT_SHADER, testFailed, true, 'test');
      debug("");

      if (parseInt(wtu.getUrlOptions().dumpShaders)) {
        var vRefInfo = {
          shader: referenceVertexShader,
          shaderSuccess: true,
          label: "reference vertex shader",
          source: referenceVertexShaderSource
        };
        var fRefInfo = {
          shader: referenceFragmentShader,
          shaderSuccess: true,
          label: "reference fragment shader",
          source: referenceFragmentShaderSource
        };
        wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vRefInfo, fRefInfo);

        var vTestInfo = {
          shader: testVertexShader,
          shaderSuccess: true,
          label: "test vertex shader",
          source: testVertexShaderSource
        };
        var fTestInfo = {
          shader: testFragmentShader,
          shaderSuccess: true,
          label: "test fragment shader",
          source: testFragmentShaderSource
        };
        wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vTestInfo, fTestInfo);
      }

      var refData = draw(referenceVertexShader, referenceFragmentShader);
      var refImg = wtu.makeImageFromCanvas(canvas);
      if (ss == 0) {
        var testData = draw(testVertexShader, referenceFragmentShader);
      } else {
        var testData = draw(referenceVertexShader, testFragmentShader);
      }
      var testImg = wtu.makeImageFromCanvas(canvas);

      _reportResults(refData, refImg, testData, testImg, shaderInfo.tolerance,
                     width, height, ctx, imgData, wtu, canvas2d, consoleDiv);
    }
  }

  finishTest();

  function draw(vertexShader, fragmentShader) {
    var program = wtu.createProgram(gl, vertexShader, fragmentShader, testFailed);

    var posLoc = gl.getAttribLocation(program, "aPosition");
    wtu.setupIndexedQuad(gl, gridRes, posLoc);

    gl.useProgram(program);
    wtu.clearAndDrawIndexedQuad(gl, gridRes, [0, 0, 255, 255]);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "no errors from draw");

    var img = new Uint8Array(width * height * 4);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, img);
    return img;
  }

};

var runReferenceImageTest = function(params) {
  var wtu = WebGLTestUtils;
  var gridRes = params.gridRes;
  var vertexTolerance = params.tolerance || 0;
  var fragmentTolerance = vertexTolerance;
  if ('fragmentTolerance' in params)
    fragmentTolerance = params.fragmentTolerance || 0;

  description("Testing GLSL feature: " + params.feature);

  var width = 32;
  var height = 32;

  var consoleDiv = document.getElementById("console");
  var canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  var gl = wtu.create3DContext(canvas, { antialias: false, premultipliedAlpha: false });
  if (!gl) {
    testFailed("context does not exist");
    finishTest();
    return;
  }

  var canvas2d = document.createElement('canvas');
  canvas2d.width = width;
  canvas2d.height = height;
  var ctx = canvas2d.getContext("2d");
  var imgData = ctx.getImageData(0, 0, width, height);

  // State for reference images for vertex shader tests.
  // These are drawn with the same tessellated grid as the test vertex
  // shader so that the interpolation is identical. The grid is reused
  // from test to test; the colors are changed.

  var indexedQuadForReferenceVertexShader =
    wtu.setupIndexedQuad(gl, gridRes, 0);
  var referenceVertexShaderProgram =
    wtu.setupProgram(gl, [ baseVertexShaderWithColor, baseFragmentShader ],
                     ["aPosition", "aColor"]);
  var referenceVertexShaderColorBuffer = gl.createBuffer();

  var shaderInfos = [
    { type: "vertex",
      input: "color",
      output: "vColor",
      vertexShaderTemplate: vertexShaderTemplate,
      fragmentShaderTemplate: baseFragmentShader,
      tolerance: vertexTolerance
    },
    { type: "fragment",
      input: "vColor",
      output: "gl_FragColor",
      vertexShaderTemplate: baseVertexShader,
      fragmentShaderTemplate: fragmentShaderTemplate,
      tolerance: fragmentTolerance
    }
  ];
  for (var ss = 0; ss < shaderInfos.length; ++ss) {
    var shaderInfo = shaderInfos[ss];
    var tests = params.tests;
    var testTypes = params.emuFuncs || (params.bvecTest ? bvecTypes : types);
    // Test vertex shaders
    for (var ii = 0; ii < tests.length; ++ii) {
      var type = testTypes[ii];
      var isVertex = (ss == 0);
      debug("");
      var str = replaceParams(params.testFunc, {
        func: params.feature,
        type: type.type,
        arg0: type.type
      });
      var passMsg = "Testing: " + str + " in " + shaderInfo.type + " shader";
      debug(passMsg);

      var referenceVertexShaderSource = generateReferenceShader(
          shaderInfo,
          shaderInfo.vertexShaderTemplate,
          params,
          type,
          tests[ii].source);
      var referenceFragmentShaderSource = generateReferenceShader(
          shaderInfo,
          shaderInfo.fragmentShaderTemplate,
          params,
          type,
          tests[ii].source);
      var testVertexShaderSource = generateTestShader(
          shaderInfo,
          shaderInfo.vertexShaderTemplate,
          params,
          tests[ii].source);
      var testFragmentShaderSource = generateTestShader(
          shaderInfo,
          shaderInfo.fragmentShaderTemplate,
          params,
          tests[ii].source);
      var referenceTextureOrArray = generateReferenceImage(
          gl,
          tests[ii].generator,
          isVertex ? gridRes : width,
          isVertex ? gridRes : height,
          isVertex);

      debug("");
      var testVertexShader = wtu.loadShader(gl, testVertexShaderSource, gl.VERTEX_SHADER, testFailed, true);
      var testFragmentShader = wtu.loadShader(gl, testFragmentShaderSource, gl.FRAGMENT_SHADER, testFailed, true);
      debug("");


      if (parseInt(wtu.getUrlOptions().dumpShaders)) {
        var vRefInfo = {
          shader: referenceVertexShader,
          shaderSuccess: true,
          label: "reference vertex shader",
          source: referenceVertexShaderSource
        };
        var fRefInfo = {
          shader: referenceFragmentShader,
          shaderSuccess: true,
          label: "reference fragment shader",
          source: referenceFragmentShaderSource
        };
        wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vRefInfo, fRefInfo);

        var vTestInfo = {
          shader: testVertexShader,
          shaderSuccess: true,
          label: "test vertex shader",
          source: testVertexShaderSource
        };
        var fTestInfo = {
          shader: testFragmentShader,
          shaderSuccess: true,
          label: "test fragment shader",
          source: testFragmentShaderSource
        };
        wtu.dumpShadersInfo(gl, window.location.pathname, passMsg, vTestInfo, fTestInfo);
      }

      var refData;
      if (isVertex) {
        refData = drawVertexReferenceImage(referenceTextureOrArray);
      } else {
        refData = drawFragmentReferenceImage(referenceTextureOrArray);
      }
      var refImg = wtu.makeImageFromCanvas(canvas);
      var testData;
      if (isVertex) {
        var referenceFragmentShader = wtu.loadShader(gl, referenceFragmentShaderSource, gl.FRAGMENT_SHADER, testFailed);
        testData = draw(
          testVertexShader, referenceFragmentShader);
      } else {
        var referenceVertexShader = wtu.loadShader(gl, referenceVertexShaderSource, gl.VERTEX_SHADER, testFailed);
        testData = draw(
          referenceVertexShader, testFragmentShader);
      }
      var testImg = wtu.makeImageFromCanvas(canvas);
      var testTolerance = shaderInfo.tolerance;
      // Provide per-test tolerance so that we can increase it only for those desired.
      if ('tolerance' in tests[ii])
        testTolerance = tests[ii].tolerance || 0;
      _reportResults(refData, refImg, testData, testImg, testTolerance,
                     width, height, ctx, imgData, wtu, canvas2d, consoleDiv);
    }
  }

  finishTest();

  function draw(vertexShader, fragmentShader) {
    var program = wtu.createProgram(gl, vertexShader, fragmentShader, testFailed);

    var posLoc = gl.getAttribLocation(program, "aPosition");
    wtu.setupIndexedQuad(gl, gridRes, posLoc);

    gl.useProgram(program);
    wtu.clearAndDrawIndexedQuad(gl, gridRes, [0, 0, 255, 255]);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "no errors from draw");

    var img = new Uint8Array(width * height * 4);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, img);
    return img;
  }

  function drawVertexReferenceImage(colors) {
    gl.bindBuffer(gl.ARRAY_BUFFER, indexedQuadForReferenceVertexShader[0]);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
    gl.bindBuffer(gl.ARRAY_BUFFER, referenceVertexShaderColorBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, colors, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 4, gl.UNSIGNED_BYTE, true, 0, 0);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexedQuadForReferenceVertexShader[1]);
    gl.useProgram(referenceVertexShaderProgram);
    wtu.clearAndDrawIndexedQuad(gl, gridRes);
    gl.disableVertexAttribArray(0);
    gl.disableVertexAttribArray(1);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "no errors from draw");

    var img = new Uint8Array(width * height * 4);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, img);
    return img;
  }

  function drawFragmentReferenceImage(texture) {
    var program = wtu.setupTexturedQuad(gl);

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    var texLoc = gl.getUniformLocation(program, "tex");
    gl.uniform1i(texLoc, 0);
    wtu.clearAndDrawUnitQuad(gl);
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "no errors from draw");

    var img = new Uint8Array(width * height * 4);
    gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, img);
    return img;
  }

  /**
   * Creates and returns either a Uint8Array (for vertex shaders) or
   * WebGLTexture (for fragment shaders) containing the reference
   * image for the function being tested. Exactly how the function is
   * evaluated, and the size of the returned texture or array, depends on
   * whether we are testing a vertex or fragment shader. If a fragment
   * shader, the function is evaluated at the pixel centers. If a
   * vertex shader, the function is evaluated at the triangle's
   * vertices.
   *
   * @param {!WebGLRenderingContext} gl The WebGLRenderingContext to use to generate texture objects.
   * @param {!function(number,number,number,number): !Array.<number>} generator The reference image generator function.
   * @param {number} width The width of the texture to generate if testing a fragment shader; the grid resolution if testing a vertex shader.
   * @param {number} height The height of the texture to generate if testing a fragment shader; the grid resolution if testing a vertex shader.
   * @param {boolean} isVertex True if generating a reference image for a vertex shader; false if for a fragment shader.
   * @return {!WebGLTexture|!Uint8Array} The texture object or array that was generated.
   */
  function generateReferenceImage(
    gl,
    generator,
    width,
    height,
    isVertex) {

    // Note: the math in this function must match that in the vertex and
    // fragment shader templates above.
    function computeTexCoord(x) {
      return x * 0.5 + 0.5;
    }

    function computeVertexColor(texCoordX, texCoordY) {
      return [ texCoordX,
               texCoordY,
               texCoordX * texCoordY,
               (1.0 - texCoordX) * texCoordY * 0.5 + 0.5 ];
    }

    /**
     * Computes fragment color according to the algorithm used for interpolation
     * in OpenGL (GLES 2.0 spec 3.5.1, OpenGL 4.3 spec 14.6.1).
     */
    function computeInterpolatedColor(texCoordX, texCoordY) {
      // Calculate grid line indexes below and to the left from texCoord.
      var gridBottom = Math.floor(texCoordY * gridRes);
      if (gridBottom == gridRes) {
        --gridBottom;
      }
      var gridLeft = Math.floor(texCoordX * gridRes);
      if (gridLeft == gridRes) {
        --gridLeft;
      }

      // Calculate coordinates relative to the grid cell.
      var cellX = texCoordX * gridRes - gridLeft;
      var cellY = texCoordY * gridRes - gridBottom;

      // Barycentric coordinates inside either triangle ACD or ABC
      // are used as weights for the vertex colors in the corners:
      // A--B
      // |\ |
      // | \|
      // D--C

      var aColor = computeVertexColor(gridLeft / gridRes, (gridBottom + 1) / gridRes);
      var bColor = computeVertexColor((gridLeft + 1) / gridRes, (gridBottom + 1) / gridRes);
      var cColor = computeVertexColor((gridLeft + 1) / gridRes, gridBottom / gridRes);
      var dColor = computeVertexColor(gridLeft / gridRes, gridBottom / gridRes);

      // Calculate weights.
      var a, b, c, d;

      if (cellX + cellY < 1) {
        // In bottom triangle ACD.
        a = cellY; // area of triangle C-D-(cellX, cellY) relative to ACD
        c = cellX; // area of triangle D-A-(cellX, cellY) relative to ACD
        d = 1 - a - c;
        b = 0;
      } else {
        // In top triangle ABC.
        a = 1 - cellX; // area of the triangle B-C-(cellX, cellY) relative to ABC
        c = 1 - cellY; // area of the triangle A-B-(cellX, cellY) relative to ABC
        b = 1 - a - c;
        d = 0;
      }

      var interpolated = [];
      for (var ii = 0; ii < aColor.length; ++ii) {
        interpolated.push(a * aColor[ii] + b * bColor[ii] + c * cColor[ii] + d * dColor[ii]);
      }
      return interpolated;
    }

    function clamp(value, minVal, maxVal) {
      return Math.max(minVal, Math.min(value, maxVal));
    }

    // Evaluates the function at clip coordinates (px,py), storing the
    // result in the array "pixel". Each channel's result is clamped
    // between 0 and 255.
    function evaluateAtClipCoords(px, py, pixel, colorFunc) {
      var tcx = computeTexCoord(px);
      var tcy = computeTexCoord(py);

      var color = colorFunc(tcx, tcy);

      var output = generator(color[0], color[1], color[2], color[3]);

      // Multiply by 256 to get even distribution for all values between 0 and 1.
      // Use rounding rather than truncation to more closely match the GPU's behavior.
      pixel[0] = clamp(Math.round(256 * output[0]), 0, 255);
      pixel[1] = clamp(Math.round(256 * output[1]), 0, 255);
      pixel[2] = clamp(Math.round(256 * output[2]), 0, 255);
      pixel[3] = clamp(Math.round(256 * output[3]), 0, 255);
    }

    function generateFragmentReference() {
      var data = new Uint8Array(4 * width * height);

      var horizTexel = 1.0 / width;
      var vertTexel = 1.0 / height;
      var halfHorizTexel = 0.5 * horizTexel;
      var halfVertTexel = 0.5 * vertTexel;

      var pixel = new Array(4);

      for (var yi = 0; yi < height; ++yi) {
        for (var xi = 0; xi < width; ++xi) {
          // The function must be evaluated at pixel centers.

          // Compute desired position in clip space
          var px = -1.0 + 2.0 * (halfHorizTexel + xi * horizTexel);
          var py = -1.0 + 2.0 * (halfVertTexel + yi * vertTexel);

          evaluateAtClipCoords(px, py, pixel, computeInterpolatedColor);
          var index = 4 * (width * yi + xi);
          data[index + 0] = pixel[0];
          data[index + 1] = pixel[1];
          data[index + 2] = pixel[2];
          data[index + 3] = pixel[3];
        }
      }

      var texture = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0,
                    gl.RGBA, gl.UNSIGNED_BYTE, data);
      return texture;
    }

    function generateVertexReference() {
      // We generate a Uint8Array which contains the evaluation of the
      // function at the vertices of the triangle mesh. It is expected
      // that the width and the height are identical, and equivalent
      // to the grid resolution.
      if (width != height) {
        throw "width and height must be equal";
      }

      var texSize = 1 + width;
      var data = new Uint8Array(4 * texSize * texSize);

      var step = 2.0 / width;

      var pixel = new Array(4);

      for (var yi = 0; yi < texSize; ++yi) {
        for (var xi = 0; xi < texSize; ++xi) {
          // The function is evaluated at the triangles' vertices.

          // Compute desired position in clip space
          var px = -1.0 + (xi * step);
          var py = -1.0 + (yi * step);

          evaluateAtClipCoords(px, py, pixel, computeVertexColor);
          var index = 4 * (texSize * yi + xi);
          data[index + 0] = pixel[0];
          data[index + 1] = pixel[1];
          data[index + 2] = pixel[2];
          data[index + 3] = pixel[3];
        }
      }

      return data;
    }

    //----------------------------------------------------------------------
    // Body of generateReferenceImage
    //

    if (isVertex) {
      return generateVertexReference();
    } else {
      return generateFragmentReference();
    }
  }
};

return {
  /**
   * runs a bunch of GLSL tests using the passed in parameters
   * The parameters are:
   *
   * feature:
   *    the name of the function being tested (eg, sin, dot,
   *    normalize)
   *
   * testFunc:
   *    The prototype of function to be tested not including the
   *    return type.
   *
   * emuFunc:
   *    A base function that can be used to generate emulation
   *    functions. Example for 'ceil'
   *
   *      float $(func)_base(float value) {
   *        float m = mod(value, 1.0);
   *        return m != 0.0 ? (value + 1.0 - m) : value;
   *      }
   *
   * args:
   *    The arguments to the function
   *
   * baseArgs: (optional)
   *    The arguments when a base function is used to create an
   *    emulation function. For example 'float sign_base(float v)'
   *    is used to implemenent vec2 sign_emu(vec2 v).
   *
   * simpleEmu:
   *    if supplied, the code that can be used to generate all
   *    functions for all types.
   *
   *    Example for 'normalize':
   *
   *        $(type) $(func)_emu($(args)) {
   *           return value / length(value);
   *        }
   *
   * gridRes: (optional)
   *    The resolution of the mesh to generate. The default is a
   *    1x1 grid but many vertex shaders need a higher resolution
   *    otherwise the only values passed in are the 4 corners
   *    which often have the same value.
   *
   * tests:
   *    The code for each test. It is assumed the tests are for
   *    float, vec2, vec3, vec4 in that order.
   *
   * tolerance: (optional)
   *    Allow some tolerance in the comparisons. The tolerance is applied to
   *    both vertex and fragment shaders. The default tolerance is 0, meaning
   *    the values have to be identical.
   *
   * fragmentTolerance: (optional)
   *    Specify a tolerance which only applies to fragment shaders. The
   *    fragment-only tolerance will override the shared tolerance for
   *    fragment shaders if both are specified. Fragment shaders usually
   *    use mediump float precision so they sometimes require higher tolerance
   *    than vertex shaders which use highp by default.
   */
  runFeatureTest: runFeatureTest,

  /*
   * Runs a bunch of GLSL tests using the passed in parameters
   *
   * The parameters are:
   *
   * tests:
   *    Array of tests. For each test the following parameters are expected
   *
   *    name:
   *       some description of the test
   *    reference:
   *       parameters for the reference shader (see below)
   *    test:
   *       parameters for the test shader (see below)
   *
   *    The parameter for the reference and test shaders are
   *
   *    shader: the GLSL for the shader
   *    subs: any substitutions you wish to define for the shader.
   *
   *    Each shader is created from a basic template that
   *    defines an input and an output. You can see the
   *    templates at the top of this file. The input and output
   *    change depending on whether or not we are generating
   *    a vertex or fragment shader.
   *
   *    All this code function does is a bunch of string substitutions.
   *    A substitution is defined by $(name). If name is found in
   *    the 'subs' parameter it is replaced. 4 special names exist.
   *
   *    'input' the input to your GLSL. Always a vec4. All change
   *    from 0 to 1 over the quad to be drawn.
   *
   *    'output' the output color. Also a vec4
   *
   *    'emu' a place to insert extra stuff
   *    'extra' a place to insert extra stuff.
   *
   *    You can think of the templates like this
   *
   *       $(extra)
   *       $(emu)
   *
   *       void main() {
   *          // do math to calculate input
   *          ...
   *
   *          $(shader)
   *       }
   *
   *    Your shader first has any subs you provided applied as well
   *    as 'input' and 'output'
   *
   *    It is then inserted into the template which is also provided
   *    with your subs.
   *
   * gridRes: (optional)
   *    The resolution of the mesh to generate. The default is a
   *    1x1 grid but many vertex shaders need a higher resolution
   *    otherwise the only values passed in are the 4 corners
   *    which often have the same value.
   *
   * tolerance: (optional)
   *    Allow some tolerance in the comparisons. The tolerance is applied to
   *    both vertex and fragment shaders. The default tolerance is 0, meaning
   *    the values have to be identical.
   *
   * fragmentTolerance: (optional)
   *    Specify a tolerance which only applies to fragment shaders. The
   *    fragment-only tolerance will override the shared tolerance for
   *    fragment shaders if both are specified. Fragment shaders usually
   *    use mediump float precision so they sometimes require higher tolerance
   *    than vertex shaders which use highp.
   */
  runBasicTest: runBasicTest,

  /**
   * Runs a bunch of GLSL tests using the passed in parameters. The
   * expected results are computed as a reference image in JavaScript
   * instead of on the GPU. The parameters are:
   *
   * feature:
   *    the name of the function being tested (eg, sin, dot,
   *    normalize)
   *
   * testFunc:
   *    The prototype of function to be tested not including the
   *    return type.
   *
   * args:
   *    The arguments to the function
   *
   * gridRes: (optional)
   *    The resolution of the mesh to generate. The default is a
   *    1x1 grid but many vertex shaders need a higher resolution
   *    otherwise the only values passed in are the 4 corners
   *    which often have the same value.
   *
   * tests:
   *    Array of tests. It is assumed the tests are for float, vec2,
   *    vec3, vec4 in that order. For each test the following
   *    parameters are expected:
   *
   *       source: the GLSL source code for the tests
   *
   *       generator: a JavaScript function taking four parameters
   *       which evaluates the same function as the GLSL source,
   *       returning its result as a newly allocated array.
   *
   *       tolerance: (optional) a per-test tolerance.
   *
   * extra: (optional)
   *    Extra GLSL code inserted at the top of each test's shader.
   *
   * tolerance: (optional)
   *    Allow some tolerance in the comparisons. The tolerance is applied to
   *    both vertex and fragment shaders. The default tolerance is 0, meaning
   *    the values have to be identical.
   *
   * fragmentTolerance: (optional)
   *    Specify a tolerance which only applies to fragment shaders. The
   *    fragment-only tolerance will override the shared tolerance for
   *    fragment shaders if both are specified. Fragment shaders usually
   *    use mediump float precision so they sometimes require higher tolerance
   *    than vertex shaders which use highp.
   */
  runReferenceImageTest: runReferenceImageTest,

  none: false
};

}());

