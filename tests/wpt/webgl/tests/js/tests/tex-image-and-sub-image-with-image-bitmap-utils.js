/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


function runOneIterationImageBitmapTest(useTexSubImage, bindingTarget, program, bitmap, flipY, premultiplyAlpha, optionsVal,
    internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance)
{
    var halfRed = [128, 0, 0];
    var halfGreen = [0, 128, 0];
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];
    var blackColor = [0, 0, 0];

    switch (gl[pixelFormat]) {
      case gl.RED:
      case gl.RED_INTEGER:
        greenColor = [0, 0, 0];
        halfGreen = [0, 0, 0];
        break;
      case gl.LUMINANCE:
      case gl.LUMINANCE_ALPHA:
        redColor = [255, 255, 255];
        greenColor = [0, 0, 0];
        halfRed = [128, 128, 128];
        halfGreen = [0, 0, 0];
        break;
      case gl.ALPHA:
        redColor = [0, 0, 0];
        greenColor = [0, 0, 0];
        halfRed = [0, 0, 0];
        halfGreen = [0, 0, 0];
        break;
      default:
        break;
    }

    switch (gl[internalFormat]) {
      case gl.SRGB8:
      case gl.SRGB8_ALPHA8:
        halfRed = wtu.sRGBToLinear(halfRed);
        halfGreen = wtu.sRGBToLinear(halfGreen);
        break;
      default:
        break;
    }

    var str;
    if (optionsVal.is3D) {
        str = 'Testing ' + (useTexSubImage ? 'texSubImage3D' : 'texImage3D') +
            ' with flipY=' + flipY + ', premultiplyAlpha=' + premultiplyAlpha +
            ', bindingTarget=' + (bindingTarget == gl.TEXTURE_3D ? 'TEXTURE_3D' : 'TEXTURE_2D_ARRAY');
    } else {
        str = 'Testing ' + (useTexSubImage ? 'texSubImage2D' : 'texImage2D') +
              ' with flipY=' + flipY + ', premultiplyAlpha=' + premultiplyAlpha +
              ', bindingTarget=' + (bindingTarget == gl.TEXTURE_2D ? 'TEXTURE_2D' : 'TEXTURE_CUBE_MAP');
    }
    debug(str);
    bufferedLogToConsole(str);

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

    var targets = [bindingTarget];
    if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
        targets = [gl.TEXTURE_CUBE_MAP_POSITIVE_X,
                   gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
                   gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
                   gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
                   gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
                   gl.TEXTURE_CUBE_MAP_NEGATIVE_Z];
    }

    bufferedLogToConsole("Start uploading the image into a texture");
    // Upload the image into the texture
    for (var tt = 0; tt < targets.length; ++tt) {
        if (optionsVal.is3D) {
            gl.texImage3D(targets[tt], 0, gl[internalFormat], bitmap.width, bitmap.height, 1 /* depth */, 0,
                    gl[pixelFormat], gl[pixelType], null);
            gl.texSubImage3D(targets[tt], 0, 0, 0, 0, bitmap.width, bitmap.height, 1,
                             gl[pixelFormat], gl[pixelType], bitmap);
        } else {
            if (useTexSubImage) {
                // Initialize the texture to black first
                gl.texImage2D(targets[tt], 0, gl[internalFormat], bitmap.width, bitmap.height, 0,
                              gl[pixelFormat], gl[pixelType], null);
                gl.texSubImage2D(targets[tt], 0, 0, 0, gl[pixelFormat], gl[pixelType], bitmap);
            } else {
                gl.texImage2D(targets[tt], 0, gl[internalFormat], gl[pixelFormat], gl[pixelType], bitmap);
            }
        }
    }
    bufferedLogToConsole("Uploading into texture completed");

    var width = gl.canvas.width;
    var halfWidth = Math.floor(width / 2);
    var quarterWidth = Math.floor(halfWidth / 2);
    var height = gl.canvas.height;
    var halfHeight = Math.floor(height / 2);
    var quarterHeight = Math.floor(halfHeight / 2);

    var top = flipY ? quarterHeight : (height - halfHeight + quarterHeight);
    var bottom = flipY ? (height - halfHeight + quarterHeight) : quarterHeight;
    var left = quarterWidth;
    var right = halfWidth + quarterWidth / 2;

    var tl = redColor;
    var tr = premultiplyAlpha ? ((optionsVal.alpha == 0.5) ? halfRed : (optionsVal.alpha == 1) ? redColor : blackColor) : redColor;
    var bl = greenColor;
    var br = premultiplyAlpha ? ((optionsVal.alpha == 0.5) ? halfGreen : (optionsVal.alpha == 1) ? greenColor : blackColor) : greenColor;

    var loc;
    if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
        loc = gl.getUniformLocation(program, "face");
    }

    for (var tt = 0; tt < targets.length; ++tt) {
        if (bindingTarget == gl.TEXTURE_CUBE_MAP) {
            gl.uniform1i(loc, targets[tt]);
        }
        // Draw the triangles
        wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

        // Check the top pixel and bottom pixel and make sure they have
        // the right color.
        let skipAlphaTests = (premultiplyAlpha === undefined && optionsVal.alpha != 1.0);
        let skipStr = " (Skipping checking right pixel since premultiplyAlpha was undefined and alpha != 1.0)";
        bufferedLogToConsole("Checking " + (flipY ? "top" : "bottom"));
        wtu.checkCanvasRect(gl, left, bottom, 2, 2, tl, "shouldBe " + tl + " +/-" + tolerance, tolerance);
        if (skipAlphaTests) {
            bufferedLogToConsole(skipStr);
        } else {
            wtu.checkCanvasRect(gl, right, bottom, 2, 2, tr, "shouldBe " + tr + " +/-" + tolerance, tolerance);
        }
        bufferedLogToConsole("Checking " + (flipY ? "bottom" : "top"));
        wtu.checkCanvasRect(gl, left, top, 2, 2, bl, "shouldBe " + bl + " +/-" + tolerance, tolerance);
        if (skipAlphaTests) {
            bufferedLogToConsole(skipStr);
        } else {
            wtu.checkCanvasRect(gl, right, top, 2, 2, br, "shouldBe " + br + " +/-" + tolerance, tolerance);
        }
    }
    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
}

