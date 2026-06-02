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
goog.provide('functional.gles3.es3fReadPixelTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {
    var es3fReadPixelTests = functional.gles3.es3fReadPixelTests;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuTexture = framework.common.tcuTexture;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var deString = framework.delibs.debase.deString;
    var deRandom = framework.delibs.debase.deRandom;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var gluShaderProgram = framework.opengl.gluShaderProgram;

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     * @param {boolean} chooseFormat
     * @param {number} alignment
     * @param {number} rowLength
     * @param {number} skipRows
     * @param {number} skipPixels
     * @param {number=} format
     * @param {number=} type
     */
    es3fReadPixelTests.ReadPixelsTest = function(name, description, chooseFormat, alignment, rowLength, skipRows, skipPixels, format, type) {
        tcuTestCase.DeqpTest.call(this, name, description);

        /** @type {number} */ this.m_seed = deString.deStringHash(name);
        /** @type {boolean} */ this.m_chooseFormat = chooseFormat;
        /** @type {number} */ this.m_alignment = alignment;
        /** @type {number} */ this.m_rowLength = rowLength;
        /** @type {number} */ this.m_skipRows = skipRows;
        /** @type {number} */ this.m_skipPixels = skipPixels;
        /** @type {number} */ this.m_format = format !== undefined ? format : gl.RGBA;
        /** @type {number} */ this.m_type = type !== undefined ? type : gl.UNSIGNED_BYTE;

        /** @const {number} */ this.m_width = 13;
        /** @const {number} */ this.m_height = 13;
    };

    es3fReadPixelTests.ReadPixelsTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fReadPixelTests.ReadPixelsTest.prototype.constructor = es3fReadPixelTests.ReadPixelsTest;

    /**
     * @param {tcuTexture.Texture2D} reference
     */
    es3fReadPixelTests.ReadPixelsTest.prototype.render = function(reference) {
        var refType = /** @type {tcuTexture.ChannelType} */ (reference.getFormat().type);
        /** @type {number} */ var width = reference.getWidth();
        /** @type {number} */ var height = reference.getHeight();
        /** @return {tcuTexture.PixelBufferAccess} */ var level0 = reference.getLevel(0);

        // Create program
        /** @type {string} */ var vertexSource = '#version 300 es\n' +
            'in mediump vec2 i_coord;\n' +
            'void main (void)\n' +
            '{\n' +
            '\tgl_Position = vec4(i_coord, 0.0, 1.0);\n' +
            '}\n';

        /** @type {string} */ var fragmentSource = '#version 300 es\n';

        if (refType === tcuTexture.ChannelType.SIGNED_INT32)
            fragmentSource += 'layout(location = 0) out mediump ivec4 o_color;\n';
        else if (refType === tcuTexture.ChannelType.UNSIGNED_INT32)
            fragmentSource += 'layout(location = 0) out mediump uvec4 o_color;\n';
        else
            fragmentSource += 'layout(location = 0) out mediump vec4 o_color;\n';

        fragmentSource += 'void main (void)\n' +
            '{\n';

        if (refType === tcuTexture.ChannelType.UNSIGNED_INT32)
            fragmentSource += '\to_color = uvec4(0, 0, 0, 1000);\n';
        else if (refType === tcuTexture.ChannelType.SIGNED_INT32)
            fragmentSource += '\to_color = ivec4(0, 0, 0, 1000);\n';
        else
            fragmentSource += '\to_color = vec4(0.0, 0.0, 0.0, 1.0);\n';

        fragmentSource += '}\n';

        /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexSource, fragmentSource));

        assertMsgOptions(program.isOk(), 'Program failed', false, true);

        gl.useProgram(program.getProgram());

        // Render
        /** @type {Array<number>} */ var coords = [
            -0.5, -0.5,
            0.5, -0.5,
            0.5, 0.5,

            0.5, 0.5,
            -0.5, 0.5,
            -0.5, -0.5
        ];
        /** @type {number} */ var coordLoc;

        coordLoc = gl.getAttribLocation(program.getProgram(), 'i_coord');

        gl.enableVertexAttribArray(coordLoc);

        /** @type {WebGLBuffer} */ var coordsGLBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, coordsGLBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(coords), gl.STATIC_DRAW);
        gl.vertexAttribPointer(coordLoc, 2, gl.FLOAT, false, 0, 0);

        gl.drawArrays(gl.TRIANGLES, 0, 6);
        gl.disableVertexAttribArray(coordLoc);

        // Render reference

        /** @type {number} */ var coordX1 = Math.floor((-0.5 * width / 2.0) + width / 2.0);
        /** @type {number} */ var coordY1 = Math.floor((-0.5 * height / 2.0) + height / 2.0);
        /** @type {number} */ var coordX2 = Math.floor((0.5 * width / 2.0) + width / 2.0);
        /** @type {number} */ var coordY2 = Math.floor((0.5 * height / 2.0) + height / 2.0);

        for (var x = 0; x < width; x++) {
            if (x < coordX1 || x > coordX2)
                continue;

            for (var y = 0; y < height; y++) {
                if (y >= coordY1 && y <= coordY2) {
                    if (refType === tcuTexture.ChannelType.SIGNED_INT32)
                        level0.setPixelInt([0, 0, 0, 1000], x, y);
                    else if (refType === tcuTexture.ChannelType.UNSIGNED_INT32)
                        level0.setPixelInt([0, 0, 0, 1000], x, y);
                    else
                        level0.setPixel([0.0, 0.0, 0.0, 1.0], x, y);
                }
            }
        }
    };

    /**
     * @return {{format: tcuTexture.TextureFormat, pixelSize: number, align: boolean}}
     */
    es3fReadPixelTests.ReadPixelsTest.prototype.getFormatInfo = function() {
        if (this.m_chooseFormat) {
            this.m_format = /** @type {number} */ (gl.getParameter(gl.IMPLEMENTATION_COLOR_READ_FORMAT));
            this.m_type = /** @type {number} */ (gl.getParameter(gl.IMPLEMENTATION_COLOR_READ_TYPE));
        }

        /** @type {tcuTexture.TextureFormat} */ var fmt = gluTextureUtil.mapGLTransferFormat(this.m_format, this.m_type);
        /** @type {boolean} */ var align_;
        switch (this.m_type) {
            case gl.BYTE:
            case gl.UNSIGNED_BYTE:
            case gl.SHORT:
            case gl.UNSIGNED_SHORT:
            case gl.INT:
            case gl.UNSIGNED_INT:
            case gl.FLOAT:
            case gl.HALF_FLOAT:
                align_ = true;
                break;

            case gl.UNSIGNED_SHORT_5_6_5:
            case gl.UNSIGNED_SHORT_4_4_4_4:
            case gl.UNSIGNED_SHORT_5_5_5_1:
            case gl.UNSIGNED_INT_2_10_10_10_REV:
            case gl.UNSIGNED_INT_10F_11F_11F_REV:
            case gl.UNSIGNED_INT_24_8:
            case gl.FLOAT_32_UNSIGNED_INT_24_8_REV:
            case gl.UNSIGNED_INT_5_9_9_9_REV:
                align_ = false;
                break;

            default:
                throw new Error('Unsupported format');
        }

        /** @type {number} */ var pxSize = fmt.getPixelSize();

        return {format: fmt, pixelSize: pxSize, align: align_};
    };

    /**
     * @param {tcuTexture.Texture2D} reference
     * @param {boolean} align
     * @param {number} pixelSize
     * @return {goog.TypedArray}
     */
    es3fReadPixelTests.ReadPixelsTest.prototype.clearColor = function(reference, align, pixelSize) {
        /** @type {number} */ var width = reference.getWidth();
        /** @type {number} */ var height = reference.getHeight();
        /** @return {tcuTexture.PixelBufferAccess} */ var level0 = reference.getLevel(0);

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(this.m_seed);
        /** @type {WebGLFramebuffer} */ var framebuffer;
        /** @type {WebGLRenderbuffer} */ var renderbuffer;
        /** @type {number} */ var red;
        /** @type {number} */ var green;
        /** @type {number} */ var blue;
        /** @type {number} */ var alpha;
        /** @type {Array<number>} */ var color;

        if (this.m_format === gl.RGBA_INTEGER) {
            if (this.m_type === gl.UNSIGNED_INT) {
                renderbuffer = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA32UI, this.m_width, this.m_height);
            } else if (this.m_type === gl.INT) {
                renderbuffer = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA32I, this.m_width, this.m_height);
            } else
                throw new Error('Type not supported');

            gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer);
            framebuffer = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, framebuffer);
            gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbuffer);
        } else if (this.m_format === gl.RGBA || /*this.m_format === gl.BGRA ||*/ this.m_format === gl.RGB) {
            // Empty
        } else
            throw new Error('Format not supported');

        gl.viewport(0, 0, width, height);

        // Clear color
        if (this.m_format === gl.RGBA || this.m_format === gl.RGB) {
            red = rnd.getFloat();
            green = rnd.getFloat();
            blue = rnd.getFloat();
            alpha = rnd.getFloat();

            color = [red, green, blue, alpha];
            // Clear target
            gl.clearColor(red, green, blue, alpha);
            bufferedLogToConsole('ClearColor: (' + red + ', ' + green + ', ' + blue + ')');

            gl.clearBufferfv(gl.COLOR, 0, color);

            // Clear reference
            level0.clear(color);
        } else if (this.m_format === gl.RGBA_INTEGER) {
            if (this.m_type === gl.INT) {
                red = Math.abs(rnd.getInt());
                green = Math.abs(rnd.getInt());
                blue = Math.abs(rnd.getInt());
                alpha = Math.abs(rnd.getInt());

                color = [red, green, blue, alpha];
                bufferedLogToConsole('ClearColor: (' + red + ', ' + green + ', ' + blue + ')');

                gl.clearBufferiv(gl.COLOR, 0, color);

                // Clear reference
                level0.clear([red, green, blue, alpha]);
            } else if (this.m_type === gl.UNSIGNED_INT) {
                red = Math.abs(rnd.getInt());
                green = Math.abs(rnd.getInt());
                blue = Math.abs(rnd.getInt());
                alpha = Math.abs(rnd.getInt());

                color = [red, green, blue, alpha];
                bufferedLogToConsole('ClearColor: (' + red + ', ' + green + ', ' + blue + ')');

                gl.clearBufferuiv(gl.COLOR, 0, color);

                // Clear reference
                level0.clear(color);
            } else
                throw new Error('Type not supported.');
        } else
            throw new Error('Format not supported.');

        this.render(reference);

        /** @type {number} */ var rowWidth = (this.m_rowLength === 0 ? this.m_width : this.m_rowLength) + this.m_skipPixels;
        /** @type {number} */ var rowPitch = (align ? this.m_alignment * Math.ceil(pixelSize * rowWidth / this.m_alignment) : rowWidth * pixelSize);

        var arrayType = tcuTexture.getTypedArray(reference.getFormat().type);
        /** @type {goog.TypedArray} */ var pixelData = new arrayType(rowPitch * (this.m_height + this.m_skipRows));
        gl.readPixels(0, 0, this.m_width, this.m_height, this.m_format, this.m_type, pixelData);

        if (framebuffer)
            gl.deleteFramebuffer(framebuffer);

        if (renderbuffer)
            gl.deleteRenderbuffer(renderbuffer);

        return pixelData;
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fReadPixelTests.ReadPixelsTest.prototype.iterate = function() {
        /** @type {tcuTexture.TextureFormat} */ var format = new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);
        /** @type {number} */ var pixelSize;
        /** @type {boolean} */ var align;

        /** @type {{format: tcuTexture.TextureFormat, pixelSize: number, align: boolean}} */ var formatInfo = this.getFormatInfo();
        format = formatInfo.format;
        align = formatInfo.align;
        pixelSize = formatInfo.pixelSize;

        bufferedLogToConsole('Format: ' + this.m_format + ', Type: ' + this.m_type);

        /** @type {tcuTexture.Texture2D} */ var reference = new tcuTexture.Texture2D(format, this.m_width, this.m_height);
        reference.allocLevel(0);
        /** @return {tcuTexture.PixelBufferAccess} */ var level0 = reference.getLevel(0);

        this.m_alignment = /** @type {number} */ (gl.getParameter(gl.PACK_ALIGNMENT));
        bufferedLogToConsole('gl.PACK_ALIGNMENT: ' + this.m_alignment);

        this.m_rowLength = /** @type {number} */ (gl.getParameter(gl.PACK_ROW_LENGTH));
        bufferedLogToConsole('gl.PACK_ROW_LENGTH: ' + this.m_rowLength);

        this.m_skipRows = /** @type {number} */ (gl.getParameter(gl.PACK_SKIP_ROWS));
        bufferedLogToConsole('gl.PACK_SKIP_ROWS: ' + this.m_skipRows);

        this.m_skipPixels = /** @type {number} */ (gl.getParameter(gl.PACK_SKIP_PIXELS));
        bufferedLogToConsole('gl.PACK_SKIP_PIXELS: ' + this.m_skipPixels);

        gl.viewport(0, 0, this.m_width, this.m_height);

        /** @type {goog.TypedArray} */ var pixelData = this.clearColor(reference, align, pixelSize);

        /** @type {number} */ var rowWidth = (this.m_rowLength === 0 ? this.m_width : this.m_rowLength);
        /** @type {number} */ var rowPitch = (align ? this.m_alignment * Math.ceil(pixelSize * rowWidth / this.m_alignment) : rowWidth * pixelSize);
        /** @type {Array<number>} */ var formatBitDepths = [];
        /** @type {number} */ var redThreshold;
        /** @type {number} */ var greenThreshold;
        /** @type {number} */ var blueThreshold;
        /** @type {number} */ var alphaThreshold;
        var redBits = /** @type {number} */ (gl.getParameter(gl.RED_BITS));
        var blueBits = /** @type {number} */ (gl.getParameter(gl.BLUE_BITS));
        var greenBits = /** @type {number} */ (gl.getParameter(gl.GREEN_BITS));
        var alphaBits = /** @type {number} */ (gl.getParameter(gl.ALPHA_BITS));
        /** @type {(tcuRGBA.RGBA|Array<number>)} */ var threshold;
        /** @type {tcuTexture.PixelBufferAccess} */ var result;
        // \note gl.RGBA_INTEGER uses always renderbuffers that are never multisampled. Otherwise default framebuffer is used.
        if (this.m_format !== gl.RGBA_INTEGER && /** @type {number} */ (gl.getParameter(gl.SAMPLES)) > 1) {
            formatBitDepths = tcuTextureUtil.getTextureFormatBitDepth(format);
            redThreshold = Math.ceil(256.0 * (2.0 / (1 << Math.min(redBits, formatBitDepths[0]))));
            greenThreshold = Math.ceil(256.0 * (2.0 / (1 << Math.min(greenBits, formatBitDepths[1]))));
            blueThreshold = Math.ceil(256.0 * (2.0 / (1 << Math.min(blueBits, formatBitDepths[2]))));
            alphaThreshold = Math.ceil(256.0 * (2.0 / (1 << Math.min(alphaBits, formatBitDepths[3]))));

            result = tcuTexture.PixelBufferAccess.newFromTextureFormat(format, this.m_width, this.m_height, 1, rowPitch, 0, pixelData.buffer);
            threshold = new tcuRGBA.RGBA([redThreshold, greenThreshold, blueThreshold, alphaThreshold]);
            if (tcuImageCompare.bilinearCompare('Result', 'Result', level0, result, threshold))
                testPassedOptions('Pass', true);
            else
                testFailedOptions('Fail', false);
        } else {
            formatBitDepths = tcuTextureUtil.getTextureFormatBitDepth(format);
            redThreshold = 2.0 / (1 << Math.min(redBits, formatBitDepths[0]));
            greenThreshold = 2.0 / (1 << Math.min(greenBits, formatBitDepths[1]));
            blueThreshold = 2.0 / (1 << Math.min(blueBits, formatBitDepths[2]));
            alphaThreshold = 2.0 / (1 << Math.min(alphaBits, formatBitDepths[3]));

            // Compare
            result = new tcuTexture.PixelBufferAccess({
                format: format,
                width: this.m_width,
                height: this.m_height,
                rowPitch: rowPitch,
                data: pixelData.buffer,
                offset: pixelSize * this.m_skipPixels + this.m_skipRows * rowPitch
            });

            threshold = [redThreshold, greenThreshold, blueThreshold, alphaThreshold];
            if (tcuImageCompare.floatThresholdCompare('Result', 'Result', level0, result, threshold))
                testPassedOptions('Pass', true);
            else
                testFailedOptions('Fail', false);
        }

        return tcuTestCase.IterateResult.STOP;
    };

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fReadPixelTests.ReadPixelTests = function() {
        tcuTestCase.DeqpTest.call(this, 'read_pixels', 'ReadPixel tests');
    };

    es3fReadPixelTests.ReadPixelTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fReadPixelTests.ReadPixelTests.prototype.constructor = es3fReadPixelTests.ReadPixelTests;

    es3fReadPixelTests.ReadPixelTests.prototype.init = function() {
        /** @type {tcuTestCase.DeqpTest} */ var groupAlignment = tcuTestCase.newTest('alignment', 'Read pixels pack alignment parameter tests');

        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_1', '', false, 1, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_2', '', false, 2, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_4', '', false, 4, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_8', '', false, 8, 0, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));

        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_1', '', false, 1, 0, 0, 0, gl.RGBA_INTEGER, gl.INT));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_2', '', false, 2, 0, 0, 0, gl.RGBA_INTEGER, gl.INT));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_4', '', false, 4, 0, 0, 0, gl.RGBA_INTEGER, gl.INT));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_8', '', false, 8, 0, 0, 0, gl.RGBA_INTEGER, gl.INT));

        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_1', '', false, 1, 0, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_2', '', false, 2, 0, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_4', '', false, 4, 0, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_8', '', false, 8, 0, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));

        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_1', '', true, 1, 0, 0, 0));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_2', '', true, 2, 0, 0, 0));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_4', '', true, 4, 0, 0, 0));
        groupAlignment.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_8', '', true, 8, 0, 0, 0));

        this.addChild(groupAlignment);

        /** @type {tcuTestCase.DeqpTest} */ var groupRowLength = tcuTestCase.newTest('rowlength', 'Read pixels rowlength test');
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_17', '', false, 4, 17, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_19', '', false, 4, 19, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_23', '', false, 4, 23, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_29', '', false, 4, 29, 0, 0, gl.RGBA, gl.UNSIGNED_BYTE));

        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_17', '', false, 4, 17, 0, 0, gl.RGBA_INTEGER, gl.INT));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_19', '', false, 4, 19, 0, 0, gl.RGBA_INTEGER, gl.INT));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_23', '', false, 4, 23, 0, 0, gl.RGBA_INTEGER, gl.INT));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_29', '', false, 4, 29, 0, 0, gl.RGBA_INTEGER, gl.INT));

        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_17', '', false, 4, 17, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_19', '', false, 4, 19, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_23', '', false, 4, 23, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_29', '', false, 4, 29, 0, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));

        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_17', '', true, 4, 17, 0, 0));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_19', '', true, 4, 19, 0, 0));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_23', '', true, 4, 23, 0, 0));
        groupRowLength.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_29', '', true, 4, 29, 0, 0));

        this.addChild(groupRowLength);

        /** @type {tcuTestCase.DeqpTest} */ var groupSkip = tcuTestCase.newTest('skip', 'Read pixels skip pixels and rows test');
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_0_3', '', false, 4, 17, 0, 3, gl.RGBA, gl.UNSIGNED_BYTE));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_3_0', '', false, 4, 17, 3, 0, gl.RGBA, gl.UNSIGNED_BYTE));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_3_3', '', false, 4, 17, 3, 3, gl.RGBA, gl.UNSIGNED_BYTE));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_ubyte_3_5', '', false, 4, 17, 3, 5, gl.RGBA, gl.UNSIGNED_BYTE));

        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_0_3', '', false, 4, 17, 0, 3, gl.RGBA_INTEGER, gl.INT));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_3_0', '', false, 4, 17, 3, 0, gl.RGBA_INTEGER, gl.INT));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_3_3', '', false, 4, 17, 3, 3, gl.RGBA_INTEGER, gl.INT));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_int_3_5', '', false, 4, 17, 3, 5, gl.RGBA_INTEGER, gl.INT));

        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_0_3', '', false, 4, 17, 0, 3, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_3_0', '', false, 4, 17, 3, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_3_3', '', false, 4, 17, 3, 3, gl.RGBA_INTEGER, gl.UNSIGNED_INT));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('rgba_uint_3_5', '', false, 4, 17, 3, 5, gl.RGBA_INTEGER, gl.UNSIGNED_INT));

        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_0_3', '', true, 4, 17, 0, 3));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_3_0', '', true, 4, 17, 3, 0));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_3_3', '', true, 4, 17, 3, 3));
        groupSkip.addChild(new es3fReadPixelTests.ReadPixelsTest('choose_3_5', '', true, 4, 17, 3, 5));

        this.addChild(groupSkip);
    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fReadPixelTests.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fReadPixelTests.ReadPixelTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fReadPixelTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
