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
goog.provide('framework.opengl.gluTexture');
goog.require('framework.common.tcuCompressedTexture');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

var gluTexture = framework.opengl.gluTexture;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var tcuCompressedTexture = framework.common.tcuCompressedTexture;
var deMath = framework.delibs.debase.deMath;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

/** @enum {number} */
gluTexture.Type = {
    TYPE_NONE: 0,
    TYPE_2D: 1,
    TYPE_CUBE_MAP: 2,
    TYPE_2D_ARRAY: 3,
    TYPE_3D: 4
};

/**
 * @constructor
 */
gluTexture.Texture2D = function(gl, format, isCompressed, refTexture) {
    this.gl = gl;
    this.m_glTexture = gl.createTexture();
    this.m_isCompressed = isCompressed;
    this.m_format = format; // Internal format
    this.m_refTexture = refTexture;
    this.m_type = gluTexture.Type.TYPE_2D;
};

gluTexture.Texture2D.prototype.getType = function() {
    return this.m_type;
};

gluTexture.Texture2D.prototype.getRefTexture = function() {
    return this.m_refTexture;
};

gluTexture.Texture2D.prototype.getGLTexture = function() {
    return this.m_glTexture;
};

gluTexture.texture2DFromFormat = function(gl, format, dataType, width, height) {
    var tex = new gluTexture.Texture2D(gl, format, false, new tcuTexture.Texture2D(gluTextureUtil.mapGLTransferFormat(format, dataType), width, height));
    return tex;
};

gluTexture.texture2DFromInternalFormat = function(gl, internalFormat, width, height) {
    var tex = new gluTexture.Texture2D(gl, internalFormat, false, new tcuTexture.Texture2D(gluTextureUtil.mapGLInternalFormat(internalFormat), width, height));
    return tex;
};

/**
 * @param {number} numLevels
 * @param {Array<tcuCompressedTexture.CompressedTexture>} levels
 * @return {gluTexture.Texture2D}
 */
gluTexture.texture2DFromCompressedTexture = function(gl, numLevels, levels) {
    var level = levels[0];
    var format = gluTextureUtil.getGLFormat(level.getFormat());
    var refTex = new tcuTexture.Texture2D(level.getUncompressedFormat(), level.getWidth(), level.getHeight());
    /** @type {gluTexture.Texture2D} */ var tex2d = new gluTexture.Texture2D(gl, format, true, refTex);

    tex2d.loadCompressed(numLevels, levels);

    return tex2d;
};
/**
 * @param {number} numLevels
 * @param {Array<tcuCompressedTexture.CompressedTexture>} levels
 */
gluTexture.Texture2D.prototype.loadCompressed = function(numLevels, levels) {
    /** @type {number} */ var compressedFormat = gluTextureUtil.getGLFormat(levels[0].getFormat());

    assertMsgOptions(this.m_glTexture, 'm_glTexture not defined', false, true);
    gl.bindTexture(gl.TEXTURE_2D, this.m_glTexture);

    for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
        /** @type {tcuCompressedTexture.CompressedTexture} */ var level = levels[levelNdx];

        // Decompress to reference texture.
        this.m_refTexture.allocLevel(levelNdx);
        /** @type {tcuTexture.PixelBufferAccess} */ var refLevelAccess = this.m_refTexture.getLevel(levelNdx);
        assertMsgOptions(level.getWidth() == refLevelAccess.getWidth() && level.getHeight() == refLevelAccess.getHeight(), 'level and reference sizes not equal', false, true);
        level.decompress(refLevelAccess);

        // Upload to GL texture in compressed form.
        gl.compressedTexImage2D(gl.TEXTURE_2D, levelNdx, compressedFormat,
                                level.getWidth(), level.getHeight(), 0, level.getData());
    }
};

gluTexture.computePixelStore = function(/*const tcu::TextureFormat&*/ format) {
    var pixelSize = format.getPixelSize();
    if (deMath.deIsPowerOfTwo32(pixelSize))
        return Math.min(pixelSize, 8);
    else
        return 1;
};

