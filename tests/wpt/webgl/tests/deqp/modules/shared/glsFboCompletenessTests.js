'use strict';
goog.provide('modules.shared.glsFboCompletenessTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluObjectWrapper');
goog.require('framework.opengl.gluStrUtil');
goog.require('modules.shared.glsFboUtil');

goog.scope(function() {

    var glsFboCompletenessTests = modules.shared.glsFboCompletenessTests;
    var glsFboUtil = modules.shared.glsFboUtil;
    var gluObjectWrapper = framework.opengl.gluObjectWrapper;
    var gluStrUtil = framework.opengl.gluStrUtil;
    var tcuTestCase = framework.common.tcuTestCase;

    /**
     * @param {WebGL2RenderingContext} gl
     */
    glsFboCompletenessTests.initGlDependents = function(gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid gl object');

        // The following extensions are applicable both to ES2 and ES3.
        /**
         * OES_depth_texture
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesDepthTextureFormats = [
            glsFboUtil.formatkey(gl.DEPTH_COMPONENT, gl.UNSIGNED_SHORT),
            glsFboUtil.formatkey(gl.DEPTH_COMPONENT, gl.UNSIGNED_INT)
        ];

        /**
         * OES_packed_depth_stencil
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesPackedDepthStencilSizedFormats = [
            gl.DEPTH24_STENCIL8
        ];

        /**
         * s_oesPackedDepthStencilTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesPackedDepthStencilTexFormats = [
            glsFboUtil.formatkey(gl.DEPTH_STENCIL, gl.UNSIGNED_INT_24_8)
        ];

        /**
         * OES_required_internalformat
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesRequiredInternalFormatColorFormats = [
            // Same as ES2 RBO formats, plus RGBA8 (even without OES_rgb8_rgba8)
            gl.RGB5_A1, gl.RGBA8, gl.RGBA4, gl.RGB565
        ];

        /**
         * s_oesRequiredInternalFormatDepthFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesRequiredInternalFormatDepthFormats = [
            gl.DEPTH_COMPONENT16
        ];

        /**
         * EXT_color_buffer_half_float
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extColorBufferHalfFloatFormats = [
            gl.RGBA16F, gl.RGB16F, gl.RG16F, gl.R16F
        ];

        /**
         * s_oesDepth24SizedFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesDepth24SizedFormats = [
            gl.DEPTH_COMPONENT24
        ];

        /**
         * s_oesDepth32SizedFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesDepth32SizedFormats = [
            gl['DEPTH_COMPONENT32']
        ];

        /**
         * s_oesRgb8Rgba8RboFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesRgb8Rgba8RboFormats = [
            gl.RGB8, gl.RGBA8
        ];

        /**
         * s_oesRequiredInternalFormatRgb8ColorFormat
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesRequiredInternalFormatRgb8ColorFormat = [
            gl.RGB8
        ];

        /**
         * s_extTextureType2101010RevFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extTextureType2101010RevFormats = [
            glsFboUtil.formatkey(gl.RGBA, gl.UNSIGNED_INT_2_10_10_10_REV),
            glsFboUtil.formatkey(gl.RGB, gl.UNSIGNED_INT_2_10_10_10_REV)
        ];

        /**
         * s_oesRequiredInternalFormat10bitColorFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesRequiredInternalFormat10bitColorFormats = [
            gl.RGB10_A2, gl['RGB10']
        ];

        /**
         * s_extTextureRgRboFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extTextureRgRboFormats = [
            gl.R8, gl.RG8
        ];

        /**
         * s_extTextureRgTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extTextureRgTexFormats = [
            glsFboUtil.formatkey(gl.RED, gl.UNSIGNED_BYTE),
            glsFboUtil.formatkey(gl.RG, gl.UNSIGNED_BYTE)
        ];

        /**
         * s_extTextureRgFloatTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extTextureRgFloatTexFormats = [
            glsFboUtil.formatkey(gl.RED, gl.FLOAT),
            glsFboUtil.formatkey(gl.RG, gl.FLOAT)
        ];

        /**
         * s_extTextureRgHalfFloatTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extTextureRgHalfFloatTexFormats = [
            glsFboUtil.formatkey(gl.RED, gl['HALF_FLOAT_OES']),
            glsFboUtil.formatkey(gl.RG, gl['HALF_FLOAT_OES'])
        ];

        /**
         * s_nvPackedFloatRboFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_nvPackedFloatRboFormats = [
            gl.R11F_G11F_B10F
        ];

        /**
         * s_nvPackedFloatTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_nvPackedFloatTexFormats = [
            glsFboUtil.formatkey(gl.RGB, gl.UNSIGNED_INT_10F_11F_11F_REV)
        ];

        /**
         * s_extSrgbRboFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extSrgbRboFormats = [
            gl.SRGB8_ALPHA8
        ];

        /**
         * s_extSrgbRenderableTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extSrgbRenderableTexFormats = [
            glsFboUtil.formatkey(gl['SRGB_ALPHA'], gl.UNSIGNED_BYTE)
        ];

        /**
         * s_extSrgbNonRenderableTexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_extSrgbNonRenderableTexFormats = [
            glsFboUtil.formatkey(gl.SRGB, gl.UNSIGNED_BYTE),
            gl.SRGB8
        ];

        /**
         * s_nvSrgbFormatsRboFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_nvSrgbFormatsRboFormats = [
            gl.SRGB8
        ];

        /**
         * s_nvSrgbFormatsTextureFormats
         * The extension does not actually require any unsized format
         * to be renderable. However, the renderablility of unsized
         * SRGB,UBYTE internalformat-type pair is implied.
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_nvSrgbFormatsTextureFormats = [
            gl.SRGB8,
            glsFboUtil.formatkey(gl.SRGB, gl.UNSIGNED_BYTE)
        ];

        /**
         * s_oesRgb8Rgba8TexFormats
         * @type {Array<number>}
         */
        glsFboCompletenessTests.s_oesRgb8Rgba8TexFormats = [
            glsFboUtil.formatkey(gl.RGB, gl.UNSIGNED_BYTE),
            glsFboUtil.formatkey(gl.RGBA, gl.UNSIGNED_BYTE)
        ];

        var fmt = glsFboUtil.FormatFlags;

        /**
         * s_esExtFormats
         * @type {Array<glsFboUtil.FormatExtEntry>}
         */
        glsFboCompletenessTests.s_esExtFormats = [
            new glsFboUtil.FormatExtEntry(
                'OES_depth_texture',
                fmt.REQUIRED_RENDERABLE | fmt.DEPTH_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesDepthTextureFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_packed_depth_stencil',
                fmt.REQUIRED_RENDERABLE | fmt.DEPTH_RENDERABLE | fmt.STENCIL_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesPackedDepthStencilSizedFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_packed_depth_stencil OES_required_internalformat',
                fmt.DEPTH_RENDERABLE | fmt.STENCIL_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesPackedDepthStencilTexFormats)
            ),

            // \todo [2013-12-10 lauri] Find out if OES_texture_half_float is really a
            // requirement on ES3 also. Or is color_buffer_half_float applicatble at
            // all on ES3, since there's also EXT_color_buffer_float?
            new glsFboUtil.FormatExtEntry(
                'OES_texture_half_float EXT_color_buffer_half_float',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extColorBufferHalfFloatFormats)
            ),

            // OES_required_internalformat doesn't actually specify that these are renderable,
            // since it was written against ES 1.1.
            new glsFboUtil.FormatExtEntry(
                'OES_required_internalformat',
                // Allow but don't require RGBA8 to be color-renderable if
                // OES_rgb8_rgba8 is not present.
                fmt.COLOR_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesRequiredInternalFormatColorFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_required_internalformat',
                fmt.DEPTH_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesRequiredInternalFormatDepthFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_texture_rg',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extTextureRgRboFormats)
            ),

            // These are not specified to be color-renderable, but the wording is
            // exactly as ambiguous as the wording in the ES2 spec.
            new glsFboUtil.FormatExtEntry(
                'EXT_texture_rg',
                fmt.REQUIRED_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extTextureRgTexFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_texture_rg OES_texture_float',
                fmt.REQUIRED_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extTextureRgFloatTexFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_texture_rg OES_texture_half_float',
                fmt.REQUIRED_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extTextureRgHalfFloatTexFormats)
            ),

            // Some Tegra drivers report gl.EXT_packed_float even for ES. Treat it as
            // a synonym for the NV_ version.
            new glsFboUtil.FormatExtEntry(
                'EXT_packed_float',
                fmt.REQUIRED_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_nvPackedFloatTexFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_packed_float EXT_color_buffer_half_float',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_nvPackedFloatRboFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_sRGB',
                fmt.COLOR_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extSrgbRenderableTexFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_sRGB',
                fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extSrgbNonRenderableTexFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_sRGB',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extSrgbRboFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'NV_sRGB_formats',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_nvSrgbFormatsRboFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'NV_sRGB_formats',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_nvSrgbFormatsTextureFormats)
            ),

            // In Khronos bug 7333 discussion, the consensus is that these texture
            // formats, at least, should be color-renderable. Still, that cannot be
            // found in any extension specs, so only allow it, not require it.
            new glsFboUtil.FormatExtEntry(
                'OES_rgb8_rgba8',
                fmt.COLOR_RENDERABLE | fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesRgb8Rgba8TexFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_rgb8_rgba8',
                fmt.REQUIRED_RENDERABLE | fmt.COLOR_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesRgb8Rgba8RboFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_rgb8_rgba8 OES_required_internalformat',
                fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesRequiredInternalFormatRgb8ColorFormat)
            ),

            // The depth-renderability of the depth RBO formats is not explicitly
            // spelled out, but all renderbuffer formats are meant to be renderable.
            new glsFboUtil.FormatExtEntry(
                'OES_depth24',
                fmt.REQUIRED_RENDERABLE | fmt.DEPTH_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesDepth24SizedFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_depth24 OES_required_internalformat OES_depth_texture',
                fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesDepth24SizedFormats)
            ),

            new glsFboUtil.FormatExtEntry(
                'OES_depth32',
                fmt.REQUIRED_RENDERABLE | fmt.DEPTH_RENDERABLE | fmt.RENDERBUFFER_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesDepth32SizedFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'OES_depth32 OES_required_internalformat OES_depth_texture',
                fmt.TEXTURE_VALID,
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesDepth32SizedFormats)
            ),

            new glsFboUtil.FormatExtEntry(
                'EXT_texture_type_2_10_10_10_REV',
                fmt.TEXTURE_VALID, // explicitly unrenderable
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_extTextureType2101010RevFormats)
            ),
            new glsFboUtil.FormatExtEntry(
                'EXT_texture_type_2_10_10_10_REV OES_required_internalformat',
                fmt.TEXTURE_VALID, // explicitly unrenderable
                glsFboUtil.rangeArray(glsFboCompletenessTests.s_oesRequiredInternalFormat10bitColorFormats)
            )
        ];

    }; // initGlDependents ----------------------------------------

    /**
    * @constructor
    * @param {null} testCtx
    * @param {WebGLRenderingContextBase} renderCtx
    * @param {glsFboUtil.CheckerFactory} factory
    */
    glsFboCompletenessTests.Context = function(testCtx, renderCtx, factory) {

        this.m_testCtx = testCtx;
        this.m_renderCtx = renderCtx;
        this.m_ctxFormats = new glsFboUtil.FormatDB();
        this.m_minFormats = new glsFboUtil.FormatDB();
        this.m_maxFormats = new glsFboUtil.FormatDB();
        this.m_verifier = new glsFboUtil.FboVerifier(this.m_ctxFormats, factory);
        this.m_haveMultiColorAtts = false;

        // FormatExtEntries
        var extRange = glsFboUtil.rangeArray(glsFboCompletenessTests.s_esExtFormats);
        this.addExtFormats(extRange);

    };

    // RenderContext&
    glsFboCompletenessTests.Context.prototype.getRenderContext = function() {
        return this.m_renderCtx;
    };

    // TestContext&
    glsFboCompletenessTests.Context.prototype.getTestContext = function() {
        return this.m_testCtx;
    };

    // const FboVerifier&
    glsFboCompletenessTests.Context.prototype.getVerifier = function() {
        return this.m_verifier;
    };

    // const FormatDB&
    glsFboCompletenessTests.Context.prototype.getMinFormats = function() {
        return this.m_minFormats;
    };

    // const FormatDB&
    glsFboCompletenessTests.Context.prototype.getCtxFormats = function() {
        return this.m_ctxFormats;
    };

    // bool
    glsFboCompletenessTests.Context.prototype.haveMultiColorAtts = function() {
        return this.m_haveMultiColorAtts;
    };

    glsFboCompletenessTests.Context.prototype.setHaveMulticolorAtts = function(have) {
        this.m_haveMultiColorAtts = (have == true);
    };

    glsFboCompletenessTests.Context.prototype.addFormats = function(fmtRange) {
        glsFboUtil.addFormats(this.m_minFormats, fmtRange);
        glsFboUtil.addFormats(this.m_ctxFormats, fmtRange);
        glsFboUtil.addFormats(this.m_maxFormats, fmtRange);
    };
    glsFboCompletenessTests.Context.prototype.addExtFormats = function(extRange) {
        glsFboUtil.addExtFormats(this.m_ctxFormats, extRange, this.m_renderCtx);
        glsFboUtil.addExtFormats(this.m_maxFormats, extRange, this.m_renderCtx);
    };

    glsFboCompletenessTests.Context.prototype.createRenderableTests = function(gl) {

        /** @type {tcuTestCase.DeqpTest} */
        var renderableTests = tcuTestCase.newTest('renderable', 'Tests for support of renderable image formats');
        /** @type {tcuTestCase.DeqpTest} */
        var rbRenderableTests = tcuTestCase.newTest('renderbuffer', 'Tests for renderbuffer formats');
        /** @type {tcuTestCase.DeqpTest} */
        var texRenderableTests = tcuTestCase.newTest('texture', 'Tests for texture formats');

        var attPoints = [
            [gl.DEPTH_ATTACHMENT, 'depth', 'Tests for depth attachments'],
            [gl.STENCIL_ATTACHMENT, 'stencil', 'Tests for stencil attachments'],
            [gl.COLOR_ATTACHMENT0, 'color0', 'Tests for color attachments']
        ];

        // At each attachment point, iterate through all the possible formats to
        // detect both false positives and false negatives.
        var rboFmts = this.m_maxFormats.getFormats(glsFboUtil.FormatFlags.ANY_FORMAT);
        var texFmts = this.m_maxFormats.getFormats(glsFboUtil.FormatFlags.ANY_FORMAT);

        for (var i = 0, l_attPoints = attPoints.length; i < l_attPoints; ++i) {
            var rbAttTests = tcuTestCase.newTest(attPoints[i][1], attPoints[i][2]);
            var texAttTests = tcuTestCase.newTest(attPoints[i][1], attPoints[i][2]);

            for (var j = 0, l_rboFmts = rboFmts.length; j < l_rboFmts; ++j) {
                var params = glsFboCompletenessTests.renderableParams(
                    attPoints[i][0], gl.RENDERBUFFER, rboFmts[j]
                );
                rbAttTests.addChild(
                    new glsFboCompletenessTests.RenderableTest(
                        glsFboCompletenessTests.renderableParams.getName(params),
                        glsFboCompletenessTests.renderableParams.getDescription(params),
                        this, params
                    )
                );
            }
            rbRenderableTests.addChild(rbAttTests);

            for (var j = 0, l_texFmts = texFmts.length; j < l_texFmts; ++j) {
                var params = glsFboCompletenessTests.renderableParams(
                    attPoints[i][0], gl.TEXTURE, texFmts[j]
                );
                texAttTests.addChild(
                    new glsFboCompletenessTests.RenderableTest(
                        glsFboCompletenessTests.renderableParams.getName(params),
                        glsFboCompletenessTests.renderableParams.getDescription(params),
                        this, params
                    )
                );
            }
            texRenderableTests.addChild(texAttTests);

        }
        renderableTests.addChild(rbRenderableTests);
        renderableTests.addChild(texRenderableTests);

        return renderableTests;
    };

    glsFboCompletenessTests.Context.prototype.createAttachmentTests = function(gl) {

        var attCombTests = tcuTestCase.newTest('attachment_combinations', 'Tests for attachment combinations');

        var s_bufTypes = [gl.NONE, gl.RENDERBUFFER, gl.TEXTURE];
        var ls_bufTypes = s_bufTypes.length;

        for (var col0 = 0; col0 < ls_bufTypes; ++col0)
            for (var coln = 0; coln < ls_bufTypes; ++coln)
                for (var dep = 0; dep < ls_bufTypes; ++dep)
                    for (var stc = 0; stc < ls_bufTypes; ++stc) {
                        var params = glsFboCompletenessTests.attachmentParams(
                            s_bufTypes[col0], s_bufTypes[coln], s_bufTypes[dep], s_bufTypes[stc]
                        );
                        attCombTests.addChild(new glsFboCompletenessTests.AttachmentTest(
                            glsFboCompletenessTests.attachmentParams.getName(params),
                            glsFboCompletenessTests.attachmentParams.getDescription(params),
                            this, params
                        ));
                    }
        return attCombTests;
    };

    glsFboCompletenessTests.Context.prototype.createSizeTests = function(gl) {

        var sizeTests = tcuTestCase.newTest('size', 'Tests for attachment sizes');

        sizeTests.addChild(new glsFboCompletenessTests.EmptyImageTest(
            'zero', 'Test for zero-sized image attachment', this
        ));

        return sizeTests;

    };

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    * @param {string} name
    * @param {string} desc
    * @param {Object} params
    */
    glsFboCompletenessTests.TestBase = function(name, desc, params) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_params = params;
    };
    glsFboCompletenessTests.TestBase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsFboCompletenessTests.TestBase.prototype.constructor = glsFboCompletenessTests.TestBase;

    glsFboCompletenessTests.TestBase.prototype.getContext = function() {
        return tcuTestCase.runner;
    };

    // GLenum attPoint, GLenum bufType
    /**
    * @param {number} attPoint
    * @param {number} bufType
    * @param {WebGLRenderingContextBase=} gl
    */
    glsFboCompletenessTests.TestBase.prototype.getDefaultFormat = function(attPoint, bufType, gl) {
        gl = gl || window.gl;

        if (bufType == gl.NONE) {
            return glsFboUtil.ImageFormat.none();
        }

        // Prefer a standard format, if there is one, but if not, use a format
        // provided by an extension.
        var formats = this.m_ctx.getMinFormats().getFormats(
            glsFboUtil.formatFlag(attPoint, gl) | glsFboUtil.formatFlag(bufType, gl)
        );

        if (!formats.length) {
            formats = this.m_ctx.getCtxFormats().getFormats(
                glsFboUtil.formatFlag(attPoint, gl) | glsFboUtil.formatFlag(bufType, gl)
            );
        }
        if (!formats.length) {
            throw new Error('Unsupported attachment kind for attachment point');
        }

        return formats[0];

    };

    /**
    * @param {number} bufType
    * @param {glsFboUtil.ImageFormat} format
    * @param {number} width
    * @param {number} height
    * @param {glsFboUtil.FboBuilder} builder
    * @param {WebGLRenderingContextBase=} gl
    * @return {glsFboUtil.Image}
    */
    glsFboCompletenessTests.makeImage = function(bufType, format, width, height, builder, gl) {
        gl = gl || window.gl;
        var image = 0;
        switch (bufType) {
            case gl.NONE:
                return null;
                break;
            case gl.RENDERBUFFER:
                image = /** @type {glsFboUtil.Renderbuffer}*/(builder.makeConfig(glsFboUtil.Renderbuffer));
                break;
            case gl.TEXTURE:
                image = /** @type {glsFboUtil.Texture2D}*/(builder.makeConfig(glsFboUtil.Texture2D));
                break;
            default:
                throw new Error('Impossible case');
        }
        image.internalFormat = format;
        image.width = width;
        image.height = height;
        return image;
    };

    /**
    * @param {number} bufType
    * @param {glsFboUtil.ImageFormat} format
    * @param {number} width
    * @param {number} height
    * @param {glsFboUtil.FboBuilder} builder
    * @param {WebGLRenderingContextBase=} gl
    * @return {glsFboUtil.Attachment}
    */
    glsFboCompletenessTests.makeAttachment = function(bufType, format, width, height, builder, gl) {
        gl = gl || window.gl;
        var cfg = glsFboCompletenessTests.makeImage(bufType, format, width, height, builder, gl);
        if (cfg == null) return null;

        /** @type {glsFboUtil.Attachment} */ var att = null;
        var img = 0;

        var mask = glsFboUtil.Config.s_types.RENDERBUFFER | glsFboUtil.Config.s_types.TEXTURE_2D;

        switch (cfg.type & mask) {
            case glsFboUtil.Config.s_types.RENDERBUFFER:
                img = builder.glCreateRbo(/** @type {glsFboUtil.Renderbuffer} */(cfg));
                att = /** @type {glsFboUtil.RenderbufferAttachment} */ (builder.makeConfig(glsFboUtil.RenderbufferAttachment));
                break;
            case glsFboUtil.Config.s_types.TEXTURE_2D:
                img = builder.glCreateTexture(/** @type {glsFboUtil.Texture2D} */(cfg));
                att = /** @type {glsFboUtil.TextureFlatAttachment} */ (builder.makeConfig(glsFboUtil.TextureFlatAttachment));
                att.texTarget = gl.TEXTURE_2D;
                break;
            default:
                throw new Error('Unsupported config.');
        }
        att.imageName = img;
        return att;
    };

    //GLenum target, GLenum bufType, ImageFormat format, GLsizei width, GLsizei height, FboBuilder& builder, webglctx
    /**
     * @param {number} target
     * @param {number} bufType
     * @param {glsFboUtil.ImageFormat} format
     * @param {number} width
     * @param {number} height
     * @param {glsFboUtil.FboBuilder} builder
     * @param {WebGL2RenderingContext} gl
     */
    glsFboCompletenessTests.TestBase.prototype.attachTargetToNew = function(
        target, bufType, format, width, height, builder, gl
    ) {
        var imgFmt = format;
        if (imgFmt.format == gl.NONE)
            imgFmt = this.getDefaultFormat(target, bufType, gl);
        var att = glsFboCompletenessTests.makeAttachment(bufType, imgFmt, width, height, builder, gl);
        builder.glAttach(target, att);
    };

    /**
    * @param {number} status
    * @param {WebGLRenderingContextBase=} gl
    * @return {string}
    */
    glsFboCompletenessTests.statusName = function(status, gl) {
        gl = gl || window.gl;

        var errorName = gluStrUtil.getErrorName(status);
        if (status != gl.NO_ERROR && errorName != '')
            return errorName + ' (during FBO initialization)';

        var fbStatusName = gluStrUtil.getFramebufferStatusName(status);
        if (fbStatusName != '')
            return fbStatusName;

        return 'unknown value (' + status + ')';
    };

    glsFboCompletenessTests.TestBase.prototype.iterate = function() {
        var gl = window.gl;

        var fbo = new gluObjectWrapper.Framebuffer(gl);
        var builder = new glsFboUtil.FboBuilder(fbo.get(), gl.FRAMEBUFFER, gl);
        var ret = this.build(builder, gl);
        var statuses = this.m_ctx.getVerifier().validStatusCodes(builder, gl);

        var errorCode = builder.getError();
        if (errorCode != gl.NO_ERROR) {
            bufferedLogToConsole('Received ' + gluStrUtil.getErrorName(errorCode) + ' (during FBO initialization).');
            if (statuses.isErrorCodeValid(errorCode))
                testPassed();
            else if (statuses.isErrorCodeRequired(gl.NO_ERROR))
                testFailedOptions('Excepted no error but got ' + gluStrUtil.getErrorName(errorCode), true);
            else
                testFailedOptions('Got wrong error code', true);
        } else {
            var fboStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            var validStatus = statuses.isFBOStatusValid(fboStatus);
            bufferedLogToConsole('Received ' + gluStrUtil.getFramebufferStatusName(fboStatus));
            if (!validStatus) {
                if (fboStatus == gl.FRAMEBUFFER_COMPLETE) {
                    testFailedOptions('Framebuffer checked as complete, expected incomplete', true);
                } else if (statuses.isFBOStatusRequired(gl.FRAMEBUFFER_COMPLETE)) {
                    testFailedOptions('Framebuffer checked as incomplete, expected complete', true);
                } else {
                    // An incomplete status is allowed, but not _this_ incomplete status.
                    testFailedOptions('Framebuffer checked as incomplete, but with wrong status', true);
                }
            } else if (fboStatus != gl.FRAMEBUFFER_COMPLETE && statuses.isFBOStatusValid(gl.FRAMEBUFFER_COMPLETE)) {
                testPassedOptions('Warning: framebuffer object could have checked as complete but did not.', true);
            } else {
                // pass
                testPassed();
            }
        }
        builder.deinit();

        return tcuTestCase.IterateResult.STOP;
    };

    glsFboCompletenessTests.formatName = function(format, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid GL object');

        var s = gluStrUtil.getPixelFormatName(format.format).substr(3).toLowerCase();

        if (format.unsizedType != gl.NONE)
            s += '_' + gluStrUtil.getTypeName(format.unsizedType).substr(3).toLowerCase();

        return s;
    };
    glsFboCompletenessTests.formatDesc = function(format, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid GL object');

        var s = gluStrUtil.getPixelFormatName(format.format);

        if (format.unsizedType != gl.NONE)
            s += ' with type ' + gluStrUtil.getTypeName(format.unsizedType);

        return s;
    };

    /**
    * @typedef {{attPoint: number, bufType: number, format: glsFboUtil.ImageFormat}}
    */
    glsFboCompletenessTests.renderableParamsT;

    /**
    * @param {number} attPoint
    * @param {number} bufType
    * @param {glsFboUtil.ImageFormat} format
    * @return {glsFboCompletenessTests.renderableParamsT}
    */
    glsFboCompletenessTests.renderableParams = function(attPoint, bufType, format) {
        var ret = {
            attPoint: attPoint,
            bufType: bufType,
            format: format
        };
        return ret;
    };
    /**
    * @param {glsFboCompletenessTests.renderableParamsT} params
    * @param {WebGLRenderingContextBase=} gl
    * @return {string}
    */
    glsFboCompletenessTests.renderableParams.getName = function(params, gl) {
        return glsFboCompletenessTests.formatName(params.format, gl);
    };
    /**
    * @param {glsFboCompletenessTests.renderableParamsT} params
    * @param {WebGLRenderingContextBase=} gl
    * @return {string}
    */
    glsFboCompletenessTests.renderableParams.getDescription = function(params, gl) {
        return glsFboCompletenessTests.formatDesc(params.format, gl);
    };

    /**
    * @constructor
    * @extends {glsFboCompletenessTests.TestBase}
    * @param {string} name
    * @param {string} desc
    * @param {glsFboCompletenessTests.Context} ctx
    * @param {glsFboCompletenessTests.renderableParamsT} params
    */
    glsFboCompletenessTests.RenderableTest = function(name, desc, ctx, params) {
        glsFboCompletenessTests.TestBase.call(this, name, desc, params);
        this.m_ctx = ctx;
    };
    glsFboCompletenessTests.RenderableTest.prototype = Object.create(glsFboCompletenessTests.TestBase.prototype);
    glsFboCompletenessTests.RenderableTest.prototype.constructor = glsFboCompletenessTests.RenderableTest;

    glsFboCompletenessTests.RenderableTest.prototype.build = function(builder, gl) {
        this.attachTargetToNew(this.m_params.attPoint, this.m_params.bufType, this.m_params.format, 64, 64, builder, gl);
        return true;
    };

    glsFboCompletenessTests.attTypeName = function(bufType, gl) {
        if (!(gl = gl || window.gl)) throw new Error('Invalid GL object');
        switch (bufType) {
            case gl.NONE: return 'none';
            case gl.RENDERBUFFER: return 'rbo';
            case gl.TEXTURE: return 'tex';
            default: break;
        }
        throw new Error('Impossible case');
    };

    /**
    * @typedef {{color0Kind: number, colornKind: number, depthKind: number, stencilKind: number}}
    */
    glsFboCompletenessTests.attachmentParamsT;

    /**
    * @param {number} color0Kind
    * @param {number} colornKind
    * @param {number} depthKind
    * @param {number} stencilKind
    * @return {glsFboCompletenessTests.attachmentParamsT}
    */
    glsFboCompletenessTests.attachmentParams = function(color0Kind, colornKind, depthKind, stencilKind) {
        var ret = {
            color0Kind: color0Kind,
            colornKind: colornKind,
            depthKind: depthKind,
            stencilKind: stencilKind
        };
        return ret;
    };
    /**
    * @param {glsFboCompletenessTests.attachmentParamsT} params
    * @param {WebGLRenderingContextBase=} gl
    * @return {string}
    */
    glsFboCompletenessTests.attachmentParams.getName = function(params, gl) {
        return (glsFboCompletenessTests.attTypeName(params.color0Kind, gl) + '_' +
                glsFboCompletenessTests.attTypeName(params.colornKind, gl) + '_' +
                glsFboCompletenessTests.attTypeName(params.depthKind, gl) + '_' +
                glsFboCompletenessTests.attTypeName(params.stencilKind, gl));
    };
    /**
    * @param {glsFboCompletenessTests.attachmentParamsT} params
    * @return {string}
    */
    glsFboCompletenessTests.attachmentParams.getDescription = glsFboCompletenessTests.attachmentParams.getName;

    /**
    * @constructor
    * @extends {glsFboCompletenessTests.TestBase}
    * @param {string} name
    * @param {string} desc
    * @param {glsFboCompletenessTests.Context} ctx
    * @param {glsFboCompletenessTests.attachmentParamsT} params
    */
    glsFboCompletenessTests.AttachmentTest = function(name, desc, ctx, params) {
        glsFboCompletenessTests.TestBase.call(this, name, desc, params);
        this.m_ctx = ctx;
    };
    glsFboCompletenessTests.AttachmentTest.prototype = Object.create(glsFboCompletenessTests.TestBase.prototype);
    glsFboCompletenessTests.AttachmentTest.prototype.constructor = glsFboCompletenessTests.AttachmentTest;

    glsFboCompletenessTests.AttachmentTest.prototype.makeDepthAndStencil = function(builder, gl) {

        /** @type {glsFboUtil.Attachment} */
        var att = null;

        if (this.m_params.stencilKind == this.m_params.depthKind) {
            // If there is a common stencil+depth -format, try to use a common
            // image for both attachments.
            var flags = glsFboUtil.FormatFlags.DEPTH_RENDERABLE |
                        glsFboUtil.FormatFlags.STENCIL_RENDERABLE |
                        glsFboUtil.formatFlag(this.m_params.stencilKind, gl);

            var formats = this.m_ctx.getMinFormats().getFormats(flags);
            if (formats.length) {
                var format = formats[0];
                att = glsFboCompletenessTests.makeAttachment(this.m_params.depthKind, format, 64, 64, builder, gl);
                builder.glAttach(gl.DEPTH_ATTACHMENT, att);
                builder.glAttach(gl.STENCIL_ATTACHMENT, att);
                return;
            }
        }
        // Either the kinds were separate, or a suitable format was not found.
        // Create separate images.
        this.attachTargetToNew(gl.STENCIL_ATTACHMENT, this.m_params.stencilKind,
                               glsFboUtil.ImageFormat.none(), 64, 64, builder, gl);
        this.attachTargetToNew(gl.DEPTH_ATTACHMENT, this.m_params.depthKind,
                               glsFboUtil.ImageFormat.none(), 64, 64, builder, gl);
    };

    glsFboCompletenessTests.AttachmentTest.prototype.build = function(builder, gl) {

        this.attachTargetToNew(gl.COLOR_ATTACHMENT0, this.m_params.color0Kind,
                               glsFboUtil.ImageFormat.none(), 64, 64, builder, gl);

        if (this.m_params.colornKind != gl.NONE) {
            if (this.m_ctx.haveMultiColorAtts())
                throw new Error('Multiple attachments not supported');
            var maxAttachments = gl.getParameter(gl.MAX_COLOR_ATTACHMENTS);

            for (var i = 1; i < maxAttachments; ++i) {
                this.attachTargetToNew(gl.COLOR_ATTACHMENT0 + i, this.m_params.colornKind,
                                       glsFboUtil.ImageFormat.none(), 64, 64, builder, gl);
            }
        }

        this.makeDepthAndStencil(builder, gl);

        return true;
    };

    /**
    * @constructor
    * @extends {glsFboCompletenessTests.TestBase}
    * @param {string} name
    * @param {string} desc
    * @param {glsFboCompletenessTests.Context} ctx
    */
    glsFboCompletenessTests.EmptyImageTest = function(name, desc, ctx) {
        glsFboCompletenessTests.TestBase.call(this, name, desc, null);
        this.m_ctx = ctx;
    };
    glsFboCompletenessTests.EmptyImageTest.prototype = Object.create(glsFboCompletenessTests.TestBase.prototype);
    glsFboCompletenessTests.EmptyImageTest.prototype.constructor = glsFboCompletenessTests.EmptyImageTest;

    glsFboCompletenessTests.EmptyImageTest.prototype.build = function(builder, gl) {
        this.attachTargetToNew(gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER,
                               glsFboUtil.ImageFormat.none(), 0, 0, builder, gl);
        return true;
    };

});
