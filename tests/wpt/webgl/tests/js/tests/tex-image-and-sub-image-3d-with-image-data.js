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
    var imageData = null;
    var blackColor = [0, 0, 0];
    var originalPixels = (function() {
        // (red|green|blue|cyan)(opaque|transparent)
        var ro = [255, 0, 0, 255]; var rt = [255, 0, 0, 0];
        var go = [0, 255, 0, 255]; var gt = [0, 255, 0, 0];
        var bo = [0, 0, 255, 255]; var bt = [0, 0, 255, 0];
        var co = [0, 255, 255, 255]; var ct = [0, 255, 255, 0];
        return [ro, rt, go, gt,
                ro, rt, go, gt,
                bo, bt, co, ct,
                bo, bt, co, ct];
    })();

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking ImageData (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);
        gl.disable(gl.BLEND);

        var canvas2d = document.getElementById("texcanvas");
        var context2d = canvas2d.getContext("2d");
        imageData = context2d.createImageData(4, 4);
        var data = imageData.data;
        for (var i = 0; i < originalPixels.length; i++) {
            data.set(originalPixels[i], 4 * i);
        }

        runTest();
    }

    function runOneIteration(useTexSubImage3D, flipY, premultiplyAlpha, bindingTarget,
                             depth, sourceSubRectangle, rTexCoord, program)
    {
        var expected = simulate(flipY, premultiplyAlpha, depth, sourceSubRectangle, rTexCoord);
        var sourceSubRectangleString = '';
        if (sourceSubRectangle) {
            sourceSubRectangleString = ', sourceSubRectangle=' + sourceSubRectangle;
            sourceSubRectangleString += ', rTexCoord=' + rTexCoord;
        }
        debug('');
        debug('Testing ' + (useTexSubImage3D ? 'texSubImage3D' : 'texImage3D') +
              ' with flipY=' + flipY + ', premultiplyAlpha=' + premultiplyAlpha +
              ', bindingTarget=' + (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY') +
              sourceSubRectangleString);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Enable writes to the RGBA channels
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
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, premultiplyAlpha);
        gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL, gl.NONE);
        var uploadWidth = imageData.width;
        var uploadHeight = imageData.height;
        if (sourceSubRectangle) {
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, sourceSubRectangle[0]);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, sourceSubRectangle[1]);
            uploadWidth = sourceSubRectangle[2];
            uploadHeight = sourceSubRectangle[3];
        }
        // Upload the image into the texture
        if (useTexSubImage3D) {
            // Initialize the texture to black first
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage3D(bindingTarget, 0, 0, 0, 0, uploadWidth, uploadHeight, depth,
                             gl[pixelFormat], gl[pixelType], imageData);
        } else {
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], uploadWidth, uploadHeight, depth, 0,
                          gl[pixelFormat], gl[pixelType], imageData);
        }
        gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
        gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors from texture upload");

        var tl = expected[0][0];
        var tr = expected[0][1];
        var bl = expected[1][0];
        var br = expected[1][1];

        var rCoordLocation = gl.getUniformLocation(program, 'uRCoord');
        if (!rCoordLocation) {
            testFailed("Shader incorrectly set up; couldn't find uRCoord uniform");
            return;
        }
        gl.uniform1f(rCoordLocation, rTexCoord);
        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

        var width = gl.canvas.width;
        var halfWidth = Math.floor(width / 2);
        var height = gl.canvas.height;
        var halfHeight = Math.floor(height / 2);

        var top = 0;
        var bottom = height - halfHeight;
        var left = 0;
        var right = width - halfWidth;

        debug("Checking pixel values");
        debug("Expecting: " + expected);
        var expectedH = expected.length;
        var expectedW = expected[0].length;
        var texelH = Math.floor(gl.canvas.height / expectedH);
        var texelW = Math.floor(gl.canvas.width / expectedW);
        // For each entry of the expected[][] array, check the appropriate
        // canvas rectangle for correctness.
        for (var row = 0; row < expectedH; row++) {
            var y = row * texelH;
            for (var col = 0; col < expectedW; col++) {
                var x = col * texelW;
                var val = expected[row][col];
                wtu.checkCanvasRect(gl, x, y, texelW, texelH, val, "should be " + val);
            }
        }
    }

    function runTest()
    {
        var program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_3D, program);
        program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_2D_ARRAY, program);

        debug("");
        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
        finishTest();
    }

    function simulate(flipY, premultiplyAlpha, depth, sourceSubRectangle, rTexCoord) {
        var ro = [255, 0, 0];   var rt = premultiplyAlpha ? [0, 0, 0] : [255, 0, 0];
        var go = [0, 255, 0];   var gt = premultiplyAlpha ? [0, 0, 0] : [0, 255, 0];
        var bo = [0, 0, 255];   var bt = premultiplyAlpha ? [0, 0, 0] : [0, 0, 255];
        var co = [0, 255, 255]; var ct = premultiplyAlpha ? [0, 0, 0] : [0, 255, 255];
        var expected = [[ro, rt, go, gt],
                        [ro, rt, go, gt],
                        [bo, bt, co, ct],
                        [bo, bt, co, ct]];
        switch (gl[pixelFormat]) {
          case gl.RED:
          case gl.RED_INTEGER:
            for (var row = 0; row < 4; row++) {
                for (var col = 0; col < 4; col++) {
                    expected[row][col][1] = 0; // zero the green channel
                }
            }
            // fall-through
          case gl.RG:
          case gl.RG_INTEGER:
            for (var row = 0; row < 4; row++) {
                for (var col = 0; col < 4; col++) {
                    expected[row][col][2] = 0; // zero the blue channel
                }
            }
            break;
          default:
            break;
        }

        if (flipY) {
            expected.reverse();
        }

        if (sourceSubRectangle) {
            let expected2 = [];
            for (var row = 0; row < sourceSubRectangle[3]; row++) {
                expected2[row] = [];
                for (var col = 0; col < sourceSubRectangle[2]; col++) {
                    expected2[row][col] =
                        expected[sourceSubRectangle[1] + row + rTexCoord * sourceSubRectangle[3]][sourceSubRectangle[0] + col];
                }
            }
            expected = expected2;
        }

        return expected;
    }

    function runTestOnBindingTarget(bindingTarget, program) {
        var rects = [
            undefined,
            [0, 0, 2, 2],
            [2, 0, 2, 2],
        ];
        var dbg = false;  // Set to true for debug output images
        if (dbg) {
            (function() {
                debug("");
                debug("Original ImageData (transparent pixels appear black):");
                var cvs = document.createElement("canvas");
                cvs.width = 4;
                cvs.height = 4;
                cvs.style.width = "32px";
                cvs.style.height = "32px";
                cvs.style.imageRendering = "pixelated";
                cvs.style.background = "#000";
                var ctx = cvs.getContext("2d");
                ctx.putImageData(imageData, 0, 0);
                var output = document.getElementById("console");
                output.appendChild(cvs);
            })();
        }
        for (const sub of [false, true]) {
            for (const flipY of [false, true]) {
                for (const premul of [false, true]) {
                    for (let irect = 0; irect < rects.length; irect++) {
                        var rect = rects[irect];
                        let depth = rect ? 2 : 1;
                        for (let rTexCoord = 0; rTexCoord < depth; rTexCoord++) {
                            // TODO: add tests for UNPACK_IMAGE_HEIGHT.
                            runOneIteration(sub, flipY, premul, bindingTarget,
                                    depth, rect, rTexCoord, program);
                            if (dbg) {
                                debug("Actual:");
                                var img = document.createElement("img");
                                img.src = gl.canvas.toDataURL("image/png");
                                var output = document.getElementById("console");
                                output.appendChild(img);
                            }
                        }
                    }
                }
            }
        }
    }

    return init;
}
