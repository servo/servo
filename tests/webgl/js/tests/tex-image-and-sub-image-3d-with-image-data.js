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

function generateTest(internalFormat, pixelFormat, pixelType, prologue, resourcePath) {
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
        description('Verify texImage3D and texSubImage3D code paths taking ImageData (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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

    function runOneIteration(flipY, premultiplyAlpha, bindingTarget, program)
    {
        debug('Testing ' + ' with flipY=' + flipY + ' and premultiplyAlpha=' + premultiplyAlpha +
              ', bindingTarget=' + (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY'));
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
        // Upload the image into the texture
        // Initialize the texture to black first
        gl.texImage3D(bindingTarget, 0, gl[internalFormat], imageData.width, imageData.height, 1 /* depth */, 0,
                      gl[pixelFormat], gl[pixelType], null);
        gl.texSubImage3D(bindingTarget, 0, 0, 0, 0, gl[pixelFormat], gl[pixelType], imageData);

        var width = gl.canvas.width;
        var halfWidth = Math.floor(width / 2);
        var height = gl.canvas.height;
        var halfHeight = Math.floor(height / 2);

        var top = flipY ? 0 : (height - halfHeight);
        var bottom = flipY ? (height - halfHeight) : 0;

        var tl = redColor;
        var tr = premultiplyAlpha ? blackColor : redColor;
        var bl = greenColor;
        var br = premultiplyAlpha ? blackColor : greenColor;

        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

        // Check the top pixel and bottom pixel and make sure they have
        // the right color.
        debug("Checking " + (flipY ? "top" : "bottom"));
        wtu.checkCanvasRect(gl, 0, bottom, halfWidth, halfHeight, tl, "shouldBe " + tl);
        wtu.checkCanvasRect(gl, halfWidth, bottom, width, halfHeight, tr, "shouldBe " + tr);
        debug("Checking " + (flipY ? "bottom" : "top"));
        wtu.checkCanvasRect(gl, 0, top, halfWidth, halfHeight, bl, "shouldBe " + bl);
        wtu.checkCanvasRect(gl, halfWidth, top, width, halfHeight, br, "shouldBe " + br);
    }

    function runTest()
    {
        var program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_3D, program);
        program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
        runTestOnBindingTarget(gl.TEXTURE_2D_ARRAY, program);

        wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
        finishTest();
    }

    function runTestOnBindingTarget(bindingTarget, program) {
        var cases = [
            { flipY: true, premultiplyAlpha: false },
            { flipY: false, premultiplyAlpha: false },
            { flipY: true, premultiplyAlpha: true },
            { flipY: false, premultiplyAlpha: true },
        ];

        for (var i in cases) {
            runOneIteration(cases[i].flipY, cases[i].premultiplyAlpha,
                            bindingTarget, program);
        }
    }

    return init;
}
