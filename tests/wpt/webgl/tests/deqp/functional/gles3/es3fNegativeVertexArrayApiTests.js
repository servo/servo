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
 * \brief Negative Vertex Array API tests.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('functional.gles3.es3fNegativeVertexArrayApiTests');

goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('functional.gles3.es3fApiCase');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.simplereference.sglrGLContext');

goog.scope(function() {

    var es3fNegativeVertexArrayApiTests = functional.gles3.es3fNegativeVertexArrayApiTests;
    var tcuTexture = framework.common.tcuTexture;
    var es3fApiCase = functional.gles3.es3fApiCase;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;

    /**
     * @type {string}
     * @const
     */
    var vertexShaderSource = '#version 300 es\n' +
    'void main (void)\n' +
    '{\n' +
    ' gl_Position = vec4(0.0);\n' +
    '}\n';

    /**
     * @type {string}
     * @const
     */
    var fragmentShaderSource = '#version 300 es\n' +
    'layout(location = 0) out mediump vec4 fragColor;\n' +
    'void main (void)\n' +
    '{\n' +
    ' fragColor = vec4(0.0);\n' +
    '}\n';

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeVertexArrayApiTests.init = function(gl) {

        var testGroup = tcuTestCase.runner.testCases;

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attribf', 'Invalid glVertexAttrib{1234}f() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            gl.vertexAttrib1f(maxVertexAttribs, 0.0);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttrib2f(maxVertexAttribs, 0.0, 0.0);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttrib3f(maxVertexAttribs, 0.0, 0.0, 0.0);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttrib4f(maxVertexAttribs, 0.0, 0.0, 0.0, 0.0);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attribfv', 'Invalid glVertexAttrib{1234}fv() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            /** @type{Array<number>} */ var v = [0.0];
            gl.vertexAttrib1fv(maxVertexAttribs, v);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttrib2fv(maxVertexAttribs, v);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttrib3fv(maxVertexAttribs, v);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttrib4fv(maxVertexAttribs, v);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attribi4', 'Invalid glVertexAttribI4{i|ui}f() usage', gl, function() {
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            /** @type{number} */ var valInt = 0;
            /** @type{number} */ var valUint = 0;

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            gl.vertexAttribI4i(maxVertexAttribs, valInt, valInt, valInt, valInt);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttribI4ui(maxVertexAttribs, valUint, valUint, valUint, valUint);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attribi4v', 'Invalid glVertexAttribI4{i|ui}fv() usage', gl, function() {
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            /** @type{Array<number>} */ var valInt = [0];
            /** @type{Array<number>} */ var valUint = [0];

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            gl.vertexAttribI4iv(maxVertexAttribs, valInt);
            this.expectError(gl.INVALID_VALUE);
            gl.vertexAttribI4uiv(maxVertexAttribs, valUint);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attrib_pointer', 'Invalid gl.vertexAttribPointer() usage', gl, function() {
            /** @type{WebGLBuffer} */ var buffer = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not an accepted value.');
            gl.vertexAttribPointer(0, 1, 0, true, 0, 0);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            gl.vertexAttribPointer(maxVertexAttribs, 1, gl.BYTE, true, 0, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if size is not 1, 2, 3, or 4.');
            gl.vertexAttribPointer(0, 0, gl.BYTE, true, 0, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if stride is negative.');
            gl.vertexAttribPointer(0, 1, gl.BYTE, true, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if type is gl.INT_2_10_10_10_REV or gl.UNSIGNED_INT_2_10_10_10_REV and size is not 4.');
            gl.vertexAttribPointer(0, 2, gl.INT_2_10_10_10_REV, true, 0, 0);
            this.expectError(gl.INVALID_OPERATION);
            gl.vertexAttribPointer(0, 2, gl.UNSIGNED_INT_2_10_10_10_REV, true, 0, 0);
            this.expectError(gl.INVALID_OPERATION);
            gl.vertexAttribPointer(0, 4, gl.INT_2_10_10_10_REV, true, 0, 0);
            this.expectError(gl.NO_ERROR);
            gl.vertexAttribPointer(0, 4, gl.UNSIGNED_INT_2_10_10_10_REV, true, 0, 0);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated a non-zero vertex array object is bound, zero is bound to the gl.ARRAY_BUFFER buffer object binding point and the pointer argument is not NULL.');
            /** @type{WebGLVertexArrayObject} */ var vao;
            /** @type{number} */ var offset = 1;
            vao = gl.createVertexArray();
            gl.bindVertexArray(vao);
            gl.bindBuffer(gl.ARRAY_BUFFER, null);
            this.expectError(gl.NO_ERROR);

            gl.vertexAttribPointer(0, 1, gl.BYTE, true, 0, offset);
            this.expectError(gl.INVALID_OPERATION);

            gl.bindVertexArray(null);
            gl.deleteVertexArray(vao);
            this.expectError(gl.NO_ERROR);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attrib_i_pointer', 'Invalid gl.vertexAttribIPointer() usage', gl, function() {
            /** @type{WebGLBuffer} */ var buffer = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not an accepted value.');
            gl.vertexAttribIPointer(0, 1, 0, 0, 0);
            this.expectError(gl.INVALID_ENUM);
            gl.vertexAttribIPointer(0, 4, gl.INT_2_10_10_10_REV, 0, 0);
            this.expectError(gl.INVALID_ENUM);
            gl.vertexAttribIPointer(0, 4, gl.UNSIGNED_INT_2_10_10_10_REV, 0, 0);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            gl.vertexAttribIPointer(maxVertexAttribs, 1, gl.BYTE, 0, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if size is not 1, 2, 3, or 4.');
            gl.vertexAttribIPointer(0, 0, gl.BYTE, 0, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if stride is negative.');
            gl.vertexAttribIPointer(0, 1, gl.BYTE, -1, 0);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated a non-zero vertex array object is bound, zero is bound to the gl.ARRAY_BUFFER buffer object binding point and the pointer argument is not NULL.');
            /** @type{WebGLVertexArrayObject} */ var vao;
            /** @type{number} */ var offset = 1;
            vao = gl.createVertexArray();
            gl.bindVertexArray(vao);
            gl.bindBuffer(gl.ARRAY_BUFFER, null);
            this.expectError(gl.NO_ERROR);

            gl.vertexAttribIPointer(0, 1, gl.BYTE, 0, offset);
            this.expectError(gl.INVALID_OPERATION);

            gl.bindVertexArray(null);
            gl.deleteVertexArray(vao);
            this.expectError(gl.NO_ERROR);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('enable_vertex_attrib_array', 'Invalid gl.enableVertexAttribArray() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            gl.enableVertexAttribArray(maxVertexAttribs);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('disable_vertex_attrib_array', 'Invalid gl.disableVertexAttribArray() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            gl.disableVertexAttribArray(maxVertexAttribs);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('vertex_attrib_divisor', 'Invalid gl.vertexAttribDivisor() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            var maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            gl.vertexAttribDivisor(maxVertexAttribs, 0);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_arrays', 'Invalid gl.drawArrays() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawArrays(-1, 0, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count is negative.');
            gl.drawArrays(gl.POINTS, 0, -1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawArrays(gl.POINTS, 0, 1);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_arrays_invalid_program', 'Invalid gl.drawArrays() usage', gl, function() {
            gl.useProgram(null);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.useProgram(null) is used.');
            gl.drawArrays(gl.POINTS, 0, 1);
            this.expectError(gl.INVALID_OPERATION);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_arrays_incomplete_primitive', 'Invalid gl.drawArrays() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawArrays(-1, 0, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count is negative.');
            gl.drawArrays(gl.TRIANGLES, 0, -1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawArrays(gl.TRIANGLES, 0, 1);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_elements', 'Invalid gl.drawElements() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{WebGLBuffer} */ var buf;
            /** @type{WebGLTransformFeedback} */ var tfID;
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawElements(-1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not one of the accepted values.');
            gl.drawElements(gl.POINTS, 1, -1, vertices);
            this.expectError(gl.INVALID_ENUM);
            gl.drawElements(gl.POINTS, 1, gl.FLOAT, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count is negative.');
            gl.drawElements(gl.POINTS, -1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawElements(gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            if (!sglrGLContext.isExtensionSupported(gl, 'EXT_geometry_shader')) { // gl.EXT_geometry_shader removes error
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is active and not paused.');
                /** @type{Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram (program.getProgram());
                gl.transformFeedbackVaryings (program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram (program.getProgram());
                gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer (gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData (gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase (gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback (gl.POINTS);
                this.expectError (gl.NO_ERROR);

                gl.drawElements (gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.INVALID_OPERATION);

                gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

                gl.pauseTransformFeedback();
                gl.drawElements (gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.NO_ERROR);

                gl.endTransformFeedback ();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(bufElements);
                this.expectError (gl.NO_ERROR);

            }

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_elements_invalid_program', 'Invalid gl.drawElements() usage', gl, function() {
            gl.useProgram(null);
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.useProgram(null) was set.');
            gl.drawElements(gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteBuffer(bufElements);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_elements_incomplete_primitive', 'Invalid gl.drawElements() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{WebGLBuffer} */ var buf;
            /** @type{WebGLTransformFeedback} */ var tfID;
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawElements(-1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not one of the accepted values.');
            gl.drawElements(gl.TRIANGLES, 1, -1, vertices);
            this.expectError(gl.INVALID_ENUM);
            gl.drawElements(gl.TRIANGLES, 1, gl.FLOAT, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count is negative.');
            gl.drawElements(gl.TRIANGLES, -1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawElements(gl.TRIANGLES, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            if (!sglrGLContext.isExtensionSupported(gl, 'EXT_geometry_shader')) {// gl.EXT_geometry_shader removes error
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is active and not paused.');
                /** @type{Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram (program.getProgram());
                gl.transformFeedbackVaryings (program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram (program.getProgram());
                gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer (gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData (gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase (gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback (gl.TRIANGLES);
                this.expectError (gl.NO_ERROR);

                gl.drawElements (gl.TRIANGLES, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.INVALID_OPERATION);

                gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

                gl.pauseTransformFeedback();
                gl.drawElements (gl.TRIANGLES, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.NO_ERROR);

                gl.endTransformFeedback ();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(bufElements);
                this.expectError (gl.NO_ERROR);

            }

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_arrays_instanced', 'Invalid gl.drawArraysInstanced() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, bufElements);

            gl.vertexAttribDivisor(0, 1);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawArraysInstanced(-1, 0, 1, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count or primcount are negative.');
            gl.drawArraysInstanced(gl.POINTS, 0, -1, 1);
            this.expectError(gl.INVALID_VALUE);
            gl.drawArraysInstanced(gl.POINTS, 0, 1, -1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawArraysInstanced(gl.POINTS, 0, 1, 1);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            gl.deleteBuffer(bufElements);

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_arrays_instanced_invalid_program', 'Invalid gl.drawArraysInstanced() usage', gl, function() {
            gl.useProgram(null);

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, bufElements);

            gl.vertexAttribDivisor(0, 1);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.useProgram(null) is set.');
            gl.drawArraysInstanced(gl.POINTS, 0, 1, 1);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteBuffer(bufElements);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_arrays_instanced_incomplete_primitive', 'Invalid gl.drawArraysInstanced() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, bufElements);

            gl.vertexAttribDivisor(0, 1);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawArraysInstanced(-1, 0, 1, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count or primcount are negative.');
            gl.drawArraysInstanced(gl.TRIANGLES, 0, -1, 1);
            this.expectError(gl.INVALID_VALUE);
            gl.drawArraysInstanced(gl.TRIANGLES, 0, 1, -1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawArraysInstanced(gl.TRIANGLES, 0, 1, 1);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            gl.deleteBuffer(bufElements);

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_elements_instanced', 'Invalid gl.drawElementsInstanced() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{WebGLBuffer} */ var buf;
            /** @type{WebGLTransformFeedback} */ var tfID;
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            gl.vertexAttribDivisor(0, 1);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawElementsInstanced(-1, 1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not one of the accepted values.');
            gl.drawElementsInstanced(gl.POINTS, 1, -1, vertices, 1);
            this.expectError(gl.INVALID_ENUM);
            gl.drawElementsInstanced(gl.POINTS, 1, gl.FLOAT, vertices, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count or primcount are negative.');
            gl.drawElementsInstanced(gl.POINTS, -1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_VALUE);
            gl.drawElementsInstanced(gl.POINTS, 11, gl.UNSIGNED_BYTE, vertices, -1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawElementsInstanced(gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            if (!sglrGLContext.isExtensionSupported(gl, 'EXT_geometry_shader')) {// gl.EXT_geometry_shader removes error
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is active and not paused.');
                /** @type{Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram (program.getProgram());
                gl.transformFeedbackVaryings (program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram (program.getProgram());
                gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer (gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData (gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase (gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback (gl.POINTS);
                this.expectError (gl.NO_ERROR);

                gl.drawElementsInstanced (gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices, 1);
                this.expectError (gl.INVALID_OPERATION);

                gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

                gl.pauseTransformFeedback();
                gl.drawElementsInstanced (gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices, 1);
                this.expectError (gl.NO_ERROR);

                gl.endTransformFeedback ();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(bufElements);
                this.expectError (gl.NO_ERROR);

            }

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_elements_instanced_invalid_program', 'Invalid gl.drawElementsInstanced() usage', gl, function() {
            gl.useProgram(null);
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements;
            bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            gl.vertexAttribDivisor(0, 1);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.useProgram(null) is set.');
            gl.drawElementsInstanced(gl.POINTS, 1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteBuffer(bufElements);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_elements_instanced_incomplete_primitive', 'Invalid gl.drawElementsInstanced() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{WebGLBuffer} */ var buf;
            /** @type{WebGLTransformFeedback} */ var tfID;
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements;
            bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            gl.vertexAttribDivisor(0, 1);
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawElementsInstanced(-1, 1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not one of the accepted values.');
            gl.drawElementsInstanced(gl.TRIANGLES, 1, -1, vertices, 1);
            this.expectError(gl.INVALID_ENUM);
            gl.drawElementsInstanced(gl.TRIANGLES, 1, gl.FLOAT, vertices, 1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count or primcount are negative.');
            gl.drawElementsInstanced(gl.TRIANGLES, -1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_VALUE);
            gl.drawElementsInstanced(gl.TRIANGLES, 11, gl.UNSIGNED_BYTE, vertices, -1);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawElementsInstanced(gl.TRIANGLES, 1, gl.UNSIGNED_BYTE, vertices, 1);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            if (!sglrGLContext.isExtensionSupported(gl, 'EXT_geometry_shader')) {// gl.EXT_geometry_shader removes error
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is active and not paused.');
                /** @type{Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram (program.getProgram());
                gl.transformFeedbackVaryings (program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram (program.getProgram());
                gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer (gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData (gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase (gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback (gl.TRIANGLES);
                this.expectError (gl.NO_ERROR);

                gl.drawElementsInstanced (gl.TRIANGLES, 1, gl.UNSIGNED_BYTE, vertices, 1);
                this.expectError (gl.INVALID_OPERATION);

                gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

                gl.pauseTransformFeedback();
                gl.drawElementsInstanced (gl.TRIANGLES, 1, gl.UNSIGNED_BYTE, vertices, 1);
                this.expectError (gl.NO_ERROR);

                gl.endTransformFeedback ();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(bufElements);
                this.expectError (gl.NO_ERROR);

            }

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_range_elements', 'Invalid gl.drawRangeElements() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{WebGLBuffer} */ var buf;
            /** @type{WebGLTransformFeedback} */ var tfID;
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements;
            bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawRangeElements(-1, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not one of the accepted values.');
            gl.drawRangeElements(gl.POINTS, 0, 1, 1, -1, vertices);
            this.expectError(gl.INVALID_ENUM);
            gl.drawRangeElements(gl.POINTS, 0, 1, 1, gl.FLOAT, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count is negative.');
            gl.drawRangeElements(gl.POINTS, 0, 1, -1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if end < start.');
            gl.drawRangeElements(gl.POINTS, 1, 0, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawRangeElements(gl.POINTS, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            if (!sglrGLContext.isExtensionSupported(gl, 'EXT_geometry_shader')) {// gl.EXT_geometry_shader removes error
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is active and not paused.');
                /** @type{Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram (program.getProgram());
                gl.transformFeedbackVaryings (program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram (program.getProgram());
                gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer (gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData (gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase (gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback (gl.POINTS);
                this.expectError (gl.NO_ERROR);

                gl.drawRangeElements (gl.POINTS, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.INVALID_OPERATION);

                gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

                gl.pauseTransformFeedback();
                gl.drawRangeElements (gl.POINTS, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.NO_ERROR);

                gl.endTransformFeedback ();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(bufElements);
                this.expectError (gl.NO_ERROR);

            }

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_range_elements_invalid_program', 'Invalid gl.drawRangeElements() usage', gl, function() {
            gl.useProgram(null);
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements;
            bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);
            gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if gl.useProgram(null) is set.');
            gl.drawRangeElements(gl.POINTS, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteBuffer(bufElements);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('draw_range_elements_incomplete_primitive', 'Invalid gl.drawRangeElements() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
            gl.useProgram(program.getProgram());
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{WebGLBuffer} */ var buf;
            /** @type{WebGLTransformFeedback} */ var tfID;
            /** @type{number} */ var vertices = 0;

            /** @type{WebGLBuffer} */ var bufElements;
            bufElements = gl.createBuffer();
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufElements);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if mode is not an accepted value.');
            gl.drawRangeElements(-1, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if type is not one of the accepted values.');
            gl.drawRangeElements(gl.TRIANGLES, 0, 1, 1, -1, vertices);
            this.expectError(gl.INVALID_ENUM);
            gl.drawRangeElements(gl.TRIANGLES, 0, 1, 1, gl.FLOAT, vertices);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if count is negative.');
            gl.drawRangeElements(gl.TRIANGLES, 0, 1, -1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if end < start.');
            gl.drawRangeElements(gl.TRIANGLES, 1, 0, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_FRAMEBUFFER_OPERATION is generated if the currently bound framebuffer is not framebuffer complete.');
            fbo = gl.createFramebuffer();
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.checkFramebufferStatus(gl.FRAMEBUFFER);
            gl.drawRangeElements(gl.TRIANGLES, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
            this.expectError(gl.INVALID_FRAMEBUFFER_OPERATION);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);

            if (!sglrGLContext.isExtensionSupported(gl, 'EXT_geometry_shader')) {// gl.EXT_geometry_shader removes error
                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is active and not paused.');
                /** @type{Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram (program.getProgram());
                gl.transformFeedbackVaryings (program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram (program.getProgram());
                gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer (gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData (gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase (gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback (gl.TRIANGLES);
                this.expectError (gl.NO_ERROR);

                gl.drawRangeElements (gl.TRIANGLES, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.INVALID_OPERATION);

                gl.bufferData (gl.ELEMENT_ARRAY_BUFFER, 32, gl.STATIC_DRAW);

                gl.pauseTransformFeedback();
                gl.drawRangeElements (gl.TRIANGLES, 0, 1, 1, gl.UNSIGNED_BYTE, vertices);
                this.expectError (gl.NO_ERROR);

                gl.endTransformFeedback ();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(bufElements);
                this.expectError (gl.NO_ERROR);

            }

            gl.useProgram(null);
        }));
    };

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeVertexArrayApiTests.run = function(gl) {
        var testName = 'vertex_array';
        var testDescription = 'Negative Vertex Array API Cases';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fNegativeVertexArrayApiTests.init(gl);
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };

});