gluTexture.cubeFaceToGLFace = function(/*tcu::CubeFace*/ face) {
    switch (face) {
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X: return gl.TEXTURE_CUBE_MAP_NEGATIVE_X;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X: return gl.TEXTURE_CUBE_MAP_POSITIVE_X;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y: return gl.TEXTURE_CUBE_MAP_NEGATIVE_Y;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y: return gl.TEXTURE_CUBE_MAP_POSITIVE_Y;
        case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z: return gl.TEXTURE_CUBE_MAP_NEGATIVE_Z;
        case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z: return gl.TEXTURE_CUBE_MAP_POSITIVE_Z;
    }
    throw new Error('Unrecognized face: ' + face);
};

gluTexture.Texture2D.prototype.upload = function() {
    DE_ASSERT(!this.m_isCompressed);

    if (this.m_glTexture == null)
        testFailedOptions('Failed to create GL texture', true);

    gl.bindTexture(gl.TEXTURE_2D, this.m_glTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, gluTexture.computePixelStore(this.m_refTexture.getFormat()));
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Setting pixel store failed', false, true);

    var transferFormat = gluTextureUtil.getTransferFormat(this.m_refTexture.getFormat());

    for (var levelNdx = 0; levelNdx < this.m_refTexture.getNumLevels(); levelNdx++) {
        if (this.m_refTexture.isLevelEmpty(levelNdx))
            continue; // Don't upload.

        var access = this.m_refTexture.getLevel(levelNdx);
        DE_ASSERT(access.getRowPitch() == access.getFormat().getPixelSize() * access.getWidth());
        var data = access.getDataPtr();
        gl.texImage2D(gl.TEXTURE_2D, levelNdx, this.m_format, access.getWidth(), access.getHeight(), 0 /* border */, transferFormat.format, transferFormat.dataType, access.getDataPtr());
    }

    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);
};

/**
 * @constructor
 * @extends {gluTexture.Texture2D}
 */
gluTexture.TextureCube = function(gl, format, isCompressed, refTexture) {
    gluTexture.Texture2D.call(this, gl, format, isCompressed, refTexture);
    this.m_type = gluTexture.Type.TYPE_CUBE_MAP;
};

gluTexture.TextureCube.prototype = Object.create(gluTexture.Texture2D.prototype);
gluTexture.TextureCube.prototype.constructor = gluTexture.TextureCube;

gluTexture.TextureCube.prototype.upload = function() {
    DE_ASSERT(!this.m_isCompressed);

    if (this.m_glTexture == null)
        testFailedOptions('Failed to create GL texture', true);

    gl.bindTexture(gl.TEXTURE_CUBE_MAP, this.m_glTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, gluTexture.computePixelStore(this.m_refTexture.getFormat()));
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Setting pixel store failed', false, true);

    var transferFormat = gluTextureUtil.getTransferFormat(this.m_refTexture.getFormat());

    for (var face in tcuTexture.CubeFace) {
        for (var levelNdx = 0; levelNdx < this.m_refTexture.getNumLevels(); levelNdx++) {
            if (this.m_refTexture.isLevelEmpty(tcuTexture.CubeFace[face], levelNdx))
                continue; // Don't upload.

            /*tcu::ConstPixelBufferAccess*/ var access = this.m_refTexture.getLevelFace(levelNdx, tcuTexture.CubeFace[face]);
            DE_ASSERT(access.getRowPitch() == access.getFormat().getPixelSize() * access.getWidth());
            gl.texImage2D(gluTexture.cubeFaceToGLFace(tcuTexture.CubeFace[face]), levelNdx, this.m_format, access.getWidth(), access.getHeight(), 0 /* border */, transferFormat.format, transferFormat.dataType, access.getDataPtr());
        }
    }

    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);
};

gluTexture.cubeFromFormat = function(gl, format, dataType, size) {
    var tex = new gluTexture.TextureCube(gl, format, false, new tcuTexture.TextureCube(gluTextureUtil.mapGLTransferFormat(format, dataType), size));
    return tex;
};

gluTexture.cubeFromInternalFormat = function(gl, internalFormat, size) {
    var tex = new gluTexture.TextureCube(gl, internalFormat, false, new tcuTexture.TextureCube(gluTextureUtil.mapGLInternalFormat(internalFormat), size));
    return tex;
};

