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
goog.provide('functional.gles3.es3fFragDepthTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluDrawUtil');
goog.require('modules.shared.glsShaderRenderCase');

goog.scope(function() {
	var es3fFragDepthTests = functional.gles3.es3fFragDepthTests;
	var deMath = framework.delibs.debase.deMath;
	var deRandom = framework.delibs.debase.deRandom;
	var deString = framework.delibs.debase.deString;
	var glsShaderRenderCase = modules.shared.glsShaderRenderCase;
	var gluShaderProgram = framework.opengl.gluShaderProgram;
	var gluDrawUtil = framework.opengl.gluDrawUtil;
	var tcuImageCompare = framework.common.tcuImageCompare;
	var tcuRGBA = framework.common.tcuRGBA;
	var tcuSurface = framework.common.tcuSurface;
	var tcuTestCase = framework.common.tcuTestCase;
	/** @typedef {function(Array<number>):number} */ es3fFragDepthTests.EvalFragDepthFunc;

	/** @const {string} */ es3fFragDepthTests.s_vertexShaderSrc = '' +
		'#version 300 es\n' +
		'in highp vec4 a_position;\n' +
		'in highp vec2 a_coord;\n' +
		'out highp vec2 v_coord;\n' +
		'void main (void)\n' +
		'{\n' +
		'	gl_Position = a_position;\n' +
		'	v_coord = a_coord;\n' +
		'}\n';

	/** @const {string} */ es3fFragDepthTests.s_defaultFragmentShaderSrc = '' +
		'#version 300 es\n' +
		'uniform highp vec4 u_color;\n' +
		'layout(location = 0) out mediump vec4 o_color;\n' +
		'void main (void)\n' +
		'{\n' +
		'	o_color = u_color;\n' +
		'}\n';

	/**
	 * @param {number} func
	 * @param {*} a
	 * @param {*} b
	 * @return {boolean}
	 */
	es3fFragDepthTests.compare = function(func, a, b)	{
		switch (func) {
			case gl.NEVER: return false;
			case gl.ALWAYS: return true;
			case gl.LESS: return a < b;
			case gl.LEQUAL: return a <= b;
			case gl.EQUAL: return a === b;
			case gl.NOTEQUAL: return a !== b;
			case gl.GEQUAL: return a >= b;
			case gl.GREATER: return a > b;
		}
		bufferedLogToConsole('Compare function not supported.');
		return false;
	};

	/**
	 * @constructor
	 * @extends {tcuTestCase.DeqpTest}
	 * @param {string} name
	 * @param {string} desc
	 * @param {string} fragSrc
	 * @param {?es3fFragDepthTests.EvalFragDepthFunc} evalFunc
	 * @param {number} compareFunc
	 */
	es3fFragDepthTests.FragDepthCompareCase = function(name, desc, fragSrc, evalFunc, compareFunc) {
		tcuTestCase.DeqpTest.call(this, name, desc);
		/** @type {string} */ this.m_fragSrc = fragSrc;
		/** @type {?es3fFragDepthTests.EvalFragDepthFunc} */ this.m_evalFunc = evalFunc;
		/** @type {number} */ this.m_compareFunc = compareFunc;
	};

	es3fFragDepthTests.FragDepthCompareCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
	es3fFragDepthTests.FragDepthCompareCase.prototype.constructor = es3fFragDepthTests.FragDepthCompareCase;

	/**
	 * @return {tcuTestCase.IterateResult}
	 */
	es3fFragDepthTests.FragDepthCompareCase.prototype.iterate = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));
		/** @type {number} */ var viewportW = Math.min(128, gl.drawingBufferWidth);
		/** @type {number} */ var viewportH = Math.min(128, gl.drawingBufferHeight);
		/** @type {number} */ var viewportX = rnd.getInt(0, gl.drawingBufferWidth - viewportW);
		/** @type {number} */ var viewportY = rnd.getInt(0, gl.drawingBufferHeight - viewportH);
		/** @type {tcuSurface.Surface} */ var renderedFrame = new tcuSurface.Surface(viewportW, viewportH);
		/** @type {tcuSurface.Surface} */ var referenceFrame = new tcuSurface.Surface(viewportW, viewportH);
		/** @type {number} */ var constDepth = 0.1;
		var depthBits = /** @type {number} */ (gl.getParameter(gl.DEPTH_BITS));

		/** @type {number} */ var xf;
		/** @type {number} */ var d;
		/** @type {boolean} */ var dpass;

		if (depthBits == 0)
			throw new Error('Depth buffer is required');

		gl.depthMask(true);
		gl.viewport(viewportX, viewportY, viewportW, viewportH);
		gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
		gl.enable(gl.DEPTH_TEST);

		/** @type {Array<number>} */ var quadIndices = [0, 1, 2, 2, 1, 3];

		// Fill viewport with 2 quads - one with constant depth and another with d = [-1..1]
		/** @type {gluShaderProgram.ShaderProgram} */
		var basicQuadProgram = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(es3fFragDepthTests.s_vertexShaderSrc, es3fFragDepthTests.s_defaultFragmentShaderSrc));

		if (!basicQuadProgram.isOk()) {
			bufferedLogToConsole(basicQuadProgram.getProgramInfo().infoLog);
			throw new Error('Compile failed');
		}

		/** @type {Array<number>} */ var constDepthCoord = [
			 -1.0, -1.0, constDepth, 1.0,
			 -1.0, 1.0, constDepth, 1.0,
			 0.0, -1.0, constDepth, 1.0,
			 0.0, 1.0, constDepth, 1.0
		];

		/** @type {Array<number>} */ var varyingDepthCoord = [
			 0.0, -1.0, 1.0, 1.0,
			 0.0, 1.0, 0.0, 1.0,
			 1.0, -1.0, 0.0, 1.0,
			 1.0, 1.0, -1.0, 1.0
		];

		gl.useProgram(basicQuadProgram.getProgram());
		gl.uniform4f(gl.getUniformLocation(basicQuadProgram.getProgram(), 'u_color'), 0.0, 0.0, 1.0, 1.0);
		gl.depthFunc(gl.ALWAYS);

		/** @type {gluDrawUtil.VertexArrayBinding} */ var posBinding = gluDrawUtil.newFloatVertexArrayBinding('a_position', 4, 4, 0, constDepthCoord);
		gluDrawUtil.draw(gl, basicQuadProgram.getProgram(), [posBinding], gluDrawUtil.triangles(quadIndices));

		posBinding = gluDrawUtil.newFloatVertexArrayBinding('a_position', 4, 4, 0, varyingDepthCoord);
		gluDrawUtil.draw(gl, basicQuadProgram.getProgram(), [posBinding], gluDrawUtil.triangles(quadIndices));

		// Render with depth test.
		/** @type {gluShaderProgram.ShaderProgram} */
		var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(es3fFragDepthTests.s_vertexShaderSrc, this.m_fragSrc));
		bufferedLogToConsole(program.getProgramInfo().infoLog);

		if (!program.isOk())
			throw new Error('Compile failed');

		/** @type {Array<number>} */ var coord = [
			0.0, 0.0,
			0.0, 1.0,
			1.0, 0.0,
			1.0, 1.0
		];

		/** @type {Array<number>} */ var position = [
			-1.0, -1.0, 1.0, 1.0,
			-1.0, 1.0, 0.0, 1.0,
			1.0, -1.0, 0.0, 1.0,
			1.0, 1.0, -1.0, 1.0
		];

		gl.useProgram(program.getProgram());
		gl.depthFunc(this.m_compareFunc);
		gl.uniform4f(gl.getUniformLocation(program.getProgram(), 'u_color'), 0.0, 1.0, 0.0, 1.0);

		// Setup default helper uniforms.
		glsShaderRenderCase.setupDefaultUniforms(program.getProgram());

		/** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var  vertexArrays = [
			gluDrawUtil.newFloatVertexArrayBinding('a_position', 4, 4, 0, position),
			gluDrawUtil.newFloatVertexArrayBinding('a_coord', 2, 4, 0, coord)
		];

		gluDrawUtil.draw(gl, program.getProgram(), vertexArrays, gluDrawUtil.triangles(quadIndices));

		renderedFrame.readViewport(gl, [viewportX, viewportY, viewportW, viewportH]);

		// Render reference.
		for (var y = 0; y < referenceFrame.getHeight(); y++) {
			/** @type {number} */ var yf = (y + 0.5) / referenceFrame.getHeight();
			/** @type {number} */ var half = deMath.clamp(Math.floor(referenceFrame.getWidth() * 0.5 + 0.5), 0, referenceFrame.getWidth());

			// Fill left half - comparison to constant 0.5
			for (var x = 0; x < half; x++) {
				xf = (x + 0.5) / referenceFrame.getWidth();
				d = this.m_evalFunc([xf, yf]);
				dpass = es3fFragDepthTests.compare(this.m_compareFunc, d, constDepth * 0.5 + 0.5);

				referenceFrame.setPixel(x, y, dpass ? tcuRGBA.RGBA.green.toIVec() : tcuRGBA.RGBA.blue.toIVec());
			}

			// Fill right half - comparison to interpolated depth
			for (var x = half; x < referenceFrame.getWidth(); x++) {
				xf = (x + 0.5) / referenceFrame.getWidth();
				/** @type {number} */ var xh = (x - half + 0.5) / (referenceFrame.getWidth() - half);
				/** @type {number} */ var rd = 1.0 - (xh + yf) * 0.5;
				d = this.m_evalFunc([xf, yf]);
				dpass = es3fFragDepthTests.compare(this.m_compareFunc, d, rd);

				referenceFrame.setPixel(x, y, dpass ? tcuRGBA.RGBA.green.toIVec() : tcuRGBA.RGBA.blue.toIVec());
			}
		}

		/** @type {boolean} */ var isOk = tcuImageCompare.fuzzyCompare('Result', 'Image comparison result', referenceFrame.getAccess(), renderedFrame.getAccess(), 0.05);

		if (!isOk)
			testFailedOptions('Fail', false);
		else
			testPassedOptions('Pass', true);

		return tcuTestCase.IterateResult.STOP;
	};

	/**
	 * @constructor
	 * @extends {tcuTestCase.DeqpTest}
	 * @param {string} name
	 * @param {string} desc
	 * @param {string} fragSrc
	 * @param {es3fFragDepthTests.EvalFragDepthFunc} evalFunc
	 */
	es3fFragDepthTests.FragDepthWriteCase = function(name, desc, fragSrc, evalFunc) {
		tcuTestCase.DeqpTest.call(this, name, desc);
		/** @type {string} */ this.m_fragSrc = fragSrc;
		/** @type {es3fFragDepthTests.EvalFragDepthFunc} */ this.m_evalFunc = evalFunc;
	};

	es3fFragDepthTests.FragDepthWriteCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
	es3fFragDepthTests.FragDepthWriteCase.prototype.constructor = es3fFragDepthTests.FragDepthWriteCase;

	/**
	 * @return {tcuTestCase.IterateResult}
	 */
	es3fFragDepthTests.FragDepthWriteCase.prototype.iterate = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));
		/** @type {number} */ var viewportW = Math.min(128, gl.drawingBufferWidth);
		/** @type {number} */ var viewportH = Math.min(128, gl.drawingBufferHeight);
		/** @type {number} */ var viewportX = rnd.getInt(0, gl.drawingBufferWidth - viewportW);
		/** @type {number} */ var viewportY = rnd.getInt(0, gl.drawingBufferHeight - viewportH);
		/** @type {tcuSurface.Surface} */ var renderedFrame = new tcuSurface.Surface(viewportW, viewportH);
		/** @type {tcuSurface.Surface} */ var referenceFrame = new tcuSurface.Surface(viewportW, viewportH);
		/** @type {number} */ var numDepthSteps = 16;
		/** @type {number} */ var depthStep = 1.0 / (numDepthSteps - 1);
		var depthBits = /** @type {number} */ (gl.getParameter(gl.DEPTH_BITS));

		if (depthBits === 0)
			throw new Error('Depth buffer is required');

		gl.depthMask(true);
		gl.viewport(viewportX, viewportY, viewportW, viewportH);
		gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
		gl.enable(gl.DEPTH_TEST);
		gl.depthFunc(gl.LESS);

		/** @type {Array<number>} */ var quadIndices = [0, 1, 2, 2, 1, 3];

		// Render with given shader.
		/** @type {gluShaderProgram.ShaderProgram} */
		var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(es3fFragDepthTests.s_vertexShaderSrc, this.m_fragSrc));
		bufferedLogToConsole(program.getProgramInfo().infoLog);

		if (!program.isOk())
			throw new Error('Compile failed');

		/** @type {Array<number>} */ var coord = [
			0.0, 0.0,
			0.0, 1.0,
			1.0, 0.0,
			1.0, 1.0
		];

		/** @type {Array<number>} */ var position = [
			-1.0, -1.0, +1.0, 1.0,
			-1.0, 1.0, 0.0, 1.0,
			1.0, -1.0, 0.0, 1.0,
			1.0, 1.0, -1.0, 1.0
		];

		gl.useProgram(program.getProgram());
		gl.uniform4f(gl.getUniformLocation(program.getProgram(), 'u_color'), 0.0, 1.0, 0.0, 1.0);

		// Setup default helper uniforms.
		glsShaderRenderCase.setupDefaultUniforms(program.getProgram());

		/** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var  vertexArrays = [
			gluDrawUtil.newFloatVertexArrayBinding('a_position', 4, 4, 0, position),
			gluDrawUtil.newFloatVertexArrayBinding('a_coord', 2, 4, 0, coord)
		];
		gluDrawUtil.draw(gl, program.getProgram(), vertexArrays, gluDrawUtil.triangles(quadIndices));

		// Visualize by rendering full-screen quads with increasing depth and color.
		program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(es3fFragDepthTests.s_vertexShaderSrc, es3fFragDepthTests.s_defaultFragmentShaderSrc));

		if (!program.isOk()) {
			bufferedLogToConsole(program.getProgramInfo().infoLog);
			throw new Error('Compile failed');
		}

		/** @type {WebGLUniformLocation} */ var colorLoc = gl.getUniformLocation(program.getProgram(), 'u_color');

		gl.useProgram(program.getProgram());
		gl.depthMask(false);

		for (var stepNdx = 0; stepNdx < numDepthSteps; stepNdx++) {
			/** @type {number} */ var f = stepNdx * depthStep;
			/** @type {number} */ var depth = f * 2.0 - 1.0;
			/** @type {Array<number>} */ var color = [f, f, f, 1.0];

			position = [
				-1.0, -1.0, depth, 1.0,
				-1.0, 1.0, depth, 1.0,
				1.0, -1.0, depth, 1.0,
				1.0, 1.0, depth, 1.0
			];

			/** @type {gluDrawUtil.VertexArrayBinding} */
			var posBinding = gluDrawUtil.newFloatVertexArrayBinding('a_position', 4, 4, 0, position);

			gl.uniform4fv(colorLoc, color);
			gluDrawUtil.draw(gl, program.getProgram(), [posBinding], gluDrawUtil.triangles(quadIndices));
		}

		renderedFrame.readViewport(gl, [viewportX, viewportY, viewportW, viewportH]);

		// Render reference.
		for (var y = 0; y < referenceFrame.getHeight(); y++)
		for (var x = 0; x < referenceFrame.getWidth(); x++) {
			/** @type {number} */ var xf = (x + 0.5) / referenceFrame.getWidth();
			/** @type {number} */ var yf = (y + 0.5) / referenceFrame.getHeight();
			/** @type {number} */ var d = this.m_evalFunc([xf, yf]);
			/** @type {number} */ var step = Math.floor(d / depthStep);
			/** @type {number} */ var col = deMath.clamp(Math.floor(step * depthStep * 255.0), 0, 255);

			referenceFrame.setPixel(x, y, [col, col, col, 0xff]);
		}

		/** @type {boolean} */ var isOk = tcuImageCompare.fuzzyCompare('Result', 'Image comparison result', referenceFrame.getAccess(), renderedFrame.getAccess(), 0.05);

		if (!isOk)
			testFailedOptions('Fail', false);
		else
			testPassedOptions('Pass', true);

		return tcuTestCase.IterateResult.STOP;
	};


	/**
	 * @constructor
	 * @extends {tcuTestCase.DeqpTest}
	 */
	es3fFragDepthTests.FragDepthTests = function() {
		tcuTestCase.DeqpTest.call(this, 'fragdepth', 'gl_FragDepth tests');
	};

	es3fFragDepthTests.FragDepthTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
	es3fFragDepthTests.FragDepthTests.prototype.constructor = es3fFragDepthTests.FragDepthTests;

	/**
	 * @param {Array<number>} coord
	 * @return {number}
	 */
	es3fFragDepthTests.evalConstDepth = function(coord) {
		return 0.5;
	};

	/**
	 * @param {Array<number>} coord
	 * @return {number}
	 */
	es3fFragDepthTests.evalDynamicDepth = function(coord) {
		return (coord[0] + coord[1]) * 0.5;
	};

	/**
	 * @param {Array<number>} coord
	 * @return {number}
	 */
	es3fFragDepthTests.evalNoWrite = function(coord) {
		return 1.0 - (coord[0] + coord[1]) * 0.5;
	};

	/**
	 * @param {Array<number>} coord
	 * @return {number}
	 */
	es3fFragDepthTests.evalDynamicConditionalDepth = function(coord) {
		/** @type {number} */ var d = (coord[0] + coord[1]) * 0.5;
		if (coord[1] < 0.5)
			return d;
		else
			return 1.0 - d;
	};

	es3fFragDepthTests.FragDepthTests.prototype.init = function() {
		/**
		 * @struct
		 * @constructor
		 * @param {string} name
		 * @param {string} desc
		 * @param {es3fFragDepthTests.EvalFragDepthFunc} evalFunc
		 * @param {string} fragSrc
		 */
		var Case = function(name, desc, evalFunc, fragSrc) {
			/** @type {string} */ this.name = name;
			/** @type {string} */ this.desc = desc;
			/** @type {es3fFragDepthTests.EvalFragDepthFunc} */ this.evalFunc = evalFunc;
			/** @type {string} */ this.fragSrc = fragSrc;
		};

		/** @type {Array<Case>} */ var cases = [
			new Case('no_write', 'No gl_FragDepth write', es3fFragDepthTests.evalNoWrite,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'}\n'
			),
			new Case('const', 'Const depth write', es3fFragDepthTests.evalConstDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	gl_FragDepth = 0.5;\n' +
				'}\n'
			),
			new Case('uniform', 'Uniform depth write', es3fFragDepthTests.evalConstDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'uniform highp float uf_half;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	gl_FragDepth = uf_half;\n' +
				'}\n'
			),
			new Case('dynamic', 'Dynamic depth write', es3fFragDepthTests.evalDynamicDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'in highp vec2 v_coord;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	gl_FragDepth = (v_coord.x+v_coord.y)*0.5;\n' +
				'}\n'
			),
			new Case('fragcoord_z', 'gl_FragDepth write from gl_FragCoord.z', es3fFragDepthTests.evalNoWrite,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	gl_FragDepth = gl_FragCoord.z;\n' +
				'}\n'
			),
			new Case('uniform_conditional_write', 'Uniform conditional write', es3fFragDepthTests.evalDynamicDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'uniform bool ub_true;\n' +
				'in highp vec2 v_coord;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	if (ub_true)\n' +
				'		gl_FragDepth = (v_coord.x+v_coord.y)*0.5;\n' +
				'}\n'
			),
			new Case('dynamic_conditional_write', 'Uniform conditional write', es3fFragDepthTests.evalDynamicConditionalDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'uniform bool ub_true;\n' +
				'in highp vec2 v_coord;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	mediump float d = (v_coord.x+v_coord.y)*0.5f;\n' +
				'	if (v_coord.y < 0.5)\n' +
				'		gl_FragDepth = d;\n' +
				'	else\n' +
				'		gl_FragDepth = 1.0 - d;\n' +
				'}\n'
			),
			new Case('uniform_loop_write', 'Uniform loop write', es3fFragDepthTests.evalConstDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'uniform int ui_two;\n' +
				'uniform highp float uf_fourth;\n' +
				'in highp vec2 v_coord;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	gl_FragDepth = 0.0;\n' +
				'	for (int i = 0; i < ui_two; i++)\n' +
				'		gl_FragDepth += uf_fourth;\n' +
				'}\n'
			),
			new Case('write_in_function', 'Uniform loop write', es3fFragDepthTests.evalDynamicDepth,
				'#version 300 es\n' +
				'uniform highp vec4 u_color;\n' +
				'uniform highp float uf_half;\n' +
				'in highp vec2 v_coord;\n' +
				'layout(location = 0) out mediump vec4 o_color;\n' +
				'void myfunc (highp vec2 coord)\n' +
				'{\n' +
				'	gl_FragDepth = (coord.x+coord.y)*0.5;\n' +
				'}\n' +
				'void main (void)\n' +
				'{\n' +
				'	o_color = u_color;\n' +
				'	myfunc(v_coord);\n' +
				'}\n'
			)
		];

		var testGroup = tcuTestCase.runner.testCases;

		// .write
		/** @type {tcuTestCase.DeqpTest} */ var writeGroup = tcuTestCase.newTest('write', 'gl_FragDepth write tests');
		testGroup.addChild(writeGroup);
		for (var ndx = 0; ndx < cases.length; ndx++)
			writeGroup.addChild(new es3fFragDepthTests.FragDepthWriteCase(cases[ndx].name, cases[ndx].desc, cases[ndx].fragSrc, cases[ndx].evalFunc));

		// .compare
		/** @type {tcuTestCase.DeqpTest} */ var compareGroup = tcuTestCase.newTest('compare', 'gl_FragDepth used with depth comparison');
		testGroup.addChild(compareGroup);
		for (var ndx = 0; ndx < cases.length; ndx++)
			compareGroup.addChild(new es3fFragDepthTests.FragDepthCompareCase(cases[ndx].name, cases[ndx].desc, cases[ndx].fragSrc, cases[ndx].evalFunc, gl.LESS));
	};

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fFragDepthTests.run = function(context) {
    	gl = context;
    	//Set up Test Root parameters
    	var state = tcuTestCase.runner;
    	state.setRoot(new es3fFragDepthTests.FragDepthTests());

    	//Set up name and description of this test series.
    	setCurrentTestName(state.testCases.fullName());
    	description(state.testCases.getDescription());

    	try {
    		//Run test cases
    		tcuTestCase.runTestCases();
    	}
    	catch (err) {
    		testFailedOptions('Failed to es3fFragDepthTests.run tests', false);
    		tcuTestCase.runner.terminate();
    	}
    };

});
