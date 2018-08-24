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

function generateTest(pixelFormat, pixelType, prologue) {
    var wtu = WebGLTestUtils;
    var gl = null;
    var textureLoc = null;
    var successfullyParsed = false;
    var imageData = null;

    var init = function()
    {
        description('Verify texImage2D and texSubImage2D code paths taking ImageData (' + pixelFormat + '/' + pixelType + ')');

        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        var program = wtu.setupTexturedQuad(gl);
        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);
        gl.disable(gl.BLEND);

        textureLoc = gl.getUniformLocation(program, "tex");

        var canvas2d = document.getElementById("texcanvas");
        var context2d = canvas2d.getContext("2d");
        imageData = context2d.createImageData(1, 2);
        var data = imageData.data;
        data[0] = 255;
        data[1] = 0;
        data[2] = 0;
        data[3] = 255;
        data[4] = 0;
        data[5] = 255;
        data[6] = 0;
        data[7] = 0;

        runTest();
    }

    function runOneIteration(useTexSubImage2D, flipY, premultiplyAlpha, topColor, bottomColor)
    {
        debug('Testing ' + (useTexSubImage2D ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY + ' and premultiplyAlpha=' + premultiplyAlpha);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        // Enable writes to the RGBA channels
        gl.colorMask(1, 1, 1, 0);
        var texture = gl.createTexture();
        // Bind the texture to texture unit 0
        gl.bindTexture(gl.TEXTURE_2D, texture);
        // Set up texture parameters
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        // Set up pixel store parameters
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, flipY);
        gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, premultiplyAlpha);
        // Upload the image into the texture
        if (useTexSubImage2D) {
            // Initialize the texture to black first
            gl.texImage2D(gl.TEXTURE_2D, 0, gl[pixelFormat], 1, 2, 0,
                          gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, gl[pixelFormat], gl[pixelType], imageData);
        } else {
            gl.texImage2D(gl.TEXTURE_2D, 0, gl[pixelFormat], gl[pixelFormat], gl[pixelType], imageData);
        }

        // Point the uniform sampler to texture unit 0
        gl.uniform1i(textureLoc, 0);
        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

        // Check the top pixel and bottom pixel and make sure they have
        // the right color.
        debug("Checking bottom pixel");
        wtu.checkCanvasRect(gl, 0, 0, 1, 1, bottomColor, "shouldBe " + bottomColor);
        debug("Checking top pixel");
        wtu.checkCanvasRect(gl, 0, 1, 1, 1, topColor, "shouldBe " + topColor);
    }

    function runTest()
    {
        var red = [255, 0, 0, 255];
        var green = [0, 255, 0, 255];
        var redPremultiplyAlpha = [255, 0, 0, 255];
        var greenPremultiplyAlpha = [0, 0, 0, 255];

        runOneIteration(false, true, false,
                        red, green);
        runOneIteration(false, false, false,
                        green, red);
        runOneIteration(false, true, true,
                        redPremultiplyAlpha, greenPremultiplyAlpha);
        runOneIteration(false, false, true,
                        greenPremultiplyAlpha, redPremultiplyAlpha);
        runOneIteration(true, true, false,
                        red, green);
        runOneIteration(true, false, false,
                        green, red);
        runOneIteration(true, true, true,
                        redPremultiplyAlpha, greenPremultiplyAlpha);
        runOneIteration(true, false, true,
                        greenPremultiplyAlpha, redPremultiplyAlpha);

        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
        finishTest();
    }

    return init;
}
