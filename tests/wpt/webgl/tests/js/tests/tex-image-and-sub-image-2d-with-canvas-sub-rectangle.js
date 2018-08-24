/*
** Copyright (c) 2016 The Khronos Group Inc.
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
    var realRedColor = [255, 0, 0];
    var realGreenColor = [0, 255, 0];
    var realBlueColor = [0, 0, 255];
    var realCyanColor = [0, 255, 255];
    var redColor = realRedColor;
    var greenColor = realGreenColor;
    var blueColor = realBlueColor;
    var cyanColor = realCyanColor;

    function init()
    {
        description('Verify texImage2D and texSubImage2D code paths taking a sub-rectangle of a canvas (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);

        // The sub-rectangle tests only apply to WebGL 2.0 for the
        // time being, though the tests for the WebGL 1.0
        // format/internal format/type combinations are generated into
        // conformance/textures/.
        if (wtu.getDefault3DContextVersion() < 2) {
            debug('Test only applies to WebGL 2.0');
            finishTest();
            return;
        }

        gl = wtu.create3DContext("example", { preserveDrawingBuffer: true });

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        switch (gl[pixelFormat]) {
        case gl.RED:
        case gl.RED_INTEGER:
          greenColor = [0, 0, 0];
          blueColor = [0, 0, 0];
          cyanColor = [0, 0, 0];
          break;

        case gl.RG:
        case gl.RG_INTEGER:
          blueColor = [0, 0, 0];
          cyanColor = [0, 255, 0];
          break;

        default:
          break;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);
        gl.disable(gl.BLEND);

        var canvas2d = document.createElement('canvas');
        runTest(canvas2d, setupSourceCanvas2D, '2D-rendered canvas');

        var canvasWebGL = document.createElement('canvas');
        runTest(canvasWebGL, setupSourceCanvasWebGL, 'WebGL-rendered canvas');

        finishTest();
    }

    function fillStyle2D(ctx, color) {
        ctx.fillStyle = 'rgb(' + color[0] + ', ' + color[1] + ', ' + color[2] + ')';
    }

    function setupSourceCanvas2D(canvas) {
        var width = canvas.width;
        var height = canvas.height;
        var halfWidth = Math.floor(width / 2);
        var halfHeight = Math.floor(height / 2);

        var ctx = canvas.getContext('2d');
        // Always use the same pattern for this test: four quadrants:
        //   red    green
        //   blue   cyan
        // Handle odd-sized canvases
        fillStyle2D(ctx, realRedColor);
        ctx.fillRect(0, 0, halfWidth, halfHeight);
        fillStyle2D(ctx, realGreenColor);
        ctx.fillRect(halfWidth, 0, width - halfWidth, halfHeight);
        fillStyle2D(ctx, realBlueColor);
        ctx.fillRect(0, halfHeight, halfWidth, height - halfHeight);
        fillStyle2D(ctx, realCyanColor);
        ctx.fillRect(halfWidth, halfHeight, width - halfWidth, height - halfHeight);
    }

    function clearColorWebGL(ctx, color) {
        ctx.clearColor(color[0] / 255.0, color[1] / 255.0, color[2] / 255.0, 1.0);
        ctx.clear(ctx.COLOR_BUFFER_BIT);
    }

    function setupSourceCanvasWebGL(canvas) {
        var width = canvas.width;
        var height = canvas.height;
        var halfWidth = Math.floor(width / 2);
        var halfHeight = Math.floor(height / 2);

        var ctx = canvas.getContext('webgl');
        // Always use the same pattern for this test: four quadrants:
        //   red    green
        //   blue   cyan
        // Handle odd-sized canvases

        ctx.viewport(0, 0, width, height);
        ctx.enable(ctx.SCISSOR_TEST);
        // OpenGL origin is lower-left
        ctx.scissor(0, 0, halfWidth, halfHeight);
        clearColorWebGL(ctx, realBlueColor);
        ctx.scissor(halfWidth, 0, width - halfWidth, halfHeight);
        clearColorWebGL(ctx, realCyanColor);
        ctx.scissor(0, halfHeight, halfWidth, height - halfHeight);
        clearColorWebGL(ctx, realRedColor);
        ctx.scissor(halfWidth, halfHeight, width - halfWidth, height - halfHeight);
        clearColorWebGL(ctx, realGreenColor);
    }

    function runOneIteration(sourceDescription, useTexSubImage2D, flipY,
                             canvas, canvasSize, canvasSetupFunction,
                             sourceSubRectangle, expected,
                             bindingTarget, program)
    {
        sourceSubRectangleString = '';
        if (sourceSubRectangle) {
            sourceSubRectangleString = ', sourceSubRectangle=' + sourceSubRectangle;
        }
        debug('');
        debug('Testing ' + sourceDescription + ' with ' +
              (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ', flipY=' + flipY +
              ', bindingTarget=' + (bindingTarget == gl.TEXTURE_2D ? 'TEXTURE_2D' : 'TEXTURE_CUBE_MAP') +
              sourceSubRectangleString);

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

        if (skipCorner && sourceSubRectangle &&
                sourceSubRectangle[2] == 1 && sourceSubRectangle[3] == 1) {
            debug("Test skipped, see WebGL#1819");
            return;
        }

        // Initialize the contents of the source canvas.
        var width = canvasSize[0];
        var height = canvasSize[1];
        var halfWidth = Math.floor(width / 2);
        var halfHeight = Math.floor(height / 2);
        canvas.width = width;
        canvas.height = height;
        canvasSetupFunction(canvas);

        // Upload the source canvas to the texture and draw it to a quad.
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Enable writes to the RGBA channels
        gl.colorMask(1, 1, 1, 0);
        var texture = gl.createTexture();
        // Bind the texture to texture unit 0
        gl.bindTexture(bindingTarget, texture);
        // Set up texture parameters
        gl.texParameteri(bindingTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
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
        // In this test, this is always specified. It's currently WebGL 2.0-specific.
        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, sourceSubRectangle[0]);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, sourceSubRectangle[1]);
        // Upload the image into the texture
        var uploadWidth = sourceSubRectangle[2];
        var uploadHeight = sourceSubRectangle[3];
        for (var tt = 0; tt < targets.length; ++tt) {
            if (useTexSubImage2D) {
                // Initialize the texture to black first
                gl.texImage2D(targets[tt], 0, gl[internalFormat],
                              uploadWidth, uploadHeight, 0,
                              gl[pixelFormat], gl[pixelType], null);
                gl.texSubImage2D(targets[tt], 0, 0, 0,
                                 uploadWidth, uploadHeight,
                                 gl[pixelFormat], gl[pixelType], canvas);
            } else {
                gl.texImage2D(targets[tt], 0, gl[internalFormat],
                              uploadWidth, uploadHeight, 0,
                              gl[pixelFormat], gl[pixelType], canvas);
            }
        }

        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);

        // The tests are constructed to upload a single solid color
        // out of the canvas.
        var outputCanvasWidth = gl.drawingBufferWidth;
        var outputCanvasHeight = gl.drawingBufferHeight;
        var outputCanvasHalfWidth = Math.floor(outputCanvasWidth / 2);
        var outputCanvasHalfHeight = Math.floor(outputCanvasHeight / 2);
        var top = 0;
        var bottom = outputCanvasHeight - outputCanvasHalfHeight;
        var left = 0;
        var right = outputCanvasWidth - outputCanvasHalfWidth;

        for (var tt = 0; tt < targets.length; ++tt) {
            if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                gl.uniform1i(loc, targets[tt]);
            }
            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

            // Check the four quadrants and make sure they have the right color.
            // This is split up into four tests only because of the driver bug above.
            var msg = 'should be ' + expected;
            wtu.checkCanvasRect(gl, left, top, outputCanvasHalfWidth, outputCanvasHalfHeight, expected, msg);
            if (!skipCorner) {
                wtu.checkCanvasRect(gl, right, top, outputCanvasHalfWidth, outputCanvasHalfHeight, expected, msg);
            }
            wtu.checkCanvasRect(gl, left, bottom, outputCanvasHalfWidth, outputCanvasHalfHeight, expected, msg);
            if (!skipCorner) {
                wtu.checkCanvasRect(gl, right, bottom, outputCanvasHalfWidth, outputCanvasHalfHeight, expected, msg);
            }
        }
    }

    function runTest(canvas, canvasSetupFunction, sourceDescription)
    {
        var program = tiu.setupTexturedQuad(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_2D, program, canvas, canvasSetupFunction, sourceDescription);
        program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_CUBE_MAP, program, canvas, canvasSetupFunction, sourceDescription);

        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
    }

    function runTestOnBindingTarget(bindingTarget, program, canvas, canvasSetupFunction, sourceDescription) {
        var cases = [
            // Small canvas cases. Expected that these won't be
            // GPU-accelerated in most browsers' implementations.
            { expected: redColor,   flipY: false, size: [2, 2], subRect: [0, 0, 1, 1] },
            { expected: greenColor, flipY: false, size: [2, 2], subRect: [1, 0, 1, 1] },
            { expected: blueColor,  flipY: false, size: [2, 2], subRect: [0, 1, 1, 1] },
            { expected: cyanColor,  flipY: false, size: [2, 2], subRect: [1, 1, 1, 1] },
            { expected: redColor,   flipY: true,  size: [2, 2], subRect: [0, 1, 1, 1] },
            { expected: greenColor, flipY: true,  size: [2, 2], subRect: [1, 1, 1, 1] },
            { expected: blueColor,  flipY: true,  size: [2, 2], subRect: [0, 0, 1, 1] },
            { expected: cyanColor,  flipY: true,  size: [2, 2], subRect: [1, 0, 1, 1] },

            // Larger canvas cases. Expected that these will be
            // GPU-accelerated in most browsers' implementations.
            // Changes will be gladly accepted to trigger more
            // browsers' heuristics to accelerate these canvases.
            { expected: redColor,   flipY: false, size: [384, 384], subRect: [  0,   0, 192, 192] },
            { expected: greenColor, flipY: false, size: [384, 384], subRect: [192,   0, 192, 192] },
            { expected: blueColor,  flipY: false, size: [384, 384], subRect: [  0, 192, 192, 192] },
            { expected: cyanColor,  flipY: false, size: [384, 384], subRect: [192, 192, 192, 192] },
            { expected: blueColor,  flipY: true,  size: [384, 384], subRect: [  0,   0, 192, 192] },
            { expected: cyanColor,  flipY: true,  size: [384, 384], subRect: [192,   0, 192, 192] },
            { expected: redColor,   flipY: true,  size: [384, 384], subRect: [  0, 192, 192, 192] },
            { expected: greenColor, flipY: true,  size: [384, 384], subRect: [192, 192, 192, 192] },

        ];

        for (var i in cases) {
            runOneIteration(sourceDescription, false, cases[i].flipY,
                            canvas, cases[i].size, canvasSetupFunction,
                            cases[i].subRect,
                            cases[i].expected, bindingTarget, program);

            // In Chrome, this hits a bug on Mac with Intel GPU.
            // Chromium bug: crbug.com/665656
            // Apple Radar: 29563996
            //runOneIteration(sourceDescription, true, cases[i].flipY,
            //                canvas, cases[i].size, canvasSetupFunction,
            //                cases[i].subRect,
            //                cases[i].expected, bindingTarget, program);
        }
    }

    return init;
}
