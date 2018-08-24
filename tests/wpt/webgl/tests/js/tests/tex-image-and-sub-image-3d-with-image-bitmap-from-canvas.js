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

    function init()
    {
        description('Verify texImage3D and texSubImage3D code paths taking ImageBitmap created from an HTMLCanvasElement (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

        if(!window.createImageBitmap || !window.ImageBitmap) {
            finishTest();
            return;
        }

        // Set the default context version while still allowing the webglVersion URL query string to override it.
        wtu.setDefault3DContextVersion(defaultContextVersion);
        gl = wtu.create3DContext("example");

        if (!prologue(gl)) {
            finishTest();
            return;
        }

        gl.clearColor(0,0,0,1);
        gl.clearDepth(1);

        var testCanvas = document.createElement('canvas');
        var ctx = testCanvas.getContext("2d");
        setCanvasToMin(ctx);
        runImageBitmapTest(testCanvas, 0.5, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, true)
        .then(() => {
            setCanvasTo257x257(ctx);
            return runImageBitmapTest(testCanvas, 0.5, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, true);
        }).then(() => {
            finishTest();
        });
    }

    function setCanvasToRedGreen(ctx) {
        var width = ctx.canvas.width;
        var halfWidth = Math.floor(width / 2);
        var height = ctx.canvas.height;
        var halfHeight = Math.floor(height / 2);
        ctx.fillStyle = "rgba(255, 0, 0, 1)";
        ctx.fillRect(0, 0, halfWidth, halfHeight);
        ctx.fillStyle = "rgba(255, 0, 0, 0.5)";
        ctx.fillRect(halfWidth, 0, halfWidth, halfHeight);
        ctx.fillStyle = "rgba(0, 255, 0, 1)";
        ctx.fillRect(0, halfHeight, halfWidth, halfHeight);
        ctx.fillStyle = "rgba(0, 255, 0, 0.5)";
        ctx.fillRect(halfWidth, halfHeight, halfWidth, halfHeight);
    }

    function setCanvasToMin(ctx) {
        ctx.canvas.width = 2;
        ctx.canvas.height = 2;
        setCanvasToRedGreen(ctx);
    }

    function setCanvasTo257x257(ctx) {
        ctx.canvas.width = 257;
        ctx.canvas.height = 257;
        setCanvasToRedGreen(ctx);
    }

    return init;
}
