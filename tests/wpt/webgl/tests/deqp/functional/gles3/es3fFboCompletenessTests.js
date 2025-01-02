
// FboCompletenessTests
'use strict';
goog.provide('functional.gles3.es3fFboCompletenessTests');
goog.require('framework.common.tcuTestCase');
goog.require('modules.shared.glsFboCompletenessTests');
goog.require('modules.shared.glsFboUtil');

goog.scope(function() {

    var es3fFboCompletenessTests = functional.gles3.es3fFboCompletenessTests;
    var glsFboUtil = modules.shared.glsFboUtil;
    var glsFboCompletenessTests = modules.shared.glsFboCompletenessTests;
    var tcuTestCase = framework.common.tcuTestCase;

    es3fFboCompletenessTests.initGlDependents = function(gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        /**
        * @type {Array<number>}
        */
        es3fFboCompletenessTests.s_es3ColorRenderables = [
            // GLES3, 4.4.4: "An internal format is color-renderable if it is one of
            // the formats from table 3.12 noted as color-renderable..."
            gl.R8, gl.RG8, gl.RGB8, gl.RGB565, gl.RGBA4, gl.RGB5_A1, gl.RGBA8,
            gl.RGB10_A2, gl.RGB10_A2UI, gl.SRGB8_ALPHA8,
            gl.R8I, gl.R8UI, gl.R16I, gl.R16UI, gl.R32I, gl.R32UI,
            gl.RG8I, gl.RG8UI, gl.RG16I, gl.RG16UI, gl.RG32I, gl.RG32UI,
            gl.RGBA81, gl.RGBA8UI, gl.RGB16I, gl.RGBA16UI, gl.RGBA32I, gl.RGBA32UI
        ];

        /**
        * @type {Array<number>}
        */
        es3fFboCompletenessTests.s_es3UnsizedColorRenderables = [
            // "...or if it is unsized format RGBA or RGB."
            // See Table 3.3 in GLES3.
            glsFboUtil.formatkey(gl.RGBA, gl.UNSIGNED_BYTE),
            glsFboUtil.formatkey(gl.RGBA, gl.UNSIGNED_SHORT_4_4_4_4),
            glsFboUtil.formatkey(gl.RGBA, gl.UNSIGNED_SHORT_5_5_5_1),
            glsFboUtil.formatkey(gl.RGB, gl.UNSIGNED_BYTE),
            glsFboUtil.formatkey(gl.RGB, gl.UNSIGNED_SHORT_5_6_5)
        ];

        /**
        * @type {Array<number>}
        */
        es3fFboCompletenessTests.s_es3DepthRenderables = [
            // GLES3, 4.4.4: "An internal format is depth-renderable if it is one of
            // the formats from table 3.13."
            gl.DEPTH_COMPONENT16, gl.DEPTH_COMPONENT24, gl.DEPTH_COMPONENT32F,
            gl.DEPTH24_STENCIL8, gl.DEPTH32F_STENCIL8
        ];

        /**
        * @type {Array<number>}
        */
        es3fFboCompletenessTests.s_es3StencilRboRenderables = [
            // GLES3, 4.4.4: "An internal format is stencil-renderable if it is
            // STENCIL_INDEX8..."
            gl.STENCIL_INDEX8
        ];

        /**
        * @type {Array<number>}
        */
        es3fFboCompletenessTests.s_es3StencilRenderables = [
            // "...or one of the formats from table 3.13 whose base internal format is
            // DEPTH_STENCIL."
            gl.DEPTH24_STENCIL8, gl.DEPTH32F_STENCIL8
        ];

        /**
        * @type {Array<number>}
        */
        es3fFboCompletenessTests.s_es3TextureFloatFormats = [
            gl.RGBA32F, gl.RGBA16F, gl.R11F_G11F_B10F,
            gl.RG32F, gl.RG16F, gl.R32F, gl.R16F,
            gl.RGBA16F, gl.RGB16F, gl.RG16F, gl.R16F
        ];

        /**
        * @type {Array<glsFboUtil.formatT>}
        */
        es3fFboCompletenessTests.s_es3Formats = [
            [
                (
                    glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                    glsFboUtil.FormatFlags.COLOR_RENDERABLE |
                    glsFboUtil.FormatFlags.TEXTURE_VALID
                ),
                glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3UnsizedColorRenderables)
            ],
            [
                (
                    glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                    glsFboUtil.FormatFlags.COLOR_RENDERABLE |
                    glsFboUtil.FormatFlags.RENDERBUFFER_VALID |
                    glsFboUtil.FormatFlags.TEXTURE_VALID
                ),
                glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3ColorRenderables)
            ], [
                (
                    glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                    glsFboUtil.FormatFlags.DEPTH_RENDERABLE |
                    glsFboUtil.FormatFlags.RENDERBUFFER_VALID |
                    glsFboUtil.FormatFlags.TEXTURE_VALID
                ),
                glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3DepthRenderables)
            ], [
                (
                    glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                    glsFboUtil.FormatFlags.STENCIL_RENDERABLE |
                    glsFboUtil.FormatFlags.RENDERBUFFER_VALID
                ),
                glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3StencilRboRenderables)
            ], [
                (
                    glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                    glsFboUtil.FormatFlags.STENCIL_RENDERABLE |
                    glsFboUtil.FormatFlags.RENDERBUFFER_VALID |
                    glsFboUtil.FormatFlags.TEXTURE_VALID
                ),
                glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3StencilRenderables)
            ],

            // These are not color-renderable in vanilla ES3, but we need to mark them
            // as valid for textures, since EXT_color_buffer_(half_)float brings in
            // color-renderability and only renderbuffer-validity.
            [
                glsFboUtil.FormatFlags.TEXTURE_VALID,
                glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3TextureFloatFormats)
            ]
        ];

        // gl.EXT_color_buffer_float
        es3fFboCompletenessTests.s_extColorBufferFloatFormats = [
            gl.RGBA32F, gl.RGBA16F, gl.R11F_G11F_B10F, gl.RG32F, gl.RG16F, gl.R32F, gl.R16F
        ];

        // gl.OES_texture_stencil8
        es3fFboCompletenessTests.s_extOESTextureStencil8 = [
            gl.STENCIL_INDEX8
        ];

        es3fFboCompletenessTests.s_es3ExtFormats = [{
                extensions: 'gl.EXT_color_buffer_float',
                flags: glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                            glsFboUtil.FormatFlags.COLOR_RENDERABLE |
                            glsFboUtil.FormatFlags.RENDERBUFFER_VALID,
                formats: new glsFboUtil.Range(es3fFboCompletenessTests.s_extColorBufferFloatFormats)
            }, {
                extensions: 'gl.OES_texture_stencil8',
                flags: glsFboUtil.FormatFlags.REQUIRED_RENDERABLE |
                            glsFboUtil.FormatFlags.STENCIL_RENDERABLE |
                            glsFboUtil.FormatFlags.TEXTURE_VALID,
                formats: new glsFboUtil.Range(es3fFboCompletenessTests.s_extOESTextureStencil8)
            }
        ];

        glsFboCompletenessTests.initGlDependents(gl);
    };

    /**
     * @constructor
     * @extends {glsFboUtil.Checker}
     */
    es3fFboCompletenessTests.ES3Checker = function() {
        glsFboUtil.Checker.call(this, gl);
        /** @type {number} */ this.m_numSamples = -1; // GLsizei
        /** @type {number} */ this.m_depthStencilImage = 0; // GLuint
        /** @type {number} */ this.m_depthStencilType = gl.NONE;
    };
    es3fFboCompletenessTests.ES3Checker.prototype = Object.create(glsFboUtil.Checker.prototype);
    es3fFboCompletenessTests.ES3Checker.prototype.constructor = es3fFboCompletenessTests.ES3Checker;

    es3fFboCompletenessTests.ES3Checker.prototype.check = function(attPoint, att, image) {

        var imgSamples = glsFboUtil.imageNumSamples(image);

        if (this.m_numSamples == -1) {
            this.m_numSamples = imgSamples;
        } else {
            // GLES3: "The value of RENDERBUFFER_SAMPLES is the same for all attached
            // renderbuffers and, if the attached images are a mix of renderbuffers
            // and textures, the value of RENDERBUFFER_SAMPLES is zero."
            //
            // On creating a renderbuffer: "If _samples_ is zero, then
            // RENDERBUFFER_SAMPLES is set to zero. Otherwise [...] the resulting
            // value for RENDERBUFFER_SAMPLES is guaranteed to be greater than or
            // equal to _samples_ and no more than the next larger sample count
            // supported by the implementation."

            // Either all attachments are zero-sample renderbuffers and/or
            // textures, or none of them are.
            this.addFBOStatus(
                (this.m_numSamples == 0) == (imgSamples == 0),
                gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE
            );

            // If the attachments requested a different number of samples, the
            // implementation is allowed to report this as incomplete. However, it
            // is also possible that despite the different requests, the
            // implementation allocated the same number of samples to both. Hence
            // reporting the framebuffer as complete is also legal.
            this.addPotentialFBOStatus(
                this.m_numSamples == imgSamples,
                gl.FRAMEBUFFER_INCOMPLETE_MULTISAMPLE
            );
        }

        // "Depth and stencil attachments, if present, are the same image."
        if (attPoint == gl.DEPTH_ATTACHMENT || attPoint == gl.STENCIL_ATTACHMENT) {
            if (this.m_depthStencilImage == 0) {
                this.m_depthStencilImage = att.imageName;
                this.m_depthStencilType = glsFboUtil.attachmentType(att);

            } else {
                this.addFBOStatus(
                    this.m_depthStencilImage == att.imageName && this.m_depthStencilType == glsFboUtil.attachmentType(att),
                    gl.FRAMEBUFFER_UNSUPPORTED
                );
            }
        }

    };

    /**
    * @typedef {{textureKind: number, numLayers: number, attachmentLayer: number}}
    */
    es3fFboCompletenessTests.numLayersParamsT;

    /**
    * @param {number} textureKind
    * @param {number} numLayers
    * @param {number} attachmentLayer
    * @return {es3fFboCompletenessTests.numLayersParamsT}
    */
    es3fFboCompletenessTests.numLayersParams = function(textureKind, numLayers, attachmentLayer) {
        if (typeof(attachmentLayer) == 'undefined') {
            textureKind = 0;
            numLayers = 0;
            attachmentLayer = 0;
        }
        return {
            textureKind: textureKind, //< gl.TEXTURE_3D or gl.TEXTURE_2D_ARRAY
            numLayers: numLayers, //< Number of layers in texture
            attachmentLayer: attachmentLayer //< Layer referenced by attachment
        };
    };

    /**
     * es3fFboCompletenessTests.numLayersParams.getName
     * @param {es3fFboCompletenessTests.numLayersParamsT} params
     * @return {string}
     */
    es3fFboCompletenessTests.numLayersParams.getName = function(params) {
        return (
            (params.textureKind == gl.TEXTURE_3D ? '3d' : '2darr') + '_' +
            params.numLayers + '_' +
            params.attachmentLayer
        );
    };
    /**
     * es3fFboCompletenessTests.numLayersParams.getDescription
     * @param {es3fFboCompletenessTests.numLayersParamsT} params
     * @return {string}
     */
    es3fFboCompletenessTests.numLayersParams.getDescription = function(params) {
        return (
            (params.textureKind == gl.TEXTURE_3D ? '3D Texture' : '2D Array Texture') + ', ' +
            params.numLayers + ' layers, ' +
            'attached layer ' + params.attachmentLayer + '.'
        );
    };

    // string, string, glsFboCompleteness::context, params.
    /**
    * @constructor
    * @extends {glsFboCompletenessTests.TestBase}
    * @param {string} name
    * @param {string} desc
    * @param {glsFboCompletenessTests.Context} ctx
    * @param {es3fFboCompletenessTests.numLayersParamsT} params
    */
    es3fFboCompletenessTests.NumLayersTest = function(name, desc, ctx, params) {
        glsFboCompletenessTests.TestBase.call(this, name, desc, params);
        this.m_ctx = ctx;
    };

    es3fFboCompletenessTests.NumLayersTest.prototype = Object.create(glsFboCompletenessTests.TestBase.prototype);
    es3fFboCompletenessTests.NumLayersTest.prototype.constructor = es3fFboCompletenessTests.NumLayersTest;

    es3fFboCompletenessTests.NumLayersTest.prototype.build = function(builder, gl) {

        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        var target = gl.COLOR_ATTACHMENT0;
        var texCfg = builder.makeConfig(
            function(kind) {
                switch (kind) {
                    case gl.TEXTURE_3D: return glsFboUtil.Texture3D;
                    case gl.TEXTURE_2D_ARRAY: return glsFboUtil.Texture2DArray;
                    default: throw new Error('Impossible case');
                }
            }(this.m_params.textureKind)
        );

        texCfg.internalFormat = this.getDefaultFormat(target, gl.TEXTURE, gl);
        texCfg.width = 64;
        texCfg.height = 64;
        texCfg.numLayers = this.m_params.numLayers;
        var tex = builder.glCreateTexture(texCfg);

        var att = builder.makeConfig(glsFboUtil.TextureLayerAttachment);
        att.layer = this.m_params.attachmentLayer;
        att.imageName = tex;

        builder.glAttach(target, att);

    //  return tcuTestCase.IterateResult.STOP;
    };
