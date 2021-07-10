/*
** Copyright (c) 2015 The Khronos Group Inc.
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
    var imgCanvas;
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];
    var blueColor = [0, 0, 255];
    var cyanColor = [0, 255, 255];
    var imageURLs = [resourcePath + "red-green.png",
                     resourcePath + "red-green-blue-cyan-4x4.png"];

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking image elements (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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

        wtu.loadImagesAsync(imageURLs, runTest);
    }

    function uploadImageToTexture(image, useTexSubImage3D, flipY, bindingTarget,
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
        var uploadWidth = image.width;
        var uploadHeight = image.height;
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
                             gl[pixelFormat], gl[pixelType], image);
        } else {
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], image);
        }
        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, 0);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors from texture upload");
    }

    function runRedGreenTest(image) {
        function runOneIteration(image, useTexSubImage3D, flipY, bindingTarget, topColor, bottomColor, program)
        {
            debug('Testing ' + (useTexSubImage3D ? 'texSubImage3D' : 'texImage3D') +
                  ' with flipY=' + flipY + ' bindingTarget=' +
                  (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY'));

            uploadImageToTexture(image, useTexSubImage3D, flipY, bindingTarget, 1);

            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);
            // Check a few pixels near the top and bottom and make sure they have
            // the right color.
            debug("Checking lower left corner");
            wtu.checkCanvasRect(gl, 4, 4, 2, 2, bottomColor,
                                "shouldBe " + bottomColor);
            debug("Checking upper left corner");
            wtu.checkCanvasRect(gl, 4, gl.canvas.height - 8, 2, 2, topColor,
                                "shouldBe " + topColor);
        }

        var cases = [
            { sub: false, flipY: true, topColor: redColor, bottomColor: greenColor },
            { sub: false, flipY: false, topColor: greenColor, bottomColor: redColor },
            { sub: true, flipY: true, topColor: redColor, bottomColor: greenColor },
            { sub: true, flipY: false, topColor: greenColor, bottomColor: redColor },
        ];

        var program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
        for (var i in cases) {
            runOneIteration(image, cases[i].sub, cases[i].flipY, gl.TEXTURE_3D,
                            cases[i].topColor, cases[i].bottomColor, program);
        }
        program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
        for (var i in cases) {
            runOneIteration(image, cases[i].sub, cases[i].flipY, gl.TEXTURE_2D_ARRAY,
                            cases[i].topColor, cases[i].bottomColor, program);
        }
    }

    function runRedGreenBlueCyanTest(image) {
        function runOneIteration(image, useTexSubImage3D, flipY, bindingTarget,
                                 depth, sourceSubRectangle, unpackImageHeight,
                                 rTextureCoord, topColor, bottomColor, program)
        {
            sourceSubRectangleString = '';
            if (sourceSubRectangle) {
                sourceSubRectangleString = ' sourceSubRectangle=' + sourceSubRectangle;
            }
            unpackImageHeightString = '';
            if (unpackImageHeight) {
                unpackImageHeightString = ' unpackImageHeight=' + unpackImageHeight;
            }
            debug('Testing ' + (useTexSubImage3D ? 'texSubImage3D' : 'texImage3D') +
                  ' with flipY=' + flipY + ' bindingTarget=' +
                  (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY') +
                  sourceSubRectangleString + ' depth=' + depth + unpackImageHeightString +
                  ' rTextureCoord=' + rTextureCoord);

            uploadImageToTexture(image, useTexSubImage3D, flipY, bindingTarget,
                                 depth, sourceSubRectangle, unpackImageHeight);
            var rCoordLocation = gl.getUniformLocation(program, 'uRCoord');
            if (!rCoordLocation) {
                testFailed('Shader incorrectly set up; couldn\'t find uRCoord uniform');
                return;
            }
            gl.uniform1f(rCoordLocation, rTextureCoord);

            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);
            // Check a few pixels near the top and bottom and make sure they have
            // the right color.
            debug("Checking lower left corner");
            wtu.checkCanvasRect(gl, 4, 4, 2, 2, bottomColor,
                                "shouldBe " + bottomColor);
            debug("Checking upper left corner");
            wtu.checkCanvasRect(gl, 4, gl.canvas.height - 8, 2, 2, topColor,
                                "shouldBe " + topColor);
        }

        var cases = [
            // No UNPACK_IMAGE_HEIGHT specified.
            { flipY: false, sourceSubRectangle: [0, 0, 2, 2], depth: 2, rTextureCoord: 0.0,
              topColor: redColor, bottomColor: redColor },
            { flipY: false, sourceSubRectangle: [0, 0, 2, 2], depth: 2, rTextureCoord: 1.0,
              topColor: blueColor, bottomColor: blueColor },
            { flipY: true, sourceSubRectangle: [0, 0, 2, 2], depth: 2, rTextureCoord: 0.0,
              topColor: blueColor, bottomColor: blueColor },
            { flipY: true, sourceSubRectangle: [0, 0, 2, 2], depth: 2, rTextureCoord: 1.0,
              topColor: redColor, bottomColor: redColor },
            { flipY: false, sourceSubRectangle: [2, 0, 2, 2], depth: 2, rTextureCoord: 0.0,
              topColor: greenColor, bottomColor: greenColor },
            { flipY: false, sourceSubRectangle: [2, 0, 2, 2], depth: 2, rTextureCoord: 1.0,
              topColor: cyanColor, bottomColor: cyanColor },
            { flipY: true, sourceSubRectangle: [2, 0, 2, 2], depth: 2, rTextureCoord: 0.0,
              topColor: cyanColor, bottomColor: cyanColor },
            { flipY: true, sourceSubRectangle: [2, 0, 2, 2], depth: 2, rTextureCoord: 1.0,
              topColor: greenColor, bottomColor: greenColor },

            // Use UNPACK_IMAGE_HEIGHT to skip some pixels.
            { flipY: false, sourceSubRectangle: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0,
              topColor: redColor, bottomColor: redColor },
            { flipY: false, sourceSubRectangle: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0,
              topColor: blueColor, bottomColor: blueColor },
            { flipY: true, sourceSubRectangle: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0,
              topColor: blueColor, bottomColor: blueColor },
            { flipY: true, sourceSubRectangle: [0, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0,
              topColor: redColor, bottomColor: redColor },
            { flipY: false, sourceSubRectangle: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0,
              topColor: greenColor, bottomColor: greenColor },
            { flipY: false, sourceSubRectangle: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0,
              topColor: cyanColor, bottomColor: cyanColor },
            { flipY: true, sourceSubRectangle: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 0.0,
              topColor: cyanColor, bottomColor: cyanColor },
            { flipY: true, sourceSubRectangle: [2, 0, 1, 1], depth: 2, unpackImageHeight: 2, rTextureCoord: 1.0,
              topColor: greenColor, bottomColor: greenColor },
        ];

        var program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
        for (var i in cases) {
            runOneIteration(image, false, cases[i].flipY, gl.TEXTURE_3D,
                            cases[i].depth, cases[i].sourceSubRectangle,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].topColor, cases[i].bottomColor,
                            program);
            runOneIteration(image, true, cases[i].flipY, gl.TEXTURE_3D,
                            cases[i].depth, cases[i].sourceSubRectangle,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].topColor, cases[i].bottomColor,
                            program);
        }

        program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
        for (var i in cases) {
            runOneIteration(image, false, cases[i].flipY, gl.TEXTURE_2D_ARRAY,
                            cases[i].depth, cases[i].sourceSubRectangle,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].topColor, cases[i].bottomColor,
                            program);
            runOneIteration(image, true, cases[i].flipY, gl.TEXTURE_2D_ARRAY,
                            cases[i].depth, cases[i].sourceSubRectangle,
                            cases[i].unpackImageHeight, cases[i].rTextureCoord,
                            cases[i].topColor, cases[i].bottomColor,
                            program);
        }
    }

    function runTest(imageMap)
    {
        runRedGreenTest(imageMap[imageURLs[0]]);
        runRedGreenBlueCyanTest(imageMap[imageURLs[1]]);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
        finishTest();
    }

    return init;
}
