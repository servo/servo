/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

"use strict";

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;
    var redColor = [255, 0, 0, 255];
    var greenColor = [0, 255, 0, 255];
    var repeatCount;

    function shouldRepeatTestForTextureFormat(internalFormat, pixelFormat, pixelType)
    {
        // There were bugs in early WebGL 1.0 implementations when repeatedly uploading canvas
        // elements into textures. In response, this test was changed into a regression test by
        // repeating all of the cases multiple times. Unfortunately, this means that adding a new
        // case above significantly increases the run time of the test suite. The problem is made
        // even worse by the addition of many more texture formats in WebGL 2.0.
        //
        // Doing repeated runs with just a couple of WebGL 1.0's supported texture formats acts as a
        // sufficient regression test for the old bugs. For this reason the test has been changed to
        // only repeat for those texture formats.
        return ((internalFormat == 'RGBA' && pixelFormat == 'RGBA' && pixelType == 'UNSIGNED_BYTE') ||
                (internalFormat == 'RGB' && pixelFormat == 'RGB' && pixelType == 'UNSIGNED_BYTE'));
    }

    async function init()
    {
        description('Verify texImage2D and texSubImage2D code paths taking webgl canvas elements (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            return;
        }

        repeatCount = (shouldRepeatTestForTextureFormat(internalFormat, pixelFormat, pixelType) ? 4 : 1);

        switch (gl[pixelFormat]) {
          case gl.RED:
          case gl.RED_INTEGER:
            greenColor = [0, 0, 0];
            break;
          case gl.LUMINANCE:
          case gl.LUMINANCE_ALPHA:
            redColor = [255, 255, 255];
            greenColor = [0, 0, 0];
            break;
          case gl.ALPHA:
            redColor = [0, 0, 0];
            greenColor = [0, 0, 0];
            break;
          default:
            break;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

        await runTest();
    }

    function setCanvasToRedGreen(ctx, hasAlpha) {
      var width = ctx.canvas.width;
      var height = ctx.canvas.height;
      var halfHeight = Math.floor(height / 2);

      ctx.viewport(0, 0, width, height);

      ctx.enable(ctx.SCISSOR_TEST);
      ctx.scissor(0, 0, width, halfHeight);
      if (hasAlpha) {
        ctx.clearColor(1.0, 0, 0, 1.0);
      } else {
        // The WebGL implementation is responsible for making all
        // alpha values appear as though they were 1.0.
        ctx.clearColor(1.0, 0, 0, 0.0);
      }
      ctx.clear(ctx.COLOR_BUFFER_BIT);
      ctx.scissor(0, halfHeight, width, height - halfHeight);
      if (hasAlpha) {
        ctx.clearColor(0.0, 1.0, 0, 1.0);
      } else {
        // The WebGL implementation is responsible for making all
        // alpha values appear as though they were 1.0.
        ctx.clearColor(0.0, 1.0, 0, 0.0);
      }
      ctx.clear(ctx.COLOR_BUFFER_BIT);
      ctx.disable(ctx.SCISSOR_TEST);
    }

    function setCanvasTo257x257(ctx, bindingTarget, hasAlpha) {
      ctx.canvas.width = 257;
      ctx.canvas.height = 257;
      setCanvasToRedGreen(ctx, hasAlpha);
    }

    function setCanvasToMin(ctx, bindingTarget, hasAlpha) {
      if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
        // cube map texture must be square.
        ctx.canvas.width = 2;
      } else {
        ctx.canvas.width = 1;
      }
      ctx.canvas.height = 2;
      setCanvasToRedGreen(ctx, hasAlpha);
    }

    function runOneIteration(canvas, useTexSubImage2D, alpha, flipY, program, bindingTarget, opt_texture)
    {
        var objType = 'canvas';
        if (canvas.transferToImageBitmap)
            objType = 'OffscreenCanvas';
        else if (canvas.parentNode)
            objType = 'canvas attached to DOM';
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') + ' with alpha=' +
              alpha + ' flipY=' + flipY + ' source object: ' + objType +
              ' bindingTarget=' + (bindingTarget == gl.TEXTURE_2D ? 'TEXTURE_2D' : 'TEXTURE_CUBE_MAP') +
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
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors before pixelStorei setup");
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors after setting UNPACK_FLIP_Y_WEBGL");
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors after setting UNPACK_PREMULTIPLY_ALPHA_WEBGL");
        gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors after setting UNPACK_COLORSPACE_CONVERSION_WEBGL");
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
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            loc = gl.getUniformLocation(program, "face");
        }

        for (var tt = 0; tt < targets.length; ++tt) {
            if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                gl.uniform1i(loc, targets[tt]);
            }
            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 255, 0, 255]);

            // Check the top and bottom halves and make sure they have the right color.
            debug("Checking " + (flipY ? "top" : "bottom"));
            wtu.checkCanvasRect(gl, 0, bottom, width, halfHeight, redColor,
                    "shouldBe " + redColor);
            debug("Checking " + (flipY ? "bottom" : "top"));
            wtu.checkCanvasRect(gl, 0, top, width, halfHeight, greenColor,
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

    async function runTest()
    {
        for (let alpha of [ true, false ]) {
            let ctx = wtu.create3DContext(null, { alpha:alpha });
            let canvas = ctx.canvas;
            // Note: We use preserveDrawingBuffer:true to prevent canvas
            // visibility from interfering with the tests.
            let visibleCtx = wtu.create3DContext(null, { preserveDrawingBuffer:true, alpha:alpha });
            if (!visibleCtx) {
                testFailed("context does not exist");
                return;
            }
            let visibleCanvas = visibleCtx.canvas;
            let descriptionNode = document.getElementById("description");
            document.body.insertBefore(visibleCanvas, descriptionNode);

            let cases = [
                { sub: false, flipY: true,  ctx: ctx, init: setCanvasToMin },
                { sub: false, flipY: false, ctx: ctx },
                { sub: true,  flipY: true,  ctx: ctx },
                { sub: true,  flipY: false, ctx: ctx },
                { sub: false, flipY: true,  ctx: ctx, init: setCanvasTo257x257 },
                { sub: false, flipY: false, ctx: ctx },
                { sub: true,  flipY: true,  ctx: ctx },
                { sub: true,  flipY: false, ctx: ctx },
                { sub: false, flipY: true,  ctx: visibleCtx, init: setCanvasToMin },
                { sub: false, flipY: false, ctx: visibleCtx },
                { sub: true,  flipY: true,  ctx: visibleCtx },
                { sub: true,  flipY: false, ctx: visibleCtx },
            ];

            if (window.OffscreenCanvas) {
                let offscreen = new OffscreenCanvas(1, 1);
                let offscreenCtx = wtu.create3DContext(offscreen, { alpha:alpha });
                cases = cases.concat([
                    { sub: false, flipY: true,  ctx: offscreenCtx, init: setCanvasToMin },
                    { sub: false, flipY: false, ctx: offscreenCtx },
                    { sub: true,  flipY: true,  ctx: offscreenCtx },
                    { sub: true,  flipY: false, ctx: offscreenCtx },
                ]);
            }

            async function runTexImageTest(bindingTarget) {
                let program;
                if (bindingTarget == gl.TEXTURE_2D) {
                    program = tiu.setupTexturedQuad(gl, internalFormat);
                } else {
                    program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
                }

                let count = repeatCount;
                let caseNdx = 0;
                let texture = undefined;
                while (true) {
                    let c = cases[caseNdx];
                    if (c.init) {
                        c.init(c.ctx, bindingTarget, alpha);
                    }
                    texture = runOneIteration(c.ctx.canvas, c.sub, alpha, c.flipY, program, bindingTarget, texture);
                    // for the first 2 iterations always make a new texture.
                    if (count < 2) {
                        gl.deleteTexture(texture);
                        texture = undefined;
                    }
                    ++caseNdx;
                    if (caseNdx == cases.length) {
                        caseNdx = 0;
                        --count;
                        if (!count)
                            return;
                    }
                    await wtu.dispatchPromise(function() {});
                }
            }

            await runTexImageTest(gl.TEXTURE_2D);
            await runTexImageTest(gl.TEXTURE_CUBE_MAP);
        }

        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }

    return function() {
        init().then(function(val) {
            finishTest();
        });
    };
}
