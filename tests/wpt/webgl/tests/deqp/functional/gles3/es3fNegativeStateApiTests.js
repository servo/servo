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
 * \brief Negative GL State API tests.
 *//*--------------------------------------------------------------------*/
'use strict';
goog.provide('functional.gles3.es3fNegativeStateApiTests');

goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {

    var es3fNegativeStateApiTests = functional.gles3.es3fNegativeStateApiTests;
    var es3fApiCase = functional.gles3.es3fApiCase;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderProgram = framework.opengl.gluShaderProgram;

    /**
    * @type {string}
    * @const
    */
    var uniformTestVertSource = '#version 300 es\n' +
    'uniform mediump vec4 vUnif_vec4;\n' +
    'in mediump vec4 attr;\n' +
    'layout(std140) uniform Block { mediump vec4 blockVar; };\n' +
    'void main (void)\n' +
    '{\n' +
    ' gl_Position = vUnif_vec4 + blockVar + attr;\n' +
    '}\n';

    /**
    * @type {string}
    * @const
    */
    var uniformTestFragSource = '#version 300 es\n' +
    'uniform mediump ivec4 fUnif_ivec4;\n' +
    'uniform mediump uvec4 fUnif_uvec4;\n' +
    'layout(location = 0) out mediump vec4 fragColor;\n' +
    'void main (void)\n' +
    '{\n' +
    ' fragColor = vec4(vec4(fUnif_ivec4) + vec4(fUnif_uvec4));\n' +
    '}\n';

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeStateApiTests.init = function(gl) {

        var testGroup = tcuTestCase.runner.testCases;

        // Enabling & disabling states

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('enable', 'Invalid gl.enable() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if cap is not one of the allowed values.');
            gl.enable(-1);
            this.expectError(gl.INVALID_ENUM);

        }));
        testGroup.addChild(new es3fApiCase.ApiCaseCallback('disable', 'Invalid gl.disable() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if cap is not one of the allowed values.');
            gl.disable(-1);
            this.expectError(gl.INVALID_ENUM);

        }));

        // Simple state queries

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_parameter', 'Invalid gl.getParameter() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not one of the allowed values.');
            /** @type{boolean} */ var params = false;
            //glGetBooleanv(-1, params);
            params = /** @type{boolean} */ (gl.getParameter(-1));
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_indexed_parameter', 'Invalid gl.getIndexedParameter() usage', gl, function() {
            /** @type{number} */ var data = -1;
            /** @type{number} */ var maxUniformBufferBindings;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if name is not an accepted value.');
            data = /** @type{number} */ (gl.getIndexedParameter(-1, 0));
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is outside of the valid range for the indexed state target.');
            maxUniformBufferBindings = /** @type{number} */ (gl.getParameter(gl.MAX_UNIFORM_BUFFER_BINDINGS));
            this.expectError(gl.NO_ERROR);
            data = /** @type{number} */ (gl.getIndexedParameter(gl.UNIFORM_BUFFER_BINDING, maxUniformBufferBindings));
            this.expectError(gl.INVALID_VALUE);

        }));

        // Enumerated state queries: Shaders

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_attached_shaders', 'Invalid gl.getAttachedShaders() usage', gl, function() {
            /** @type{WebGLShader} */ var shaderObject = gl.createShader(gl.VERTEX_SHADER);
            /** @type{WebGLProgram} */ var program = gl.createProgram();

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getAttachedShaders(null);
            });

            gl.deleteShader(shaderObject);
            gl.deleteProgram(program);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_shader_parameter', 'Invalid gl.getShaderParameter() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{WebGLProgram} */ var program = gl.createProgram();
            /** @type{number} */ var param = -1;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
            param = /** @type{number} */ (gl.getShaderParameter(shader, -1));
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('An exception is thrown if shader is null.');
            this.expectThrowNoError(function() {
                gl.getShaderParameter(null, gl.SHADER_TYPE);
            });

            gl.deleteShader(shader);
            gl.deleteProgram(program);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_shader_info_log', 'Invalid gl.getShaderInfoLog() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{WebGLProgram} */ var program = gl.createProgram();

            bufferedLogToConsole('An exception is thrown if shader is null.');
            this.expectThrowNoError(function() {
                gl.getShaderInfoLog(null);
            });

            gl.deleteShader(shader);
            gl.deleteProgram(program);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_shader_precision_format', 'Invalid gl.getShaderPrecisionFormat() usage', gl, function() {
            /** @type{WebGLShaderPrecisionFormat } */ var precision;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if shaderType or precisionType is not an accepted value.');
            precision = gl.getShaderPrecisionFormat (-1, gl.MEDIUM_FLOAT);
            this.expectError(gl.INVALID_ENUM);
            precision = gl.getShaderPrecisionFormat (gl.VERTEX_SHADER, -1);
            this.expectError(gl.INVALID_ENUM);
            precision = gl.getShaderPrecisionFormat (-1, -1);
            this.expectError(gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_shader_source', 'Invalid gl.getShaderSource() usage', gl, function() {
            /** @type{WebGLProgram} */ var program = gl.createProgram();
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);

            bufferedLogToConsole('An exception is thrown if shader is null.');
            this.expectThrowNoError(function() {
                gl.getShaderSource(null);
            });

            gl.deleteProgram(program);
            gl.deleteShader(shader);
        }));

        // Enumerated state queries: Programs

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_program_parameter', 'Invalid gl.getProgramParameter() usage', gl, function() {
            /** @type{WebGLProgram} */ var program = gl.createProgram();
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{boolean} */ var params;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
            params = /** @type{boolean} */ (gl.getProgramParameter(program, -1));
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getProgramParameter(null, gl.LINK_STATUS);
            });

            gl.deleteProgram(program);
            gl.deleteShader(shader);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_program_info_log', 'Invalid gl.getProgramInfoLog() usage', gl, function() {
            /** @type{WebGLProgram} */ var program = gl.createProgram();
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getProgramInfoLog (null);
            });

            gl.deleteProgram(program);
            gl.deleteShader(shader);
        }));

        // Enumerated state queries: Shader variables

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_tex_parameter', 'Invalid gl.getTexParameter() usage', gl, function() {
            /** @type{WebGLTexture} */ var texture = gl.createTexture();
            gl.bindTexture(gl.TEXTURE_2D, texture);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or pname is not an accepted value.');
            gl.getTexParameter (-1, gl.TEXTURE_MAG_FILTER);
            this.expectError(gl.INVALID_ENUM);
            gl.getTexParameter (gl.TEXTURE_2D, -1);
            this.expectError(gl.INVALID_ENUM);
            gl.getTexParameter (-1, -1);
            this.expectError(gl.INVALID_ENUM);

            gl.deleteTexture(texture);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_uniform', 'Invalid gl.getUniform() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            gl.useProgram(program.getProgram());

            /** @type{WebGLUniformLocation} */ var unif = gl.getUniformLocation(program.getProgram(), 'vUnif_vec4'); // vec4
            assertMsgOptions(unif != null, 'Failed to retrieve uniform location', false, true);

            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{WebGLProgram} */ var programEmpty = gl.createProgram();
            /** @type{*} */ var params;

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getUniform (null, unif);
            });

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if program has not been successfully linked.');
            params = gl.getUniform (programEmpty, unif);
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('An exception is thrown if location is null.');
            this.expectThrowNoError(function() {
                gl.getUniform (program.getProgram(), null);
            });

            gl.deleteShader(shader);
            gl.deleteProgram(programEmpty);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_active_uniform', 'Invalid gl.getActiveUniform() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            /** @type{number} */ var numActiveUniforms = -1;

            numActiveUniforms = /** @type{number} */ (gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORMS));
            bufferedLogToConsole('// gl.ACTIVE_UNIFORMS = ' + numActiveUniforms + ' (expected 4).');

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getActiveUniform(null, 0);
            });

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to the number of active uniform variables in program.');
            gl.useProgram(program.getProgram());
            gl.getActiveUniform(program.getProgram(), numActiveUniforms);
            this.expectError(gl.INVALID_VALUE);

            gl.useProgram(null);
            gl.deleteShader(shader);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_active_uniforms', 'Invalid gl.getActiveUniforms() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            /** @type{Array<number>} */ var dummyUniformIndex = [1];
            /** @type{Array<number>} */ var dummyParamDst;
            /** @type{number} */ var numActiveUniforms = -1;

            gl.useProgram(program.getProgram());

            numActiveUniforms = /** @type{number} */ (gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORMS));
            bufferedLogToConsole('// gl.ACTIVE_UNIFORMS = ' + numActiveUniforms + ' (expected 4).');

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getActiveUniforms(null, dummyUniformIndex, gl.UNIFORM_TYPE);
            });

            bufferedLogToConsole('gl.INVALID_VALUE is generated if any value in uniformIndices is greater than or equal to the value of gl.ACTIVE_UNIFORMS for program.');
            /** @type{Array<number>} */ var invalidUniformIndices;
            /** @type{Array<number>} */ var dummyParamsDst;
            for (var excess = 0; excess <= 2; excess++) {
                invalidUniformIndices = [1, numActiveUniforms - 1 + excess, 1];
                dummyParamsDst = gl.getActiveUniforms(program.getProgram(), invalidUniformIndices, gl.UNIFORM_TYPE);
                this.expectError(excess == 0 ? gl.NO_ERROR : gl.INVALID_VALUE);
            }

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted token.');
            dummyParamDst = gl.getActiveUniforms(program.getProgram(), dummyUniformIndex, -1);
            this.expectError(gl.INVALID_ENUM);

            gl.useProgram(null);
            gl.deleteShader(shader);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_active_uniform_block_parameter', 'Invalid gl.getActiveUniformBlockParameter() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            /** @type{*} */ var params;
            /** @type{number} */ var numActiveBlocks = -1;

            numActiveBlocks = /** @type{number} */ (gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORM_BLOCKS));
            bufferedLogToConsole('// gl.ACTIVE_UNIFORM_BLOCKS = ' + numActiveBlocks + ' (expected 1).');
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if uniformBlockIndex is greater than or equal to the value of gl.ACTIVE_UNIFORM_BLOCKS or is not the index of an active uniform block in program.');
            gl.useProgram(program.getProgram());
            this.expectError(gl.NO_ERROR);
            params = gl.getActiveUniformBlockParameter(program.getProgram(), numActiveBlocks, gl.UNIFORM_BLOCK_BINDING);
            this.expectError(gl.INVALID_VALUE);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not one of the accepted tokens.');
            params = gl.getActiveUniformBlockParameter(program.getProgram(), 0, -1);
            this.expectError(gl.INVALID_ENUM);

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_active_uniform_block_name', 'Invalid gl.getActiveUniformBlockName() usage', gl, function() {
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            /** @type{number} */ var length = -1;
            /** @type{number} */ var numActiveBlocks = -1;
            /** @type{string} */ var uniformBlockName;

            numActiveBlocks = /** @type{number} */ (gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORM_BLOCKS));
            bufferedLogToConsole('// gl.ACTIVE_UNIFORM_BLOCKS = ' + numActiveBlocks + ' (expected 1).');
            this.expectError(gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if uniformBlockIndex is greater than or equal to the value of gl.ACTIVE_UNIFORM_BLOCKS or is not the index of an active uniform block in program.');
            gl.useProgram(program.getProgram());
            this.expectError(gl.NO_ERROR);
            uniformBlockName = gl.getActiveUniformBlockName(program.getProgram(), numActiveBlocks);
            this.expectError(gl.INVALID_VALUE);

            gl.useProgram(null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_active_attrib', 'Invalid gl.getActiveAttrib() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            /** @type{number} */ var numActiveAttributes = -1;

            /** @type{WebGLActiveInfo} */ var activeInfo;
            /** @type{number} */ var size = -1;
            /** @type{number} */ var type = -1;
            /** @type{string} */ var name;

            numActiveAttributes = /** @type{number} */(gl.getProgramParameter(program.getProgram(), gl.ACTIVE_ATTRIBUTES));
            bufferedLogToConsole('// gl.ACTIVE_ATTRIBUTES = ' + numActiveAttributes + ' (expected 1).');

            gl.useProgram(program.getProgram());

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getActiveAttrib(null, 0);
            });

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.ACTIVE_ATTRIBUTES.');
            activeInfo = gl.getActiveAttrib(program.getProgram(), numActiveAttributes);
            this.expectError(gl.INVALID_VALUE);

            gl.useProgram(null);
            gl.deleteShader(shader);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_uniform_indices', 'Invalid gl.getUniformIndices() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl,gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));
            gl.useProgram(program.getProgram());
            /** @type{number} */ var numActiveBlocks = -1;
            /** @type{Array<string>} */ var uniformName = ['Block.blockVar'];
            /** @type{Array<number>} */ var uniformIndices = [-1];

            numActiveBlocks = /** @type{number} */(gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORM_BLOCKS));
            bufferedLogToConsole('// gl.ACTIVE_UNIFORM_BLOCKS = ' + numActiveBlocks);
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('An exception is thrown if program is null.');
            this.expectThrowNoError(function() {
                gl.getUniformIndices(null, uniformName);
            });

            gl.useProgram(null);
            gl.deleteShader(shader);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_vertex_attrib', 'Invalid gl.getVertexAttrib() usage', gl, function() {
            /** @type{*} */ var params;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
            params = gl.getVertexAttrib(0, -1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            /** @type{number} */ var maxVertexAttribs;
            maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            params = gl.getVertexAttrib(maxVertexAttribs, gl.VERTEX_ATTRIB_ARRAY_ENABLED);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_vertex_attrib_offset', 'Invalid gl.getVertexAttribOffset() usage', gl, function() {
            /** @type{number} */ var ptr;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
            ptr = gl.getVertexAttribOffset(0, -1);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
            /** @type{number} */ var maxVertexAttribs;
            maxVertexAttribs = /** @type{number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
            ptr = gl.getVertexAttribOffset(maxVertexAttribs, gl.VERTEX_ATTRIB_ARRAY_POINTER);
            this.expectError(gl.INVALID_VALUE);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_frag_data_location', 'Invalid gl.getFragDataLocation() usage', gl, function() {
            /** @type{WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
            /** @type{WebGLProgram} */ var program = gl.createProgram();

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if program has not been linked.');
            gl.getFragDataLocation(program, 'gl_FragColor');
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteProgram(program);
            gl.deleteShader(shader);
        }));

        // Enumerated state queries: Buffers

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_buffer_parameter', 'Invalid gl.getBufferParameter() usage', gl, function() {
            /** @type{number} */ var params = -1;
            /** @type{WebGLBuffer} */ var buf;
            buf = gl.createBuffer();
            gl.bindBuffer(gl.ARRAY_BUFFER, buf);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or value is not an accepted value.');
            params = /** @type{number} */ (gl.getBufferParameter(-1, gl.BUFFER_SIZE));
            this.expectError(gl.INVALID_ENUM);
            params = /** @type{number} */ (gl.getBufferParameter(gl.ARRAY_BUFFER, -1));
            this.expectError(gl.INVALID_ENUM);
            params = /** @type{number} */ (gl.getBufferParameter(-1, -1));
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the reserved buffer object name 0 is bound to target.');
            gl.bindBuffer(gl.ARRAY_BUFFER, null);
            params = /** @type{number} */ (gl.getBufferParameter(gl.ARRAY_BUFFER, gl.BUFFER_SIZE));
            this.expectError(gl.INVALID_OPERATION);

            gl.deleteBuffer(buf);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_framebuffer_attachment_parameter', 'Invalid gl.getFramebufferAttachmentParameter() usage', gl, function() {
            /** @type{*} */ var params;
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{Array<WebGLRenderbuffer>} */ var rbo = [];

            fbo = gl.createFramebuffer();
            rbo[0] = gl.createRenderbuffer();
            rbo[1] = gl.createRenderbuffer();

            gl.bindFramebuffer (gl.FRAMEBUFFER, fbo);
            gl.bindRenderbuffer (gl.RENDERBUFFER, rbo[0]);
            gl.renderbufferStorage (gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, 16, 16);
            gl.framebufferRenderbuffer (gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, rbo[0]);
            gl.bindRenderbuffer (gl.RENDERBUFFER, rbo[1]);
            gl.renderbufferStorage (gl.RENDERBUFFER, gl.STENCIL_INDEX8, 16, 16);
            gl.framebufferRenderbuffer (gl.FRAMEBUFFER, gl.STENCIL_ATTACHMENT, gl.RENDERBUFFER, rbo[1]);
            gl.checkFramebufferStatus (gl.FRAMEBUFFER);
            this.expectError (gl.NO_ERROR);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not one of the accepted tokens.');
            gl.getFramebufferAttachmentParameter(-1, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE); // TYPE is gl.RENDERBUFFER
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not valid for the value of gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE.');
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL); // TYPE is gl.RENDERBUFFER
            this.expectError(gl.INVALID_ENUM);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME); // TYPE is gl.FRAMEBUFFER_DEFAULT
            this.expectError(gl.INVALID_ENUM);
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if attachment is gl.DEPTH_STENCIL_ATTACHMENT and different objects are bound to the depth and stencil attachment points of target.');
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.DEPTH_STENCIL_ATTACHMENT, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME);
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if the value of gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is gl.NONE and pname is not gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME.');
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME); // TYPE is gl.NONE
            this.expectError(gl.NO_ERROR);
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE); // TYPE is gl.NONE
            this.expectError(gl.INVALID_OPERATION);

            bufferedLogToConsole('gl.INVALID_OPERATION or gl.INVALID_ENUM is generated if attachment is not one of the accepted values for the current binding of target.');
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.BACK, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME); // A FBO is bound so gl.BACK is invalid
            this.expectError([gl.INVALID_OPERATION, gl.INVALID_ENUM]);
            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
            gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME); // Default framebuffer is bound so gl.COLOR_ATTACHMENT0 is invalid
            this.expectError([gl.INVALID_OPERATION, gl.INVALID_ENUM]);

            gl.deleteFramebuffer(fbo);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_renderbuffer_parameter', 'Invalid gl.getRenderbufferParameter() usage', gl, function() {
            /** @type{number} */ var params = -1;
            /** @type{WebGLRenderbuffer} */ var rbo;
            rbo = gl.createRenderbuffer();
            gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.RENDERBUFFER.');
            gl.getRenderbufferParameter(-1, gl.RENDERBUFFER_WIDTH);
            this.expectError(gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not one of the accepted tokens.');
            gl.getRenderbufferParameter(gl.RENDERBUFFER, -1);
            this.expectError(gl.INVALID_ENUM);

            gl.deleteRenderbuffer(rbo);
            gl.bindRenderbuffer(gl.RENDERBUFFER, null);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_internalformat_parameter', 'Invalid gl.getInternalformatParameter() usage', gl, function() {
            /** @type{WebGLRenderbuffer} */ var rbo = gl.createRenderbuffer();
            /** @type{WebGLFramebuffer} */ var fbo = gl.createFramebuffer();
            /** @type{WebGLTexture} */ var tex = gl.createTexture();
            gl.bindRenderbuffer(gl.RENDERBUFFER, rbo);
            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.bindTexture(gl.TEXTURE_2D, tex);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not gl.SAMPLES or gl.NUM_SAMPLE_COUNTS.');
            gl.getInternalformatParameter (gl.RENDERBUFFER, gl.RGBA8, -1);
            this.expectError (gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if internalformat is not color-, depth-, or stencil-renderable.');
            gl.getInternalformatParameter (gl.RENDERBUFFER, gl.RG8_SNORM, gl.NUM_SAMPLE_COUNTS);
            this.expectError (gl.INVALID_ENUM);
            gl.getInternalformatParameter (gl.RENDERBUFFER, gl.COMPRESSED_RGB8_ETC2, gl.NUM_SAMPLE_COUNTS);
            this.expectError (gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.RENDERBUFFER.');
            gl.getInternalformatParameter (-1, gl.RGBA8, gl.NUM_SAMPLE_COUNTS);
            this.expectError (gl.INVALID_ENUM);
            gl.getInternalformatParameter (gl.FRAMEBUFFER, gl.RGBA8, gl.NUM_SAMPLE_COUNTS);
            this.expectError (gl.INVALID_ENUM);
            gl.getInternalformatParameter (gl.TEXTURE_2D, gl.RGBA8, gl.NUM_SAMPLE_COUNTS);
            this.expectError (gl.INVALID_ENUM);

            gl.deleteRenderbuffer(rbo);
            gl.deleteFramebuffer(fbo);
            gl.deleteTexture(tex);

        }));

        // Query object queries

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_query', 'Invalid gl.getQuery() usage', gl, function() {
            /** @type{number} */ var params = -1;

            bufferedLogToConsole('gl.INVALID_ENUM is generated if target or pname is not an accepted value.');
            gl.getQuery (gl.ANY_SAMPLES_PASSED, -1);
            this.expectError (gl.INVALID_ENUM);
            gl.getQuery (-1, gl.CURRENT_QUERY);
            this.expectError (gl.INVALID_ENUM);
            gl.getQuery (-1, -1);
            this.expectError (gl.INVALID_ENUM);

        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_query_parameter', 'Invalid gl.getQueryParameter() usage', gl, function() {

            /** @type{WebGLQuery} */ var id;
            id = gl.createQuery();

            bufferedLogToConsole('An exception is thrown if the query object is null.');
            this.expectThrowNoError(function() {
                gl.getQueryParameter (null, gl.QUERY_RESULT_AVAILABLE);
            });

            bufferedLogToConsole('// Note: ' + id + ' is not a query object yet, since it hasn\'t been used by gl.beginQuery');
            gl.getQueryParameter (id, gl.QUERY_RESULT_AVAILABLE);
            this.expectError (gl.INVALID_OPERATION);

            gl.beginQuery (gl.ANY_SAMPLES_PASSED, id);
            gl.endQuery (gl.ANY_SAMPLES_PASSED);

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
            gl.getQueryParameter (id, -1);
            this.expectError (gl.INVALID_ENUM);

            bufferedLogToConsole('gl.INVALID_OPERATION is generated if id is the name of a currently active query object.');
            gl.beginQuery (gl.ANY_SAMPLES_PASSED, id);
            this.expectError (gl.NO_ERROR);
            gl.getQueryParameter (id, gl.QUERY_RESULT_AVAILABLE);
            this.expectError (gl.INVALID_OPERATION);
            gl.endQuery (gl.ANY_SAMPLES_PASSED);
            this.expectError (gl.NO_ERROR);

            gl.deleteQuery(id);
        }));

        // Sync object queries

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_sync_parameter', 'Invalid gl.getSyncParameter() usage', gl, function() {
            /** @type{WebGLSync} */ var sync;

            bufferedLogToConsole('An exception is thrown if sync is null.');
            this.expectThrowNoError(function() {
                gl.getSyncParameter (null, gl.OBJECT_TYPE);
            });

            bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not one of the accepted tokens.');
            sync = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
            this.expectError (gl.NO_ERROR);
            gl.getSyncParameter (sync, -1);
            this.expectError (gl.INVALID_ENUM);

        }));

        // Enumerated boolean state queries

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_enabled', 'Invalid gl.isEnabled() usage', gl, function() {
            bufferedLogToConsole('gl.INVALID_ENUM is generated if cap is not an accepted value.');
            gl.isEnabled(-1);
            this.expectError(gl.INVALID_ENUM);
            gl.isEnabled(gl.TRIANGLES);
            this.expectError(gl.INVALID_ENUM);

        }));

        // Named Object Usage

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_buffer', 'Invalid gl.isBuffer() usage', gl, function() {
            /** @type{WebGLBuffer} */ var buffer;
            /** @type{boolean} */ var isBuffer;

            bufferedLogToConsole('A name returned by glGenBuffers, but not yet associated with a buffer object by calling glBindBuffer, is not the name of a buffer object.');
            isBuffer = gl.isBuffer(buffer);
            assertMsgOptions(!isBuffer, 'Got invalid boolean value', false, true);

            buffer = gl.createBuffer();
            isBuffer = gl.isBuffer(buffer);
            assertMsgOptions(!isBuffer, 'Got invalid boolean value', false, true);

            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
            isBuffer = gl.isBuffer(buffer);
            assertMsgOptions(isBuffer, 'Got invalid boolean value', false, true);

            gl.bindBuffer(gl.ARRAY_BUFFER, null);
            gl.deleteBuffer(buffer);
            isBuffer = gl.isBuffer(buffer);
            assertMsgOptions(!isBuffer, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_framebuffer', 'Invalid gl.isFramebuffer() usage', gl, function() {
            /** @type{WebGLFramebuffer} */ var fbo;
            /** @type{boolean} */ var isFbo;

            bufferedLogToConsole('A name returned by glGenFramebuffers, but not yet bound through a call to gl.bindFramebuffer is not the name of a framebuffer object.');
            isFbo = gl.isFramebuffer(fbo);
            assertMsgOptions(!isFbo, 'Got invalid boolean value', false, true);

            fbo = gl.createFramebuffer();
            isFbo = gl.isFramebuffer(fbo);
            assertMsgOptions(!isFbo, 'Got invalid boolean value', false, true);

            gl.bindFramebuffer (gl.FRAMEBUFFER, fbo);
            isFbo = gl.isFramebuffer(fbo);
            assertMsgOptions(isFbo, 'Got invalid boolean value', false, true);

            gl.bindFramebuffer (gl.FRAMEBUFFER, null);
            gl.deleteFramebuffer(fbo);
            isFbo = gl.isFramebuffer(fbo);
            assertMsgOptions(!isFbo, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_program', 'Invalid gl.isProgram() usage', gl, function() {
            /** @type{WebGLProgram} */ var program;
            /** @type{boolean} */ var isProgram;

            bufferedLogToConsole('A name created with gl.createProgram, and not yet deleted with glDeleteProgram is a name of a program object.');
            isProgram = gl.isProgram(program);
            assertMsgOptions(!isProgram, 'Got invalid boolean value', false, true);

            program = gl.createProgram();
            isProgram = gl.isProgram(program);
            assertMsgOptions(isProgram, 'Got invalid boolean value', false, true);

            gl.deleteProgram(program);
            isProgram = gl.isProgram(program);
            assertMsgOptions(!isProgram, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_renderbuffer', 'Invalid gl.isRenderbuffer() usage', gl, function() {
            /** @type{WebGLRenderbuffer} */ var rbo;
            /** @type{boolean} */ var isRbo;

            bufferedLogToConsole('A name returned by glGenRenderbuffers, but not yet bound through a call to gl.bindRenderbuffer or gl.framebufferRenderbuffer is not the name of a renderbuffer object.');
            isRbo = gl.isRenderbuffer(rbo);
            assertMsgOptions(!isRbo, 'Got invalid boolean value', false, true);

            rbo = gl.createRenderbuffer();
            isRbo = gl.isRenderbuffer(rbo);
            assertMsgOptions(!isRbo, 'Got invalid boolean value', false, true);

            gl.bindRenderbuffer (gl.RENDERBUFFER, rbo);
            isRbo = gl.isRenderbuffer(rbo);
            assertMsgOptions(isRbo, 'Got invalid boolean value', false, true);

            gl.bindRenderbuffer (gl.RENDERBUFFER, null);
            gl.deleteRenderbuffer(rbo);
            isRbo = gl.isRenderbuffer(rbo);
            assertMsgOptions(!isRbo, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_shader', 'Invalid gl.isShader() usage', gl, function() {
            /** @type{WebGLShader} */ var shader;
            /** @type{boolean} */ var isShader;

            bufferedLogToConsole('A name created with glCreateShader, and not yet deleted with glDeleteShader is a name of a shader object.');
            isShader = gl.isProgram(shader);
            assertMsgOptions(!isShader, 'Got invalid boolean value', false, true);

            shader = gl.createShader(gl.VERTEX_SHADER);
            isShader = gl.isShader(shader);
            assertMsgOptions(isShader, 'Got invalid boolean value', false, true);

            gl.deleteShader (shader);
            isShader = gl.isShader(shader);
            assertMsgOptions(!isShader, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_texture', 'Invalid gl.isTexture() usage', gl, function() {
            /** @type{WebGLTexture} */ var texture;
            /** @type{boolean} */ var isTexture;

            bufferedLogToConsole('A name returned by glGenTextures, but not yet bound through a call to glBindTexture is not the name of a texture.');
            isTexture = gl.isTexture(texture);
            assertMsgOptions(!isTexture, 'Got invalid boolean value', false, true);

            texture = gl.createTexture();
            isTexture = gl.isTexture(texture);
            assertMsgOptions(!isTexture, 'Got invalid boolean value', false, true);

            gl.bindTexture (gl.TEXTURE_2D, texture);
            isTexture = gl.isTexture(texture);
            assertMsgOptions(isTexture, 'Got invalid boolean value', false, true);

            gl.bindTexture (gl.TEXTURE_2D, null);
            gl.deleteTexture(texture);
            isTexture = gl.isTexture(texture);
            assertMsgOptions(!isTexture, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_query', 'Invalid gl.isQuery() usage', gl, function() {
            /** @type{WebGLQuery} */ var query;
            /** @type{boolean} */ var isQuery;

            bufferedLogToConsole('A name returned by glGenQueries, but not yet associated with a query object by calling gl.beginQuery, is not the name of a query object.');
            isQuery = gl.isQuery(query);
            assertMsgOptions(!isQuery, 'Got invalid boolean value', false, true);

            query = gl.createQuery();
            isQuery = gl.isQuery(query);
            assertMsgOptions(!isQuery, 'Got invalid boolean value', false, true);

            gl.beginQuery (gl.ANY_SAMPLES_PASSED, query);
            isQuery = gl.isQuery(query);
            assertMsgOptions(isQuery, 'Got invalid boolean value', false, true);

            gl.endQuery (gl.ANY_SAMPLES_PASSED);
            gl.deleteQuery (query);
            isQuery = gl.isQuery(query);
            assertMsgOptions(!isQuery, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_sampler', 'Invalid gl.isSampler() usage', gl, function() {
            /** @type{WebGLSampler} */ var sampler;
            /** @type{boolean} */ var isSampler;

            bufferedLogToConsole('A name returned by glGenSamplers is the name of a sampler object.');
            isSampler = gl.isSampler(sampler);
            assertMsgOptions(!isSampler, 'Got invalid boolean value', false, true);

            sampler = gl.createSampler();
            isSampler = gl.isSampler(sampler);
            assertMsgOptions(isSampler, 'Got invalid boolean value', false, true);

            gl.bindSampler(0, sampler);
            isSampler = gl.isSampler(sampler);
            assertMsgOptions(isSampler, 'Got invalid boolean value', false, true);

            gl.deleteSampler(sampler);
            isSampler = gl.isSampler(sampler);
            assertMsgOptions(!isSampler, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_sync', 'Invalid gl.isSync() usage', gl, function() {
            /** @type{WebGLSync} */ var sync;
            /** @type{boolean} */ var isSync;

            bufferedLogToConsole('A name returned by gl.fenceSync is the name of a sync object.');
            isSync = gl.isSync(sync);
            assertMsgOptions(!isSync, 'Got invalid boolean value', false, true);

            sync = gl.fenceSync (gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
            isSync = gl.isSync(sync);
            assertMsgOptions(isSync, 'Got invalid boolean value', false, true);

            gl.deleteSync (sync);
            isSync = gl.isSync(sync);
            assertMsgOptions(!isSync, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_transform_feedback', 'Invalid gl.isTransformFeedback() usage', gl, function() {
            /** @type{WebGLTransformFeedback} */ var tf;
            /** @type{boolean} */ var isTF;

            bufferedLogToConsole('A name returned by glGenTransformFeedbacks, but not yet bound using glBindTransformFeedback, is not the name of a transform feedback object.');
            isTF = gl.isTransformFeedback(tf);
            assertMsgOptions(!isTF, 'Got invalid boolean value', false, true);

            tf = gl.createTransformFeedback();
            isTF = gl.isTransformFeedback(tf);
            assertMsgOptions(!isTF, 'Got invalid boolean value', false, true);

            gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, tf);
            isTF = gl.isTransformFeedback(tf);
            assertMsgOptions(isTF, 'Got invalid boolean value', false, true);

            gl.bindTransformFeedback (gl.TRANSFORM_FEEDBACK, null);
            gl.deleteTransformFeedback (tf);
            isTF = gl.isTransformFeedback(tf);
            assertMsgOptions(!isTF, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('is_vertex_array', 'Invalid gl.isVertexArray() usage', gl, function() {
            /** @type{WebGLVertexArrayObject} */ var vao;
            /** @type{boolean} */ var isVao;

            bufferedLogToConsole('A name returned by glGenVertexArrays, but not yet bound using glBindVertexArray, is not the name of a vertex array object.');
            isVao = gl.isVertexArray(vao);
            assertMsgOptions(!isVao, 'Got invalid boolean value', false, true);

            vao = gl.createVertexArray();
            isVao = gl.isVertexArray(vao);
            assertMsgOptions(!isVao, 'Got invalid boolean value', false, true);

            gl.bindVertexArray (vao);
            isVao = gl.isVertexArray(vao);
            assertMsgOptions(isVao, 'Got invalid boolean value', false, true);

            gl.bindVertexArray (null);
            gl.deleteVertexArray (vao);
            isVao = gl.isVertexArray(vao);
            assertMsgOptions(!isVao, 'Got invalid boolean value', false, true);

            this.expectError (gl.NO_ERROR);
        }));
    };

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeStateApiTests.run = function(gl) {
        var testName = 'state';
        var testDescription = 'Negative GL State API Cases';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fNegativeStateApiTests.init(gl);
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };

});
