/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('framework.common.tcuLogImage');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var tcuLogImage = framework.common.tcuLogImage;
var tcuTexture = framework.common.tcuTexture;
var tcuSurface = framework.common.tcuSurface;
var deMath = framework.delibs.debase.deMath;

/** @const */ var MAX_IMAGE_SIZE_2D = 4096;
/**
 * @param {tcuTexture.ConstPixelBufferAccess} src
 */
tcuLogImage.createImage = function(ctx, src) {
    var w = src.getWidth();
    var h = src.getHeight();
    var pixelSize = src.getFormat().getPixelSize();
    var imgData = ctx.createImageData(w, h);
    var index = 0;
    for (var y = 0; y < h; y++) {
        for (var x = 0; x < w; x++) {
            var pixel = src.getPixelInt(x, h - y - 1, 0);
            for (var i = 0; i < pixelSize; i++) {
                imgData.data[index] = pixel[i];
                index = index + 1;
            }
            if (pixelSize < 4)
                imgData.data[index++] = 255;
        }
    }
    return imgData;
};

/**
 * @param {tcuTexture.ConstPixelBufferAccess} image
 * @param {string} info
 */
tcuLogImage.logImageWithInfo = function(image, info) {
    var elem = document.getElementById('console');
    var span = document.createElement('span');
    tcuLogImage.logImage.counter = tcuLogImage.logImage.counter || 0;
    var i = tcuLogImage.logImage.counter++;
    var width = image.getWidth();
    var height = image.getHeight();

    elem.appendChild(span);
    span.innerHTML = info + '<br> <canvas id="logImage' + i + '" width=' + width + ' height=' + height + '></canvas><br>';

    var imageCanvas = document.getElementById('logImage' + i);
    var ctx = imageCanvas.getContext('2d');
    var data = tcuLogImage.createImage(ctx, image);
    ctx.putImageData(data, 0, 0);
};


/**
 * @param {Array<number>=} scale
 * @param {Array<number>=} bias
 * @return {string} HTML string to add to log.
 */
tcuLogImage.logScaleAndBias = function(scale, bias) {
    if (scale && bias)
        return '<br> Image normalized with formula p * (' + scale + ') + (' + bias + ')';
    else if (scale)
        return '<br> Image normalized with formula p * (' + scale + ')';
    else if (bias)
        return '<br> Image normalized with formula p + (' + bias + ')';
    return '';
};

/**
 * @param {string} name
 * @param {string} description
 * @param {tcuTexture.ConstPixelBufferAccess} image
 * @param {Array<number>=} scale
 * @param {Array<number>=} bias
 */
tcuLogImage.logImageRGB = function(name, description, image, scale, bias) {
    var elem = document.getElementById('console');
    var span = document.createElement('span');
    var info = name + ' ' + description + '<br> ' + image;
    if (scale || bias)
        info += tcuLogImage.logScaleAndBias(scale, bias);
    tcuLogImage.logImageWithInfo(image, info);
};

/**
 * @param {string} name
 * @param {string} description
 * @param {tcuTexture.ConstPixelBufferAccess} access
 * @param {Array<number>=} pixelScale
 * @param {Array<number>=} pixelBias
 */
tcuLogImage.logImage = function(name, description, access, pixelScale, pixelBias) {
    pixelScale = pixelScale || [1, 1, 1, 1];
    pixelBias = pixelBias || [0, 0, 0, 0];
    var format = access.getFormat();
    var width = access.getWidth();
    var height = access.getHeight();
    var depth = access.getDepth();
    var needScaling = pixelBias[0] != 0 || pixelBias[1] != 0 || pixelBias[2] != 0 || pixelBias[3] != 0 ||
        pixelScale[0] != 1 || pixelScale[1] != 1 || pixelScale[2] != 1 || pixelScale[3] != 1;

    if (depth == 1 && format.type == tcuTexture.ChannelType.UNORM_INT8 &&
        width <= MAX_IMAGE_SIZE_2D && height <= MAX_IMAGE_SIZE_2D &&
        (format.order == tcuTexture.ChannelOrder.RGB || tcuTexture.ChannelOrder.RGBA) &&
        !needScaling)
        // Fast-path.
        tcuLogImage.logImageRGB(name, description, access);
    else if (depth == 1) {
        var sampler = new tcuTexture.Sampler(tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE, tcuTexture.WrapMode.CLAMP_TO_EDGE,
            tcuTexture.FilterMode.LINEAR, tcuTexture.FilterMode.NEAREST);
        var logImageSize = [width, height]; /* TODO: Add scaling */
        var logImageAccess = new tcuSurface.Surface(width, height).getAccess();

        for (var y = 0; y < logImageAccess.getHeight(); y++) {
            for (var x = 0; x < logImageAccess.getWidth(); x++) {
                var yf = (y + 0.5) / logImageAccess.getHeight();
                var xf = (x + 0.5) / logImageAccess.getWidth();
                var s = access.sample2D(sampler, sampler.minFilter, xf, yf, 0);

                if (needScaling)
                    s = deMath.add(deMath.multiply(s, pixelScale), pixelBias);

                logImageAccess.setPixel(s, x, y);
            }
        }
        var info = name + ' ' + description + '<br> ' + access;
        if (needScaling) {
            info += tcuLogImage.logScaleAndBias(pixelScale, pixelBias);
        }

        tcuLogImage.logImageWithInfo(logImageAccess, info);
    } else {
        /* TODO: Implement */
    }
};

});
