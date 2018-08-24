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

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath, defaultContextVersion) {
    var wtu = WebGLTestUtils;
    var tiu = TexImageUtils;
    var gl = null;
    var successfullyParsed = false;
    var imageData = null;
    var blackColor = [0, 0, 0];
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];

    function init()
    {
        description('Verify texImage2D and texSubImage2D code paths taking ImageData (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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
        gl.disable(gl.BLEND);

        var canvas2d = document.getElementById("texcanvas");
        var context2d = canvas2d.getContext("2d");
        imageData = context2d.createImageData(2, 2);
        var data = imageData.data;
        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        data[3] = 255;
        data[4] = 255;
        data[5] = 0;
        data[6] = 0;
        data[7] = 0;
        data[8] = 0;
        data[9] = 255;
        data[10] = 0;
        data[11] = 255;
        data[12] = 0;
        data[13] = 255;
        data[14] = 0;
        data[15] = 0;

        runTest();
    }

    function runOneIteration(useTexSubImage2D, flipY, premultiplyAlpha,
                             sourceSubRectangle, expected,
                             bindingTarget, program)
    {
        sourceSubRectangleString = '';
        if (sourceSubRectangle) {
            sourceSubRectangleString = ', sourceSubRectangle=' + sourceSubRectangle;
        }
        debug('');
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY + ' and premultiplyAlpha=' + premultiplyAlpha +
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

        if (skipCorner && expected.length == 1 && (flipY ^ sourceSubRectangle[1] == 0)) {
            debug("Test skipped, see WebGL#1819");
            return;
        }

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
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, premultiplyAlpha);
        var targets = [gl.TEXTURE_2D];
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            targets = [gl.TEXTURE_CUBE_MAP_POSITIVE_X,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
                       gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
                       gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
                       gl.TEXTURE_CUBE_MAP_NEGATIVE_Z];
        }
        // Handle the source sub-rectangle if specified (WebGL 2.0 only)
        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, sourceSubRectangle[0]);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, sourceSubRectangle[1]);
        }
        // Upload the image into the texture
        for (var tt = 0; tt < targets.length; ++tt) {
            if (sourceSubRectangle) {
                if (useTexSubImage2D) {
                    // Initialize the texture to black first
                    gl.texImage2D(targets[tt], 0, gl[internalFormat],
                                  sourceSubRectangle[2], sourceSubRectangle[3], 0,
                                  gl[pixelFormat], gl[pixelType], null);
                    gl.texSubImage2D(targets[tt], 0, 0, 0,
                                     sourceSubRectangle[2], sourceSubRectangle[3],
                                     gl[pixelFormat], gl[pixelType], imageData);
                } else {
                    gl.texImage2D(targets[tt], 0, gl[internalFormat],
                                  sourceSubRectangle[2], sourceSubRectangle[3], 0,
                                  gl[pixelFormat], gl[pixelType], imageData);
                }
            } else {
                if (useTexSubImage2D) {
                    // Initialize the texture to black first
                    gl.texImage2D(targets[tt], 0, gl[internalFormat], imageData.width, imageData.height, 0,
                                  gl[pixelFormat], gl[pixelType], null);
                    gl.texSubImage2D(targets[tt], 0, 0, 0, gl[pixelFormat], gl[pixelType], imageData);
                } else {
                    gl.texImage2D(targets[tt], 0, gl[internalFormat], gl[pixelFormat], gl[pixelType], imageData);
                }
            }
        }

        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        }

        var width = gl.canvas.width;
        var halfWidth = Math.floor(width / 2);
        var height = gl.canvas.height;
        var halfHeight = Math.floor(height / 2);

        var top = 0;
        var bottom = height - halfHeight;
        var left = 0;
        var right = width - halfWidth;

        var tl, tr, bl, br;
        if (expected.length == 1) {
            tl = tr = bl = br = expected[0];
        } else {
            tl = expected[0];
            tr = expected[1];
            bl = expected[2];
            br = expected[3];
        }

        for (var tt = 0; tt < targets.length; ++tt) {
            if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
                gl.uniform1i(loc, targets[tt]);
            }
            // Draw the triangles
            wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

            // Check the top pixel and bottom pixel and make sure they have
            // the right color.
            wtu.checkCanvasRect(gl, left, top, halfWidth, halfHeight, tl, "shouldBe " + tl);
            if (!skipCorner) {
                wtu.checkCanvasRect(gl, right, top, halfWidth, halfHeight, tr, "shouldBe " + tr);
            }
            wtu.checkCanvasRect(gl, left, bottom, halfWidth, halfHeight, bl, "shouldBe " + bl);
            if (!skipCorner) {
                wtu.checkCanvasRect(gl, right, bottom, halfWidth, halfHeight, br, "shouldBe " + br);
            }
        }
    }

    function runTest()
    {
        var program = tiu.setupTexturedQuad(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_2D, program);
        program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_CUBE_MAP, program);

        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
        finishTest();
    }

    function runTestOnBindingTarget(bindingTarget, program) {
        var k = blackColor;
        var r = redColor;
        var g = greenColor;
        var cases = [
            { expected: [r, r, g, g], flipY: false, premultiplyAlpha: false, sub: false },
            { expected: [r, r, g, g], flipY: false, premultiplyAlpha: false, sub: true },
            { expected: [r, k, g, k], flipY: false, premultiplyAlpha: true, sub: false },
            { expected: [r, k, g, k], flipY: false, premultiplyAlpha: true, sub: true },
            { expected: [g, g, r, r], flipY: true, premultiplyAlpha: false, sub: false },
            { expected: [g, g, r, r], flipY: true, premultiplyAlpha: false, sub: true },
            { expected: [g, k, r, k], flipY: true, premultiplyAlpha: true, sub: false },
            { expected: [g, k, r, k], flipY: true, premultiplyAlpha: true, sub: true },
        ];

        if (wtu.getDefault3DContextVersion() > 1) {
            var morecases = [];
            // Make 2 copies of the original case: top left and bottom right 1x1 rectangles
            for (var i = 0; i < cases.length; i++) {
                for (var subX = 0; subX <= 1; subX++) {
                    var subY = subX == 0 ? 1 : 0;
                    // shallow-copy cases[i] into newcase
                    var newcase = Object.assign({}, cases[i]);
                    newcase.expected = [cases[i].expected[subY * 2 + subX]];
                    newcase.sourceSubRectangle = [subX, subY, 1, 1];
                    morecases.push(newcase);
                }
            }
            cases = cases.concat(morecases);
        }

        for (var i in cases) {
            runOneIteration(cases[i].sub, cases[i].flipY, cases[i].premultiplyAlpha,
                            cases[i].sourceSubRectangle, cases[i].expected,
                            bindingTarget, program);
        }
    }

    return init;
}
