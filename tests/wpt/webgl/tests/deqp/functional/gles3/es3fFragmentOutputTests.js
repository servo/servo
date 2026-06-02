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
goog.provide('functional.gles3.es3fFragmentOutputTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('functional.gles3.es3fFboTestUtil');

goog.scope(function() {

var es3fFragmentOutputTests = functional.gles3.es3fFragmentOutputTests;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var deRandom = framework.delibs.debase.deRandom;
var tcuTestCase = framework.common.tcuTestCase;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var deMath = framework.delibs.debase.deMath;
var tcuImageCompare = framework.common.tcuImageCompare;

    /** @type {WebGL2RenderingContext} */ var gl;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * es3fFragmentOutputTests.BufferSpec. Constructs the es3fFragmentOutputTests.BufferSpec object
     * @constructor
     * @param {WebGLRenderingContextBase.GLenum} format_
     * @param {number} width_
     * @param {number} height_
     * @param {number} samples_
     */
    es3fFragmentOutputTests.BufferSpec = function(format_, width_, height_, samples_) {
        this.format = format_;
        this.width = width_;
        this.height = height_;
        this.samples = samples_;
    };

    /**
     * es3fFragmentOutputTests.FragmentOutput. Constructs the es3fFragmentOutputTests.FragmentOutput object
     * @constructor
     * @param {gluShaderUtil.DataType} type_
     * @param {gluShaderUtil.precision} precision_
     * @param {number} location_
     * @param {number=} arrayLength_
     */
    es3fFragmentOutputTests.FragmentOutput = function(type_, precision_, location_, arrayLength_) {
        this.type = type_;
        this.precision = precision_;
        this.location = location_;
        this.arrayLength = arrayLength_ || 0;
    };

    /**
     * es3fFragmentOutputTests.FragmentOutputCase. Constructs the es3fFragmentOutputTests.FragmentOutputCase object
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     * @param {Array<es3fFragmentOutputTests.BufferSpec>} fboSpec
     * @param {Array<es3fFragmentOutputTests.FragmentOutput>} outputs
     * @return {Object} The currently modified object
     */
    es3fFragmentOutputTests.FragmentOutputCase = function(name, description, fboSpec, outputs) {
        tcuTestCase.DeqpTest.call(this, name, description);
        /** @type {Array<es3fFragmentOutputTests.BufferSpec>} */ this.m_fboSpec = fboSpec;
        /** @type {Array<es3fFragmentOutputTests.FragmentOutput>} */ this.m_outputs = outputs;
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {WebGLFramebuffer} */ this.m_framebuffer = null;

        /** @type {WebGLRenderbuffer} */ this.m_renderbuffer = null;
    };

    es3fFragmentOutputTests.FragmentOutputCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fFragmentOutputTests.FragmentOutputCase.prototype.constructor = es3fFragmentOutputTests.FragmentOutputCase;

    /**
     * es3fFragmentOutputTests.createProgram. Returns a ShaderProgram object
     * @param {Array<es3fFragmentOutputTests.FragmentOutput>} outputs
     * @return {gluShaderProgram.ShaderProgram} program
     */
    es3fFragmentOutputTests.createProgram = function(outputs) {

        var vtx = '';
        var frag = '';

        vtx = '#version 300 es\n' + 'in highp vec4 a_position;\n';
        frag = '#version 300 es\n';

    /** @type {es3fFragmentOutputTests.FragmentOutput} */ var output = null;
    /** @type {boolean} */ var isArray = false;
     // Input-output declarations.
        for (var outNdx = 0; outNdx < outputs.length; outNdx++) {
            output = outputs[outNdx];
            isArray = output.arrayLength > 0;
            /** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(output.type);
            /** @type {string} */ var precName = gluShaderUtil.getPrecisionName(output.precision);
            /** @type {boolean} */ var isFloat = gluShaderUtil.isDataTypeFloatOrVec(output.type);
            /** @type {string} */ var interp = isFloat ? 'smooth' : 'flat';

            if (isArray) {
                for (var elemNdx = 0; elemNdx < output.arrayLength; elemNdx++) {
                    vtx += 'in ' + precName + ' ' + typeName + ' in' + outNdx + '_' + elemNdx + ';\n' +
                    interp + ' out ' + precName + ' ' + typeName + ' var' + outNdx + '_' + elemNdx + ';\n';
                    frag += interp + ' in ' + precName + ' ' + typeName + ' var' + outNdx + '_' + elemNdx + ';\n';
                }
                frag += 'layout(location = ' + output.location + ') out ' + precName + ' ' + typeName + ' out' + outNdx + '[' + output.arrayLength + '];\n';
            } else {
                vtx += 'in ' + precName + ' ' + typeName + ' in' + outNdx + ';\n' +
                interp + ' out ' + precName + ' ' + typeName + ' var' + outNdx + ';\n';
                frag += interp + ' in ' + precName + ' ' + typeName + ' var' + outNdx + ';\n' +
                'layout(location = ' + output.location + ') out ' + precName + ' ' + typeName + ' out' + outNdx + ';\n';
            }
        }

        vtx += '\nvoid main()\n{\n';
        frag += '\nvoid main()\n{\n';

        vtx += ' gl_Position = a_position;\n';

        // Copy body
        for (var outNdx = 0; outNdx < outputs.length; outNdx++) {
            output = outputs[outNdx];
            isArray = output.arrayLength > 0;

            if (isArray) {
                for (var elemNdx = 0; elemNdx < output.arrayLength; elemNdx++) {
                    vtx += '\tvar' + outNdx + '_' + elemNdx + ' = in' + outNdx + '_' + elemNdx + ';\n';
                    frag += '\tout' + outNdx + '[' + elemNdx + '] = var' + outNdx + '_' + elemNdx + ';\n';
                }
            } else {
                vtx += '\tvar' + outNdx + ' = in' + outNdx + ';\n';
                frag += '\tout' + outNdx + ' = var' + outNdx + ';\n';
            }
        }

        vtx += '}\n';
        frag += '}\n';

        /** @type {gluShaderProgram.ShaderProgram} */
        var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vtx, frag));
        return program;
    };

    es3fFragmentOutputTests.FragmentOutputCase.prototype.init = function() {
        // Check that all attachments are supported
        for (var iter = 0; iter < this.m_fboSpec.length; ++iter) {
            if (!gluTextureUtil.isSizedFormatColorRenderable(this.m_fboSpec[iter].format))
                throw new Error('Unsupported attachment format');
        }

        DE_ASSERT(!this.m_program);
        this.m_program = es3fFragmentOutputTests.createProgram(this.m_outputs);

       // log << *m_program;
        if (!this.m_program.isOk())
            throw new Error('Compile failed. Program no created');

        /*
        // Print render target info to log.
        log << TestLog::Section("Framebuffer", "Framebuffer configuration");

        for (int ndx = 0; ndx < (int)m_fboSpec.size(); ndx++)
            log << TestLog::Message << "COLOR_ATTACHMENT" << ndx << ": "
                                    << glu::getPixelFormatStr(m_fboSpec[ndx].format) << ", "
                                    << m_fboSpec[ndx].width << "x" << m_fboSpec[ndx].height << ", "
                                    << m_fboSpec[ndx].samples << " samples"
                << TestLog::EndMessage;

        log << TestLog::EndSection;*/

        // Create framebuffer.
        this.m_framebuffer = gl.createFramebuffer();
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);

        for (var bufNdx = 0; bufNdx < /* m_renderbuffers.size() */ this.m_fboSpec.length; bufNdx++) {
            this.m_renderbuffer = gl.createRenderbuffer();
            /** @type {es3fFragmentOutputTests.BufferSpec} */ var bufSpec = this.m_fboSpec[bufNdx];
            /** @type {number} */ var attachment = gl.COLOR_ATTACHMENT0 + bufNdx;

            gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_renderbuffer);

            gl.renderbufferStorageMultisample(gl.RENDERBUFFER, bufSpec.samples, bufSpec.format, bufSpec.width, bufSpec.height);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, attachment, gl.RENDERBUFFER, this.m_renderbuffer);
        }
        /** @type {number} */ var fboStatus = gl.checkFramebufferStatus(gl.FRAMEBUFFER);

        if (fboStatus == gl.FRAMEBUFFER_UNSUPPORTED)
            throw new Error('Framebuffer not supported');
        else if (fboStatus != gl.FRAMEBUFFER_COMPLETE)
            throw new Error('Incomplete framebuffer');
            // throw tcu::TestError((string("Incomplete framebuffer: ") + glu::getFramebufferStatusStr(fboStatus), "", __FILE__, __LINE__);

        // gl.bindRenderbuffer(gl.RENDERBUFFER, null); // TODO: maybe needed?
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    };

    es3fFragmentOutputTests.FragmentOutputCase.prototype.deinit = function() {
        // TODO: implement?
    };

    /**
     * es3fFragmentOutputTests.getMinSize.
     * @param {Array<es3fFragmentOutputTests.BufferSpec>} fboSpec
     * @return {Array<number>} minSize
     */
    es3fFragmentOutputTests.getMinSize = function(fboSpec) {
        /** @type {Array<number>} */ var minSize = [0x7fffffff, 0x7fffffff];
        for (var i = 0; i < fboSpec.length; i++) {
            minSize[0] = Math.min(minSize[0], fboSpec[i].width);
            minSize[1] = Math.min(minSize[1], fboSpec[i].height);
        }
        return minSize;
    };

    /**
     * es3fFragmentOutputTests.getNumInputVectors. Returns the length of the array of all the outputs (es3fFragmentOutputTests.FragmentOutput object)
     * @param {Array<es3fFragmentOutputTests.FragmentOutput>} outputs
     * @return {number} numVecs
     */
    es3fFragmentOutputTests.getNumInputVectors = function(outputs) {
        /** @type {number} */ var numVecs = 0;
        for (var i = 0; i < outputs.length; i++)
            numVecs += (outputs[i].arrayLength > 0 ? outputs[i].arrayLength : 1);
        return numVecs;
    };

    /**
     * es3fFragmentOutputTests.getFloatRange
     * @param {gluShaderUtil.precision} precision
     * @return {Array<number>} Vec2
     */
    es3fFragmentOutputTests.getFloatRange = function(precision) {
        /** @type {Array<Array<number>>} */
        var ranges = // Vec2
        [
            [-2.0, 2.0],
            [-16000.0, 16000.0],
            [-1e35, 1e35]
        ];
        // DE_STATIC_ASSERT(DE_LENGTH_OF_ARRAY(ranges) == glu::PRECISION_LAST);
        // DE_ASSERT(de::inBounds<int>(precision, 0, DE_LENGTH_OF_ARRAY(ranges)));
        return ranges[precision];
    };

    /**
     * es3fFragmentOutputTests.getIntRange
     * @param {gluShaderUtil.precision} precision
     * @return {Array<number>} IVec2
     */
    es3fFragmentOutputTests.getIntRange = function(precision) {
        /** @type {Array<Array<number>>} */
        var ranges = // IVec2
        [
            [-(1 << 7), (1 << 7) - 1],
            [-(1 << 15), (1 << 15) - 1],
            [-0x80000000, 0x7fffffff]
        ];
        // DE_STATIC_ASSERT(DE_LENGTH_OF_ARRAY(ranges) == glu::PRECISION_LAST);
        // DE_ASSERT(de::inBounds<int>(precision, 0, DE_LENGTH_OF_ARRAY(ranges)));
        return ranges[precision];
    };

    /**
     * es3fFragmentOutputTests.getUintRange
     * @param {gluShaderUtil.precision} precision
     * @return {Array<number>} UVec2
     */
    es3fFragmentOutputTests.getUintRange = function(precision) {
        /** @type {Array<Array<number>>} */
        var ranges = // UVec2
        [
            [0, (1 << 8) - 1],
            [0, (1 << 16) - 1],
            [0, 0xffffffff]
        ];
        // DE_STATIC_ASSERT(DE_LENGTH_OF_ARRAY(ranges) == glu::PRECISION_LAST);
        // DE_ASSERT(de::inBounds<int>(precision, 0, DE_LENGTH_OF_ARRAY(ranges)));
        return ranges[precision];

    };

    /**
     * es3fFragmentOutputTests.readVec4
     * @param {Array<number>} ptr
     * @param {number} index
     * @param {number} numComponents
     * @return {Array<number>} Vec4
     */
    es3fFragmentOutputTests.readVec4 = function(ptr, index, numComponents) {
        DE_ASSERT(numComponents >= 1);
        return [
                ptr[index + 0],
                numComponents >= 2 ? ptr[index + 1] : 0.0,
                numComponents >= 3 ? ptr[index + 2] : 0.0,
                numComponents >= 4 ? ptr[index + 3] : 0.0
                ];
    };

    /**
     * es3fFragmentOutputTests.readIVec4
     * @param {Array<number>} ptr
     * @param {number} numComponents
     * @return {Array<number>} IVec4
     */
    es3fFragmentOutputTests.readIVec4 = function(ptr, index, numComponents) {
        DE_ASSERT(numComponents >= 1);
        return [
                ptr[index + 0],
                numComponents >= 2 ? ptr[index + 1] : 0,
                numComponents >= 3 ? ptr[index + 2] : 0,
                numComponents >= 4 ? ptr[index + 3] : 0
                ];
    };

    /**
     * es3fFragmentOutputTests.renderFloatReference
     * @param {tcuTexture.PixelBufferAccess} dst
     * @param {number} gridWidth
     * @param {number} gridHeight
     * @param {number} numComponents
     * @param {Array<number>} vertices
     */
    es3fFragmentOutputTests.renderFloatReference = function(dst, gridWidth, gridHeight, numComponents, vertices) {

        /** @type {boolean} */ var isSRGB = dst.getFormat().order == tcuTexture.ChannelOrder.sRGB || dst.getFormat().order == tcuTexture.ChannelOrder.sRGBA;
        /** @type {number} */ var cellW = dst.getWidth() / (gridWidth - 1);
        /** @type {number} */ var cellH = dst.getHeight() / (gridHeight - 1);

        for (var y = 0; y < dst.getHeight(); y++) {
            for (var x = 0; x < dst.getWidth(); x++) {
                /** @type {number} */ var cellX = deMath.clamp(Math.floor(x / cellW), 0, gridWidth - 2);
                /** @type {number} */ var cellY = deMath.clamp(Math.floor(y / cellH), 0, gridHeight - 2);
                /** @type {number} */ var xf = (x - cellX * cellW + 0.5) / cellW;
                /** @type {number} */ var yf = (y - cellY * cellH + 0.5) / cellH;

                /** @type {Array<number>} */ var v00 = es3fFragmentOutputTests.readVec4(vertices, ((cellY + 0) * gridWidth + cellX + 0) * numComponents, numComponents); // Vec4
                /** @type {Array<number>} */ var v01 = es3fFragmentOutputTests.readVec4(vertices, ((cellY + 1) * gridWidth + cellX + 0) * numComponents, numComponents); // Vec4
                /** @type {Array<number>} */ var v10 = es3fFragmentOutputTests.readVec4(vertices, ((cellY + 0) * gridWidth + cellX + 1) * numComponents, numComponents); // Vec4
                /** @type {Array<number>} */ var v11 = es3fFragmentOutputTests.readVec4(vertices, ((cellY + 1) * gridWidth + cellX + 1) * numComponents, numComponents); // Vec4

                /** @type {boolean} */ var tri = xf + yf >= 1.0;
                /** @type {Array<number>} */ var v0 = tri ? v11 : v00; // Vec4&
                /** @type {Array<number>} */ var v1 = tri ? v01 : v10; // Vec4&
                /** @type {Array<number>} */ var v2 = tri ? v10 : v01; // Vec4&
                /** @type {number} */ var s = tri ? 1.0 - xf : xf;
                /** @type {number} */ var t = tri ? 1.0 - yf : yf;
                /** @type {Array<number>} */ var color = deMath.add(v0, deMath.add(deMath.multiply((deMath.subtract(v1, v0)), [s, s, s, s]), deMath.multiply((deMath.subtract(v2, v0)), [t, t, t, t]))); // Vec4

                dst.setPixel(isSRGB ? tcuTextureUtil.linearToSRGB(color) : color, x, y);
            }
        }
    };

    /**
     * es3fFragmentOutputTests.renderIntReference
     * @param {tcuTexture.PixelBufferAccess} dst
     * @param {number} gridWidth
     * @param {number} gridHeight
     * @param {number} numComponents
     * @param {Array<number>} vertices
     */
    es3fFragmentOutputTests.renderIntReference = function(dst, gridWidth, gridHeight, numComponents, vertices) {

        /** @type {number} */ var cellW = dst.getWidth() / (gridWidth - 1);
        /** @type {number} */ var cellH = dst.getHeight() / (gridHeight - 1);

        for (var y = 0; y < dst.getHeight(); y++) {
            for (var x = 0; x < dst.getWidth(); x++) {
                /** @type {number} */ var cellX = deMath.clamp(Math.floor(x / cellW), 0, gridWidth - 2);
                /** @type {number} */ var cellY = deMath.clamp(Math.floor(y / cellH), 0, gridHeight - 2);
                /** @type {Array<number>} */ var c = es3fFragmentOutputTests.readIVec4(vertices, (cellY * gridWidth + cellX + 1) * numComponents, numComponents); // IVec4

                dst.setPixelInt(c, x, y);
            }
        }
    };

    /**
     * es3fFragmentOutputTests.s_swizzles
     * @return {Array<Array<number>>}
     */
    es3fFragmentOutputTests.s_swizzles = function() {
        var mat_swizzles = [
            [0, 1, 2, 3],
            [1, 2, 3, 0],
            [2, 3, 0, 1],
            [3, 0, 1, 2],
            [3, 2, 1, 0],
            [2, 1, 0, 3],
            [1, 0, 3, 2],
            [0, 3, 2, 1]
        ];

        return mat_swizzles;
    };

    /**
     * es3fFragmentOutputTests.swizzleVec. Returns an Array from a position contained in the Array es3fFragmentOutputTests.s_swizzles []
     * @param {Array<number>} vec
     * @param {number} swzNdx
     * @return {Array<number>} Swizzled array
     */
    es3fFragmentOutputTests.swizzleVec = function(vec, swzNdx) {
    /** @type {Array<number>} */ var swz = es3fFragmentOutputTests.s_swizzles()[swzNdx % es3fFragmentOutputTests.s_swizzles().length];

        return deMath.swizzle(vec, swz);
    };

    /**
     * es3fFragmentOutputTests.AttachmentData struct class
     * @constructor
     * @return {Object}
     */
    es3fFragmentOutputTests.AttachmentData = function() {
        return {
        /** @type {tcuTexture.TextureFormat} */ format: null, //!< Actual format of attachment.
        /** @type {tcuTexture.TextureFormat} */ referenceFormat: null, //!< Used for reference rendering.
        /** @type {tcuTexture.TextureFormat} */ readFormat: null,
        /** @type {number} */ numWrittenChannels: 0,
        /** @type {gluShaderUtil.precision} */ outPrecision: gluShaderUtil.precision.PRECISION_LOWP,
        /** @type {ArrayBuffer} */ renderedData: null,
        /** @type {ArrayBuffer} */ referenceData: null
        };
    };

    es3fFragmentOutputTests.FragmentOutputCase.prototype.iterate = function() {
        // Compute grid size & index list.
        /** @type {number} */ var minCellSize = 8;
        /** @type {Array<number>} */ var minBufSize = es3fFragmentOutputTests.getMinSize(this.m_fboSpec); // IVec2
        /** @type {number} */ var gridWidth = deMath.clamp(Math.floor(minBufSize[0] / minCellSize), 1, 255) + 1;
        /** @type {number} */ var gridHeight = deMath.clamp(Math.floor(minBufSize[1] / minCellSize), 1, 255) + 1;
        /** @type {number} */ var numVertices = gridWidth * gridHeight;
        /** @type {number} */ var numQuads = (gridWidth - 1) * (gridHeight - 1);
        /** @type {number} */ var numIndices = numQuads * 6;

        /** @type {number} */ var numInputVecs = es3fFragmentOutputTests.getNumInputVectors(this.m_outputs);
        /** @type {Array<Array<number>>} */ var inputs = []; // originally vector<vector<deUint32>

        for (var inputNdx = 0; inputNdx < numInputVecs; inputNdx++)
            inputs[inputNdx] = []; // inputs.length = numInputVecs;

        /** @type {Array<number>} */ var positions = []; // originally vector<float>
        /** @type {Array<number>} */ var indices = []; // originally vector<deUint16>

        /** @type {number} */ var readAlignment = 4;
        /** @type {number} */ var viewportW = minBufSize[0];
        /** @type {number} */ var viewportH = minBufSize[1];
        /** @type {number} */ var numAttachments = this.m_fboSpec.length;

        /** @type {Array<number>} */ var drawBuffers = []; // originally vector<deUint32>
        /** @type {Array<es3fFragmentOutputTests.AttachmentData>} */ var attachments = [];
        /** @type {number} */ var attachmentW;
        /** @type {number} */ var attachmentH;

        // Initialize attachment data.
        for (var ndx = 0; ndx < numAttachments; ndx++) {
            /** @type {tcuTexture.TextureFormat} */ var texFmt = gluTextureUtil.mapGLInternalFormat(this.m_fboSpec[ndx].format);
            /** @type {tcuTexture.TextureChannelClass} */ var chnClass = tcuTexture.getTextureChannelClass(texFmt.type);
            /** @type {boolean} */ var isFixedPoint = (chnClass == tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT ||
                                                              chnClass == tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT);

            // \note Fixed-point formats use float reference to enable more accurate result verification.
            /** @type {tcuTexture.TextureFormat} */ var refFmt = isFixedPoint ? new tcuTexture.TextureFormat(texFmt.order, tcuTexture.ChannelType.FLOAT) : texFmt;
            /** @type {tcuTexture.TextureFormat} */ var readFmt = es3fFboTestUtil.getFramebufferReadFormat(texFmt);
            attachmentW = this.m_fboSpec[ndx].width;
            attachmentH = this.m_fboSpec[ndx].height;

            drawBuffers[ndx] = gl.COLOR_ATTACHMENT0 + ndx;
            attachments[ndx] = new es3fFragmentOutputTests.AttachmentData();
            attachments[ndx].format = texFmt;
            attachments[ndx].readFormat = readFmt;
            attachments[ndx].referenceFormat = refFmt;
            attachments[ndx].renderedData = new ArrayBuffer(readFmt.getPixelSize() * attachmentW * attachmentH);
            attachments[ndx].referenceData = new ArrayBuffer(refFmt.getPixelSize() * attachmentW * attachmentH);
        }

        // Initialize indices.
        for (var quadNdx = 0; quadNdx < numQuads; quadNdx++) {
            /** @type {number} */ var quadY = Math.floor(quadNdx / (gridWidth - 1));
            /** @type {number} */ var quadX = quadNdx - quadY * (gridWidth - 1);

            indices[quadNdx * 6 + 0] = quadX + quadY * gridWidth;
            indices[quadNdx * 6 + 1] = quadX + (quadY + 1) * gridWidth;
            indices[quadNdx * 6 + 2] = quadX + quadY * gridWidth + 1;
            indices[quadNdx * 6 + 3] = indices[quadNdx * 6 + 1];
            indices[quadNdx * 6 + 4] = quadX + (quadY + 1) * gridWidth + 1;
            indices[quadNdx * 6 + 5] = indices[quadNdx * 6 + 2];
        }

        /** @type {number} */ var xf = 0;
        /** @type {number} */ var yf = 0;
        for (var y = 0; y < gridHeight; y++) {
            for (var x = 0; x < gridWidth; x++) {
                xf = x / (gridWidth - 1);
                yf = y / (gridHeight - 1);

                positions[(y * gridWidth + x) * 4 + 0] = 2.0 * xf - 1.0;
                positions[(y * gridWidth + x) * 4 + 1] = 2.0 * yf - 1.0;
                positions[(y * gridWidth + x) * 4 + 2] = 0.0;
                positions[(y * gridWidth + x) * 4 + 3] = 1.0;
            }
        }
        /** @type {es3fFragmentOutputTests.FragmentOutput} */ var output;
        /** @type {boolean} */ var isArray;
        /** @type {boolean} */ var isFloat;
        /** @type {boolean} */ var isInt;
        /** @type {boolean} */ var isUint;
        /** @type {number} */ var numVecs;
        /** @type {number} */ var numScalars;

        var curInVec = 0;
        for (var outputNdx = 0; outputNdx < this.m_outputs.length; outputNdx++) {
            output = this.m_outputs[outputNdx];
            isFloat = gluShaderUtil.isDataTypeFloatOrVec(output.type);
            isInt = gluShaderUtil.isDataTypeIntOrIVec(output.type);
            isUint = gluShaderUtil.isDataTypeUintOrUVec(output.type);
            numVecs = output.arrayLength > 0 ? output.arrayLength : 1;
            numScalars = gluShaderUtil.getDataTypeScalarSize(output.type);

            for (var vecNdx = 0; vecNdx < numVecs; vecNdx++) {
                inputs[curInVec].length = numVertices * numScalars;

                // Record how many outputs are written in attachment.
                DE_ASSERT(output.location + vecNdx < attachments.length);
                attachments[output.location + vecNdx].numWrittenChannels = numScalars;
                attachments[output.location + vecNdx].outPrecision = output.precision;

                /** @type {Array<number>} */ var range = null;
                /** @type {Array<number>} */ var minVal = null;
                /** @type {Array<number>} */ var maxVal = null;
                /** @type {Array<number>} */ var fmtBits = null;
                /** @type {Array<number>} */ var fmtMaxVal = [];
                /** @type {Array<number>} */ var rangeDiv = null;
                /** @type {Array<number>} */ var step = [];
                /** @type {number} */ var ix = 0;
                /** @type {number} */ var iy = 0;
                /** @type {Array<number>} */ var c = null;
                /** @type {number} */ var pos = 0;
               if (isFloat) {
                    range = es3fFragmentOutputTests.getFloatRange(output.precision); // Vec2
                    minVal = [range[0], range[0], range[0], range[0]]; // Vec4
                    maxVal = [range[1], range[1], range[1], range[1]]; // Vec4

                    if (deMath.deInBounds32(output.location + vecNdx, 0, attachments.length)) {
                    // \note Floating-point precision conversion is not well-defined. For that reason we must
                    // limit value range to intersection of both data type and render target value ranges.
                    /** @type {tcuTextureUtil.TextureFormatInfo} */ var fmtInfo = tcuTextureUtil.getTextureFormatInfo(attachments[output.location + vecNdx].format);
                        minVal = deMath.max(minVal, fmtInfo.valueMin);
                        maxVal = deMath.min(maxVal, fmtInfo.valueMax);
                    }

                    bufferedLogToConsole('out ' + curInVec + ' value range: ' + minVal + ' -> ' + maxVal);

                    for (var y = 0; y < gridHeight; y++) {
                        for (var x = 0; x < gridWidth; x++) {
                            xf = x / (gridWidth - 1);
                            yf = y / (gridHeight - 1);
                            /** @type {number} */ var f0 = (xf + yf) * 0.5;
                            /** @type {number} */ var f1 = 0.5 + (xf - yf) * 0.5;

                            /** @type {Array<number>} */ var f = es3fFragmentOutputTests.swizzleVec([f0, f1, 1.0 - f0, 1.0 - f1], curInVec); // Vec4
                            c = deMath.add(minVal, deMath.multiply(deMath.subtract(maxVal, minVal), f)); // Vec4

                            pos = (y * gridWidth + x) * numScalars;

                            for (var ndx = 0; ndx < numScalars; ndx++)
                                inputs[curInVec][pos + ndx] = c[ndx];
                        }
                    }
                } else if (isInt) {
                    range = es3fFragmentOutputTests.getIntRange(output.precision); // IVec2
                    minVal = [range[0], range[0], range[0], range[0]]; // IVec4
                    maxVal = [range[1], range[1], range[1], range[1]]; // IVec4

                    if (deMath.deInBounds32(output.location + vecNdx, 0, attachments.length)) {
                        // Limit to range of output format as conversion mode is not specified.
                        fmtBits = tcuTextureUtil.getTextureFormatBitDepth(attachments[output.location + vecNdx].format); // IVec4
                        /** @type {Array<boolean>} */ var isZero = deMath.lessThanEqual(fmtBits, [0, 0, 0, 0]); // BVec4, array of booleans, size = 4

                        /** @type {Array<number>} */ var fmtMinVal = []; // IVec4

                        for (var i = 0; i < 4; i++) {

                            // const IVec4 fmtMinVal = (-(tcu::Vector<deInt64, 4>(1) << (fmtBits - 1 ).cast<deInt64>())).asInt();
                            fmtMinVal[i] = -1 * Math.pow(2, fmtBits[i] - 1); // TODO: check implementation, original above
                            // const IVec4 fmtMaxVal = ((tcu::Vector<deInt64, 4>(1) << (fmtBits - 1 ).cast<deInt64>()) - deInt64(1)).asInt();
                            fmtMaxVal[i] = Math.pow(2, fmtBits[i] - 1) - 1; // TODO: check implementation, original above
                        }

                        minVal = tcuTextureUtil.select(minVal, deMath.max(minVal, fmtMinVal), isZero);
                        maxVal = tcuTextureUtil.select(maxVal, deMath.min(maxVal, fmtMaxVal), isZero);
                    }

                    bufferedLogToConsole('out ' + curInVec + ' value range: ' + minVal + ' -> ' + maxVal);

                    rangeDiv = es3fFragmentOutputTests.swizzleVec([gridWidth - 1, gridHeight - 1, gridWidth - 1, gridHeight - 1], curInVec); // IVec4
                    for (var i = 0; i < 4; i++) {
                        // const IVec4 step = ((maxVal.cast<deInt64>() - minVal.cast<deInt64>()) / (rangeDiv.cast<deInt64>())).asInt();
                        step[i] = Math.floor((maxVal[i] - minVal[i]) / rangeDiv[i]); // TODO: check with the above line of code
                    }

                    for (var y = 0; y < gridHeight; y++) {
                        for (var x = 0; x < gridWidth; x++) {
                            ix = gridWidth - x - 1;
                            iy = gridHeight - y - 1;
                            c = deMath.add(minVal, deMath.multiply(step, es3fFragmentOutputTests.swizzleVec([x, y, ix, iy], curInVec))); // IVec4

                            pos = (y * gridWidth + x) * numScalars;

                            for (var ndx = 0; ndx < numScalars; ndx++)
                                inputs[curInVec][pos + ndx] = c[ndx];
                        }
                    }
                } else if (isUint) {
                    range = es3fFragmentOutputTests.getUintRange(output.precision); // UVec2
                    maxVal = [range[1], range[1], range[1], range[1]]; // UVec4

                    if (deMath.deInBounds32(output.location + vecNdx, 0, attachments.length)) {
                        // Limit to range of output format as conversion mode is not specified.
                        fmtBits = tcuTextureUtil.getTextureFormatBitDepth(attachments[output.location + vecNdx].format); // IVec4

                        for (var i = 0; i < 4; i++) {
                            fmtMaxVal[i] = Math.pow(2, fmtBits[i]) - 1;
                        }

                        maxVal = deMath.min(maxVal, fmtMaxVal);
                    }

                    bufferedLogToConsole('out ' + curInVec + ' value range: ' + minVal + ' -> ' + maxVal);

                    rangeDiv = es3fFragmentOutputTests.swizzleVec([gridWidth - 1, gridHeight - 1, gridWidth - 1, gridHeight - 1], curInVec); // IVec4

                    for (var stepPos = 0; stepPos < maxVal.length; stepPos++) {
                        step[stepPos] = Math.floor(maxVal[stepPos] / rangeDiv[stepPos]);
                    }

                    DE_ASSERT(range[0] == 0);

                    for (var y = 0; y < gridHeight; y++) {
                        for (var x = 0; x < gridWidth; x++) {
                            ix = gridWidth - x - 1;
                            iy = gridHeight - y - 1;
                            c = deMath.multiply(step, es3fFragmentOutputTests.swizzleVec([x, y, ix, iy], curInVec)); // UVec4
                            pos = (y * gridWidth + x) * numScalars;

                            DE_ASSERT(deMath.boolAll(deMath.lessThanEqual(c, maxVal))); // TODO: sometimes crashes here, condition not asserted

                            for (var ndx = 0; ndx < numScalars; ndx++)
                                inputs[curInVec][pos + ndx] = c[ndx];
                        }
                    }
                } else
                    DE_ASSERT(false);

                curInVec += 1;
            }
        }

        // Render using gl.
        gl.useProgram(this.m_program.getProgram());
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_framebuffer);
        gl.viewport(0, 0, viewportW, viewportH);
        gl.drawBuffers(drawBuffers);
        gl.disable(gl.DITHER); // Dithering causes issues with unorm formats. Those issues could be worked around in threshold, but it makes validation less accurate.

        /** @type {WebGLBuffer} */ var buffer = null;
        /** @type {string} */ var name;
        curInVec = 0;
        for (var outputNdx = 0; outputNdx < this.m_outputs.length; outputNdx++) {
            output = this.m_outputs[outputNdx];
            isArray = output.arrayLength > 0;
            isFloat = gluShaderUtil.isDataTypeFloatOrVec(output.type);
            isInt = gluShaderUtil.isDataTypeIntOrIVec(output.type);
            isUint = gluShaderUtil.isDataTypeUintOrUVec(output.type);
            /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(output.type);
            /** @type {number} */ var glScalarType = isFloat ? /* gluShaderUtil.DataType.FLOAT */ gl.FLOAT :
                                                     isInt ? /* gluShaderUtil.DataType.INT */ gl.INT :
                                                     isUint ? /* gluShaderUtil.DataType.UINT */ gl.UNSIGNED_INT : /* gluShaderUtil.DataType.INVALID */ gl.NONE;
            numVecs = isArray ? output.arrayLength : 1;

            for (var vecNdx = 0; vecNdx < numVecs; vecNdx++) {
                name = 'in' + outputNdx + (isArray ? '_' + vecNdx : '');
                /** @type {number} */ var loc = gl.getAttribLocation(this.m_program.getProgram(), name);

                if (loc >= 0) {
                    buffer = gl.createBuffer();
                    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

                    gl.enableVertexAttribArray(loc);
                    if (isFloat) {
                        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(inputs[curInVec]), gl.STATIC_DRAW);
                        // KHRONOS WebGL 1.0 specification:
                        // void vertexAttribPointer(GLuint indx, GLint size, GLenum type, GLboolean normalized, GLsizei stride, GLintptr offset);
                        gl.vertexAttribPointer(loc, scalarSize, glScalarType, false, 0, 0); // offset = 0
                    } else {
                        gl.bufferData(gl.ARRAY_BUFFER, new Int32Array(inputs[curInVec]), gl.STATIC_DRAW);
                        // KHRONOS WebGL 2.0 specification:
                        // void vertexAttribIPointer(GLuint index, GLint size, GLenum type, GLsizei stride, GLintptr offset)
                        gl.vertexAttribIPointer(loc, scalarSize, glScalarType, 0, 0); // offset = 0
                    }
                } else
                    bufferedLogToConsole('Warning: No location for attribute "' + name + '" found.');

                curInVec += 1;
            }
        }

        /** @type {number} */ var posLoc = gl.getAttribLocation(this.m_program.getProgram(), 'a_position');
        // TCU_CHECK(posLoc >= 0);
        buffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);

        gl.enableVertexAttribArray(posLoc);
        gl.vertexAttribPointer(posLoc, 4, gl.FLOAT, false, 0, 0); // offset = 0

        /** @type {WebGLBuffer} */ var indexObject = gl.createBuffer();
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexObject);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);

        gl.drawElements(gl.TRIANGLES, numIndices, gl.UNSIGNED_SHORT, 0); // offset = 0

        // Render reference images.

        var curInNdx = 0;
        for (var outputNdx = 0; outputNdx < this.m_outputs.length; outputNdx++) {
            output = this.m_outputs[outputNdx];
            isArray = output.arrayLength > 0;
            isFloat = gluShaderUtil.isDataTypeFloatOrVec(output.type);
            isInt = gluShaderUtil.isDataTypeIntOrIVec(output.type);
            isUint = gluShaderUtil.isDataTypeUintOrUVec(output.type);
            scalarSize = gluShaderUtil.getDataTypeScalarSize(output.type);
            numVecs = isArray ? output.arrayLength : 1;

            for (var vecNdx = 0; vecNdx < numVecs; vecNdx++) {
                /** @type {number} */ var location = output.location + vecNdx;
                /** @type {Array<number>} */ var inputData = inputs[curInNdx];

                DE_ASSERT(deMath.deInBounds32(location, 0, this.m_fboSpec.length));

                /** @type {number} */ var bufW = this.m_fboSpec[location].width;
                /** @type {number} */ var bufH = this.m_fboSpec[location].height;
                /** @type {Object} */ var descriptor = {
                        format: attachments[location].referenceFormat,
                        width: bufW,
                        height: bufH,
                        depth: 1,
                        data: attachments[location].referenceData // ArrayBuffer
                };
                /** @type {tcuTexture.PixelBufferAccess} */ var buf = new tcuTexture.PixelBufferAccess(descriptor);
                /** @type {tcuTexture.PixelBufferAccess} */ var viewportBuf = tcuTextureUtil.getSubregion(buf, 0, 0, 0, viewportW, viewportH, 1);

                if (isInt || isUint)
                    es3fFragmentOutputTests.renderIntReference(viewportBuf, gridWidth, gridHeight, scalarSize, inputData);
                else if (isFloat)
                    es3fFragmentOutputTests.renderFloatReference(viewportBuf, gridWidth, gridHeight, scalarSize, inputData);
                else
                    DE_ASSERT(false);

                curInNdx += 1;
            }
        }

        // Compare all images.
        /** @type {boolean} */ var allLevelsOk = true;
        for (var attachNdx = 0; attachNdx < numAttachments; attachNdx++) {
            attachmentW = this.m_fboSpec[attachNdx].width;
            attachmentH = this.m_fboSpec[attachNdx].height;
            /** @type {number} */ var numValidChannels = attachments[attachNdx].numWrittenChannels;
            /** @type {Array<boolean>} */ var cmpMask = [numValidChannels >= 1, numValidChannels >= 2, numValidChannels >= 3, numValidChannels >= 4];
            /** @type {gluShaderUtil.precision} */ var outPrecision = attachments[attachNdx].outPrecision;
            /** @type {tcuTexture.TextureFormat} */ var format = attachments[attachNdx].format;
            /** @type {Object} */
            var renderedDescriptor = {
                    format: attachments[attachNdx].readFormat,
                    width: attachmentW,
                    height: attachmentH,
                    depth: 1,
                    rowPitch: deMath.deAlign32(attachments[attachNdx].readFormat.getPixelSize() * attachmentW, readAlignment),
                    slicePitch: 0,
                    data: attachments[attachNdx].renderedData // ArrayBuffer
            };
            /** @type {tcuTexture.PixelBufferAccess} */ var rendered = new tcuTexture.PixelBufferAccess(renderedDescriptor);
            /** @type {gluTextureUtil.TransferFormat} */ var transferFmt = gluTextureUtil.getTransferFormat(attachments[attachNdx].readFormat);
            gl.readBuffer(gl.COLOR_ATTACHMENT0 + attachNdx);
            gl.readPixels(0, 0, attachmentW, attachmentH, transferFmt.format, transferFmt.dataType, rendered.getDataPtr());

            /** @type {Object} */
            var referenceDescriptor = {
                    format: attachments[attachNdx].referenceFormat,
                    width: attachmentW,
                    height: attachmentH,
                    depth: 1,
                    data: attachments[attachNdx].referenceData // ArrayBuffer
            };
            /** @type {tcuTexture.ConstPixelBufferAccess} */ var reference = new tcuTexture.ConstPixelBufferAccess(referenceDescriptor);
            /** @type {tcuTexture.TextureChannelClass} */ var texClass = tcuTexture.getTextureChannelClass(format.type);
            /** @type {boolean} */ var isOk = true;
            name = 'Attachment ' + attachNdx;
            /** @type {string} */ var desc = 'Color attachment ' + attachNdx;
            /** @type {Array<number>} */ var threshold;

            bufferedLogToConsole('Attachment ' + attachNdx + ': ' + numValidChannels + ' channels have defined values and used for comparison');

            switch (texClass) {
                case tcuTexture.TextureChannelClass.FLOATING_POINT: {
                    /** @type {Array<number>} */ var formatThreshold = []; // UVec4 //!< Threshold computed based on format.
                    formatThreshold.length = 4;
                    /** @type {number} */ var precThreshold = 0; // deUint32 //!< Threshold computed based on output type precision
                    /** @type {Array<number>} */ var finalThreshold = []; // UVec4
                    finalThreshold.length = 4;

                    switch (format.type) {
                        case tcuTexture.ChannelType.FLOAT:
                            formatThreshold = [4, 4, 4, 4]; // UVec4
                            break;
                        case tcuTexture.ChannelType.HALF_FLOAT:
                            formatThreshold = [(1 << 13) + 4, (1 << 13) + 4, (1 << 13) + 4, (1 << 13) + 4]; // UVec4
                            break;
                        case tcuTexture.ChannelType.UNSIGNED_INT_11F_11F_10F_REV:
                            formatThreshold = [(1 << 17) + 4, (1 << 17) + 4, (1 << 18) + 4, 4]; // UVec4
                            break;
                        default:
                            DE_ASSERT(false);
                            break;
                    }

                    switch (outPrecision) {
                        case gluShaderUtil.precision.PRECISION_LOWP:
                            precThreshold = (1 << 21);
                            break;
                        case gluShaderUtil.precision.PRECISION_MEDIUMP:
                            precThreshold = (1 << 13);
                            break;
                        case gluShaderUtil.precision.PRECISION_HIGHP:
                            precThreshold = 0;
                            break;
                        default:
                            DE_ASSERT(false);
                    }

                    finalThreshold = tcuTextureUtil.select(
                                    deMath.max(formatThreshold, [precThreshold, precThreshold, precThreshold, precThreshold]),
                                    [0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff], // C++ version: UVec4(~0u) bitwise not, all bits in the integer will be flipped
                                    cmpMask);

                    isOk = tcuImageCompare.floatUlpThresholdCompare(name, desc, reference, rendered, finalThreshold /*, tcu::COMPARE_LOG_RESULT*/);
                    break;
                }

                case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT: {
                    // \note glReadPixels() allows only 8 bits to be read. This means that RGB10_A2 will loose some
                    // bits in the process and it must be taken into account when computing threshold.
                    /** @type {Array<number>} */ var bits = deMath.min([8, 8, 8, 8], tcuTextureUtil.getTextureFormatBitDepth(format)); // IVec4

                    /** @type {Array<number>} */ var baseThreshold = []; // Vec4
                    baseThreshold.length = 4;
                    for (var inc = 0; inc < baseThreshold.length; inc++) {
                        // TODO: check the operation below: baseThreshold = 1.0f / ((IVec4(1) << bits)-1).asFloat();
                        baseThreshold[inc] = 1.0 / ((1 << bits[inc]) - 1);
                    }

                    threshold = tcuTextureUtil.select(baseThreshold, [2.0, 2.0, 2.0, 2.0], cmpMask); // Vec4

                    isOk = tcuImageCompare.floatThresholdCompare(name, desc, reference, rendered, threshold/*, tcu::COMPARE_LOG_RESULT*/);
                    break;
                }

                case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER: {
                    // The C++ dEQP code uses ~0u but ~0 is -1 in Javascript
                    var UINT_MAX = Math.pow(2.0, 32.0) - 1;
                    threshold = tcuTextureUtil.select(
                                    [0, 0, 0, 0],
                                    [UINT_MAX, UINT_MAX, UINT_MAX, UINT_MAX],
                                    cmpMask
                                    ); // UVec4
                    isOk = tcuImageCompare.intThresholdCompare(name, desc, reference, rendered, threshold/*, tcu::COMPARE_LOG_RESULT*/);
                    break;
                }

                default:
                    testFailedOptions('Unsupported comparison', true);
                    break;
            }

            if (!isOk)
                allLevelsOk = false;
        }

        if (numAttachments > 1) {
            if (allLevelsOk)
                testPassed('Image comparison passed for ' +  numAttachments + ' attachments');
            else
                testFailed('Image comparison failed for some of ' +  numAttachments + ' attachments');
        } else {
            if (allLevelsOk)
                testPassed('Image comparison passed');
            else
                testFailed('Image comparison failed');
        }

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * es3fFragmentOutputTests.createRandomCase. Constructs the es3fFragmentOutputTests.createRandomCase, child class of es3fFragmentOutputTests.FragmentOutputCase
     * @constructor
     * @param {number} minRenderTargets
     * @param {number} maxRenderTargets
     * @param {number} seed
     * @return {es3fFragmentOutputTests.FragmentOutputCase} The currently modified object
     */
    es3fFragmentOutputTests.createRandomCase = function(minRenderTargets, maxRenderTargets, seed, colorBufferFloatSupported) {

        /** @type {Array<gluShaderUtil.DataType>} */
        var outputTypes = [
                           gluShaderUtil.DataType.FLOAT,
                           gluShaderUtil.DataType.FLOAT_VEC2,
                           gluShaderUtil.DataType.FLOAT_VEC3,
                           gluShaderUtil.DataType.FLOAT_VEC4,
                           gluShaderUtil.DataType.INT,
                           gluShaderUtil.DataType.INT_VEC2,
                           gluShaderUtil.DataType.INT_VEC3,
                           gluShaderUtil.DataType.INT_VEC4,
                           gluShaderUtil.DataType.UINT,
                           gluShaderUtil.DataType.UINT_VEC2,
                           gluShaderUtil.DataType.UINT_VEC3,
                           gluShaderUtil.DataType.UINT_VEC4
                           ];

        /** @type {Array<gluShaderUtil.precision>} */
        var precisions = [
                          gluShaderUtil.precision.PRECISION_LOWP,
                          gluShaderUtil.precision.PRECISION_MEDIUMP,
                          gluShaderUtil.precision.PRECISION_HIGHP
                          ];

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var floatFormats = [
                            gl.RGBA32F,
                            gl.RGBA16F,
                            gl.R11F_G11F_B10F,
                            gl.RG32F,
                            gl.RG16F,
                            gl.R32F,
                            gl.R16F,
                            gl.RGBA8,
                            gl.SRGB8_ALPHA8,
                            gl.RGB10_A2,
                            gl.RGBA4,
                            gl.RGB5_A1,
                            gl.RGB8,
                            gl.RGB565,
                            gl.RG8,
                            gl.R8
                            ];

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var colorBufferFloatFormats = [
                                       gl.RGBA32F,
                                       gl.RGBA16F,
                                       gl.R11F_G11F_B10F,
                                       gl.RG32F,
                                       gl.RG16F,
                                       gl.R32F,
                                       gl.R16F
        ];


        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var intFormats = [
                            gl.RGBA32I,
                            gl.RGBA16I,
                            gl.RGBA8I,
                            gl.RG32I,
                            gl.RG16I,
                            gl.RG8I,
                            gl.R32I,
                            gl.R16I,
                            gl.R8I
                            ];

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var uintFormats = [
                           gl.RGBA32UI,
                           gl.RGBA16UI,
                           gl.RGBA8UI,
                           gl.RGB10_A2UI,
                           gl.RG32UI,
                           gl.RG16UI,
                           gl.RG8UI,
                           gl.R32UI,
                           gl.R16UI,
                           gl.R8UI
                           ];

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(seed);
        /** @type {Array<es3fFragmentOutputTests.FragmentOutput>} */ var outputs = [];
        /** @type {Array<es3fFragmentOutputTests.BufferSpec>} */ var targets = [];
        /** @type {Array<gluShaderUtil.DataType>} */ var outTypes = [];

        /** @type {number} */ var numTargets = rnd.getInt(minRenderTargets, maxRenderTargets);
        /** @type {number} */ var width = 128; // \todo [2012-04-10 pyry] Separate randomized sizes per target?
        /** @type {number} */ var height = 64;
        /** @type {number} */ var samples = 0;

        // Compute outputs.
        /** @type {number} */ var curLoc = 0;
        while (curLoc < numTargets) {
            /** @type {boolean} */ var useArray = rnd.getFloat() < 0.3;
            /** @type {number} */ var maxArrayLen = numTargets - curLoc;
            /** @type {number} */ var arrayLen = useArray ? rnd.getInt(1, maxArrayLen) : 0;
            /** @type {Array<gluShaderUtil.DataType>} */ var basicTypeArray = rnd.choose(outputTypes, undefined, 1);
            /** @type {gluShaderUtil.DataType} */ var basicType = basicTypeArray[0];
            /** @type {Array<gluShaderUtil.precision>} */ var precisionArray = rnd.choose(precisions, undefined, 1);
            /** @type {gluShaderUtil.precision} */ var precision = precisionArray[0];
            /** @type {number} */ var numLocations = useArray ? arrayLen : 1;

            outputs.push(new es3fFragmentOutputTests.FragmentOutput(basicType, precision, curLoc, arrayLen));

            for (var ndx = 0; ndx < numLocations; ndx++)
                outTypes.push(basicType);

            curLoc += numLocations;
        }
        DE_ASSERT(curLoc == numTargets);
        DE_ASSERT(outTypes.length == numTargets);

        // Compute buffers.
        while (targets.length < numTargets) {
            /** @type {gluShaderUtil.DataType} */ var outType = outTypes[targets.length];
            /** @type {boolean} */ var isFloat = gluShaderUtil.isDataTypeFloatOrVec(outType);
            /** @type {boolean} */ var isInt = gluShaderUtil.isDataTypeIntOrIVec(outType);
            /** @type {boolean} */ var isUint = gluShaderUtil.isDataTypeUintOrUVec(outType);
            /** @type {Array} */ var formatArray = [];
            /** @type {number} */ var format = 0;

            if (isFloat) {
                formatArray = rnd.choose(floatFormats, undefined, 1);
                format = formatArray[0];
                if (colorBufferFloatFormats.indexOf(format) >= 0 && !colorBufferFloatSupported)
                    return null;
            } else if (isInt) {
                formatArray = rnd.choose(intFormats, undefined, 1);
                format = formatArray[0];
            } else if (isUint) {
                formatArray = rnd.choose(uintFormats, undefined, 1);
                format = formatArray[0];
            } else
                DE_ASSERT(false);

            targets.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));
        }

        return new es3fFragmentOutputTests.FragmentOutputCase(seed.toString(), '', targets, outputs);

    };

    es3fFragmentOutputTests.init = function(gl) {
        var state = tcuTestCase.runner;
        state.testCases = tcuTestCase.newTest('fragment_outputs', 'Top level');
        /** @const @type {tcuTestCase.DeqpTest} */ var testGroup = state.testCases;

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var requiredFloatFormats = [
            gl.RGBA32F,
            gl.RGBA16F,
            gl.R11F_G11F_B10F,
            gl.RG32F,
            gl.RG16F,
            gl.R32F,
            gl.R16F
        ];

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var requiredFixedFormats = [
            gl.RGBA8,
            gl.SRGB8_ALPHA8,
            gl.RGB10_A2,
            gl.RGBA4,
            gl.RGB5_A1,
            gl.RGB8,
            gl.RGB565,
            gl.RG8,
            gl.R8
        ];

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var requiredIntFormats = [
            gl.RGBA32I,
            gl.RGBA16I,
            gl.RGBA8I,
            gl.RG32I,
            gl.RG16I,
            gl.RG8I,
            gl.R32I,
            gl.R16I,
            gl.R8I
        ];

        /** @type {Array<WebGLRenderingContextBase.GLenum>} */
        var requiredUintFormats = [
            gl.RGBA32UI,
            gl.RGBA16UI,
            gl.RGBA8UI,
            gl.RGB10_A2UI,
            gl.RG32UI,
            gl.RG16UI,
            gl.RG8UI,
            gl.R32UI,
            gl.R16UI,
            gl.R8UI
        ];

        /** @type {Array<gluShaderUtil.precision>} */
        var precisions = [

            gluShaderUtil.precision.PRECISION_LOWP,
            gluShaderUtil.precision.PRECISION_MEDIUMP,
            gluShaderUtil.precision.PRECISION_HIGHP

        ];

     // .basic.

        /** @const @type {number} */ var width = 64;
        /** @const @type {number} */ var height = 64;
        /** @const @type {number} */ var samples = 0;
        /** @type {Array<es3fFragmentOutputTests.BufferSpec>} */ var fboSpec = null;
        /** @type {gluShaderUtil.precision} */ var prec;
        /** @type {string} */ var precName;

    // .float
        if (gl.getExtension('EXT_color_buffer_float')) {
            /** @type {tcuTestCase.DeqpTest} */ var floatGroup = tcuTestCase.newTest('basic.float', 'Floating-point output tests');
            testGroup.addChild(floatGroup);

            for (var fmtNdx = 0; fmtNdx < requiredFloatFormats.length; fmtNdx++) {
                var format = requiredFloatFormats[fmtNdx];
                var fmtName = es3fFboTestUtil.getFormatName(format);
                fboSpec = [];

                fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

                for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                    prec = precisions[precNdx];
                    precName = gluShaderUtil.getPrecisionName(prec);

                    // NOTE: Eliminated original OutputVec and toVec(), as it only returned an element of the outputs array in OutputVec
                    floatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_float', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT, prec, 0)]));
                    floatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC2, prec, 0)]));
                    floatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC3, prec, 0)]));
                    floatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC4, prec, 0)]));
                }
            }
        }

        // .fixed
        /** @type {tcuTestCase.DeqpTest} */ var fixedGroup = tcuTestCase.newTest('basic.fixed', 'Fixed-point output tests');
        testGroup.addChild(fixedGroup);
        for (var fmtNdx = 0; fmtNdx < requiredFixedFormats.length; fmtNdx++) {
            var format = requiredFixedFormats[fmtNdx];
            var fmtName = es3fFboTestUtil.getFormatName(format);
            fboSpec = [];

            fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                prec = precisions[precNdx];
                precName = gluShaderUtil.getPrecisionName(prec);

                fixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_float', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT, prec, 0)]));
                fixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC2, prec, 0)]));
                fixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC3, prec, 0)]));
                fixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC4, prec, 0)]));
            }
        }

        // .int
        /** @type {tcuTestCase.DeqpTest} */ var intGroup = tcuTestCase.newTest('basic.int', 'Integer output tests');
        testGroup.addChild(intGroup);
        for (var fmtNdx = 0; fmtNdx < requiredIntFormats.length; fmtNdx++) {
            var format = requiredIntFormats[fmtNdx];
            var fmtName = es3fFboTestUtil.getFormatName(format);
            fboSpec = [];

            fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                prec = precisions[precNdx];
                precName = gluShaderUtil.getPrecisionName(prec);

                intGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_int', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT, prec, 0)]));
                intGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_ivec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT_VEC2, prec, 0)]));
                intGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_ivec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT_VEC3, prec, 0)]));
                intGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_ivec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT_VEC4, prec, 0)]));
            }
        }

        // .uint
        /** @type {tcuTestCase.DeqpTest} */ var uintGroup = tcuTestCase.newTest('basic.uint', 'Usigned integer output tests');
        testGroup.addChild(uintGroup);
        for (var fmtNdx = 0; fmtNdx < requiredUintFormats.length; fmtNdx++) {
            var format = requiredUintFormats[fmtNdx];
            var fmtName = es3fFboTestUtil.getFormatName(format);
            fboSpec = [];

            fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                prec = precisions[precNdx];
                precName = gluShaderUtil.getPrecisionName(prec);

                uintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uint', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT, prec, 0)]));
                uintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uvec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT_VEC2, prec, 0)]));
                uintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uvec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT_VEC3, prec, 0)]));
                uintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uvec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT_VEC4, prec, 0)]));

            }
        }

     // .array

        /** @type {number} */ var numTargets = 3;

        // .float
        if (gl.getExtension('EXT_color_buffer_float')) {
            /** @type {tcuTestCase.DeqpTest} */ var arrayFloatGroup = tcuTestCase.newTest('array.float', 'Floating-point output tests');
            testGroup.addChild(arrayFloatGroup);
            for (var fmtNdx = 0; fmtNdx < requiredFloatFormats.length; fmtNdx++) {
                var format = requiredFloatFormats[fmtNdx];
                var fmtName = es3fFboTestUtil.getFormatName(format);
                fboSpec = [];

                for (var ndx = 0; ndx < numTargets; ndx++)
                    fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

                for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                    prec = precisions[precNdx];
                    precName = gluShaderUtil.getPrecisionName(prec);

                    arrayFloatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_float', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT, prec, 0, numTargets)]));
                    arrayFloatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC2, prec, 0, numTargets)]));
                    arrayFloatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC3, prec, 0, numTargets)]));
                    arrayFloatGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC4, prec, 0, numTargets)]));
                }
            }
        }

        // .fixed
        /** @type {tcuTestCase.DeqpTest} */ var arrayFixedGroup = tcuTestCase.newTest('array.fixed', 'Fixed-point output tests');
        testGroup.addChild(arrayFixedGroup);
        for (var fmtNdx = 0; fmtNdx < requiredFixedFormats.length; fmtNdx++) {
            var format = requiredFixedFormats[fmtNdx];
            var fmtName = es3fFboTestUtil.getFormatName(format);
            fboSpec = [];

            for (var ndx = 0; ndx < numTargets; ndx++)
                fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                prec = precisions[precNdx];
                precName = gluShaderUtil.getPrecisionName(prec);

                arrayFixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_float', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT, prec, 0, numTargets)]));
                arrayFixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC2, prec, 0, numTargets)]));
                arrayFixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC3, prec, 0, numTargets)]));
                arrayFixedGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_vec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.FLOAT_VEC4, prec, 0, numTargets)]));
            }
        }

        // .int
        /** @type {tcuTestCase.DeqpTest} */ var arrayIntGroup = tcuTestCase.newTest('array.int', 'Integer output tests');
        testGroup.addChild(arrayIntGroup);
        for (var fmtNdx = 0; fmtNdx < requiredIntFormats.length; fmtNdx++) {
            var format = requiredIntFormats[fmtNdx];
            var fmtName = es3fFboTestUtil.getFormatName(format);
            fboSpec = [];

            for (var ndx = 0; ndx < numTargets; ndx++)
                fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                prec = precisions[precNdx];
                precName = gluShaderUtil.getPrecisionName(prec);

                arrayIntGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_int', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT, prec, 0, numTargets)]));
                arrayIntGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_ivec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT_VEC2, prec, 0, numTargets)]));
                arrayIntGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_ivec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT_VEC3, prec, 0, numTargets)]));
                arrayIntGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_ivec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.INT_VEC4, prec, 0, numTargets)]));
            }
        }

        // .uint
        /** @type {tcuTestCase.DeqpTest} */ var arrayUintGroup = tcuTestCase.newTest('array.uint', 'Usigned integer output tests');
        testGroup.addChild(arrayUintGroup);
        for (var fmtNdx = 0; fmtNdx < requiredUintFormats.length; fmtNdx++) {
            var format = requiredUintFormats[fmtNdx];
            var fmtName = es3fFboTestUtil.getFormatName(format);
            fboSpec = [];

            for (var ndx = 0; ndx < numTargets; ndx++)
                fboSpec.push(new es3fFragmentOutputTests.BufferSpec(format, width, height, samples));

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                prec = precisions[precNdx];
                precName = gluShaderUtil.getPrecisionName(prec);

                arrayUintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uint', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT, prec, 0, numTargets)]));
                arrayUintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uvec2', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT_VEC2, prec, 0, numTargets)]));
                arrayUintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uvec3', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT_VEC3, prec, 0, numTargets)]));
                arrayUintGroup.addChild(new es3fFragmentOutputTests.FragmentOutputCase(fmtName + '_' + precName + '_uvec4', '', fboSpec, [new es3fFragmentOutputTests.FragmentOutput(gluShaderUtil.DataType.UINT_VEC4, prec, 0, numTargets)]));
            }
        }

    // .random

        /** @type {Array<tcuTestCase.DeqpTest>} */ var randomGroup = [];
        var numRandomGroups = 3;
        for (var ii = 0; ii < numRandomGroups; ++ii) {
            randomGroup[ii] = tcuTestCase.newTest('random', 'Random fragment output cases');
            testGroup.addChild(randomGroup[ii]);
        }

        /** @type {boolean} */ var colorBufferFloatSupported = (gl.getExtension('EXT_color_buffer_float') != null);
        for (var seed = 0; seed < 100; seed++) {
            var test = es3fFragmentOutputTests.createRandomCase(2, 4, seed, colorBufferFloatSupported);
            if (test !== null) {
                randomGroup[seed % numRandomGroups].addChild(test);
            }
        }

    };

    /**
     * Create and execute the test cases
     */
    es3fFragmentOutputTests.run = function(context, range) {
        gl = context;
      //Set up Test Root parameters
        var testName = 'fragment_output';
        var testDescription = 'Fragment Output Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

      //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            es3fFragmentOutputTests.init(gl);
            if (range)
                state.setRange(range);
            tcuTestCase.runTestCases();
        } catch (err) {
            testFailedOptions('Failed to es3fFragmentOutputTests.run tests', false);
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }

    };

});