/**
 * @constructor
 * @extends {gluTexture.Texture2D}
 */
gluTexture.Texture2DArray = function(gl, format, isCompressed, refTexture) {
    gluTexture.Texture2D.call(this, gl, format, isCompressed, refTexture);
    this.m_type = gluTexture.Type.TYPE_2D_ARRAY;
};

gluTexture.Texture2DArray.prototype = Object.create(gluTexture.Texture2D.prototype);
gluTexture.Texture2DArray.prototype.constructor = gluTexture.Texture2DArray;

gluTexture.Texture2DArray.prototype.upload = function() {
    if (!gl.texImage3D)
        throw new Error('gl.TexImage3D() is not supported');

    gl.bindTexture(gl.TEXTURE_2D_ARRAY, this.m_glTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, gluTexture.computePixelStore(this.m_refTexture.getFormat()));
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);

    var transferFormat = gluTextureUtil.getTransferFormat(this.m_refTexture.getFormat());

    for (var levelNdx = 0; levelNdx < this.m_refTexture.getNumLevels(); levelNdx++) {
        if (this.m_refTexture.isLevelEmpty(levelNdx))
            continue; // Don't upload.

        /*tcu::ConstPixelBufferAccess*/ var access = this.m_refTexture.getLevel(levelNdx);
        DE_ASSERT(access.getRowPitch() == access.getFormat().getPixelSize() * access.getWidth());
        DE_ASSERT(access.getSlicePitch() == access.getFormat().getPixelSize() * access.getWidth() * access.getHeight());
        gl.texImage3D(gl.TEXTURE_2D_ARRAY, levelNdx, this.m_format, access.getWidth(), access.getHeight(), access.getDepth(), 0 /* border */, transferFormat.format, transferFormat.dataType, access.getDataPtr());
    }

    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);
};

gluTexture.texture2DArrayFromFormat = function(gl, format, dataType, width, height, numLayers) {
    var tex = new gluTexture.Texture2DArray(gl, format, false, new tcuTexture.Texture2DArray(gluTextureUtil.mapGLTransferFormat(format, dataType), width, height, numLayers));
    return tex;
};

gluTexture.texture2DArrayFromInternalFormat = function(gl, internalFormat, width, height, numLayers) {
    var tex = new gluTexture.Texture2DArray(gl, internalFormat, false, new tcuTexture.Texture2DArray(gluTextureUtil.mapGLInternalFormat(internalFormat), width, height, numLayers));
    return tex;
};

/**
 * @constructor
 * @extends {gluTexture.Texture2D}
 */
gluTexture.Texture3D = function(gl, format, isCompressed, refTexture) {
    gluTexture.Texture2D.call(this, gl, format, isCompressed, refTexture);
    this.m_type = gluTexture.Type.TYPE_3D;
};

gluTexture.Texture3D.prototype = Object.create(gluTexture.Texture2D.prototype);
gluTexture.Texture3D.prototype.constructor = gluTexture.Texture3D;

gluTexture.Texture3D.prototype.upload = function() {
    if (!gl.texImage3D)
        throw new Error('gl.TexImage3D() is not supported');

    gl.bindTexture(gl.TEXTURE_3D, this.m_glTexture);
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, gluTexture.computePixelStore(this.m_refTexture.getFormat()));
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);

    var transferFormat = gluTextureUtil.getTransferFormat(this.m_refTexture.getFormat());

    for (var levelNdx = 0; levelNdx < this.m_refTexture.getNumLevels(); levelNdx++) {
        if (this.m_refTexture.isLevelEmpty(levelNdx))
            continue; // Don't upload.

        /*tcu::ConstPixelBufferAccess*/ var access = this.m_refTexture.getLevel(levelNdx);
        DE_ASSERT(access.getRowPitch() == access.getFormat().getPixelSize() * access.getWidth());
        DE_ASSERT(access.getSlicePitch() == access.getFormat().getPixelSize() * access.getWidth() * access.getHeight());
        gl.texImage3D(gl.TEXTURE_3D, levelNdx, this.m_format, access.getWidth(), access.getHeight(), access.getDepth(), 0 /* border */, transferFormat.format, transferFormat.dataType, access.getDataPtr());
    }

    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);
};

