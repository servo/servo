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
        description('Verify texImage3D and texSubImage3D code paths taking ImageBitmap created from an ImageBitmap (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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
        gl.disable(gl.BLEND);

        var imageData = new ImageData(new Uint8ClampedArray(
                                      [255, 0, 0, 255,
                                      255, 0, 0, 0,
                                      0, 255, 0, 255,
                                      0, 255, 0, 0]),
                                      2, 2);

        createImageBitmap(imageData, {imageOrientation: "none", premultiplyAlpha: "none"})
        .catch( () => {
            testPassed("createImageBitmap with options may be rejected if it is not supported. Retrying without options.");
            return createImageBitmap(imageData);
        }).then( bitmap => {
            return runImageBitmapTest(bitmap, 0, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, true);
        }, () => {
            testFailed("createImageBitmap(imageData) should succeed.");
        }).then(() => {
            finishTest();
        });
    }

    return init;
}
