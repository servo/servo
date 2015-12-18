/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the 'License');
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an 'AS IS' BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('functional.gles3.es3fFboStateQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fFboStateQueryTests = functional.gles3.es3fFboStateQueryTests;
var tcuTestCase = framework.common.tcuTestCase;
var glsStateQuery = modules.shared.glsStateQuery;
var es3fApiCase = functional.gles3.es3fApiCase;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

// WebGL bit depths
es3fFboStateQueryTests.colorBits = [8, 8, 8, 8];
es3fFboStateQueryTests.depthBits = 0;
es3fFboStateQueryTests.stencilBits = 0;

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {number} framebufferTarget
 */
es3fFboStateQueryTests.DefaultFramebufferCase = function(name, description, framebufferTarget) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_framebufferTarget = framebufferTarget;
};

setParentClass(es3fFboStateQueryTests.DefaultFramebufferCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.DefaultFramebufferCase.prototype.test = function() {
    var hasColorBuffer = es3fFboStateQueryTests.colorBits[0] > 0 ||
                            es3fFboStateQueryTests.colorBits[1] > 0 ||
                            es3fFboStateQueryTests.colorBits[2] > 0 ||
                            es3fFboStateQueryTests.colorBits[3] > 0;
    var attachments = [
        gl.BACK,
        gl.DEPTH,
        gl.STENCIL
    ];
    var attachmentExists = [
        hasColorBuffer,
        es3fFboStateQueryTests.depthBits > 0,
        es3fFboStateQueryTests.stencilBits > 0
    ];

    for (var ndx = 0; ndx < attachments.length; ++ndx) {
        var objType = gl.getFramebufferAttachmentParameter(this.m_framebufferTarget, attachments[ndx], gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
        if (attachmentExists[ndx]) {
            this.check(objType === gl.FRAMEBUFFER_DEFAULT);
        } else {
            // \note [jarkko] FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE "identifes the type of object which contains the attached image". However, it
            // is unclear if an object of type FRAMEBUFFER_DEFAULT can contain a null image (or a 0-bits-per-pixel image). Accept both
            // FRAMEBUFFER_DEFAULT and NONE as valid results in these cases.
            this.check(objType === gl.FRAMEBUFFER_DEFAULT || objType === gl.NONE);
        }
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentObjectCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentObjectCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentObjectCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);

    // initial
    this.check(glsStateQuery.verifyAttachment(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE, gl.NONE));
    this.check(glsStateQuery.verifyAttachment(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME, null));

    // texture
    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, textureID);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 128, 128, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, textureID, 0);

    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE, gl.TEXTURE));
    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME, textureID));

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);

    // rb
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB8, 128, 128);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE, gl.RENDERBUFFER));
    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME, renderbufferID));

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);

    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentTextureLevelCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentTextureLevelCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentTextureLevelCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    for (var mipmapLevel = 0; mipmapLevel < 7; ++mipmapLevel) {
        var textureID = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, textureID);
        gl.texStorage2D(gl.TEXTURE_2D, 7, gl.RGB8, 128, 128);

        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, textureID, mipmapLevel);

        this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL, mipmapLevel));

        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
        gl.deleteTexture(textureID);
    }
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentTextureCubeMapFaceCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentTextureCubeMapFaceCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentTextureCubeMapFaceCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_CUBE_MAP, textureID);

    gl.texStorage2D(gl.TEXTURE_CUBE_MAP, 1, gl.RGB8, 128, 128);

    var faces = [
        gl.TEXTURE_CUBE_MAP_POSITIVE_X, gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
        gl.TEXTURE_CUBE_MAP_POSITIVE_Y, gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
        gl.TEXTURE_CUBE_MAP_POSITIVE_Z, gl.TEXTURE_CUBE_MAP_NEGATIVE_Z
    ];

    for (var ndx = 0; ndx < faces.length; ++ndx) {
        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, faces[ndx], textureID, 0);
        this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE, faces[ndx]));
    }

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentTextureLayerCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentTextureLayerCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentTextureLayerCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    // tex3d
    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_3D, textureID);
    gl.texStorage3D(gl.TEXTURE_3D, 1, gl.RGBA8, 16, 16, 16);

    for (var layer = 0; layer < 16; ++layer) {
        gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, textureID, 0, layer);
        this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER, layer));
    }

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
    // tex2d array
    textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D_ARRAY, textureID);
    gl.texStorage3D(gl.TEXTURE_2D_ARRAY, 1, gl.RGBA8, 16, 16, 16);

    for (var layer = 0; layer < 16; ++layer) {
        gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, textureID, 0, layer);
        this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER, layer));
    }

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentTextureColorCodingCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentTextureColorCodingCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentTextureColorCodingCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    // rgb8 color
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB8, 128, 128);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING, gl.LINEAR));

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);

    // srgb8_alpha8 color
    renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.SRGB8_ALPHA8, 128, 128);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING, gl.SRGB));

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);

    // depth
    renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, 128, 128);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyAttachment(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING, gl.LINEAR));

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentTextureComponentTypeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentTextureComponentTypeCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentTextureComponentTypeCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    // color-renderable required texture formats
    var requiredColorformats = [
        [gl.R8, gl.UNSIGNED_NORMALIZED],
        [gl.RG8, gl.UNSIGNED_NORMALIZED],
        [gl.RGB8, gl.UNSIGNED_NORMALIZED],
        [gl.RGB565, gl.UNSIGNED_NORMALIZED],
        [gl.RGBA4, gl.UNSIGNED_NORMALIZED],
        [gl.RGB5_A1, gl.UNSIGNED_NORMALIZED],
        [gl.RGBA8, gl.UNSIGNED_NORMALIZED],
        [gl.RGB10_A2, gl.UNSIGNED_NORMALIZED],
        [gl.RGB10_A2UI, gl.UNSIGNED_INT],
        [gl.SRGB8_ALPHA8, gl.UNSIGNED_NORMALIZED],
        [gl.R8I, gl.INT],
        [gl.R8UI, gl.UNSIGNED_INT],
        [gl.R16I, gl.INT],
        [gl.R16UI, gl.UNSIGNED_INT],
        [gl.R32I, gl.INT],
        [gl.R32UI, gl.UNSIGNED_INT],
        [gl.RG8I, gl.INT],
        [gl.RG8UI, gl.UNSIGNED_INT],
        [gl.RG16I, gl.INT],
        [gl.RG16UI, gl.UNSIGNED_INT],
        [gl.RG32I, gl.INT],
        [gl.RG32UI, gl.UNSIGNED_INT],
        [gl.RGBA8I, gl.INT],
        [gl.RGBA8UI, gl.UNSIGNED_INT],
        [gl.RGBA16I, gl.INT],
        [gl.RGBA16UI, gl.UNSIGNED_INT],
        [gl.RGBA32I, gl.INT],
        [gl.RGBA32UI, gl.UNSIGNED_INT]
    ];

    for (var ndx = 0; ndx < requiredColorformats.length; ++ndx) {
        var colorFormat = requiredColorformats[ndx][0];
        var componentType = requiredColorformats[ndx][1];

        var textureID = gl.createTexture();
        gl.bindTexture(gl.TEXTURE_2D, textureID);
        gl.texStorage2D(gl.TEXTURE_2D, 1, colorFormat, 128, 128);

        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, textureID, 0);

        this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE, componentType));

        gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
        gl.deleteTexture(textureID);
    }
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentSizeInitialCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentSizeInitialCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentSizeInitialCase.prototype.attachmentExists = function(attachment) {
    var objType = gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, attachment, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
    return objType !== gl.NONE;
};

