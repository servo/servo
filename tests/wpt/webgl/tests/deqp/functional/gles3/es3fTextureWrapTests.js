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
goog.provide('functional.gles3.es3fTextureWrapTests');
goog.require('framework.common.tcuCompressedTexture');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexLookupVerifier');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluTextureUtil');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {
    /** @type {?WebGL2RenderingContext} */ var gl;

    var es3fTextureWrapTests = functional.gles3.es3fTextureWrapTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluTexture = framework.opengl.gluTexture;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
    var tcuCompressedTexture = framework.common.tcuCompressedTexture;
    var deRandom = framework.delibs.debase.deRandom;
    var deMath = framework.delibs.debase.deMath;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexLookupVerifier = framework.common.tcuTexLookupVerifier;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var deString = framework.delibs.debase.deString;

    /**
     * @enum {number}
     */
    var Viewport = {
        WIDTH: 256,
        HEIGHT: 256
    };

    /**
     * @constructor
     * @param {Array<number>=} bl
     * @param {Array<number>=} tr
     */
    es3fTextureWrapTests.Case = function(bl, tr) {
        /** @type {?Array<number>} */ this.bottomLeft = bl || null;
        /** @type {?Array<number>} */ this.topRight = tr || null;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     * @param {?tcuCompressedTexture.Format} compressedFormat
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} width
     * @param {number} height
     */
    es3fTextureWrapTests.TextureWrapCase = function(name, description, compressedFormat, wrapS, wrapT, minFilter, magFilter, width, height) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {number} */ this.m_format = gl.NONE;
        /** @type {number} */ this.m_dataType = gl.NONE;
        /** @type {?tcuCompressedTexture.Format} */ this.m_compressedFormat = compressedFormat;
        /** @type {number} */ this.m_wrapS = wrapS;
        /** @type {number} */ this.m_wrapT = wrapT;
        /** @type {number} */ this.m_minFilter = minFilter;
        /** @type {number} */ this.m_magFilter = magFilter;
        /** @type {number} */ this.m_width = width;
        /** @type {number} */ this.m_height = height;
        /** @type {Array<es3fTextureWrapTests.Case>} */ this.m_cases = [];
        /** @type {number} */ this.m_caseNdx = 0;
        /** @type {gluTexture.Texture2D} */ this.m_texture = null;
        /** @type {glsTextureTestUtil.TextureRenderer} */
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(gluShaderUtil.getGLSLVersionString(gluShaderUtil.GLSLVersion.V300_ES), gluShaderUtil.precision.PRECISION_MEDIUMP);
    };

    es3fTextureWrapTests.TextureWrapCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);

    /** Copy the constructor */
    es3fTextureWrapTests.TextureWrapCase.prototype.constructor = es3fTextureWrapTests.TextureWrapCase;

    /**
     * @param {string} name
     * @param {string} description
     * @param {number} format
     * @param {number} dataType
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} width
     * @param {number} height
     * @return {es3fTextureWrapTests.TextureWrapCase}
     */
    es3fTextureWrapTests.textureWrapCaseFromFormat = function(name, description, format, dataType, wrapS, wrapT, minFilter, magFilter, width, height) {
        var texWrapCase = new es3fTextureWrapTests.TextureWrapCase(name, description, null, wrapS, wrapT, minFilter, magFilter, width, height);
        texWrapCase.m_format = gl.RGBA;
        texWrapCase.m_dataType = gl.UNSIGNED_BYTE;
        return texWrapCase;
    };

    /**
     */
    es3fTextureWrapTests.TextureWrapCase.prototype.init = function() {
        if (this.m_compressedFormat !== null) {
            // Generate compressed texture.

            assertMsgOptions(this.m_format == gl.NONE && this.m_dataType == gl.NONE, 'init/compressedFormat', false, true);
            if (tcuCompressedTexture.isEtcFormat(this.m_compressedFormat)) {
                // Create ETC texture. Any content is valid.

                /** @type {tcuCompressedTexture.CompressedTexture}*/
                var compressedTexture = new tcuCompressedTexture.CompressedTexture(this.m_compressedFormat, this.m_width, this.m_height);
                /** @type {number} */ var dataSize = compressedTexture.getDataSize();
                /** @type {goog.NumberArray} */ var data = compressedTexture.getData();
                /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));

                for (var i = 0; i < dataSize; i++)
                    data[i] = rnd.getFloat() & 0xff;

                this.m_texture = gluTexture.texture2DFromCompressedTexture(gl, 1, [compressedTexture]);
            } else
                throw new Error('Only ETC2 and EAC are supported.');
        } else{
            this.m_texture = gluTexture.texture2DFromFormat(gl, this.m_format, this.m_dataType, this.m_width, this.m_height);

            // Fill level 0.
            this.m_texture.getRefTexture().allocLevel(0);
            tcuTextureUtil.fillWithComponentGradients(this.m_texture.getRefTexture().getLevel(0), [-0.5, -0.5, -0.5, 2.0], [1.0, 1.0, 1.0, 0.0]);

            this.m_texture.upload();
        }

        // Sub-cases.

        this.m_cases.push(new es3fTextureWrapTests.Case([-1.5, -3.0], [1.5, 2.5]));
        this.m_cases.push(new es3fTextureWrapTests.Case([-0.5, 0.75], [0.25, 1.25]));
        assertMsgOptions(this.m_caseNdx == 0, 'm_caseNdx != 0', false, true);
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureWrapTests.TextureWrapCase.prototype.iterate = function() {
        /** @type {glsTextureTestUtil.RandomViewport} */ var viewport = new glsTextureTestUtil.RandomViewport(gl.canvas, Viewport.WIDTH, Viewport.HEIGHT, deString.deStringHash(this.name) + this.m_caseNdx);
        /** @type {tcuSurface.Surface} */ var renderedFrame = new tcuSurface.Surface(viewport.width, viewport.height);
        /** @type {tcuSurface.Surface} */ var referenceFrame = new tcuSurface.Surface(viewport.width, viewport.height);
        /** @type {glsTextureTestUtil.ReferenceParams} */ var refParams = new glsTextureTestUtil.ReferenceParams(glsTextureTestUtil.textureType.TEXTURETYPE_2D);
        /** @type {tcuTexture.TextureFormat} */ var texFormat = this.m_texture.getRefTexture().getFormat();
        /** @type {Array<number>} */ var texCoord;
        /** @type {tcuTextureUtil.TextureFormatInfo} */ var texFormatInfo = tcuTextureUtil.getTextureFormatInfo(texFormat);
        /** @type {boolean} */ var useDefaultColorScaleAndBias = true;

        // Bind to unit 0.
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, this.m_texture.getGLTexture());

        // Setup filtering and wrap modes.
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, this.m_wrapT);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, this.m_minFilter);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, this.m_magFilter);

        // Parameters for reference images.
        refParams.sampler = gluTextureUtil.mapGLSamplerWrapST(this.m_wrapS, this.m_wrapT, this.m_minFilter, this.m_magFilter);
        refParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        refParams.samplerType = glsTextureTestUtil.getSamplerType(this.m_texture.getRefTexture().getFormat());
        refParams.colorScale = useDefaultColorScaleAndBias ? texFormatInfo.lookupScale : [1.0, 1.0, 1.0, 1.0];
        refParams.colorBias = useDefaultColorScaleAndBias ? texFormatInfo.lookupBias : [0, 0, 0, 0];

        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        texCoord = glsTextureTestUtil.computeQuadTexCoord2D(this.m_cases[this.m_caseNdx].bottomLeft, this.m_cases[this.m_caseNdx].topRight);
        this.m_renderer.renderQuad(0, texCoord, refParams);

        // gluPixelTransfer.readPixels(viewport.x, viewport.y, renderedFrame.getAccess());
        /** @type {number} */ var pixelSize = renderedFrame.getAccess().getFormat().getPixelSize();
        /** @type {number} */ var param = deMath.deIsPowerOfTwo32(pixelSize) ? Math.min(pixelSize, 8) : 1;

        gl.pixelStorei(gl.PACK_ALIGNMENT, param);
        /** @type {gluTextureUtil.TransferFormat} */ var format = gluTextureUtil.getTransferFormat(renderedFrame.getAccess().getFormat());

        renderedFrame.readViewport(gl, viewport);

        // const tcu::ScopedLogSection section (log, string("Test") + de::toString(m_caseNdx), string("Test ") + de::toString(m_caseNdx));
        /** @type {boolean} */ var isNearestOnly = this.m_minFilter == gl.NEAREST && this.m_magFilter == gl.NEAREST;
        /** @type {boolean} */ var isSRGB = texFormat.order == tcuTexture.ChannelOrder.sRGB || texFormat.order == tcuTexture.ChannelOrder.sRGBA;
        /** @type {tcuPixelFormat.PixelFormat} */ var pixelFormat = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        /** @type {Array<number>} */ var colorBits = deMath.max(deMath.subtract(glsTextureTestUtil.getBitsVec(pixelFormat), (isNearestOnly && !isSRGB ? [1, 1, 1, 1] : [2, 2, 2, 2])), [0, 0, 0, 0]);
        /** @type {tcuTexLookupVerifier.LodPrecision} */ var lodPrecision = new tcuTexLookupVerifier.LodPrecision(18, 5);
        /** @type {tcuTexLookupVerifier.LookupPrecision} */
        var lookupPrecision = new tcuTexLookupVerifier.LookupPrecision(
            [20, 20, 0],
            [5, 5, 0],
            deMath.divide(tcuTexLookupVerifier.computeFixedPointThreshold(colorBits), refParams.colorScale),
            glsTextureTestUtil.getCompareMask(pixelFormat)
        );

        // log << TestLog::Message << "Note: lookup coordinates: bottom-left " << m_cases[m_caseNdx].bottomLeft << ", top-right " << m_cases[m_caseNdx].topRight << TestLog::EndMessage;

        /** @type {boolean} */ var isOk = glsTextureTestUtil.verifyTexture2DResult(renderedFrame.getAccess(), this.m_texture.getRefTexture(), texCoord, refParams, lookupPrecision, lodPrecision, pixelFormat);

        if (!isOk)
            testFailedOptions('Case ' + this.m_caseNdx + ': verifyTexture2DResult is false', false);
        else
            testPassedOptions('Case ' + this.m_caseNdx + ': OK', true);

        this.m_caseNdx++;

        return this.m_caseNdx < this.m_cases.length ? tcuTestCase.IterateResult.CONTINUE : tcuTestCase.IterateResult.STOP;
    };

    /**
     * Initialize test
     */
    es3fTextureWrapTests.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        /** @type {string} */ var name;
        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {number} mode
         */
        var WrapMode = function(name, mode) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.mode = mode;
        };

        /** @type {Array<WrapMode>} */ var wrapModes = [
            new WrapMode('clamp', gl.CLAMP_TO_EDGE),
            new WrapMode('repeat', gl.REPEAT),
            new WrapMode('mirror', gl.MIRRORED_REPEAT)
        ];

        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {number} mode
         */
        var FilteringMode = function(name, mode) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.mode = mode;
        };

        /** @type {Array<FilteringMode>} */ var filteringModes = [
            new FilteringMode('nearest', gl.NEAREST),
            new FilteringMode('linear', gl.LINEAR)
        ];

        /* Begin RGBA8 Cases */
        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {number} width
         * @param {number} height
         */
        var Rgba8Size = function(name, width, height) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.width = width;
            /** @type {number} */ this.height = height;
        };

        /** @type {Array<Rgba8Size>} */ var rgba8Sizes = [
            new Rgba8Size('pot', 64, 128),
            new Rgba8Size('npot', 63, 112)
        ];

        for (var size = 0; size < rgba8Sizes.length; size++) {
            /** @type {tcuTestCase.DeqpTest} */ var rgba8Group = tcuTestCase.newTest('rgba8', '');
            testGroup.addChild(rgba8Group);
            for (var wrapS = 0; wrapS < wrapModes.length; wrapS++) {
                for (var wrapT = 0; wrapT < wrapModes.length; wrapT++) {
                    for (var filter = 0; filter < filteringModes.length; filter++) {
                        name = [
                            wrapModes[wrapS].name,
                            wrapModes[wrapT].name,
                            filteringModes[filter].name,
                            rgba8Sizes[size].name
                        ].join('_');

                        rgba8Group.addChild(es3fTextureWrapTests.textureWrapCaseFromFormat(
                            name, '',
                            gl.RGBA, gl.UNSIGNED_BYTE,
                            wrapModes[wrapS].mode,
                            wrapModes[wrapT].mode,
                            filteringModes[filter].mode, filteringModes[filter].mode,
                            rgba8Sizes[size].width, rgba8Sizes[size].height
                        ));
                    }
                }
            }
        }
        /* End RGBA8 Cases */

        /* Begin ETC-2 (and EAC) cases */
        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {tcuCompressedTexture.Format} format
         */
        var Etc2Format = function(name, format) {
            /** @type {string} */ this.name = name;
            /** @type {tcuCompressedTexture.Format} */ this.format = format;
        };

        var etc2Formats = [
            new Etc2Format('eac_r11', tcuCompressedTexture.Format.EAC_R11),
            new Etc2Format('eac_signed_r11', tcuCompressedTexture.Format.EAC_SIGNED_R11),
            new Etc2Format('eac_rg11', tcuCompressedTexture.Format.EAC_RG11),
            new Etc2Format('eac_signed_rg11', tcuCompressedTexture.Format.EAC_SIGNED_RG11),
            new Etc2Format('etc2_rgb8', tcuCompressedTexture.Format.ETC2_RGB8),
            new Etc2Format('etc2_srgb8', tcuCompressedTexture.Format.ETC2_SRGB8),
            new Etc2Format('etc2_rgb8_punchthrough_alpha1', tcuCompressedTexture.Format.ETC2_RGB8_PUNCHTHROUGH_ALPHA1),
            new Etc2Format('etc2_srgb8_punchthrough_alpha1', tcuCompressedTexture.Format.ETC2_SRGB8_PUNCHTHROUGH_ALPHA1),
            new Etc2Format('etc2_eac_rgba8', tcuCompressedTexture.Format.ETC2_EAC_RGBA8),
            new Etc2Format('etc2_eac_srgb8_alpha8', tcuCompressedTexture.Format.ETC2_EAC_SRGB8_ALPHA8)
        ];
        if (!gluTextureUtil.enableCompressedTextureETC()) {
            debug('Skipping ETC2/EAC texture format tests: no support for WEBGL_compressed_texture_etc');
            etc2Formats = [];
        }

        /**
         * @constructor
         * @struct
         * @param {string} name
         * @param {number} width
         * @param {number} height
         */
        var Etc2Size = function(name, width, height) {
            /** @type {string} */ this.name = name;
            /** @type {number} */ this.width = width;
            /** @type {number} */ this.height = height;
        };

        /** @type {Array<Etc2Size>} */ var etc2Sizes = [
            new Etc2Size('pot', 64, 128),
            new Etc2Size('npot', 123, 107)
        ];

        for (var formatNdx = 0; formatNdx < etc2Formats.length; formatNdx++) {
            for (var size = 0; size < etc2Sizes.length; size++) {
                /** @type {tcuTestCase.DeqpTest} */ var formatGroup = tcuTestCase.newTest(etc2Formats[formatNdx].name, '');
                testGroup.addChild(formatGroup);
                for (var wrapS = 0; wrapS < wrapModes.length; wrapS++) {
                    for (var wrapT = 0; wrapT < wrapModes.length; wrapT++) {
                        for (var filter = 0; filter < filteringModes.length; filter++) {
                            name = [
                                wrapModes[wrapS].name,
                                wrapModes[wrapT].name,
                                filteringModes[filter].name,
                                etc2Sizes[size].name
                            ].join('_');

                            formatGroup.addChild(new es3fTextureWrapTests.TextureWrapCase(
                                name, '',
                                etc2Formats[formatNdx].format,
                                wrapModes[wrapS].mode,
                                wrapModes[wrapT].mode,
                                filteringModes[filter].mode, filteringModes[filter].mode,
                                etc2Sizes[size].width, etc2Sizes[size].height
                            ));
                        }
                    }
                }
            }
        }
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
    es3fTextureWrapTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'texture_wrap';
        var testDescription = 'Texture Wrap Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.setRoot(tcuTestCase.newTest(testName, testDescription, null));

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fTextureWrapTests.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fTextureWrapTests.run tests', false);
            state.terminate();
        }
    };

});
