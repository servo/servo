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
goog.provide('functional.gles3.es3fTextureFilteringTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
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
goog.require('functional.gles3.es3fFboTestUtil');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {

    var es3fTextureFilteringTests = functional.gles3.es3fTextureFilteringTests;
    var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuLogImage = framework.common.tcuLogImage;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuTexLookupVerifier = framework.common.tcuTexLookupVerifier;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var deRandom = framework.delibs.debase.deRandom;
    var gluTexture = framework.opengl.gluTexture;
    var glsTextureTestUtil = modules.shared.glsTextureTestUtil;

    /** @type {WebGL2RenderingContext} */ var gl;

    es3fTextureFilteringTests.version =
        gluShaderUtil.getGLSLVersionString(gluShaderUtil.GLSLVersion.V300_ES);

    var TEX2D_VIEWPORT_WIDTH = 64;
    var TEX2D_VIEWPORT_HEIGHT = 64;
    var TEX2D_MIN_VIEWPORT_WIDTH = 64;
    var TEX2D_MIN_VIEWPORT_HEIGHT = 64;

    var TEX3D_VIEWPORT_WIDTH = 64;
    var TEX3D_VIEWPORT_HEIGHT = 64;
    var TEX3D_MIN_VIEWPORT_WIDTH = 64;
    var TEX3D_MIN_VIEWPORT_HEIGHT = 64;

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fTextureFilteringTests.TextureFilteringTests = function() {
        tcuTestCase.DeqpTest.call(this, 'filtering', 'Texture Filtering Tests');
    };

    es3fTextureFilteringTests.TextureFilteringTests.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);

    es3fTextureFilteringTests.TextureFilteringTests.prototype.constructor =
        es3fTextureFilteringTests.TextureFilteringTests;

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     */
    es3fTextureFilteringTests.Texture2DFilteringCase = function(
        name, desc, minFilter, magFilter, wrapS, wrapT,
        internalFormat, width, height
    ) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_minFilter = minFilter;
        this.m_magFilter = magFilter;
        this.m_wrapS = wrapS;
        this.m_wrapT = wrapT;
        this.m_internalFormat = internalFormat;
        this.m_width = width;
        this.m_height = height;
        /** @type {glsTextureTestUtil.TextureRenderer} */
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(
            es3fTextureFilteringTests.version,
            gluShaderUtil.precision.PRECISION_HIGHP
        );
        this.m_caseNdx = 0;
        /** @type {Array<gluTexture.Texture2D>} */ this.m_textures = [];
        this.m_cases = [];
    };

    es3fTextureFilteringTests.Texture2DFilteringCase.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);
    es3fTextureFilteringTests.Texture2DFilteringCase.prototype.constructor =
        es3fTextureFilteringTests.Texture2DFilteringCase;

    /**
     * @constructor
     * @param {gluTexture.Texture2D} tex_
     * @param {Array<number>} minCoord_
     * @param {Array<number>} maxCoord_
     */
    es3fTextureFilteringTests.Texture2DFilteringCase.FilterCase = function(
        tex_, minCoord_, maxCoord_
    ) {
        this.texture = tex_;
        this.minCoord = minCoord_;
        this.maxCoord = maxCoord_;
    };

    /** @typedef {{texNdx: number, lodX: number,
     * lodY: number, oX: number, oY: number}} */
    es3fTextureFilteringTests.Cases;

    /**
     * init
     */
    es3fTextureFilteringTests.Texture2DFilteringCase.prototype.init =
    function() {
        try {
            // Create 2 textures.
            for (var ndx = 0; ndx < 2; ndx++)
                this.m_textures.push(
                    gluTexture.texture2DFromInternalFormat(
                        gl, this.m_internalFormat,
                        this.m_width, this.m_height
                    )
                );

            var mipmaps = true;
            var numLevels = mipmaps ? deMath.logToFloor(
                Math.max(this.m_width, this.m_height)
            ) + 1 : 1;

            /** @type {tcuTextureUtil.TextureFormatInfo} */
            var fmtInfo = tcuTextureUtil.getTextureFormatInfo(
                this.m_textures[0].getRefTexture().getFormat()
            );
            /** @type {Array<number>} */ var cBias = fmtInfo.valueMin;
            /** @type {Array<number>} */
            var cScale = deMath.subtract(
                fmtInfo.valueMax, fmtInfo.valueMin
            );

            // Fill first gradient texture.
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                /** @type {Array<number>} */ var gMin = deMath.add(
                    deMath.multiply([0.0, 0.0, 0.0, 1.0], cScale), cBias
                );
                /** @type {Array<number>} */ var gMax = deMath.add(
                    deMath.multiply([1.0, 1.0, 1.0, 0.0], cScale), cBias
                );

                this.m_textures[0].getRefTexture().allocLevel(levelNdx);
                tcuTextureUtil.fillWithComponentGradients(
                    this.m_textures[0].getRefTexture().getLevel(levelNdx),
                    gMin, gMax
                );
            }

            // Fill second with grid texture.
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                /** @type {number} */ var step = 0x00ffffff / numLevels;
                /** @type {number} */ var rgb = step * levelNdx;
                /** @type {number} */ var colorA = deMath.binaryOp(
                    0xff000000, rgb, deMath.BinaryOp.OR
                );
                /** @type {number} */ var colorB = deMath.binaryOp(
                    0xff000000, deMath.binaryNot(rgb), deMath.BinaryOp.OR
                );

                this.m_textures[1].getRefTexture().allocLevel(levelNdx);
                tcuTextureUtil.fillWithGrid(
                    this.m_textures[1].getRefTexture().getLevel(levelNdx),
                    4,
                    deMath.add(deMath.multiply(
                        tcuRGBA.newRGBAFromValue(colorA).toVec(), cScale),
                        cBias
                    ),
                    deMath.add(deMath.multiply(
                        tcuRGBA.newRGBAFromValue(colorB).toVec(), cScale),
                        cBias
                    )
                );
            }

            // Upload.
            for (var i = 0; i < this.m_textures.length; i++)
                this.m_textures[i].upload();

            // Compute cases.

            /** @type {Array<es3fTextureFilteringTests.Cases>} */
            var cases = [{
                    texNdx: 0, lodX: 1.6, lodY: 2.9, oX: -1.0, oY: -2.7
                }, {
                    texNdx: 0, lodX: -2.0, lodY: -1.35, oX: -0.2, oY: 0.7
                }, {
                    texNdx: 1, lodX: 0.14, lodY: 0.275, oX: -1.5, oY: -1.1
                }, {
                    texNdx: 1, lodX: -0.92, lodY: -2.64, oX: 0.4, oY: -0.1
                }
            ];

            var viewportW = Math.min(
                TEX2D_VIEWPORT_WIDTH, gl.canvas.width
            );
            var viewportH = Math.min(
                TEX2D_VIEWPORT_HEIGHT, gl.canvas.height
            );

            for (var caseNdx = 0; caseNdx < cases.length; caseNdx++) {
                /** @type {number} */ var texNdx = deMath.clamp(
                    cases[caseNdx].texNdx, 0, this.m_textures.length - 1
                );
                /** @type {number} */ var lodX = cases[caseNdx].lodX;
                /** @type {number} */ var lodY = cases[caseNdx].lodY;
                /** @type {number} */ var oX = cases[caseNdx].oX;
                /** @type {number} */ var oY = cases[caseNdx].oY;
                /** @type {number} */ var sX = Math.exp(lodX * Math.log(2)) * viewportW /
                    this.m_textures[texNdx].getRefTexture().getWidth();
                /** @type {number} */ var sY = Math.exp(lodY * Math.log(2)) * viewportH /
                    this.m_textures[texNdx].getRefTexture().getHeight();

                this.m_cases.push(
                    new
                    es3fTextureFilteringTests.Texture2DFilteringCase.FilterCase(
                        this.m_textures[texNdx], [oX, oY], [oX + sX, oY + sY]
                    )
                );
            }

            this.m_caseNdx = 0;
        }
        catch (e) {
            // Clean up to save memory.
            this.deinit();
            throw e;
        }
    };

    /**
     * deinit
     */
    es3fTextureFilteringTests.Texture2DFilteringCase.prototype.deinit =
    function() {
        while (this.m_textures.length > 0) {
            gl.deleteTexture(this.m_textures[0].getGLTexture());
            this.m_textures.splice(0, 1);
        }
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureFilteringTests.Texture2DFilteringCase.prototype.iterate =
    function() {
        /** @type {glsTextureTestUtil.RandomViewport} */
        var viewport = new glsTextureTestUtil.RandomViewport(
            gl.canvas, TEX2D_VIEWPORT_WIDTH,
            TEX2D_VIEWPORT_HEIGHT, deMath.binaryOp(
                deString.deStringHash(this.fullName()),
                deMath.deMathHash(this.m_caseNdx),
                deMath.BinaryOp.XOR
            )
        );
        /** @type {tcuTexture.TextureFormat} */
        var texFmt = this.m_textures[0].getRefTexture().getFormat();

        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        var curCase = this.m_cases[this.m_caseNdx];
        bufferedLogToConsole('Test ' + this.m_caseNdx);
        var refParams = new glsTextureTestUtil.ReferenceParams(
            glsTextureTestUtil.textureType.TEXTURETYPE_2D
        );
        var rendered = new tcuSurface.Surface(viewport.width, viewport.height);
        var texCoord = [0, 0];

        if (viewport.width < TEX2D_MIN_VIEWPORT_WIDTH ||
            viewport.height < TEX2D_MIN_VIEWPORT_HEIGHT)
            throw new Error('Too small render target');

        // Setup params for reference.
        refParams.sampler = gluTextureUtil.mapGLSamplerWrapST(
            this.m_wrapS, this.m_wrapT, this.m_minFilter, this.m_magFilter
        );
        refParams.samplerType = glsTextureTestUtil.getSamplerType(texFmt);
        refParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        refParams.colorBias = fmtInfo.lookupBias;
        refParams.colorScale = fmtInfo.lookupScale;

        // Compute texture coordinates.
        bufferedLogToConsole(
            'Texture coordinates: ' + curCase.minCoord +
            ' -> ' + curCase.maxCoord
        );
        texCoord = glsTextureTestUtil.computeQuadTexCoord2D(
            curCase.minCoord, curCase.maxCoord
        );

        gl.bindTexture(gl.TEXTURE_2D, curCase.texture.getGLTexture());
        gl.texParameteri(
            gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, this.m_minFilter
        );
        gl.texParameteri(
            gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, this.m_magFilter
        );
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, this.m_wrapT);

        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

        this.m_renderer.renderQuad(0, texCoord, refParams);
        rendered.readViewport(
            gl, [viewport.x, viewport.y, viewport.width, viewport.height]
        );

        /** @type {boolean} */ var isNearestOnly =
            this.m_minFilter == gl.NEAREST && this.m_magFilter == gl.NEAREST;
        /** @type {tcuPixelFormat.PixelFormat} */
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);

        //(iVec4)
        var colorBits = deMath.max(
            deMath.addScalar(
                glsTextureTestUtil.getBitsVec(pixelFormat),
                // 1 inaccurate bit if nearest only, 2 otherwise
                -1 * (isNearestOnly ? 1 : 2)
            ),
            [0, 0, 0, 0]
        );

        /** @type {tcuTexLookupVerifier.LodPrecision} */
        var lodPrecision = new tcuTexLookupVerifier.LodPrecision();
        /** @type {tcuTexLookupVerifier.LookupPrecision} */
        var lookupPrecision = new tcuTexLookupVerifier.LookupPrecision();

        lodPrecision.derivateBits = 18;
        lodPrecision.lodBits = 6;
        lookupPrecision.colorThreshold = deMath.divide(
            tcuTexLookupVerifier.computeFixedPointThreshold(colorBits),
            refParams.colorScale
        );
        lookupPrecision.coordBits = [20, 20, 0];
        lookupPrecision.uvwBits = [7, 7, 0];
        lookupPrecision.colorMask =
            glsTextureTestUtil.getCompareMask(pixelFormat);

        var isHighQuality = glsTextureTestUtil.verifyTexture2DResult(
            rendered.getAccess(), curCase.texture.getRefTexture(),
            texCoord, refParams, lookupPrecision, lodPrecision, pixelFormat
        );

        if (!isHighQuality) {
            // Evaluate against lower precision requirements.
            lodPrecision.lodBits = 4;
            lookupPrecision.uvwBits = [4, 4, 0];

            bufferedLogToConsole('Warning: Verification against high ' +
                'precision requirements failed, trying with lower ' +
                'requirements.'
            );

            var isOk = glsTextureTestUtil.verifyTexture2DResult(
                rendered.getAccess(), curCase.texture.getRefTexture(),
                texCoord, refParams, lookupPrecision, lodPrecision,
                pixelFormat
            );

            if (!isOk) {
                bufferedLogToConsole(
                    'ERROR: Verification against low ' +
                    'precision requirements failed, failing test case.'
                );
                testFailedOptions('Image verification failed', false);
                //In JS version, one mistake and you're out
                return tcuTestCase.IterateResult.STOP;
            } else
                checkMessage(
                    false,
                    'Low-quality filtering result in iteration no. ' +
                    this.m_caseNdx
                );
        }

        this.m_caseNdx += 1;
        if (this.m_caseNdx < this.m_cases.length)
            return tcuTestCase.IterateResult.CONTINUE;

        testPassed('Verified');
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {boolean} onlySampleFaceInterior
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     */
    es3fTextureFilteringTests.TextureCubeFilteringCase = function(
        name, desc, minFilter, magFilter, wrapS, wrapT, onlySampleFaceInterior,
        internalFormat, width, height
    ) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_minFilter = minFilter;
        this.m_magFilter = magFilter;
        this.m_wrapS = wrapS;
        this.m_wrapT = wrapT;
        /** @type {boolean}*/
        this.m_onlySampleFaceInterior = onlySampleFaceInterior;
        this.m_internalFormat = internalFormat;
        this.m_width = width;
        this.m_height = height;
        /** @type {glsTextureTestUtil.TextureRenderer} */
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(
            es3fTextureFilteringTests.version,
            gluShaderUtil.precision.PRECISION_HIGHP
        );
        this.m_caseNdx = 0;
        /** @type {Array<gluTexture.TextureCube>} */ this.m_textures = [];
        /** @type {Array<es3fTextureFilteringTests.
         *      TextureCubeFilteringCase.FilterCase>}
         */
        this.m_cases = [];
    };

    /**
     * @constructor
     * @param {gluTexture.TextureCube} tex_
     * @param {Array<number>} bottomLeft_
     * @param {Array<number>} topRight_
     */
    es3fTextureFilteringTests.TextureCubeFilteringCase.FilterCase = function(
        tex_, bottomLeft_, topRight_
    ) {
        this.texture = tex_;
        this.bottomLeft = bottomLeft_;
        this.topRight = topRight_;
    };

    es3fTextureFilteringTests.TextureCubeFilteringCase.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);

    es3fTextureFilteringTests.TextureCubeFilteringCase.prototype.constructor =
        es3fTextureFilteringTests.TextureCubeFilteringCase;

    /**
     * init
     */
    es3fTextureFilteringTests.TextureCubeFilteringCase.prototype.init =
    function() {
        try {
            assertMsgOptions(
                this.m_width == this.m_height, 'Texture has to be squared',
                false, true
            );
            for (var ndx = 0; ndx < 2; ndx++)
                this.m_textures.push(gluTexture.cubeFromInternalFormat(
                    gl, this.m_internalFormat, this.m_width
                ));

            var numLevels = deMath.logToFloor(
                Math.max(this.m_width, this.m_height)
            ) + 1;
            /** @type {tcuTextureUtil.TextureFormatInfo} */
            var fmtInfo = tcuTextureUtil.getTextureFormatInfo(
                this.m_textures[0].getRefTexture().getFormat()
            );
            /** @type {Array<number>} */
            var cBias = fmtInfo.valueMin;
            /** @type {Array<number>} */
            var cScale = deMath.subtract(
                fmtInfo.valueMax, fmtInfo.valueMin
            );

            // Fill first with gradient texture.
            /** @type {Array<Array<Array<number>>>}
             * (array of 4 component vectors)
             */
            var gradients = [
                [ // negative x
                    [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 0.0]
                ], [ // positive x
                    [0.5, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 0.0]
                ], [ // negative y
                    [0.0, 0.5, 0.0, 1.0], [1.0, 1.0, 1.0, 0.0]
                ], [ // positive y
                    [0.0, 0.0, 0.5, 1.0], [1.0, 1.0, 1.0, 0.0]
                ], [ // negative z
                    [0.0, 0.0, 0.0, 0.5], [1.0, 1.0, 1.0, 1.0]
                ], [ // positive z
                    [0.5, 0.5, 0.5, 1.0], [1.0, 1.0, 1.0, 0.0]
                ]
            ];
            for (var face = 0;
                face < Object.keys(tcuTexture.CubeFace).length;
                face++) {
                for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                    this.m_textures[0].getRefTexture().allocLevel(
                        face, levelNdx
                    );
                    tcuTextureUtil.fillWithComponentGradients(
                        this.m_textures[0].getRefTexture().getLevelFace(
                            levelNdx, face
                        ),
                        deMath.add(deMath.multiply(
                            gradients[face][0], cScale
                        ), cBias),
                        deMath.add(deMath.multiply(
                            gradients[face][1], cScale
                        ), cBias)
                    );
                }
            }

            // Fill second with grid texture.
            for (var face = 0;
                face < Object.keys(tcuTexture.CubeFace).length;
                face++) {
                for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                    var step = 0x00ffffff / (
                        numLevels * Object.keys(tcuTexture.CubeFace).length
                    );
                    var rgb = step * levelNdx * face;
                    /** @type {number} */ var colorA = deMath.binaryOp(
                        0xff000000, rgb, deMath.BinaryOp.OR
                    );
                    /** @type {number} */ var colorB = deMath.binaryOp(
                        0xff000000, deMath.binaryNot(rgb),
                        deMath.BinaryOp.OR
                    );

                    this.m_textures[1].getRefTexture().allocLevel(
                        face, levelNdx
                    );
                    tcuTextureUtil.fillWithGrid(
                        this.m_textures[1].getRefTexture().getLevelFace(
                            levelNdx, face
                        ), 4, deMath.add(
                            deMath.multiply(
                                tcuRGBA.newRGBAFromValue(colorA).toVec(),
                                cScale
                            ), cBias
                        ), deMath.add(
                            deMath.multiply(
                                tcuRGBA.newRGBAFromValue(colorB).toVec(),
                                cScale
                            ), cBias
                        )
                    );
                }
            }

            // Upload.
            for (var i = 0; i < this.m_textures.length; i++)
                this.m_textures[i].upload();

            // Compute cases
            /** @type {gluTexture.TextureCube} */
            var tex0 = this.m_textures[0];
            /** @type {gluTexture.TextureCube} */
            var tex1 = this.m_textures.length > 1 ? this.m_textures[1] : tex0;

            if (this.m_onlySampleFaceInterior) {
                // minification
                this.m_cases.push(new es3fTextureFilteringTests.
                    TextureCubeFilteringCase.FilterCase(
                    tex0, [-0.8, -0.8], [0.8, 0.8]
                ));
                // magnification
                this.m_cases.push(new es3fTextureFilteringTests.
                    TextureCubeFilteringCase.FilterCase(
                    tex0, [0.5, 0.65], [0.8, 0.8]
                ));
                // minification
                this.m_cases.push(new es3fTextureFilteringTests.
                    TextureCubeFilteringCase.FilterCase(
                    tex1, [-0.8, -0.8], [0.8, 0.8]
                ));
                // magnification
                this.m_cases.push(new es3fTextureFilteringTests.
                    TextureCubeFilteringCase.FilterCase(
                    tex1, [0.2, 0.2], [0.6, 0.5]
                ));
            } else {
                // minification
                if (gl.getParameter(gl.SAMPLES) == 0)
                    this.m_cases.push(
                        new es3fTextureFilteringTests.TextureCubeFilteringCase.
                        FilterCase(
                            tex0, [-1.25, -1.2], [1.2, 1.25]
                        )
                    );
                // minification - w/ tweak to avoid hitting triangle
                // edges with face switchpoint.
                else
                    this.m_cases.push(
                        new es3fTextureFilteringTests.TextureCubeFilteringCase.
                        FilterCase(
                            tex0, [-1.19, -1.3], [1.1, 1.35]
                        )
                    );

                // magnification
                this.m_cases.push(
                    new es3fTextureFilteringTests.TextureCubeFilteringCase.
                    FilterCase(
                        tex0, [0.8, 0.8], [1.25, 1.20]
                    )
                );
                // minification
                this.m_cases.push(
                    new es3fTextureFilteringTests.TextureCubeFilteringCase.
                    FilterCase(
                        tex1, [-1.19, -1.3], [1.1, 1.35]
                    )
                );
                // magnification
                this.m_cases.push(
                    new es3fTextureFilteringTests.TextureCubeFilteringCase.
                    FilterCase(
                        tex1, [-1.2, -1.1], [-0.8, -0.8]
                    )
                );
            }

            this.m_caseNdx = 0;
        }
        catch (e) {
            // Clean up to save memory.
            this.deinit();
            throw e;
        }
    };

    /**
     * deinit
     */
    es3fTextureFilteringTests.TextureCubeFilteringCase.prototype.deinit =
    function() {
        while (this.m_textures.length > 0) {
            gl.deleteTexture(this.m_textures[0].getGLTexture());
            this.m_textures.splice(0, 1);
        }
    };

    /**
     * @param {tcuTexture.CubeFace} face
     * @return {string}
     */
    es3fTextureFilteringTests.getFaceDesc = function(face) {
        switch (face) {
            case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X: return '-X';
            case tcuTexture.CubeFace.CUBEFACE_POSITIVE_X: return '+X';
            case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y: return '-Y';
            case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y: return '+Y';
            case tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z: return '-Z';
            case tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z: return '+Z';
            default:
                throw new Error('Invalid cube face specified');
        }
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureFilteringTests.TextureCubeFilteringCase.prototype.iterate =
    function() {
        var viewportSize = 28;
        /** @type {glsTextureTestUtil.RandomViewport} */
        var viewport = new glsTextureTestUtil.RandomViewport(
            gl.canvas, viewportSize,
            viewportSize, deMath.binaryOp(
                deString.deStringHash(this.fullName()),
                deMath.deMathHash(this.m_caseNdx),
                deMath.BinaryOp.XOR
            )
        );
        bufferedLogToConsole('Test' + this.m_caseNdx);
        /** @type {es3fTextureFilteringTests.
         *      TextureCubeFilteringCase.FilterCase}
         */
        var curCase = this.m_cases[this.m_caseNdx];
        /** @type {tcuTexture.TextureFormat} */
        var texFmt = curCase.texture.getRefTexture().getFormat();
        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
        /** @type {glsTextureTestUtil.ReferenceParams} */
        var sampleParams = new glsTextureTestUtil.ReferenceParams(
            glsTextureTestUtil.textureType.TEXTURETYPE_CUBE
        );

        if (viewport.width < viewportSize || viewport.height < viewportSize)
            throw new Error('Too small render target');

        // Setup texture
        gl.bindTexture(gl.TEXTURE_CUBE_MAP, curCase.texture.getGLTexture());
        gl.texParameteri(
            gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, this.m_minFilter
        );
        gl.texParameteri(
            gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, this.m_magFilter
        );
        gl.texParameteri(
            gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, this.m_wrapS
        );
        gl.texParameteri(
            gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, this.m_wrapT
        );

        // Other state
        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

        // Params for reference computation.
        sampleParams.sampler = gluTextureUtil.mapGLSamplerWrapST(
            gl.CLAMP_TO_EDGE, gl.CLAMP_TO_EDGE,
            this.m_minFilter, this.m_magFilter
        );
        sampleParams.sampler.seamlessCubeMap = true;
        sampleParams.samplerType = glsTextureTestUtil.getSamplerType(texFmt);
        sampleParams.colorBias = fmtInfo.lookupBias;
        sampleParams.colorScale = fmtInfo.lookupScale;
        sampleParams.lodMode = glsTextureTestUtil.lodMode.EXACT;

        bufferedLogToConsole(
            'Coordinates: ' + curCase.bottomLeft + ' -> ' + curCase.topRight
        );

        for (var faceNdx = 0;
            faceNdx < Object.keys(tcuTexture.CubeFace).length;
            faceNdx++) {
            var face = /** @type {tcuTexture.CubeFace} */ (faceNdx);
            /** @type {tcuSurface.Surface} */
            var result = new tcuSurface.Surface(
                viewport.width, viewport.height
            );
            /** @type {Array<number>} */ var texCoord;

            texCoord = glsTextureTestUtil.computeQuadTexCoordCubeFace(
                face, curCase.bottomLeft, curCase.topRight
            );

            bufferedLogToConsole(
                'Face ' + es3fTextureFilteringTests.getFaceDesc(face)
            );

            // \todo Log texture coordinates.

            this.m_renderer.renderQuad(0, texCoord, sampleParams);

            result.readViewport(
                gl, [viewport.x, viewport.y, viewport.width, viewport.height]
            );

            /** @type {boolean} */
            var isNearestOnly = this.m_minFilter == gl.NEAREST &&
                this.m_magFilter == gl.NEAREST;
            /** @type {tcuPixelFormat.PixelFormat} */
            var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);

            //(iVec4)
            var colorBits = deMath.max(
                deMath.addScalar(
                    glsTextureTestUtil.getBitsVec(pixelFormat),
                    // 1 inaccurate bit if nearest only, 2 otherwise
                    -1 * (isNearestOnly ? 1 : 2)
                ),
                [0, 0, 0, 0]
            );
            /** @type {tcuTexLookupVerifier.LodPrecision} */
            var lodPrecision = new tcuTexLookupVerifier.LodPrecision();
            /** @type {tcuTexLookupVerifier.LookupPrecision} */
            var lookupPrecision = new tcuTexLookupVerifier.LookupPrecision();

            lodPrecision.derivateBits = 10;
            lodPrecision.lodBits = 5;
            lookupPrecision.colorThreshold = deMath.divide(
                tcuTexLookupVerifier.computeFixedPointThreshold(colorBits),
                sampleParams.colorScale
            );
            lookupPrecision.coordBits = [10, 10, 10];
            lookupPrecision.uvwBits = [6, 6, 0];
            lookupPrecision.colorMask =
                glsTextureTestUtil.getCompareMask(pixelFormat);

            var isHighQuality = glsTextureTestUtil.verifyTextureCubeResult(
                result.getAccess(), curCase.texture.getRefTexture(),
                texCoord, sampleParams, lookupPrecision, lodPrecision,
                pixelFormat
            );


            if (!isHighQuality) {
                // Evaluate against lower precision requirements.
                lodPrecision.lodBits = 2;
                lookupPrecision.uvwBits = [3, 3, 0];

                bufferedLogToConsole('Warning: Verification against high ' +
                 'precision requirements failed, trying with lower ' +
                 'requirements.');

                var isOk = glsTextureTestUtil.verifyTextureCubeResult(
                    result.getAccess(), curCase.texture.getRefTexture(),
                    texCoord, sampleParams, lookupPrecision, lodPrecision,
                    pixelFormat
                );

                if (!isOk) {
                    bufferedLogToConsole('ERROR: Verification against low' +
                        'precision requirements failed, failing test case.');
                    testFailedOptions('Image verification failed', false);
                    //In JS version, one mistake and you're out
                    return tcuTestCase.IterateResult.STOP;
                } else
                    checkMessage(
                        false,
                        'Low-quality filtering result in iteration no. ' +
                        this.m_caseNdx
                    );
            }
        }

        this.m_caseNdx += 1;
        if (this.m_caseNdx < this.m_cases.length)
            return tcuTestCase.IterateResult.CONTINUE;

        testPassed('Verified');
        return tcuTestCase.IterateResult.STOP;
    };

    // 2D array filtering

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} numLayers
     */
    es3fTextureFilteringTests.Texture2DArrayFilteringCase = function(
        name, desc, minFilter, magFilter, wrapS, wrapT,
        internalFormat, width, height, numLayers
    ) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_minFilter = minFilter;
        this.m_magFilter = magFilter;
        this.m_wrapS = wrapS;
        this.m_wrapT = wrapT;
        this.m_internalFormat = internalFormat;
        this.m_width = width;
        this.m_height = height;
        this.m_numLayers = numLayers;
        this.m_gradientTex = null;
        this.m_gridTex = null;
        /** @type {glsTextureTestUtil.TextureRenderer} */
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(
            es3fTextureFilteringTests.version,
            gluShaderUtil.precision.PRECISION_HIGHP
        );
        this.m_textures = [];
        this.m_caseNdx = 0;
        this.m_cases = [];
    };

    es3fTextureFilteringTests.Texture2DArrayFilteringCase.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);

    es3fTextureFilteringTests.Texture2DArrayFilteringCase.prototype.
    constructor = es3fTextureFilteringTests.Texture2DArrayFilteringCase;

    /**
     * @constructor
     * @param {gluTexture.Texture2DArray} tex_
     * @param {Array<number>} lod_
     * @param {Array<number>} offset_
     * @param {Array<number>} layerRange_
     */
    es3fTextureFilteringTests.Texture2DArrayFilteringCase.FilterCase =
    function(
        tex_, lod_, offset_, layerRange_
    ) {
        this.texture = tex_;
        this.lod = lod_;
        this.offset = offset_;
        this.layerRange = layerRange_;
    };

    /*
     * init
     */
    es3fTextureFilteringTests.Texture2DArrayFilteringCase.prototype.init =
    function() {
        try {
            /** @type {tcuTexture.TextureFormat} */
            var texFmt = gluTextureUtil.mapGLInternalFormat(
                this.m_internalFormat
            );
            /** @type {tcuTextureUtil.TextureFormatInfo} */
            var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
            var cScale = deMath.subtract(
                fmtInfo.valueMax, fmtInfo.valueMin
            );
            var cBias = fmtInfo.valueMin;
            var numLevels = deMath.logToFloor(
                Math.max(this.m_width, this.m_height)
            ) + 1;

            // Create textures.
            this.m_gradientTex = gluTexture.texture2DArrayFromInternalFormat(
                gl,
                this.m_internalFormat, this.m_width,
                this.m_height, this.m_numLayers
            );

            this.m_gridTex = gluTexture.texture2DArrayFromInternalFormat(
                gl,
                this.m_internalFormat, this.m_width,
                this.m_height, this.m_numLayers
            );

            var levelSwz = [
                [0, 1, 2, 3],
                [2, 1, 3, 0],
                [3, 0, 1, 2],
                [1, 3, 2, 0]
            ];

            // Fill first gradient texture
            // (gradient direction varies between layers).
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                this.m_gradientTex.getRefTexture().allocLevel(levelNdx);

                var levelBuf =
                    this.m_gradientTex.getRefTexture().getLevel(levelNdx);

                for (var layerNdx = 0;
                    layerNdx < this.m_numLayers;
                    layerNdx++) {
                    var swz = levelSwz[layerNdx % levelSwz.length];
                    var gMin = deMath.add(deMath.multiply(deMath.swizzle(
                        [0.0, 0.0, 0.0, 1.0], [swz[0], swz[1], swz[2], swz[3]]
                    ), cScale), cBias);
                    var gMax = deMath.add(deMath.multiply(deMath.swizzle(
                        [1.0, 1.0, 1.0, 0.0], [swz[0], swz[1], swz[2], swz[3]]
                    ), cScale), cBias);

                    tcuTextureUtil.fillWithComponentGradients2D(
                        tcuTextureUtil.getSubregion(
                            levelBuf, 0, 0, layerNdx, levelBuf.getWidth(),
                            levelBuf.getHeight(), 1
                        ), gMin, gMax
                    );
                }
            }

            // Fill second with grid texture (each layer has unique colors).
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                this.m_gridTex.getRefTexture().allocLevel(levelNdx);

                /** @type {tcuTexture.PixelBufferAccess} */ var levelBuf =
                    this.m_gridTex.getRefTexture().getLevel(levelNdx);

                for (
                    var layerNdx = 0;
                    layerNdx < this.m_numLayers;
                    layerNdx++) {
                    var step = 0x00ffffff / (numLevels * this.m_numLayers - 1);
                    var rgb = step * (levelNdx + layerNdx * numLevels);
                    /** @type {number} */ var colorA = deMath.binaryOp(
                        0xff000000, rgb, deMath.BinaryOp.OR
                    );
                    /** @type {number} */ var colorB = deMath.binaryOp(
                        0xff000000, deMath.binaryNot(rgb), deMath.BinaryOp.OR
                    );

                    tcuTextureUtil.fillWithGrid(
                        tcuTextureUtil.getSubregion(
                            levelBuf, 0, 0, layerNdx, levelBuf.getWidth(),
                            levelBuf.getHeight(), 1
                        ), 4,
                        deMath.add(
                            deMath.multiply(
                                tcuRGBA.newRGBAFromValue(colorA).toVec(),
                                cScale
                            ), cBias
                        ),
                        deMath.add(
                            deMath.multiply(
                                tcuRGBA.newRGBAFromValue(colorB).toVec(),
                                cScale
                            ), cBias
                        )
                    );
                }
            }

            // Upload.
            this.m_gradientTex.upload();
            this.m_gridTex.upload();

            // Test cases
            this.m_cases.push(
                new es3fTextureFilteringTests.
                Texture2DArrayFilteringCase.FilterCase(
                    this.m_gradientTex, [1.5, 2.8], [-1.0, -2.7],
                    [-0.5, this.m_numLayers + 0.5]
                )
            );
            this.m_cases.push(
                new es3fTextureFilteringTests.
                Texture2DArrayFilteringCase.FilterCase(
                    this.m_gridTex, [0.2, 0.175], [-2.0, -3.7],
                    [-0.5, this.m_numLayers + 0.5]
                )
            );
            this.m_cases.push(
                new es3fTextureFilteringTests.
                Texture2DArrayFilteringCase.FilterCase(
                    this.m_gridTex, [-0.8, -2.3], [0.2, -0.1],
                    [this.m_numLayers + 0.5, -0.5]
                )
            );

            // Level rounding - only in single-sample configs as
            // multisample configs may produce smooth transition at the middle.
            if (gl.getParameter(gl.SAMPLES) == 0)
                this.m_cases.push(
                    new es3fTextureFilteringTests.
                    Texture2DArrayFilteringCase.FilterCase(
                        this.m_gradientTex, [-2.0, -1.5], [-0.1, 0.9],
                        [1.50001, 1.49999]
                    )
                );

            this.m_caseNdx = 0;
        }
        catch (e) {
            // Clean up to save memory.
            this.deinit();
            throw e;
        }
    };

    /**
     * deinit
     */
    es3fTextureFilteringTests.Texture2DArrayFilteringCase.prototype.deinit =
    function() {
        if (this.m_gradientTex)
            gl.deleteTexture(this.m_gradientTex.getGLTexture());
        if (this.m_gridTex)
            gl.deleteTexture(this.m_gridTex.getGLTexture());

        this.m_gradientTex = null;
        this.m_gridTex = null;
    };

    /**
     * iterate
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureFilteringTests.Texture2DArrayFilteringCase.prototype.iterate =
    function() {
        /** @type {glsTextureTestUtil.RandomViewport} */
        var viewport = new glsTextureTestUtil.RandomViewport(
            gl.canvas, TEX3D_VIEWPORT_WIDTH,
            TEX3D_VIEWPORT_HEIGHT, deMath.binaryOp(
                deString.deStringHash(this.fullName()),
                deMath.deMathHash(this.m_caseNdx),
                deMath.BinaryOp.XOR
            )
        );

        /** @type {es3fTextureFilteringTests.Texture2DArrayFilteringCase.
         * FilterCase} */ var curCase = this.m_cases[this.m_caseNdx];

        /** @type {tcuTexture.TextureFormat} */
        var texFmt = curCase.texture.getRefTexture().getFormat();
        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);

        bufferedLogToConsole('Test' + this.m_caseNdx);

        /** @type {glsTextureTestUtil.ReferenceParams} */
        var refParams = new glsTextureTestUtil.ReferenceParams(
            glsTextureTestUtil.textureType.TEXTURETYPE_2D_ARRAY
        );

        /** @type {tcuSurface.Surface} */
        var rendered = new tcuSurface.Surface(viewport.width, viewport.height);

        if (viewport.width < TEX3D_MIN_VIEWPORT_WIDTH ||
            viewport.height < TEX3D_MIN_VIEWPORT_HEIGHT)
            throw new Error('Too small render target');

        // Setup params for reference.
        refParams.sampler = gluTextureUtil.mapGLSampler(
            this.m_wrapS, this.m_wrapT, this.m_wrapT,
            this.m_minFilter, this.m_magFilter
        );
        refParams.samplerType = glsTextureTestUtil.getSamplerType(texFmt);
        refParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        refParams.colorBias = fmtInfo.lookupBias;
        refParams.colorScale = fmtInfo.lookupScale;

        // Compute texture coordinates.
        bufferedLogToConsole(
            'Approximate lod per axis = ' + curCase.lod +
            ', offset = ' + curCase.offset
        );

        /** @type {number} */ var lodX = curCase.lod[0];
        /** @type {number} */ var lodY = curCase.lod[1];
        /** @type {number} */ var oX = curCase.offset[0];
        /** @type {number} */ var oY = curCase.offset[1];
        /** @type {number} */ var sX = Math.pow(2, lodX) * viewport.width /
            this.m_gradientTex.getRefTexture().getWidth();
        /** @type {number} */ var sY = Math.pow(2, lodY) * viewport.height /
            this.m_gradientTex.getRefTexture().getHeight();
        /** @type {number} */ var l0 = curCase.layerRange[0];
        /** @type {number} */ var l1 = curCase.layerRange[1];

        /** @type {Array<number>}*/
        var texCoord = [
            oX, oY, l0,
            oX, oY + sY, l0 * 0.5 + l1 * 0.5,
            oX + sX, oY, l0 * 0.5 + l1 * 0.5,
            oX + sX, oY + sY, l1
            ];

        gl.bindTexture(gl.TEXTURE_2D_ARRAY, curCase.texture.getGLTexture());
        gl.texParameteri(
            gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, this.m_minFilter
        );
        gl.texParameteri(
            gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, this.m_magFilter
        );
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, this.m_wrapT);

        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        this.m_renderer.renderQuad(
            0, texCoord,
            refParams
        );
        rendered.readViewport(
            gl, [viewport.x, viewport.y, viewport.width, viewport.height]
        );

        /** @type {boolean} */
        var isNearestOnly = this.m_minFilter == gl.NEAREST &&
            this.m_magFilter == gl.NEAREST;
        /** @type {tcuPixelFormat.PixelFormat} */
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
        //(iVec4)
        var colorBits = deMath.max(
            deMath.addScalar(
                glsTextureTestUtil.getBitsVec(pixelFormat),
                // 1 inaccurate bit if nearest only, 2 otherwise
                -1 * (isNearestOnly ? 1 : 2)
            ),
            [0, 0, 0, 0]
        );
        /** @type {tcuTexLookupVerifier.LodPrecision} */
        var lodPrecision = new tcuTexLookupVerifier.LodPrecision();
        /** @type {tcuTexLookupVerifier.LookupPrecision} */
        var lookupPrecision = new tcuTexLookupVerifier.LookupPrecision();

        lodPrecision.derivateBits = 18;
        lodPrecision.lodBits = 6;
        lookupPrecision.colorThreshold = deMath.divide(
            tcuTexLookupVerifier.computeFixedPointThreshold(colorBits),
            refParams.colorScale
        );
        lookupPrecision.coordBits = [20, 20, 20];
        lookupPrecision.uvwBits = [7, 7, 0];
        lookupPrecision.colorMask =
            glsTextureTestUtil.getCompareMask(pixelFormat);

        var isHighQuality = glsTextureTestUtil.verifyTexture2DArrayResult(
            rendered.getAccess(), curCase.texture.getRefTexture().getView(),
            texCoord, refParams, lookupPrecision, lodPrecision, pixelFormat);

        if (!isHighQuality) {
            // Evaluate against lower precision requirements.
            lodPrecision.lodBits = 3;
            lookupPrecision.uvwBits = [3, 3, 0];

            bufferedLogToConsole(
                'Warning: Verification against high ' +
                'precision requirements failed, ' +
                'trying with lower requirements.'
            );

            var isOk = glsTextureTestUtil.verifyTexture2DArrayResult(
                rendered.getAccess(), curCase.texture.getRefTexture().getView(),
                texCoord, refParams, lookupPrecision, lodPrecision, pixelFormat
            );

            if (!isOk) {
                bufferedLogToConsole(
                    'ERROR: Verification against low precision requirements ' +
                    'failed, failing test case.'
                );
                testFailedOptions('Image verification failed', false);
                //In JS version, one mistake and you're out
                return tcuTestCase.IterateResult.STOP;
            } else
                checkMessage(
                    false,
                    'Low-quality filtering result in iteration no. ' +
                    this.m_caseNdx
                );
        }

        this.m_caseNdx += 1;
        if (this.m_caseNdx < this.m_cases.length)
            return tcuTestCase.IterateResult.CONTINUE;

        testPassed('Verified');
        return tcuTestCase.IterateResult.STOP;
    };

    // 3D filtering

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} minFilter
     * @param {number} magFilter
     * @param {number} wrapS
     * @param {number} wrapT
     * @param {number} wrapR
     * @param {number} internalFormat
     * @param {number} width
     * @param {number} height
     * @param {number} depth
     */
    es3fTextureFilteringTests.Texture3DFilteringCase = function(
        name, desc, minFilter, magFilter, wrapS, wrapT, wrapR, internalFormat,
        width, height, depth
    ) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_minFilter = minFilter;
        this.m_magFilter = magFilter;
        this.m_wrapS = wrapS;
        this.m_wrapT = wrapT;
        this.m_wrapR = wrapR;
        this.m_internalFormat = internalFormat;
        this.m_width = width;
        this.m_height = height;
        this.m_depth = depth;
        this.m_gradientTex = null;
        this.m_gridTex = null;
        /** @type {glsTextureTestUtil.TextureRenderer} */
        this.m_renderer = new glsTextureTestUtil.TextureRenderer(
            es3fTextureFilteringTests.version,
            gluShaderUtil.precision.PRECISION_HIGHP
        );
        this.m_caseNdx = 0;
        this.m_cases = [];
    };

    es3fTextureFilteringTests.Texture3DFilteringCase.prototype =
        Object.create(tcuTestCase.DeqpTest.prototype);

    es3fTextureFilteringTests.Texture3DFilteringCase.prototype.constructor =
        es3fTextureFilteringTests.Texture3DFilteringCase;

    /**
     * @constructor
     * @param {gluTexture.Texture3D} tex_
     * @param {Array<number>} lod_
     * @param {Array<number>} offset_
     */
    es3fTextureFilteringTests.Texture3DFilteringCase.FilterCase = function(
        tex_, lod_, offset_
    ) {
        this.texture = tex_;
        this.lod = lod_;
        this.offset = offset_;
    };

    /**
     * init
     */
    es3fTextureFilteringTests.Texture3DFilteringCase.prototype.init = function(
    ) {
        try {
            /** @type {tcuTexture.TextureFormat} */
            var texFmt =
                gluTextureUtil.mapGLInternalFormat(this.m_internalFormat);
            /** @type {tcuTextureUtil.TextureFormatInfo} */
            var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);
            var cScale = deMath.subtract(
                fmtInfo.valueMax, fmtInfo.valueMin
            );
            var cBias = fmtInfo.valueMin;
            var numLevels = deMath.logToFloor(
                Math.max(Math.max(this.m_width, this.m_height), this.m_depth)
            ) + 1;

            // Create textures.
            this.m_gradientTex = gluTexture.texture3DFromInternalFormat(
                gl, this.m_internalFormat,
                this.m_width, this.m_height, this.m_depth
            );

            this.m_gridTex = gluTexture.texture3DFromInternalFormat(
                gl, this.m_internalFormat,
                this.m_width, this.m_height, this.m_depth
            );

            // Fill first gradient texture.
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                var gMin = deMath.add(
                    deMath.multiply([0.0, 0.0, 0.0, 1.0], cScale), cBias
                );

                var gMax = deMath.add(
                    deMath.multiply([1.0, 1.0, 1.0, 0.0], cScale), cBias
                );

                this.m_gradientTex.getRefTexture().allocLevel(levelNdx);
                tcuTextureUtil.fillWithComponentGradients(
                    this.m_gradientTex.getRefTexture().getLevel(levelNdx),
                    gMin, gMax
                );
            }

            // Fill second with grid texture.
            for (var levelNdx = 0; levelNdx < numLevels; levelNdx++) {
                /** @type {number} */ var step = 0x00ffffff / numLevels;
                /** @type {number} */ var rgb = step * levelNdx;
                /** @type {number} */ var colorA = deMath.binaryOp(
                    0xff000000, rgb, deMath.BinaryOp.OR
                );
                /** @type {number} */ var colorB = deMath.binaryOp(
                    0xff000000, deMath.binaryNot(rgb), deMath.BinaryOp.OR
                );

                this.m_gridTex.getRefTexture().allocLevel(levelNdx);
                tcuTextureUtil.fillWithGrid(
                    this.m_gridTex.getRefTexture().getLevel(levelNdx), 4,
                    deMath.add(
                        deMath.multiply(
                            tcuRGBA.newRGBAFromValue(colorA).toVec(),
                            cScale
                        ),
                        cBias
                    ),
                    deMath.add(
                        deMath.multiply(
                            tcuRGBA.newRGBAFromValue(colorB).toVec(),
                            cScale
                        ),
                        cBias
                    )
                );
            }

            // Upload.
            this.m_gradientTex.upload();
            this.m_gridTex.upload();

            // Test cases
            this.m_cases.push(
                new es3fTextureFilteringTests.Texture3DFilteringCase.FilterCase(
                    this.m_gradientTex, [1.5, 2.8, 1.0], [-1.0, -2.7, -2.275]
                )
            );
            this.m_cases.push(
                new es3fTextureFilteringTests.Texture3DFilteringCase.FilterCase(
                    this.m_gradientTex, [-2.0, -1.5, -1.8], [-0.1, 0.9, -0.25]
                )
            );
            this.m_cases.push(
                new es3fTextureFilteringTests.Texture3DFilteringCase.FilterCase(
                    this.m_gridTex, [0.2, 0.175, 0.3], [-2.0, -3.7, -1.825]
                )
            );
            this.m_cases.push(
                new es3fTextureFilteringTests.Texture3DFilteringCase.FilterCase(
                    this.m_gridTex, [-0.8, -2.3, -2.5], [0.2, -0.1, 1.325]
                )
            );

            this.m_caseNdx = 0;
        }
        catch (e) {
            // Clean up to save memory.
            this.deinit();
            throw e;
        }
    };

    /**
     * deinit
     */
    es3fTextureFilteringTests.Texture3DFilteringCase.prototype.deinit =
    function() {
        if (this.m_gradientTex)
            gl.deleteTexture(this.m_gradientTex.getGLTexture());
        if (this.m_gridTex)
            gl.deleteTexture(this.m_gridTex.getGLTexture());

        this.m_gradientTex = null;
        this.m_gridTex = null;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fTextureFilteringTests.Texture3DFilteringCase.prototype.iterate =
    function() {
        /** @type {glsTextureTestUtil.RandomViewport} */
        var viewport = new glsTextureTestUtil.RandomViewport(
            gl.canvas, TEX3D_VIEWPORT_WIDTH,
            TEX3D_VIEWPORT_HEIGHT, deMath.binaryOp(
                deString.deStringHash(this.fullName()),
                deMath.deMathHash(this.m_caseNdx),
                deMath.BinaryOp.XOR
            )
        );

        /** @type {es3fTextureFilteringTests.Texture3DFilteringCase.FilterCase}
         */ var curCase = this.m_cases[this.m_caseNdx];

        /** @type {tcuTexture.TextureFormat} */
        var texFmt = curCase.texture.getRefTexture().getFormat();
        /** @type {tcuTextureUtil.TextureFormatInfo} */
        var fmtInfo = tcuTextureUtil.getTextureFormatInfo(texFmt);

        bufferedLogToConsole('Test' + this.m_caseNdx);
        /** @type {glsTextureTestUtil.ReferenceParams} */
        var refParams = new glsTextureTestUtil.ReferenceParams(
            glsTextureTestUtil.textureType.TEXTURETYPE_3D
        );

        /** @type {tcuSurface.Surface} */
        var rendered = new tcuSurface.Surface(viewport.width, viewport.height);
        /** @type {Array<number>}*/
        var texCoord = [];

        if (viewport.width < TEX3D_MIN_VIEWPORT_WIDTH ||
            viewport.height < TEX3D_MIN_VIEWPORT_HEIGHT)
            throw new Error('Too small render target');

        // Setup params for reference.
        refParams.sampler = gluTextureUtil.mapGLSampler(
            this.m_wrapS, this.m_wrapT, this.m_wrapR,
            this.m_minFilter, this.m_magFilter
        );

        // Setup params for reference.
        refParams.samplerType = glsTextureTestUtil.getSamplerType(texFmt);
        refParams.lodMode = glsTextureTestUtil.lodMode.EXACT;
        refParams.colorBias = fmtInfo.lookupBias;
        refParams.colorScale = fmtInfo.lookupScale;

        // Compute texture coordinates.
        bufferedLogToConsole('Approximate lod per axis = ' + curCase.lod +
            ', offset = ' + curCase.offset);

        /** @type {number} */ var lodX = curCase.lod[0];
        /** @type {number} */ var lodY = curCase.lod[1];
        /** @type {number} */ var lodZ = curCase.lod[2];
        /** @type {number} */ var oX = curCase.offset[0];
        /** @type {number} */ var oY = curCase.offset[1];
        /** @type {number} */ var oZ = curCase.offset[2];
        /** @type {number} */ var sX = Math.pow(2, lodX) * viewport.width /
            this.m_gradientTex.getRefTexture().getWidth();
        /** @type {number} */ var sY = Math.pow(2, lodY) * viewport.height /
            this.m_gradientTex.getRefTexture().getHeight();
        /** @type {number} */ var sZ = Math.pow(2, lodZ) *
            Math.max(viewport.width, viewport.height) /
            this.m_gradientTex.getRefTexture().getDepth();

        texCoord[0] = oX; texCoord[1] = oY; texCoord[2] = oZ;
        texCoord[3] = oX; texCoord[4] = oY + sY; texCoord[5] = oZ + sZ * 0.5;
        texCoord[6] = oX + sX; texCoord[7] = oY; texCoord[8] = oZ + sZ * 0.5;
        texCoord[9] = oX + sX; texCoord[10] = oY + sY; texCoord[11] = oZ + sZ;

        gl.bindTexture(gl.TEXTURE_3D, curCase.texture.getGLTexture());
        gl.texParameteri(
            gl.TEXTURE_3D, gl.TEXTURE_MIN_FILTER, this.m_minFilter
        );
        gl.texParameteri(
            gl.TEXTURE_3D, gl.TEXTURE_MAG_FILTER, this.m_magFilter
        );
        gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_S, this.m_wrapS);
        gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_T, this.m_wrapT);
        gl.texParameteri(gl.TEXTURE_3D, gl.TEXTURE_WRAP_R, this.m_wrapR);

        gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
        this.m_renderer.renderQuad(0, texCoord, refParams);
        rendered.readViewport(
            gl, [viewport.x, viewport.y, viewport.width, viewport.height]
        );

        var isNearestOnly = this.m_minFilter == gl.NEAREST &&
            this.m_magFilter == gl.NEAREST;
        /** @type {tcuPixelFormat.PixelFormat} */
        var pixelFormat = tcuPixelFormat.PixelFormatFromContext(gl);
        //(iVec4)
        var colorBits = deMath.max(
            deMath.addScalar(
                glsTextureTestUtil.getBitsVec(pixelFormat),
                // 1 inaccurate bit if nearest only, 2 otherwise
                -1 * (isNearestOnly ? 1 : 2)
            ),
            [0, 0, 0, 0]
        );
        /** @type {tcuTexLookupVerifier.LodPrecision} */
        var lodPrecision = new tcuTexLookupVerifier.LodPrecision();
        /** @type {tcuTexLookupVerifier.LookupPrecision} */
        var lookupPrecision = new tcuTexLookupVerifier.LookupPrecision();

        lodPrecision.derivateBits = 18;
        lodPrecision.lodBits = 6;
        lookupPrecision.colorThreshold = deMath.divide(
            tcuTexLookupVerifier.computeFixedPointThreshold(colorBits),
            refParams.colorScale
        );
        lookupPrecision.coordBits = [20, 20, 20];
        lookupPrecision.uvwBits = [7, 7, 7];
        lookupPrecision.colorMask =
            glsTextureTestUtil.getCompareMask(pixelFormat);

        var isHighQuality = glsTextureTestUtil.verifyTexture3DResult(
            rendered.getAccess(), curCase.texture.getRefTexture(),
            texCoord, refParams, lookupPrecision, lodPrecision, pixelFormat
        );

        if (!isHighQuality) {
            // Evaluate against lower precision requirements.
            lodPrecision.lodBits = 4;
            lookupPrecision.uvwBits = [4, 4, 4];

            bufferedLogToConsole(
                'Warning: Verification against high precision ' +
                'requirements failed, trying with lower requirements.'
            );

            var isOk = glsTextureTestUtil.verifyTexture3DResult(
                rendered.getAccess(), curCase.texture.getRefTexture(),
                texCoord, refParams, lookupPrecision, lodPrecision, pixelFormat
            );

            if (!isOk) {
                bufferedLogToConsole('ERROR: Verification against low ' +
                    'precision requirements failed, failing test case.'
                );
                testFailedOptions('Image verification failed', false);
                //In JS version, one mistake and you're out
                return tcuTestCase.IterateResult.STOP;
            } else
                checkMessage(
                    false,
                    'Low-quality filtering result in iteration no. ' +
                    this.m_caseNdx
                );
        }

        this.m_caseNdx += 1;
        if (this.m_caseNdx < this.m_cases.length)
            return tcuTestCase.IterateResult.CONTINUE;

        testPassed('Verified');
        return tcuTestCase.IterateResult.STOP;
    };

    /** @typedef {{name: string, mode: number}} */
    es3fTextureFilteringTests.WrapMode;

    /** @typedef {{name: string, mode: number}} */
    es3fTextureFilteringTests.MinFilterMode;

    /** @typedef {{name: string, mode: number}} */
    es3fTextureFilteringTests.MagFilterModes;

    /** @typedef {{width: number, height: number}} */
    es3fTextureFilteringTests.Sizes2D;

    /** @typedef {{width: number, height: number}} */
    es3fTextureFilteringTests.SizesCube;

    /** @typedef {{width: number, height: number, numLayers: number}} */
    es3fTextureFilteringTests.Sizes2DArray;

    /** @typedef {{width: number, height: number, depth: number}} */
    es3fTextureFilteringTests.Sizes3D;

    /** @typedef {{name: string, format: number}} */
    es3fTextureFilteringTests.FilterableFormatsByType;

    /**
     * init
     */
    es3fTextureFilteringTests.TextureFilteringTests.prototype.init =
    function() {
        /** @type {Array<es3fTextureFilteringTests.WrapMode>} */
        var wrapModes = [{
                name: 'clamp', mode: gl.CLAMP_TO_EDGE
            }, {
                name: 'repeat', mode: gl.REPEAT
            }, {
                name: 'mirror', mode: gl.MIRRORED_REPEAT
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.MinFilterMode>} */
        var minFilterModes = [{
                name: 'nearest', mode: gl.NEAREST
            }, {
                name: 'linear', mode: gl.LINEAR
            }, {
                name: 'nearest_mipmap_nearest', mode: gl.NEAREST_MIPMAP_NEAREST
            }, {
                name: 'linear_mipmap_nearest', mode: gl.LINEAR_MIPMAP_NEAREST
            }, {
                name: 'nearest_mipmap_linear', mode: gl.NEAREST_MIPMAP_LINEAR
            }, {
                name: 'linear_mipmap_linear', mode: gl.LINEAR_MIPMAP_LINEAR
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.MagFilterModes>} */
        var magFilterModes = [{
                   name: 'nearest', mode: gl.NEAREST
            }, {
                   name: 'linear', mode: gl.LINEAR
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.Sizes2D>} */
        var sizes2D = [{
                width: 4, height: 8
            }, {
                width: 32, height: 64
            }, {
                width: 128, height: 128
            }, {
                width: 3, height: 7
            }, {
                width: 31, height: 55
            }, {
                width: 127, height: 99
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.SizesCube>} */
        var sizesCube = [{
                width: 8, height: 8
            }, {
                width: 64, height: 64
            }, {
                width: 128, height: 128
            }, {
                width: 7, height: 7
            }, {
                width: 63, height: 63
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.Sizes2DArray>} */
        var sizes2DArray = [{
                width: 4, height: 8, numLayers: 8
            }, {
                width: 32, height: 64, numLayers: 16
            }, {
                width: 128, height: 32, numLayers: 64
            }, {
                width: 3, height: 7, numLayers: 5
            }, {
                width: 63, height: 63, numLayers: 63
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.Sizes3D>} */
        var sizes3D = [{
                width: 4, height: 8, depth: 8
            }, {
                width: 32, height: 64, depth: 16
            }, {
                width: 128, height: 32, depth: 64
            }, {
                width: 3, height: 7, depth: 5
            }, {
                width: 63, height: 63, depth: 63
            }
        ];

        /** @type {Array<es3fTextureFilteringTests.FilterableFormatsByType>} */
        var filterableFormatsByType = [{
                name: 'rgba16f', format: gl.RGBA16F
            }, {
                name: 'r11f_g11f_b10f', format: gl.R11F_G11F_B10F
            }, {
                name: 'rgb9_e5', format: gl.RGB9_E5
            }, {
                name: 'rgba8', format: gl.RGBA8
            }, {
                name: 'rgba8_snorm', format: gl.RGBA8_SNORM
            }, {
                name: 'rgb565', format: gl.RGB565
            }, {
                name: 'rgba4', format: gl.RGBA4
            }, {
                name: 'rgb5_a1', format: gl.RGB5_A1
            }, {
                name: 'srgb8_alpha8', format: gl.SRGB8_ALPHA8
            }, {
                name: 'rgb10_a2', format: gl.RGB10_A2
            }
        ];

        // 2D texture filtering.

        // Formats.
        /** @type {tcuTestCase.DeqpTest} */
        var formatsGroup;
        for (var fmtNdx = 0;
            fmtNdx < filterableFormatsByType.length;
            fmtNdx++) {
            formatsGroup = new tcuTestCase.DeqpTest(
                '2d_formats', '2D Texture Formats');
            this.addChild(formatsGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                 /** @type {number} */
                var minFilter = minFilterModes[filterNdx].mode;
                 /** @type {string} */
                var filterName = minFilterModes[filterNdx].name;
                 /** @type {number} */
                var format = filterableFormatsByType[fmtNdx].format;
                 /** @type {string} */
                var formatName = filterableFormatsByType[fmtNdx].name;
                 var isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                 /** @type {number} */
                var magFilter = isMipmap ? gl.LINEAR : minFilter;
                 /** @type {string} */
                var name = formatName + '_' + filterName;
                 /** @type {number} */
                var wrapS = gl.REPEAT;
                 /** @type {number} */
                var wrapT = gl.REPEAT;
                 /** @type {number} */ var width = 64;
                 /** @type {number} */ var height = 64;

                 formatsGroup.addChild(
                    new es3fTextureFilteringTests.Texture2DFilteringCase(
                        name, '', minFilter, magFilter, wrapS, wrapT,
                         format, width, height
                    )
                );
            }
        }

        // Sizes.
        /** @type {tcuTestCase.DeqpTest} */
        var sizesGroup;
        for (var sizeNdx = 0; sizeNdx < sizes2D.length; sizeNdx++) {
            sizesGroup = new tcuTestCase.DeqpTest(
                '2d_sizes', '2D Texture Sizes');
            this.addChild(sizesGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                format = gl.RGBA8;
                isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                magFilter = isMipmap ? gl.LINEAR : minFilter;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                width = sizes2D[sizeNdx].width;
                height = sizes2D[sizeNdx].height;
                name = '' + width + 'x' + height + '_' + filterName;

                sizesGroup.addChild(
                    new es3fTextureFilteringTests.Texture2DFilteringCase(
                        name, '', minFilter, magFilter, wrapS, wrapT,
                        format, width, height
                    )
                );
            }
        }

        // Wrap modes.
        /** @type {tcuTestCase.DeqpTest} */
        var combinationsGroup;
        for (var minFilterNdx = 0;
            minFilterNdx < minFilterModes.length;
            minFilterNdx++) {
            combinationsGroup = new tcuTestCase.DeqpTest(
                '2d_combinations', '2D Filter and wrap mode combinations');
            this.addChild(combinationsGroup);
            for (var magFilterNdx = 0;
                magFilterNdx < magFilterModes.length;
                magFilterNdx++) {
                for (var wrapSNdx = 0;
                    wrapSNdx < wrapModes.length;
                    wrapSNdx++) {
                    for (var wrapTNdx = 0;
                        wrapTNdx < wrapModes.length;
                        wrapTNdx++) {
                        minFilter = minFilterModes[minFilterNdx].mode;
                        magFilter = magFilterModes[magFilterNdx].mode;
                        format = gl.RGBA8;
                        wrapS = wrapModes[wrapSNdx].mode;
                        wrapT = wrapModes[wrapTNdx].mode;
                        width = 63;
                        height = 57;
                        name = minFilterModes[minFilterNdx].name + '_' +
                            magFilterModes[magFilterNdx].name + '_' +
                            wrapModes[wrapSNdx].name + '_' +
                            wrapModes[wrapTNdx].name;

                        combinationsGroup.addChild(
                            new
                            es3fTextureFilteringTests.Texture2DFilteringCase(
                                name, '', minFilter, magFilter, wrapS, wrapT,
                                format, width, height
                            )
                        );
                    }
                }
            }
        }

        // Cube map texture filtering.

        // Formats.
        for (var fmtNdx = 0;
            fmtNdx < filterableFormatsByType.length;
            fmtNdx++) {
            formatsGroup = new tcuTestCase.DeqpTest(
                'cube_formats', 'Cube Texture Formats');
            this.addChild(formatsGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                format = filterableFormatsByType[fmtNdx].format;
                formatName = filterableFormatsByType[fmtNdx].name;
                isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                magFilter = isMipmap ? gl.LINEAR : minFilter;
                name = formatName + '_' + filterName;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                width = 64;
                height = 64;

                formatsGroup.addChild(
                    new es3fTextureFilteringTests.TextureCubeFilteringCase(
                        name, '', minFilter, magFilter, wrapS, wrapT,
                        false /* always sample exterior as well */,
                        format, width, height
                    )
                );
            }
        }

        // Sizes.
        for (var sizeNdx = 0; sizeNdx < sizesCube.length; sizeNdx++) {
            sizesGroup = new tcuTestCase.DeqpTest(
                'cube_sizes', 'Cube Texture Sizes');
            this.addChild(sizesGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                var format = gl.RGBA8;
                isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                magFilter = isMipmap ? gl.LINEAR : minFilter;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                width = sizesCube[sizeNdx].width;
                height = sizesCube[sizeNdx].height;
                name = '' + width + 'x' + height + '_' + filterName;

                sizesGroup.addChild(
                    new es3fTextureFilteringTests.TextureCubeFilteringCase(
                        name, '', minFilter, magFilter, wrapS, wrapT,
                        false, format, width, height
                    )
                );
            }
        }

        // Filter/wrap mode combinations.
        for (var minFilterNdx = 0;
            minFilterNdx < minFilterModes.length;
            minFilterNdx++) {
            combinationsGroup = new tcuTestCase.DeqpTest(
                'cube_combinations', 'Cube Filter and wrap mode combinations'
            );
            this.addChild(combinationsGroup);
            for (var magFilterNdx = 0;
                magFilterNdx < magFilterModes.length;
                magFilterNdx++) {
                for (var wrapSNdx = 0;
                    wrapSNdx < wrapModes.length;
                    wrapSNdx++) {
                    for (var wrapTNdx = 0;
                        wrapTNdx < wrapModes.length;
                        wrapTNdx++) {
                        minFilter = minFilterModes[minFilterNdx].mode;
                        magFilter = magFilterModes[magFilterNdx].mode;
                        format = gl.RGBA8;
                        wrapS = wrapModes[wrapSNdx].mode;
                        wrapT = wrapModes[wrapTNdx].mode;
                        width = 63;
                        height = 63;
                        name = minFilterModes[minFilterNdx].name + '_' +
                            magFilterModes[magFilterNdx].name + '_' +
                            wrapModes[wrapSNdx].name + '_' +
                            wrapModes[wrapTNdx].name;

                        combinationsGroup.addChild(
                            new es3fTextureFilteringTests.
                            TextureCubeFilteringCase(
                                name, '', minFilter, magFilter, wrapS, wrapT,
                                false, format, width, height
                            )
                        );
                    }
                }
            }
        }

        // Cases with no visible cube edges.
        /** @type {tcuTestCase.DeqpTest} */
        var onlyFaceInteriorGroup = new tcuTestCase.DeqpTest(
            'cube_no_edges_visible', "Don't sample anywhere near a face's edges"
        );
        this.addChild(onlyFaceInteriorGroup);

        for (var isLinearI = 0; isLinearI <= 1; isLinearI++) {
            var isLinear = isLinearI != 0;
            var filter = isLinear ? gl.LINEAR : gl.NEAREST;

            onlyFaceInteriorGroup.addChild(
                new es3fTextureFilteringTests.TextureCubeFilteringCase(
                    isLinear ? 'linear' : 'nearest', '',
                    filter, filter, gl.REPEAT, gl.REPEAT,
                    true, gl.RGBA8, 63, 63
                )
            );
        }

        // Formats.
        for (var fmtNdx = 0;
            fmtNdx < filterableFormatsByType.length;
            fmtNdx++) {
            formatsGroup = new tcuTestCase.DeqpTest(
                '2d_array_formats', '2D Array Texture Formats');
            this.addChild(formatsGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                format = filterableFormatsByType[fmtNdx].format;
                var formatName = filterableFormatsByType[fmtNdx].name;
                isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                magFilter = isMipmap ? gl.LINEAR : minFilter;
                name = formatName + '_' + filterName;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                width = 128;
                height = 128;
                /** @type {number} */ var numLayers = 8;

                formatsGroup.addChild(
                    new es3fTextureFilteringTests.Texture2DArrayFilteringCase(
                        name, '', minFilter, magFilter, wrapS, wrapT,
                        format, width, height, numLayers
                    )
                );
            }
        }

        // Sizes.
        for (var sizeNdx = 0; sizeNdx < sizes2DArray.length; sizeNdx++) {
            sizesGroup = new tcuTestCase.DeqpTest(
                '2d_array_sizes', '2D Array Texture Sizes');
            this.addChild(sizesGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                format = gl.RGBA8;
                isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                magFilter = isMipmap ? gl.LINEAR : minFilter;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                width = sizes2DArray[sizeNdx].width;
                height = sizes2DArray[sizeNdx].height;
                numLayers = sizes2DArray[sizeNdx].numLayers;
                name = '' + width + 'x' + height + 'x' +
                    numLayers + '_' + filterName;

                sizesGroup.addChild(
                    new es3fTextureFilteringTests.Texture2DArrayFilteringCase(
                        name, '', minFilter, magFilter, wrapS, wrapT,
                        format, width, height, numLayers
                    )
                );
            }
        }

        // Wrap modes.
        for (var minFilterNdx = 0;
            minFilterNdx < minFilterModes.length;
            minFilterNdx++) {
            combinationsGroup = new tcuTestCase.DeqpTest(
                '2d_array_combinations',
                '2D Array Filter and wrap mode combinations');
            this.addChild(combinationsGroup);
            for (var magFilterNdx = 0;
                magFilterNdx < magFilterModes.length;
                magFilterNdx++) {
                for (var wrapSNdx = 0;
                    wrapSNdx < wrapModes.length;
                    wrapSNdx++) {
                    for (var wrapTNdx = 0;
                        wrapTNdx < wrapModes.length;
                        wrapTNdx++) {
                        minFilter = minFilterModes[minFilterNdx].mode;
                        magFilter = magFilterModes[magFilterNdx].mode;
                        format = gl.RGBA8;
                        wrapS = wrapModes[wrapSNdx].mode;
                        wrapT = wrapModes[wrapTNdx].mode;
                        width = 123;
                        height = 107;
                        numLayers = 7;
                        name = minFilterModes[minFilterNdx].name + '_' +
                            magFilterModes[magFilterNdx].name + '_' +
                            wrapModes[wrapSNdx].name + '_' +
                            wrapModes[wrapTNdx].name;

                        combinationsGroup.addChild(
                            new es3fTextureFilteringTests.
                            Texture2DArrayFilteringCase(
                                name, '', minFilter, magFilter,
                                wrapS, wrapT, format,
                                width, height, numLayers
                            )
                        );
                    }
                }
            }
        }

        // 3D texture filtering.

        // Formats.
        /** @type {number} */ var depth = 64;
        for (var fmtNdx = 0;
            fmtNdx < filterableFormatsByType.length;
            fmtNdx++) {
            formatsGroup = new tcuTestCase.DeqpTest(
                '3d_formats', '3D Texture Formats');
            this.addChild(formatsGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                format = filterableFormatsByType[fmtNdx].format;
                formatName = filterableFormatsByType[fmtNdx].name;
                isMipmap = minFilter != gl.NEAREST &&
                    minFilter != gl.LINEAR;
                magFilter = isMipmap ? gl.LINEAR : minFilter;
                name = formatName + '_' + filterName;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                /** @type {number} */ var wrapR = gl.REPEAT;
                width = 64;
                height = 64;
                depth = 64;

                formatsGroup.addChild(
                    new es3fTextureFilteringTests.Texture3DFilteringCase(
                        name, '', minFilter, magFilter,
                        wrapS, wrapT, wrapR, format,
                        width, height, depth
                    )
                );
            }
        }

        // Sizes.
        for (var sizeNdx = 0; sizeNdx < sizes3D.length; sizeNdx++) {
            sizesGroup = new tcuTestCase.DeqpTest(
                '3d_sizes', '3D Texture Sizes');
            this.addChild(sizesGroup);
            for (var filterNdx = 0;
                filterNdx < minFilterModes.length;
                filterNdx++) {
                minFilter = minFilterModes[filterNdx].mode;
                filterName = minFilterModes[filterNdx].name;
                format = gl.RGBA8;
                isMipmap =
                    minFilter != gl.NEAREST && minFilter != gl.LINEAR;
                magFilter =
                    isMipmap ? gl.LINEAR : minFilter;
                wrapS = gl.REPEAT;
                wrapT = gl.REPEAT;
                wrapR = gl.REPEAT;
                width = sizes3D[sizeNdx].width;
                height = sizes3D[sizeNdx].height;
                depth = sizes3D[sizeNdx].depth;
                name = '' + width + 'x' + height + 'x' + depth +
                    '_' + filterName;

                sizesGroup.addChild(
                    new es3fTextureFilteringTests.Texture3DFilteringCase(
                        name, '', minFilter, magFilter,
                        wrapS, wrapT, wrapR, format,
                        width, height, depth
                    )
                );
            }
        }

        // Wrap modes.
        for (var minFilterNdx = 0;
            minFilterNdx < minFilterModes.length;
            minFilterNdx++) {
            for (var magFilterNdx = 0;
                magFilterNdx < magFilterModes.length;
                magFilterNdx++) {
                for (var wrapSNdx = 0;
                    wrapSNdx < wrapModes.length;
                    wrapSNdx++) {
                    combinationsGroup = new tcuTestCase.DeqpTest(
                        '3d_combinations',
                        '3D Filter and wrap mode combinations');
                    this.addChild(combinationsGroup);
                    for (var wrapTNdx = 0;
                        wrapTNdx < wrapModes.length;
                        wrapTNdx++) {
                        for (var wrapRNdx = 0;
                            wrapRNdx < wrapModes.length;
                            wrapRNdx++) {
                            minFilter = minFilterModes[minFilterNdx].mode;
                            magFilter = magFilterModes[magFilterNdx].mode;
                            format = gl.RGBA8;
                            wrapS = wrapModes[wrapSNdx].mode;
                            wrapT = wrapModes[wrapTNdx].mode;
                            wrapR = wrapModes[wrapRNdx].mode;
                            width = 63;
                            height = 57;
                            depth = 67;
                            name = minFilterModes[minFilterNdx].name + '_' +
                                magFilterModes[magFilterNdx].name + '_' +
                                wrapModes[wrapSNdx].name + '_' +
                                wrapModes[wrapTNdx].name + '_' +
                                wrapModes[wrapRNdx].name;

                            combinationsGroup.addChild(
                                new
                                es3fTextureFilteringTests.
                                Texture3DFilteringCase(
                                    name, '', minFilter, magFilter,
                                    wrapS, wrapT, wrapR, format,
                                    width, height, depth
                                )
                            );
                        }
                    }
                }
            }
        }
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     * @param {Array<number>=} range Test range
     */
    es3fTextureFilteringTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;

        state.setRoot(new es3fTextureFilteringTests.TextureFilteringTests());
        if (range)
            state.setRange(range);

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to run tests', false);
            tcuTestCase.runner.terminate();
        }
    };
});