/**
 * @this {es3fApiCase.ApiCase}
 */
var checkAttachmentComponentSizeAtLeast = function(target, attachment, r, g, b, a, d, s) {
    var referenceSizes = [r, g, b, a, d, s];
    var paramNames = [
        gl.FRAMEBUFFER_ATTACHMENT_RED_SIZE, gl.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE, gl.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE, gl.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE
    ];

    for (var ndx = 0; ndx < referenceSizes.length; ++ndx) {
        if (referenceSizes[ndx] == -1)
            continue;

        var value = /** @type {number} */ (gl.getFramebufferAttachmentParameter(target, attachment, paramNames[ndx]));

        this.check(value >= referenceSizes[ndx], 'Expected greater or equal to ' + referenceSizes[ndx] + ' got ' + value);
    }
};

/**
 * @this {es3fApiCase.ApiCase}
 */
var checkAttachmentComponentSizeExactly = function(target, attachment, r, g, b, a, d, s) {
    var referenceSizes = [r, g, b, a, d, s];
    var paramNames = [
        gl.FRAMEBUFFER_ATTACHMENT_RED_SIZE, gl.FRAMEBUFFER_ATTACHMENT_GREEN_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_BLUE_SIZE, gl.FRAMEBUFFER_ATTACHMENT_ALPHA_SIZE,
        gl.FRAMEBUFFER_ATTACHMENT_DEPTH_SIZE, gl.FRAMEBUFFER_ATTACHMENT_STENCIL_SIZE
    ];

    for (var ndx = 0; ndx < referenceSizes.length; ++ndx) {
        if (referenceSizes[ndx] == -1)
            continue;

        var value = gl.getFramebufferAttachmentParameter(target, attachment, paramNames[ndx]);

        this.check(value == referenceSizes[ndx], 'Expected equal to ' + referenceSizes[ndx] + ' got ' + value);
    }
};

