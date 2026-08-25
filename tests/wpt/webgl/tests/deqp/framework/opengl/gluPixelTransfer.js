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
goog.provide('framework.opengl.gluPixelTransfer');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

var gluPixelTransfer = framework.opengl.gluPixelTransfer;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;
var gluTextureUtil = framework.opengl.gluTextureUtil;

gluPixelTransfer.getTransferAlignment = function(format) {
    var pixelSize = format.getPixelSize();
    if (deMath.deIsPowerOfTwo32(pixelSize))
        return Math.min(pixelSize, 8);
    else
        return 1;
};

gluPixelTransfer.readPixels = function(ctx, x, y, format, dst) {
    var width = dst.getWidth();
    var height = dst.getHeight();
    var arrayType = tcuTexture.getTypedArray(format.type);
    var transferFormat = gluTextureUtil.getTransferFormat(format);
    ctx.pixelStorei(ctx.PACK_ALIGNMENT, gluPixelTransfer.getTransferAlignment(format));
    var resultPixel = dst.getAccess().getDataPtr();
    resultPixel = new arrayType(dst.getAccess().getBuffer());
    ctx.readPixels(x, y, width, height, transferFormat.format, transferFormat.dataType, resultPixel);
};

/* TODO: implement other functions in C++ file */

});
