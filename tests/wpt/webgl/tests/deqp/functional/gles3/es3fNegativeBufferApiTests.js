'use strict';
goog.provide('functional.gles3.es3fNegativeBufferApiTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluStrUtil');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {

    var es3fNegativeBufferApiTests = functional.gles3.es3fNegativeBufferApiTests;
    var es3fApiCase = functional.gles3.es3fApiCase;
    var gluStrUtil = framework.opengl.gluStrUtil;
    var tcuTestCase = framework.common.tcuTestCase;

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeBufferApiTests.init = function(gl) {

        var testGroup = tcuTestCase.runner.testCases;
        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'bind_buffer', 'Invalid gl.bindBuffer() usage', gl,
            function() {
                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the allowable values.');
                gl.bindBuffer(-1, null);
                this.expectError(gl.INVALID_ENUM);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'buffer_data', 'Invalid gl.bufferData() usage', gl,
            function() {
                var buffer = gl.createBuffer();
                gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.ARRAY_BUFFER or gl.ELEMENT_ARRAY_BUFFER.');
                gl.bufferData(-1, 0, gl.STREAM_DRAW);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if usage is not gl.STREAM_DRAW, gl.STATIC_DRAW, or gl.DYNAMIC_DRAW.');
                gl.bufferData(gl.ARRAY_BUFFER, 0, -1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if size is negative.');
                gl.bufferData(gl.ARRAY_BUFFER, -1, gl.STREAM_DRAW);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the reserved buffer object name 0 is bound to target.');
                gl.bindBuffer(gl.ARRAY_BUFFER, null);
                gl.bufferData(gl.ARRAY_BUFFER, 0, gl.STREAM_DRAW);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteBuffer(buffer);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'buffer_sub_data', 'Invalid gl.bufferSubData() usage', gl,
            function() {
                var buffer = gl.createBuffer();
                var data = new ArrayBuffer(5);
                gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
                gl.bufferData(gl.ARRAY_BUFFER, 10, gl.STREAM_DRAW);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.ARRAY_BUFFER or gl.ELEMENT_ARRAY_BUFFER.');
                gl.bufferSubData(-1, 1, data);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the reserved buffer object name 0 is bound to target.');
                gl.bindBuffer(gl.ARRAY_BUFFER, null);
                gl.bufferSubData(gl.ARRAY_BUFFER, 0, data);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteBuffer(buffer);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'buffer_sub_data_size_offset', 'Invalid gl.bufferSubData() usage', gl,
            function() {
                var buffer = gl.createBuffer();
                var data = new ArrayBuffer(5);
                gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
                gl.bufferData(gl.ARRAY_BUFFER, 10, gl.STREAM_DRAW);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if offset is negative');
                gl.bufferSubData(gl.ARRAY_BUFFER, -1, data);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if the data would be written past the end of the buffer.');
                gl.bufferSubData(gl.ARRAY_BUFFER, 7, data);
                this.expectError(gl.INVALID_VALUE);
                gl.bufferSubData(gl.ARRAY_BUFFER, 15, data);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('An exception is thrown if data is null.');
                this.expectThrowNoError(function() {
                    gl.bufferSubData(gl.ARRAY_BUFFER, 0, null);
                });

                gl.deleteBuffer(buffer);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'clear', 'Invalid gl.clear() usage', gl,
            function() {
                bufferedLogToConsole('gl.INVALID_VALUE is generated if any bit other than the three defined bits is set in mask.');
                gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
                this.expectError(gl.NO_ERROR);
                gl.clear(0x0200);
                this.expectError(gl.INVALID_VALUE);
                gl.clear(0x1000);
                this.expectError(gl.INVALID_VALUE);
                gl.clear(0x0010);
                this.expectError(gl.INVALID_VALUE);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'read_pixels', 'Invalid gl.readPixels() usage', gl,
            function() {
                var buffer = new ArrayBuffer(8);
                var ubyteData = new Uint8Array(buffer);
                var ushortData = new Uint16Array(buffer);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the combination of format and type is unsupported.');
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_SHORT_4_4_4_4, ushortData);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the ArrayBuffer type does not match the type parameter.');
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, ushortData);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if either width or height is negative.');
                gl.readPixels(0, 0, -1, 1, gl.RGBA, gl.UNSIGNED_BYTE, ubyteData);
                this.expectError(gl.INVALID_VALUE);
                gl.readPixels(0, 0, 1, -1, gl.RGBA, gl.UNSIGNED_BYTE, ubyteData);
                this.expectError(gl.INVALID_VALUE);
                gl.readPixels(0, 0, -1, -1, gl.RGBA, gl.UNSIGNED_BYTE, ubyteData);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, ubyteData);
                this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.deleteFramebuffer(fbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'read_pixels_format_mismatch', 'Invalid glReadPixels() usage', gl,
            function() {
                var buffer = new ArrayBuffer(8);
                var ubyteData = new Uint8Array(buffer);
                var ushortData = new Uint16Array(buffer);

                bufferedLogToConsole('Unsupported combinations of format and type will generate a gl.INVALID_OPERATION error.');
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_SHORT_5_6_5, ushortData);
                this.expectError(gl.INVALID_OPERATION);
                gl.readPixels(0, 0, 1, 1, gl.ALPHA, gl.UNSIGNED_SHORT_5_6_5, ushortData);
                this.expectError(gl.INVALID_OPERATION);
                gl.readPixels(0, 0, 1, 1, gl.RGB, gl.UNSIGNED_SHORT_4_4_4_4, ushortData);
                this.expectError(gl.INVALID_OPERATION);
                gl.readPixels(0, 0, 1, 1, gl.ALPHA, gl.UNSIGNED_SHORT_4_4_4_4, ushortData);
                this.expectError(gl.INVALID_OPERATION);
                gl.readPixels(0, 0, 1, 1, gl.RGB, gl.UNSIGNED_SHORT_5_5_5_1, ushortData);
                this.expectError(gl.INVALID_OPERATION);
                gl.readPixels(0, 0, 1, 1, gl.ALPHA, gl.UNSIGNED_SHORT_5_5_5_1, ushortData);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.RGBA/gl.UNSIGNED_BYTE is always accepted and the other acceptable pair can be discovered by querying gl.IMPLEMENTATION_COLOR_READ_FORMAT and gl.IMPLEMENTATION_COLOR_READ_TYPE.');
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, ubyteData);
                this.expectError(gl.NO_ERROR);
                var readFormat = /** @type {number} */ (gl.getParameter(gl.IMPLEMENTATION_COLOR_READ_FORMAT));
                var readType = /** @type {number} */ (gl.getParameter(gl.IMPLEMENTATION_COLOR_READ_TYPE));
                gl.readPixels(0, 0, 1, 1, readFormat, readType, ubyteData);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'read_pixels_fbo_format_mismatch', 'Invalid gl.readPixels() usage', gl,
            function() {
                var ubyteData = new Uint8Array(4);
                var floatData = new Float32Array(4);

                var texture = gl.createTexture();
                gl.bindTexture(gl.TEXTURE_2D, texture);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if currently bound framebuffer format is incompatible with format and type.');

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, floatData);
                this.expectError(gl.INVALID_OPERATION);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32I, 32, 32, 0, gl.RGBA_INTEGER, gl.INT, null);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, floatData);
                this.expectError(gl.INVALID_OPERATION);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32UI, 32, 32, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT, null);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);
                gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.FLOAT, floatData);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.READ_FRAMEBUFFER_BINDING is non-zero, the read framebuffer is complete, and the value of gl.SAMPLE_BUFFERS for the read framebuffer is greater than zero.');

                var rbo = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA8, 32, 32);
                gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo);

                var binding = /** @type {WebGLFramebuffer} */ (gl.getParameter(gl.READ_FRAMEBUFFER_BINDING));
                bufferedLogToConsole('gl.READ_FRAMEBUFFER_BINDING: ' + binding);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                var sampleBuffers = /** @type {number} */ (gl.getParameter(gl.SAMPLE_BUFFERS));
                bufferedLogToConsole('gl.SAMPLE_BUFFERS: ' + sampleBuffers);
                this.expectError(gl.NO_ERROR);

                if (binding == null || sampleBuffers <= 0) {
                    this.testFailed('expected gl.READ_FRAMEBUFFER_BINDING to be non-zero and gl.SAMPLE_BUFFERS to be greater than zero');
                } else {
                    gl.readPixels(0, 0, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, ubyteData);
                    this.expectError(gl.INVALID_OPERATION);
                }

                gl.bindRenderbuffer(gl.RENDERBUFFER, null);
                gl.deleteRenderbuffer(rbo);
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.deleteFramebuffer(fbo);
                gl.bindTexture(gl.TEXTURE_2D, null);
                gl.deleteTexture(texture);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'bind_buffer_range', 'Invalid glBindBufferRange() usage', gl,
            function() {
                var bufEmpty = new ArrayBuffer(16);

                var bufUniform = gl.createBuffer();
                gl.bindBuffer(gl.UNIFORM_BUFFER, bufUniform);
                gl.bufferData(gl.UNIFORM_BUFFER, bufEmpty, gl.STREAM_DRAW);

                var bufTF = gl.createBuffer();
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, bufTF);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, bufEmpty, gl.STREAM_DRAW);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.TRANSFORM_FEEDBACK_BUFFER or gl.UNIFORM_BUFFER.');
                gl.bindBufferRange(gl.ARRAY_BUFFER, 0, bufUniform, 0, 4);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if target is gl.TRANSFORM_FEEDBACK_BUFFER and index is greater than or equal to gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS.');
                var maxTFSize = /** @type {number} */ (gl.getParameter(gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS));
                gl.bindBufferRange(gl.TRANSFORM_FEEDBACK_BUFFER, maxTFSize, bufTF, 0, 4);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if target is gl.UNIFORM_BUFFER and index is greater than or equal to gl.MAX_UNIFORM_BUFFER_BINDINGS.');
                var maxUSize = /** @type {number} */ (gl.getParameter(gl.MAX_UNIFORM_BUFFER_BINDINGS));
                gl.bindBufferRange(gl.UNIFORM_BUFFER, maxUSize, bufUniform, 0, 4);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if size is less than or equal to zero.');
                gl.bindBufferRange(gl.UNIFORM_BUFFER, 0, bufUniform, 0, -1);
                this.expectError(gl.INVALID_VALUE);
                gl.bindBufferRange(gl.UNIFORM_BUFFER, 0, bufUniform, 0, 0);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if target is gl.TRANSFORM_FEEDBACK_BUFFER and size or offset are not multiples of 4.');
                gl.bindBufferRange(gl.TRANSFORM_FEEDBACK_BUFFER, 0, bufTF, 4, 5);
                this.expectError(gl.INVALID_VALUE);
                gl.bindBufferRange(gl.TRANSFORM_FEEDBACK_BUFFER, 0, bufTF, 5, 4);
                this.expectError(gl.INVALID_VALUE);
                gl.bindBufferRange(gl.TRANSFORM_FEEDBACK_BUFFER, 0, bufTF, 5, 7);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if target is gl.UNIFORM_BUFFER and offset is not a multiple of gl.UNIFORM_BUFFER_OFFSET_ALIGNMENT.');
                var alignment = /** @type {number} */ (gl.getParameter(gl.UNIFORM_BUFFER_OFFSET_ALIGNMENT));
                gl.bindBufferRange(gl.UNIFORM_BUFFER, 0, bufUniform, alignment + 1, 4);
                this.expectError(gl.INVALID_VALUE);

                gl.deleteBuffer(bufUniform);
                gl.deleteBuffer(bufTF);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'bind_buffer_base', 'Invalid glBindBufferBase() usage', gl,
            function() {
                var bufEmpty = new ArrayBuffer(16);

                var bufUniform = gl.createBuffer();
                gl.bindBuffer(gl.UNIFORM_BUFFER, bufUniform);
                gl.bufferData(gl.UNIFORM_BUFFER, bufEmpty, gl.STREAM_DRAW);

                var bufTF = gl.createBuffer();
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, bufTF);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, bufEmpty, gl.STREAM_DRAW);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.TRANSFORM_FEEDBACK_BUFFER or gl.UNIFORM_BUFFER.');
                gl.bindBufferBase(-1, 0, bufUniform);
                this.expectError(gl.INVALID_ENUM);
                gl.bindBufferBase(gl.ARRAY_BUFFER, 0, bufUniform);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if target is gl.UNIFORM_BUFFER and index is greater than or equal to gl.MAX_UNIFORM_BUFFER_BINDINGS.');
                var maxUSize = /** @type {number} */ (gl.getParameter(gl.MAX_UNIFORM_BUFFER_BINDINGS));
                gl.bindBufferBase(gl.UNIFORM_BUFFER, maxUSize, bufUniform);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if target is gl.TRANSFORM_FEEDBACK_BUFFER andindex is greater than or equal to gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS.');
                var maxTFSize = /** @type {number} */ (gl.getParameter(gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS));
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, maxTFSize, bufTF);
                this.expectError(gl.INVALID_VALUE);

                gl.deleteBuffer(bufUniform);
                gl.deleteBuffer(bufTF);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'clear_bufferiv', 'Invalid gl.clearBufferiv() usage', gl,
            function() {
                var data = new Int32Array(32 * 32);

                var texture = gl.createTexture();
                gl.bindTexture(gl.TEXTURE_2D, texture);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32I, 32, 32, 0, gl.RGBA_INTEGER, gl.INT, null);

                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is not an accepted value.');
                gl.clearBufferiv(-1, 0, data);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferiv(gl.FRAMEBUFFER, 0, data);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if buffer is gl.COLOR, gl.FRONT, gl.BACK, gl.LEFT, gl.RIGHT, or gl.FRONT_AND_BACK and drawBuffer is greater than or equal to gl.MAX_DRAW_BUFFERS.');
                var maxDrawBuffers = /** @type {number} */ (gl.getParameter(gl.MAX_DRAW_BUFFERS));
                gl.clearBufferiv(gl.COLOR, maxDrawBuffers, data);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is gl.DEPTH or gl.DEPTH_STENCIL.');
                gl.clearBufferiv(gl.DEPTH, 1, data);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferiv(gl.DEPTH_STENCIL, 1, data);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if buffer is gl.STENCIL and drawBuffer is not zero.');
                gl.clearBufferiv(gl.STENCIL, 1, data);
                this.expectError(gl.INVALID_VALUE);

                gl.deleteFramebuffer(fbo);
                gl.deleteTexture(texture);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'clear_bufferuiv', 'Invalid gl.clearBufferuiv() usage', gl,
            function() {
                var data = new Uint32Array(32 * 32);

                var texture = gl.createTexture();
                gl.bindTexture(gl.TEXTURE_2D, texture);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32UI, 32, 32, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT, null);

                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is not an accepted value.');
                gl.clearBufferuiv(-1, 0, data);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferuiv(gl.FRAMEBUFFER, 0, data);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if buffer is gl.COLOR, gl.FRONT, gl.BACK, gl.LEFT, gl.RIGHT, or gl.FRONT_AND_BACK and drawBuffer is greater than or equal to gl.MAX_DRAW_BUFFERS.');
                var maxDrawBuffers = /** @type {number} */ (gl.getParameter(gl.MAX_DRAW_BUFFERS));
                gl.clearBufferuiv(gl.COLOR, maxDrawBuffers, data);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is gl.DEPTH, gl.STENCIL or gl.DEPTH_STENCIL.');
                gl.clearBufferuiv(gl.DEPTH, 1, data);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferuiv(gl.STENCIL, 1, data);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferuiv(gl.DEPTH_STENCIL, 1, data);
                this.expectError(gl.INVALID_ENUM);

                gl.deleteFramebuffer(fbo);
                gl.deleteTexture(texture);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'clear_bufferfv', 'Invalid gl.clearBufferfv() usage', gl,
            function() {
                var data = new Float32Array(32 * 32);

                var texture = gl.createTexture();
                // Float type texture isn't color-renderable without EXT_color_buffer_float extension.
                if (gl.getExtension('EXT_color_buffer_float')) {
                    gl.bindTexture(gl.TEXTURE_2D, texture);
                    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, 32, 32, 0, gl.RGBA, gl.FLOAT, null);

                    var fbo = gl.createFramebuffer();
                    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                    gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                    this.expectError(gl.NO_ERROR);

                    bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is not an accepted value.');
                    gl.clearBufferfv(-1, 0, data);
                    this.expectError(gl.INVALID_ENUM);
                    gl.clearBufferfv(gl.FRAMEBUFFER, 0, data);
                    this.expectError(gl.INVALID_ENUM);

                    bufferedLogToConsole('gl.INVALID_VALUE is generated if buffer is gl.COLOR, gl.FRONT, gl.BACK, gl.LEFT, gl.RIGHT, or gl.FRONT_AND_BACK and drawBuffer is greater than or equal to gl.MAX_DRAW_BUFFERS.');
                    var maxDrawBuffers = /** @type {number} */ (gl.getParameter(gl.MAX_DRAW_BUFFERS));
                    gl.clearBufferfv(gl.COLOR, maxDrawBuffers, data);
                    this.expectError(gl.INVALID_VALUE);

                    bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is gl.STENCIL or gl.DEPTH_STENCIL.');
                    gl.clearBufferfv(gl.STENCIL, 1, data);
                    this.expectError(gl.INVALID_ENUM);
                    gl.clearBufferfv(gl.DEPTH_STENCIL, 1, data);
                    this.expectError(gl.INVALID_ENUM);

                    bufferedLogToConsole('gl.INVALID_VALUE is generated if buffer is gl.DEPTH and drawBuffer is not zero.');
                    gl.clearBufferfv(gl.DEPTH, 1, data);
                    this.expectError(gl.INVALID_VALUE);
                }

                gl.deleteFramebuffer(fbo);
                gl.deleteTexture(texture);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'clear_bufferfi', 'Invalid gl.clearBufferfi() usage', gl,
            function() {
                bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is not an accepted value.');
                gl.clearBufferfi(-1, 0, 1.0, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferfi(gl.FRAMEBUFFER, 0, 1.0, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if buffer is not gl.DEPTH_STENCIL.');
                gl.clearBufferfi(gl.DEPTH, 0, 1.0, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferfi(gl.STENCIL, 0, 1.0, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.clearBufferfi(gl.COLOR, 0, 1.0, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if buffer is gl.DEPTH_STENCIL and drawBuffer is not zero.');
                gl.clearBufferfi(gl.DEPTH_STENCIL, 1, 1.0, 1);
                this.expectError(gl.INVALID_VALUE);

            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'copy_buffer_sub_data', 'Invalid gl.copyBufferSubData() usage', gl,
            function() {
                var buf = {
                    r: gl.createBuffer(),
                    w: gl.createBuffer()
                };

                gl.bindBuffer(gl.COPY_READ_BUFFER, buf.r);
                gl.bufferData(gl.COPY_READ_BUFFER, 32, gl.DYNAMIC_COPY);
                gl.bindBuffer(gl.COPY_WRITE_BUFFER, buf.w);
                gl.bufferData(gl.COPY_WRITE_BUFFER, 32, gl.DYNAMIC_COPY);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if any of readoffset, writeoffset or size is negative.');
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, -4);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, -1, 0, 4);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, -1, 4);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if readoffset + size exceeds the size of the buffer object bound to readtarget.');
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, 36);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 24, 0, 16);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 36, 0, 4);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if writeoffset + size exceeds the size of the buffer object bound to writetarget.');
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, 36);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 24, 16);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 36, 4);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if the same buffer object is bound to both readtarget and writetarget and the ranges [readoffset, readoffset + size) and [writeoffset, writeoffset + size) overlap.');
                gl.bindBuffer(gl.COPY_WRITE_BUFFER, buf.r);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 16, 4);
                this.expectError(gl.NO_ERROR);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, 4);
                this.expectError(gl.INVALID_VALUE);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 16, 18);
                this.expectError(gl.INVALID_VALUE);
                gl.bindBuffer(gl.COPY_WRITE_BUFFER, buf.w);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if null is bound to readtarget or writetarget.');
                gl.bindBuffer(gl.COPY_READ_BUFFER, null);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, 16);
                this.expectError(gl.INVALID_OPERATION);
                gl.bindBuffer(gl.COPY_READ_BUFFER, buf.r);

                gl.bindBuffer(gl.COPY_WRITE_BUFFER, null);
                gl.copyBufferSubData(gl.COPY_READ_BUFFER, gl.COPY_WRITE_BUFFER, 0, 0, 16);
                this.expectError(gl.INVALID_OPERATION);
                gl.bindBuffer(gl.COPY_WRITE_BUFFER, buf.w);

                gl.deleteBuffer(buf.w);
                gl.deleteBuffer(buf.r);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'draw_buffers', 'Invalid glDrawBuffers() usage', gl,
            function() {
                var maxDrawBuffers = /** @type {number} */ (gl.getParameter(gl.MAX_DRAW_BUFFERS));
                var values = [
                    gl.NONE,
                    gl.BACK,
                    gl.COLOR_ATTACHMENT0,
                    gl.DEPTH_ATTACHMENT
                ];

                var texture = gl.createTexture();
                gl.bindTexture(gl.TEXTURE_2D, texture);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if one of the values in bufs is not an accepted value.');
                gl.drawBuffers(values.slice(2, 4));
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the GL is bound to the default framebuffer and the number of queried buffers is not 1.');
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.drawBuffers(values.slice(0, 2));
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the GL is bound to the default framebuffer and the value in bufs is one of the gl.COLOR_ATTACHMENTn tokens.');
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.drawBuffers(values.slice(2, 3));
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the GL is bound to a framebuffer object and the ith buffer listed in bufs is anything other than gl.NONE or gl.COLOR_ATTACHMENTSi.');
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.drawBuffers(values.slice(1, 2));
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteTexture(texture);
                gl.deleteFramebuffer(fbo);
            }
        ));

        // Framebuffer Objects

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'bind_framebuffer', 'Invalid glBindFramebuffer() usage', gl,
            function() {
                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.FRAMEBUFFER.');
                gl.bindFramebuffer(-1, null);
                this.expectError(gl.INVALID_ENUM);
                gl.bindFramebuffer(gl.RENDERBUFFER, null);
                this.expectError(gl.INVALID_ENUM);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'bind_renderbuffer', 'Invalid glBindRenderbuffer() usage', gl,
            function() {
                bufferedLogToConsole('glINVALID_ENUM is generated if target is not gl.RENDERBUFFER.');
                gl.bindRenderbuffer(-1, null);
                this.expectError(gl.INVALID_ENUM);
                gl.bindRenderbuffer(gl.FRAMEBUFFER, null);
                this.expectError(gl.INVALID_ENUM);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'check_framebuffer_status', 'Invalid glCheckFramebufferStatus() usage', gl,
            function() {
                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.FRAMEBUFFER.');
                    gl.checkFramebufferStatus(-1);
                    this.expectError(gl.INVALID_ENUM);
                    gl.checkFramebufferStatus(gl.RENDERBUFFER);
                    this.expectError(gl.INVALID_ENUM);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'framebuffer_renderbuffer', 'Invalid glFramebufferRenderbuffer() usage', gl,
            function() {
                var rbo = gl.createRenderbuffer();
                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted tokens.');
                gl.framebufferRenderbuffer(-1, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if renderbuffertarget is not gl.RENDERBUFFER.');
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
                gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, -1, rbo);
                this.expectError(gl.INVALID_ENUM);
                gl.bindRenderbuffer(gl.RENDERBUFFER, null);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if zero is bound to target.');
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, null);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteRenderbuffer(rbo);
                gl.deleteFramebuffer(fbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'framebuffer_texture2d', 'Invalid glFramebufferTexture2D() usage', gl,
            function() {

                var fbo = gl.createFramebuffer();
                var tex2D = gl.createTexture();
                var texCube = gl.createTexture();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
                gl.bindTexture(gl.TEXTURE_2D, tex2D);
                gl.bindTexture(gl.TEXTURE_CUBE_MAP, texCube);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted tokens.');
                gl.framebufferTexture2D(-1, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if textarget is not an accepted texture target.');
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, -1, tex2D, 0);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if level is less than 0 or larger than log_2 of maximum texture size.');
                var maxTexSize = /** @type {number} */ (gl.getParameter(gl.MAX_TEXTURE_SIZE));
                var maxCubeTexSize = /** @type {number} */ (gl.getParameter(gl.MAX_CUBE_MAP_TEXTURE_SIZE));
                var maxSizePlane = Math.floor(Math.log2(maxTexSize)) + 1;
                var maxSizeCube = Math.floor(Math.log2(maxCubeTexSize)) + 1;
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex2D, -1);
                this.expectError(gl.INVALID_VALUE);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex2D, maxSizePlane);
                this.expectError(gl.INVALID_VALUE);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_CUBE_MAP_POSITIVE_X, texCube, maxSizeCube);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if textarget and texture are not compatible.');
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_CUBE_MAP_POSITIVE_X, tex2D, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.deleteTexture(tex2D);

                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texCube, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.deleteTexture(texCube);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if zero is bound to target.');
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, null, 0);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteFramebuffer(fbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'renderbuffer_storage', 'Invalid glRenderbufferStorage() usage', gl,
            function() {
                var rbo = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.RENDERBUFFER.');
                gl.renderbufferStorage(-1, gl.RGBA4, 1, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.renderbufferStorage(gl.FRAMEBUFFER, gl.RGBA4, 1, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if internalformat is not a color-renderable, depth-renderable, or stencil-renderable format.');
                gl.renderbufferStorage(gl.RENDERBUFFER, -1, 1, 1);
                this.expectError(gl.INVALID_ENUM);

                // EXT_color_buffer_half_float disables error
                if (gl.getExtension('EXT_color_buffer_half_float') === null) {
                    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGB16F, 1, 1);
                    this.expectError(gl.INVALID_ENUM);
                }

                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8_SNORM, 1, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than zero.');
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, -1, 1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 1, -1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, -1, -1);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is greater than gl.MAX_RENDERBUFFER_SIZE.');
                var maxSize = /** @type {number} */ (gl.getParameter(gl.MAX_RENDERBUFFER_SIZE));
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 1, maxSize + 1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, maxSize + 1, 1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, maxSize + 1, maxSize + 1);
                this.expectError(gl.INVALID_VALUE);
                gl.deleteRenderbuffer(rbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'blit_framebuffer', 'Invalid glBlitFramebuffer() usage', gl,
            function() {

                /** @type {Array<WebGLTexture>} */
                var texture = [
                    gl.createTexture(), gl.createTexture()
                ];
                gl.bindTexture(gl.TEXTURE_2D, texture[0]);

                /** @type {Array<WebGLRenderbuffer>} */
                var rbo = [
                    gl.createRenderbuffer(), gl.createRenderbuffer()
                ];
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo[0]);

                /** @type {Array<WebGLFramebuffer>} */
                var fbo = [
                    gl.createFramebuffer(), gl.createFramebuffer()
                ];
                gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fbo[0]);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH24_STENCIL8, 32, 32);
                gl.framebufferTexture2D(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[0], 0);
                gl.framebufferRenderbuffer(gl.READ_FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, rbo[0]);
                gl.checkFramebufferStatus(gl.READ_FRAMEBUFFER);
                gl.bindTexture(gl.TEXTURE_2D, texture[1]);
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo[1]);
                gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, fbo[1]);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH24_STENCIL8, 32, 32);
                gl.framebufferTexture2D(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[1], 0);
                gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, rbo[1]);
                gl.checkFramebufferStatus(gl.DRAW_FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if mask contains any of the gl.DEPTH_BUFFER_BIT or gl.STENCIL_BUFFER_BIT and filter is not gl.NEAREST.');
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT | gl.STENCIL_BUFFER_BIT, gl.LINEAR);
                this.expectError(gl.INVALID_OPERATION);
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT, gl.LINEAR);
                this.expectError(gl.INVALID_OPERATION);
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT, gl.LINEAR);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if mask contains gl.COLOR_BUFFER_BIT and read buffer format is incompatible with draw buffer format.');
                gl.bindTexture(gl.TEXTURE_2D, texture[0]);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32UI, 32, 32, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT, null);
                gl.framebufferTexture2D(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[0], 0);
                bufferedLogToConsole('// Read buffer: gl.RGBA32UI, draw buffer: gl.RGBA');
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32I, 32, 32, 0, gl.RGBA_INTEGER, gl.INT, null);
                gl.framebufferTexture2D(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[0], 0);
                bufferedLogToConsole('// Read buffer: gl.RGBA32I, draw buffer: gl.RGBA');
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                gl.framebufferTexture2D(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[0], 0);
                gl.bindTexture(gl.TEXTURE_2D, texture[1]);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32I, 32, 32, 0, gl.RGBA_INTEGER, gl.INT, null);
                gl.framebufferTexture2D(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[1], 0);
                bufferedLogToConsole('// Read buffer: gl.RGBA8, draw buffer: gl.RGBA32I');
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if filter is gl.LINEAR and the read buffer contains integer data.');
                gl.bindTexture(gl.TEXTURE_2D, texture[0]);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32UI, 32, 32, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT, null);
                gl.framebufferTexture2D(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[0], 0);
                gl.bindTexture(gl.TEXTURE_2D, texture[1]);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32UI, 32, 32, 0, gl.RGBA_INTEGER, gl.UNSIGNED_INT, null);
                gl.framebufferTexture2D(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture[1], 0);
                bufferedLogToConsole('// Read buffer: gl.RGBA32UI, filter: gl.LINEAR');
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT, gl.LINEAR);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if mask contains gl.DEPTH_BUFFER_BIT or gl.STENCIL_BUFFER_BIT and the source and destination depth and stencil formats do not match.');
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo[0]);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH32F_STENCIL8, 32, 32);
                gl.framebufferRenderbuffer(gl.READ_FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.RENDERBUFFER, rbo[0]);
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.DEPTH_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.STENCIL_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.deleteFramebuffer(fbo[1]);
                gl.deleteFramebuffer(fbo[0]);
                gl.deleteTexture(texture[1]);
                gl.deleteTexture(texture[0]);
                gl.deleteRenderbuffer(rbo[1]);
                gl.deleteRenderbuffer(rbo[0]);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'blit_framebuffer_multisample', 'Invalid glBlitFramebuffer() usage', gl,
            function() {

                /** @type {Array<WebGLRenderbuffer>} */
                var rbo = [
                    gl.createRenderbuffer(), gl.createRenderbuffer()
                ];
                /** @type {Array<WebGLFramebuffer>} */
                var fbo = [
                    gl.createFramebuffer(), gl.createFramebuffer()
                ];

                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo[0]);
                gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fbo[0]);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA8, 32, 32);
                gl.framebufferRenderbuffer(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo[0]);
                gl.checkFramebufferStatus(gl.READ_FRAMEBUFFER);

                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo[1]);
                gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, fbo[1]);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the value of gl.SAMPLE_BUFFERS for the draw buffer is greater than zero.');
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA8, 32, 32);
                gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo[1]);
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.SAMPLE_BUFFERS for the read buffer is greater than zero and the formats of draw and read buffers are not identical.');
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, 32, 32);
                gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo[1]);
                gl.blitFramebuffer(0, 0, 16, 16, 0, 0, 16, 16, gl.COLOR_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.SAMPLE_BUFFERS for the read buffer is greater than zero and the source and destination rectangles are not defined with the same (X0, Y0) and (X1, Y1) bounds.');
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 32, 32);
                gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo[1]);
                gl.blitFramebuffer(0, 0, 16, 16, 2, 2, 18, 18, gl.COLOR_BUFFER_BIT, gl.NEAREST);
                this.expectError(gl.INVALID_OPERATION);

                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.deleteRenderbuffer(rbo[0]);
                gl.deleteRenderbuffer(rbo[1]);
                gl.deleteFramebuffer(fbo[0]);
                gl.deleteFramebuffer(fbo[1]);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'framebuffer_texture_layer', 'Invalid glFramebufferTextureLayer() usage', gl,
            function() {

                var fbo = gl.createFramebuffer();
                var tex3D = gl.createTexture();
                var tex2DArray = gl.createTexture();
                var tex2D = gl.createTexture();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

                gl.bindTexture(gl.TEXTURE_3D, tex3D);
                gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                gl.bindTexture(gl.TEXTURE_2D_ARRAY, tex2DArray);
                gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                gl.bindTexture(gl.TEXTURE_2D, tex2D);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted tokens.');
                gl.framebufferTextureLayer(-1, gl.COLOR_ATTACHMENT0, tex3D, 0, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.framebufferTextureLayer(gl.RENDERBUFFER, gl.COLOR_ATTACHMENT0, tex3D, 0, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if attachment is not one of the accepted tokens.');
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, -1, tex3D, 0, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.BACK, tex3D, 0, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if texture is non-zero and not the name of a 3D texture or 2D array texture.');
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex2D, 0, 0);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if texture is not zero and layer is negative.');
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex3D, 0, -1);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if texture is not zero and layer is greater than gl.MAX_3D_TEXTURE_SIZE-1 for a 3D texture.');
                var max3DTexSize = /** @type {number} */ (gl.getParameter(gl.MAX_3D_TEXTURE_SIZE));
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex3D, 0, max3DTexSize);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if texture is not zero and layer is greater than gl.MAX_ARRAY_TEXTURE_LAYERS-1 for a 2D array texture.');
                var maxArrayTexLayers = /** @type {number} */ (gl.getParameter(gl.MAX_ARRAY_TEXTURE_LAYERS));
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex2DArray, 0, maxArrayTexLayers);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if zero is bound to target.');
                gl.bindFramebuffer(gl.FRAMEBUFFER, null);
                gl.framebufferTextureLayer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, tex3D, 0, 1);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteTexture(tex3D);
                gl.deleteTexture(tex2DArray);
                gl.deleteTexture(tex2D);
                gl.deleteFramebuffer(fbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'invalidate_framebuffer', 'Invalid gl.invalidateFramebuffer() usage', gl,
            function() {
                var maxColorAttachments = /** @type {number} */ (gl.getParameter(gl.MAX_COLOR_ATTACHMENTS));
                var attachments = [
                    gl.COLOR_ATTACHMENT0
                ];

                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

                var texture = gl.createTexture();
                gl.bindTexture(gl.TEXTURE_2D, texture);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.FRAMEBUFFER, gl.READ_FRAMEBUFFER or gl.DRAW_FRAMEBUFFER.');
                gl.invalidateFramebuffer(-1, attachments);
                this.expectError(gl.INVALID_ENUM);
                gl.invalidateFramebuffer(gl.BACK, attachments);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if attachments contains gl.COLOR_ATTACHMENTm and m is greater than or equal to the value of gl.MAX_COLOR_ATTACHMENTS.');
                gl.invalidateFramebuffer(gl.FRAMEBUFFER, [gl.COLOR_ATTACHMENT0 + maxColorAttachments]);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteTexture(texture);
                gl.deleteFramebuffer(fbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'invalidate_sub_framebuffer', 'Invalid gl.invalidateSubFramebuffer() usage', gl,
            function() {
                var fbo = gl.createFramebuffer();
                gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

                var texture = gl.createTexture();
                gl.bindTexture(gl.TEXTURE_2D, texture);
                gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 32, 32, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);

                gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
                gl.checkFramebufferStatus(gl.FRAMEBUFFER);
                this.expectError(gl.NO_ERROR);

                var maxColorAttachments = /** @type {number} */ (gl.getParameter(gl.MAX_COLOR_ATTACHMENTS));
                var att0 = [gl.COLOR_ATTACHMENT0];
                var attm = [gl.COLOR_ATTACHMENT0 + maxColorAttachments];

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.FRAMEBUFFER, gl.READ_FRAMEBUFFER or gl.DRAW_FRAMEBUFFER.');
                gl.invalidateSubFramebuffer(-1, att0, 0, 0, 16, 16);
                this.expectError(gl.INVALID_ENUM);
                gl.invalidateSubFramebuffer(gl.BACK, att0, 0, 0, 16, 16);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if attachments contains gl.COLOR_ATTACHMENTm and m is greater than or equal to the value of gl.MAX_COLOR_ATTACHMENTS.');
                gl.invalidateSubFramebuffer(gl.FRAMEBUFFER, attm, 0, 0, 16, 16);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteFramebuffer(fbo);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'renderbuffer_storage_multisample', 'Invalid glRenderbufferStorageMultisample() usage', gl,
            function() {

                var rbo = gl.createRenderbuffer();
                gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
                /** @type {Int32Array} */ var samplesSupportedRGBA4 = /** @type {Int32Array} */ gl.getInternalformatParameter(gl.RENDERBUFFER, gl.RGBA4, gl.SAMPLES);
                // supported samples are written in descending numeric order, so the first one is the max samples
                var maxSamplesSupportedRGBA4 = samplesSupportedRGBA4[0];

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.RENDERBUFFER.');
                gl.renderbufferStorageMultisample(-1, 2, gl.RGBA4, 1, 1);
                this.expectError(gl.INVALID_ENUM);
                gl.renderbufferStorageMultisample(gl.FRAMEBUFFER, 2, gl.RGBA4, 1, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if samples is greater than the maximum number of samples supported for internalformat.');
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, maxSamplesSupportedRGBA4 + 1, gl.RGBA4, 1, 1);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if internalformat is not a color-renderable, depth-renderable, or stencil-renderable format.');
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 2, -1, 1, 1);
                this.expectError(gl.INVALID_ENUM);

                // EXT_color_buffer_half_float disables error
                if (gl.getExtension('EXT_color_buffer_half_float') === null) {
                    gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 2, gl.RGB16F, 1, 1);
                    this.expectError(gl.INVALID_ENUM);
                }
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 2, gl.RGBA8_SNORM, 1, 1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is less than zero.');
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 2, gl.RGBA4, -1, 1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 2, gl.RGBA4, 1, -1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 2, gl.RGBA4, -1, -1);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if width or height is greater than gl.MAX_RENDERBUFFER_SIZE.');
                var maxSize = /** @type {number} */ (gl.getParameter(gl.MAX_RENDERBUFFER_SIZE));
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA4, 1, maxSize + 1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA4, maxSize + 1, 1);
                this.expectError(gl.INVALID_VALUE);
                gl.renderbufferStorageMultisample(gl.RENDERBUFFER, 4, gl.RGBA4, maxSize + 1, maxSize + 1);
                this.expectError(gl.INVALID_VALUE);

                gl.deleteRenderbuffer(rbo);
            }
        ));

    };

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeBufferApiTests.run = function(gl) {
        var testName = 'negativeBufferApi';
        var testDescription = 'Negative Buffer API tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fNegativeBufferApiTests.init(gl);
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };

});
