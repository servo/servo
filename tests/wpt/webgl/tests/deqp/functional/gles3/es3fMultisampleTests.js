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
goog.provide('functional.gles3.es3fMultisampleTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluStrUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTexture');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {
    /** @type {?WebGL2RenderingContext} */ var gl;
    var es3fMultisampleTests = functional.gles3.es3fMultisampleTests;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;
    var deString = framework.delibs.debase.deString;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuTexture = framework.common.tcuTexture;
    var gluStrUtil = framework.opengl.gluStrUtil;
    var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var tcuLogImage = framework.common.tcuLogImage;

    /**
     * @constructor
     * @struct
     * @param {Array<number>} p0_
     * @param {Array<number>} p1_
     * @param {Array<number>} p2_
     * @param {Array<number>} p3_
     */
    es3fMultisampleTests.QuadCorners = function(p0_, p1_, p2_, p3_) {
        /** @type {Array<number>} */ this.p0 = p0_;
        /** @type {Array<number>} */ this.p1 = p1_;
        /** @type {Array<number>} */ this.p2 = p2_;
        /** @type {Array<number>} */ this.p3 = p3_;
    };

    /**
     * @param {number} defaultCount
     * @return {number}
     */
    es3fMultisampleTests.getIterationCount = function(defaultCount) {
        // The C++ test takes an argument from the command line.
        // Leaving this function in case we want to be able to take an argument from the URL
        return defaultCount;
    };

    /**
     * @param  {Array<number>} point
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} p3
     * @return {boolean}
     */
    es3fMultisampleTests.isInsideQuad = function(point, p0, p1, p2, p3) {
        /** @type {number} */ var dot0 = (point[0] - p0[0]) * (p1[1] - p0[1]) + (point[1] - p0[1]) * (p0[0] - p1[0]);
        /** @type {number} */ var dot1 = (point[0] - p1[0]) * (p2[1] - p1[1]) + (point[1] - p1[1]) * (p1[0] - p2[0]);
        /** @type {number} */ var dot2 = (point[0] - p2[0]) * (p3[1] - p2[1]) + (point[1] - p2[1]) * (p2[0] - p3[0]);
        /** @type {number} */ var dot3 = (point[0] - p3[0]) * (p0[1] - p3[1]) + (point[1] - p3[1]) * (p3[0] - p0[0]);

        return (dot0 > 0) == (dot1 > 0) && (dot1 > 0) == (dot2 > 0) && (dot2 > 0) == (dot3 > 0);
    };

    /**
     * Check if a region in an image is unicolored.
     *
     * Checks if the pixels in img inside the convex quadilateral defined by
     * p0, p1, p2 and p3 are all (approximately) of the same color.
     *
     * @param  {tcuSurface.Surface} img
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} p3
     * @return {boolean}
     */
    es3fMultisampleTests.isPixelRegionUnicolored = function(img, p0, p1, p2, p3) {
        /** @type {number} */ var xMin = deMath.clamp(Math.min(p0[0], p1[0], p2[0], p3[0]), 0, img.getWidth() - 1);
        /** @type {number} */ var yMin = deMath.clamp(Math.min(p0[1], p1[1], p2[1], p3[1]), 0, img.getHeight() - 1);
        /** @type {number} */ var xMax = deMath.clamp(Math.max(p0[0], p1[0], p2[0], p3[0]), 0, img.getWidth() - 1);
        /** @type {number} */ var yMax = deMath.clamp(Math.max(p0[1], p1[1], p2[1], p3[1]), 0, img.getHeight() - 1);
        /** @type {boolean} */ var insideEncountered = false; //!< Whether we have already seen at least one pixel inside the region.
        /** @type {tcuRGBA.RGBA} */ var insideColor; //!< Color of the first pixel inside the region.
        /** @type {tcuRGBA.RGBA} */ var threshold = tcuRGBA.newRGBAComponents(3, 3, 3, 3);
        for (var y = yMin; y <= yMax; y++)
        for (var x = xMin; x <= xMax; x++)
            if (es3fMultisampleTests.isInsideQuad([x, y], p0, p1, p2, p3)) {
                /** @type {tcuRGBA.RGBA} */ var pixColor = new tcuRGBA.RGBA(img.getPixel(x, y));

                if (insideEncountered)
                    if (!tcuRGBA.compareThreshold(pixColor, insideColor, threshold)) // Pixel color differs from already-detected color inside same region - region not unicolored.
                        return false;
                else {
                    insideEncountered = true;
                    insideColor = pixColor;
                }
            }
        return true;
    };

    /**
     * [drawUnicolorTestErrors description]
     * @param  {tcuSurface.Surface} img
     * @param  {tcuTexture.PixelBufferAccess} errorImg
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} p3
     * @return {boolean}
     */
    es3fMultisampleTests.drawUnicolorTestErrors = function(img, errorImg, p0, p1, p2, p3) {
        /** @type {number} */ var xMin = deMath.clamp(Math.min(p0[0], p1[0], p2[0], p3[0]), 0, img.getWidth() - 1);
        /** @type {number} */ var yMin = deMath.clamp(Math.min(p0[1], p1[1], p2[1], p3[1]), 0, img.getHeight() - 1);
        /** @type {number} */ var xMax = deMath.clamp(Math.max(p0[0], p1[0], p2[0], p3[0]), 0, img.getWidth() - 1);
        /** @type {number} */ var yMax = deMath.clamp(Math.max(p0[1], p1[1], p2[1], p3[1]), 0, img.getHeight() - 1);
        /** @type {tcuRGBA.RGBA} */ var refColor = new tcuRGBA.RGBA(img.getPixel(Math.floor((xMin + xMax) / 2), Math.floor((yMin + yMax) / 2)));
        /** @type {tcuRGBA.RGBA} */ var threshold = tcuRGBA.newRGBAComponents(3, 3, 3, 3);
        for (var y = yMin; y <= yMax; y++)
        for (var x = xMin; x <= xMax; x++)
            if (es3fMultisampleTests.isInsideQuad([x, y], p0, p1, p2, p3)) {
                if (!tcuRGBA.compareThreshold(new tcuRGBA.RGBA(img.getPixel(x, y)), refColor, threshold)) {
                    img.setPixel(x, y, tcuRGBA.RGBA.red.toVec()); // TODO: this might also be toIVec()
                    errorImg.setPixel([1.0, 0.0, 0.0, 1.0], x, y);
                }
            }
        return true;
    };

    /**
     * @constructor
     * @struct
     * @param {number=} numSamples_
     * @param {boolean=} useDepth_
     * @param {boolean=} useStencil_
     */
    es3fMultisampleTests.FboParams = function(numSamples_, useDepth_, useStencil_) {
        /** @type {boolean} */ var useFbo_ = true;
        if (numSamples_ === undefined && useDepth_ === undefined && useStencil_ === undefined)
            useFbo_ = false;
        /** @type {boolean} */ this.useFbo = useFbo_;
        /** @type {number} */ this.numSamples = numSamples_ === undefined ? -1 : numSamples_;
        /** @type {boolean} */ this.useDepth = useDepth_ === undefined ? false : useDepth_;
        /** @type {boolean} */ this.useStencil = useStencil_ === undefined ? false : useStencil_;

    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} desiredViewportSize
     * @param {es3fMultisampleTests.FboParams} fboParams
     */
    es3fMultisampleTests.MultisampleCase = function(name, desc, desiredViewportSize, fboParams) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        /** @type {number} */ this.m_numSamples = 0;
        /** @type {number} */ this.m_viewportSize = 0;
        /** @type {number} */ this.m_desiredViewportSize = desiredViewportSize;
        /** @type {es3fMultisampleTests.FboParams} */ this.m_fboParams = fboParams;
        /** @type {WebGLRenderbuffer} */ this.m_msColorRbo = null;
        /** @type {WebGLRenderbuffer} */ this.m_msDepthStencilRbo = null;
        /** @type {WebGLRenderbuffer} */ this.m_resolveColorRbo = null;
        /** @type {WebGLFramebuffer} */ this.m_msFbo = null;
        /** @type {WebGLFramebuffer} */ this.m_resolveFbo = null;
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {number} */ this.m_attrPositionLoc = -1;
        /** @type {number} */ this.m_attrColorLoc = -1;
        /** @type {number} */ this.m_renderWidth = fboParams.useFbo ? 2 * desiredViewportSize : gl.drawingBufferWidth;
        /** @type {number} */ this.m_renderHeight = fboParams.useFbo ? 2 * desiredViewportSize : gl.drawingBufferHeight;
        /** @type {number} */ this.m_viewportX = 0;
        /** @type {number} */ this.m_viewportY = 0;
        /** @type {deRandom.Random} */ this.m_rnd = new deRandom.Random(deString.deStringHash(this.name));
        if (this.m_fboParams.useFbo)
            assertMsgOptions(this.m_fboParams.numSamples >= 0, 'fboParams.numSamples < 0', false, true);
    };

    es3fMultisampleTests.MultisampleCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.MultisampleCase.prototype.constructor = es3fMultisampleTests.MultisampleCase;

    /* Rest states */
    es3fMultisampleTests.MultisampleCase.prototype.deinit = function() {
        gl.colorMask(true, true, true, true);
        gl.depthMask(true);

        gl.clearColor(0.0, 0.0, 0.0, 0.0);
        gl.clearDepth(1.0);
        gl.clearStencil(0);

        gl.disable(gl.STENCIL_TEST);
        gl.disable(gl.DEPTH_TEST);
        gl.disable(gl.BLEND)
        gl.disable(gl.SAMPLE_COVERAGE);
        gl.disable(gl.SAMPLE_ALPHA_TO_COVERAGE);

        if (this.m_program) {
            gl.deleteProgram(this.m_program.getProgram());
            this.m_program = null;
        }
        if (this.m_msColorRbo) {
          gl.deleteRenderbuffer(this.m_msColorRbo);
          this.m_msColorRbo = null;
        }
        if (this.m_msDepthStencilRbo) {
          gl.deleteRenderbuffer(this.m_msDepthStencilRbo);
          this.m_msDepthStencilRbo = null;
        }
        if (this.m_resolveColorRbo) {
          gl.deleteRenderbuffer(this.m_resolveColorRbo);
          this.m_resolveColorRbo = null;
        }

        if (this.m_msFbo) {
          gl.deleteFramebuffer(this.m_msFbo);
          this.m_msFbo = null;
        }
        if (this.m_resolveFbo) {
          gl.deleteFramebuffer(this.m_resolveFbo);
          this.m_resolveFbo = null;
        }

        gl.bindRenderbuffer(gl.RENDERBUFFER, null);
        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    }

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} c0
     * @param  {Array<number>} c1
     * @param  {Array<number>} c2
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderTriangle_pAsVec3cAsVec4 = function(p0, p1, p2, c0, c1, c2) {
        /** @type {Array<number>} */ var vertexPositions = [
            p0[0], p0[1], p0[2], 1.0,
            p1[0], p1[1], p1[2], 1.0,
            p2[0], p2[1], p2[2], 1.0
        ];
        /** @type {Array<number>} */ var vertexColors = [
            c0[0], c0[1], c0[2], c0[3],
            c1[0], c1[1], c1[2], c1[3],
            c2[0], c2[1], c2[2], c2[3]
        ];

        var posGLBuffer = gl.createBuffer();
        /** @type {ArrayBufferView} */ var posBuffer = new Float32Array(vertexPositions);
        gl.bindBuffer(gl.ARRAY_BUFFER, posGLBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, posBuffer, gl.STATIC_DRAW);

        gl.enableVertexAttribArray(this.m_attrPositionLoc);
        gl.vertexAttribPointer(this.m_attrPositionLoc, 4, gl.FLOAT, false, 0, 0);

        var colGLBuffer = gl.createBuffer();
        /** @type {ArrayBufferView} */ var colBuffer = new Float32Array(vertexColors);
        gl.bindBuffer(gl.ARRAY_BUFFER, colGLBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, colBuffer, gl.STATIC_DRAW);

        gl.enableVertexAttribArray(this.m_attrColorLoc);
        gl.vertexAttribPointer(this.m_attrColorLoc, 4, gl.FLOAT, false, 0, 0);

        gl.useProgram(this.m_program.getProgram());
        gl.drawArrays(gl.TRIANGLES, 0, 3);

        gl.bindBuffer(gl.ARRAY_BUFFER, null);
        gl.deleteBuffer(colGLBuffer);
        gl.deleteBuffer(posGLBuffer);
    };

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} color
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderTriangle_pAsVec3WithColor = function(p0, p1, p2, color) {
        this.renderTriangle_pAsVec3cAsVec4(p0, p1, p2, color, color, color);
    };

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} c0
     * @param  {Array<number>} c1
     * @param  {Array<number>} c2
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderTriangle_pAsVec2 = function(p0, p1, p2, c0, c1, c2) {
        this.renderTriangle_pAsVec3cAsVec4(
            [p0[0], p0[1], 0.0],
            [p1[0], p1[1], 0.0],
            [p2[0], p2[1], 0.0],
            c0, c1, c2);
    };

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} color
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderTriangle_pAsVec2WithColor = function(p0, p1, p2, color) {
        this.renderTriangle_pAsVec2(p0, p1, p2, color, color, color);
    };

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} p3
     * @param  {Array<number>} c0
     * @param  {Array<number>} c1
     * @param  {Array<number>} c2
     * @param  {Array<number>} c3
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderQuad = function(p0, p1, p2, p3, c0, c1, c2, c3) {
        this.renderTriangle_pAsVec2(p0, p1, p2, c0, c1, c2);
        this.renderTriangle_pAsVec2(p2, p1, p3, c2, c1, c3);
    };

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} p2
     * @param  {Array<number>} p3
     * @param  {Array<number>} color
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderQuad_WithColor = function(p0, p1, p2, p3, color) {
        this.renderQuad(p0, p1, p2, p3, color, color, color, color);
    };

    /**
     * @protected
     * @param  {Array<number>} p0
     * @param  {Array<number>} p1
     * @param  {Array<number>} color
     */
    es3fMultisampleTests.MultisampleCase.prototype.renderLine = function(p0, p1, color) {
        /** @type {Array<number>} */ var vertexPositions = [
            p0[0], p0[1], 0.0, 1.0,
            p1[0], p1[1], 0.0, 1.0
        ];
        /** @type {Array<number>} */ var vertexColors = [
            color[0], color[1], color[2], color[3],
            color[0], color[1], color[2], color[3]
        ];

        var posGLBuffer = gl.createBuffer();
        /** @type {ArrayBufferView} */ var posBuffer = new Float32Array(vertexPositions);
        gl.bindBuffer(gl.ARRAY_BUFFER, posGLBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, posBuffer, gl.STATIC_DRAW);

        gl.enableVertexAttribArray(this.m_attrPositionLoc);
        gl.vertexAttribPointer(this.m_attrPositionLoc, 4, gl.FLOAT, false, 0, 0);

        var colGLBuffer = gl.createBuffer();
        /** @type {ArrayBufferView} */ var colBuffer = new Float32Array(vertexColors);
        gl.bindBuffer(gl.ARRAY_BUFFER, colGLBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, colBuffer, gl.STATIC_DRAW);

        gl.enableVertexAttribArray(this.m_attrColorLoc);
        gl.vertexAttribPointer(this.m_attrColorLoc, 4, gl.FLOAT, false, 0, 0);

        gl.useProgram(this.m_program.getProgram());
        gl.drawArrays(gl.LINES, 0, 2);

        gl.bindBuffer(gl.ARRAY_BUFFER, null);
        gl.deleteBuffer(colGLBuffer);
        gl.deleteBuffer(posGLBuffer);
    };

    /**
     * @protected
     */
    es3fMultisampleTests.MultisampleCase.prototype.randomizeViewport = function() {
        this.m_viewportX = this.m_rnd.getInt(0, this.m_renderWidth - this.m_viewportSize);
        this.m_viewportY = this.m_rnd.getInt(0, this.m_renderHeight - this.m_viewportSize);

        gl.viewport(this.m_viewportX, this.m_viewportY, this.m_viewportSize, this.m_viewportSize);
    };

    /**
     * @protected
     * @return {tcuSurface.Surface}
     */
    es3fMultisampleTests.MultisampleCase.prototype.readImage = function() {
        /** @type {tcuSurface.Surface} */
        var dst = new tcuSurface.Surface(this.m_viewportSize, this.m_viewportSize);
        /** @type {number} */ var pixelSize = dst.getAccess().getFormat().getPixelSize();
        /** @type {number} */ var param = deMath.deIsPowerOfTwo32(pixelSize) ? Math.min(pixelSize, 8) : 1;
        /** @type {gluTextureUtil.TransferFormat} */ var format = gluTextureUtil.getTransferFormat(dst.getAccess().getFormat());
        /** @type {number} */ var width = dst.getAccess().getWidth();
        /** @type {number} */ var height = dst.getAccess().getHeight();
        if (this.m_fboParams.useFbo) {
            gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, this.m_resolveFbo);
            gl.blitFramebuffer(0, 0, this.m_renderWidth, this.m_renderHeight, 0, 0, this.m_renderWidth, this.m_renderHeight, gl.COLOR_BUFFER_BIT, gl.NEAREST);
            gl.bindFramebuffer(gl.READ_FRAMEBUFFER, this.m_resolveFbo);

            gl.pixelStorei(gl.PACK_ALIGNMENT, param);
            gl.readPixels(this.m_viewportX, this.m_viewportY, width, height, format.format, format.dataType, dst.getAccess().getDataPtr());

            gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_msFbo);
        }
        else {
            gl.pixelStorei(gl.PACK_ALIGNMENT, param);
            gl.readPixels(this.m_viewportX, this.m_viewportY, width, height, format.format, format.dataType, dst.getAccess().getDataPtr());
        }
        return dst;
    };

    es3fMultisampleTests.MultisampleCase.prototype.init = function() {
        /** @type {string} */ var vertShaderSource = '' +
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in mediump vec4 a_color;\n' +
            'out mediump vec4 v_color;\n' +
            'void main()\n' +
            '{\n' +
            '    gl_Position = a_position;\n' +
            '    v_color = a_color;\n' +
            '}\n';

        /** @type {string} */ var fragShaderSource = '' +
            '#version 300 es\n' +
            'in mediump vec4 v_color;\n' +
            'layout(location = 0) out mediump vec4 o_color;\n' +
            'void main()\n' +
            '{\n' +
            '    o_color = v_color;\n' +
            '}\n';

        var numSamples = /** @type {number} */  (gl.getParameter(gl.SAMPLES));
        if (!this.m_fboParams.useFbo && numSamples <= 1) {
            var msg = 'No multisample buffers';
            checkMessage(false, msg);
            return false;
        }

        if (this.m_fboParams.useFbo) {
            if (this.m_fboParams.numSamples > 0)
                this.m_numSamples = this.m_fboParams.numSamples;
            else {
                bufferedLogToConsole('Querying maximum number of samples for ' + gluStrUtil.getPixelFormatName(gl.RGBA8) + ' with gl.getInternalformatParameter()');
                var supportedSampleCountArray = /** @type {Int32Array} */ (gl.getInternalformatParameter(gl.RENDERBUFFER, gl.RGBA8, gl.SAMPLES));
                if (supportedSampleCountArray.length == 0) {
                    var msg = 'No supported sample counts';
                    checkMessage(false, msg);
                    return false;
                }
                this.m_numSamples = supportedSampleCountArray[0];
            }

            bufferedLogToConsole('Using FBO of size (' + this.m_renderWidth + ', ' + this.m_renderHeight + ') with ' + this.m_numSamples + ' samples');
        }
        else {
            // Query and log number of samples per pixel.
            this.m_numSamples =  numSamples;
            bufferedLogToConsole('gl.SAMPLES =' + this.m_numSamples);
        }

        // Prepare program.

        assertMsgOptions(!this.m_program, 'Program loaded when it should not be.', false, true);

        this.m_program = new gluShaderProgram.ShaderProgram(
            gl,
            gluShaderProgram.makeVtxFragSources(vertShaderSource, fragShaderSource));

        if (!this.m_program.isOk())
            throw new Error('Failed to compile program');

        this.m_attrPositionLoc = gl.getAttribLocation(this.m_program.getProgram(), 'a_position');
        this.m_attrColorLoc = gl.getAttribLocation(this.m_program.getProgram(), 'a_color');

        if (this.m_attrPositionLoc < 0 || this.m_attrColorLoc < 0) {
            this.m_program = null;
            throw new Error('Invalid attribute locations');
        }

        if (this.m_fboParams.useFbo) {
            // Setup ms color RBO.
            this.m_msColorRbo = gl.createRenderbuffer();
            gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_msColorRbo);

            /** @type {Int32Array} */ var supportedSampleCountArray = /** @type {Int32Array} */ (gl.getInternalformatParameter(gl.RENDERBUFFER, gl.RGBA8, gl.SAMPLES));
            var maxSampleCount = supportedSampleCountArray[0];
            if (maxSampleCount < this.m_numSamples) {
                bufferedLogToConsole('skipping test: ' + this.m_numSamples + ' samples not supported; max is ' + maxSampleCount);
                return false;
            }

            assertMsgOptions(gl.getError() === gl.NO_ERROR, 'should be no GL error before renderbufferStorageMultisample');
            gl.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, gl.RGBA8, this.m_renderWidth, this.m_renderHeight);
            assertMsgOptions(gl.getError() === gl.NO_ERROR, 'should be no GL error after renderbufferStorageMultisample');


            if (this.m_fboParams.useDepth || this.m_fboParams.useStencil) {
                // Setup ms depth & stencil RBO.
                this.m_msDepthStencilRbo = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_msDepthStencilRbo);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, this.m_numSamples, gl.DEPTH24_STENCIL8, this.m_renderWidth, this.m_renderHeight);
            }

            // Setup ms FBO.
            this.m_msFbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_msFbo);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, this.m_msColorRbo);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, this.m_msDepthStencilRbo);

            // Setup resolve color RBO.
            this.m_resolveColorRbo = gl.createRenderbuffer();
            gl.bindRenderbuffer(gl.RENDERBUFFER, this.m_resolveColorRbo);
            gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, this.m_renderWidth, this.m_renderHeight);

            // Setup resolve FBO.
            this.m_resolveFbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_resolveFbo);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, this.m_resolveColorRbo);

            // Use ms FBO.
            gl.bindFramebuffer(gl.FRAMEBUFFER, this.m_msFbo);
        }

        // Get suitable viewport size.

        this.m_viewportSize = Math.min(this.m_desiredViewportSize, this.m_renderWidth, this.m_renderHeight);
        this.randomizeViewport();
        return true;
    };

    /**
     * Base class for cases testing the value of sample count.
     *
     * Draws a test pattern (defined by renderPattern() of an inheriting class)
     * and counts the number of distinct colors in the resulting image. That
     * number should be at least the value of sample count plus one. This is
     * repeated with increased values of m_currentIteration until this correct
     * number of colors is detected or m_currentIteration reaches
     * m_maxNumIterations.
     *
     * @extends {es3fMultisampleTests.MultisampleCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fMultisampleTests.FboParams} fboParams
     */
    es3fMultisampleTests.NumSamplesCase = function(name, desc, fboParams) {
        es3fMultisampleTests.MultisampleCase.call(this, name, desc, 256, fboParams);
        /** @type {number} */ var DEFAULT_MAX_NUM_ITERATIONS = 16;
        /** @type {number} */ this.m_currentIteration = 0;
        /** @type {number} */ this.m_maxNumIterations = es3fMultisampleTests.getIterationCount(DEFAULT_MAX_NUM_ITERATIONS);
        /** @type {Array<tcuRGBA.RGBA>} */ this.m_detectedColors = [];
    };

    es3fMultisampleTests.NumSamplesCase.prototype = Object.create(es3fMultisampleTests.MultisampleCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.NumSamplesCase.prototype.constructor = es3fMultisampleTests.NumSamplesCase;

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fMultisampleTests.NumSamplesCase.prototype.iterate = function() {
        this.randomizeViewport();

        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        this.renderPattern();

        // Read and log rendered image.

        /** @type {tcuSurface.Surface} */ var renderedImg = this.readImage();
        tcuLogImage.logImage('RenderedImage', 'Rendered image', renderedImg.getAccess());

        // Detect new, previously unseen colors from image.

        /** @type {number} */ var requiredNumDistinctColors = this.m_numSamples + 1;

        // If the number of samples is high (64 or more), we need to lower the threshold for detecting unique colors, otherwise two expected unique colors would be treated as the same color.
        var threshold = Math.min(3, Math.floor(255 / this.m_numSamples) - 1);
        var thresholdRGBA = tcuRGBA.newRGBAComponents(threshold, threshold, threshold, threshold);

        for (var y = 0; y < renderedImg.getHeight() && this.m_detectedColors.length < requiredNumDistinctColors; y++)
        for (var x = 0; x < renderedImg.getWidth() && this.m_detectedColors.length < requiredNumDistinctColors; x++) {
            /** @type {tcuRGBA.RGBA} */ var color = new tcuRGBA.RGBA(renderedImg.getPixel(x, y));

            /** @type {number} */ var i;
            for (i = 0; i < this.m_detectedColors.length; i++) {
                if (tcuRGBA.compareThreshold(color, this.m_detectedColors[i], thresholdRGBA))
                    break;
            }

            if (i === this.m_detectedColors.length)
                this.m_detectedColors.push(color); // Color not previously detected.
        }

        // Log results.

            bufferedLogToConsole('Number of distinct colors detected so far: ' + (this.m_detectedColors.length >= requiredNumDistinctColors ? 'at least ' : '') + this.m_detectedColors.length);


        if (this.m_detectedColors.length < requiredNumDistinctColors) {
            // Haven't detected enough different colors yet.

            this.m_currentIteration++;

            if (this.m_currentIteration >= this.m_maxNumIterations) {
                testFailedOptions('Failure: Number of distinct colors detected is lower than sample count+1', false);
                return tcuTestCase.IterateResult.STOP;
            }
            else {
                bufferedLogToConsole('The number of distinct colors detected is lower than sample count+1 - trying again with a slightly altered pattern');
                return tcuTestCase.IterateResult.CONTINUE;
            }
        }
        else {
            testPassedOptions('Success: The number of distinct colors detected is at least sample count+1', true);
            return tcuTestCase.IterateResult.STOP;
        }
    };

    /**
    * @extends {es3fMultisampleTests.NumSamplesCase}
    * @constructor
    * @param {string} name
    * @param {string} desc
    * @param {number=} numFboSamples
    */
    es3fMultisampleTests.PolygonNumSamplesCase = function(name, desc, numFboSamples) {
        numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
        /** @type {es3fMultisampleTests.FboParams} */
        var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, false) : new es3fMultisampleTests.FboParams();
        es3fMultisampleTests.NumSamplesCase.call(this, name, desc, params);
    };

    es3fMultisampleTests.PolygonNumSamplesCase.prototype = Object.create(es3fMultisampleTests.NumSamplesCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.PolygonNumSamplesCase.prototype.constructor = es3fMultisampleTests.PolygonNumSamplesCase;

    es3fMultisampleTests.PolygonNumSamplesCase.prototype.renderPattern = function() {
        // The test pattern consists of several triangles with edges at different angles.

        /** @type {number} */ var numTriangles = 25;
        for (var i = 0; i < numTriangles; i++) {
            /** @type {number} */ var angle0 = 2.0 * Math.PI * i / numTriangles + 0.001 * this.m_currentIteration;
            /** @type {number} */ var angle1 = 2.0 * Math.PI * (i + 0.5) / numTriangles + 0.001 * this.m_currentIteration;

            this.renderTriangle_pAsVec2WithColor(
                [0.0, 0.0],
                [Math.cos(angle0) * 0.95, Math.sin(angle0) * 0.95],
                [Math.cos(angle1) * 0.95, Math.sin(angle1) * 0.95],
                [1.0, 1.0, 1.0, 1.0]);
        }
    };

    /**
    * @extends {es3fMultisampleTests.NumSamplesCase}
    * @constructor
    * @param {string} name
    * @param {string} desc
    * @param {number=} numFboSamples
    */
    es3fMultisampleTests.LineNumSamplesCase = function(name, desc, numFboSamples) {
        numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
        /** @type {es3fMultisampleTests.FboParams} */
        var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, false) : new es3fMultisampleTests.FboParams();
        es3fMultisampleTests.NumSamplesCase.call(this, name, desc, params);
    };

    es3fMultisampleTests.LineNumSamplesCase.prototype = Object.create(es3fMultisampleTests.NumSamplesCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.LineNumSamplesCase.prototype.constructor = es3fMultisampleTests.LineNumSamplesCase;

    es3fMultisampleTests.LineNumSamplesCase.prototype.renderPattern = function() {
        // The test pattern consists of several lines at different angles.

        // We scale the number of lines based on the viewport size. This is because a gl line's thickness is
        // constant in pixel units, i.e. they get relatively thicker as viewport size decreases. Thus we must
        // decrease the number of lines in order to decrease the extent of overlap among the lines in the
        // center of the pattern.
        /** @type {number} */ var numLines = Math.floor(100.0 * Math.sqrt(this.m_viewportSize / 256.0));

        for (var i = 0; i < numLines; i++) {
            /** @type {number} */ var angle = 2.0 * Math.PI * i / numLines + 0.001 * this.m_currentIteration;
            this.renderLine([0.0, 0.0], [Math.cos(angle) * 0.95, Math.sin(angle) * 0.95], [1.0, 1.0, 1.0, 1.0]);
        }
    };

    /**
     * Case testing behaviour of common edges when multisampling.
     *
     * Draws a number of test patterns, each with a number of quads, each made
     * of two triangles, rotated at different angles. The inner edge inside the
     * quad (i.e. the common edge of the two triangles) still should not be
     * visible, despite multisampling - i.e. the two triangles forming the quad
     * should never get any common coverage bits in any pixel.
     *
     * @extends {es3fMultisampleTests.MultisampleCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fMultisampleTests.CommonEdgeCase.CaseType} caseType
     * @param {number} numFboSamples
     */
    es3fMultisampleTests.CommonEdgeCase = function(name, desc, caseType, numFboSamples) {
        /** @type {number} */ var cases = caseType === es3fMultisampleTests.CommonEdgeCase.CaseType.SMALL_QUADS ? 128 : 32;
        numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
        /** @type {es3fMultisampleTests.FboParams} */
        var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, false) : new es3fMultisampleTests.FboParams();

        es3fMultisampleTests.MultisampleCase.call(this, name, desc, cases, params);
        /** @type {number} */ var DEFAULT_SMALL_QUADS_ITERATIONS = 16;
        /** @type {number} */ var DEFAULT_BIGGER_THAN_VIEWPORT_QUAD_ITERATIONS = 64; // 8*8
        /** @type {es3fMultisampleTests.CommonEdgeCase.CaseType} */ this.m_caseType = caseType;
        /** @type {number} */ this.m_currentIteration = 0;
        /** @type {number} */
        this.m_numIterations = caseType === es3fMultisampleTests.CommonEdgeCase.CaseType.SMALL_QUADS ? es3fMultisampleTests.getIterationCount(DEFAULT_SMALL_QUADS_ITERATIONS) :
                               caseType === es3fMultisampleTests.CommonEdgeCase.CaseType.BIGGER_THAN_VIEWPORT_QUAD ? es3fMultisampleTests.getIterationCount(DEFAULT_BIGGER_THAN_VIEWPORT_QUAD_ITERATIONS) :
                               8;
    };

    es3fMultisampleTests.CommonEdgeCase.prototype = Object.create(es3fMultisampleTests.MultisampleCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.CommonEdgeCase.prototype.constructor = es3fMultisampleTests.CommonEdgeCase;

    /**
     * @enum {number}
     */
    es3fMultisampleTests.CommonEdgeCase.CaseType = {
        SMALL_QUADS: 0,                  //!< Draw several small quads per iteration.
        BIGGER_THAN_VIEWPORT_QUAD: 1, //!< Draw one bigger-than-viewport quad per iteration.
        FIT_VIEWPORT_QUAD: 2          //!< Draw one exactly viewport-sized, axis aligned quad per iteration.
    };

    es3fMultisampleTests.CommonEdgeCase.prototype.init = function() {
        var inited = es3fMultisampleTests.MultisampleCase.prototype.init.call(this);
        if (!inited) {
            return false;
        }

        if (this.m_caseType === es3fMultisampleTests.CommonEdgeCase.CaseType.SMALL_QUADS) {
            // Check for a big enough viewport. With too small viewports the test case can't analyze the resulting image well enough.

            /** @type {number} */ var minViewportSize = 32;

            if (this.m_viewportSize < minViewportSize)
                throw new Error('Render target width or height too low (is ' + this.m_viewportSize + ', should be at least ' + minViewportSize + ')');
        }

        gl.enable(gl.BLEND);
        gl.blendEquation(gl.FUNC_ADD);
        gl.blendFunc(gl.ONE, gl.ONE);
        bufferedLogToConsole('Additive blending enabled in order to detect (erroneously) overlapping samples');
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fMultisampleTests.CommonEdgeCase.prototype.iterate = function() {
        /** @type {tcuSurface.Surface} */ var errorImg = new tcuSurface.Surface(this.m_viewportSize, this.m_viewportSize);

        this.randomizeViewport();

        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        // Draw test pattern. Test patterns consist of quads formed with two triangles.
        // After drawing the pattern, we check that the interior pixels of each quad are
        // all the same color - this is meant to verify that there are no artifacts on the inner edge.

        /** @type {Array<es3fMultisampleTests.QuadCorners>} */ var unicoloredRegions = [];

        /** @type {Array<Array<number>>} */ var corners;
        /** @type {number} */ var angleCos;
        /** @type {number} */ var angleSin;
        /** @type {number} */ var angle;
        /** @type {number} */ var quadDiagLen;
        /** @type {number} */ var unicolorRegionScale;
        /** @type {number} */ var quadBaseAngleNdx
        /** @type {number} */ var quadSubAngleNdx;

        if (this.m_caseType == es3fMultisampleTests.CommonEdgeCase.CaseType.SMALL_QUADS) {
            // Draw several quads, rotated at different angles.

            quadDiagLen = 2.0 / 3.0 * 0.9; // \note Fit 3 quads in both x and y directions.


            // \note First and second iteration get exact 0 (and 90, 180, 270) and 45 (and 135, 225, 315) angle quads, as they are kind of a special case.

            if (this.m_currentIteration === 0) {
                angleCos = 1.0;
                angleSin = 0.0;
            }
            else if (this.m_currentIteration === 1) {
                angleCos = Math.SQRT1_2;
                angleSin = Math.SQRT1_2;
            }
            else {
                angle = 0.5 * Math.PI * (this.m_currentIteration - 1) / (this.m_numIterations - 1);
                angleCos = Math.cos(angle);
                angleSin = Math.sin(angle);
            }

            corners = [
                deMath.scale([angleCos, angleSin], 0.5 * quadDiagLen),
                deMath.scale([-angleSin, angleCos], 0.5 * quadDiagLen),
                deMath.scale([-angleCos, -angleSin], 0.5 * quadDiagLen),
                deMath.scale([angleSin, -angleCos], 0.5 * quadDiagLen)
            ];

            // Draw 8 quads.
            // First four are rotated at angles angle+0, angle+90, angle+180 and angle+270.
            // Last four are rotated the same angles as the first four, but the ordering of the last triangle's vertices is reversed.

            for (var quadNdx = 0; quadNdx < 8; quadNdx++) {
                /** @type {Array<number>} */
                var center = deMath.addScalar(
                    deMath.scale([quadNdx % 3, quadNdx / 3], (2.0 - quadDiagLen)/ 2.0),
                    (-0.5 * (2.0 - quadDiagLen)));

                this.renderTriangle_pAsVec2WithColor(
                    deMath.add(corners[(0 + quadNdx) % 4], center),
                    deMath.add(corners[(1 + quadNdx) % 4], center),
                    deMath.add(corners[(2 + quadNdx) % 4], center),
                    [0.5, 0.5, 0.5, 1.0]);

                if (quadNdx >= 4) {
                    this.renderTriangle_pAsVec2WithColor(
                        deMath.add(corners[(3 + quadNdx) % 4], center),
                        deMath.add(corners[(2 + quadNdx) % 4], center),
                        deMath.add(corners[(0 + quadNdx) % 4], center),
                        [0.5, 0.5, 0.5, 1.0]);
                }
                else {
                    this.renderTriangle_pAsVec2WithColor(
                        deMath.add(corners[(0 + quadNdx) % 4], center),
                        deMath.add(corners[(2 + quadNdx) % 4], center),
                        deMath.add(corners[(3 + quadNdx) % 4], center),
                        [0.5, 0.5, 0.5, 1.0]);
                }

                // The size of the 'interior' of a quad is assumed to be approximately unicolorRegionScale*<actual size of quad>.
                // By 'interior' we here mean the region of non-boundary pixels of the rendered quad for which we can safely assume
                // that it has all coverage bits set to 1, for every pixel.
                unicolorRegionScale = 1.0 - 6.0 * 2.0 / this.m_viewportSize / quadDiagLen;
                unicoloredRegions.push(
                    new es3fMultisampleTests.QuadCorners(
                        deMath.add(center, deMath.scale(corners[0], unicolorRegionScale)),
                        deMath.add(center, deMath.scale(corners[1], unicolorRegionScale)),
                        deMath.add(center, deMath.scale(corners[2], unicolorRegionScale)),
                        deMath.add(center, deMath.scale(corners[3], unicolorRegionScale))));
            }
        }
        else if (this.m_caseType === es3fMultisampleTests.CommonEdgeCase.CaseType.BIGGER_THAN_VIEWPORT_QUAD) {
            // Draw a bigger-than-viewport quad, rotated at an angle depending on m_currentIteration.

            quadBaseAngleNdx = Math.floor(this.m_currentIteration / 8);
            quadSubAngleNdx = this.m_currentIteration % 8;

            if (quadBaseAngleNdx === 0) {
                angleCos = 1.0;
                angleSin = 0.0;
            }
            else if (quadBaseAngleNdx === 1) {
                angleCos = Math.SQRT1_2;
                angleSin = Math.SQRT1_2;
            }
            else {
                angle = 0.5 * Math.PI * (this.m_currentIteration - 1) / (this.m_numIterations - 1);
                angleCos = Math.cos(angle);
                angleSin = Math.sin(angle);
            }

            quadDiagLen = 2.5 / Math.max(angleCos, angleSin);

            corners = [
                deMath.scale([angleCos, angleSin], 0.5 * quadDiagLen),
                deMath.scale([-angleSin, angleCos], 0.5 * quadDiagLen),
                deMath.scale([-angleCos, -angleSin], 0.5 * quadDiagLen),
                deMath.scale([angleSin, -angleCos], 0.5 * quadDiagLen)
            ];

            this.renderTriangle_pAsVec2WithColor(
                corners[(0 + quadSubAngleNdx) % 4],
                corners[(1 + quadSubAngleNdx) % 4],
                corners[(2 + quadSubAngleNdx) % 4],
                [0.5, 0.5, 0.5, 1.0]);

            if (quadSubAngleNdx >= 4) {
                this.renderTriangle_pAsVec2WithColor(
                    corners[(3 + quadSubAngleNdx) % 4],
                    corners[(2 + quadSubAngleNdx) % 4],
                    corners[(0 + quadSubAngleNdx) % 4],
                    [0.5, 0.5, 0.5, 1.0]);
            }
            else {
                this.renderTriangle_pAsVec2WithColor(
                    corners[(0 + quadSubAngleNdx) % 4],
                    corners[(2 + quadSubAngleNdx) % 4],
                    corners[(3 + quadSubAngleNdx) % 4],
                    [0.5, 0.5, 0.5, 1.0]);
            }

            unicolorRegionScale = 1.0 - 6.0 * 2.0 / this.m_viewportSize / quadDiagLen;
            unicoloredRegions.push(
                new es3fMultisampleTests.QuadCorners(
                    deMath.scale(corners[0], unicolorRegionScale),
                    deMath.scale(corners[1], unicolorRegionScale),
                    deMath.scale(corners[2], unicolorRegionScale),
                    deMath.scale(corners[3], unicolorRegionScale)));
        }
        else if (this.m_caseType === es3fMultisampleTests.CommonEdgeCase.CaseType.FIT_VIEWPORT_QUAD) {
            // Draw an exactly viewport-sized quad, rotated by multiples of 90 degrees angle depending on m_currentIteration.

            quadSubAngleNdx = this.m_currentIteration % 8;

            corners = [
                [1.0, 1.0],
                [-1.0, 1.0],
                [-1.0, -1.0],
                [1.0, -1.0]
            ];

            this.renderTriangle_pAsVec2WithColor(
                corners[(0 + quadSubAngleNdx) % 4],
                corners[(1 + quadSubAngleNdx) % 4],
                corners[(2 + quadSubAngleNdx) % 4],
                [0.5, 0.5, 0.5, 1.0]);

            if (quadSubAngleNdx >= 4) {
                this.renderTriangle_pAsVec2WithColor(
                    corners[(3 + quadSubAngleNdx) % 4],
                    corners[(2 + quadSubAngleNdx) % 4],
                    corners[(0 + quadSubAngleNdx) % 4],
                    [0.5, 0.5, 0.5, 1.0]);
            }
            else {
                this.renderTriangle_pAsVec2WithColor(
                    corners[(0 + quadSubAngleNdx) % 4],
                    corners[(2 + quadSubAngleNdx) % 4],
                    corners[(3 + quadSubAngleNdx) % 4],
                    [0.5, 0.5, 0.5, 1.0]);
            }

            unicoloredRegions.push(new es3fMultisampleTests.QuadCorners(corners[0], corners[1], corners[2], corners[3]));
        }
        else
            throw new Error('CaseType not supported.');

        // Read pixels and check unicolored regions.

        /** @type {tcuSurface.Surface} */ var renderedImg = this.readImage();

        errorImg.getAccess().clear([0.0, 1.0, 0.0, 1.0]);
        tcuLogImage.logImage('RenderedImage', 'Rendered image', renderedImg.getAccess());

        /** @type {boolean} */ var errorsDetected = false;
        for (var i = 0; i < unicoloredRegions.length; i++) {
            /** @type {es3fMultisampleTests.QuadCorners} */ var region = unicoloredRegions[i];
            /** @type {Array<number>} */ var p0Win = deMath.scale(deMath.addScalar(region.p0, 1.0), 0.5 * (this.m_viewportSize - 1) + 0.5);
            /** @type {Array<number>} */ var p1Win = deMath.scale(deMath.addScalar(region.p1, 1.0), 0.5 * (this.m_viewportSize - 1) + 0.5);
            /** @type {Array<number>} */ var p2Win = deMath.scale(deMath.addScalar(region.p2, 1.0), 0.5 * (this.m_viewportSize - 1) + 0.5);
            /** @type {Array<number>} */ var p3Win = deMath.scale(deMath.addScalar(region.p3, 1.0), 0.5 * (this.m_viewportSize - 1) + 0.5);
            /** @type {boolean} */ var errorsInCurrentRegion = !es3fMultisampleTests.isPixelRegionUnicolored(renderedImg, p0Win, p1Win, p2Win, p3Win);

            if (errorsInCurrentRegion)
                es3fMultisampleTests.drawUnicolorTestErrors(renderedImg, errorImg.getAccess(), p0Win, p1Win, p2Win, p3Win);

            errorsDetected = errorsDetected || errorsInCurrentRegion;
        }

        this.m_currentIteration++;

        if (errorsDetected) {
            bufferedLogToConsole('Failure: Not all quad interiors seem unicolored - common-edge artifacts?');
            bufferedLogToConsole('Erroneous pixels are drawn red in the following image');
            tcuLogImage.logImage('RenderedImageWithErrors', 'Rendered image with errors marked', renderedImg.getAccess());
            tcuLogImage.logImage('ErrorsOnly', 'Image with error pixels only', errorImg.getAccess());
            testFailedOptions('Failed: iteration ' + (this.m_currentIteration - 1), false);
            return tcuTestCase.IterateResult.STOP;
        }
        else if (this.m_currentIteration < this.m_numIterations) {
            bufferedLogToConsole('Quads seem OK - moving on to next pattern');
            return tcuTestCase.IterateResult.CONTINUE;
        }
        else {
            bufferedLogToConsole('Success: All quad interiors seem unicolored (no common-edge artifacts)');
            testPassedOptions('Passed: iteration ' + (this.m_currentIteration - 1), true);
            return tcuTestCase.IterateResult.STOP;
        }
    };

    /**
     * Test that depth values are per-sample.
     *
     * Draws intersecting, differently-colored polygons and checks that there
     * are at least sample count+1 distinct colors present, due to some of the
     * samples at the intersection line belonging to one and some to another
     * polygon.
     *
     * @extends {es3fMultisampleTests.NumSamplesCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {number=} numFboSamples
     */
    es3fMultisampleTests.SampleDepthCase = function(name, desc, numFboSamples) {
        numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
        /** @type {es3fMultisampleTests.FboParams} */
        var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, true, false) : new es3fMultisampleTests.FboParams();
        es3fMultisampleTests.NumSamplesCase.call(this, name, desc, params);
    };

    es3fMultisampleTests.SampleDepthCase.prototype = Object.create(es3fMultisampleTests.NumSamplesCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.SampleDepthCase.prototype.constructor = es3fMultisampleTests.SampleDepthCase;

    es3fMultisampleTests.SampleDepthCase.prototype.init = function() {
        var inited = es3fMultisampleTests.MultisampleCase.prototype.init.call(this);
        if (!inited) {
            return false;
        }

        gl.enable(gl.DEPTH_TEST);
        gl.depthFunc(gl.LESS);

        bufferedLogToConsole('Depth test enabled, depth func is gl.LESS');
        bufferedLogToConsole('Drawing several bigger-than-viewport black or white polygons intersecting each other');
    };

    es3fMultisampleTests.SampleDepthCase.prototype.renderPattern = function() {
        gl.clearColor(0.0, 0.0, 0.0, 0.0);
        gl.clearDepth(1.0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        /** @type {number} */ var numPolygons = 50;

        for (var i = 0; i < numPolygons; i++) {
            /** @type {Array<number>} */ var color = i % 2 == 0 ? [1.0, 1.0, 1.0, 1.0] : [0.0, 0.0, 0.0, 1.0];
            /** @type {number} */ var angle = 2.0 * Math.PI * i / numPolygons + 0.001 * this.m_currentIteration;
            /** @type {Array<number>} */ var pt0 = [3.0 * Math.cos(angle + 2.0 * Math.PI * 0.0 / 3.0), 3.0 * Math.sin(angle + 2.0 * Math.PI * 0.0 / 3.0), 1.0];
            /** @type {Array<number>} */ var pt1 = [3.0 * Math.cos(angle + 2.0 * Math.PI * 1.0 / 3.0), 3.0 * Math.sin(angle + 2.0 * Math.PI * 1.0 / 3.0), 0.0];
            /** @type {Array<number>} */ var pt2 = [3.0 * Math.cos(angle + 2.0 * Math.PI * 2.0 / 3.0), 3.0 * Math.sin(angle + 2.0 * Math.PI * 2.0 / 3.0), 0.0];

            this.renderTriangle_pAsVec3WithColor(pt0, pt1, pt2, color);
        }
    };

    /**
     * Test that stencil buffer values are per-sample.
     *
     * Draws a unicolored pattern and marks drawn samples in stencil buffer;
     * then clears and draws a viewport-size quad with that color and with
     * proper stencil test such that the resulting image should be exactly the
     * same as after the pattern was first drawn.
     *
     * @extends {es3fMultisampleTests.MultisampleCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {number=} numFboSamples
     */
    es3fMultisampleTests.SampleStencilCase = function(name, desc, numFboSamples) {
        numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
        /** @type {es3fMultisampleTests.FboParams} */
        var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, true) : new es3fMultisampleTests.FboParams();
        es3fMultisampleTests.MultisampleCase.call(this, name, desc, 256, params);
    };

    es3fMultisampleTests.SampleStencilCase.prototype = Object.create(es3fMultisampleTests.MultisampleCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.SampleStencilCase.prototype.constructor = es3fMultisampleTests.SampleStencilCase;

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fMultisampleTests.SampleStencilCase.prototype.iterate = function() {
        this.randomizeViewport();

        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clearStencil(0);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        gl.enable(gl.STENCIL_TEST);
        gl.stencilFunc(gl.ALWAYS, 1, 1);
        gl.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE);

        bufferedLogToConsole('Drawing a pattern with gl.stencilFunc(gl.ALWAYS, 1, 1) and gl.stencilOp(gl.KEEP, gl.KEEP, gl.REPLACE)');

        /** @type {number} */ var numTriangles = 25;
        for (var i = 0; i < numTriangles; i++) {
            /** @type {number} */ var angle0 = 2.0 * Math.PI * i / numTriangles;
            /** @type {number} */ var angle1 = 2.0 * Math.PI * (i + 0.5) / numTriangles;

            this.renderTriangle_pAsVec2WithColor(
                [0.0, 0.0],
                [Math.cos(angle0) * 0.95, Math.sin(angle0) * 0.95],
                [Math.cos(angle1) * 0.95, Math.sin(angle1) * 0.95],
                [1.0, 1.0, 1.0, 1.0]);
        }

        /** @type {tcuSurface.Surface} */ var renderedImgFirst = this.readImage();
        tcuLogImage.logImage('RenderedImgFirst', 'First image rendered', renderedImgFirst.getAccess());
        bufferedLogToConsole('Clearing color buffer to black');

        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.stencilFunc(gl.EQUAL, 1, 1);
        gl.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP);

        bufferedLogToConsole('Checking that color buffer was actually cleared to black');

        /** @type {tcuSurface.Surface} */ var clearedImg = this.readImage();

        for (var y = 0; y < clearedImg.getHeight(); y++)
        for (var x = 0; x < clearedImg.getWidth(); x++) {
            /** @type {tcuRGBA.RGBA} */ var clr = new tcuRGBA.RGBA(clearedImg.getPixel(x, y));
            if (!clr.equals(tcuRGBA.RGBA.black)) {
                bufferedLogToConsole('Failure: first non-black pixel, color ' + clr.toString() + ', detected at coordinates (' + x + ', ' + y + ')');
                tcuLogImage.logImage('ClearedImg', 'Image after clearing, erroneously non-black', clearedImg.getAccess());
                testFailedOptions('Failed', false);
                return tcuTestCase.IterateResult.STOP;
            }
        }

        bufferedLogToConsole('Drawing a viewport-sized quad with gl.stencilFunc(gl.EQUAL, 1, 1) and gl.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP) - should result in same image as the first');

        this.renderQuad_WithColor(
            [-1.0, -1.0],
            [1.0, -1.0],
            [-1.0, 1.0],
            [1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0]);

        /** @type {tcuSurface.Surface} */ var renderedImgSecond = this.readImage();
        tcuLogImage.logImage('RenderedImgSecond', 'Second image rendered', renderedImgSecond.getAccess());
        /** @type {boolean} */
        var passed = tcuImageCompare.pixelThresholdCompare(
            'ImageCompare',
            'Image comparison',
            renderedImgFirst,
            renderedImgSecond,
            [0,0,0,0]);

        if (passed) {
            bufferedLogToConsole('Success: The two images rendered are identical');
            testPassedOptions('Passed', true);
        }
        else
            testFailedOptions('Failed', false);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * Tests coverage mask generation proportionality property.
     *
     * Tests that the number of coverage bits in a coverage mask created by
     * gl.SAMPLE_ALPHA_TO_COVERAGE or gl.SAMPLE_COVERAGE is, on average,
     * proportional to the alpha or coverage value, respectively. Draws
     * multiple frames, each time increasing the alpha or coverage value used,
     * and checks that the average color is changing appropriately.
     *
     * @extends {es3fMultisampleTests.MultisampleCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fMultisampleTests.MaskProportionalityCase.CaseType} type
     * @param {number=} numFboSamples
     */
    es3fMultisampleTests.MaskProportionalityCase = function(name, desc, type, numFboSamples) {
        numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
        /** @type {es3fMultisampleTests.FboParams} */
        var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, false) : new es3fMultisampleTests.FboParams();
        es3fMultisampleTests.MultisampleCase.call(this, name, desc, 32, params);
        /** @type {es3fMultisampleTests.MaskProportionalityCase.CaseType} */ this.m_type = type;
        /** @type {number} */ this.m_numIterations;
        /** @type {number} */ this.m_currentIteration = 0;
        /** @type {number} */ this.m_previousIterationColorSum = -1;
    };

    es3fMultisampleTests.MaskProportionalityCase.prototype = Object.create(es3fMultisampleTests.MultisampleCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.MaskProportionalityCase.prototype.constructor = es3fMultisampleTests.MaskProportionalityCase;

    /**
     * @enum {number}
     */
    es3fMultisampleTests.MaskProportionalityCase.CaseType = {
        ALPHA_TO_COVERAGE: 0,
        SAMPLE_COVERAGE: 1,
        SAMPLE_COVERAGE_INVERTED: 2
    };

    es3fMultisampleTests.MaskProportionalityCase.prototype.init = function() {
        var inited = es3fMultisampleTests.MultisampleCase.prototype.init.call(this);
        if (!inited) {
            return false;
        }

        if (this.m_type == es3fMultisampleTests.MaskProportionalityCase.CaseType.ALPHA_TO_COVERAGE) {
            gl.enable(gl.SAMPLE_ALPHA_TO_COVERAGE);
            bufferedLogToConsole('gl.SAMPLE_ALPHA_TO_COVERAGE is enabled');
        }
        else {
            assertMsgOptions(
                this.m_type == es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE ||
                this.m_type == es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE_INVERTED,
                'CaseType should be SAMPLE_COVERAGE or SAMPLE_COVERAGE_INVERTED', false, true);

            gl.enable(gl.SAMPLE_COVERAGE);
            bufferedLogToConsole('gl.SAMPLE_COVERAGE is enabled');
        }

        this.m_numIterations = Math.max(2, es3fMultisampleTests.getIterationCount(this.m_numSamples * 5));

        this.randomizeViewport(); // \note Using the same viewport for every iteration since coverage mask may depend on window-relative pixel coordinate.
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fMultisampleTests.MaskProportionalityCase.prototype.iterate = function() {
        bufferedLogToConsole('Clearing color to black');
        gl.colorMask(true, true, true, true);
        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        if (this.m_type === es3fMultisampleTests.MaskProportionalityCase.CaseType.ALPHA_TO_COVERAGE) {
            gl.colorMask(true, true, true, false);
            bufferedLogToConsole('Using color mask TRUE, TRUE, TRUE, FALSE');
        }

        // Draw quad.

        /** @type {Array<number>} */ var pt0 = [-1.0, -1.0];
        /** @type {Array<number>} */ var pt1 = [1.0, -1.0];
        /** @type {Array<number>} */ var pt2 = [-1.0, 1.0];
        /** @type {Array<number>} */ var pt3 = [1.0, 1.0];
        /** @type {Array<number>} */ var quadColor = [1.0, 0.0, 0.0, 1.0];
        /** @type {number} */ var alphaOrCoverageValue    = this.m_currentIteration / (this.m_numIterations-1);

        if (this.m_type === es3fMultisampleTests.MaskProportionalityCase.CaseType.ALPHA_TO_COVERAGE) {
            bufferedLogToConsole('Drawing a red quad using alpha value ' + alphaOrCoverageValue);
            quadColor[3] = alphaOrCoverageValue;
        }
        else {
            assertMsgOptions(
                this.m_type === es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE ||
                this.m_type === es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE_INVERTED,
                'CaseType should be SAMPLE_COVERAGE or SAMPLE_COVERAGE_INVERTED', false, true);

            /** @type {boolean} */ var isInverted = (this.m_type === es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE_INVERTED);
            /** @type {number} */ var coverageValue    = isInverted ? 1.0 - alphaOrCoverageValue : alphaOrCoverageValue;
            bufferedLogToConsole('Drawing a red quad using sample coverage value ' + coverageValue + (isInverted ? ' (inverted)' : ''));
            gl.sampleCoverage(coverageValue, isInverted ? true : false);
        }

        this.renderQuad_WithColor(pt0, pt1, pt2, pt3, quadColor);

        // Read and log image.
        /** @type {tcuSurface.Surface} */ var renderedImg = this.readImage();
        /** @type {number} */ var numPixels = renderedImg.getWidth() * renderedImg.getHeight();

        tcuLogImage.logImage('RenderedImage', 'Rendered image', renderedImg.getAccess());
        // Compute average red component in rendered image.

        /** @type {number} */ var sumRed = 0;

        for (var y = 0; y < renderedImg.getHeight(); y++)
        for (var x = 0; x < renderedImg.getWidth(); x++)
            sumRed += new tcuRGBA.RGBA(renderedImg.getPixel(x, y)).getRed();

        bufferedLogToConsole('Average red color component: ' + (sumRed / 255.0 / numPixels));

        // Check if average color has decreased from previous frame's color.

        if (sumRed < this.m_previousIterationColorSum) {
            bufferedLogToConsole('Failure: Current average red color component is lower than previous');
            testFailedOptions('Failed', false);
            return tcuTestCase.IterateResult.STOP;
        }

        // Check if coverage mask is not all-zeros if alpha or coverage value is 0 (or 1, if inverted).

        if (this.m_currentIteration == 0 && sumRed != 0)
        {
            bufferedLogToConsole('Failure: Image should be completely black');
            testFailedOptions('Failed', false);
            return tcuTestCase.IterateResult.STOP;
        }

        if (this.m_currentIteration == this.m_numIterations-1 && sumRed != 0xff*numPixels)
        {
            bufferedLogToConsole('Failure: Image should be completely red');

            testFailedOptions('Failed', false);
            return tcuTestCase.IterateResult.STOP;
        }

        this.m_previousIterationColorSum = sumRed;

        this.m_currentIteration++;

        if (this.m_currentIteration >= this.m_numIterations)
        {
            bufferedLogToConsole('Success: Number of coverage mask bits set appears to be, on average, proportional to ' +
                (this.m_type == es3fMultisampleTests.MaskProportionalityCase.CaseType.ALPHA_TO_COVERAGE ? 'alpha' :
                this.m_type == es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE ? 'sample coverage value' :
                'inverted sample coverage value'));

            testPassedOptions('Passed', true);
            return tcuTestCase.IterateResult.STOP;
        }
        else
            return tcuTestCase.IterateResult.CONTINUE;
    };

    /**
     * Tests coverage mask generation constancy property.
     *
     * Tests that the coverage mask created by gl.SAMPLE_ALPHA_TO_COVERAGE or
     * gl.SAMPLE_COVERAGE is constant at given pixel coordinates, with a given
     * alpha component or coverage value, respectively. Draws two quads, with
     * the second one fully overlapping the first one such that at any given
     * pixel, both quads have the same alpha or coverage value. This way, if
     * the constancy property is fulfilled, only the second quad should be
     * visible.
     * @extends {es3fMultisampleTests.MultisampleCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fMultisampleTests.MaskConstancyCase.CaseType} type
     * @param {number=} numFboSamples
     */
    es3fMultisampleTests.MaskConstancyCase = function(name, desc, type, numFboSamples) {
       numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
       /** @type {es3fMultisampleTests.FboParams} */
       var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, false) : new es3fMultisampleTests.FboParams();
       es3fMultisampleTests.MultisampleCase.call(this, name, desc, 256, params);
       var CaseType = es3fMultisampleTests.MaskConstancyCase.CaseType;
       /** @type {boolean} */ this.m_isAlphaToCoverageCase = (type === CaseType.ALPHA_TO_COVERAGE || type === CaseType.BOTH || type === CaseType.BOTH_INVERTED);
       /** @type {boolean} */ this.m_isSampleCoverageCase = (type === CaseType.SAMPLE_COVERAGE || type === CaseType.SAMPLE_COVERAGE_INVERTED || type === CaseType.BOTH || type === CaseType.BOTH_INVERTED);
       /** @type {boolean} */ this.m_isInvertedSampleCoverageCase = (type === CaseType.SAMPLE_COVERAGE_INVERTED || type === CaseType.BOTH_INVERTED);
    };

    es3fMultisampleTests.MaskConstancyCase.prototype = Object.create(es3fMultisampleTests.MultisampleCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.MaskConstancyCase.prototype.constructor = es3fMultisampleTests.MaskConstancyCase;

    /**
     * @enum {number}
     */
    es3fMultisampleTests.MaskConstancyCase.CaseType = {
        ALPHA_TO_COVERAGE:  0,        //!< Use only alpha-to-coverage.
        SAMPLE_COVERAGE: 1,            //!< Use only sample coverage.
        SAMPLE_COVERAGE_INVERTED: 2,    //!< Use only inverted sample coverage.
        BOTH: 3,                        //!< Use both alpha-to-coverage and sample coverage.
        BOTH_INVERTED: 4                //!< Use both alpha-to-coverage and inverted sample coverage.
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fMultisampleTests.MaskConstancyCase.prototype.iterate = function() {
        this.randomizeViewport();

        bufferedLogToConsole('Clearing color to black');
        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);

        if (this.m_isAlphaToCoverageCase) {
            gl.enable(gl.SAMPLE_ALPHA_TO_COVERAGE);
            gl.colorMask(true, true, true, false);
            bufferedLogToConsole('gl.SAMPLE_ALPHA_TO_COVERAGE is enabled');
            bufferedLogToConsole('Color mask is TRUE, TRUE, TRUE, FALSE');
        }

        if (this.m_isSampleCoverageCase) {
            gl.enable(gl.SAMPLE_COVERAGE);
            bufferedLogToConsole('gl.SAMPLE_COVERAGE is enabled');
        }

        bufferedLogToConsole('Drawing several green quads, each fully overlapped by a red quad with the same ' +
            (this.m_isAlphaToCoverageCase ? 'alpha' : '') +
            (this.m_isAlphaToCoverageCase && this.m_isSampleCoverageCase ? ' and ' : '') +
            (this.m_isInvertedSampleCoverageCase ? 'inverted ' : '') +
            (this.m_isSampleCoverageCase ? 'sample coverage' : '') +
            ' values');

        /** @type {number} */ var numQuadRowsCols = this.m_numSamples * 4;

        for (var row = 0; row < numQuadRowsCols; row++) {
            for (var col = 0; col < numQuadRowsCols; col++) {
                /** @type {number} */ var x0 = (col + 0) / numQuadRowsCols * 2.0 - 1.0;
                /** @type {number} */ var x1 = (col + 1) / numQuadRowsCols * 2.0 - 1.0;
                /** @type {number} */ var y0 = (row + 0) / numQuadRowsCols * 2.0 - 1.0;
                /** @type {number} */ var y1 = (row + 1) / numQuadRowsCols * 2.0 - 1.0;
                /** @type {Array<number>} */ var baseGreen = [0.0, 1.0, 0.0, 0.0];
                /** @type {Array<number>} */ var baseRed = [1.0, 0.0, 0.0, 0.0];
                /** @type {Array<number>} */ var alpha0 = [0.0, 0.0, 0.0, this.m_isAlphaToCoverageCase ? col / (numQuadRowsCols - 1) : 1.0];
                /** @type {Array<number>} */ var alpha1 = [0.0, 0.0, 0.0, this.m_isAlphaToCoverageCase ? row / (numQuadRowsCols - 1) : 1.0];

                if (this.m_isSampleCoverageCase) {
                    /** @type {number} */ var value = (row*numQuadRowsCols + col) / (numQuadRowsCols*numQuadRowsCols - 1);
                    gl.sampleCoverage(this.m_isInvertedSampleCoverageCase ? 1.0 - value : value, this.m_isInvertedSampleCoverageCase ? true : false);
                }

                this.renderQuad([x0, y0], [x1, y0], [x0, y1], [x1, y1],
                    deMath.add(baseGreen, alpha0), deMath.add(baseGreen, alpha1),
                    deMath.add(baseGreen, alpha0), deMath.add(baseGreen, alpha1));
                this.renderQuad([x0, y0], [x1, y0], [x0, y1], [x1, y1],
                    deMath.add(baseRed, alpha0), deMath.add(baseRed, alpha1),
                    deMath.add(baseRed, alpha0), deMath.add(baseRed, alpha1));
            }
        }

        /** @type {tcuSurface.Surface} */ var renderedImg = this.readImage();

        tcuLogImage.logImage('RenderedImage', 'Rendered image', renderedImg.getAccess());
        for (var y = 0; y < renderedImg.getHeight(); y++)
        for (var x = 0; x < renderedImg.getWidth(); x++) {
            if (new tcuRGBA.RGBA(renderedImg.getPixel(x, y)).getGreen() > 0) {
                bufferedLogToConsole('Failure: Non-zero green color component detected - should have been completely overwritten by red quad');
                testFailedOptions('Failed', false);
                return tcuTestCase.IterateResult.STOP;
            }
        }

        bufferedLogToConsole('Success: Coverage mask appears to be constant at a given pixel coordinate with a given ' +
            (this.m_isAlphaToCoverageCase ? 'alpha' : '') +
            (this.m_isAlphaToCoverageCase && this.m_isSampleCoverageCase ? ' and ' : '') +
            (this.m_isSampleCoverageCase ? 'coverage value' : ''));

        testPassedOptions('Passed', true);

        return tcuTestCase.IterateResult.STOP;
    }


    /**
     * Tests coverage mask inversion validity.
     *
     * Tests that the coverage masks obtained by glSampleCoverage(..., true)
     * and glSampleCoverage(..., false) are indeed each others' inverses.
     * This is done by drawing a pattern, with varying coverage values,
     * overlapped by a pattern that has inverted masks and is otherwise
     * identical. The resulting image is compared to one obtained by drawing
     * the same pattern but with all-ones coverage masks.
     * @extends {es3fMultisampleTests.MultisampleCase}
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {number=} numFboSamples
     */
    es3fMultisampleTests.CoverageMaskInvertCase = function(name, desc, numFboSamples) {
      numFboSamples = numFboSamples === undefined ? 0 : numFboSamples;
      /** @type {es3fMultisampleTests.FboParams} */
      var params = numFboSamples >= 0 ? new es3fMultisampleTests.FboParams(numFboSamples, false, false) : new es3fMultisampleTests.FboParams();
      es3fMultisampleTests.MultisampleCase.call(this, name, desc, 256, params);
    };

    es3fMultisampleTests.CoverageMaskInvertCase.prototype = Object.create(es3fMultisampleTests.MultisampleCase.prototype);

    /** Copy the constructor */
    es3fMultisampleTests.CoverageMaskInvertCase.prototype.constructor = es3fMultisampleTests.CoverageMaskInvertCase;

    /**
     * @param {boolean} invertSampleCoverage
     */
    es3fMultisampleTests.CoverageMaskInvertCase.prototype.drawPattern = function(invertSampleCoverage) {
        /** @type {number} */ var numTriangles = 25;
        for (var i = 0; i < numTriangles; i++) {
            gl.sampleCoverage(i / (numTriangles - 1), invertSampleCoverage ? true : false);

            /** @type {number} */ var angle0 = 2.0 * Math.PI * i / numTriangles;
            /** @type {number} */ var angle1 = 2.0 * Math.PI * (i + 0.5) / numTriangles;

            this.renderTriangle_pAsVec2WithColor(
                [0.0, 0.0],
                [Math.cos(angle0) * 0.95, Math.sin(angle0) * 0.95],
                [Math.cos(angle1) * 0.95, Math.sin(angle1) * 0.95],
                [0.4 + i / numTriangles * 0.6,
                 0.5 + i / numTriangles * 0.3,
                 0.6 - i / numTriangles * 0.5,
                 0.7 - i / numTriangles * 0.7]);
        }
    };

    /**
    * @return {tcuTestCase.IterateResult}
    */
    es3fMultisampleTests.CoverageMaskInvertCase.prototype.iterate = function() {
        this.randomizeViewport();

        gl.enable(gl.BLEND);
        gl.blendEquation(gl.FUNC_ADD);
        gl.blendFunc(gl.ONE, gl.ONE);
        bufferedLogToConsole('Additive blending enabled in order to detect (erroneously) overlapping samples');

        bufferedLogToConsole('Clearing color to all-zeros');
        gl.clearColor(0.0, 0.0, 0.0, 0.0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        bufferedLogToConsole('Drawing the pattern with gl.SAMPLE_COVERAGE disabled');
        this.drawPattern(false);
        /** @type {tcuSurface.Surface} */ var renderedImgNoSampleCoverage = this.readImage();

        tcuLogImage.logImage('RenderedImageNoSampleCoverage', 'Rendered image with gl.SAMPLE_COVERAGE disabled', renderedImgNoSampleCoverage.getAccess());
        bufferedLogToConsole('Clearing color to all-zeros');
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.enable(gl.SAMPLE_COVERAGE);
        bufferedLogToConsole('Drawing the pattern with gl.SAMPLE_COVERAGE enabled, using non-inverted masks');
        this.drawPattern(false);
        bufferedLogToConsole('Drawing the pattern with gl.SAMPLE_COVERAGE enabled, using same sample coverage values but inverted masks');
        this.drawPattern(true);
        /** @type {tcuSurface.Surface} */ var renderedImgSampleCoverage = this.readImage();

        tcuLogImage.logImage('RenderedImageSampleCoverage', 'Rendered image with gl.SAMPLE_COVERAGE enabled', renderedImgSampleCoverage.getAccess());
        /** @type {boolean} */ var passed = tcuImageCompare.pixelThresholdCompare(
            'CoverageVsNoCoverage',
            'Comparison of same pattern with gl.SAMPLE_COVERAGE disabled and enabled',
            renderedImgNoSampleCoverage,
            renderedImgSampleCoverage,
            [0, 0, 0, 0]);

        if (passed) {
            bufferedLogToConsole('Success: The two images rendered are identical');
            testPassedOptions('Passed', true);
        }
        else {
            testFailedOptions('Failed', false);
        }

        return tcuTestCase.IterateResult.STOP;
    };

    es3fMultisampleTests.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        /**
         * @enum {number}
         */
        var CaseType = {
            DEFAULT_FRAMEBUFFER: 0,
            FBO_4_SAMPLES: 1,
            FBO_8_SAMPLES: 2,
            FBO_MAX_SAMPLES: 3
        };

        for (var caseTypeI in CaseType) {
            /** @type {CaseType} */ var caseType = CaseType[caseTypeI];
            /** @type {number} */
            var numFboSamples = caseType === CaseType.DEFAULT_FRAMEBUFFER ? -1 :
                                caseType === CaseType.FBO_4_SAMPLES ? 4 :
                                caseType === CaseType.FBO_8_SAMPLES ? 8 :
                                caseType === CaseType.FBO_MAX_SAMPLES ? 0 :
                                -2;

            /** @type {?string} */
            var name = caseType === CaseType.DEFAULT_FRAMEBUFFER ? 'default_framebuffer' :
                       caseType === CaseType.FBO_4_SAMPLES ? 'fbo_4_samples' :
                       caseType === CaseType.FBO_8_SAMPLES ? 'fbo_8_samples' :
                       caseType === CaseType.FBO_MAX_SAMPLES ? 'fbo_max_samples' :
                       null;
            /** @type {?string} */
            var desc = caseType === CaseType.DEFAULT_FRAMEBUFFER ? 'Render into default framebuffer' :
                       caseType === CaseType.FBO_4_SAMPLES ? 'Render into a framebuffer object with 4 samples' :
                       caseType === CaseType.FBO_8_SAMPLES ? 'Render into a framebuffer object with 8 samples' :
                       caseType === CaseType.FBO_MAX_SAMPLES ? 'Render into a framebuffer object with the maximum number of samples' :
                       null;

            /** @type {tcuTestCase.DeqpTest} */ var group = tcuTestCase.newTest(name, desc);

            assertMsgOptions(group.name != null, 'Error: No Test Name', false, true);
            assertMsgOptions(group.description != null, 'Error: No Test Description', false, true);
            assertMsgOptions(numFboSamples >= -1, 'Assert Failed: numFboSamples >= -1', false, true);
            testGroup.addChild(group);

            group.addChild(new es3fMultisampleTests.PolygonNumSamplesCase(
                'num_samples_polygon',
                'Test sanity of the sample count, with polygons',
                numFboSamples));

            group.addChild(new es3fMultisampleTests.LineNumSamplesCase(
                'num_samples_line',
                'Test sanity of the sample count, with lines',
                numFboSamples));

            group.addChild(new es3fMultisampleTests.CommonEdgeCase(
                'common_edge_small_quads',
                'Test polygons\'s common edges with small quads',
                es3fMultisampleTests.CommonEdgeCase.CaseType.SMALL_QUADS,
                numFboSamples));

            group.addChild(new es3fMultisampleTests.CommonEdgeCase(
                'common_edge_big_quad',
                'Test polygon\'s common edges with bigger-than-viewport quads',
                es3fMultisampleTests.CommonEdgeCase.CaseType.BIGGER_THAN_VIEWPORT_QUAD,
                numFboSamples));

            group.addChild(new es3fMultisampleTests.CommonEdgeCase(
                'common_edge_viewport_quad',
                'Test polygons\' common edges with exactly viewport-sized quads',
                es3fMultisampleTests.CommonEdgeCase.CaseType.FIT_VIEWPORT_QUAD,
                numFboSamples));

            group.addChild(new es3fMultisampleTests.SampleDepthCase(
                'depth',
                'Test that depth values are per-sample',
                numFboSamples));

            group.addChild(new es3fMultisampleTests.SampleStencilCase(
                'stencil',
                'Test that stencil values are per-sample',
                numFboSamples));

            group.addChild(new es3fMultisampleTests.CoverageMaskInvertCase(
                'sample_coverage_invert',
                'Test that non-inverted and inverted sample coverage masks are each other\'s negations',
                numFboSamples));


            group.addChild(new es3fMultisampleTests.MaskProportionalityCase(
                'proportionality_alpha_to_coverage',
                'Test the proportionality property of GL_SAMPLE_ALPHA_TO_COVERAGE',
                es3fMultisampleTests.MaskProportionalityCase.CaseType.ALPHA_TO_COVERAGE,
                numFboSamples));
            group.addChild(new es3fMultisampleTests.MaskProportionalityCase(
                'proportionality_sample_coverage',
                'Test the proportionality property of GL_SAMPLE_COVERAGE',
                es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE,
                numFboSamples));
            group.addChild(new es3fMultisampleTests.MaskProportionalityCase(
                'proportionality_sample_coverage_inverted',
                'Test the proportionality property of inverted-mask GL_SAMPLE_COVERAGE',
                es3fMultisampleTests.MaskProportionalityCase.CaseType.SAMPLE_COVERAGE_INVERTED,
                numFboSamples));

            group.addChild(new es3fMultisampleTests.MaskConstancyCase(
                'constancy_alpha_to_coverage',
                'Test that coverage mask is constant at given coordinates with a given alpha or coverage value, using GL_SAMPLE_ALPHA_TO_COVERAGE',
                es3fMultisampleTests.MaskConstancyCase.CaseType.ALPHA_TO_COVERAGE,
                numFboSamples));
            group.addChild(new es3fMultisampleTests.MaskConstancyCase(
                'constancy_sample_coverage',
                'Test that coverage mask is constant at given coordinates with a given alpha or coverage value, using GL_SAMPLE_COVERAGE',
                es3fMultisampleTests.MaskConstancyCase.CaseType.SAMPLE_COVERAGE,
                numFboSamples));
            group.addChild(new es3fMultisampleTests.MaskConstancyCase(
                'constancy_sample_coverage_inverted',
                'Test that coverage mask is constant at given coordinates with a given alpha or coverage value, using inverted-mask GL_SAMPLE_COVERAGE',
                es3fMultisampleTests.MaskConstancyCase.CaseType.SAMPLE_COVERAGE_INVERTED,
                numFboSamples));
            group.addChild(new es3fMultisampleTests.MaskConstancyCase(
                'constancy_both',
                'Test that coverage mask is constant at given coordinates with a given alpha or coverage value, using GL_SAMPLE_ALPHA_TO_COVERAGE and GL_SAMPLE_COVERAGE',
                es3fMultisampleTests.MaskConstancyCase.CaseType.BOTH,
                numFboSamples));
            group.addChild(new es3fMultisampleTests.MaskConstancyCase(
                'constancy_both_inverted',
                'Test that coverage mask is constant at given coordinates with a given alpha or coverage value, using GL_SAMPLE_ALPHA_TO_COVERAGE and inverted-mask GL_SAMPLE_COVERAGE',
                es3fMultisampleTests.MaskConstancyCase.CaseType.BOTH_INVERTED,
                numFboSamples));
        }
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
     es3fMultisampleTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'multisample';
        var testDescription = 'Multisample Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.setRoot(tcuTestCase.newTest(testName, testDescription, null));

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fMultisampleTests.init();
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fMultisampleTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