function resetUnpackParams(gl)
{
    gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
    gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
    gl.pixelStorei(gl.UNPACK_SKIP_IMAGES, 0);
    gl.pixelStorei(gl.UNPACK_ROW_LENGTH, 0);
    gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, 0);
}

function runOneIterationImageBitmapTestSubSource(useTexSubImage, bindingTarget, program, bitmap, flipY, premultiplyAlpha, optionsVal,
    internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance)
{
    var halfRed = [128, 0, 0];
    var halfGreen = [0, 128, 0];
    var redColor = [255, 0, 0];
    var greenColor = [0, 255, 0];
    var blackColor = [0, 0, 0];

    switch (gl[pixelFormat]) {
      case gl.RED:
      case gl.RED_INTEGER:
        greenColor = [0, 0, 0];
        halfGreen = [0, 0, 0];
        break;
      case gl.LUMINANCE:
      case gl.LUMINANCE_ALPHA:
        redColor = [255, 255, 255];
        greenColor = [0, 0, 0];
        halfRed = [128, 128, 128];
        halfGreen = [0, 0, 0];
        break;
      case gl.ALPHA:
        redColor = [0, 0, 0];
        greenColor = [0, 0, 0];
        halfRed = [0, 0, 0];
        halfGreen = [0, 0, 0];
        break;
      default:
        break;
    }

    switch (gl[internalFormat]) {
      case gl.SRGB8:
      case gl.SRGB8_ALPHA8:
        halfRed = wtu.sRGBToLinear(halfRed);
        halfGreen = wtu.sRGBToLinear(halfGreen);
        break;
      default:
        break;
    }

    var str;
    if (optionsVal.is3D) {
        str = 'Testing ' + (useTexSubImage ? 'texSubImage3D' : 'texImage3D') + '[SubSource]' +
            ' with flipY=' + flipY + ', premultiplyAlpha=' + premultiplyAlpha +
            ', bindingTarget=TEXTURE_3D';
    } else {
        str = 'Testing ' + (useTexSubImage ? 'texSubImage2D' : 'texImage2D') + '[SubSource]' +
            ' with flipY=' + flipY + ', premultiplyAlpha=' + premultiplyAlpha +
            ', bindingTarget=TEXTURE_2D';
    }
    debug(str);
    bufferedLogToConsole(str);

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

    var srcTL = redColor;
    var srcTR = premultiplyAlpha ? ((optionsVal.alpha == 0.5) ? halfRed : (optionsVal.alpha == 1) ? redColor : blackColor) : redColor;
    var srcBL = greenColor;
    var srcBR = premultiplyAlpha ? ((optionsVal.alpha == 0.5) ? halfGreen : (optionsVal.alpha == 1) ? greenColor : blackColor) : greenColor;

    var tl, tr, bl, br;

    bufferedLogToConsole("Start uploading the image into a texture");
    // Upload the image into the texture
    if (optionsVal.is3D) {
        if (useTexSubImage) {
            // Initialize the texture to black first
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], bitmap.width, bitmap.height, 1 /* depth */, 0,
                          gl[pixelFormat], gl[pixelType], null);
            // Only upload the left half image to the right half texture.
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
            gl.pixelStorei(gl.UNPACK_SKIP_IMAGES, 0);
            gl.texSubImage3D(bindingTarget, 0, bitmap.width / 2, 0, 0, bitmap.width / 2, bitmap.height, 1,
                             gl[pixelFormat], gl[pixelType], bitmap);
            tl = blackColor;
            tr = srcTL;
            bl = blackColor;
            br = srcBL;
        } else {
            // Only upload the bottom middle quarter image
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, bitmap.height / 2);
            gl.pixelStorei(gl.UNPACK_SKIP_IMAGES, 0);
            gl.texImage3D(bindingTarget, 0, gl[internalFormat], bitmap.width, bitmap.height / 2, 1 /* depth */, 0,
                          gl[pixelFormat], gl[pixelType], bitmap);
            if (!flipY) {
                tl = srcBL;
                tr = srcBR;
                bl = srcBL;
                br = srcBR;
            } else {
                tl = srcTL;
                tr = srcTR;
                bl = srcTL;
                br = srcTR;
            }
        }
    } else {
        if (useTexSubImage) {
            // Initialize the texture to black first
            gl.texImage2D(bindingTarget, 0, gl[internalFormat], bitmap.width, bitmap.height, 0,
                          gl[pixelFormat], gl[pixelType], null);
            // Only upload the left half image to the right half texture.
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, 0);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, 0);
            gl.texSubImage2D(bindingTarget, 0, bitmap.width / 2, 0, bitmap.width / 2, bitmap.height,
                             gl[pixelFormat], gl[pixelType], bitmap);
            tl = blackColor;
            tr = srcTL;
            bl = blackColor;
            br = srcBL;
        } else {
            // Only upload the right bottom image.
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, bitmap.width / 2);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, bitmap.height / 2);
            gl.texImage2D(bindingTarget, 0, gl[internalFormat], bitmap.width / 2, bitmap.height / 2, 0,
                          gl[pixelFormat], gl[pixelType], bitmap);
            resetUnpackParams(gl);
            if (!flipY) {
                tl = srcBR;
                tr = srcBR;
                bl = srcBR;
                br = srcBR;
            } else {
                tl = srcTR;
                tr = srcTR;
                bl = srcTR;
                br = srcTR;
            }
        }
    }
    bufferedLogToConsole("Uploading into texture completed");

    var width = gl.canvas.width;
    var halfWidth = Math.floor(width / 2);
    var quarterWidth = Math.floor(halfWidth / 2);
    var height = gl.canvas.height;
    var halfHeight = Math.floor(height / 2);
    var quarterHeight = Math.floor(halfHeight / 2);

    var top = flipY ? quarterHeight : (height - halfHeight + quarterHeight);
    var bottom = flipY ? (height - halfHeight + quarterHeight) : quarterHeight;

    // Draw the triangles
    wtu.clearAndDrawUnitQuad(gl, [0, 0, 0, 255]);

    // Check the top pixel and bottom pixel and make sure they have
    // the right color.
    // For right side, check pixels closer to left to avoid border in the video tests.
    let skipAlphaTests = (premultiplyAlpha === undefined && optionsVal.alpha != 1.0);
    let skipStr = " (Skipping checking right pixel since premultiplyAlpha was undefined and alpha != 1.0)";
    bufferedLogToConsole("Checking " + (flipY ? "top" : "bottom"));
    wtu.checkCanvasRect(gl, quarterWidth, bottom, 2, 2, tl, "shouldBe " + tl + " +/-" + tolerance, tolerance);
    if (skipAlphaTests) {
        bufferedLogToConsole(skipStr);
    } else {
        wtu.checkCanvasRect(gl, halfWidth + quarterWidth / 2, bottom, 2, 2, tr, "shouldBe " + tr + " +/-" + tolerance, tolerance);
    }
    bufferedLogToConsole("Checking " + (flipY ? "bottom" : "top"));
    wtu.checkCanvasRect(gl, quarterWidth, top, 2, 2, bl, "shouldBe " + bl + " +/-" + tolerance, tolerance);
    if (skipAlphaTests) {
        bufferedLogToConsole(skipStr);
    } else {
        wtu.checkCanvasRect(gl, halfWidth + quarterWidth / 2, top, 2, 2, br, "shouldBe " + br + " +/-" + tolerance, tolerance);
    }

    wtu.glErrorShouldBe(gl, gl.NO_ERROR, "should be no errors");
}