es3fFboStateQueryTests.AttachmentSizeInitialCase.prototype.test = function() {
    // check default
    if (this.attachmentExists(gl.BACK)) {
        checkAttachmentComponentSizeAtLeast.bind(this,
            gl.FRAMEBUFFER,
            gl.BACK,
            es3fFboStateQueryTests.colorBits[0],
            es3fFboStateQueryTests.colorBits[1],
            es3fFboStateQueryTests.colorBits[2],
            es3fFboStateQueryTests.colorBits[3],
            -1,
            -1);
    }

    if (this.attachmentExists(gl.DEPTH)) {
        checkAttachmentComponentSizeAtLeast.bind(this,
            gl.FRAMEBUFFER,
            gl.DEPTH,
            -1,
            -1,
            -1,
            -1,
            es3fFboStateQueryTests.depthBits,
            -1);
    }

    if (this.attachmentExists(gl.STENCIL)) {
        checkAttachmentComponentSizeAtLeast.bind(this,
            gl.FRAMEBUFFER,
            gl.STENCIL,
            -1,
            -1,
            -1,
            -1,
            -1,
            es3fFboStateQueryTests.stencilBits);
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentSizeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.AttachmentSizeCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.AttachmentSizeCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    // check some color targets

    var colorAttachments = [
        //format, red, green, blue, alpha
        [gl.RGBA8, 8, 8, 8, 8],
        [gl.RGB565, 5, 6, 5, 0],
        [gl.RGBA4, 4, 4, 4, 4],
        [gl.RGB5_A1, 5, 5, 5, 1],
        [gl.RGBA8I, 8, 8, 8, 8],
        [gl.RG32UI, 32, 32, 0, 0]
    ];
    for (var ndx = 0; ndx < colorAttachments.length; ++ndx)
        this.testColorAttachment(colorAttachments[ndx][0], gl.COLOR_ATTACHMENT0, colorAttachments[ndx][1], colorAttachments[ndx][2], colorAttachments[ndx][3], colorAttachments[ndx][4]);

    // check some depth targets

    var depthAttachments = [
        // format, attachment, depth, stencil
        [gl.DEPTH_COMPONENT16, gl.DEPTH_ATTACHMENT, 16, 0],
        [gl.DEPTH_COMPONENT24, gl.DEPTH_ATTACHMENT, 24, 0],
        [gl.DEPTH_COMPONENT32F, gl.DEPTH_ATTACHMENT, 32, 0],
        [gl.DEPTH24_STENCIL8, gl.DEPTH_STENCIL_ATTACHMENT, 24, 8],
        [gl.DEPTH32F_STENCIL8, gl.DEPTH_STENCIL_ATTACHMENT, 32, 8]
    ];
    for (var ndx = 0; ndx < depthAttachments.length; ++ndx)
        this.testDepthAttachment(depthAttachments[ndx][0], depthAttachments[ndx][1], depthAttachments[ndx][2], depthAttachments[ndx][3]);
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fFboStateQueryTests.AttachmentSizeCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentSizeRboCase = function(name, description) {
    es3fFboStateQueryTests.AttachmentSizeCase.call(this, name, description);
};

setParentClass(es3fFboStateQueryTests.AttachmentSizeRboCase, es3fFboStateQueryTests.AttachmentSizeCase);

es3fFboStateQueryTests.AttachmentSizeRboCase.prototype.testColorAttachment = function(internalFormat, attachment, r, g, b, a) {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);
    gl.renderbufferStorage(gl.RENDERBUFFER, internalFormat, 128, 128);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, attachment, gl.RENDERBUFFER, renderbufferID);

    checkAttachmentComponentSizeAtLeast.bind(this, gl.FRAMEBUFFER, attachment, r, g, b, a, -1, -1);
    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, attachment, -1, -1, -1, -1, 0, 0);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, attachment, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);
};

