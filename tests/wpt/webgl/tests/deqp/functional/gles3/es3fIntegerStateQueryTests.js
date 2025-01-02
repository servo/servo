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
goog.provide('functional.gles3.es3fIntegerStateQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluTextureUtil');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
        var es3fIntegerStateQueryTests = functional.gles3.es3fIntegerStateQueryTests;
        var tcuTestCase = framework.common.tcuTestCase;
        var deRandom = framework.delibs.debase.deRandom;
        var es3fApiCase = functional.gles3.es3fApiCase;
        var glsStateQuery = modules.shared.glsStateQuery;

        /** @type {string} */ var transformFeedbackTestVertSource = '' +
                '#version 300 es\n' +
                'void main (void)\n' +
                '{\n' +
                '        gl_Position = vec4(0.0);\n' +
                '}\n';

        /** @type {string} */ var transformFeedbackTestFragSource = '' +
                '#version 300 es\n' +
                'layout(location = 0) out mediump vec4 fragColor;' +
                'void main (void)\n' +
                '{\n' +
                '        fragColor = vec4(0.0);\n' +
                '}\n';

        /** @type {string} */ var testVertSource = '' +
                '#version 300 es\n' +
                'void main (void)\n' +
                '{\n' +
                '        gl_Position = vec4(0.0);\n' +
                '}\n';

        /** @type {string} */ var testFragSource = '' +
                '#version 300 es\n' +
                'layout(location = 0) out mediump vec4 fragColor;' +
                'void main (void)\n' +
                '{\n' +
                '        fragColor = vec4(0.0);\n' +
                '}\n';

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.TransformFeedbackTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {WebGLTransformFeedback} */ this.m_transformfeedback;
        };

        es3fIntegerStateQueryTests.TransformFeedbackTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.TransformFeedbackTestCase.prototype.constructor = es3fIntegerStateQueryTests.TransformFeedbackTestCase;

        es3fIntegerStateQueryTests.TransformFeedbackTestCase.prototype.testTransformFeedback = function() {
                throw new Error('This method should be implemented by child classes.');
        };

        es3fIntegerStateQueryTests.TransformFeedbackTestCase.prototype.test = function() {
                this.beforeTransformFeedbackTest(); // [dag] added this as there is no other way this method would be called.

                this.m_transformfeedback = gl.createTransformFeedback();

                /** @type {WebGLShader} */ var shaderVert = gl.createShader(gl.VERTEX_SHADER);
                gl.shaderSource(shaderVert, transformFeedbackTestVertSource);
                gl.compileShader(shaderVert);

                var compileStatus = /** @type {boolean} */ (gl.getShaderParameter(shaderVert, gl.COMPILE_STATUS));
                glsStateQuery.compare(compileStatus, true);

                /** @type {WebGLShader} */ var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);
                gl.shaderSource(shaderFrag, transformFeedbackTestFragSource);
                gl.compileShader(shaderFrag);

                compileStatus = /** @type {boolean} */ (gl.getShaderParameter(shaderFrag, gl.COMPILE_STATUS));
                glsStateQuery.compare(compileStatus, true);

                /** @type {WebGLProgram} */ var shaderProg = gl.createProgram();
                gl.attachShader(shaderProg, shaderVert);
                gl.attachShader(shaderProg, shaderFrag);
                /** @type {Array<string>} */ var transform_feedback_outputs = ['gl_Position'];
                gl.transformFeedbackVaryings(shaderProg, transform_feedback_outputs, gl.INTERLEAVED_ATTRIBS);
                gl.linkProgram(shaderProg);

                var linkStatus = /** @type {boolean} */ (gl.getProgramParameter(shaderProg, gl.LINK_STATUS));
                glsStateQuery.compare(linkStatus, true);

                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, this.m_transformfeedback);


                /** @type {WebGLBuffer} */ var feedbackBufferId = gl.createBuffer();
                gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackBufferId);
                gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, new Float32Array(16), gl.DYNAMIC_READ);
                gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, feedbackBufferId);

                gl.useProgram(shaderProg);

                this.testTransformFeedback();

                gl.useProgram(null);
                gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
                gl.deleteTransformFeedback(this.m_transformfeedback);
                gl.deleteBuffer(feedbackBufferId);
                gl.deleteShader(shaderVert);
                gl.deleteShader(shaderFrag);
                gl.deleteProgram(shaderProg);

                this.afterTransformFeedbackTest(); // [dag] added this as there is no other way this method would be called.
        };

        /**
         * @constructor
         * @extends {es3fIntegerStateQueryTests.TransformFeedbackTestCase}
         * @param {string} name
         */
        es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase = function(name) {
                es3fIntegerStateQueryTests.TransformFeedbackTestCase.call(this, name, 'GL_TRANSFORM_FEEDBACK_BINDING');
        };

        es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase.prototype = Object.create(es3fIntegerStateQueryTests.TransformFeedbackTestCase.prototype);
        es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase;


        es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase.prototype.beforeTransformFeedbackTest = function() {
                this.check(glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_BINDING, null), 'beforeTransformFeedbackTest');
        };

        es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase.prototype.testTransformFeedback = function() {
                this.check(glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_BINDING, this.m_transformfeedback), 'testTransformFeedback');
        };

        es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase.prototype.afterTransformFeedbackTest = function() {
                this.check(glsStateQuery.verify(gl.TRANSFORM_FEEDBACK_BINDING, null), 'afterTransformFeedbackTest');
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} targetName
         * @param {number} minValue
         */
        es3fIntegerStateQueryTests.ConstantMinimumValueTestCase = function(name, description, targetName, minValue) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_targetName = targetName;
                /** @type {number} */ this.m_minValue = minValue;
        };

        es3fIntegerStateQueryTests.ConstantMinimumValueTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ConstantMinimumValueTestCase.prototype.constructor = es3fIntegerStateQueryTests.ConstantMinimumValueTestCase;

        es3fIntegerStateQueryTests.ConstantMinimumValueTestCase.prototype.test = function() {
                this.check(glsStateQuery.verifyGreaterOrEqual(this.m_targetName, this.m_minValue), 'Fail');
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} targetName
         * @param {number} minValue
         */
        es3fIntegerStateQueryTests.ConstantMaximumValueTestCase = function(name, description, targetName, minValue) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_targetName = targetName;
                /** @type {number} */ this.m_minValue = minValue;
        };

        es3fIntegerStateQueryTests.ConstantMaximumValueTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ConstantMaximumValueTestCase.prototype.constructor = es3fIntegerStateQueryTests.ConstantMaximumValueTestCase;

        es3fIntegerStateQueryTests.ConstantMaximumValueTestCase.prototype.test = function() {
                this.check(glsStateQuery.verifyLessOrEqual(this.m_targetName, this.m_minValue), 'Fail');
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.SampleBuffersTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.SampleBuffersTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.SampleBuffersTestCase.prototype.constructor = es3fIntegerStateQueryTests.SampleBuffersTestCase;

        es3fIntegerStateQueryTests.SampleBuffersTestCase.prototype.test = function() {
                /** @type {number} */ var expectedSampleBuffers = (/** @type {number} */ (gl.getParameter(gl.SAMPLES)) > 1) ? 1 : 0;

                bufferedLogToConsole('Sample count is ' + expectedSampleBuffers + ', expecting GL_SAMPLE_BUFFERS to be ' + expectedSampleBuffers);

                this.check(glsStateQuery.verify(gl.SAMPLE_BUFFERS, expectedSampleBuffers));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.SamplesTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.SamplesTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.SamplesTestCase.prototype.constructor = es3fIntegerStateQueryTests.SamplesTestCase;

        es3fIntegerStateQueryTests.SamplesTestCase.prototype.test = function() {
                var numSamples = /** @type {number} */ (gl.getParameter(gl.SAMPLES));
                // MSAA?
                if (numSamples > 1) {
                        bufferedLogToConsole('Sample count is ' + numSamples);

                        this.check(glsStateQuery.verify(gl.SAMPLES, numSamples));
                } else {
                        /** @type {Array<number>} */ var validSamples = [0, 1];

                        bufferedLogToConsole('Expecting GL_SAMPLES to be 0 or 1');

                        this.check(glsStateQuery.verifyAnyOf(gl.SAMPLES, validSamples));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} targetName
         */
        es3fIntegerStateQueryTests.HintTestCase = function(name, description, targetName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_targetName = targetName;
        };

        es3fIntegerStateQueryTests.HintTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.HintTestCase.prototype.constructor = es3fIntegerStateQueryTests.HintTestCase;

        es3fIntegerStateQueryTests.HintTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_targetName, gl.DONT_CARE));

                gl.hint(this.m_targetName, gl.NICEST);
                this.check(glsStateQuery.verify(this.m_targetName, gl.NICEST));

                gl.hint(this.m_targetName, gl.FASTEST);
                this.check(glsStateQuery.verify(this.m_targetName, gl.FASTEST));

                gl.hint(this.m_targetName, gl.DONT_CARE);
                this.check(glsStateQuery.verify(this.m_targetName, gl.DONT_CARE));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.DepthFuncTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.DepthFuncTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.DepthFuncTestCase.prototype.constructor = es3fIntegerStateQueryTests.DepthFuncTestCase;

        es3fIntegerStateQueryTests.DepthFuncTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.DEPTH_FUNC, gl.LESS));

                /** @type {Array<number>} */ var depthFunctions = [gl.NEVER, gl.ALWAYS, gl.LESS, gl.LEQUAL, gl.EQUAL, gl.GREATER, gl.GEQUAL, gl.NOTEQUAL];
                for (var ndx = 0; ndx < depthFunctions.length; ndx++) {
                        gl.depthFunc(depthFunctions[ndx]);

                        this.check(glsStateQuery.verify(gl.DEPTH_FUNC, depthFunctions[ndx]));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.CullFaceTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.CullFaceTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.CullFaceTestCase.prototype.constructor = es3fIntegerStateQueryTests.CullFaceTestCase;

        es3fIntegerStateQueryTests.CullFaceTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.CULL_FACE_MODE, gl.BACK));

                /** @type {Array<number>} */ var cullFaces = [gl.FRONT, gl.BACK, gl.FRONT_AND_BACK];
                for (var ndx = 0; ndx < cullFaces.length; ndx++) {
                        gl.cullFace(cullFaces[ndx]);

                        this.check(glsStateQuery.verify(gl.CULL_FACE_MODE, cullFaces[ndx]));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.FrontFaceTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.FrontFaceTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.FrontFaceTestCase.prototype.constructor = es3fIntegerStateQueryTests.FrontFaceTestCase;

        es3fIntegerStateQueryTests.FrontFaceTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.FRONT_FACE, gl.CCW));

                /** @type {Array<number>} */ var frontFaces = [gl.CW, gl.CCW];
                for (var ndx = 0; ndx < frontFaces.length; ndx++) {
                        gl.frontFace(frontFaces[ndx]);

                        this.check(glsStateQuery.verify(gl.FRONT_FACE, frontFaces[ndx]));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.ViewPortTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.ViewPortTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ViewPortTestCase.prototype.constructor = es3fIntegerStateQueryTests.ViewPortTestCase;

        es3fIntegerStateQueryTests.ViewPortTestCase.prototype.test = function() {
                /** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

                var maxViewportDimensions = /** @type {Array<number>} */ (gl.getParameter(gl.MAX_VIEWPORT_DIMS));

                // verify initial value of first two values
                this.check(glsStateQuery.verify(gl.VIEWPORT, new Int32Array([0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight])));

                /** @type {number} */ var numIterations = 120;
                for (var i = 0; i < numIterations; ++i) {
                        /** @type {number} */ var x = rnd.getInt(-64000, 64000);
                        /** @type {number} */ var y = rnd.getInt(-64000, 64000);
                        /** @type {number} */ var width = rnd.getInt(0, maxViewportDimensions[0]);
                        /** @type {number} */ var height = rnd.getInt(0, maxViewportDimensions[1]);

                        gl.viewport(x, y, width, height);
                        this.check(glsStateQuery.verify(gl.VIEWPORT, new Int32Array([x, y, width, height])));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.ScissorBoxTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.ScissorBoxTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ScissorBoxTestCase.prototype.constructor = es3fIntegerStateQueryTests.ScissorBoxTestCase;

        es3fIntegerStateQueryTests.ScissorBoxTestCase.prototype.test = function() {
                /** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

                // verify initial value of first two values
                this.check(glsStateQuery.verifyMask(gl.SCISSOR_BOX, [0, 0, 0, 0], [true, true, false, false]));

                /** @type {number} */ var numIterations = 120;
                for (var i = 0; i < numIterations; ++i) {
                        /** @type {number} */ var left = rnd.getInt(-64000, 64000);
                        /** @type {number} */ var bottom = rnd.getInt(-64000, 64000);
                        /** @type {number} */ var width = rnd.getInt(0, 64000);
                        /** @type {number} */ var height = rnd.getInt(0, 64000);

                        gl.scissor(left, bottom, width, height);
                        this.check(glsStateQuery.verify(gl.SCISSOR_BOX, new Int32Array([left, bottom, width, height])));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.MaxViewportDimsTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.MaxViewportDimsTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.MaxViewportDimsTestCase.prototype.constructor = es3fIntegerStateQueryTests.MaxViewportDimsTestCase;

        es3fIntegerStateQueryTests.MaxViewportDimsTestCase.prototype.test = function() {
                this.check(glsStateQuery.verifyGreaterOrEqual(gl.MAX_VIEWPORT_DIMS, [gl.drawingBufferWidth, gl.drawingBufferHeight]));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         */
        es3fIntegerStateQueryTests.StencilRefTestCase = function(name, description, testTargetName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
        };

        es3fIntegerStateQueryTests.StencilRefTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilRefTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilRefTestCase;

        es3fIntegerStateQueryTests.StencilRefTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_testTargetName, 0));

                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var ref = 1 << stencilBit;

                        gl.stencilFunc(gl.ALWAYS, ref, 0); // mask should not affect the REF

                        this.check(glsStateQuery.verify(this.m_testTargetName, ref));

                        gl.stencilFunc(gl.ALWAYS, ref, ref);

                        this.check(glsStateQuery.verify(this.m_testTargetName, ref));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} stencilFuncTargetFace
         */
        es3fIntegerStateQueryTests.StencilRefSeparateTestCase = function(name, description, testTargetName, stencilFuncTargetFace) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
                /** @type {number} */ this.m_stencilFuncTargetFace = stencilFuncTargetFace;
        };

        es3fIntegerStateQueryTests.StencilRefSeparateTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilRefSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilRefSeparateTestCase;

        es3fIntegerStateQueryTests.StencilRefSeparateTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_testTargetName, 0));

                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var ref = 1 << stencilBit;

                        gl.stencilFuncSeparate(this.m_stencilFuncTargetFace, gl.ALWAYS, ref, 0);

                        this.check(glsStateQuery.verify(this.m_testTargetName, ref));

                        gl.stencilFuncSeparate(this.m_stencilFuncTargetFace, gl.ALWAYS, ref, ref);

                        this.check(glsStateQuery.verify(this.m_testTargetName, ref));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} stencilOpName
         */
        es3fIntegerStateQueryTests.StencilOpTestCase = function(name, description, stencilOpName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_stencilOpName = stencilOpName;
        };

        es3fIntegerStateQueryTests.StencilOpTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilOpTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilOpTestCase;

        es3fIntegerStateQueryTests.StencilOpTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_stencilOpName, gl.KEEP));

                /** @type {Array<number>} */ var stencilOpValues = [gl.KEEP, gl.ZERO, gl.REPLACE, gl.INCR, gl.DECR, gl.INVERT, gl.INCR_WRAP, gl.DECR_WRAP];
                for (var ndx = 0; ndx < stencilOpValues.length; ++ndx) {
                        this.setStencilOp(stencilOpValues[ndx]);

                        this.check(glsStateQuery.verify(this.m_stencilOpName, stencilOpValues[ndx]));
                }
        };

        es3fIntegerStateQueryTests.StencilOpTestCase.prototype.deinit = function() {
                // [dag] need to reset everything once the test is done, otherwise related tests fail
                gl.stencilOp(gl.KEEP, gl.KEEP, gl.KEEP);
        };

        /**
         * @param  {number} stencilOpValue
         */
        es3fIntegerStateQueryTests.StencilOpTestCase.prototype.setStencilOp = function(stencilOpValue) {
                switch (this.m_stencilOpName) {
                        case gl.STENCIL_FAIL:
                        case gl.STENCIL_BACK_FAIL:
                                gl.stencilOp(stencilOpValue, gl.KEEP, gl.KEEP);
                                break;

                        case gl.STENCIL_PASS_DEPTH_FAIL:
                        case gl.STENCIL_BACK_PASS_DEPTH_FAIL:
                                gl.stencilOp(gl.KEEP, stencilOpValue, gl.KEEP);
                                break;

                        case gl.STENCIL_PASS_DEPTH_PASS:
                        case gl.STENCIL_BACK_PASS_DEPTH_PASS:
                                gl.stencilOp(gl.KEEP, gl.KEEP, stencilOpValue);
                                break;

                        default:
                                throw new Error('should not happen');
                }
        };

        /**
         * @constructor
         * @extends {es3fIntegerStateQueryTests.StencilOpTestCase}
         * @param {string} name
         * @param {string} description
         * @param {number} stencilOpName
         * @param {number} stencilOpFace
         */
        es3fIntegerStateQueryTests.StencilOpSeparateTestCase = function(name, description, stencilOpName, stencilOpFace) {
                es3fIntegerStateQueryTests.StencilOpTestCase.call(this, name, description, stencilOpName);
                /** @type {number} */ this.m_stencilOpName = stencilOpName;
                /** @type {number} */ this.m_stencilOpFace = stencilOpFace;
        };

        es3fIntegerStateQueryTests.StencilOpSeparateTestCase.prototype = Object.create(es3fIntegerStateQueryTests.StencilOpTestCase.prototype);
        es3fIntegerStateQueryTests.StencilOpSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilOpSeparateTestCase;

        es3fIntegerStateQueryTests.StencilOpSeparateTestCase.prototype.test = function() {};

        /**
         * @param  {number} stencilOpValue
         */
        es3fIntegerStateQueryTests.StencilOpSeparateTestCase.prototype.setStencilOp = function(stencilOpValue) {
                switch (this.m_stencilOpName) {
                        case gl.STENCIL_FAIL:
                        case gl.STENCIL_BACK_FAIL:
                                gl.stencilOpSeparate(this.m_stencilOpFace, stencilOpValue, gl.KEEP, gl.KEEP);
                                break;

                        case gl.STENCIL_PASS_DEPTH_FAIL:
                        case gl.STENCIL_BACK_PASS_DEPTH_FAIL:
                                gl.stencilOpSeparate(this.m_stencilOpFace, gl.KEEP, stencilOpValue, gl.KEEP);
                                break;

                        case gl.STENCIL_PASS_DEPTH_PASS:
                        case gl.STENCIL_BACK_PASS_DEPTH_PASS:
                                gl.stencilOpSeparate(this.m_stencilOpFace, gl.KEEP, gl.KEEP, stencilOpValue);
                                break;

                        default:
                                throw new Error('should not happen');
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.StencilFuncTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.StencilFuncTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilFuncTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilFuncTestCase;

        es3fIntegerStateQueryTests.StencilFuncTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.STENCIL_FUNC, gl.ALWAYS));

                /** @type {Array<number>} */ var stencilfuncValues = [gl.NEVER, gl.ALWAYS, gl.LESS, gl.LEQUAL, gl.EQUAL, gl.GEQUAL, gl.GREATER, gl.NOTEQUAL];

                for (var ndx = 0; ndx < stencilfuncValues.length; ++ndx) {
                        gl.stencilFunc(stencilfuncValues[ndx], 0, 0);

                        this.check(glsStateQuery.verify(gl.STENCIL_FUNC, stencilfuncValues[ndx]));

                        this.check(glsStateQuery.verify(gl.STENCIL_BACK_FUNC, stencilfuncValues[ndx]));
                }
        };

        es3fIntegerStateQueryTests.StencilFuncTestCase.prototype.deinit = function() {
                // [dag] reset stencilFunc to ALWAYS
                gl.stencilFunc(gl.ALWAYS, 0, 0);
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} stencilFuncName
         * @param {number} stencilFuncFace
         */
        es3fIntegerStateQueryTests.StencilFuncSeparateTestCase = function(name, description, stencilFuncName, stencilFuncFace) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_stencilFuncName = stencilFuncName;
                /** @type {number} */ this.m_stencilFuncFace = stencilFuncFace;
        };

        es3fIntegerStateQueryTests.StencilFuncSeparateTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilFuncSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilFuncSeparateTestCase;

        es3fIntegerStateQueryTests.StencilFuncSeparateTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_stencilFuncName, gl.ALWAYS));

                /** @type {Array<number>} */ var stencilfuncValues = [gl.NEVER, gl.ALWAYS, gl.LESS, gl.LEQUAL, gl.EQUAL, gl.GEQUAL, gl.GREATER, gl.NOTEQUAL];

                for (var ndx = 0; ndx < stencilfuncValues.length; ++ndx) {
                        gl.stencilFuncSeparate(this.m_stencilFuncFace, stencilfuncValues[ndx], 0, 0);

                        this.check(glsStateQuery.verify(this.m_stencilFuncName, stencilfuncValues[ndx]));
                }
        };

        es3fIntegerStateQueryTests.StencilFuncSeparateTestCase.prototype.deinit = function() {
                // [dag] reset the stencil func
                gl.stencilFuncSeparate(this.m_stencilFuncFace, gl.ALWAYS, 0, 0);
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         */
        es3fIntegerStateQueryTests.StencilMaskTestCase = function(name, description, testTargetName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
        };

        es3fIntegerStateQueryTests.StencilMaskTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilMaskTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilMaskTestCase;

        es3fIntegerStateQueryTests.StencilMaskTestCase.prototype.test = function() {
                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                this.check(glsStateQuery.verify(this.m_testTargetName, stencilBits));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var mask = 1 << stencilBit;

                        gl.stencilFunc(gl.ALWAYS, 0, mask);

                        this.check(glsStateQuery.verify(this.m_testTargetName, mask));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} stencilFuncTargetFace
         */
        es3fIntegerStateQueryTests.StencilMaskSeparateTestCase = function(name, description, testTargetName, stencilFuncTargetFace) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
                /** @type {number} */ this.m_stencilFuncTargetFace = stencilFuncTargetFace;
        };

        es3fIntegerStateQueryTests.StencilMaskSeparateTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilMaskSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilMaskSeparateTestCase;

        es3fIntegerStateQueryTests.StencilMaskSeparateTestCase.prototype.test = function() {
                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                this.check(glsStateQuery.verify(this.m_testTargetName, stencilBits));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var mask = 1 << stencilBit;

                        gl.stencilFuncSeparate(this.m_stencilFuncTargetFace, gl.ALWAYS, 0, mask);

                        this.check(glsStateQuery.verify(this.m_testTargetName, mask));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         */
        es3fIntegerStateQueryTests.StencilWriteMaskTestCase = function(name, description, testTargetName) {
                /** @type {number} */ this.m_testTargetName = testTargetName;
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.StencilWriteMaskTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilWriteMaskTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilWriteMaskTestCase;

        es3fIntegerStateQueryTests.StencilWriteMaskTestCase.prototype.test = function() {
                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var mask = 1 << stencilBit;

                        gl.stencilMask(mask);

                        this.check(glsStateQuery.verify(this.m_testTargetName, mask));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} stencilTargetFace
         */
        es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase = function(name, description, testTargetName, stencilTargetFace) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
                /** @type {number} */ this.m_stencilTargetFace = stencilTargetFace;
        };

        es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase;

        es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase.prototype.test = function() {
                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var mask = 1 << stencilBit;

                        gl.stencilMaskSeparate(this.m_stencilTargetFace, mask);

                        this.check(glsStateQuery.verify(this.m_testTargetName, mask));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} initialValue
         */
        es3fIntegerStateQueryTests.PixelStoreTestCase = function(name, description, testTargetName, initialValue) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
                /** @type {number} */ this.m_initialValue = initialValue;
        };

        es3fIntegerStateQueryTests.PixelStoreTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.PixelStoreTestCase.prototype.constructor = es3fIntegerStateQueryTests.PixelStoreTestCase;

        es3fIntegerStateQueryTests.PixelStoreTestCase.prototype.test = function() {
                /** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

                this.check(glsStateQuery.verify(this.m_testTargetName, this.m_initialValue));

                /** @type {number} */ var numIterations = 120;
                for (var i = 0; i < numIterations; ++i) {
                        /** @type {number} */ var referenceValue = rnd.getInt(0, 64000);

                        gl.pixelStorei(this.m_testTargetName, referenceValue);

                        this.check(glsStateQuery.verify(this.m_testTargetName, referenceValue));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         */
        es3fIntegerStateQueryTests.PixelStoreAlignTestCase = function(name, description, testTargetName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
        };

        es3fIntegerStateQueryTests.PixelStoreAlignTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.PixelStoreAlignTestCase.prototype.constructor = es3fIntegerStateQueryTests.PixelStoreAlignTestCase;

        es3fIntegerStateQueryTests.PixelStoreAlignTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_testTargetName, 4));

                /** @type {Array<number>} */ var alignments = [1, 2, 4, 8];

                for (var ndx = 0; ndx < alignments.length; ++ndx) {
                        /** @type {number} */ var referenceValue = alignments[ndx];

                        gl.pixelStorei(this.m_testTargetName, referenceValue);

                        this.check(glsStateQuery.verify(this.m_testTargetName, referenceValue));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} initialValue
         */
        es3fIntegerStateQueryTests.BlendFuncTestCase = function(name, description, testTargetName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_testTargetName = testTargetName;
        };

        es3fIntegerStateQueryTests.BlendFuncTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.BlendFuncTestCase.prototype.constructor = es3fIntegerStateQueryTests.BlendFuncTestCase;

        es3fIntegerStateQueryTests.BlendFuncTestCase.prototype.test = function() {
                /** @type {Array<number>} */ var blendFuncValues = [
                        gl.ZERO, gl.ONE, gl.SRC_COLOR, gl.ONE_MINUS_SRC_COLOR, gl.DST_COLOR, gl.ONE_MINUS_DST_COLOR,
                        gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.DST_ALPHA, gl.ONE_MINUS_DST_ALPHA, gl.CONSTANT_COLOR,
                        gl.ONE_MINUS_CONSTANT_COLOR, gl.CONSTANT_ALPHA, gl.ONE_MINUS_CONSTANT_ALPHA,
                        gl.SRC_ALPHA_SATURATE
                ];

                for (var ndx = 0; ndx < blendFuncValues.length; ++ndx) {
                        /** @type {number} */ var referenceValue = blendFuncValues[ndx];

                        this.setBlendFunc(referenceValue);

                        this.check(glsStateQuery.verify(this.m_testTargetName, referenceValue));
                }};

        /**
         * @param  {number} func
         */
        es3fIntegerStateQueryTests.BlendFuncTestCase.prototype.setBlendFunc = function(func) {
                switch (this.m_testTargetName) {
                        case gl.BLEND_SRC_RGB:
                        case gl.BLEND_SRC_ALPHA:
                                gl.blendFunc(func, gl.ZERO);
                                break;

                        case gl.BLEND_DST_RGB:
                        case gl.BLEND_DST_ALPHA:
                                gl.blendFunc(gl.ZERO, func);
                                break;

                        default:
                                throw new Error('should not happen');
                }
        };

        es3fIntegerStateQueryTests.BlendFuncTestCase.prototype.deinit = function() {
                gl.blendFunc(gl.ONE, gl.ZERO);
        };

        /**
         * @constructor
         * @extends {es3fIntegerStateQueryTests.BlendFuncTestCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} initialValue
         */
        es3fIntegerStateQueryTests.BlendFuncSeparateTestCase = function(name, description, testTargetName) {
                es3fIntegerStateQueryTests.BlendFuncTestCase.call(this, name, description, testTargetName);
                /** @type {number} */ this.m_testTargetName = testTargetName;
        };

        es3fIntegerStateQueryTests.BlendFuncSeparateTestCase.prototype = Object.create(es3fIntegerStateQueryTests.BlendFuncTestCase.prototype);
        es3fIntegerStateQueryTests.BlendFuncSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.BlendFuncSeparateTestCase;

        /**
         * @param  {number} func
         */
        es3fIntegerStateQueryTests.BlendFuncSeparateTestCase.prototype.setBlendFunc = function(func) {
                switch (this.m_testTargetName) {
                        case gl.BLEND_SRC_RGB:
                                gl.blendFuncSeparate(func, gl.ZERO, gl.ZERO, gl.ZERO);
                                break;

                        case gl.BLEND_DST_RGB:
                                gl.blendFuncSeparate(gl.ZERO, func, gl.ZERO, gl.ZERO);
                                break;

                        case gl.BLEND_SRC_ALPHA:
                                gl.blendFuncSeparate(gl.ZERO, gl.ZERO, func, gl.ZERO);
                                break;

                        case gl.BLEND_DST_ALPHA:
                                gl.blendFuncSeparate(gl.ZERO, gl.ZERO, gl.ZERO, func);
                                break;

                        default:
                                throw new Error('should not happen');
                }
        };

        es3fIntegerStateQueryTests.BlendFuncSeparateTestCase.prototype.deinit = function() {
                gl.blendFuncSeparate(gl.ONE, gl.ZERO, gl.ONE, gl.ZERO);
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} initialValue
         */
        es3fIntegerStateQueryTests.BlendEquationTestCase = function(name, description, testTargetName, initialValue) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
         /** @type {number} */ this.m_testTargetName = testTargetName;
         /** @type {number} */ this.m_initialValue = initialValue;
        };

        es3fIntegerStateQueryTests.BlendEquationTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.BlendEquationTestCase.prototype.constructor = es3fIntegerStateQueryTests.BlendEquationTestCase;

        es3fIntegerStateQueryTests.BlendEquationTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_testTargetName, this.m_initialValue));

                /** @type {Array<number>} */ var blendFuncValues = [gl.FUNC_ADD, gl.FUNC_SUBTRACT, gl.FUNC_REVERSE_SUBTRACT, gl.MIN, gl.MAX];

                for (var ndx = 0; ndx < blendFuncValues.length; ++ndx) {
                        /** @type {number} */ var referenceValue = blendFuncValues[ndx];

                        this.setBlendEquation(referenceValue);

                        this.check(glsStateQuery.verify(this.m_testTargetName, referenceValue));
                }
        };

        /**
         * @param  {number} equation
         */
        es3fIntegerStateQueryTests.BlendEquationTestCase.prototype.setBlendEquation = function(equation) {
                gl.blendEquation(equation);
        };

        es3fIntegerStateQueryTests.BlendEquationTestCase.prototype.deinit = function() {
                gl.blendEquation(this.m_initialValue);
        };

        /**
         * @constructor
         * @extends {es3fIntegerStateQueryTests.BlendEquationTestCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} initialValue
         */
        es3fIntegerStateQueryTests.BlendEquationSeparateTestCase = function(name, description, testTargetName, initialValue) {
                es3fIntegerStateQueryTests.BlendEquationTestCase.call(this, name, description, testTargetName, initialValue);
         /** @type {number} */ this.m_testTargetName = testTargetName;
         /** @type {number} */ this.m_initialValue = initialValue;
        };

        es3fIntegerStateQueryTests.BlendEquationSeparateTestCase.prototype = Object.create(es3fIntegerStateQueryTests.BlendEquationTestCase.prototype);
        es3fIntegerStateQueryTests.BlendEquationSeparateTestCase.prototype.constructor = es3fIntegerStateQueryTests.BlendEquationSeparateTestCase;

        /**
         * @param  {number} equation
         */
        es3fIntegerStateQueryTests.BlendEquationSeparateTestCase.prototype.setBlendEquation = function(equation) {
                switch (this.m_testTargetName) {
                        case gl.BLEND_EQUATION_RGB:
                                gl.blendEquationSeparate(equation, gl.FUNC_ADD);
                                break;

                        case gl.BLEND_EQUATION_ALPHA:
                                gl.blendEquationSeparate(gl.FUNC_ADD, equation);
                                break;

                        default:
                                throw new Error('should not happen');
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testTargetName
         * @param {number} minValue
         */
        es3fIntegerStateQueryTests.ImplementationArrayTestCase = function(name, description, testTargetName, minValue) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                 /** @type {number} */ this.m_testTargetName = testTargetName;
                 /** @type {number} */ this.m_minValue = minValue;
        };

        es3fIntegerStateQueryTests.ImplementationArrayTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ImplementationArrayTestCase.prototype.constructor = es3fIntegerStateQueryTests.ImplementationArrayTestCase;

        es3fIntegerStateQueryTests.ImplementationArrayTestCase.prototype.test = function() {
                if (!framework.opengl.gluTextureUtil.enableCompressedTextureETC()) {
                        debug('Skipping ETC2 texture format tests: no support for WEBGL_compressed_texture_etc');
                        return;
                }

                var queryResult = /** @type {Array<number>} */ (gl.getParameter(this.m_testTargetName));
                this.check(glsStateQuery.compare(queryResult.length, this.m_minValue));

                /** @type {Array<number>} */ var textureFormats = [
                        gl.COMPRESSED_R11_EAC, gl.COMPRESSED_SIGNED_R11_EAC, gl.COMPRESSED_RG11_EAC, gl.COMPRESSED_SIGNED_RG11_EAC, gl.COMPRESSED_RGB8_ETC2, gl.COMPRESSED_SRGB8_ETC2,
                        gl.COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2, gl.COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2, gl.COMPRESSED_RGBA8_ETC2_EAC, gl.COMPRESSED_SRGB8_ALPHA8_ETC2_EAC
                ];

                for (var ndx = 0; ndx < textureFormats.length; ndx++) {
                        /** @type {number} */ var format = textureFormats[ndx];
                        /** @type {boolean} */ var isInArray = queryResult.indexOf(format) !== -1;
                        this.check(glsStateQuery.compare(isInArray, true));
                }

        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.CurrentProgramBindingTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.CurrentProgramBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.CurrentProgramBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.CurrentProgramBindingTestCase;

        es3fIntegerStateQueryTests.CurrentProgramBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.CURRENT_PROGRAM, null));

                /** @type {WebGLShader} */ var shaderVert = gl.createShader(gl.VERTEX_SHADER);
                gl.shaderSource(shaderVert, testVertSource);
                gl.compileShader(shaderVert);
                var compileStatus = /** @type {boolean} */ (gl.getShaderParameter(shaderVert, gl.COMPILE_STATUS));
                this.check(glsStateQuery.compare(compileStatus, true));

                /** @type {WebGLShader} */ var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);
                gl.shaderSource(shaderFrag, testFragSource);
                gl.compileShader(shaderFrag);
                compileStatus = /** @type {boolean} */ (gl.getShaderParameter(shaderFrag, gl.COMPILE_STATUS));
                this.check(glsStateQuery.compare(compileStatus, true));

                /** @type {WebGLProgram} */ var shaderProg = gl.createProgram();
                gl.attachShader(shaderProg, shaderVert);
                gl.attachShader(shaderProg, shaderFrag);
                gl.linkProgram(shaderProg);
                var linkStatus = /** @type {boolean} */ (gl.getProgramParameter(shaderProg, gl.LINK_STATUS));
                this.check(glsStateQuery.compare(linkStatus, true));

                gl.useProgram(shaderProg);

                this.check(glsStateQuery.verify(gl.CURRENT_PROGRAM, shaderProg));

                gl.deleteShader(shaderVert);
                gl.deleteShader(shaderFrag);
                gl.deleteProgram(shaderProg);

                this.check(glsStateQuery.verify(gl.CURRENT_PROGRAM, shaderProg));

                gl.useProgram(null);
                this.check(glsStateQuery.verify(gl.CURRENT_PROGRAM, null));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.VertexArrayBindingTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.VertexArrayBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.VertexArrayBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.VertexArrayBindingTestCase;

        es3fIntegerStateQueryTests.VertexArrayBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.VERTEX_ARRAY_BINDING, null));

                /** @type {WebGLVertexArrayObject} */ var vertexArrayObject = gl.createVertexArray();

                gl.bindVertexArray(vertexArrayObject);
                this.check(glsStateQuery.verify(gl.VERTEX_ARRAY_BINDING, vertexArrayObject));

                gl.deleteVertexArray(vertexArrayObject);
                this.check(glsStateQuery.verify(gl.VERTEX_ARRAY_BINDING, null));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} bufferBindingName
         * @param {number} bufferType
         */
        es3fIntegerStateQueryTests.BufferBindingTestCase = function(name, description, bufferBindingName, bufferType) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
         /** @type {number} */ this.m_bufferBindingName = bufferBindingName;
         /** @type {number} */ this.m_bufferType = bufferType;
        };

        es3fIntegerStateQueryTests.BufferBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.BufferBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.BufferBindingTestCase;

        es3fIntegerStateQueryTests.BufferBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_bufferBindingName, null));

                /** @type {WebGLBuffer} */ var bufferObject = gl.createBuffer();

                gl.bindBuffer(this.m_bufferType, bufferObject);
                this.check(glsStateQuery.verify(this.m_bufferBindingName, bufferObject));

                gl.deleteBuffer(bufferObject);
                this.check(glsStateQuery.verify(this.m_bufferBindingName, null));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         */
        es3fIntegerStateQueryTests.ElementArrayBufferBindingTestCase = function(name) {
                es3fApiCase.ApiCase.call(this, name, 'GL_ELEMENT_ARRAY_BUFFER_BINDING', gl);
        };

        es3fIntegerStateQueryTests.ElementArrayBufferBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ElementArrayBufferBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.ElementArrayBufferBindingTestCase;

        es3fIntegerStateQueryTests.ElementArrayBufferBindingTestCase.prototype.test = function() {
                // Test with default VAO
                bufferedLogToConsole('DefaultVAO: Test with default VAO');

                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, null));

                /** @type {WebGLBuffer} */ var bufferObject = gl.createBuffer();

                gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, bufferObject);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, bufferObject));

                gl.deleteBuffer(bufferObject);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, null));

                // Test with multiple VAOs
                bufferedLogToConsole('WithVAO: Test with VAO');

                /** @type {Array<WebGLVertexArrayObject>} */ var vaos = [];
                /** @type {Array<WebGLBuffer>} */ var buffers = [];

                for (var ndx = 0; ndx < 2; ndx++) {
                        vaos[ndx] = gl.createVertexArray();
                        buffers[ndx] = gl.createBuffer();
                }

                // initial
                gl.bindVertexArray(vaos[0]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, null));

                // after setting
                gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffers[0]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, buffers[0]));

                // initial of vao 2
                gl.bindVertexArray(vaos[1]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, null));

                // after setting to 2
                gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffers[1]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, buffers[1]));

                // vao 1 still has buffer 1 bound?
                gl.bindVertexArray(vaos[0]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, buffers[0]));

                // deleting clears from bound vaos ...
                for (var ndx = 0; ndx < 2; ndx++)
                        gl.deleteBuffer(buffers[ndx]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, null));

                // ... but does not from non-bound vaos?
                gl.bindVertexArray(vaos[1]);
                this.check(glsStateQuery.verify(gl.ELEMENT_ARRAY_BUFFER_BINDING, buffers[1]));

                for (var ndx = 0; ndx < 2; ndx++)
                        gl.deleteVertexArray(vaos[ndx]);
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.StencilClearValueTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.StencilClearValueTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.StencilClearValueTestCase.prototype.constructor = es3fIntegerStateQueryTests.StencilClearValueTestCase;

        es3fIntegerStateQueryTests.StencilClearValueTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.STENCIL_CLEAR_VALUE, 0));

                var stencilBits = /** @type {number} */ (gl.getParameter(gl.STENCIL_BITS));

                for (var stencilBit = 0; stencilBit < stencilBits; ++stencilBit) {
                        /** @type {number} */ var ref = 1 << stencilBit;

                        gl.clearStencil(ref); // mask should not affect the REF

                        this.check(glsStateQuery.verify(gl.STENCIL_CLEAR_VALUE, ref));
                }
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.ActiveTextureTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.ActiveTextureTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ActiveTextureTestCase.prototype.constructor = es3fIntegerStateQueryTests.ActiveTextureTestCase;

        es3fIntegerStateQueryTests.ActiveTextureTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.ACTIVE_TEXTURE, gl.TEXTURE0));

                var textureUnits = /** @type {number} */ (gl.getParameter(gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS));

                for (var ndx = 0; ndx < textureUnits; ++ndx) {
                        gl.activeTexture(gl.TEXTURE0 + ndx);

                        this.check(glsStateQuery.verify(gl.ACTIVE_TEXTURE, gl.TEXTURE0 + ndx));
                }
        };

        es3fIntegerStateQueryTests.ActiveTextureTestCase.prototype.deinit = function() {
                // [dag] reset the state of the context
                 gl.activeTexture(gl.TEXTURE0);
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.RenderbufferBindingTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.RenderbufferBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.RenderbufferBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.RenderbufferBindingTestCase;

        es3fIntegerStateQueryTests.RenderbufferBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.RENDERBUFFER_BINDING, null));

                /** @type {WebGLRenderbuffer} */ var renderBuffer = gl.createRenderbuffer();

                gl.bindRenderbuffer(gl.RENDERBUFFER, renderBuffer);

                this.check(glsStateQuery.verify(gl.RENDERBUFFER_BINDING, renderBuffer));

                gl.deleteRenderbuffer(renderBuffer);
                this.check(glsStateQuery.verify(gl.RENDERBUFFER_BINDING, null));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.SamplerObjectBindingTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.SamplerObjectBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.SamplerObjectBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.SamplerObjectBindingTestCase;

        es3fIntegerStateQueryTests.SamplerObjectBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.SAMPLER_BINDING, null));

                bufferedLogToConsole('SingleUnit: Single unit');
                /** @type {WebGLSampler} */ var sampler = gl.createSampler();

                gl.bindSampler(0, sampler);

                this.check(glsStateQuery.verify(gl.SAMPLER_BINDING, sampler));

                gl.deleteSampler(sampler);
                this.check(glsStateQuery.verify(gl.SAMPLER_BINDING, null));

                bufferedLogToConsole('MultipleUnits: Multiple units');

                /** @type {WebGLSampler} */ var samplerA = gl.createSampler();
                /** @type {WebGLSampler} */ var samplerB = gl.createSampler();

                gl.bindSampler(1, samplerA);
                gl.bindSampler(2, samplerB);

                this.check(glsStateQuery.verify(gl.SAMPLER_BINDING, null));

                gl.activeTexture(gl.TEXTURE1);
                this.check(glsStateQuery.verify(gl.SAMPLER_BINDING, samplerA));

                gl.activeTexture(gl.TEXTURE2);
                this.check(glsStateQuery.verify(gl.SAMPLER_BINDING, samplerB));

                gl.deleteSampler(samplerB);
                gl.deleteSampler(samplerA);
        };

        es3fIntegerStateQueryTests.SamplerObjectBindingTestCase.prototype.deinit = function() {
                gl.activeTexture(gl.TEXTURE0);
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} testBindingName
         * @param {number} textureType
         */
        es3fIntegerStateQueryTests.TextureBindingTestCase = function(name, description, testBindingName, textureType) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
         /** @type {number} */ this.m_testBindingName = testBindingName;
         /** @type {number} */ this.m_textureType = textureType;
        };

        es3fIntegerStateQueryTests.TextureBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.TextureBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.TextureBindingTestCase;

        es3fIntegerStateQueryTests.TextureBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(this.m_testBindingName, null));

                /** @type {WebGLTexture} */ var texture = gl.createTexture();

                gl.bindTexture(this.m_textureType, texture);
                this.check(glsStateQuery.verify(this.m_testBindingName, texture));

                gl.deleteTexture(texture);

                this.check(glsStateQuery.verify(this.m_testBindingName, null));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.FrameBufferBindingTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.FrameBufferBindingTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.FrameBufferBindingTestCase.prototype.constructor = es3fIntegerStateQueryTests.FrameBufferBindingTestCase;

        es3fIntegerStateQueryTests.FrameBufferBindingTestCase.prototype.test = function() {
                this.check(glsStateQuery.verify(gl.DRAW_FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.READ_FRAMEBUFFER_BINDING, null));

                /** @type {WebGLFramebuffer} */ var framebufferId = gl.createFramebuffer();

                gl.bindFramebuffer(gl.FRAMEBUFFER, framebufferId);

                this.check(glsStateQuery.verify(gl.DRAW_FRAMEBUFFER_BINDING,        framebufferId));
                this.check(glsStateQuery.verify(gl.FRAMEBUFFER_BINDING,        framebufferId));
                this.check(glsStateQuery.verify(gl.READ_FRAMEBUFFER_BINDING,        framebufferId));

                gl.bindFramebuffer(gl.FRAMEBUFFER, null);

                this.check(glsStateQuery.verify(gl.DRAW_FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.READ_FRAMEBUFFER_BINDING, null));

                gl.bindFramebuffer(gl.READ_FRAMEBUFFER, framebufferId);

                this.check(glsStateQuery.verify(gl.DRAW_FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.READ_FRAMEBUFFER_BINDING,        framebufferId));

                gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, framebufferId);

                this.check(glsStateQuery.verify(gl.DRAW_FRAMEBUFFER_BINDING,        framebufferId));
                this.check(glsStateQuery.verify(gl.FRAMEBUFFER_BINDING,        framebufferId));
                this.check(glsStateQuery.verify(gl.READ_FRAMEBUFFER_BINDING,        framebufferId));

                gl.deleteFramebuffer(framebufferId);

                this.check(glsStateQuery.verify(gl.DRAW_FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.FRAMEBUFFER_BINDING, null));
                this.check(glsStateQuery.verify(gl.READ_FRAMEBUFFER_BINDING, null));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.ImplementationColorReadTestCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.ImplementationColorReadTestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ImplementationColorReadTestCase.prototype.constructor = es3fIntegerStateQueryTests.ImplementationColorReadTestCase;

        es3fIntegerStateQueryTests.ImplementationColorReadTestCase.prototype.test = function() {
                /** @type {Array<number>} */ var defaultColorTypes = [
                        gl.UNSIGNED_BYTE, gl.BYTE, gl.UNSIGNED_SHORT, gl.SHORT,
                        gl.UNSIGNED_INT, gl.INT, gl.HALF_FLOAT, gl.FLOAT, gl.UNSIGNED_SHORT_5_6_5,
                        gl.UNSIGNED_SHORT_4_4_4_4, gl.UNSIGNED_SHORT_5_5_5_1,
                        gl.UNSIGNED_INT_2_10_10_10_REV, gl.UNSIGNED_INT_10F_11F_11F_REV
                ];

                /** @type {Array<number>} */ var defaultColorFormats = [
                        gl.RGBA, gl.RGBA_INTEGER, gl.RGB, gl.RGB_INTEGER,
                        gl.RG, gl.RG_INTEGER, gl.RED, gl.RED_INTEGER
                ];

                /** @type {Array<number>} */ var validColorTypes = [];
                /** @type {Array<number>} */ var validColorFormats = [];

                // Defined by the spec

                for (var ndx = 0; ndx < defaultColorTypes.length; ++ndx)
                        validColorTypes.push(defaultColorTypes[ndx]);
                for (var ndx = 0; ndx < defaultColorFormats.length; ++ndx)
                        validColorFormats.push(defaultColorFormats[ndx]);

                // Extensions

                // if (this.m_context.getContextInfo().isExtensionSupported("gl.EXT_texture_format_BGRA8888") ||
                //         this.m_context.getContextInfo().isExtensionSupported("gl.APPLE_texture_format_BGRA8888"))
                //         validColorFormats.push(gl.BGRA);
                //
                // if (this.m_context.getContextInfo().isExtensionSupported("gl.EXT_read_format_bgra")) {
                //         validColorFormats.push(gl.BGRA);
                //         validColorTypes.push(gl.UNSIGNED_SHORT_4_4_4_4_REV);
                //         validColorTypes.push(gl.UNSIGNED_SHORT_1_5_5_5_REV);
                // }
                //
                // if (this.m_context.getContextInfo().isExtensionSupported("gl.IMG_read_format")) {
                //         validColorFormats.push(gl.BGRA);
                //         validColorTypes.push(gl.UNSIGNED_SHORT_4_4_4_4_REV);
                // }
                //
                // if (this.m_context.getContextInfo().isExtensionSupported("gl.NV_sRGB_formats")) {
                //         validColorFormats.push(gl.SLUMINANCE_NV);
                //         validColorFormats.push(gl.SLUMINANCE_ALPHA_NV);
                // }
                //
                // if (this.m_context.getContextInfo().isExtensionSupported("gl.NV_bgr")) {
                //         validColorFormats.push(gl.BGR_NV);
                // }

                this.check(glsStateQuery.verifyAnyOf(gl.IMPLEMENTATION_COLOR_READ_TYPE, validColorTypes));
                this.check(glsStateQuery.verifyAnyOf(gl.IMPLEMENTATION_COLOR_READ_FORMAT, validColorFormats));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.ReadBufferCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.ReadBufferCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ReadBufferCase.prototype.constructor = es3fIntegerStateQueryTests.ReadBufferCase;

        es3fIntegerStateQueryTests.ReadBufferCase.prototype.test = function() {
                /** @type {Array<number>} */ var validInitialValues = [gl.BACK, gl.NONE];
                this.check(glsStateQuery.verifyAnyOf(gl.READ_BUFFER, validInitialValues));

                gl.readBuffer(gl.NONE);
                this.check(glsStateQuery.verify(gl.READ_BUFFER, gl.NONE));

                gl.readBuffer(gl.BACK);
                this.check(glsStateQuery.verify(gl.READ_BUFFER, gl.BACK));

                // test gl.READ_BUFFER with framebuffers

                /** @type {WebGLFramebuffer} */ var framebufferId = gl.createFramebuffer();

                /** @type {WebGLRenderbuffer} */ var renderbuffer_id = gl.createRenderbuffer();

                gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer_id);

                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 128, 128);

                gl.bindFramebuffer(gl.READ_FRAMEBUFFER, framebufferId);

                gl.framebufferRenderbuffer(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbuffer_id);

                this.check(glsStateQuery.verify(gl.READ_BUFFER, gl.COLOR_ATTACHMENT0));

                gl.deleteFramebuffer(framebufferId);
                gl.deleteRenderbuffer(renderbuffer_id);

                this.check(glsStateQuery.verify(gl.READ_BUFFER, gl.BACK));
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         */
        es3fIntegerStateQueryTests.DrawBufferCase = function(name, description) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
        };

        es3fIntegerStateQueryTests.DrawBufferCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.DrawBufferCase.prototype.constructor = es3fIntegerStateQueryTests.DrawBufferCase;

        es3fIntegerStateQueryTests.DrawBufferCase.prototype.test = function() {
                /** @type {Array<number>} */ var validInitialValues = [gl.BACK, gl.NONE];
                this.check(glsStateQuery.verifyAnyOf(gl.DRAW_BUFFER0, validInitialValues));

                /** @type {number} */ var bufs = gl.NONE;
                gl.drawBuffers([bufs]);
                this.check(glsStateQuery.verify(gl.DRAW_BUFFER0, gl.NONE));

                bufs = gl.BACK;
                gl.drawBuffers([bufs]);
                this.check(glsStateQuery.verify(gl.DRAW_BUFFER0, gl.BACK));

                // test gl.DRAW_BUFFER with framebuffers

                /** @type {WebGLFramebuffer} */ var framebufferId = gl.createFramebuffer();

                /** @type {Array<WebGLRenderbuffer>} */ var renderbuffer_ids = [];

                for (var ndx = 0; ndx < 2; ndx++)
                        renderbuffer_ids[ndx] = gl.createRenderbuffer();

                gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer_ids[0]);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 128, 128);

                gl.bindRenderbuffer(gl.RENDERBUFFER, renderbuffer_ids[1]);
                gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, 128, 128);

                gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, framebufferId);

                gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, renderbuffer_ids[0]);
                gl.framebufferRenderbuffer(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT1, gl.RENDERBUFFER, renderbuffer_ids[1]);

                // only the initial state the draw buffer for fragment color zero is defined
                this.check(glsStateQuery.verify(gl.DRAW_BUFFER0, gl.COLOR_ATTACHMENT0));

                /** @type {Array<number>} */ var bufTargets = [gl.NONE, gl.COLOR_ATTACHMENT1];
                gl.drawBuffers(bufTargets);
                this.check(glsStateQuery.verify(gl.DRAW_BUFFER0, gl.NONE));
                this.check(glsStateQuery.verify(gl.DRAW_BUFFER1, gl.COLOR_ATTACHMENT1));

                gl.deleteFramebuffer(framebufferId);
                gl.deleteRenderbuffer(renderbuffer_ids[0]);
                gl.deleteRenderbuffer(renderbuffer_ids[1]);

                this.check(glsStateQuery.verify(gl.DRAW_BUFFER0, gl.BACK));
        };

        // Integer64
        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} targetName
         * @param {number} minValue
         */
        es3fIntegerStateQueryTests.ConstantMinimumValue64TestCase = function(name, description, targetName, minValue) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_targetName = targetName;
                /** @type {number} */ this.m_minValue = minValue;
        };

        es3fIntegerStateQueryTests.ConstantMinimumValue64TestCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.ConstantMinimumValue64TestCase.prototype.constructor = es3fIntegerStateQueryTests.ConstantMinimumValue64TestCase;

        es3fIntegerStateQueryTests.ConstantMinimumValue64TestCase.prototype.test = function() {
                this.check(glsStateQuery.verifyGreaterOrEqual(this.m_targetName, this.m_minValue), 'Fail');
        };

        /**
         * @constructor
         * @extends {es3fApiCase.ApiCase}
         * @param {string} name
         * @param {string} description
         * @param {number} targetName
         * @param {number} targetMaxUniformBlocksName
         * @param {number} targetMaxUniformComponentsName
         */
        es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase = function(name, description, targetName, targetMaxUniformBlocksName, targetMaxUniformComponentsName) {
                es3fApiCase.ApiCase.call(this, name, description, gl);
                /** @type {number} */ this.m_targetName = targetName;
                /** @type {number} */ this.m_targetMaxUniformBlocksName = targetMaxUniformBlocksName;
                /** @type {number} */ this.m_targetMaxUniformComponentsName = targetMaxUniformComponentsName;
        };

        es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
        es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase.prototype.constructor = es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase;

        es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase.prototype.test = function() {
                var uniformBlockSize = /** @type {number} */ (gl.getParameter(gl.MAX_UNIFORM_BLOCK_SIZE));
                var maxUniformBlocks = /** @type {number} */ (gl.getParameter(this.m_targetMaxUniformBlocksName));
                var maxUniformComponents = /** @type {number} */ (gl.getParameter(this.m_targetMaxUniformComponentsName));

                // MAX_stage_UNIFORM_BLOCKS * MAX_UNIFORM_BLOCK_SIZE / 4 + MAX_stage_UNIFORM_COMPONENTS
                /** @type {number} */ var minCombinedUniformComponents = maxUniformBlocks * uniformBlockSize / 4 + maxUniformComponents;

                this.check(glsStateQuery.verifyGreaterOrEqual(this.m_targetName, minCombinedUniformComponents));
        };

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fIntegerStateQueryTests.IntegerStateQueryTests = function() {
        tcuTestCase.DeqpTest.call(this, 'integers', 'Integer Values');
    };

    es3fIntegerStateQueryTests.IntegerStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fIntegerStateQueryTests.IntegerStateQueryTests.prototype.constructor = es3fIntegerStateQueryTests.IntegerStateQueryTests;

    es3fIntegerStateQueryTests.IntegerStateQueryTests.prototype.init = function() {
                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} targetName
                 * @param {number} value
                 */
                var LimitedStateInteger = function(name, description, targetName, value) {
                        /** @type {string} */ this.name = name;
                        /** @type {string} */ this.description = description;
                        /** @type {number} */ this.targetName = targetName;
                        /** @type {number} */ this.value = value;
                };

                /** @type {Array<LimitedStateInteger>} */ var implementationMinLimits = [
                        new LimitedStateInteger('subpixel_bits', 'SUBPIXEL_BITS has minimum value of 4', gl.SUBPIXEL_BITS, 4),
                        new LimitedStateInteger('max_3d_texture_size', 'MAX_3D_TEXTURE_SIZE has minimum value of 256', gl.MAX_3D_TEXTURE_SIZE, 256),
                        new LimitedStateInteger('max_texture_size', 'MAX_TEXTURE_SIZE has minimum value of 2048', gl.MAX_TEXTURE_SIZE, 2048),
                        new LimitedStateInteger('max_array_texture_layers', 'MAX_ARRAY_TEXTURE_LAYERS has minimum value of 256', gl.MAX_ARRAY_TEXTURE_LAYERS, 256),
                        new LimitedStateInteger('max_cube_map_texture_size', 'MAX_CUBE_MAP_TEXTURE_SIZE has minimum value of 2048', gl.MAX_CUBE_MAP_TEXTURE_SIZE, 2048),
                        new LimitedStateInteger('max_renderbuffer_size', 'MAX_RENDERBUFFER_SIZE has minimum value of 2048', gl.MAX_RENDERBUFFER_SIZE, 2048),
                        new LimitedStateInteger('max_draw_buffers', 'MAX_DRAW_BUFFERS has minimum value of 4', gl.MAX_DRAW_BUFFERS, 4),
                        new LimitedStateInteger('max_color_attachments', 'MAX_COLOR_ATTACHMENTS has minimum value of 4', gl.MAX_COLOR_ATTACHMENTS, 4),
                        new LimitedStateInteger('max_elements_indices', 'MAX_ELEMENTS_INDICES has minimum value of 0', gl.MAX_ELEMENTS_INDICES, 0),
                        new LimitedStateInteger('max_elements_vertices', 'MAX_ELEMENTS_VERTICES has minimum value of 0', gl.MAX_ELEMENTS_VERTICES, 0),
                        new LimitedStateInteger('max_vertex_attribs', 'MAX_VERTEX_ATTRIBS has minimum value of 16', gl.MAX_VERTEX_ATTRIBS, 16),
                        new LimitedStateInteger('max_vertex_uniform_components', 'MAX_VERTEX_UNIFORM_COMPONENTS has minimum value of 1024', gl.MAX_VERTEX_UNIFORM_COMPONENTS, 1024),
                        new LimitedStateInteger('max_vertex_uniform_vectors', 'MAX_VERTEX_UNIFORM_VECTORS has minimum value of 256', gl.MAX_VERTEX_UNIFORM_VECTORS, 256),
                        new LimitedStateInteger('max_vertex_uniform_blocks', 'MAX_VERTEX_UNIFORM_BLOCKS has minimum value of 12', gl.MAX_VERTEX_UNIFORM_BLOCKS, 12),
                        new LimitedStateInteger('max_vertex_output_components', 'MAX_VERTEX_OUTPUT_COMPONENTS has minimum value of 64', gl.MAX_VERTEX_OUTPUT_COMPONENTS, 64),
                        new LimitedStateInteger('max_vertex_texture_image_units', 'MAX_VERTEX_TEXTURE_IMAGE_UNITS has minimum value of 16', gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS, 16),
                        new LimitedStateInteger('max_fragment_uniform_components', 'MAX_FRAGMENT_UNIFORM_COMPONENTS has minimum value of 896', gl.MAX_FRAGMENT_UNIFORM_COMPONENTS, 896),
                        new LimitedStateInteger('max_fragment_uniform_vectors', 'MAX_FRAGMENT_UNIFORM_VECTORS has minimum value of 224', gl.MAX_FRAGMENT_UNIFORM_VECTORS, 224),
                        new LimitedStateInteger('max_fragment_uniform_blocks', 'MAX_FRAGMENT_UNIFORM_BLOCKS has minimum value of 12', gl.MAX_FRAGMENT_UNIFORM_BLOCKS, 12),
                        new LimitedStateInteger('max_fragment_input_components', 'MAX_FRAGMENT_INPUT_COMPONENTS has minimum value of 60', gl.MAX_FRAGMENT_INPUT_COMPONENTS, 60),
                        new LimitedStateInteger('max_texture_image_units', 'MAX_TEXTURE_IMAGE_UNITS has minimum value of 16', gl.MAX_TEXTURE_IMAGE_UNITS, 16),
                        new LimitedStateInteger('max_program_texel_offset', 'MAX_PROGRAM_TEXEL_OFFSET has minimum value of 7', gl.MAX_PROGRAM_TEXEL_OFFSET, 7),
                        new LimitedStateInteger('max_uniform_buffer_bindings', 'MAX_UNIFORM_BUFFER_BINDINGS has minimum value of 24', gl.MAX_UNIFORM_BUFFER_BINDINGS, 24),
                        new LimitedStateInteger('max_combined_uniform_blocks', 'MAX_COMBINED_UNIFORM_BLOCKS has minimum value of 24', gl.MAX_COMBINED_UNIFORM_BLOCKS, 24),
                        new LimitedStateInteger('max_varying_components', 'MAX_VARYING_COMPONENTS has minimum value of 60', gl.MAX_VARYING_COMPONENTS, 60),
                        new LimitedStateInteger('max_varying_vectors', 'MAX_VARYING_VECTORS has minimum value of 15', gl.MAX_VARYING_VECTORS, 15),
                        new LimitedStateInteger('max_combined_texture_image_units', 'MAX_COMBINED_TEXTURE_IMAGE_UNITS has minimum value of 32', gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS, 32),
                        new LimitedStateInteger('max_transform_feedback_interleaved_components', 'MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS has minimum value of 64', gl.MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS, 64),
                        new LimitedStateInteger('max_transform_feedback_separate_attribs', 'MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS has minimum value of 4', gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS, 4),
                        new LimitedStateInteger('max_transform_feedback_separate_components', 'MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS has minimum value of 4', gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS, 4),
                        new LimitedStateInteger('max_samples', 'MAX_SAMPLES has minimum value of 4', gl.MAX_SAMPLES, 4),
                        new LimitedStateInteger('red_bits', 'RED_BITS has minimum value of 0', gl.RED_BITS, 0),
                        new LimitedStateInteger('green_bits', 'GREEN_BITS has minimum value of 0', gl.GREEN_BITS, 0),
                        new LimitedStateInteger('blue_bits', 'BLUE_BITS has minimum value of 0', gl.BLUE_BITS, 0),
                        new LimitedStateInteger('alpha_bits', 'ALPHA_BITS has minimum value of 0', gl.ALPHA_BITS, 0),
                        new LimitedStateInteger('depth_bits', 'DEPTH_BITS has minimum value of 0', gl.DEPTH_BITS, 0),
                        new LimitedStateInteger('stencil_bits', 'STENCIL_BITS has minimum value of 0', gl.STENCIL_BITS, 0)
                ];

                /** @type {Array<LimitedStateInteger>} */ var implementationMaxLimits = [
                        new LimitedStateInteger('min_program_texel_offset', 'MIN_PROGRAM_TEXEL_OFFSET has maximum value of -8', gl.MIN_PROGRAM_TEXEL_OFFSET, -8),
                        new LimitedStateInteger('uniform_buffer_offset_alignment', 'UNIFORM_BUFFER_OFFSET_ALIGNMENT has minimum value of 1', gl.UNIFORM_BUFFER_OFFSET_ALIGNMENT, 256)
                ];

                var testCtx = this;

                for (var testNdx = 0; testNdx < implementationMinLimits.length; testNdx++)
                        testCtx.addChild(new es3fIntegerStateQueryTests.ConstantMinimumValueTestCase(implementationMinLimits[testNdx].name, implementationMinLimits[testNdx].description, implementationMinLimits[testNdx].targetName, implementationMinLimits[testNdx].value));

                for (var testNdx = 0; testNdx < implementationMaxLimits.length; testNdx++)
                        testCtx.addChild(new es3fIntegerStateQueryTests.ConstantMaximumValueTestCase(implementationMaxLimits[testNdx].name, implementationMaxLimits[testNdx].description, implementationMaxLimits[testNdx].targetName, implementationMaxLimits[testNdx].value));

                testCtx.addChild(new es3fIntegerStateQueryTests.SampleBuffersTestCase('sample_buffers', 'SAMPLE_BUFFERS'));
                testCtx.addChild(new es3fIntegerStateQueryTests.SamplesTestCase('samples' , 'SAMPLES'));
                testCtx.addChild(new es3fIntegerStateQueryTests.HintTestCase('generate_mipmap_hint', 'GENERATE_MIPMAP_HINT', gl.GENERATE_MIPMAP_HINT));
                testCtx.addChild(new es3fIntegerStateQueryTests.HintTestCase('fragment_shader_derivative_hint', 'FRAGMENT_SHADER_DERIVATIVE_HINT', gl.FRAGMENT_SHADER_DERIVATIVE_HINT));
                testCtx.addChild(new es3fIntegerStateQueryTests.DepthFuncTestCase('depth_func', 'DEPTH_FUNC'));
                testCtx.addChild(new es3fIntegerStateQueryTests.CullFaceTestCase('cull_face_mode', 'CULL_FACE_MODE'));
                testCtx.addChild(new es3fIntegerStateQueryTests.FrontFaceTestCase('front_face_mode', 'FRONT_FACE'));
                testCtx.addChild(new es3fIntegerStateQueryTests.ViewPortTestCase('viewport', 'VIEWPORT'));
                testCtx.addChild(new es3fIntegerStateQueryTests.ScissorBoxTestCase('scissor_box', 'SCISSOR_BOX'));
                testCtx.addChild(new es3fIntegerStateQueryTests.MaxViewportDimsTestCase('max_viewport_dims', 'MAX_VIEWPORT_DIMS'));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilRefTestCase('stencil_ref', 'STENCIL_REF', gl.STENCIL_REF));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilRefTestCase('stencil_back_ref', 'STENCIL_BACK_REF', gl.STENCIL_BACK_REF));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilRefSeparateTestCase('stencil_ref_separate', 'STENCIL_REF (separate)', gl.STENCIL_REF, gl.FRONT));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilRefSeparateTestCase('stencil_ref_separate_both', 'STENCIL_REF (separate)', gl.STENCIL_REF, gl.FRONT_AND_BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilRefSeparateTestCase('stencil_back_ref_separate', 'STENCIL_BACK_REF (separate)', gl.STENCIL_BACK_REF, gl.BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilRefSeparateTestCase('stencil_back_ref_separate_both', 'STENCIL_BACK_REF (separate)', gl.STENCIL_BACK_REF, gl.FRONT_AND_BACK));

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} frontDescription
                 * @param {number} frontTarget
                 * @param {string} backDescription
                 * @param {number} backTarget
                 */
                 var NamedStencilOp = function(name, frontDescription, frontTarget, backDescription, backTarget) {
                        /** @type {string} */ this.name = name;
                    /** @type {string} */ this.frontDescription = frontDescription;
                    /** @type {number} */ this.frontTarget = frontTarget;
                    /** @type {string} */ this.backDescription = backDescription;
                    /** @type {number} */ this.backTarget = backTarget;
                };

                /** @type {Array<NamedStencilOp>} */ var stencilOps = [
                        new NamedStencilOp('fail', 'STENCIL_FAIL', gl.STENCIL_FAIL, 'STENCIL_BACK_FAIL', gl.STENCIL_BACK_FAIL),
                        new NamedStencilOp('depth_fail', 'STENCIL_PASS_DEPTH_FAIL', gl.STENCIL_PASS_DEPTH_FAIL, 'STENCIL_BACK_PASS_DEPTH_FAIL', gl.STENCIL_BACK_PASS_DEPTH_FAIL),
                        new NamedStencilOp('depth_pass', 'STENCIL_PASS_DEPTH_PASS', gl.STENCIL_PASS_DEPTH_PASS, 'STENCIL_BACK_PASS_DEPTH_PASS', gl.STENCIL_BACK_PASS_DEPTH_PASS)
                ];

                for (var testNdx = 0; testNdx < stencilOps.length; testNdx++) {
                        testCtx.addChild(new es3fIntegerStateQueryTests.StencilOpTestCase('stencil_' + stencilOps[testNdx].name, stencilOps[testNdx].frontDescription, stencilOps[testNdx].frontTarget));
                        testCtx.addChild(new es3fIntegerStateQueryTests.StencilOpTestCase('stencil_back_' + stencilOps[testNdx].name, stencilOps[testNdx].backDescription, stencilOps[testNdx].backTarget));

                        testCtx.addChild(new es3fIntegerStateQueryTests.StencilOpSeparateTestCase('stencil_' + stencilOps[testNdx].name + '_separate_both', stencilOps[testNdx].frontDescription, stencilOps[testNdx].frontTarget, gl.FRONT_AND_BACK));
                        testCtx.addChild(new es3fIntegerStateQueryTests.StencilOpSeparateTestCase('stencil_back_' + stencilOps[testNdx].name + '_separate_both', stencilOps[testNdx].backDescription, stencilOps[testNdx].backTarget, gl.FRONT_AND_BACK));

                        testCtx.addChild(new es3fIntegerStateQueryTests.StencilOpSeparateTestCase('stencil_' + stencilOps[testNdx].name + '_separate', stencilOps[testNdx].frontDescription, stencilOps[testNdx].frontTarget, gl.FRONT));
                        testCtx.addChild(new es3fIntegerStateQueryTests.StencilOpSeparateTestCase('stencil_back_' + stencilOps[testNdx].name + '_separate', stencilOps[testNdx].backDescription, stencilOps[testNdx].backTarget, gl.BACK));
                }

                testCtx.addChild(new es3fIntegerStateQueryTests.StencilFuncTestCase('stencil_func', 'STENCIL_FUNC'));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilFuncSeparateTestCase('stencil_func_separate', 'STENCIL_FUNC (separate)', gl.STENCIL_FUNC, gl.FRONT));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilFuncSeparateTestCase('stencil_func_separate_both', 'STENCIL_FUNC (separate)', gl.STENCIL_FUNC, gl.FRONT_AND_BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilFuncSeparateTestCase('stencil_back_func_separate', 'STENCIL_FUNC (separate)', gl.STENCIL_BACK_FUNC, gl.BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilFuncSeparateTestCase('stencil_back_func_separate_both', 'STENCIL_FUNC (separate)', gl.STENCIL_BACK_FUNC, gl.FRONT_AND_BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilMaskTestCase('stencil_value_mask', 'STENCIL_VALUE_MASK', gl.STENCIL_VALUE_MASK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilMaskTestCase('stencil_back_value_mask', 'STENCIL_BACK_VALUE_MASK', gl.STENCIL_BACK_VALUE_MASK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilMaskSeparateTestCase('stencil_value_mask_separate', 'STENCIL_VALUE_MASK (separate)', gl.STENCIL_VALUE_MASK, gl.FRONT));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilMaskSeparateTestCase('stencil_value_mask_separate_both', 'STENCIL_VALUE_MASK (separate)', gl.STENCIL_VALUE_MASK, gl.FRONT_AND_BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilMaskSeparateTestCase('stencil_back_value_mask_separate', 'STENCIL_BACK_VALUE_MASK (separate)', gl.STENCIL_BACK_VALUE_MASK, gl.BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilMaskSeparateTestCase('stencil_back_value_mask_separate_both', 'STENCIL_BACK_VALUE_MASK (separate)', gl.STENCIL_BACK_VALUE_MASK, gl.FRONT_AND_BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilWriteMaskTestCase('stencil_writemask', 'STENCIL_WRITEMASK', gl.STENCIL_WRITEMASK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilWriteMaskTestCase('stencil_back_writemask', 'STENCIL_BACK_WRITEMASK', gl.STENCIL_BACK_WRITEMASK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase('stencil_writemask_separate', 'STENCIL_WRITEMASK (separate)', gl.STENCIL_WRITEMASK, gl.FRONT));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase('stencil_writemask_separate_both', 'STENCIL_WRITEMASK (separate)', gl.STENCIL_WRITEMASK, gl.FRONT_AND_BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase('stencil_back_writemask_separate', 'STENCIL_BACK_WRITEMASK (separate)', gl.STENCIL_BACK_WRITEMASK, gl.BACK));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilWriteMaskSeparateTestCase('stencil_back_writemask_separate_both', 'STENCIL_BACK_WRITEMASK (separate)', gl.STENCIL_BACK_WRITEMASK, gl.FRONT_AND_BACK));

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} target
                 * @param {number} initialValue
                 */
                var PixelStoreState = function(name, description, target, initialValue) {
                    /** @type {string} */ this.name = name;
                    /** @type {string} */ this.description = description;
                    /** @type {number} */ this.target = target;
                    /** @type {number} */ this.initialValue = initialValue;
                };

                /** @type {Array<PixelStoreState>} */ var pixelStoreStates = [
                        new PixelStoreState('unpack_image_height', 'UNPACK_IMAGE_HEIGHT', gl.UNPACK_IMAGE_HEIGHT, 0),
                        new PixelStoreState('unpack_skip_images', 'UNPACK_SKIP_IMAGES', gl.UNPACK_SKIP_IMAGES, 0),
                        new PixelStoreState('unpack_row_length', 'UNPACK_ROW_LENGTH', gl.UNPACK_ROW_LENGTH, 0),
                        new PixelStoreState('unpack_skip_rows', 'UNPACK_SKIP_ROWS', gl.UNPACK_SKIP_ROWS, 0),
                        new PixelStoreState('unpack_skip_pixels', 'UNPACK_SKIP_PIXELS', gl.UNPACK_SKIP_PIXELS, 0),
                        new PixelStoreState('pack_row_length', 'PACK_ROW_LENGTH', gl.PACK_ROW_LENGTH, 0),
                        new PixelStoreState('pack_skip_rows', 'PACK_SKIP_ROWS', gl.PACK_SKIP_ROWS, 0),
                        new PixelStoreState('pack_skip_pixels', 'PACK_SKIP_PIXELS', gl.PACK_SKIP_PIXELS, 0)
                ];

                for (var testNdx = 0; testNdx < pixelStoreStates.length; testNdx++)
                        testCtx.addChild(new es3fIntegerStateQueryTests.PixelStoreTestCase(pixelStoreStates[testNdx].name, pixelStoreStates[testNdx].description, pixelStoreStates[testNdx].target, pixelStoreStates[testNdx].initialValue));

                testCtx.addChild(new es3fIntegerStateQueryTests.PixelStoreAlignTestCase('unpack_alignment', 'UNPACK_ALIGNMENT', gl.UNPACK_ALIGNMENT));
                testCtx.addChild(new es3fIntegerStateQueryTests.PixelStoreAlignTestCase('pack_alignment', 'PACK_ALIGNMENT', gl.PACK_ALIGNMENT));

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} target
                 * @param {number} initialValue
                 */
                var BlendColorState = function(name, description, target, initialValue) {
                    /** @type {string} */ this.name = name;
                    /** @type {string} */ this.description = description;
                    /** @type {number} */ this.target = target;
                    /** @type {number} */ this.initialValue = initialValue;
                };

                /** @type {Array<PixelStoreState>} */ var blendColorStates = [
                        new BlendColorState('blend_src_rgb', 'BLEND_SRC_RGB', gl.BLEND_SRC_RGB),
                        new BlendColorState('blend_src_alpha', 'BLEND_SRC_ALPHA', gl.BLEND_SRC_ALPHA),
                        new BlendColorState('blend_dst_rgb', 'BLEND_DST_RGB', gl.BLEND_DST_RGB),
                        new BlendColorState('blend_dst_alpha', 'BLEND_DST_ALPHA', gl.BLEND_DST_ALPHA)
                ];

                for (var testNdx = 0; testNdx < blendColorStates.length; testNdx++) {
                        testCtx.addChild(new es3fIntegerStateQueryTests.BlendFuncTestCase(blendColorStates[testNdx].name, blendColorStates[testNdx].description, blendColorStates[testNdx].target));
                        testCtx.addChild(new es3fIntegerStateQueryTests.BlendFuncSeparateTestCase(blendColorStates[testNdx].name + '_separate', blendColorStates[testNdx].description, blendColorStates[testNdx].target));
                }

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} target
                 * @param {number} initialValue
                 */
                var BlendEquationState = function(name, description, target, initialValue) {
                    /** @type {string} */ this.name = name;
                    /** @type {string} */ this.description = description;
                    /** @type {number} */ this.target = target;
                    /** @type {number} */ this.initialValue = initialValue;
                };

                /** @type {Array<PixelStoreState>} */ var blendEquationStates = [
                        new BlendEquationState('blend_equation_rgb', 'BLEND_EQUATION_RGB', gl.BLEND_EQUATION_RGB, gl.FUNC_ADD),
                        new BlendEquationState('blend_equation_alpha', 'BLEND_EQUATION_ALPHA', gl.BLEND_EQUATION_ALPHA, gl.FUNC_ADD)
                ];

                for (var testNdx = 0; testNdx < blendEquationStates.length; testNdx++) {
                        testCtx.addChild(new es3fIntegerStateQueryTests.BlendEquationTestCase(blendEquationStates[testNdx].name, blendEquationStates[testNdx].description, blendEquationStates[testNdx].target, blendEquationStates[testNdx].initialValue));
                        testCtx.addChild(new es3fIntegerStateQueryTests.BlendEquationSeparateTestCase(blendEquationStates[testNdx].name + '_separate', blendEquationStates[testNdx].description, blendEquationStates[testNdx].target, blendEquationStates[testNdx].initialValue));
                }

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} target
                 * @param {number} minValue
                 */
                var ImplementationArrayReturningState = function(name, description, target, minValue) {
                    /** @type {string} */ this.name = name;
                    /** @type {string} */ this.description = description;
                    /** @type {number} */ this.target = target;
                    /** @type {number} */ this.minValue = minValue;
                };

                /** @type {ImplementationArrayReturningState} */ var implementationArrayReturningStates = new ImplementationArrayReturningState('compressed_texture_formats', 'COMPRESSED_TEXTURE_FORMATS', gl.COMPRESSED_TEXTURE_FORMATS, 10);

                testCtx.addChild(new es3fIntegerStateQueryTests.ImplementationArrayTestCase(implementationArrayReturningStates.name, implementationArrayReturningStates.description, implementationArrayReturningStates.target, implementationArrayReturningStates.minValue));

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} target
                 * @param {number} type
                 */
                var BufferBindingState = function(name, description, target, type) {
                        /** @type {string} */ this.name = name;
                        /** @type {string} */ this.description = description;
                        /** @type {number} */ this.target = target;
                        /** @type {number} */ this.type = type;
                };

                /** @type {Array<BufferBindingState>} */ var bufferBindingStates = [
                        new BufferBindingState('array_buffer_binding', 'ARRAY_BUFFER_BINDING', gl.ARRAY_BUFFER_BINDING, gl.ARRAY_BUFFER),
                        new BufferBindingState('uniform_buffer_binding', 'UNIFORM_BUFFER_BINDING', gl.UNIFORM_BUFFER_BINDING, gl.UNIFORM_BUFFER),
                        new BufferBindingState('pixel_pack_buffer_binding', 'PIXEL_PACK_BUFFER_BINDING', gl.PIXEL_PACK_BUFFER_BINDING, gl.PIXEL_PACK_BUFFER),
                        new BufferBindingState('pixel_unpack_buffer_binding', 'PIXEL_UNPACK_BUFFER_BINDING', gl.PIXEL_UNPACK_BUFFER_BINDING, gl.PIXEL_UNPACK_BUFFER),
                        new BufferBindingState('transform_feedback_buffer_binding', 'TRANSFORM_FEEDBACK_BUFFER_BINDING', gl.TRANSFORM_FEEDBACK_BUFFER_BINDING, gl.TRANSFORM_FEEDBACK_BUFFER),
                        new BufferBindingState('copy_read_buffer_binding', 'COPY_READ_BUFFER_BINDING', gl.COPY_READ_BUFFER_BINDING, gl.COPY_READ_BUFFER),
                        new BufferBindingState('copy_write_buffer_binding', 'COPY_WRITE_BUFFER_BINDING', gl.COPY_WRITE_BUFFER_BINDING, gl.COPY_WRITE_BUFFER)
                ];

                for (var testNdx = 0; testNdx < bufferBindingStates.length; testNdx++)
                        testCtx.addChild(new es3fIntegerStateQueryTests.BufferBindingTestCase(bufferBindingStates[testNdx].name, bufferBindingStates[testNdx].description, bufferBindingStates[testNdx].target, bufferBindingStates[testNdx].type));

                testCtx.addChild(new es3fIntegerStateQueryTests.ElementArrayBufferBindingTestCase('element_array_buffer_binding'));
                testCtx.addChild(new es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase('transform_feedback_binding'));
                testCtx.addChild(new es3fIntegerStateQueryTests.TransformFeedbackBindingTestCase('transform_feedback_binding'));
                testCtx.addChild(new es3fIntegerStateQueryTests.CurrentProgramBindingTestCase('current_program_binding', 'CURRENT_PROGRAM'));
                testCtx.addChild(new es3fIntegerStateQueryTests.VertexArrayBindingTestCase('vertex_array_binding', 'VERTEX_ARRAY_BINDING'));
                testCtx.addChild(new es3fIntegerStateQueryTests.StencilClearValueTestCase('stencil_clear_value', 'STENCIL_CLEAR_VALUE'));
                testCtx.addChild(new es3fIntegerStateQueryTests.ActiveTextureTestCase('active_texture', 'ACTIVE_TEXTURE'));
                testCtx.addChild(new es3fIntegerStateQueryTests.RenderbufferBindingTestCase('renderbuffer_binding', 'RENDERBUFFER_BINDING'));
                testCtx.addChild(new es3fIntegerStateQueryTests.SamplerObjectBindingTestCase('sampler_binding', 'SAMPLER_BINDING'));

                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} target
                 * @param {number} type
                 */
                var TextureBinding = function(name, description, target, type) {
                        /** @type {string} */ this.name = name;
                        /** @type {string} */ this.description = description;
                        /** @type {number} */ this.target = target;
                        /** @type {number} */ this.type = type;
                };

                /** @type {Array<TextureBinding>} */ var textureBindings = [
                        new TextureBinding('texture_binding_2d', 'TEXTURE_BINDING_2D', gl.TEXTURE_BINDING_2D, gl.TEXTURE_2D),
                        new TextureBinding('texture_binding_3d', 'TEXTURE_BINDING_3D', gl.TEXTURE_BINDING_3D, gl.TEXTURE_3D),
                        new TextureBinding('texture_binding_2d_array', 'TEXTURE_BINDING_2D_ARRAY', gl.TEXTURE_BINDING_2D_ARRAY, gl.TEXTURE_2D_ARRAY),
                        new TextureBinding('texture_binding_cube_map', 'TEXTURE_BINDING_CUBE_MAP', gl.TEXTURE_BINDING_CUBE_MAP, gl.TEXTURE_CUBE_MAP)
                ];

                for (var testNdx = 0; testNdx < textureBindings.length; testNdx++)
                        testCtx.addChild(new es3fIntegerStateQueryTests.TextureBindingTestCase(textureBindings[testNdx].name, textureBindings[testNdx].description, textureBindings[testNdx].target, textureBindings[testNdx].type));

                testCtx.addChild(new es3fIntegerStateQueryTests.FrameBufferBindingTestCase('framebuffer_binding', 'DRAW_FRAMEBUFFER_BINDING and READ_FRAMEBUFFER_BINDING'));
                testCtx.addChild(new es3fIntegerStateQueryTests.ImplementationColorReadTestCase('implementation_color_read', 'IMPLEMENTATION_COLOR_READ_TYPE and IMPLEMENTATION_COLOR_READ_FORMAT'));
                testCtx.addChild(new es3fIntegerStateQueryTests.ReadBufferCase('read_buffer', 'READ_BUFFER'));
                testCtx.addChild(new es3fIntegerStateQueryTests.DrawBufferCase('draw_buffer', 'DRAW_BUFFER'));


                // Integer64
                /**
                 * @struct
                 * @constructor
                 * @param {string} name
                 * @param {string} description
                 * @param {number} targetName
                 * @param {number} minValue
                 */
                var LimitedStateInteger64 = function(name, description, targetName, minValue) {
                        /** @type {string} */ this.name = name;
                        /** @type {string} */ this.description = description;
                        /** @type {number} */ this.targetName = targetName;
                        /** @type {number} */ this.minValue = minValue;

                };

                /** @type {Array<LimitedStateInteger64>} */ var implementationLimits = [
                        new LimitedStateInteger64('max_element_index', 'MAX_ELEMENT_INDEX', gl.MAX_ELEMENT_INDEX, 0x00FFFFFF),
                        new LimitedStateInteger64('max_server_wait_timeout', 'MAX_SERVER_WAIT_TIMEOUT', gl.MAX_SERVER_WAIT_TIMEOUT, 0),
                        new LimitedStateInteger64('max_uniform_block_size', 'MAX_UNIFORM_BLOCK_SIZE', gl.MAX_UNIFORM_BLOCK_SIZE, 16384)
                ];

                for (var testNdx = 0; testNdx < implementationLimits.length; testNdx++)
                        this.addChild(new es3fIntegerStateQueryTests.ConstantMinimumValue64TestCase(implementationLimits[testNdx].name, implementationLimits[testNdx].description, implementationLimits[testNdx].targetName, implementationLimits[testNdx].minValue));

                this.addChild(new es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase('max_combined_vertex_uniform_components', 'MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS', gl.MAX_COMBINED_VERTEX_UNIFORM_COMPONENTS, gl.MAX_VERTEX_UNIFORM_BLOCKS, gl.MAX_VERTEX_UNIFORM_COMPONENTS));
                this.addChild(new es3fIntegerStateQueryTests.MaxCombinedStageUniformComponentsCase('max_combined_fragment_uniform_components', 'MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS', gl.MAX_COMBINED_FRAGMENT_UNIFORM_COMPONENTS, gl.MAX_FRAGMENT_UNIFORM_BLOCKS, gl.MAX_FRAGMENT_UNIFORM_COMPONENTS));

        };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fIntegerStateQueryTests.run = function(context) {
            gl = context;
            //Set up Test Root parameters
            var state = tcuTestCase.runner;
            state.setRoot(new es3fIntegerStateQueryTests.IntegerStateQueryTests());

            //Set up name and description of this test series.
            setCurrentTestName(state.testCases.fullName());
            description(state.testCases.getDescription());

            try {
                    //Run test cases
                    tcuTestCase.runTestCases();
            }
            catch (err) {
                    testFailedOptions('Failed to es3fIntegerStateQueryTests.run tests', false);
                    tcuTestCase.runner.terminate();
            }
    };

});
