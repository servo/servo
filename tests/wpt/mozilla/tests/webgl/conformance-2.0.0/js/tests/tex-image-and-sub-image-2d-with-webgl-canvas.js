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

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];

    function init()
    {
        description('Verify texImage2D and texSubImage2D code paths taking webgl canvas elements (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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
            greenColor = [0, 0, 0];
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

      ctx.viewport(0, 0, width, height);

      ctx.enable(ctx.SCISSOR_TEST);
      ctx.scissor(0, 0, width, halfHeight);
      ctx.clearColor(1.0, 0, 0, 1.0);
      ctx.clear(ctx.COLOR_BUFFER_BIT);
      ctx.scissor(0, halfHeight, width, height - halfHeight);
      ctx.clearColor(0.0, 1.0, 0, 1.0);
      ctx.clear(ctx.COLOR_BUFFER_BIT);
      ctx.disable(ctx.SCISSOR_TEST);
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

    function runOneIteration(canvas, useTexSubImage2D, flipY, program, bindingTarget, opt_texture)
    {
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') + ' with flipY=' +
              flipY + ' bindingTarget=' + (bindingTarget == gl.TEXTURE_2D ? 'TEXTURE_2D' : 'TEXTURE_CUBE_MAP') +
              ' canvas size: ' + canvas.width + 'x' + canvas.height + ' with red-green');
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        if (!opt_texture) {
            var texture = gl.createTexture();
            // Bind the texture to texture unit 0
            gl.bindTexture(bindingTarget, texture);
            // Set up texture parameters
            gl.texParameteri(bindingTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
            gl.texParameteri(bindingTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
            gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
            gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        } else {
            var texture = opt_texture;
        }
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        wtu.failIfGLError(gl, 'gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);');
        var targets = [gl.TEXTURE_2D];
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            targets = [gl.TEXTURE_CUBE_MAP_POSITIVE_X,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
                       gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
                       gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_Z];
        }
        // Upload the image into the texture
        for (var tt = 0; tt < targets.length; ++tt) {
            // Initialize the texture to black first
            if (useTexSubImage2D) {
                gl.texImage2D(targets[tt], 0, gl[internalFormat], canvas.width, canvas.height, 0,
                              gl[pixelFormat], gl[pixelType], null);
                gl.texSubImage2D(targets[tt], 0, 0, 0, gl[pixelFormat], gl[pixelType], canvas);
            } else {
                gl.texImage2D(targets[tt], 0, gl[internalFormat], gl[pixelFormat], gl[pixelType], canvas);
            }
        }

        var width = gl.canvas.width;
        var height = gl.canvas.height;
        var halfWidth = Math.floor(width / 2);
        var halfHeight = Math.floor(height / 2);
        var top = flipY ? (height - halfHeight) : 0;
        var bottom = flipY ? 0 : (height - halfHeight);

        var loc;
        var skipCorner = false;
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            loc = gl.getUniformLocation(program, "face");
            switch (gl[pixelFormat]) {
              case gl.RED_INTEGER:
              case gl.RG_INTEGER:
              case gl.RGB_INTEGER:
              case gl.RGBA_INTEGER:
                // https://github.com/KhronosGroup/WebGL/issues/1819
                skipCorner = true;
                break;
            }
        }

        for (var tt = 0; tt < targets.length; ++tt) {
            if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                gl.uniform1i(loc, targets[tt]);
            }
            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 255, 0, 255]);

            // Check the top and bottom halves and make sure they have the right color.
            debug("Checking " + (flipY ? "top" : "bottom"));
            wtu.checkCanvasRect(gl, 0, bottom, (skipCorner && !flipY) ? halfWidth : width, halfHeight, redColor,
                    "shouldBe " + redColor);
            debug("Checking " + (flipY ? "bottom" : "top"));
            wtu.checkCanvasRect(gl, 0, top, (skipCorner && flipY) ? halfWidth : width, halfHeight, greenColor,
                    "shouldBe " + greenColor);
        }

        if (false) {
          var ma = wtu.makeImageFromCanvas(canvas);
          document.getElementById("console").appendChild(ma);

          var m = wtu.makeImageFromCanvas(gl.canvas);
          document.getElementById("console").appendChild(m);
          document.getElementById("console").appendChild(document.createElement("hr"));
        }

        return texture;
    }

    function runTest()
    {
        var ctx = wtu.create3DContext();
        var canvas = ctx.canvas;

        var cases = [
            { sub: false, flipY: true, init: setCanvasToMin },
            { sub: false, flipY: false },
            { sub: true,  flipY: true },
            { sub: true,  flipY: false },
            { sub: false, flipY: true, init: setCanvasTo257x257 },
            { sub: false, flipY: false },
            { sub: true,  flipY: true },
            { sub: true,  flipY: false },
        ];

        function runTexImageTest(bindingTarget) {
            var program;
            if (bindingTarget == gl.TEXTURE_2D) {
                program = tiu.setupTexturedQuad(gl, internalFormat);
            } else {
                program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
            }

            return new Promise(function(resolve, reject) {
                var count = 4;
                var caseNdx = 0;
                var texture = undefined;
                function runNextTest() {
                    var c = cases[caseNdx];
                    if (c.init) {
                      c.init(ctx, bindingTarget);
                    }
                    texture = runOneIteration(canvas, c.sub, c.flipY, program, bindingTarget, texture);
                    // for the first 2 iterations always make a new texture.
                    if (count > 2) {
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

        runTexImageTest(gl.TEXTURE_2D).then(function(val) {
            runTexImageTest(gl.TEXTURE_CUBE_MAP).then(function(val) {
                wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
                finishTest();
            });
        });
    }

    return init;
}
