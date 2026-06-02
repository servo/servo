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
goog.provide('functional.gles3.es3fIndexedStateQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {
	var es3fIndexedStateQueryTests = functional.gles3.es3fIndexedStateQueryTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var es3fApiCase = functional.gles3.es3fApiCase;

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fIndexedStateQueryTests.TransformFeedbackCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fIndexedStateQueryTests.TransformFeedbackCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fIndexedStateQueryTests.TransformFeedbackCase.prototype.constructor = es3fIndexedStateQueryTests.TransformFeedbackCase;

	es3fIndexedStateQueryTests.TransformFeedbackCase.prototype.testTransformFeedback = function() {
		throw new Error('This method should be overriden.');
	};

	es3fIndexedStateQueryTests.TransformFeedbackCase.prototype.test = function() {
		/** @type {string} */ var transformFeedbackTestVertSource = '' +
			'#version 300 es\n' +
			'out highp vec4 anotherOutput;\n' +
			'void main (void)\n' +
			'{\n' +
			'	gl_Position = vec4(0.0);\n' +
			'	anotherOutput = vec4(0.0);\n' +
			'}\n';
		/** @type {string} */ var transformFeedbackTestFragSource = '' +
			'#version 300 es\n' +
			'layout(location = 0) out mediump vec4 fragColor;' +
			'void main (void)\n' +
			'{\n' +
			'	fragColor = vec4(0.0);\n' +
			'}\n';

		/** @type {WebGLShader} */ var shaderVert = gl.createShader(gl.VERTEX_SHADER);
		/** @type {WebGLShader} */ var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

		gl.shaderSource(shaderVert, transformFeedbackTestVertSource);
		gl.shaderSource(shaderFrag, transformFeedbackTestFragSource);

		gl.compileShader(shaderVert);
		gl.compileShader(shaderFrag);

		/** @type {WebGLProgram} */ var shaderProg = gl.createProgram();
		gl.attachShader(shaderProg, shaderVert);
		gl.attachShader(shaderProg, shaderFrag);

		/** @type {Array<string>} */ var transformFeedbackOutputs = ['gl_Position', 'anotherOutput'];

		gl.transformFeedbackVaryings(shaderProg, transformFeedbackOutputs, gl.INTERLEAVED_ATTRIBS);
		gl.linkProgram(shaderProg);

		/** @type {WebGLTransformFeedback} */ var transformFeedbackId = gl.createTransformFeedback();
		gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, transformFeedbackId);

		this.testTransformFeedback();

		// cleanup

		gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);

		gl.deleteTransformFeedback(transformFeedbackId);
		gl.deleteShader(shaderVert);
		gl.deleteShader(shaderFrag);
		gl.deleteProgram(shaderProg);
	};

	/**
	 * @constructor
	 * @extends {es3fIndexedStateQueryTests.TransformFeedbackCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fIndexedStateQueryTests.TransformFeedbackBufferBindingCase = function(name, description) {
		es3fIndexedStateQueryTests.TransformFeedbackCase.call(this, name, description);
	};

	es3fIndexedStateQueryTests.TransformFeedbackBufferBindingCase.prototype = Object.create(es3fIndexedStateQueryTests.TransformFeedbackCase.prototype);
	es3fIndexedStateQueryTests.TransformFeedbackBufferBindingCase.prototype.constructor = es3fIndexedStateQueryTests.TransformFeedbackBufferBindingCase;

	es3fIndexedStateQueryTests.TransformFeedbackBufferBindingCase.prototype.testTransformFeedback = function() {
		/** @type {number} */ var feedbackPositionIndex = 0;
		/** @type {number} */ var feedbackOutputIndex = 1;
		/** @type {Array<number>} */ var feedbackIndex = [feedbackPositionIndex, feedbackOutputIndex];

		// bind buffers

		/** @type {Array<WebGLBuffer>} */ var feedbackBuffers = [];
		for (var ndx = 0; ndx < 2; ndx++)
			feedbackBuffers[ndx] = gl.createBuffer();

		for (var ndx = 0; ndx < feedbackBuffers.length; ndx++) {
			gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackBuffers[ndx]);
			gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, new Float32Array(16), gl.DYNAMIC_READ);
			gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackIndex[ndx], feedbackBuffers[ndx]);
		}

		// test TRANSFORM_FEEDBACK_BUFFER_BINDING
		for (var ndx = 0; ndx < feedbackBuffers.length; ndx++) {
			var boundBuffer = /** @type {WebGLBuffer} */ (gl.getIndexedParameter(gl.TRANSFORM_FEEDBACK_BUFFER_BINDING, ndx));
			this.check(boundBuffer === feedbackBuffers[ndx], 'buffers do not match');
		}

		// cleanup
		for (var ndx = 0; ndx < feedbackBuffers.length; ndx++)
			gl.deleteBuffer(feedbackBuffers[ndx]);
	};

	/**
	 * @constructor
	 * @extends {es3fIndexedStateQueryTests.TransformFeedbackCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fIndexedStateQueryTests.TransformFeedbackBufferBufferCase = function(name, description) {
		es3fIndexedStateQueryTests.TransformFeedbackCase.call(this, name, description);
	};

	es3fIndexedStateQueryTests.TransformFeedbackBufferBufferCase.prototype = Object.create(es3fIndexedStateQueryTests.TransformFeedbackCase.prototype);
	es3fIndexedStateQueryTests.TransformFeedbackBufferBufferCase.prototype.constructor = es3fIndexedStateQueryTests.TransformFeedbackBufferBufferCase;

	es3fIndexedStateQueryTests.TransformFeedbackBufferBufferCase.prototype.testTransformFeedback = function() {
		/** @type {number} */ var feedbackPositionIndex = 0;
		/** @type {number} */ var feedbackOutputIndex = 1;

		/** @type {number} */ var rangeBufferOffset = 4;
		/** @type {number} */ var rangeBufferSize = 8;

		// bind buffers

		/** @type {Array<WebGLBuffer>} */ var feedbackBuffers = [];
		for (var ndx = 0; ndx < 2; ndx++)
			feedbackBuffers[ndx] = gl.createBuffer();

		gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackBuffers[0]);
		gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, new Float32Array(16), gl.DYNAMIC_READ);
		gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackPositionIndex, feedbackBuffers[0]);

		gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackBuffers[1]);
		gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, new Float32Array(16), gl.DYNAMIC_READ);
		gl.bindBufferRange(gl.TRANSFORM_FEEDBACK_BUFFER, feedbackOutputIndex, feedbackBuffers[1], rangeBufferOffset, rangeBufferSize);

		/** @type {Array<{index: number, pname: number, value: number}>} */ var requirements = [
			{index: feedbackPositionIndex, pname: gl.TRANSFORM_FEEDBACK_BUFFER_START, value: 0},
			{index: feedbackPositionIndex, pname: gl.TRANSFORM_FEEDBACK_BUFFER_SIZE, value: 0},
			{index: feedbackOutputIndex, pname: gl.TRANSFORM_FEEDBACK_BUFFER_START, value: rangeBufferOffset},
			{index: feedbackOutputIndex, pname: gl.TRANSFORM_FEEDBACK_BUFFER_SIZE, value: rangeBufferSize}
		];

		for (var ndx = 0; ndx < requirements.length; ndx++) {
			var state = /** @type {number} */ (gl.getIndexedParameter(requirements[ndx].pname, requirements[ndx].index));
			this.check(state === requirements[ndx].value, 'got ' + state + '; expected ' + requirements[ndx].value);
		}

		// cleanup
		for (var ndx = 0; ndx < feedbackBuffers.length; ndx++)
			gl.deleteBuffer(feedbackBuffers[ndx]);
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fIndexedStateQueryTests.UniformBufferCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
		/** @type {?WebGLProgram} */ this.m_program = null;
	};

	es3fIndexedStateQueryTests.UniformBufferCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fIndexedStateQueryTests.UniformBufferCase.prototype.constructor = es3fIndexedStateQueryTests.UniformBufferCase;

	es3fIndexedStateQueryTests.UniformBufferCase.prototype.testUniformBuffers = function() {
		throw new Error('This method should be overriden.');
	};

	es3fIndexedStateQueryTests.UniformBufferCase.prototype.test = function() {

		/** @type {string} */ var testVertSource = '' +
			'#version 300 es\n' +
			'uniform highp vec4 input1;\n' +
			'uniform highp vec4 input2;\n' +
			'void main (void)\n' +
			'{\n' +
			'	gl_Position = input1 + input2;\n' +
			'}\n';
		/** @type {string} */ var testFragSource = '' +
			'#version 300 es\n' +
			'layout(location = 0) out mediump vec4 fragColor;' +
			'void main (void)\n' +
			'{\n' +
			'	fragColor = vec4(0.0);\n' +
			'}\n';

		/** @type {WebGLShader} */ var shaderVert = gl.createShader(gl.VERTEX_SHADER);
		/** @type {WebGLShader} */ var shaderFrag = gl.createShader(gl.FRAGMENT_SHADER);

		gl.shaderSource(shaderVert, testVertSource);
		gl.shaderSource(shaderFrag, testFragSource);

		gl.compileShader(shaderVert);
		gl.compileShader(shaderFrag);

		this.m_program = gl.createProgram();
		gl.attachShader(this.m_program, shaderVert);
		gl.attachShader(this.m_program, shaderFrag);
		gl.linkProgram(this.m_program);
		gl.useProgram(this.m_program);

		this.testUniformBuffers();

		gl.useProgram(null);
		gl.deleteShader(shaderVert);
		gl.deleteShader(shaderFrag);
		gl.deleteProgram(this.m_program);
	};

	/**
	* @constructor
	* @extends {es3fIndexedStateQueryTests.UniformBufferCase}
	* @param {string} name
	* @param {string} description
	*/
	es3fIndexedStateQueryTests.UniformBufferBindingCase = function(name, description) {
		es3fIndexedStateQueryTests.UniformBufferCase.call(this, name, description);
		/** @type {?WebGLProgram} */ this.m_program = null;
	};

	es3fIndexedStateQueryTests.UniformBufferBindingCase.prototype = Object.create(es3fIndexedStateQueryTests.UniformBufferCase.prototype);
	es3fIndexedStateQueryTests.UniformBufferBindingCase.prototype.constructor = es3fIndexedStateQueryTests.UniformBufferBindingCase;

	es3fIndexedStateQueryTests.UniformBufferBindingCase.prototype.testUniformBuffers = function() {
		/** @type {Array<string>} */ var uniformNames = ['input1', 'input2'];

		/** @type {Array<number>} */ var uniformIndices = gl.getUniformIndices(this.m_program, uniformNames);

		/** @type {Array<WebGLBuffer>} */ var buffers = [];
		for (var ndx = 0; ndx < 2; ndx++)
			buffers[ndx] = gl.createBuffer();

		for (var ndx = 0; ndx < buffers.length; ++ndx) {
			gl.bindBuffer(gl.UNIFORM_BUFFER, buffers[ndx]);
			gl.bufferData(gl.UNIFORM_BUFFER, new Float32Array(32), gl.DYNAMIC_DRAW);
			gl.bindBufferBase(gl.UNIFORM_BUFFER, uniformIndices[ndx], buffers[ndx]);
		}

		/** @type {Array<WebGLBuffer>} */ var boundBuffer = [];
		for (var ndx = 0; ndx < buffers.length; ndx++) {
			boundBuffer[ndx] = /** @type {WebGLBuffer} */ (gl.getIndexedParameter(gl.UNIFORM_BUFFER_BINDING, uniformIndices[ndx]));
			this.check(boundBuffer[ndx] === buffers[ndx], 'buffers do not match');
		}

		for (var ndx = 0; ndx < buffers.length; ndx++)
			gl.deleteBuffer(buffers[ndx]);
	};

	/**
	* @constructor
	* @extends {es3fIndexedStateQueryTests.UniformBufferCase}
	* @param {string} name
	* @param {string} description
	*/
	es3fIndexedStateQueryTests.UniformBufferBufferCase = function(name, description) {
		es3fIndexedStateQueryTests.UniformBufferCase.call(this, name, description);
		/** @type {?WebGLProgram} */ this.m_program = null;
	};

	es3fIndexedStateQueryTests.UniformBufferBufferCase.prototype = Object.create(es3fIndexedStateQueryTests.UniformBufferCase.prototype);
	es3fIndexedStateQueryTests.UniformBufferBufferCase.prototype.constructor = es3fIndexedStateQueryTests.UniformBufferBufferCase;

	es3fIndexedStateQueryTests.UniformBufferBufferCase.prototype.testUniformBuffers = function() {
		/** @type {Array<string>} */ var uniformNames = ['input1', 'input2'];

		/** @type {Array<number>} */ var uniformIndices = gl.getUniformIndices(this.m_program, uniformNames);

		/** @type {number} */ var alignment = this.getAlignment();
		if (alignment === -1) // cannot continue without this
			return;

		bufferedLogToConsole('Alignment is ' + alignment);

		/** @type {number} */ var rangeBufferOffset = alignment;
		/** @type {number} */ var rangeBufferSize = alignment * 2;
		/** @type {number} */ var rangeBufferTotalSize = rangeBufferOffset + rangeBufferSize + 8; // + 8 has no special meaning, just to make it != with the size of the range

		/** @type {Array<WebGLBuffer>} */ var buffers = [];
		for (var ndx = 0; ndx < 2; ndx++)
			buffers[ndx] = gl.createBuffer();

		gl.bindBuffer(gl.UNIFORM_BUFFER, buffers[0]);
		gl.bufferData(gl.UNIFORM_BUFFER, new Float32Array(32), gl.DYNAMIC_DRAW);
		gl.bindBufferBase(gl.UNIFORM_BUFFER, uniformIndices[0], buffers[0]);

		gl.bindBuffer(gl.UNIFORM_BUFFER, buffers[1]);
		gl.bufferData(gl.UNIFORM_BUFFER, new Float32Array(32), gl.DYNAMIC_DRAW);
		gl.bindBufferRange(gl.UNIFORM_BUFFER, uniformIndices[1], buffers[1], rangeBufferOffset, rangeBufferSize);

		// test UNIFORM_BUFFER_START and UNIFORM_BUFFER_SIZE

		/** @type {Array<{index: number, pname: number, value: number}>} */ var requirements = [
			{index: uniformIndices[0], pname: gl.UNIFORM_BUFFER_START, value: 0},
			{index: uniformIndices[0], pname: gl.UNIFORM_BUFFER_SIZE, value: 0},
			{index: uniformIndices[1], pname: gl.UNIFORM_BUFFER_START, value: rangeBufferOffset},
			{index: uniformIndices[1], pname: gl.UNIFORM_BUFFER_SIZE, value: rangeBufferSize}
		];

		for (var ndx = 0; ndx < requirements.length; ndx++) {
			var state = /** @type {number} */ (gl.getIndexedParameter(requirements[ndx].pname, requirements[ndx].index));

			this.check(state === requirements[ndx].value, 'got ' + state + '; expected ' + requirements[ndx].value);
		}

		for (var ndx = 0; ndx < buffers.length; ndx++)
			gl.deleteBuffer(buffers[ndx]);

	};

	/**
	 * @return {number}
	 */
	es3fIndexedStateQueryTests.UniformBufferBufferCase.prototype.getAlignment = function() {
		var state = /** @type {number} */ (gl.getParameter(gl.UNIFORM_BUFFER_OFFSET_ALIGNMENT));

		if (state <= 256)
			return state;

		bufferedLogToConsole('ERROR: UNIFORM_BUFFER_OFFSET_ALIGNMENT has a maximum value of 256.');
		testFailedOptions('invalid UNIFORM_BUFFER_OFFSET_ALIGNMENT value', false);

		return -1;
	};

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fIndexedStateQueryTests.IndexedStateQueryTests = function() {
        tcuTestCase.DeqpTest.call(this, 'indexed', 'Indexed Integer Values');
    };

    es3fIndexedStateQueryTests.IndexedStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fIndexedStateQueryTests.IndexedStateQueryTests.prototype.constructor = es3fIndexedStateQueryTests.IndexedStateQueryTests;

    es3fIndexedStateQueryTests.IndexedStateQueryTests.prototype.init = function() {
		// transform feedback
		this.addChild(new es3fIndexedStateQueryTests.TransformFeedbackBufferBindingCase('transform_feedback_buffer_binding', 'TRANSFORM_FEEDBACK_BUFFER_BINDING'));
		this.addChild(new es3fIndexedStateQueryTests.TransformFeedbackBufferBufferCase('transform_feedback_buffer_start_size', 'TRANSFORM_FEEDBACK_BUFFER_START and TRANSFORM_FEEDBACK_BUFFER_SIZE'));

		// uniform buffers
		this.addChild(new es3fIndexedStateQueryTests.UniformBufferBindingCase('uniform_buffer_binding', 'UNIFORM_BUFFER_BINDING'));
		this.addChild(new es3fIndexedStateQueryTests.UniformBufferBufferCase('uniform_buffer_start_size', 'UNIFORM_BUFFER_START and UNIFORM_BUFFER_SIZE'));
    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fIndexedStateQueryTests.run = function(context) {
    	gl = context;
    	//Set up Test Root parameters
    	var state = tcuTestCase.runner;
    	state.setRoot(new es3fIndexedStateQueryTests.IndexedStateQueryTests());

    	//Set up name and description of this test series.
    	setCurrentTestName(state.testCases.fullName());
    	description(state.testCases.getDescription());

    	try {
    		//Run test cases
    		tcuTestCase.runTestCases();
    	}
    	catch (err) {
    		testFailedOptions('Failed to es3fIndexedStateQueryTests.run tests', false);
    		tcuTestCase.runner.terminate();
    	}
    };

});