gluTexture.texture3DFromFormat = function(gl, format, dataType, width, height, depth) {
    var tex = new gluTexture.Texture3D(gl, format, false, new tcuTexture.Texture3D(gluTextureUtil.mapGLTransferFormat(format, dataType), width, height, depth));
    return tex;
};

gluTexture.texture3DFromInternalFormat = function(gl, internalFormat, width, height, depth) {
    var tex = new gluTexture.Texture3D(gl, internalFormat, false, new tcuTexture.Texture3D(gluTextureUtil.mapGLInternalFormat(internalFormat), width, height, depth));
    return tex;
};

/**
 * @constructor
 * @extends {gluTexture.Texture2D}
 */
gluTexture.Compressed2D = function(gl, format, isCompressed, refTexture) {
    gluTexture.Texture2D.call(this, gl, format, isCompressed, refTexture);
};

gluTexture.Compressed2D.prototype = Object.create(gluTexture.Texture2D.prototype);
gluTexture.Compressed2D.prototype.constructor = gluTexture.Compressed2D;

gluTexture.Compressed2D.prototype.uploadLevel = function(level, source) {
    DE_ASSERT(this.m_isCompressed);

    if (this.m_glTexture == null)
        testFailedOptions('Failed to create GL texture', true);

    gl.bindTexture(gl.TEXTURE_2D, this.m_glTexture);

    gl.compressedTexImage2D(gl.TEXTURE_2D, level, this.m_format, source.m_width, source.m_height, 0 /* border */, source.m_data);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);
};

/**
 * @constructor
 * @extends {gluTexture.Texture2D}
 */
gluTexture.CompressedCube = function(gl, format, isCompressed, refTexture) {
    gluTexture.Texture2D.call(this, gl, format, isCompressed, refTexture);
};

gluTexture.CompressedCube.prototype = Object.create(gluTexture.Texture2D.prototype);
gluTexture.CompressedCube.prototype.constructor = gluTexture.CompressedCube;

gluTexture.CompressedCube.prototype.uploadLevel = function(level, source) {
    DE_ASSERT(this.m_isCompressed);

    if (this.m_glTexture == null)
        testFailedOptions('Failed to create GL texture', true);

    gl.bindTexture(gl.TEXTURE_CUBE_MAP, this.m_glTexture);

    for (var face in tcuTexture.CubeFace) {

        // Upload to GL texture in compressed form.
        gl.compressedTexImage2D(gluTexture.cubeFaceToGLFace(tcuTexture.CubeFace[face]), 0, this.m_format,
                                source.m_width, source.m_height, 0 /* border */, source.m_data);
        assertMsgOptions(gl.getError() === gl.NO_ERROR, 'Texture upload failed', false, true);
    }

};

gluTexture.compressed2DFromInternalFormat = function(gl, format, width, height, compressed) {
    var tex = new gluTexture.Compressed2D(gl, gluTextureUtil.getGLFormat(format), true, new tcuTexture.Texture2D(compressed.getUncompressedFormat(), width, height));
    tex.m_refTexture.allocLevel(0);
    compressed.decompress(tex.m_refTexture.getLevel(0));
    tex.uploadLevel(0, compressed);
    return tex;
};

gluTexture.compressedCubeFromInternalFormat = function(gl, format, size, compressed) {
    var tex = new gluTexture.CompressedCube(gl, gluTextureUtil.getGLFormat(format), true, new tcuTexture.TextureCube(compressed.getUncompressedFormat(), size));
    for (var face in tcuTexture.CubeFace) {
        tex.m_refTexture.allocLevel(tcuTexture.CubeFace[face], 0);

        /*tcu::ConstPixelBufferAccess*/ var access = tex.m_refTexture.getLevelFace(0, tcuTexture.CubeFace[face]);
        DE_ASSERT(access.getRowPitch() == access.getFormat().getPixelSize() * access.getWidth());
        compressed.decompress(access);
    }
    tex.uploadLevel(0, compressed);
    return tex;
};

});
