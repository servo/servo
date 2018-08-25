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
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];
    var blueColor = [0, 0, 255];
    var cyanColor = [0, 255, 255];

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking a sub-rectangle of a canvas (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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

        var canvas2d = document.createElement('canvas');
        runTest(canvas2d, setupSourceCanvas2D, '2D-rendered canvas');

        var canvasWebGL = document.createElement('canvas');
        runTest(canvasWebGL, setupSourceCanvasWebGL, 'WebGL-rendered canvas');

        finishTest();
    }

    function uploadCanvasToTexture(canvas, useTexSubImage3D, flipY, bindingTarget,
                                   depth, sourceSubRectangle, unpackImageHeight)
    {
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Disable any writes to the alpha channel
        gl.colorMask(1, 1, 1, 0);
        var texture = gl.createTexture();
        // Bind the texture to texture unit 0
        gl.bindTexture(bindingTarget, texture);
        // Set up texture parameters
        gl.texParameteri(bindingTarget, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.texParameteri(bindingTarget, gl.TEXTURE_WRAP_R, gl.CLAMP_TO_EDGE);
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);
        gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);
        var uploadWidth = canvas.width;
        var uploadHeight = canvas.height;
        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, sourceSubRectangle[0]);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, sourceSubRectangle[1]);
            uploadWidth = sourceSubRectangle[2];
            uploadHeight = sourceSubRectangle[3];
        }
        if (unpackImageHeight) {
            gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, unpackImageHeight);
        }
        // Upload the image into the texture
        if (useTexSubImage3D) {
            // Initialize the texture to black first
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage3D(bindingTarget, 0, 0, 0, 0, uploadWidth, uploadHeight, depth,
                             gl[pixelFormat], gl[pixelType], canvas);
        } else {
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], canvas);
        }
        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, 0);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors from texture upload");
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

    function runOneIteration(canvas, useTexSubImage3D, flipY, bindingTarget,
                             depth, sourceSubRectangle, unpackImageHeight,
                             rTextureCoord, expectedColor, program,
                             canvasSize, canvasSetupFunction, sourceDescription)
    {
        debug('');
        debug('Testing ' + sourceDescription + ' with ' +
              (useTexSubImage3D ? 'texSubImage3D' : 'texImage3D') +
              ', flipY=' + flipY + ', bindingTarget=' +
              (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY') +
              ', sourceSubRectangle=' + sourceSubRectangle +
              ', depth=' + depth +
              (unpackImageHeight ? ', unpackImageHeight=' + unpackImageHeight : '') +
              ', rTextureCoord=' + rTextureCoord);

        // Initialize the contents of the source canvas.
        var width = canvasSize[0];
        var height = canvasSize[1];
        var halfWidth = Math.floor(width / 2);
        var halfHeight = Math.floor(height / 2);
        canvas.width = width;
        canvas.height = height;
        canvasSetupFunction(canvas);

        uploadCanvasToTexture(canvas, useTexSubImage3D, flipY, bindingTarget,
                              depth, sourceSubRectangle, unpackImageHeight);
        var rCoordLocation = gl.getUniformLocation(program, 'uRCoord');
        if (!rCoordLocation) {
            testFailed('Shader incorrectly set up; couldn\'t find uRCoord uniform');
            return;
        }
        gl.uniform1f(rCoordLocation, rTextureCoord);

        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);
        // Check the rendered canvas
        wtu.checkCanvasRect(gl, 0, 0, canvasSize[0], canvasSize[1], expectedColor, "shouldBe " + expectedColor);
    }

    function runTest(canvas, canvasSetupFunction, sourceDescription)
    {
        var cases = [
            // Small canvas cases. Expected that these won't be
            // GPU-accelerated in most browsers' implementations.

            // No UNPACK_IMAGE_HEIGHT specified.
            { expected: redColor,   flipY: false, size: [4, 4], subRect: [0, 0, 2, 2], depth: 2, rTextureCoord: 0.0 },
            { expected: blueColor,  flipY: false, size: [4, 4], subRect: [0, 0, 2, 2], depth: 2, rTextureCoord: 1.0 },
            { expected: blueColor,  flipY: true,  size: [4, 4], subRect: [0, 0, 2, 2], depth: 2, rTextureCoord: 0.0 },
            { expected: redColor,   flipY: true,  size: [4, 4], subRect: [0, 0, 2, 2], depth: 2, rTextureCoord: 1.0 },
            { expected: greenColor, flipY: false, size: [4, 4], subRect: [2, 0, 2, 2], depth: 2, rTextureCoord: 0.0 },
            { expected: cyanColor,  flipY: false, size: [4, 4], subRect: [2, 0, 2, 2], depth: 2, rTextureCoord: 1.0 },
            { expected: cyanColor,  flipY: true,  size: [4, 4], subRect: [2, 0, 2, 2], depth: 2, rTextureCoord: 0.0 },
            { expected: greenColor, flipY: true,  size: [4, 4], subRect: [2, 0, 2, 2], depth: 2, rTextureCoord: 1.0 },

            // Use UNPACK_IMAGE_HEIGHT to skip some pixels.
            { expected: redColor,   flipY: false, size: [4, 4], subRect: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0 },
            { expected: blueColor,  flipY: false, size: [4, 4], subRect: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0 },
            { expected: blueColor,  flipY: true,  size: [4, 4], subRect: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0 },
            { expected: redColor,   flipY: true,  size: [4, 4], subRect: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0 },
            { expected: greenColor, flipY: false, size: [4, 4], subRect: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0 },
            { expected: cyanColor,  flipY: false, size: [4, 4], subRect: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0 },
            { expected: cyanColor,  flipY: true,  size: [4, 4], subRect: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0 },
            { expected: greenColor, flipY: true,  size: [4, 4], subRect: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0 },

            // Larger canvas cases. Expected that these will be
            // GPU-accelerated in most browsers' implementations.
            // Changes will be gladly accepted to trigger more
            // browsers' heuristics to accelerate these canvases.

            // No UNPACK_IMAGE_HEIGHT specified.
            { expected: redColor,   flipY: false, size: [384, 384], subRect: [0, 0, 192, 192], depth: 2, rTextureCoord: 0.0 },
            { expected: blueColor,  flipY: false, size: [384, 384], subRect: [0, 0, 192, 192], depth: 2, rTextureCoord: 1.0 },
            { expected: blueColor,  flipY: true,  size: [384, 384], subRect: [0, 0, 192, 192], depth: 2, rTextureCoord: 0.0 },
            { expected: redColor,   flipY: true,  size: [384, 384], subRect: [0, 0, 192, 192], depth: 2, rTextureCoord: 1.0 },
            { expected: greenColor, flipY: false, size: [384, 384], subRect: [192, 0, 192, 192], depth: 2, rTextureCoord: 0.0 },
            { expected: cyanColor,  flipY: false, size: [384, 384], subRect: [192, 0, 192, 192], depth: 2, rTextureCoord: 1.0 },
            { expected: cyanColor,  flipY: true,  size: [384, 384], subRect: [192, 0, 192, 192], depth: 2, rTextureCoord: 0.0 },
            { expected: greenColor, flipY: true,  size: [384, 384], subRect: [192, 0, 192, 192], depth: 2, rTextureCoord: 1.0 },

            // Use UNPACK_IMAGE_HEIGHT to skip some pixels.
            { expected: redColor,   flipY: false, size: [384, 384], subRect: [0, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 0.0 },
            { expected: blueColor,  flipY: false, size: [384, 384], subRect: [0, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 1.0 },
            { expected: blueColor,  flipY: true,  size: [384, 384], subRect: [0, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 0.0 },
            { expected: redColor,   flipY: true,  size: [384, 384], subRect: [0, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 1.0 },
            { expected: greenColor, flipY: false, size: [384, 384], subRect: [192, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 0.0 },
            { expected: cyanColor,  flipY: false, size: [384, 384], subRect: [192, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 1.0 },
            { expected: cyanColor,  flipY: true,  size: [384, 384], subRect: [192, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 0.0 },
            { expected: greenColor, flipY: true,  size: [384, 384], subRect: [192, 0, 96, 96], depth: 2, unpackImageHeight: 192, rTextureCoord: 1.0 },
        ];

        var program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
        for (var i in cases) {
            runOneIteration(canvas, false, cases[i].flipY, gl.TEXTURE_3D,
                            cases[i].depth, cases[i].subRect,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].expected,
                            program, cases[i].size, canvasSetupFunction, sourceDescription);
            runOneIteration(canvas, true, cases[i].flipY, gl.TEXTURE_3D,
                            cases[i].depth, cases[i].subRect,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].expected,
                            program, cases[i].size, canvasSetupFunction, sourceDescription);
        }

        program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
        for (var i in cases) {
            runOneIteration(canvas, false, cases[i].flipY, gl.TEXTURE_2D_ARRAY,
                            cases[i].depth, cases[i].subRect,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].expected,
                            program, cases[i].size, canvasSetupFunction, sourceDescription);
            runOneIteration(canvas, true, cases[i].flipY, gl.TEXTURE_2D_ARRAY,
                            cases[i].depth, cases[i].subRect,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].expected,
                            program, cases[i].size, canvasSetupFunction, sourceDescription);
        }
    }

    return init;
}
