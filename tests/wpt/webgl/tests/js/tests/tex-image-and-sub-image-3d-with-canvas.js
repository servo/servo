/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;
    var whiteColor = [255, 255, 255, 255];
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking canvas elements (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        switch (gl[pixelFormat]) {
          case gl.RED:
          case gl.RED_INTEGER:
            whiteColor = [255, 0, 0, 255];
            greenColor = [0, 0, 0];
            break;
          case gl.RG:
          case gl.RG_INTEGER:
            whiteColor = [255, 255, 0, 255];
            break;
          default:
            break;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

        runTest();
    }

    function setCanvasToRedGreen(ctx) {
      var width = ctx.canvas.width;
      var height = ctx.canvas.height;
      var halfHeight = Math.floor(height / 2);
      ctx.fillStyle = "#ff0000";
      ctx.fillRect(0, 0, width, halfHeight);
      ctx.fillStyle = "#00ff00";
      ctx.fillRect(0, halfHeight, width, height - halfHeight);
    }

    function drawTextInCanvas(ctx, bindingTarget) {
      var width = ctx.canvas.width;
      var height = ctx.canvas.height;
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(0, 0, width, height);
      ctx.font = '20pt Arial';
      ctx.fillStyle = 'black';
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillText("1234567890", width / 2, height / 4);
    }

    function setCanvasTo257x257(ctx, bindingTarget) {
      ctx.canvas.width = 257;
      ctx.canvas.height = 257;
      setCanvasToRedGreen(ctx);
    }

    function setCanvasToMin(ctx, bindingTarget) {
      if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
        // cube map texture must be square.
        ctx.canvas.width = 2;
      } else {
        ctx.canvas.width = 1;
      }
      ctx.canvas.height = 2;
      setCanvasToRedGreen(ctx);
    }

    function runOneIteration(canvas, flipY, program, bindingTarget, opt_texture, opt_fontTest)
    {
        var objType = 'canvas';
        if (canvas.transferToImageBitmap)
          objType = 'OffscreenCanvas';
        debug('Testing ' + ' with flipY=' + flipY + ' bindingTarget=' + (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY') +
              ' source object: ' + objType + ' canvas size: ' + canvas.width + 'x' + canvas.height + (opt_fontTest ? " with fonts" : " with red-green"));
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        if (!opt_texture) {
            var texture = gl.createTexture();
            // Bind the texture to texture unit 0
            gl.bindTexture(bindingTarget, texture);
            // Set up texture parameters
            gl.texParameteri(bindingTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
            gl.texParameteri(bindingTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
            gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
            gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        } else {
            var texture = opt_texture;
        }
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        wtu.failIfGLError(gl, 'gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);');
        // Initialize the texture to black first
        gl.texImage3D(bindingTarget, 0, gl[internalFormat], canvas.width, canvas.height, 1 /* depth */, 0,
                      gl[pixelFormat], gl[pixelType], null);
        gl.texSubImage3D(bindingTarget, 0, 0, 0, 0, canvas.width, canvas.height, 1 /* depth */,
                         gl[pixelFormat], gl[pixelType], canvas);

        var width = gl.canvas.width;
        var height = gl.canvas.height;
        var halfHeight = Math.floor(height / 2);
        var top = flipY ? 0 : (height - halfHeight);
        var bottom = flipY ? (height - halfHeight) : 0;

        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 255, 0, 255]);

        if (opt_fontTest) {
            // check half is a solid color.
            wtu.checkCanvasRect(
                  gl, 0, top, width, halfHeight,
                  whiteColor,
                  "should be white");
            // check other half is not a solid color.
            wtu.checkCanvasRectColor(
                  gl, 0, bottom, width, halfHeight,
                  whiteColor, 0,
                  function() {
                    testFailed("font missing");
                  },
                  function() {
                    testPassed("font renderered");
                  },
                  debug);
        } else {
            // Check the top and bottom halves and make sure they have the right color.
            debug("Checking " + (flipY ? "top" : "bottom"));
            wtu.checkCanvasRect(gl, 0, bottom, width, halfHeight, redColor,
                                "shouldBe " + redColor);
            debug("Checking " + (flipY ? "bottom" : "top"));
            wtu.checkCanvasRect(gl, 0, top, width, halfHeight, greenColor,
                                "shouldBe " + greenColor);
        }

        return texture;
    }

    function runTest(canvas)
    {
        var canvas = document.createElement('canvas');

        var cases = [
            { canvas: canvas, flipY: true,  font: false, init: setCanvasToMin },
            { canvas: canvas, flipY: false, font: false },
            { canvas: canvas, flipY: true,  font: false, init: setCanvasTo257x257 },
            { canvas: canvas, flipY: false, font: false },
            { canvas: canvas, flipY: true,  font: true, init: drawTextInCanvas },
            { canvas: canvas, flipY: false, font: true },
        ];

        if (window.OffscreenCanvas) {
            var offscreenCanvas = new OffscreenCanvas(1, 1);
            cases = cases.concat([
                { canvas: offscreenCanvas, flipY: true,  font: false, init: setCanvasToMin },
                { canvas: offscreenCanvas, flipY: false, font: false },
            ]);
        }

        function runTexImageTest(bindingTarget) {
            var program;
            if (bindingTarget == gl.TEXTURE_3D) {
                program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
            } else {  // TEXTURE_2D_ARRAY
                program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
            }

            return new Promise(function(resolve, reject) {
                var count = 4;
                var caseNdx = 0;
                var texture = undefined;
                function runNextTest() {
                    var c = cases[caseNdx];
                    if (c.init) {
                      c.init(c.canvas.getContext('2d'), bindingTarget);
                    }
                    texture = runOneIteration(c.canvas, c.flipY, program, bindingTarget, texture, c.font);
                    // for the first 2 iterations always make a new texture.
                    if (count < 2) {
                      gl.deleteTexture(texture);
                      texture = undefined;
                    }
                    ++caseNdx;
                    if (caseNdx == cases.length) {
                        caseNdx = 0;
                        --count;
                        if (!count) {
                            resolve("SUCCESS");
                            return;
                        }
                    }
                    wtu.waitForComposite(runNextTest);
                }
                runNextTest();
            });
        }

        runTexImageTest(gl.TEXTURE_3D).then(function(val) {
            runTexImageTest(gl.TEXTURE_2D_ARRAY).then(function(val) {
                wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
                finishTest();
            });
        });
    }

    return init;
}
