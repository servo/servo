/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

"use strict";

let CompressedTextureUtils = (function() {

let formatToString = function(ext, format) {
    for (let p in ext) {
        if (ext[p] == format) {
            return p;
        }
    }
    return "0x" + format.toString(16);
};

/**
 * Make an image element from Uint8Array bitmap data.
 * @param {number} imageHeight Height of the data in pixels.
 * @param {number} imageWidth Width of the data in pixels.
 * @param {number} dataWidth Width of each row in the data buffer, in pixels.
 * @param {Uint8Array} data Image data buffer to display. Each pixel takes up 4 bytes in the array regardless of the alpha parameter.
 * @param {boolean} alpha True if alpha data should be taken from data. Otherwise alpha channel is set to 255.
 * @return {HTMLImageElement} The image element.
 */
let makeScaledImage = function(imageWidth, imageHeight, dataWidth, data, alpha, opt_scale) {
    let scale = opt_scale ? opt_scale : 8;
    let c = document.createElement("canvas");
    c.width = imageWidth * scale;
    c.height = imageHeight * scale;
    let ctx = c.getContext("2d");
    for (let yy = 0; yy < imageHeight; ++yy) {
        for (let xx = 0; xx < imageWidth; ++xx) {
            let offset = (yy * dataWidth + xx) * 4;
            ctx.fillStyle = "rgba(" +
                    data[offset + 0] + "," +
                    data[offset + 1] + "," +
                    data[offset + 2] + "," +
                    (alpha ? data[offset + 3] / 255 : 1) + ")";
            ctx.fillRect(xx * scale, yy * scale, scale, scale);
        }
    }
    return wtu.makeImageFromCanvas(c);
};

let insertCaptionedImg = function(parent, caption, img) {
    let div = document.createElement("div");
    div.appendChild(img);
    let label = document.createElement("div");
    label.appendChild(document.createTextNode(caption));
    div.appendChild(label);
    parent.appendChild(div);
};

/**
 * @param {WebGLRenderingContextBase} gl
 * @param {Object} compressedFormats Mapping from format names to format enum values.
 * @param expectedByteLength A function that takes in width, height and format and returns the expected buffer size in bytes.
 */
let testCompressedFormatsUnavailableWhenExtensionDisabled = function(gl, compressedFormats, expectedByteLength, testSize) {
    let tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    for (let name in compressedFormats) {
        if (compressedFormats.hasOwnProperty(name)) {
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, compressedFormats[name], testSize, testSize, 0, new Uint8Array(expectedByteLength(testSize, testSize, compressedFormats[name])));
            wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "Trying to use format " + name + " with extension disabled.");
            if (gl.texStorage2D) {
                gl.texStorage2D(gl.TEXTURE_2D, 1, compressedFormats[name], testSize, testSize);
                wtu.glErrorShouldBe(gl, gl.INVALID_ENUM, "Trying to use format " + name + " with texStorage2D with extension disabled.");
            }
        }
    }
    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.deleteTexture(tex);
};

/**
 * @param {WebGLRenderingContextBase} gl
 * @param {Object} expectedFormats Mapping from format names to format enum values.
 */
let testCompressedFormatsListed = function(gl, expectedFormats) {
    debug("");
    debug("Testing that every format is listed by the compressed texture formats query");

    let supportedFormats = gl.getParameter(gl.COMPRESSED_TEXTURE_FORMATS);

    let failed;
    let count = 0;
    for (let name in expectedFormats) {
        if (expectedFormats.hasOwnProperty(name)) {
            ++count;
            let format = expectedFormats[name];
            failed = true;
            for (let ii = 0; ii < supportedFormats.length; ++ii) {
                if (format == supportedFormats[ii]) {
                    testPassed("supported format " + name + " exists");
                    failed = false;
                    break;
                }
            }
            if (failed) {
                testFailed("supported format " + name + " does not exist");
            }
        }
    }
    if (supportedFormats.length != count) {
        testFailed("Incorrect number of supported formats, was " + supportedFormats.length + " should be " + count);
    }
};

/**
 * @param {Object} ext Compressed texture extension object.
 * @param {Object} expectedFormats Mapping from format names to format enum values.
 */
let testCorrectEnumValuesInExt = function(ext, expectedFormats) {
    debug("");
    debug("Testing that format enum values in the extension object are correct");

    for (name in expectedFormats) {
        if (expectedFormats.hasOwnProperty(name)) {
            if (isResultCorrect(ext[name], expectedFormats[name])) {
                testPassed("Enum value for " + name + " matches 0x" + ext[name].toString(16));
            } else {
                testFailed("Enum value for " + name + " mismatch: 0x" + ext[name].toString(16) + " should be 0x" + expectedFormats[name].toString(16));
            }
        }
    }
};

/**
 * @param {WebGLRenderingContextBase} gl
 * @param {Object} validFormats Mapping from format names to format enum values.
 * @param expectedByteLength A function that takes in width, height and format and returns the expected buffer size in bytes.
 * @param getBlockDimensions A function that takes in a format and returns block size in pixels.
 */
