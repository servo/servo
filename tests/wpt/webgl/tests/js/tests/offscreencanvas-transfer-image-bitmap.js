function testTransferToImageBitmap(webglContextVersion, bitmap) {
    var internalFormat = "RGBA";
    var pixelFormat = "RGBA";
    var pixelType = "UNSIGNED_BYTE";

    var width = 32;
    var height = 32;
    var canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    var gl = WebGLTestUtils.create3DContext(canvas);
    gl.clearColor(0,0,0,1);
    gl.clearDepth(1);
    gl.disable(gl.BLEND);

    TexImageUtils.setupTexturedQuad(gl, internalFormat);

    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    // Enable writes to the RGBA channels
    gl.colorMask(1, 1, 1, 0);
    var texture = gl.createTexture();
    // Bind the texture to texture unit 0
    gl.bindTexture(gl.TEXTURE_2D, texture);
    // Set up texture parameters
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

    var targets = [gl.TEXTURE_2D];
    // Upload the image into the texture
    for (var tt = 0; tt < targets.length; ++tt) {
        gl.texImage2D(targets[tt], 0, gl[internalFormat], gl[pixelFormat], gl[pixelType], bitmap);
    }
    for (var tt = 0; tt < targets.length; ++tt) {
        // Draw the triangles
        gl.clearColor(0, 0, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        gl.drawArrays(gl.TRIANGLES, 0, 6);

        var buf = new Uint8Array(width * height * 4);
        gl.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, buf);
        _checkCanvas(buf, width, height, webglContextVersion);
    }
}

function _checkCanvas(buf, width, height, webglContextVersion)
{
    for (var i = 0; i < width * height; i++) {
        if (buf[i * 4] != 255 || buf[i * 4 + 1] != 255 ||
            buf[i * 4 + 2] != 0 || buf[i * 4 + 3] != 255) {
            testFailed("OffscreenCanvas." + webglContextVersion +
                ": This pixel should be [255, 255, 0, 255], but it is: [" + buf[i * 4] + ", " +
                buf[i * 4 + 1] + ", " + buf[i * 4 + 2] + ", " + buf[i * 4 + 3] + "].");
            return;
        }
    }
    testPassed("TransferToImageBitmap test on OffscreenCanvas." + webglContextVersion + " passed");
}