function runTestOnBindingTargetImageBitmap(bindingTarget, program, cases, optionsVal,
                                           internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance)
{
    cases.forEach(x => {
        runOneIterationImageBitmapTest(x.sub, bindingTarget, program, x.bitmap,
            x.bitmap.flipY, x.bitmap.premultiply, optionsVal, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance);
    });

    if (wtu.getDefault3DContextVersion() <= 1 ||
        (bindingTarget == gl.TEXTURE_CUBE_MAP || bindingTarget == gl.TEXTURE_2D_ARRAY))
    {
        // Skip testing source sub region on TEXTURE_CUBE_MAP and TEXTURE_2D_ARRAY on WebGL2 to save
        // running time.
        return;
    }

    cases.forEach(x => {
        runOneIterationImageBitmapTestSubSource(x.sub, bindingTarget, program, x.bitmap,
            x.bitmap.flipY, x.bitmap.premultiply, optionsVal, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance);
    });
}

function runImageBitmapTestInternal(bitmaps, alphaVal, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, is3D, tolerance)
{
    var cases = [];
    bitmaps.forEach(bitmap => {
        cases.push({bitmap: bitmap, sub: false});
        cases.push({bitmap: bitmap, sub: true});
    });

    var optionsVal = {alpha: alphaVal, is3D: is3D};
    var program;
    if (is3D) {
        program = tiu.setupTexturedQuadWith3D(gl, internalFormat);
        runTestOnBindingTargetImageBitmap(gl.TEXTURE_3D, program, cases, optionsVal,
            internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance);
    } else {
        program = tiu.setupTexturedQuad(gl, internalFormat);
        runTestOnBindingTargetImageBitmap(gl.TEXTURE_2D, program, cases, optionsVal,
            internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance);
    }

    // cube map texture must be square
    if (bitmaps[0].width == bitmaps[0].height) {
        if (is3D) {
            program = tiu.setupTexturedQuadWith2DArray(gl, internalFormat);
            runTestOnBindingTargetImageBitmap(gl.TEXTURE_2D_ARRAY, program, cases, optionsVal,
                internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance);
        } else {
            program = tiu.setupTexturedQuadWithCubeMap(gl, internalFormat);
            runTestOnBindingTargetImageBitmap(gl.TEXTURE_CUBE_MAP, program, cases, optionsVal,
                internalFormat, pixelFormat, pixelType, gl, tiu, wtu, tolerance);
        }
    }
}

