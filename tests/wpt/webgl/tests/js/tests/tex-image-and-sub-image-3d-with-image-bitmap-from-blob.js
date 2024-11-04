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
        description('Verify texImage3D and texSubImage3D code paths taking ImageBitmap created from a Blob (' + internalFormat + '/' + pixelFormat + '/' + pixelType + ')');

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

        var xhr = new XMLHttpRequest();
        xhr.open("GET", resourcePath + "red-green-semi-transparent.png");
        xhr.responseType = 'blob';
        xhr.onload = function() {
            var blob = xhr.response;
            runImageBitmapTest(blob, 0.5, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, true)
            .then(() => {
                finishTest();
            });
        };
        xhr.send();
    }

    return init;
}
