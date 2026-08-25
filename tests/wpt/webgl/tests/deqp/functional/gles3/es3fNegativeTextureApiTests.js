/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES 3.0 Module
 * -------------------------------------------------
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
 *//*!
 * \file
 * \brief Negative Texture API tests.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('functional.gles3.es3fNegativeTextureApiTests');

goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('functional.gles3.es3fApiCase');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

    var es3fNegativeTextureApiTests = functional.gles3.es3fNegativeTextureApiTests;
    var tcuTexture = framework.common.tcuTexture;
    var es3fApiCase = functional.gles3.es3fApiCase;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluTexture = framework.opengl.gluTexture;
    var gluTextureUtil = framework.opengl.gluTextureUtil;

    function etc2Unsupported() {
        debug("Skipping test: no support for WEBGL_compressed_texture_etc");
    }


    /**
     * @param {number} width
     * @param {number} height
     * @return {number}
     */
    es3fNegativeTextureApiTests.etc2DataSize = function(width, height) {
        return Math.ceil(width / 4) * Math.ceil(height / 4) * 8;
    };

    /**
     * @param {number} width
     * @param {number} height
     * @return {number}
     */
    es3fNegativeTextureApiTests.etc2EacDataSize = function(width, height) {
        return 2 * es3fNegativeTextureApiTests.etc2DataSize(width, height);
    };

    /**
     * @param {function(number)} func
     */
    es3fNegativeTextureApiTests.forCubeFaces = function(func) {
        var faceGLVar;
        for (var faceIterTcu in tcuTexture.CubeFace) {
            faceGLVar = gluTexture.cubeFaceToGLFace(tcuTexture.CubeFace[faceIterTcu]);
            func(faceGLVar);
        }
    };

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeTextureApiTests.init = function(gl) {

        var haveCompressedTextureETC = gluTextureUtil.enableCompressedTextureETC();

        var testGroup = tcuTestCase.runner.testCases;

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('activetexture', 'Invalid gl.ActiveTexture() usage', gl,
        function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if texture is not one of gl.TEXTUREi, where i ranges from 0 to (gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS - 1).');
            gl.activeTexture(-1);
            this.expectError(gl.INVALID_ENUM);
            var numMaxTextureUnits = /** @type {number} */(gl.getParameter(gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS));
            gl.activeTexture(gl.TEXTURE0 + numMaxTextureUnits);
            this.expectError(gl.INVALID_ENUM);

        }));

        // gl.bindTexture

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('bindTexture', 'Invalid gl.bindTexture() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the allowable values.');
            gl.bindTexture(0, texture[0]);
            this.expectError(gl.INVALID_ENUM);
            gl.bindTexture(gl.FRAMEBUFFER, texture[0]);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if texture was previously created with a target that doesn\'t match that of target.');
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            this.expectError(gl.NO_ERROR);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[0]);
            this.expectError(gl.INVALID_OPERATION);
            gl.bindTexture(gl.TEXTURE_3D, texture[0]);
            this.expectError(gl.INVALID_OPERATION);
            gl.bindTexture(gl.TEXTURE_2D_ARRAY, texture[0]);
            this.expectError(gl.INVALID_OPERATION);

            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            this.expectError(gl.NO_ERROR);
            gl.bindTexture(gl.TEXTURE_2D, texture[1]);
            this.expectError(gl.INVALID_OPERATION);
            gl.bindTexture(gl.TEXTURE_3D, texture[1]);
            this.expectError(gl.INVALID_OPERATION);
            gl.bindTexture(gl.TEXTURE_2D_ARRAY, texture[1]);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        // gl.compressedTexImage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_invalid_target', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(0);
            gl.compressedTexImage2D(0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_invalid_format', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_ENUM is generated if internalformat is not a supported format returned in gl.COMPRESSED_TEXTURE_FORMATS.');
            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(0);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, 0, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_ENUM);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_neg_level', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(0);
            gl.compressedTexImage2D(gl.TEXTURE_2D, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_max_level', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE) for a 2d texture target.');
            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16));

            /** @type {number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type {number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.compressedTexImage2D(gl.TEXTURE_2D, log2MaxTextureSize, gl.COMPRESSED_RGB8_ETC2, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_CUBE_MAP_TEXTURE_SIZE) for a cubemap target.');
            /** @type {number} */ var log2MaxCubemapSize = Math.floor(Math.log2(/** @type {number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)))) + 1;

            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, log2MaxCubemapSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, log2MaxCubemapSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, log2MaxCubemapSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, log2MaxCubemapSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, log2MaxCubemapSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, log2MaxCubemapSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, uint8);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_neg_width_height', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than 0.');

            bufferedLogToConsole('gl.TEXTURE_2D target');
            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(0);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, 0, uint8);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_max_width_height', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);

            var maxTextureSize = /** @type {number} */ (gl.getParameter(gl.MAX_TEXTURE_SIZE)) + 1;
            var maxCubemapSize = /** @type {number} */ (gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)) + 1;
            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is greater than gl.MAX_TEXTURE_SIZE.');

            var maxSideSize = Math.max(maxCubemapSize, maxTextureSize);
            var scratchBuffer = new ArrayBuffer(
                Math.max(es3fNegativeTextureApiTests.etc2EacDataSize(maxSideSize, 1),
                         es3fNegativeTextureApiTests.etc2EacDataSize(1, maxSideSize)));
            function getUint8ArrayEtc2EacDataSize(w, h) {
                return new Uint8Array(scratchBuffer, 0, es3fNegativeTextureApiTests.etc2EacDataSize(w, h));
            }

            var dataTextureMaxByOne = getUint8ArrayEtc2EacDataSize(maxTextureSize, 1);
            var dataTextureOneByMax = getUint8ArrayEtc2EacDataSize(1, maxTextureSize);

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxTextureSize, 1, 0, dataTextureMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxTextureSize, 0, dataTextureOneByMax);
            this.expectError(gl.INVALID_VALUE);

            var dataCubemapMaxByOne = getUint8ArrayEtc2EacDataSize(maxCubemapSize, 1);
            var dataCubemapOneByMax = getUint8ArrayEtc2EacDataSize(1, maxCubemapSize);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxCubemapSize, 1, 0, dataCubemapMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxCubemapSize, 0, dataCubemapOneByMax);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxCubemapSize, 1, 0, dataCubemapMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxCubemapSize, 0, dataCubemapOneByMax);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxCubemapSize, 1, 0, dataCubemapMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxCubemapSize, 0, dataCubemapOneByMax);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxCubemapSize, 1, 0, dataCubemapMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxCubemapSize, 0, dataCubemapOneByMax);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxCubemapSize, 1, 0, dataCubemapMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxCubemapSize, 0, dataCubemapOneByMax);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxCubemapSize, 1, 0, dataCubemapMaxByOne);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 1, maxCubemapSize, 0, dataCubemapOneByMax);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_invalid_border', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(0);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if border is not 0.');

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 1, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, uint8);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage2d_invalid_size', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }


            /** @type {WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if imageSize is not consistent with the format, dimensions, and contents of the specified compressed image data.');
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, new Uint8Array(1));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, new Uint8Array(4 * 4 * 8));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGB8_ETC2, 16, 16, 0, new Uint8Array(4 * 4 * 16));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_SIGNED_R11_EAC, 16, 16, 0, new Uint8Array(4 * 4 * 16));
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture);


        }));

        // gl.copyTexImage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_invalid_target', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);


            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.copyTexImage2D(0, 0, gl.RGB, 0, 0, 64, 64, 0);
            this.expectError(gl.INVALID_ENUM);


            gl.deleteTexture(texture);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_invalid_format', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_ENUM or gl.INVALID_VALUE is generated if internalformat is not an accepted format.');
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 64, 64, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, 0, 0, 0, 16, 16, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, 0, 0, 0, 16, 16, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, 0, 0, 0, 16, 16, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, 0, 0, 0, 16, 16, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, 0, 0, 0, 16, 16, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, 0, 0, 0, 16, 16, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_VALUE]);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_inequal_width_height_cube', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if target is one of the six cube map 2D image targets and the width and height parameters are not equal.');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, 16, 17, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, 16, 17, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, 16, 17, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, 16, 17, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, 16, 17, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, 16, 17, 0);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_neg_level', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.copyTexImage2D(gl.TEXTURE_2D, -1, gl.RGB, 0, 0, 64, 64, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, -1, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, -1, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, -1, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, -1, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, -1, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, -1, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_max_level', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type {number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type {number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.copyTexImage2D(gl.TEXTURE_2D, log2MaxTextureSize, gl.RGB, 0, 0, 64, 64, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_CUBE_MAP_TEXTURE_SIZE).');
            /** @type {number} */ var log2MaxCubemapSize = Math.floor(Math.log2(/** @type {number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)))) + 1;
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, log2MaxCubemapSize, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, log2MaxCubemapSize, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, log2MaxCubemapSize, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, log2MaxCubemapSize, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, log2MaxCubemapSize, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, log2MaxCubemapSize, gl.RGB, 0, 0, 16, 16, 0);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_neg_width_height', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than 0.');

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, -1, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, 1, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, -1, -1, 0);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_max_width_height', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            var maxTextureSize = /** @type {number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)) + 1;
            var maxCubemapSize = /** @type {number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)) + 1;

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is greater than gl.MAX_TEXTURE_SIZE.');

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, maxTextureSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, 1, maxTextureSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, maxTextureSize, maxTextureSize, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, 1, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, maxCubemapSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, maxCubemapSize, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, 1, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, maxCubemapSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, maxCubemapSize, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, 1, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, maxCubemapSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, maxCubemapSize, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, 1, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, maxCubemapSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, maxCubemapSize, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, 1, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, maxCubemapSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, maxCubemapSize, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, 1, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, maxCubemapSize, 1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, maxCubemapSize, maxCubemapSize, 0);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_invalid_border', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if border is not 0.');

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 0, 0, 0, 0, 1);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copyteximage2d_incomplete_framebuffer', 'Invalid gl.copyTexImage2D() usage', gl,
        function() {


            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            /** @type {WebGLFramebuffer} */ var fbo;
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            gl.copyTexImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGBA8, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);

            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        // gl.copyTexSubImage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_invalid_target', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {

            /** @type {WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.copyTexSubImage2D(0, 0, 0, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_ENUM);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_neg_level', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {
            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.texImage2D(faceGL, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            });

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.copyTexSubImage2D(gl.TEXTURE_2D, -1, 0, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            var local = this;
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.copyTexSubImage2D(faceGL, -1, 0, 0, 0, 0, 4, 4);
                local.expectError(gl.INVALID_VALUE);
            });

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_max_level', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.texImage2D (gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.texImage2D(faceGL, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            });

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE) for 2D texture targets.');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.copyTexSubImage2D(gl.TEXTURE_2D, log2MaxTextureSize, 0, 0, 0, 0, 4, 4);
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_CUBE_MAP_SIZE) for cubemap targets.');
            /** @type{number} */ var log2MaxCubemapSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)))) + 1;
            var local = this;
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.copyTexSubImage2D(faceGL, log2MaxCubemapSize, 0, 0, 0, 0, 4, 4);
                local.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            });

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_neg_offset', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D (gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset < 0 or yoffset < 0.');
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, -1, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, -1, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, -1, -1, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_invalid_offset', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D (gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset + width > texture_width or yoffset + height > texture_height.');
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 14, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, 14, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 14, 14, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_neg_width_height', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D (gl.TEXTURE_2D, 0, gl.RGBA, 16, 16, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than 0.');
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, -1, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, 0, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, -1, -1);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage2d_incomplete_framebuffer', 'Invalid gl.copyTexSubImage2D() usage', gl,
        function() {
            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            /** @type{Array<WebGLTexture>} */ var texture = [];
            /** @type{WebGLFramebuffer} */ var fbo;
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.NO_ERROR);

            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            this.expectError(gl.NO_ERROR);

            gl.copyTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, 0, 0, 0, 0, 0, 0);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);

            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);
            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        // glDeleteTextures

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('deletetextures', 'glDeleteTextures() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();

            bufferedLogToConsole('gl.NO_ERROR is generated if texture is null.');
            gl.deleteTexture(null);
            this.expectError(gl.NO_ERROR);

            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.deleteTexture(null);
            this.expectError(gl.NO_ERROR);

            gl.deleteTexture(texture);
        }));

        // gl.generateMipmap

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('generatemipmap', 'Invalid gl.generateMipmap() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            /** @type{WebGLFramebuffer} */ var fbo;
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.TEXTURE_2D or gl.TEXTURE_CUBE_MAP.');
            gl.generateMipmap(0);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('INVALID_OPERATION is generated if the texture bound to target is not cube complete.');
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[0]);
            gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, gl.REPEAT);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 0, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.generateMipmap(gl.TEXTURE_CUBE_MAP);
            this.expectError(gl.INVALID_OPERATION);

            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[0]);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 16, 16, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 16, 16, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 16, 16, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 16, 16, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 16, 16, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 32, 32, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.generateMipmap(gl.TEXTURE_CUBE_MAP);
            this.expectError(gl.INVALID_OPERATION);

            if (haveCompressedTextureETC) {
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the zero level array is stored in a compressed internal format.');
                gl.bindTexture(gl.TEXTURE_2D, texture[1]);
                gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, new Uint8Array(0));
                gl.generateMipmap(gl.TEXTURE_2D);
                this.expectError(gl.INVALID_OPERATION);
            } else {
                etc2Unsupported();
            }

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the level base array was not specified with an unsized internal format or a sized internal format that is both color-renderable and texture-filterable.');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB8_SNORM, 0, 0, 0, gl.RGB, gl.BYTE, null);
            gl.generateMipmap(gl.TEXTURE_2D);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8I, 0, 0, 0, gl.RED_INTEGER, gl.BYTE, null);
            gl.generateMipmap(gl.TEXTURE_2D);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 0, 0, 0, gl.RGBA, gl.FLOAT, null);
            gl.generateMipmap(gl.TEXTURE_2D);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        // gl.pixelStorei

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('pixelstorei', 'Invalid gl.pixelStorei() usage', gl,
        function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
            gl.pixelStorei(0,1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if a negative row length, pixel skip, or row skip value is specified, or if alignment is specified as other than 1, 2, 4, or 8.');
            gl.pixelStorei(gl.PACK_ROW_LENGTH, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.PACK_SKIP_ROWS, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.PACK_SKIP_PIXELS, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_ROW_LENGTH, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_IMAGE_HEIGHT, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_SKIP_ROWS, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_SKIP_PIXELS, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_SKIP_IMAGES, -1);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.PACK_ALIGNMENT, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_ALIGNMENT, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.PACK_ALIGNMENT, 16);
            this.expectError(gl.INVALID_VALUE);
            gl.pixelStorei(gl.UNPACK_ALIGNMENT, 16);
            this.expectError(gl.INVALID_VALUE);

        }));

        // gl.texImage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d', 'Invalid gl.texImage2D() usage', gl,
        function() {


            /** @type {WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);


            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.texImage2D(0, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not a type constant.');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, 0, null);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if format is not an accepted format constant.');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, 0, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if internalFormat is not one of the accepted resolution and format symbolic constants.');
            gl.texImage2D(gl.TEXTURE_2D, 0, 0, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the combination of internalFormat, format and type is invalid.');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGB, gl.UNSIGNED_SHORT_4_4_4_4, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB5_A1, 1, 1, 0, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB10_A2, 1, 1, 0, gl.RGB, gl.UNSIGNED_INT_2_10_10_10_REV, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32UI, 1, 1, 0, gl.RGBA_INTEGER, gl.INT, null);
            this.expectError(gl.INVALID_OPERATION);


            gl.deleteTexture(texture);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d_inequal_width_height_cube', 'Invalid gl.texImage2D() usage', gl,
        function() {

            /** @type {WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if target is one of the six cube map 2D image targets and the width and height parameters are not equal.');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 1, 2, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 1, 2, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 1, 2, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 1, 2, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 1, 2, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 1, 2, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d_neg_level', 'Invalid gl.texImage2D() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.texImage2D(gl.TEXTURE_2D, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, -1, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d_max_level', 'Invalid gl.texImage2D() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.texImage2D(gl.TEXTURE_2D, log2MaxTextureSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_CUBE_MAP_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxCubemapSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)))) + 1;
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, log2MaxCubemapSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, log2MaxCubemapSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, log2MaxCubemapSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, log2MaxCubemapSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, log2MaxCubemapSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, log2MaxCubemapSize, gl.RGB, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d_neg_width_height', 'Invalid gl.texImage2D() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than 0.');

            bufferedLogToConsole('gl.TEXTURE_2D target');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, -1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, -1, -1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d_max_width_height', 'Invalid gl.texImage2D() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            var maxTextureSize = /** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)) + 1;
            var maxCubemapSize = /** @type{number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)) + 1;

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is greater than gl.MAX_TEXTURE_SIZE.');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, maxTextureSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, maxTextureSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, maxTextureSize, maxTextureSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is greater than gl.MAX_CUBE_MAP_TEXTURE_SIZE.');

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_X target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, maxCubemapSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 1, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, maxCubemapSize, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Y target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, maxCubemapSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 1, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, maxCubemapSize, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_POSITIVE_Z target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, maxCubemapSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 1, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, maxCubemapSize, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_X target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, maxCubemapSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 1, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, maxCubemapSize, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Y target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, maxCubemapSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 1, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, maxCubemapSize, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.TEXTURE_CUBE_MAP_NEGATIVE_Z target');
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, maxCubemapSize, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 1, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, maxCubemapSize, maxCubemapSize, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);


        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage2d_invalid_border', 'Invalid gl.texImage2D() usage', gl,
        function() {

            /** @type {Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);


            bufferedLogToConsole('gl.INVALID_VALUE is generated if border is not 0.');
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 1, 1, -1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Y, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_POSITIVE_Z, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_X, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Y, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage2D(gl.TEXTURE_CUBE_MAP_NEGATIVE_Z, 0, gl.RGB, 1, 1, 1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);


            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        // gl.texSubImage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage2d', 'Invalid gl.texSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(64);
            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.texSubImage2D(0, 0, 0, 0, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if format is not an accepted format constant.');
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 4, 4, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not a type constant.');
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGB, 0, uint8);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the combination of internalFormat of the previously specified texture array, format and type is not valid.');
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGBA, gl.UNSIGNED_SHORT_5_6_5, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGB, gl.UNSIGNED_SHORT_4_4_4_4, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGBA_INTEGER, gl.UNSIGNED_INT, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 4, 4, gl.RGB, gl.FLOAT, uint8);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage2d_neg_level', 'Invalid gl.texSubImage2D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 32, 32, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.texImage2D(faceGL, 0, gl.RGB, 32, 32, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            });
            this.expectError(gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.texSubImage2D(gl.TEXTURE_2D, -1, 0, 0, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            var local = this;
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.texSubImage2D(faceGL, -1, 0, 0, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
                local.expectError(gl.INVALID_VALUE);
            });

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage2d_max_level', 'Invalid gl.texSubImage2D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture[0]);
            gl.texImage2D (gl.TEXTURE_2D, 0, gl.RGB, 32, 32, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            gl.bindTexture (gl.TEXTURE_CUBE_MAP, texture[1]);
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.texImage2D(faceGL, 0, gl.RGB, 32, 32, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            });

            this.expectError (gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.texSubImage2D(gl.TEXTURE_2D, log2MaxTextureSize, 0, 0, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_CUBE_MAP_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxCubemapSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)))) + 1;
            var local = this;
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.texSubImage2D(faceGL, log2MaxCubemapSize, 0, 0, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
                local.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            });

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage2d_neg_offset', 'Invalid gl.texSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB, 32, 32, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset or yoffset are negative.');
            gl.texSubImage2D(gl.TEXTURE_2D, 0, -1, 0, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, -1, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, -1, -1, 0, 0, gl.RGB, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage2d_invalid_offset', 'Invalid gl.texSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(64);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset + width > texture_width or yoffset + height > texture_height.');
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 30, 0, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 30, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 30, 30, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage2d_neg_width_height', 'Invalid gl.texSubImage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than 0.');
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, -1, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, -1, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, -1, -1, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        // gl.texParameteri

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texparameteri', 'Invalid gl.texParameteri() usage', gl,
        function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or pname is not one of the accepted defined values.');
            gl.texParameteri(0, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameteri(gl.TEXTURE_2D, 0, gl.LINEAR);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameteri(0, 0, gl.LINEAR);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined symbolic constant value (based on the value of pname) and does not.');
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.REPEAT);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.NEAREST);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or pname is not one of the accepted defined values.');
            gl.texParameteri(0, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameteri(gl.TEXTURE_2D, 0, gl.LINEAR);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameteri(0, 0, gl.LINEAR);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined symbolic constant value (based on the value of pname) and does not.');
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, 0);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.REPEAT);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, 0);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.NEAREST);
            this.expectError(gl.INVALID_ENUM);

            gl.deleteTexture(texture);
        }));

        // gl.texParameterf

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texparameterf', 'Invalid gl.texParameterf() usage', gl,
        function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or pname is not one of the accepted defined values.');
            gl.texParameterf(0, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameterf(gl.TEXTURE_2D, 0, gl.LINEAR);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameterf(0, 0, gl.LINEAR);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined symbolic constant value (based on the value of pname) and does not.');
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.REPEAT);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, 0);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.NEAREST);
            this.expectError([gl.INVALID_ENUM, gl.INVALID_OPERATION]);

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or pname is not one of the accepted defined values.');
            gl.texParameterf(0, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameterf(gl.TEXTURE_2D, 0, gl.LINEAR);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameterf(0, 0, gl.LINEAR);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined symbolic constant value (based on the value of pname) and does not.');
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, 0);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.REPEAT);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, 0);
            this.expectError(gl.INVALID_ENUM);
            gl.texParameterf(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.NEAREST);
            this.expectError(gl.INVALID_ENUM);

            gl.deleteTexture(texture);
        }));

        // gl.compressedTexSubImage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d', 'Invalid gl.compressedTexSubImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.compressedTexSubImage2D(0, 0, 0, 0, 0, 0, gl.COMPRESSED_RGB8_ETC2, new Uint8Array(0));
            this.expectError(gl.INVALID_ENUM);

            gl.compressedTexImage2D (gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 18, 18, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if format does not match the internal format of the texture image being modified.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, gl.COMPRESSED_RGB8_ETC2, new Uint8Array(0));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('For ETC2/EAC images gl.INVALID_OPERATION is generated if width is not a multiple of four, and width + xoffset is not equal to the width of the texture level.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 4, 0, 10, 4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(10, 4)));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('For ETC2/EAC images gl.INVALID_OPERATION is generated if height is not a multiple of four, and height + yoffset is not equal to the height of the texture level.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 4, 4, 10, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 10)));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('For ETC2/EAC images gl.INVALID_OPERATION is generated if xoffset or yoffset is not a multiple of four.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 1, 4, 4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError(gl.INVALID_OPERATION);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 1, 0, 4, 4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError(gl.INVALID_OPERATION);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 1, 1, 4, 4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d_neg_level', 'Invalid gl.compressedTexSubImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.compressedTexImage2D (gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 18, 18, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.compressedTexImage2D(faceGL, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 18, 18, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            });

            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, -1, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            var local = this;
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.compressedTexSubImage2D(faceGL, -1, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
                local.expectError(gl.INVALID_VALUE);
            });

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d_max_level', 'Invalid gl.compressedTexSubImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture[0]);
            gl.compressedTexImage2D (gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 18, 18, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture[1]);
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.compressedTexImage2D(faceGL, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 18, 18, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            });

            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, log2MaxTextureSize, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_CUBE_MAP_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxCubemapSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE)))) + 1;
            var local = this;
            es3fNegativeTextureApiTests.forCubeFaces(function(faceGL) {
                gl.compressedTexSubImage2D(faceGL, log2MaxCubemapSize, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
                local.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            });

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d_neg_offset', 'Invalid gl.compressedTexSubImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);
            gl.compressedTexImage2D(gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 8, 8, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(8, 8)));

            // \note Both gl.INVALID_VALUE and gl.INVALID_OPERATION are valid here since implementation may
            //         first check if offsets are valid for certain format and only after that check that they
            //         are not negative.
            bufferedLogToConsole('gl.INVALID_VALUE or gl.INVALID_OPERATION is generated if xoffset or yoffset are negative.');

            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, -4, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, -4, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, -4, -4, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d_invalid_offset', 'Invalid gl.compressedTexSubImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);
            gl.compressedTexImage2D (gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE or gl.INVALID_OPERATION is generated if xoffset + width > texture_width or yoffset + height > texture_height.');

            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 12, 0, 8, 4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(8, 4)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 12, 4, 8, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 8)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 12, 12, 8, 8, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(8, 8)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d_neg_width_height', 'Invalid gl.compressedTexSubImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);
            gl.compressedTexImage2D (gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE or gl.INVALID_OPERATION is generated if width or height is less than 0.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, -4, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, -4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, -4, -4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage2d_invalid_size', 'Invalid gl.compressedTexImage2D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);
            gl.compressedTexImage2D (gl.TEXTURE_2D, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if imageSize is not consistent with the format, dimensions, and contents of the specified compressed image data.');
            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(1));
            this.expectError(gl.INVALID_VALUE);

            gl.compressedTexSubImage2D(gl.TEXTURE_2D, 0, 0, 0, 16, 16, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(4*4*16-1));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        // gl.texImage3D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage3d', 'Invalid gl.texImage3D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture[0]);
            gl.bindTexture (gl.TEXTURE_3D, texture[1]);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.texImage3D(0, 0, gl.RGBA, 1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_ENUM);
            gl.texImage3D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not a type constant.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, 1, 0, gl.RGBA, 0, null);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if format is not an accepted format constant.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, 1, 0, 0, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if internalFormat is not one of the accepted resolution and format symbolic constants.');
            gl.texImage3D(gl.TEXTURE_3D, 0, 0, 1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if target is gl.TEXTURE_3D and format is gl.DEPTH_COMPONENT, or gl.DEPTH_STENCIL.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, 1, 0, gl.DEPTH_STENCIL, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, 1, 0, gl.DEPTH_COMPONENT, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the combination of internalFormat, format and type is invalid.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGB, 1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_SHORT_4_4_4_4, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGB5_A1, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGB10_A2, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_INT_2_10_10_10_REV, null);
            this.expectError(gl.INVALID_OPERATION);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA32UI, 1, 1, 1, 0, gl.RGBA_INTEGER, gl.INT, null);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage3d_neg_level', 'Invalid gl.texImage3D() usage', gl,
        function() {
            // NOTE: this method hangs the browser if the textures are binded.
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.texImage3D(gl.TEXTURE_3D, -1, gl.RGB, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, -1, gl.RGB, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage3d_max_level', 'Invalid gl.texImage3D() usage', gl,
        function() {

            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_3D_TEXTURE_SIZE).');
            /** @type{number} */ var log2Max3DTextureSize = Math.floor(Math.log2(/** @type{number} */ (gl.getParameter(gl.MAX_3D_TEXTURE_SIZE)))) + 1;
            gl.texImage3D(gl.TEXTURE_3D, log2Max3DTextureSize, gl.RGB, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */ (gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, log2MaxTextureSize, gl.RGB, 1, 1, 1, 0, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage3d_neg_width_height_depth', 'Invalid gl.texImage3D() usage', gl,
        function() {

            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than 0.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, -1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, -1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, -1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, -1, -1, -1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, -1, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 1, -1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 1, 1, -1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, -1, -1, -1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage3d_max_width_height_depth', 'Invalid gl.texImage3D() usage', gl,
        function() {

            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);

            var max3DTextureSize = /** @type{number} */ (gl.getParameter(gl.MAX_3D_TEXTURE_SIZE)) + 1;
            var maxTextureSize = /** @type{number} */ (gl.getParameter(gl.MAX_TEXTURE_SIZE)) + 1;

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width, height or depth is greater than gl.MAX_3D_TEXTURE_SIZE.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, max3DTextureSize, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, max3DTextureSize, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 1, 1, max3DTextureSize, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, max3DTextureSize, max3DTextureSize, max3DTextureSize, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width, height or depth is greater than gl.MAX_TEXTURE_SIZE.');
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, maxTextureSize, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 1, maxTextureSize, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 1, 1, maxTextureSize, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, maxTextureSize, maxTextureSize, maxTextureSize, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('teximage3d_invalid_border', 'Invalid gl.texImage3D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if border is not 0 or 1.');
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGB, 1, 1, 1, -1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGB, 1, 1, 1, 2, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGB, 1, 1, 1, -1, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGB, 1, 1, 1, 2, gl.RGB, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        // gl.texSubImage3D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage3d', 'Invalid gl.texSubImage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(256);
            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.texSubImage3D(0, 0, 0, 0, 0, 4, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_ENUM);
            gl.texSubImage3D(gl.TEXTURE_2D, 0, 0, 0, 0, 4, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if format is not an accepted format constant.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 0, 4, 4, 4, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not a type constant.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 4, 4, 4, gl.RGB, 0, uint8);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the combination of internalFormat of the previously specified texture array, format and type is not valid.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 4, 4, 4, gl.RGB, gl.UNSIGNED_SHORT_4_4_4_4, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 4, 4, 4, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 4, 4, 4, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 4, 4, 4, gl.RGBA_INTEGER, gl.UNSIGNED_INT, uint8);
            this.expectError(gl.INVALID_OPERATION);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 4, 4, 4, gl.RGB, gl.FLOAT, uint8);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage3d_neg_level', 'Invalid gl.texSubImage3D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);
            gl.texImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.texSubImage3D(gl.TEXTURE_3D, -1, 0, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, -1, 0, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage3d_max_level', 'Invalid gl.texSubImage3D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);
            gl.texImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            /** @type{number} */ var log2Max3DTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_3D_TEXTURE_SIZE)))) + 1;
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_3D_TEXTURE_SIZE).');
            gl.texSubImage3D(gl.TEXTURE_3D, log2Max3DTextureSize, 0, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, log2MaxTextureSize, 0, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage3d_neg_offset', 'Invalid gl.texSubImage3D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);
            gl.texImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset, yoffset or zoffset are negative.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, -1, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, -1, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, -1, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, -1, -1, -1, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, -1, 0, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, -1, 0, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, -1, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_2D_ARRAY, 0, -1, -1, -1, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage3d_invalid_offset', 'Invalid gl.texSubImage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(256);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset + width > texture_width.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 2, 0, 0, 4, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if yoffset + height > texture_height.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 2, 0, 4, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if zoffset + depth > texture_depth.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 2, 4, 4, 4, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texsubimage3d_neg_width_height', 'Invalid gl.texSubImage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            /** @type {ArrayBufferView} */ var uint8 = new Uint8Array(4);
            bufferedLogToConsole('gl.INVALID_VALUE is generated if width, height or depth is less than 0.');
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, -1, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 0, -1, 0, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 0, 0, -1, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);
            gl.texSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, -1, -1, -1, gl.RGBA, gl.UNSIGNED_BYTE, uint8);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.copyTexSubImage3D(0, 0, 0, 0, 0, 0, 0, 4, 0);
            this.expectError(gl.INVALID_ENUM);

            gl.deleteTexture(texture);
        }));
        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d_neg_level', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);
            gl.texImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, -1, 0, 0, 0, 0, 0, 4, 0);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage3D(gl.TEXTURE_2D_ARRAY, -1, 0, 0, 0, 0, 0, 4, 0);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d_max_level', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            /** @type{number} */ var log2Max3DTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_3D_TEXTURE_SIZE)))) + 1;
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;

            /** @type{Array<WebGLTexture>} */ var texture = [];
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture[0]);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);
            gl.texImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_3D_TEXTURE_SIZE).');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, log2Max3DTextureSize, 0, 0, 0, 0, 0, 4, 0);
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            gl.copyTexSubImage3D(gl.TEXTURE_2D_ARRAY, log2MaxTextureSize, 0, 0, 0, 0, 0, 4, 0);
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d_neg_offset', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset, yoffset or zoffset is negative.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, -1, 0, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, -1, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, 0, -1, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, -1, -1, -1, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d_invalid_offset', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if xoffset + width > texture_width.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 1, 0, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if yoffset + height > texture_height.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, 1, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if zoffset + 1 > texture_depth.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 4, 0, 0, 4, 4);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d_neg_width_height', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);
            gl.texImage3D (gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width < 0.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 0, 0, -4, 4);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if height < 0.');
            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 0, 0, 4, -4);
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('copytexsubimage3d_incomplete_framebuffer', 'Invalid gl.copyTexSubImage3D() usage', gl,
        function() {
            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            /** @type{Array<WebGLTexture>} */ var texture = [];
            /** @type{WebGLFramebuffer} */ var fbo;
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_3D, texture[0]);
            gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            gl.bindTexture(gl.TEXTURE_2D_ARRAY, texture[1]);
            gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
            this.expectError(gl.NO_ERROR);

            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.READ_FRAMEBUFFER);
            this.expectError(gl.NO_ERROR);

            gl.copyTexSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.copyTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 0, 0, 4, 4);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);

            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);
            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        // gl.compressedTexImage3D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{Array<WebGLTexture>} */ var texture = [];

            // We have to create and bind textures to each target for the test because default textures are not supported by WebGL.
            texture[0] = gl.createTexture();
            texture[1] = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_CUBE_MAP, texture[0]);
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture[1]);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            gl.compressedTexImage3D(0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage3D(gl.TEXTURE_CUBE_MAP_POSITIVE_X, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if internalformat is not one of the specific compressed internal formats.');
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_ENUM);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA8, 0, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_ENUM);

            gl.deleteTexture(texture[0]);
            gl.deleteTexture(texture[1]);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d_neg_level', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, -1, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d_max_level', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, log2MaxTextureSize, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, 0, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d_neg_width_height_depth', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width, height or depth is less than 0.');
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, -1, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, -1, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, -1, -1, -1, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d_max_width_height_depth', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);

            var maxTextureSize = /** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)) + 1;

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width, height or depth is greater than gl.MAX_TEXTURE_SIZE.');
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxTextureSize, 0, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, maxTextureSize, 0, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, maxTextureSize, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, maxTextureSize, maxTextureSize, maxTextureSize, 0, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d_invalid_border', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if border is not 0.');
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, -1, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, 1, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedteximage3d_invalid_size', 'Invalid gl.compressedTexImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{ WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if imageSize is not consistent with the format, dimensions, and contents of the specified compressed image data.');
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 0, 0, 0, 0, new Uint8Array(1));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 1, 0, new Uint8Array(4*4*8));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGB8_ETC2, 16, 16, 1, 0, new Uint8Array(4*4*16));
            this.expectError(gl.INVALID_VALUE);
            gl.compressedTexImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_SIGNED_R11_EAC, 16, 16, 1, 0, new Uint8Array(4*4*16));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);

        }));

        // gl.compressedTexSubImage3D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is invalid.');
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.compressedTexSubImage3D(0, 0, 0, 0, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError(gl.INVALID_ENUM);

            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 18, 18, 1, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if format does not match the internal format of the texture image being modified.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 0, 0, 0, gl.COMPRESSED_RGB8_ETC2, new Uint8Array(0));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if internalformat is an ETC2/EAC format and target is not gl.TEXTURE_2D_ARRAY.');
            gl.compressedTexSubImage3D(gl.TEXTURE_3D, 0, 0, 0, 0, 18, 18, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(18, 18)));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('For ETC2/EAC images gl.INVALID_OPERATION is generated if width is not a multiple of four, and width + xoffset is not equal to the width of the texture level.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 4, 0, 0, 10, 4, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(10, 4)));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('For ETC2/EAC images gl.INVALID_OPERATION is generated if height is not a multiple of four, and height + yoffset is not equal to the height of the texture level.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 4, 0, 4, 10, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 10)));
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('For ETC2/EAC images gl.INVALID_OPERATION is generated if xoffset or yoffset is not a multiple of four.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 1, 0, 0, 4, 4, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError(gl.INVALID_OPERATION);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 1, 0, 4, 4, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError(gl.INVALID_OPERATION);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 1, 1, 0, 4, 4, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d_neg_level', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 1, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, -1, 0, 0, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d_max_level', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 1, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if level is greater than log_2(gl.MAX_TEXTURE_SIZE).');
            /** @type{number} */ var log2MaxTextureSize = Math.floor(Math.log2(/** @type{number} */(gl.getParameter(gl.MAX_TEXTURE_SIZE)))) + 1;
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, log2MaxTextureSize, 0, 0, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d_neg_offset', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 1, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE or gl.INVALID_OPERATION is generated if xoffset, yoffset or zoffset are negative.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, -4, 0, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, -4, 0, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, -4, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, -4, -4, -4, 0, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d_invalid_offset', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 4, 4, 1, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE or gl.INVALID_OPERATION is generated if xoffset + width > texture_width or yoffset + height > texture_height.');

            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 12, 0, 0, 8, 4, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(8, 4)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 12, 0, 4, 8, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 8)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 12, 4, 4, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(4, 4)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 12, 12, 12, 8, 8, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(8, 8)));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d_neg_width_height_depth', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 1, 0, new Uint8Array(es3fNegativeTextureApiTests.etc2EacDataSize(16, 16)));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE or gl.INVALID_OPERATION is generated if width, height or depth are negative.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, -4, 0, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 0, -4, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 0, 0, -4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, -4, -4, -4, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError([gl.INVALID_VALUE, gl.INVALID_OPERATION]);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('compressedtexsubimage3d_invalid_size', 'Invalid gl.compressedTexSubImage3D() usage', gl,
        function() {
            if (!haveCompressedTextureETC) { etc2Unsupported(); return; }

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D_ARRAY, texture);
            gl.compressedTexImage3D (gl.TEXTURE_2D_ARRAY, 0, gl.COMPRESSED_RGBA8_ETC2_EAC, 16, 16, 1, 0, new Uint8Array(4*4*16));
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if imageSize is not consistent with the format, dimensions, and contents of the specified compressed image data.');
            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 16, 16, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(0));
            this.expectError(gl.INVALID_VALUE);

            gl.compressedTexSubImage3D(gl.TEXTURE_2D_ARRAY, 0, 0, 0, 0, 16, 16, 1, gl.COMPRESSED_RGBA8_ETC2_EAC, new Uint8Array(4*4*16-1));
            this.expectError(gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        // gl.texStorage2D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texstorage2d', 'Invalid gl.texStorage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_ENUM or gl.INVALID_VALUE is generated if internalformat is not a valid sized internal format.');
            gl.texStorage2D (gl.TEXTURE_2D, 1, 0, 16, 16);
            this.expectError ([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA_INTEGER, 16, 16);
            this.expectError ([gl.INVALID_ENUM, gl.INVALID_VALUE]);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted target enumerants.');
            gl.texStorage2D (0, 1, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_ENUM);
            gl.texStorage2D (gl.TEXTURE_3D, 1, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_ENUM);
            gl.texStorage2D (gl.TEXTURE_2D_ARRAY, 1, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height are less than 1.');
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA8, 0, 16);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA8, 16, 0);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA8, 0, 0);
            this.expectError (gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texstorage2d_invalid_binding', 'Invalid gl.texStorage2D() usage', gl,
        function() {
            gl.bindTexture (gl.TEXTURE_2D, null);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if there is no texture object curently bound to target.');
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_OPERATION);

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the texture object currently bound to target already has gl.TEXTURE_IMMUTABLE_FORMAT set to true.');
            /** @type{number} */ var immutable;
            immutable = /** @type{number} */(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_IMMUTABLE_FORMAT));
            bufferedLogToConsole('// gl.TEXTURE_IMMUTABLE_FORMAT = ' + ((immutable != 0) ? 'true' : 'false'));
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA8, 16, 16);
            this.expectError (gl.NO_ERROR);
            immutable = /** @type{number} */(gl.getTexParameter(gl.TEXTURE_2D, gl.TEXTURE_IMMUTABLE_FORMAT));
            bufferedLogToConsole('// gl.TEXTURE_IMMUTABLE_FORMAT = ' + ((immutable != 0) ? 'true' : 'false'));
            gl.texStorage2D (gl.TEXTURE_2D, 1, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texstorage2d_invalid_levels', 'Invalid gl.texStorage2D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if levels is less than 1.');
            gl.texStorage2D (gl.TEXTURE_2D, 0, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage2D (gl.TEXTURE_2D, 0, gl.RGBA8, 0, 0);
            this.expectError (gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if levels is greater than floor(log_2(max(width, height))) + 1');
            /** @type{number} */ var log2MaxSize = Math.floor(Math.log2(Math.max(16, 4))) + 1 + 1;
            gl.texStorage2D (gl.TEXTURE_2D, log2MaxSize, gl.RGBA8, 16, 4);
            this.expectError (gl.INVALID_OPERATION);
            gl.texStorage2D (gl.TEXTURE_2D, log2MaxSize, gl.RGBA8, 4, 16);
            this.expectError (gl.INVALID_OPERATION);
            gl.texStorage2D (gl.TEXTURE_2D, log2MaxSize, gl.RGBA8, 16, 16);
            this.expectError (gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        // gl.texStorage3D

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texstorage3d', 'Invalid gl.texStorage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);

            bufferedLogToConsole('gl.INVALID_ENUM or gl.INVALID_VALUE is generated if internalformat is not a valid sized internal format.');
            gl.texStorage3D (gl.TEXTURE_3D, 1, 0, 4, 4, 4);
            this.expectError ([gl.INVALID_ENUM, gl.INVALID_VALUE]);
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA_INTEGER, 4, 4, 4);
            this.expectError ([gl.INVALID_ENUM, gl.INVALID_VALUE]);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted target enumerants.');
            gl.texStorage3D (0, 1, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.INVALID_ENUM);
            gl.texStorage3D (gl.TEXTURE_CUBE_MAP, 1, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.INVALID_ENUM);
            gl.texStorage3D (gl.TEXTURE_2D, 1, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if width, height or depth are less than 1.');
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 0, 4, 4);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 4, 0, 4);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 4, 4, 0);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 0, 0, 0);
            this.expectError (gl.INVALID_VALUE);

            gl.deleteTexture(texture);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texstorage3d_invalid_binding', 'Invalid gl.texStorage3D() usage', gl,
        function() {
            gl.bindTexture (gl.TEXTURE_3D, null);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if there is no texture object curently bound to target.');
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.INVALID_OPERATION);

            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the texture object currently bound to target already has gl.TEXTURE_IMMUTABLE_FORMAT set to true.');
            /** @type{number} */ var immutable;
            immutable = /** @type{number} */(gl.getTexParameter(gl.TEXTURE_3D, gl.TEXTURE_IMMUTABLE_FORMAT));
            bufferedLogToConsole('// gl.TEXTURE_IMMUTABLE_FORMAT = ' + ((immutable != 0) ? 'true' : 'false'));
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.NO_ERROR);
            immutable = /** @type{number} */(gl.getTexParameter(gl.TEXTURE_3D, gl.TEXTURE_IMMUTABLE_FORMAT));
            bufferedLogToConsole('// gl.TEXTURE_IMMUTABLE_FORMAT = ' + ((immutable != 0) ? 'true' : 'false'));
            gl.texStorage3D (gl.TEXTURE_3D, 1, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));

        // NOTE: the test doesn't cause glError using the parameters defined in the original test of C code
        testGroup.addChild(new es3fApiCase.ApiCaseCallback('texstorage3d_invalid_levels', 'Invalid gl.texStorage3D() usage', gl,
        function() {
            /** @type{WebGLTexture} */ var texture;
            texture = gl.createTexture();
            gl.bindTexture (gl.TEXTURE_3D, texture);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if levels is less than 1.');
            gl.texStorage3D (gl.TEXTURE_3D, 0, gl.RGBA8, 4, 4, 4);
            this.expectError (gl.INVALID_VALUE);
            gl.texStorage3D (gl.TEXTURE_3D, 0, gl.RGBA8, 0, 0, 0);
            this.expectError (gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if levels is greater than floor(log_2(max(width, height, depth))) + 1');
            /** @type{number} */ var log2MaxSize = Math.floor(Math.log2(8)) + 1 + 1;
            gl.texStorage3D (gl.TEXTURE_3D, log2MaxSize, gl.RGBA8, 8, 2, 2);
            this.expectError (gl.INVALID_OPERATION);
            gl.texStorage3D (gl.TEXTURE_3D, log2MaxSize, gl.RGBA8, 2, 8, 2);
            this.expectError (gl.INVALID_OPERATION);
            gl.texStorage3D (gl.TEXTURE_3D, log2MaxSize, gl.RGBA8, 2, 2, 8);
            this.expectError (gl.INVALID_OPERATION);
            gl.texStorage3D (gl.TEXTURE_3D, log2MaxSize, gl.RGBA8, 8, 8, 8);
            this.expectError (gl.INVALID_OPERATION);

            gl.deleteTexture(texture);
        }));
    };

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeTextureApiTests.run = function(gl) {
        var testName = 'negativeTextureApi';
        var testDescription = 'Negative Texture API tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fNegativeTextureApiTests.init(gl);
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };

});
