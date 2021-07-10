/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
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
 */

'use strict';
goog.provide('functional.gles3.es3fNegativeShaderApiTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluShaderProgram');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {

    var es3fNegativeShaderApiTests = functional.gles3.es3fNegativeShaderApiTests;
    var es3fApiCase = functional.gles3.es3fApiCase;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderProgram = framework.opengl.gluShaderProgram;

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
    * @type {string}
    * @const
    */
    var uniformTestVertSource = '#version 300 es\n' +
    'uniform mediump vec4 vec4_v;\n' +
    'uniform mediump mat4 mat4_v;\n' +
    'void main (void)\n' +
    '{\n' +
    ' gl_Position = mat4_v * vec4_v;\n' +
    '}\n';

    /**
    * @type {string}
    * @const
    */
    var uniformTestFragSource = '#version 300 es\n' +
    'uniform mediump ivec4 ivec4_f;\n' +
    'uniform mediump uvec4 uvec4_f;\n' +
    'uniform sampler2D sampler_f;\n' +
    'layout(location = 0) out mediump vec4 fragColor;\n' +
    'void main (void)\n' +
    '{\n' +
    ' fragColor.xy = (vec4(uvec4_f) + vec4(ivec4_f)).xy;\n' +
    ' fragColor.zw = texture(sampler_f, vec2(0.0, 0.0)).zw;\n' +
    '}\n';

    /**
    * @type {string}
    * @const
    */
    var uniformBlockVertSource = '#version 300 es\n' +
    'layout(std140) uniform Block { lowp float var; };\n' +
    'void main (void)\n' +
    '{\n' +
    ' gl_Position = vec4(var);\n' +
    '}\n';

    /**
    * @param {WebGL2RenderingContext} gl
    */
    es3fNegativeShaderApiTests.init = function(gl) {
        var testGroup = tcuTestCase.runner.testCases;

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'create_shader', 'Invalid gl.createShader() usage', gl,
            function() {
                bufferedLogToConsole('INVALID_ENUM is generated if shaderType is not an accepted value.');
                gl.createShader(-1);
                this.expectError(gl.INVALID_ENUM);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('attach_shader', 'Invalid gl.attachShader() usage', gl,
            function() {
                /** @type {WebGLShader} */ var shader1 = gl.createShader(gl.VERTEX_SHADER);
                /** @type {WebGLShader} */ var shader2 = gl.createShader(gl.VERTEX_SHADER);
                /** @type {WebGLProgram} */ var program = gl.createProgram();

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if shader is already attached to program.');
                gl.attachShader(program, shader1);
                    this.expectError(gl.NO_ERROR);
                gl.attachShader(program, shader1);
                    this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if a shader of the same type as shader is already attached to program.');
                gl.attachShader(program, shader2);
                    this.expectError(gl.INVALID_OPERATION);

                gl.deleteProgram(program);
                gl.deleteShader(shader1);
                gl.deleteShader(shader2);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('detach_shader', 'Invalid gl.detachShader() usage', gl,
            function() {
                /** @type {WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
                /** @type {WebGLProgram} */ var program = gl.createProgram();

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if shader is not attached to program.');
                gl.detachShader(program, shader);
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteProgram(program);
                gl.deleteShader(shader);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('link_program', 'Invalid gl.linkProgram() usage', gl,
            function() {

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if program is the currently active program object and transform feedback mode is active.');

                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {WebGLTransformFeedback} */ var tfID;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                tfID = gl.createTransformFeedback();
                buf = gl.createBuffer();

                gl.useProgram(program.getProgram());
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback(gl.TRIANGLES);

                this.expectError(gl.NO_ERROR);

                gl.linkProgram(program.getProgram());
                this.expectError(gl.INVALID_OPERATION);

                gl.endTransformFeedback();
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(buf);
                this.expectError(gl.NO_ERROR);

            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('use_program', 'Invalid gl.useProgram() usage', gl,
            function() {

                /** @type {WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback mode is active and not paused.');
                /** @type {gluShaderProgram.ShaderProgram} */ var program1 = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {gluShaderProgram.ShaderProgram} */ var program2 = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {WebGLTransformFeedback} */ var tfID;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                tfID = gl.createTransformFeedback();
                buf = gl.createBuffer();

                gl.useProgram(program1.getProgram());
                gl.transformFeedbackVaryings(program1.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program1.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback(gl.TRIANGLES);
                this.expectError(gl.NO_ERROR);

                gl.useProgram(program2.getProgram());
                this.expectError(gl.INVALID_OPERATION);

                gl.pauseTransformFeedback();
                gl.useProgram(program2.getProgram());
                this.expectError(gl.NO_ERROR);

                gl.endTransformFeedback();
                gl.deleteTransformFeedback(tfID);
                gl.deleteBuffer(buf);
                this.expectError(gl.NO_ERROR);

                gl.useProgram(null);
                gl.deleteShader(shader);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('bind_sampler', 'Invalid gl.bindSampler() usage', gl,
            function() {
                /** @type {number} */ var maxTexImageUnits;
                /** @type {WebGLSampler} */ var sampler;
                /** @type {WebGLSampler} */ var buf;
                maxTexImageUnits = /** @type {number} */ (gl.getParameter(gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS));
                sampler = gl.createSampler();

                bufferedLogToConsole('gl.INVALID_VALUE is generated if unit is greater than or equal to the value of gl.MAX_COMBIED_TEXTURE_IMAGE_UNITS.');
                gl.bindSampler(maxTexImageUnits, sampler);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if sampler has been deleted by a call to glDeleteSamplers.');
                gl.deleteSampler(sampler);
                gl.bindSampler(1, sampler);
                this.expectError(gl.INVALID_OPERATION);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_sampler_parameteriv', 'Invalid gl.getSamplerParameter() usage', gl,
            function() {
                /** @type {number} */ var params;
                /** @type {WebGLSampler} */ var sampler;
                sampler = gl.createSampler();

                bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
                params = /** @type {number} */ (gl.getSamplerParameter(sampler, -1));
                this.expectError(gl.INVALID_ENUM);

                gl.deleteSampler(sampler);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_sampler_parameterfv', 'Invalid gl.getSamplerParameter() usage', gl,
            function() {
                /** @type {number} */ var params;
                /** @type {WebGLSampler} */ var sampler;
                sampler = gl.createSampler();

                bufferedLogToConsole('gl.INVALID_ENUM is generated if pname is not an accepted value.');
                params = /** @type {number} */ (gl.getSamplerParameter(sampler, -1));
                this.expectError(gl.INVALID_ENUM);

                gl.deleteSampler(sampler);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('sampler_parameteri', 'Invalid gl.samplerParameteri() usage', gl,
            function() {
                /** @type {WebGLSampler} */ var sampler;
                sampler = gl.createSampler();

                bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined constant value (based on the value of pname) and does not.');
                gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_S, -1);
                this.expectError(gl.INVALID_ENUM);

                gl.deleteSampler(sampler);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'sampler_parameteriv', 'Invalid gl.samplerParameteri() usage', gl,
            function() {
                /** @type {number} */ var params;
                /** @type {WebGLSampler} */ var sampler;
                sampler = gl.createSampler();

                bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined constant value (based on the value of pname) and does not.');
                params = -1;
                gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_S, params);
                this.expectError(gl.INVALID_ENUM);

                gl.deleteSampler(sampler);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'sampler_parameterf', 'Invalid glSamplerParameterf() usage', gl,
            function() {
                /** @type {WebGLSampler} */ var sampler;
                sampler = gl.createSampler();

                bufferedLogToConsole('gl.INVALID_ENUM is generated if params should have a defined constant value (based on the value of pname) and does not.');
                gl.samplerParameterf(sampler, gl.TEXTURE_WRAP_S, -1.0);
                this.expectError(gl.INVALID_ENUM);

                gl.deleteSampler(sampler);
            }
        ));

        // Shader data commands

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'get_attrib_location', 'Invalid gl.getAttribLocation() usage', gl,
            function() {
                /** @type {WebGLProgram} */ var programEmpty = gl.createProgram();
                /** @type {WebGLShader} */ var shader = gl.createShader(gl.VERTEX_SHADER);
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if program has not been successfully linked.');
                gl.bindAttribLocation(programEmpty, 0, 'test');
                gl.getAttribLocation(programEmpty, 'test');
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
                gl.deleteShader(shader);
                gl.deleteProgram(programEmpty);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'get_uniform_location', 'Invalid gl.getUniformLocation() usage', gl,
            function() {
                /** @type {WebGLProgram} */ var programEmpty = gl.createProgram();

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if program has not been successfully linked.');
                gl.getUniformLocation(programEmpty, 'test');
                this.expectError(gl.INVALID_OPERATION);
                gl.deleteProgram(programEmpty);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback(
            'bind_attrib_location', 'Invalid gl.bindAttribLocation() usage', gl,
            function() {
                /** @type {WebGLProgram} */ var program = gl.createProgram();
                var maxIndex = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));

                bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater than or equal to gl.MAX_VERTEX_ATTRIBS.');
                gl.bindAttribLocation(program, maxIndex, 'test');
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if name starts with the reserved prefix \'gl.\'.');
                gl.bindAttribLocation(program, maxIndex-1, 'gl_test');
                this.expectError(gl.INVALID_OPERATION);

                gl.deleteProgram(program);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniform_block_binding', 'Invalid gl.uniformBlockBinding() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformBlockVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());

                /** @type {number} */ var maxUniformBufferBindings;
                /** @type {number} */ var numActiveUniforms = -1;
                /** @type {number} */ var numActiveBlocks = -1;
                maxUniformBufferBindings = /** @type {number} */ (gl.getParameter(gl.MAX_UNIFORM_BUFFER_BINDINGS));
                numActiveUniforms = /** @type {number} */ (gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORMS));
                numActiveBlocks = /** @type {number} */ (gl.getProgramParameter(program.getProgram(), gl.ACTIVE_UNIFORM_BLOCKS));
                bufferedLogToConsole('// gl.MAX_UNIFORM_BUFFER_BINDINGS = ' + maxUniformBufferBindings);
                bufferedLogToConsole('// gl.ACTIVE_UNIFORMS = ' + numActiveUniforms);
                bufferedLogToConsole('// gl.ACTIVE_UNIFORM_BLOCKS = ' + numActiveBlocks);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if uniformBlockIndex is not an active uniform block index of program.');
                gl.uniformBlockBinding(program.getProgram(), -1, 0);
                this.expectError(gl.INVALID_VALUE);
                gl.uniformBlockBinding(program.getProgram(), 5, 0);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_VALUE is generated if uniformBlockBinding is greater than or equal to the value of gl.MAX_UNIFORM_BUFFER_BINDINGS.');
                gl.uniformBlockBinding(program.getProgram(), maxUniformBufferBindings, 0);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('An exception is thrown if program is null.');
                this.expectThrowNoError(function() {
                    gl.uniformBlockBinding(null, 0, 0);
                });
            }
        ));

        // glUniform*f

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformf_incompatible_type', 'Invalid glUniform{1234}f() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null || ivec4_f == null || uvec4_f == null || sampler_f == null) {
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);
                }

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniform1f(vec4_v, 0.0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2f(vec4_v, 0.0, 0.0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3f(vec4_v, 0.0, 0.0, 0.0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4f(vec4_v, 0.0, 0.0, 0.0, 0.0);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}f is used to load a uniform variable of type int, ivec2, ivec3, ivec4, unsigned int, uvec2, uvec3, uvec4.');
                gl.useProgram(program.getProgram());
                gl.uniform4f(ivec4_f, 0.0, 0.0, 0.0, 0.0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4f(uvec4_f, 0.0, 0.0, 0.0, 0.0);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if a sampler is loaded using a command other than glUniform1i and glUniform1iv.');
                gl.useProgram(program.getProgram());
                gl.uniform1f(sampler_f, 0.0);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformfv_incompatible_type', 'Invalid glUniform{1234}fv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null || ivec4_f == null || uvec4_f == null || sampler_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Float32Array} */ var data = new Float32Array(4);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniform1fv(vec4_v, new Float32Array(1));
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2fv(vec4_v, new Float32Array(2));
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3fv(vec4_v, new Float32Array(3));
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4fv(vec4_v, new Float32Array(4));
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}fv is used to load a uniform variable of type /** @type {number} */ var , ivec2, ivec3, ivec4, unsigned /** @type {number} */ var , uvec2, uvec3, uvec4.');
                gl.useProgram(program.getProgram());
                gl.uniform4fv(ivec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4fv(uvec4_f, data);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if a sampler is loaded using a command other than glUniform1i and glUniform1iv.');
                gl.useProgram(program.getProgram());
                gl.uniform1fv(sampler_f, new Float32Array(1));
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformfv_invalid_count', 'Invalid glUniform{1234}fv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Float32Array} */ var data = new Float32Array(12);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if count is greater than 1 and the indicated uniform variable is not an array variable.');
                gl.useProgram(program.getProgram());
                gl.uniform1fv(vec4_v, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2fv(vec4_v, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3fv(vec4_v, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4fv(vec4_v, data);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformi_incompatible_type', 'Invalid glUniform{1234}i() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null || ivec4_f == null || uvec4_f == null || sampler_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniform1i(ivec4_f, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2i(ivec4_f, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3i(ivec4_f, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4i(ivec4_f, 0, 0, 0, 0);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}i is used to load a uniform variable of type unsigned /** @type {number} */ var , uvec2, uvec3, uvec4, or an array of these.');
                gl.useProgram(program.getProgram());
                gl.uniform1i(uvec4_f, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2i(uvec4_f, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3i(uvec4_f, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4i(uvec4_f, 0, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}i is used to load a uniform variable of type /** @type {number} */ var , vec2, vec3, or vec4.');
                gl.useProgram(program.getProgram());
                gl.uniform1i(vec4_v, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2i(vec4_v, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3i(vec4_v, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4i(vec4_v, 0, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformiv_incompatible_type', 'Invalid glUniform{1234}iv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null || ivec4_f == null || uvec4_f == null || sampler_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Int32Array} */ var data1 = new Int32Array(1);
                /** @type {Int32Array} */ var data2 = new Int32Array(2);
                /** @type {Int32Array} */ var data3 = new Int32Array(3);
                /** @type {Int32Array} */ var data4 = new Int32Array(4);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniform1iv(ivec4_f, data1);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2iv(ivec4_f, data2);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3iv(ivec4_f, data3);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4iv(ivec4_f, data4);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}iv is used to load a uniform variable of type /** @type {number} */ var , vec2, vec3, or vec4.');
                gl.useProgram(program.getProgram());
                gl.uniform1iv(vec4_v, data1);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2iv(vec4_v, data2);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3iv(vec4_v, data3);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4iv(vec4_v, data4);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}iv is used to load a uniform variable of type unsigned /** @type {number} */ var , uvec2, uvec3 or uvec4.');
                gl.useProgram(program.getProgram());
                gl.uniform1iv(uvec4_f, data1);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2iv(uvec4_f, data2);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3iv(uvec4_f, data3);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4iv(uvec4_f, data4);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformiv_invalid_count', 'Invalid glUniform{1234}iv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                this.expectError(gl.NO_ERROR);

                if (ivec4_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Int32Array} */ var data = new Int32Array(12);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if count is greater than 1 and the indicated uniform variable is not an array variable.');
                gl.useProgram(program.getProgram());
                gl.uniform1iv(ivec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2iv(ivec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3iv(ivec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4iv(ivec4_f, data);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformui_incompatible_type', 'Invalid glUniform{1234}ui() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null || ivec4_f == null || uvec4_f == null || sampler_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniform1ui(uvec4_f, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2ui(uvec4_f, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3ui(uvec4_f, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4ui(uvec4_f, 0, 0, 0, 0);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}i is used to load a uniform variable of type /** @type {number} */ var , ivec2, ivec3, ivec4, or an array of these.');
                gl.useProgram(program.getProgram());
                gl.uniform1ui(ivec4_f, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2ui(ivec4_f, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3ui(ivec4_f, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4ui(ivec4_f, 0, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}i is used to load a uniform variable of type /** @type {number} */ var , vec2, vec3, or vec4.');
                gl.useProgram(program.getProgram());
                gl.uniform1ui(vec4_v, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2ui(vec4_v, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3ui(vec4_v, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4ui(vec4_v, 0, 0, 0, 0);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if a sampler is loaded using a command other than glUniform1i and glUniform1iv.');
                gl.useProgram(program.getProgram());
                gl.uniform1ui(sampler_f, 0);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformuiv_incompatible_type', 'Invalid glUniform{1234}uiv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var vec4_v = gl.getUniformLocation(program.getProgram(), 'vec4_v'); // vec4
                /** @type {WebGLUniformLocation} */ var ivec4_f = gl.getUniformLocation(program.getProgram(), 'ivec4_f'); // ivec4
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (vec4_v == null || ivec4_f == null || uvec4_f == null || sampler_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Uint32Array} */ var data1 = new Uint32Array(1);
                /** @type {Uint32Array} */ var data2 = new Uint32Array(2);
                /** @type {Uint32Array} */ var data3 = new Uint32Array(3);
                /** @type {Uint32Array} */ var data4 = new Uint32Array(4);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniform1uiv(uvec4_f, data1);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2uiv(uvec4_f, data2);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3uiv(uvec4_f, data3);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4uiv(uvec4_f, data4);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}uiv is used to load a uniform variable of type /** @type {number} */ var , vec2, vec3, or vec4.');
                gl.useProgram(program.getProgram());
                gl.uniform1uiv(vec4_v, data1);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2uiv(vec4_v, data2);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3uiv(vec4_v, data3);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4uiv(vec4_v, data4);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if glUniform{1234}uiv is used to load a uniform variable of type /** @type {number} */ var , ivec2, ivec3 or ivec4.');
                gl.useProgram(program.getProgram());
                gl.uniform1uiv(ivec4_f, data1);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2uiv(ivec4_f, data2);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3uiv(ivec4_f, data3);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4uiv(ivec4_f, data4);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if a sampler is loaded using a command other than glUniform1i and glUniform1iv.');
                gl.useProgram(program.getProgram());
                gl.uniform1uiv(sampler_f, data1);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniformuiv_invalid_count', 'Invalid glUniform{1234}uiv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var uvec4_f = gl.getUniformLocation(program.getProgram(), 'uvec4_f'); // uvec4
                this.expectError(gl.NO_ERROR);

                if (uvec4_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Uint32Array} */ var data = new Uint32Array(12);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if count is greater than 1 and the indicated uniform variable is not an array variable.');
                gl.useProgram(program.getProgram());
                gl.uniform1uiv(uvec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform2uiv(uvec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform3uiv(uvec4_f, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniform4uiv(uvec4_f, data);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniform_matrixfv_incompatible_type', 'Invalid glUniformMatrix{234}fv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var mat4_v = gl.getUniformLocation(program.getProgram(), 'mat4_v'); // mat4
                /** @type {WebGLUniformLocation} */ var sampler_f = gl.getUniformLocation(program.getProgram(), 'sampler_f'); // sampler2D
                this.expectError(gl.NO_ERROR);

                if (mat4_v == null || sampler_f == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Float32Array} */ var data4 = new Float32Array(4);
                /** @type {Float32Array} */ var data9 = new Float32Array(9);
                /** @type {Float32Array} */ var data16 = new Float32Array(16);
                /** @type {Float32Array} */ var data6 = new Float32Array(6);
                /** @type {Float32Array} */ var data8 = new Float32Array(8);
                /** @type {Float32Array} */ var data12 = new Float32Array(12);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the size of the uniform variable declared in the shader does not match the size indicated by the glUniform command.');
                gl.useProgram(program.getProgram());
                gl.uniformMatrix2fv(mat4_v, false, data4);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3fv(mat4_v, false, data9);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4fv(mat4_v, false, data16);
                this.expectError(gl.NO_ERROR);

                gl.uniformMatrix2x3fv(mat4_v, false, data6);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3x2fv(mat4_v, false, data6);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix2x4fv(mat4_v, false, data8);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4x2fv(mat4_v, false, data8);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3x4fv(mat4_v, false, data12);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4x3fv(mat4_v, false, data12);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if a sampler is loaded using a command other than glUniform1i and glUniform1iv.');
                gl.useProgram(program.getProgram());
                gl.uniformMatrix2fv(sampler_f, false, data4);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3fv(sampler_f, false, data9);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4fv(sampler_f, false, data16);
                this.expectError(gl.INVALID_OPERATION);

                gl.uniformMatrix2x3fv(sampler_f, false, data6);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3x2fv(sampler_f, false, data6);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix2x4fv(sampler_f, false, data8);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4x2fv(sampler_f, false, data8);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3x4fv(sampler_f, false, data12);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4x3fv(sampler_f, false, data12);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('uniform_matrixfv_invalid_count', 'Invalid glUniformMatrix{234}fv() usage', gl,
            function() {
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(uniformTestVertSource, uniformTestFragSource));

                gl.useProgram(program.getProgram());
                /** @type {WebGLUniformLocation} */ var mat4_v = gl.getUniformLocation(program.getProgram(), 'mat4_v'); // mat4
                this.expectError(gl.NO_ERROR);

                if (mat4_v == null)
                    assertMsgOptions(false, 'Failed to retrieve uniform location', false, true);

                /** @type {Float32Array} */ var data = new Float32Array(144);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if count is greater than 1 and the indicated uniform variable is not an array variable.');
                gl.useProgram(program.getProgram());
                gl.uniformMatrix2fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);

                gl.uniformMatrix2x3fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3x2fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix2x4fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4x2fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix3x4fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);
                gl.uniformMatrix4x3fv(mat4_v, false, data);
                this.expectError(gl.INVALID_OPERATION);

                gl.useProgram(null);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('bind_transform_feedback', 'Invalid gl.bindTransformFeedback() usage', gl,
            function() {
                /** @type {Array<WebGLTransformFeedback>} */ var tfID = [];
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID[0] = gl.createTransformFeedback();
                tfID[1] = gl.createTransformFeedback();

                bufferedLogToConsole('gl.INVALID_ENUM is generated if target is not gl.TRANSFORM_FEEDBACK.');
                gl.bindTransformFeedback(-1, tfID[0]);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the transform feedback operation is active on the currently bound transform feedback object, and is not paused.');
                gl.useProgram(program.getProgram());
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID[0]);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                gl.beginTransformFeedback(gl.TRIANGLES);
                this.expectError(gl.NO_ERROR);

                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID[1]);
                this.expectError(gl.INVALID_OPERATION);

                gl.endTransformFeedback();
                this.expectError(gl.NO_ERROR);

                gl.useProgram(null);
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID[0]);
                gl.deleteTransformFeedback(tfID[1]);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('begin_transform_feedback', 'Invalid gl.beginTransformFeedback() usage', gl,
            function() {
                /** @type {Array<WebGLTransformFeedback>} */ var tfID = [];
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID[0] = gl.createTransformFeedback();
                tfID[1] = gl.createTransformFeedback();

                gl.useProgram(program.getProgram());
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID[0]);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_ENUM is generated if primitiveMode is not one of gl.POINTS, gl.LINES, or gl.TRIANGLES.');
                gl.beginTransformFeedback(-1);
                this.expectError(gl.INVALID_ENUM);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is already active.');
                gl.beginTransformFeedback(gl.TRIANGLES);
                this.expectError(gl.NO_ERROR);
                gl.beginTransformFeedback(gl.POINTS);
                this.expectError(gl.INVALID_OPERATION);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if any binding point used in transform feedback mode does not have a buffer object bound.');
                /** @type{WebGLBuffer} */ var dummyBuf = gl.createBuffer()
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, dummyBuf);
                gl.beginTransformFeedback(gl.TRIANGLES);
                this.expectError(gl.INVALID_OPERATION);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if no binding points would be used because no program object is active.');
                gl.useProgram(null);
                gl.beginTransformFeedback(gl.TRIANGLES);
                this.expectError(gl.INVALID_OPERATION);
                gl.useProgram(program.getProgram());

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if no binding points would be used because the active program object has specified no varying variables to record.');
                gl.transformFeedbackVaryings(program.getProgram(), [], gl.INTERLEAVED_ATTRIBS);
                gl.beginTransformFeedback(gl.TRIANGLES);
                this.expectError(gl.INVALID_OPERATION);

                gl.endTransformFeedback();
                gl.deleteBuffer(buf);
                gl.deleteBuffer(dummyBuf);
                gl.deleteTransformFeedback(tfID[0]);
                gl.deleteTransformFeedback(tfID[1]);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('pause_transform_feedback', 'Invalid gl.pauseTransformFeedback() usage', gl,
            function() {
                /** @type {Array<WebGLTransformFeedback>} */ var tfID = [];
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID[0] = gl.createTransformFeedback();
                tfID[1] = gl.createTransformFeedback();

                gl.useProgram(program.getProgram());
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID[0]);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the currently bound transform feedback object is not active or is paused.');
                gl.pauseTransformFeedback();
                this.expectError(gl.INVALID_OPERATION);
                gl.beginTransformFeedback(gl.TRIANGLES);
                gl.pauseTransformFeedback();
                this.expectError(gl.NO_ERROR);
                gl.pauseTransformFeedback();
                this.expectError(gl.INVALID_OPERATION);

                gl.endTransformFeedback();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID[0]);
                gl.deleteTransformFeedback(tfID[1]);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('resume_transform_feedback', 'Invalid gl.resumeTransformFeedback() usage', gl,
            function() {
                /** @type {Array<WebGLTransformFeedback>} */ var tfID = [];
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID[0] = gl.createTransformFeedback();
                tfID[1] = gl.createTransformFeedback();

                gl.useProgram(program.getProgram());
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID[0]);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if the currently bound transform feedback object is not active or is not paused.');
                gl.resumeTransformFeedback();
                this.expectError(gl.INVALID_OPERATION);
                gl.beginTransformFeedback(gl.TRIANGLES);
                gl.resumeTransformFeedback();
                this.expectError(gl.INVALID_OPERATION);
                gl.pauseTransformFeedback();
                gl.resumeTransformFeedback();
                this.expectError(gl.NO_ERROR);

                gl.endTransformFeedback();
                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID[0]);
                gl.deleteTransformFeedback(tfID[1]);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('end_transform_feedback', 'Invalid gl.endTransformFeedback() usage', gl,
            function() {
                /** @type {WebGLTransformFeedback} */ var tfID;
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {WebGLBuffer} */ var buf;
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];

                buf = gl.createBuffer();
                tfID = gl.createTransformFeedback();

                gl.useProgram(program.getProgram());
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(program.getProgram());
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID);
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buf);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 32, gl.DYNAMIC_DRAW);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('gl.INVALID_OPERATION is generated if transform feedback is not active.');
                gl.endTransformFeedback();
                this.expectError(gl.INVALID_OPERATION);
                gl.beginTransformFeedback(gl.TRIANGLES);
                gl.endTransformFeedback();
                this.expectError(gl.NO_ERROR);

                gl.deleteBuffer(buf);
                gl.deleteTransformFeedback(tfID);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('get_transform_feedback_varying', 'Invalid glGetTransformFeedbackVarying() usage', gl,
            function() {
                /** @type {WebGLTransformFeedback} */ var tfID;
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {gluShaderProgram.ShaderProgram} */ var programInvalid = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, ''));
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];
                /** @type {number} */ var maxTransformFeedbackVaryings = 0;

                /** @type {number} */ var length;
                /** @type {number} */ var size;
                /** @type {number} */ var type;
                /** @type {WebGLActiveInfo} */ var name;

                tfID = gl.createTransformFeedback();

                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.INTERLEAVED_ATTRIBS);
                this.expectError(gl.NO_ERROR);
                gl.linkProgram(program.getProgram());
                this.expectError(gl.NO_ERROR);

                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tfID);
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('An exception is thrown if program is null.');
                this.expectThrowNoError(function() {
                    gl.getTransformFeedbackVarying(null, 0);
                });

                bufferedLogToConsole('gl.INVALID_VALUE is generated if index is greater or equal to the value of gl.TRANSFORM_FEEDBACK_VARYINGS.');
                maxTransformFeedbackVaryings = /** @type {number} */ (gl.getProgramParameter(program.getProgram(), gl.TRANSFORM_FEEDBACK_VARYINGS));
                name = gl.getTransformFeedbackVarying(program.getProgram(), maxTransformFeedbackVaryings);
                this.expectError(gl.INVALID_VALUE);

                bufferedLogToConsole('gl.INVALID_OPERATION or gl.INVALID_VALUE is generated program has not been linked.');
                name = gl.getTransformFeedbackVarying(programInvalid.getProgram(), 0);
                this.expectError([gl.INVALID_OPERATION, gl.INVALID_VALUE]);

                gl.deleteTransformFeedback(tfID);
                this.expectError(gl.NO_ERROR);
            }
        ));

        testGroup.addChild(new es3fApiCase.ApiCaseCallback('transform_feedback_varyings', 'Invalid gl.transformFeedbackVaryings() usage', gl,
            function() {
                /** @type {WebGLTransformFeedback} */ var tfID;
                /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexShaderSource, fragmentShaderSource));
                /** @type {Array<string>} */ var tfVarying = ['gl_Position'];
                /** @type {number} */ var maxTransformFeedbackSeparateAttribs = 0;

                tfID = gl.createTransformFeedback();
                this.expectError(gl.NO_ERROR);

                bufferedLogToConsole('An exception is thrown if program is null.');
                this.expectThrowNoError(function() {
                    gl.transformFeedbackVaryings(null, tfVarying, gl.INTERLEAVED_ATTRIBS);
                });

                bufferedLogToConsole('gl.INVALID_VALUE is generated if bufferMode is gl.SEPARATE_ATTRIBS and count is greater than gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS.');
                maxTransformFeedbackSeparateAttribs = /** @type {number} */ (gl.getParameter(gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS));
                for (var count = 0; count < maxTransformFeedbackSeparateAttribs; ++count) {
                    tfVarying = tfVarying.concat(['gl_Position']);
                }
                gl.transformFeedbackVaryings(program.getProgram(), tfVarying, gl.SEPARATE_ATTRIBS);
                this.expectError(gl.INVALID_VALUE);

                gl.deleteTransformFeedback(tfID);
                this.expectError(gl.NO_ERROR);
            }
        ));
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} gl
     */
     es3fNegativeShaderApiTests.run = function(gl) {
        //Set up Test Root parameters
        var testName = 'negative_shader_api';
        var testDescription = 'Negative Shader Api Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fNegativeShaderApiTests.init(gl);
            tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };
});
