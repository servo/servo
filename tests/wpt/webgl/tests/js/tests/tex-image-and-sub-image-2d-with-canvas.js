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
    var redColor = [255, 0, 0, 255];
    var greenColor = [0, 255, 0, 255];
    var semiTransparentRedColor = [127, 0, 0, 127];
    var semiTransparentGreenColor = [0, 127, 0, 127];
    var repeatCount;

    function replicateRedChannel(color)
    {
        color[1] = color[0];
        color[2] = color[0];
    }

    function zapColorChannels(color)
    {
        color[0] = 0;
        color[1] = 0;
        color[2] = 0;
    }

    function setAlphaChannelTo1(color)
    {
        color[3] = 255;
    }

    function replicateAllRedChannels()
    {
        replicateRedChannel(redColor);
        replicateRedChannel(semiTransparentRedColor);
        replicateRedChannel(greenColor);
        replicateRedChannel(semiTransparentGreenColor);
    }

    function setAllAlphaChannelsTo1()
    {
        setAlphaChannelTo1(redColor);
        setAlphaChannelTo1(semiTransparentRedColor);
        setAlphaChannelTo1(greenColor);
        setAlphaChannelTo1(semiTransparentGreenColor);
    }

    function repeatCountForTextureFormat(internalFormat, pixelFormat, pixelType)
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
        if ((internalFormat == 'RGBA' && pixelFormat == 'RGBA' && pixelType == 'UNSIGNED_BYTE') ||
            (internalFormat == 'RGB' && pixelFormat == 'RGB' && pixelType == 'UNSIGNED_BYTE')) {
            return 4;
        }

        return 1;
    }

    function init()
    {
        description('Verify texImage2D and texSubImage2D code paths taking canvas elements (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        repeatCount = repeatCountForTextureFormat(internalFormat, pixelFormat, pixelType);

        switch (gl[pixelFormat]) {
          case gl.RED:
          case gl.RED_INTEGER:
            // Zap green and blue channels.
            whiteColor[1] = 0;
            whiteColor[2] = 0;
            greenColor[1] = 0;
            semiTransparentGreenColor[1] = 0;
            // Alpha channel is 1.0.
            setAllAlphaChannelsTo1();
            break;
          case gl.RG:
          case gl.RG_INTEGER:
            // Zap blue channel.
            whiteColor[2] = 0;
            // Alpha channel is 1.0.
            setAllAlphaChannelsTo1();
            break;
          case gl.LUMINANCE:
            // Replicate red channels.
            replicateAllRedChannels();
            // Alpha channel is 1.0.
            setAllAlphaChannelsTo1();
            break;
          case gl.ALPHA:
            // Red, green and blue channels are all 0.0.
            zapColorChannels(redColor);
            zapColorChannels(semiTransparentRedColor);
            zapColorChannels(greenColor);
            zapColorChannels(semiTransparentGreenColor);
            zapColorChannels(whiteColor);
            break;
          case gl.LUMINANCE_ALPHA:
            // Replicate red channels.
            replicateAllRedChannels();
            break;
          case gl.RGB:
          case gl.RGB_INTEGER:
            // Alpha channel is 1.0.
            setAllAlphaChannelsTo1();
            break;
          default:
            break;
        }

        switch (gl[internalFormat]) {
          case gl.SRGB8:
          case gl.SRGB8_ALPHA8:
            semiTransparentRedColor = wtu.sRGBToLinear(semiTransparentRedColor);
            semiTransparentGreenColor = wtu.sRGBToLinear(semiTransparentGreenColor);
            break;
          case gl.RGBA8UI:
            // For int and uint textures, TexImageUtils outputs the maximum value (in this case,
            // 255) for the alpha channel all the time because of differences in behavior when
            // sampling integer textures with and without alpha channels. Since changing this
            // behavior may have large impact across the test suite, leave it as is for now.
            setAllAlphaChannelsTo1();
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
      ctx.clearRect(0, 0, width, height);
      ctx.fillStyle = "#ff0000";
      ctx.fillRect(0, 0, width, halfHeight);
      ctx.fillStyle = "#00ff00";
      ctx.fillRect(0, halfHeight, width, height - halfHeight);
    }

    function setCanvasToSemiTransparentRedGreen(ctx) {
      var width = ctx.canvas.width;
      var height = ctx.canvas.height;
      var halfHeight = Math.floor(height / 2);
      ctx.clearRect(0, 0, width, height);
      ctx.fillStyle = "rgba(127, 0, 0, 0.5)";
      ctx.fillRect(0, 0, width, halfHeight);
      ctx.fillStyle = "rgba(0, 127, 0, 0.5)";
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

    function setCanvasTo257x257SemiTransparent(ctx, bindingTarget) {
      ctx.canvas.width = 257;
      ctx.canvas.height = 257;
      setCanvasToSemiTransparentRedGreen(ctx);
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

    function setCanvasToMinSemiTransparent(ctx, bindingTarget) {
      if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
        // cube map texture must be square.
        ctx.canvas.width = 2;
      } else {
        ctx.canvas.width = 1;
      }
      ctx.canvas.height = 2;
      setCanvasToSemiTransparentRedGreen(ctx);
    }

    function runOneIteration(canvas, useTexSubImage2D, flipY, semiTransparent, program, bindingTarget, opt_texture, opt_fontTest)
    {
        var objType = 'canvas';
        if (canvas.transferToImageBitmap)
          objType = 'OffscreenCanvas';
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY + ' bindingTarget=' + (bindingTarget == gl.TEXTURE_2D ? 'TEXTURE_2D' : 'TEXTURE_CUBE_MAP') +
              ' canvas size: ' + canvas.width + 'x' + canvas.height +
              ' source object type: ' + objType +
              (opt_fontTest ? " with fonts" : " with" + (semiTransparent ? " semi-transparent" : "") + " red-green"));
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
        var top = flipY ? 0 : (height - halfHeight);
        var bottom = flipY ? (height - halfHeight) : 0;

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
                        testPassed("font rendered");
                      },
                      debug);
            } else {
                var localRed   = semiTransparent ? semiTransparentRedColor : redColor;
                var localGreen = semiTransparent ? semiTransparentGreenColor : greenColor;

                // Allow a tolerance for premultiplication/unmultiplication, especially for texture
                // formats with lower bit depths.
                var tolerance = 0;
                if (semiTransparent) {
                    tolerance = 3;
                    if (pixelType == 'UNSIGNED_SHORT_5_6_5' || internalFormat == 'RGB565') {
                        tolerance = 6;
                    } else if (pixelType == 'UNSIGNED_SHORT_4_4_4_4' || internalFormat == 'RGBA4') {
                        tolerance = 9;
                    } else if (pixelType == 'UNSIGNED_SHORT_5_5_5_1' || internalFormat == 'RGB5_A1') {
                        // Semi-transparent values are allowed to convert to either 1 or 0 for this
                        // single-bit alpha format per OpenGL ES 3.0.5 section 2.1.6.2, "Conversion
                        // from Floating-Point to Normalized Fixed-Point". Ignore alpha for these
                        // tests.
                        tolerance = 6;
                        localRed = localRed.slice(0, 3);
                        localGreen = localGreen.slice(0, 3);
                    } else if (internalFormat == 'RGB10_A2') {
                        // The alpha channel is too low-resolution for any meaningful comparisons.
                        localRed = localRed.slice(0, 3);
                        localGreen = localGreen.slice(0, 3);
                    }
                }

                // Check the top and bottom halves and make sure they have the right color.
                debug("Checking " + (flipY ? "top" : "bottom"));
                wtu.checkCanvasRect(gl, 0, bottom, width, halfHeight, localRed,
                                    "shouldBe " + localRed, tolerance);
                debug("Checking " + (flipY ? "bottom" : "top"));
                wtu.checkCanvasRect(gl, 0, top, width, halfHeight, localGreen,
                                    "shouldBe " + localGreen, tolerance);
            }

            if (!useTexSubImage2D && pixelFormat == "RGBA") {
                if (pixelType == "FLOAT") {
                    // Attempt to set a pixel in the texture to ensure the texture was
                    // actually created with floats. Regression test for http://crbug.com/484968
                    var pixels = new Float32Array([1000.0, 1000.0, 1000.0, 1000.0]);
                    gl.texSubImage2D(targets[tt], 0, 0, 0, 1, 1, gl[pixelFormat], gl[pixelType], pixels);
                    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Texture should be backed by floats");
                } else if (pixelType == "HALF_FLOAT_OES" || pixelType == "HALF_FLOAT") {
                    // Attempt to set a pixel in the texture to ensure the texture was
                    // actually created with half-floats. Regression test for http://crbug.com/484968
                    var halfFloatTenK = 0x70E2; // Half float 10000
                    var pixels = new Uint16Array([halfFloatTenK, halfFloatTenK, halfFloatTenK, halfFloatTenK]);
                    gl.texSubImage2D(targets[tt], 0, 0, 0, 1, 1, gl[pixelFormat], gl[pixelType], pixels);
                    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "Texture should be backed by half-floats");
                }
            }
        }

        if (false) {
          var m = wtu.makeImageFromCanvas(gl.canvas);
          document.getElementById("console").appendChild(m);
          document.getElementById("console").appendChild(document.createElement("hr"));
        }

        return texture;
    }

    function runTest()
    {
        var canvas = document.createElement('canvas');

        var cases = [
            { canvas: canvas, sub: false, flipY: true,  semiTransparent: false, font: false, init: setCanvasToMin },
            { canvas: canvas, sub: false, flipY: false, semiTransparent: false, font: false },
            { canvas: canvas, sub: true,  flipY: true,  semiTransparent: false, font: false },
            { canvas: canvas, sub: true,  flipY: false, semiTransparent: false, font: false },
            { canvas: canvas, sub: false, flipY: true,  semiTransparent: true,  font: false, init: setCanvasToMinSemiTransparent },
            { canvas: canvas, sub: false, flipY: false, semiTransparent: true,  font: false },
            { canvas: canvas, sub: true,  flipY: true,  semiTransparent: true,  font: false },
            { canvas: canvas, sub: true,  flipY: false, semiTransparent: true,  font: false },
            { canvas: canvas, sub: false, flipY: true,  semiTransparent: false, font: false, init: setCanvasTo257x257 },
            { canvas: canvas, sub: false, flipY: false, semiTransparent: false, font: false },
            { canvas: canvas, sub: true,  flipY: true,  semiTransparent: false, font: false },
            { canvas: canvas, sub: true,  flipY: false, semiTransparent: false, font: false },
            { canvas: canvas, sub: false, flipY: true,  semiTransparent: true,  font: false, init: setCanvasTo257x257SemiTransparent },
            { canvas: canvas, sub: false, flipY: false, semiTransparent: true,  font: false },
            { canvas: canvas, sub: true,  flipY: true,  semiTransparent: true,  font: false },
            { canvas: canvas, sub: true,  flipY: false, semiTransparent: true,  font: false },
        ];

        // The font tests don't work with ALPHA-only textures since they draw to the color channels.
        if (internalFormat != 'ALPHA') {
            cases = cases.concat([
                { canvas: canvas, sub: false, flipY: true,  semiTransparent: false, font: true, init: drawTextInCanvas },
                { canvas: canvas, sub: false, flipY: false, semiTransparent: false, font: true },
                { canvas: canvas, sub: true,  flipY: true,  semiTransparent: false, font: true },
                { canvas: canvas, sub: true,  flipY: false, semiTransparent: false, font: true },
            ]);
        }

        if (window.OffscreenCanvas) {
            var offscreenCanvas = new OffscreenCanvas(1, 1);
            cases = cases.concat([
                { canvas: offscreenCanvas, sub: false, flipY: true,  semiTransparent: false, font: false, init: setCanvasToMin },
                { canvas: offscreenCanvas, sub: false, flipY: false, semiTransparent: false, font: false },
                { canvas: offscreenCanvas, sub: true,  flipY: true,  semiTransparent: false, font: false },
                { canvas: offscreenCanvas, sub: true,  flipY: false, semiTransparent: false, font: false },
                { canvas: offscreenCanvas, sub: false, flipY: true,  semiTransparent: true,  font: false, init: setCanvasToMinSemiTransparent },
                { canvas: offscreenCanvas, sub: false, flipY: false, semiTransparent: true,  font: false },
                { canvas: offscreenCanvas, sub: true,  flipY: true,  semiTransparent: true,  font: false },
                { canvas: offscreenCanvas, sub: true,  flipY: false, semiTransparent: true,  font: false },
            ]);
        }

        function runTexImageTest(bindingTarget) {
            var program;
            if (bindingTarget == gl.TEXTURE_2D) {
                program = tiu.setupTexturedQuad(gl, internalFormat);
            } else {
                program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
            }

            return new Promise(function(resolve, reject) {
                var count = repeatCount;
                var caseNdx = 0;
                var texture = undefined;
                function runNextTest() {
                    var c = cases[caseNdx];
                    var imageDataBefore = null;
                    if (c.init) {
                      c.init(c.canvas.getContext('2d'), bindingTarget);
                    }
                    texture = runOneIteration(c.canvas, c.sub, c.flipY, c.semiTransparent, program, bindingTarget, texture, c.font);
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
                    // While we are working with Canvases, it's really unlikely that
                    // waiting for composition will change anything here, and it's much
                    // slower, so just dispatchPromise. If we want to test with composites,
                    // we should test a more narrow subset of tests.
                    wtu.dispatchPromise(runNextTest);
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