function runImageBitmapTest(source, alphaVal, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, is3D, opt_tolerance)
{
    if (opt_tolerance === undefined) {
        opt_tolerance = 10;
    }
    var p1 = createImageBitmap(source, {imageOrientation: "none", premultiplyAlpha: "premultiply"})
                .then(cur => { cur.flipY = false; cur.premultiply = true; return cur; });
    var p2 = createImageBitmap(source, {imageOrientation: "none", premultiplyAlpha: "none"})
                .then(cur => { cur.flipY = false; cur.premultiply = false; return cur; });
    var p3 = createImageBitmap(source, {imageOrientation: "flipY", premultiplyAlpha: "premultiply"})
                .then(cur => { cur.flipY = true; cur.premultiply = true; return cur; });
    var p4 = createImageBitmap(source, {imageOrientation: "flipY", premultiplyAlpha: "none"})
                .then(cur => { cur.flipY = true; cur.premultiply = false; return cur; });
    return Promise.all([p1, p2, p3, p4])
        .catch( () => {
            testPassed("createImageBitmap with options may be rejected if it is not supported. Retrying without options.");
            // The ImageBitmap's premultiplyAlpha setting will implicitly be
            // "default", and per spec:
            // https://html.spec.whatwg.org/multipage/imagebitmap-and-animations.html#cropped-to-the-source-rectangle-with-formatting
            // this value is implementation-dependent (either premultiplied or
            // not). Skip testing the quadrants which have alpha != 1.0.
            var p = createImageBitmap(source)
                .then(cur => { cur.flipY = false; cur.premultiply = undefined; return cur; });
            return Promise.all([p]);
        }).then( bitmaps => {
            bufferedLogToConsole("All createImageBitmap promises are resolved");
            runImageBitmapTestInternal(bitmaps, alphaVal, internalFormat, pixelFormat, pixelType, gl, tiu, wtu, is3D, opt_tolerance);
        }, (e) => {
            // This will fail here when running from file:// instead of https://.
            testFailed("createImageBitmap(source) failed: \"" + e.message + "\"");
        });
}