let testFormatRestrictionsOnBufferSize = function(gl, validFormats, expectedByteLength, getBlockDimensions) {
    debug("");
    debug("Testing format restrictions on texture upload buffer size");

    let tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    for (let formatId in validFormats) {
        if (validFormats.hasOwnProperty(formatId)) {
            let format = validFormats[formatId];
            let blockSize = getBlockDimensions(format);
            let expectedSize = expectedByteLength(blockSize.width * 4, blockSize.height * 4, format);
            let data = new Uint8Array(expectedSize);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, format, blockSize.width * 3, blockSize.height * 4, 0, data);
            wtu.glErrorShouldBe(gl, gl.INVALID_VALUE, formatId + " data size does not match dimensions (too small width)");
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, format, blockSize.width * 5, blockSize.height * 4, 0, data);
            wtu.glErrorShouldBe(gl, gl.INVALID_VALUE, formatId + " data size does not match dimensions (too large width)");
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, format, blockSize.width * 4, blockSize.height * 3, 0, data);
            wtu.glErrorShouldBe(gl, gl.INVALID_VALUE, formatId + " data size does not match dimensions (too small height)");
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, format, blockSize.width * 4, blockSize.height * 5, 0, data);
            wtu.glErrorShouldBe(gl, gl.INVALID_VALUE, formatId + " data size does not match dimensions (too large height)");
        }
    }
};

/**
 * @param {WebGLRenderingContextBase} gl
 * @param {Object} validFormats Mapping from format names to format enum values.
 * @param expectedByteLength A function that takes in width, height and format and returns the expected buffer size in bytes.
 * @param getBlockDimensions A function that takes in a format and returns block size in pixels.
 * @param {number} width Width of the image in pixels.
 * @param {number} height Height of the image in pixels.
 * @param {Object} subImageConfigs configs for compressedTexSubImage calls
 */
let testTexSubImageDimensions = function(gl, ext, validFormats, expectedByteLength, getBlockDimensions, width, height, subImageConfigs) {
    let tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);

    for (let formatId in validFormats) {
        if (validFormats.hasOwnProperty(formatId)) {
            let format = validFormats[formatId];
            let blockSize = getBlockDimensions(format);
            debug("testing " + ctu.formatToString(ext, format));
            let expectedSize = expectedByteLength(width, height, format);
            let data = new Uint8Array(expectedSize);

            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, format, width, height, 0, data);
            wtu.glErrorShouldBe(gl, gl.NO_ERROR, "setting up compressed texture");

            for (let i = 0, len = subImageConfigs.length; i < len; ++i) {
                let c = subImageConfigs[i];
                let subData = new Uint8Array(expectedByteLength(c.width, c.height, format));
                gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, c.xoffset, c.yoffset, c.width, c.height, format, subData);
                wtu.glErrorShouldBe(gl, c.expectation, c.message);
            }
        }
    }

    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.deleteTexture(tex);
};

let testTexImageLevelDimensions = function(gl, ext, validFormats, expectedByteLength, getBlockDimensions, imageConfigs) {
    let tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);

    for (let formatId in validFormats) {
        if (validFormats.hasOwnProperty(formatId)) {
            let format = validFormats[formatId];
            let blockSize = getBlockDimensions(format);
            debug("testing " + ctu.formatToString(ext, format));

            for (let i = 0, len = imageConfigs.length; i < len; ++i) {
                let c = imageConfigs[i];
                let data = new Uint8Array(expectedByteLength(c.width, c.height, format));
                gl.compressedTexImage2D(gl.TEXTURE_2D, c.level, format, c.width, c.height, 0, data);
                wtu.glErrorShouldBe(gl, c.expectation, c.message);
            }
        }
    }

    gl.bindTexture(gl.TEXTURE_2D, null);
    gl.deleteTexture(tex);
}

let testTexStorageLevelDimensions = function(gl, ext, validFormats, expectedByteLength, getBlockDimensions, imageConfigs) {
    for (let formatId in validFormats) {
        let tex = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, tex);

        if (validFormats.hasOwnProperty(formatId)) {
            let format = validFormats[formatId];
            let blockSize = getBlockDimensions(format);
            debug("testing " + ctu.formatToString(ext, format));

            for (let i = 0, len = imageConfigs.length; i < len; ++i) {
                let c = imageConfigs[i];
                let data = new Uint8Array(expectedByteLength(c.width, c.height, format));
                if (i == 0) {
                    gl.texStorage2D(gl.TEXTURE_2D, imageConfigs.length, format, c.width, c.height);
                    wtu.glErrorShouldBe(gl, c.expectation, c.message);
                }
                gl.compressedTexSubImage2D(gl.TEXTURE_2D, i, 0, 0, c.width, c.height, format, data);
                wtu.glErrorShouldBe(gl, c.expectation, c.message);
            }
        }
        gl.bindTexture(gl.TEXTURE_2D, null);
        gl.deleteTexture(tex);
    }
}

return {
    formatToString: formatToString,
    insertCaptionedImg: insertCaptionedImg,
    makeScaledImage: makeScaledImage,
    testCompressedFormatsListed: testCompressedFormatsListed,
    testCompressedFormatsUnavailableWhenExtensionDisabled: testCompressedFormatsUnavailableWhenExtensionDisabled,
    testCorrectEnumValuesInExt: testCorrectEnumValuesInExt,
    testFormatRestrictionsOnBufferSize: testFormatRestrictionsOnBufferSize,
    testTexSubImageDimensions: testTexSubImageDimensions,
    testTexImageLevelDimensions: testTexImageLevelDimensions,
    testTexStorageLevelDimensions: testTexStorageLevelDimensions,
};

})();