//es3fFboCompletenessTests.NumLayersTest.prototype.isExecutable = function() {
//    return false;
//};

    /**
     * @enum
     */
    es3fFboCompletenessTests.e_samples = {
        NONE: -2,
        TEXTURE: -1
    };

    /**
    * @typedef {{numSamples: Array<number>}}
    */
    es3fFboCompletenessTests.numSamplesParamsT;

    /**
    * @param {number} colour
    * @param {number} depth
    * @param {number} stencil
    * @return {es3fFboCompletenessTests.numSamplesParamsT}
    */
    es3fFboCompletenessTests.numSamplesParams = function(colour, depth, stencil) {
        var ret = {
            numSamples: new Array(3)
        };
        if (colour !== undefined) {
            ret.numSamples[0] = colour;
            if (depth !== undefined) {
                ret.numSamples[1] = depth;
                if (stencil !== undefined) {
                    ret.numSamples[2] = stencil;
                }
            }
        }
        return ret;
    };

    /**
    * @param {es3fFboCompletenessTests.numSamplesParamsT} params
    * @return {string}
    */
    es3fFboCompletenessTests.numSamplesParams.getName = function(params) {
        var out = '';

        var first = true;
        for (var i = 0; i < 3; ++i) {
            if (first)
                first = false;
            else
                out += '_';

            switch (params.numSamples[i]) {
                case es3fFboCompletenessTests.e_samples.NONE: out += 'none'; break;
                case es3fFboCompletenessTests.e_samples.TEXTURE: out += 'tex'; break;
                default: out += 'rbo'; break;
            }
        }
        return out;
    };
    /**
    * @param {es3fFboCompletenessTests.numSamplesParamsT} params
    * @return {string}
    */
    es3fFboCompletenessTests.numSamplesParams.getDescription = function(params) {
        var out = '';
        var names = ['color', 'depth', 'stencil'];
        var first = true;

        for (var i = 0; i < 3; ++i) {
            if (first)
                first = false;
            else
                out += ', ';

            if (params.numSamples[i] == es3fFboCompletenessTests.e_samples.TEXTURE) {
                out += 'texture ' + names[i] + ' attachment';
            } else {
                out += params.numSamples[i] + '-sample renderbuffer ' + names[i] + ' attachment';
            }
        }
        return out;
    };

    /**
    * @constructor
    * @extends {glsFboCompletenessTests.TestBase}
    * @param {string} name
    * @param {string} desc
    * @param {glsFboCompletenessTests.Context} ctx
    * @param {es3fFboCompletenessTests.numSamplesParamsT} params
    */
    es3fFboCompletenessTests.NumSamplesTest = function(name, desc, ctx, params) {
        glsFboCompletenessTests.TestBase.call(this, name, desc, params);
        this.m_ctx = ctx;
    };
    es3fFboCompletenessTests.NumSamplesTest.prototype = Object.create(glsFboCompletenessTests.TestBase.prototype);
    es3fFboCompletenessTests.NumSamplesTest.prototype.constructor = es3fFboCompletenessTests.NumSamplesTest;

    es3fFboCompletenessTests.NumSamplesTest.prototype.build = function(builder, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        var s_targets = [
            gl.COLOR_ATTACHMENT0, gl.COLOR_ATTACHMENT1, gl.DEPTH_ATTACHMENT
        ];
        // Non-integer formats for each attachment type.
        // \todo [2013-12-17 lauri] Add fixed/floating/integer metadata for formats so
        // we can pick one smartly or maybe try several.
        var s_formats = [
            gl.RGBA8, gl.RGB565, gl.DEPTH_COMPONENT24
        ];

        var l = s_targets.length;
        if (this.m_params.numSamples.length != l)
            throw new Error('Wrong number of params.');

        for (var i = 0; i < l; ++i) {
            var target = s_targets[i];
            var fmt = new glsFboUtil.ImageFormat(s_formats[i], gl.NONE);

            var ns = this.m_params.numSamples[i];
            if (ns == es3fFboCompletenessTests.e_samples.NONE)
                continue;
            if (ns == es3fFboCompletenessTests.e_samples.TEXTURE) {
                this.attachTargetToNew(target, gl.TEXTURE, fmt, 64, 64, builder, gl);
            } else {
                var rboCfg = builder.makeConfig(glsFboUtil.Renderbuffer);
                rboCfg.internalFormat = fmt;
                rboCfg.width = rboCfg.height = 64;
                rboCfg.numSamples = ns;

                var rbo = builder.glCreateRbo(rboCfg);
                // Implementations do not necessarily support sample sizes greater than 1.
                if (builder.getError() == gl.INVALID_OPERATION) {
                    throw new Error('Unsupported number of samples.');
                }
                var att = builder.makeConfig(glsFboUtil.RenderbufferAttachment);
                att.imageName = rbo;
                builder.glAttach(target, att);
            }
        }

        return true;
    };

    es3fFboCompletenessTests.init = function() {

        //(testCtx, renderCtx, factory) {
        var fboCtx = new glsFboCompletenessTests.Context(null, gl, function() {
            return new es3fFboCompletenessTests.ES3Checker();
        });

        fboCtx.addFormats(glsFboUtil.rangeArray(es3fFboCompletenessTests.s_es3Formats));

        /** @const @type {tcuTestCase.DeqpTest} */
        var testGroup = tcuTestCase.runner.testCases;

        testGroup.addChild(fboCtx.createRenderableTests(gl));
        testGroup.addChild(fboCtx.createAttachmentTests(gl));
        testGroup.addChild(fboCtx.createSizeTests(gl));

        /** @type {tcuTestCase.DeqpTest} */
        var layerTests = tcuTestCase.newTest('layer', 'Tests for layer attachments');

        /** @static */
        var s_layersParams = [
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_2D_ARRAY, 1, 0),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_2D_ARRAY, 1, 3),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_2D_ARRAY, 4, 3),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_2D_ARRAY, 4, 15),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_3D, 1, 0),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_3D, 1, 15),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_3D, 4, 15),
            es3fFboCompletenessTests.numLayersParams(gl.TEXTURE_3D, 64, 15)
        ];

        for (var i = 0; i < s_layersParams.length; ++i) {
            var name = 'name';
            var desc = 'desc';
            layerTests.addChild(new es3fFboCompletenessTests.NumLayersTest(
                es3fFboCompletenessTests.numLayersParams.getName(s_layersParams[i]),
                es3fFboCompletenessTests.numLayersParams.getDescription(s_layersParams[i]),
                fboCtx, s_layersParams[i]
            ));
        }
        testGroup.addChild(layerTests);

        /** @type {tcuTestCase.DeqpTest} */
        var sampleTests = tcuTestCase.newTest('sample', 'Tests for multisample attachments');
        // some short hand
        var samples = es3fFboCompletenessTests.e_samples;
        // sample tests
        /** @static */
        var s_samplesParams = [
            es3fFboCompletenessTests.numSamplesParams(0, samples.NONE, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(1, samples.NONE, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(2, samples.NONE, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(0, samples.TEXTURE, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(1, samples.TEXTURE, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(2, samples.TEXTURE, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(2, 1, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(2, 2, samples.NONE),
            es3fFboCompletenessTests.numSamplesParams(0, 0, samples.TEXTURE),
            es3fFboCompletenessTests.numSamplesParams(1, 2, 0),
            es3fFboCompletenessTests.numSamplesParams(2, 2, 0),
            es3fFboCompletenessTests.numSamplesParams(1, 1, 1),
            es3fFboCompletenessTests.numSamplesParams(1, 2, 4)
        ];

        for (var i = 0; i < s_samplesParams.length; ++i) {
            var name = 'name';
            var desc = 'desc';
            sampleTests.addChild(new es3fFboCompletenessTests.NumSamplesTest(
                es3fFboCompletenessTests.numSamplesParams.getName(s_samplesParams[i]),
                es3fFboCompletenessTests.numSamplesParams.getDescription(s_samplesParams[i]),
                fboCtx, s_samplesParams[i]
            ));
        }
        testGroup.addChild(sampleTests);

    };

    es3fFboCompletenessTests.run = function() {
        var testName = 'completeness';
        var testDescription = 'Completeness tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fFboCompletenessTests.init();
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }

    };

});