es3fFboStateQueryTests.AttachmentSizeRboCase.prototype.testDepthAttachment = function(internalFormat, attachment, depth, stencil) {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);
    gl.renderbufferStorage(gl.RENDERBUFFER, internalFormat, 128, 128);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, attachment, gl.RENDERBUFFER, renderbufferID);

    checkAttachmentComponentSizeAtLeast.bind(this, gl.FRAMEBUFFER, attachment, -1, -1, -1, -1, depth, stencil);
    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, attachment, 0, 0, 0, 0, -1, -1);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, attachment, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);
};

/**
 * @constructor
 * @extends {es3fFboStateQueryTests.AttachmentSizeCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.AttachmentSizeTextureCase = function(name, description) {
    es3fFboStateQueryTests.AttachmentSizeCase.call(this, name, description);
};

setParentClass(es3fFboStateQueryTests.AttachmentSizeTextureCase, es3fFboStateQueryTests.AttachmentSizeCase);

es3fFboStateQueryTests.AttachmentSizeTextureCase.prototype.testColorAttachment = function(internalFormat, attachment, r, g, b, a) {
    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, textureID);
    gl.texStorage2D(gl.TEXTURE_2D, 1, internalFormat, 128, 128);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachment, gl.TEXTURE_2D, textureID, 0);

    checkAttachmentComponentSizeAtLeast.bind(this, gl.FRAMEBUFFER, attachment, r, g, b, a, -1, -1);
    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, attachment, -1, -1, -1, -1, 0, 0);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachment, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
};

es3fFboStateQueryTests.AttachmentSizeTextureCase.prototype.testDepthAttachment = function(internalFormat, attachment, depth, stencil) {
    // don't test stencil formats with textures
    if (attachment == gl.DEPTH_STENCIL_ATTACHMENT)
        return;

    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, textureID);
    gl.texStorage2D(gl.TEXTURE_2D, 1, internalFormat, 128, 128);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachment, gl.TEXTURE_2D, textureID, 0);

    checkAttachmentComponentSizeAtLeast.bind(this, gl.FRAMEBUFFER, attachment, -1, -1, -1, -1, depth, stencil);
    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, attachment, 0, 0, 0, 0, -1, -1);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, attachment, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.UnspecifiedAttachmentTextureColorCodingCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.UnspecifiedAttachmentTextureColorCodingCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.UnspecifiedAttachmentTextureColorCodingCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    // color
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyColorAttachment(gl.FRAMEBUFFER, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING, gl.LINEAR));

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);

    // depth
    renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, renderbufferID);

    this.check(glsStateQuery.verifyAttachment(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING, gl.LINEAR));

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase.prototype.test = function() {
    var framebufferID = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferID);
    // check color target
    this.testColorAttachment();

    // check depth target
    this.testDepthAttachment();
    gl.deleteFramebuffer(framebufferID);
};

/**
 * @constructor
 * @extends {es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.UnspecifiedAttachmentSizeRboCase = function(name, description) {
    es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase.call(this, name, description);
};

setParentClass(es3fFboStateQueryTests.UnspecifiedAttachmentSizeRboCase, es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase);

es3fFboStateQueryTests.UnspecifiedAttachmentSizeRboCase.prototype.testColorAttachment = function() {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbufferID);

    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, 0, 0, 0, 0, 0, 0);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);
};

es3fFboStateQueryTests.UnspecifiedAttachmentSizeRboCase.prototype.testDepthAttachment = function() {
    var renderbufferID = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, renderbufferID);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, renderbufferID);

    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, 0, 0, 0, 0, 0, 0);

    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, null);
    gl.deleteRenderbuffer(renderbufferID);
};

/**
 * @constructor
 * @extends {es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.UnspecifiedAttachmentSizeTextureCase = function(name, description) {
    es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase.call(this, name, description);
};

setParentClass(es3fFboStateQueryTests.UnspecifiedAttachmentSizeTextureCase, es3fFboStateQueryTests.UnspecifiedAttachmentSizeCase);

es3fFboStateQueryTests.UnspecifiedAttachmentSizeTextureCase.prototype.testColorAttachment = function() {
    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, textureID);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, textureID, 0);

    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, 0, 0, 0, 0, 0, 0);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
};

es3fFboStateQueryTests.UnspecifiedAttachmentSizeTextureCase.prototype.testDepthAttachment = function() {
    var textureID = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, textureID);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, textureID, 0);

    checkAttachmentComponentSizeExactly.bind(this, gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, 0, 0, 0, 0, 0, 0);

    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.TEXTURE_2D, null, 0);
    gl.deleteTexture(textureID);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fFboStateQueryTests.UnspecifiedAttachmentTextureComponentTypeCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
};

setParentClass(es3fFboStateQueryTests.UnspecifiedAttachmentTextureComponentTypeCase, es3fApiCase.ApiCase);

es3fFboStateQueryTests.UnspecifiedAttachmentTextureComponentTypeCase.prototype.test = function() {
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fFboStateQueryTests.FboStateQueryTests = function() {
    tcuTestCase.DeqpTest.call(this, 'fbo', 'Fbo State Query tests');
};

es3fFboStateQueryTests.FboStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fFboStateQueryTests.FboStateQueryTests.prototype.constructor = es3fFboStateQueryTests.FboStateQueryTests;

es3fFboStateQueryTests.FboStateQueryTests.prototype.init = function() {
    var red = /** @type {number} */ (gl.getParameter(gl.RED_BITS));
    var green = /** @type {number} */ (gl.getParameter(gl.GREEN_BITS));
    var blue = /** @type {number} */ (gl.getParameter(gl.BLUE_BITS));
    var alpha = /** @type {number} */ (gl.getParameter(gl.ALPHA_BITS));
    es3fFboStateQueryTests.colorBits = [red, green, blue, alpha];
    es3fFboStateQueryTests.depthBits = /** @type {number} */ (gl.getParameter(gl.DEPTH_BITS));
    es3fFboStateQueryTests.stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

    this.addChild(new es3fFboStateQueryTests.DefaultFramebufferCase('draw_framebuffer_default_framebuffer', 'default framebuffer', gl.DRAW_FRAMEBUFFER));
    this.addChild(new es3fFboStateQueryTests.DefaultFramebufferCase('read_framebuffer_default_framebuffer', 'default framebuffer', gl.READ_FRAMEBUFFER));
    this.addChild(new es3fFboStateQueryTests.AttachmentObjectCase('framebuffer_attachment_object', 'FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE and FRAMEBUFFER_ATTACHMENT_OBJECT_NAME'));
    this.addChild(new es3fFboStateQueryTests.AttachmentTextureLevelCase('framebuffer_attachment_texture_level', 'FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL'));
    this.addChild(new es3fFboStateQueryTests.AttachmentTextureCubeMapFaceCase('framebuffer_attachment_texture_cube_map_face', 'FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE'));
    this.addChild(new es3fFboStateQueryTests.AttachmentTextureLayerCase('framebuffer_attachment_texture_layer', 'FRAMEBUFFER_ATTACHMENT_TEXTURE_LAYER'));
    this.addChild(new es3fFboStateQueryTests.AttachmentTextureColorCodingCase('framebuffer_attachment_color_encoding', 'FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING'));
    this.addChild(new es3fFboStateQueryTests.AttachmentTextureComponentTypeCase('framebuffer_attachment_component_type', 'FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE'));
    this.addChild(new es3fFboStateQueryTests.AttachmentSizeInitialCase('framebuffer_attachment_x_size_initial', 'FRAMEBUFFER_ATTACHMENT_x_SIZE'));
    this.addChild(new es3fFboStateQueryTests.AttachmentSizeRboCase('framebuffer_attachment_x_size_rbo', 'FRAMEBUFFER_ATTACHMENT_x_SIZE'));
    this.addChild(new es3fFboStateQueryTests.AttachmentSizeTextureCase('framebuffer_attachment_x_size_texture', 'FRAMEBUFFER_ATTACHMENT_x_SIZE'));
    this.addChild(new es3fFboStateQueryTests.UnspecifiedAttachmentTextureColorCodingCase('framebuffer_unspecified_attachment_color_encoding', 'FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING'));
    this.addChild(new es3fFboStateQueryTests.UnspecifiedAttachmentTextureComponentTypeCase('framebuffer_unspecified_attachment_component_type', 'FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE'));
    this.addChild(new es3fFboStateQueryTests.UnspecifiedAttachmentSizeRboCase('framebuffer_unspecified_attachment_x_size_rbo', 'FRAMEBUFFER_ATTACHMENT_x_SIZE'));
    this.addChild(new es3fFboStateQueryTests.UnspecifiedAttachmentSizeTextureCase('framebuffer_unspecified_attachment_x_size_texture', 'FRAMEBUFFER_ATTACHMENT_x_SIZE'));
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fFboStateQueryTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fFboStateQueryTests.FboStateQueryTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fFboStateQueryTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});
