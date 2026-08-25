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
