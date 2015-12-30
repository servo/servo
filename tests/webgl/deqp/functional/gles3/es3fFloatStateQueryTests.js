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
goog.provide('functional.gles3.es3fFloatStateQueryTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
	var es3fFloatStateQueryTests = functional.gles3.es3fFloatStateQueryTests;
    var tcuTestCase = framework.common.tcuTestCase;
	var deRandom = framework.delibs.debase.deRandom;
	var deMath = framework.delibs.debase.deMath;
	var es3fApiCase = functional.gles3.es3fApiCase;
	var glsStateQuery = modules.shared.glsStateQuery;

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.DepthRangeCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.DepthRangeCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.DepthRangeCase.prototype.constructor = es3fFloatStateQueryTests.DepthRangeCase;

	es3fFloatStateQueryTests.DepthRangeCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.DEPTH_RANGE, new Float32Array([0.0, 1.0])));

		/** @type {Array<Float32Array>} */ var fixedTests = [
			new Float32Array([0.5, 1.0]),
			new Float32Array([0.0, 0.5]),
			new Float32Array([0.0, 0.0]),
			new Float32Array([1.0, 1.0])
		];

		for (var ndx = 0; ndx < fixedTests.length; ++ndx) {
			gl.depthRange(fixedTests[ndx][0], fixedTests[ndx][1]);
			this.check(glsStateQuery.verify(gl.DEPTH_RANGE, fixedTests[ndx]));
		}

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			// [dag] sorting to keep zNear < zFar
			/** @type {Array<number>} */ var values = [rnd.getFloat(0, 1), rnd.getFloat(0, 1)].sort();
			/** @type {Float32Array} */ var depth = new Float32Array(values);
			gl.depthRange(depth[0], depth[1]);
			this.check(glsStateQuery.verify(gl.DEPTH_RANGE, depth));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.LineWidthCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.LineWidthCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.LineWidthCase.prototype.constructor = es3fFloatStateQueryTests.LineWidthCase;

	es3fFloatStateQueryTests.LineWidthCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.LINE_WIDTH, 1.0));

		/** @type {Float32Array} */ var range = /** @type {Float32Array} */ (gl.getParameter(gl.ALIASED_LINE_WIDTH_RANGE));

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var reference = rnd.getFloat(range[0], range[1]);

			gl.lineWidth(reference);
			this.check(glsStateQuery.verify(gl.LINE_WIDTH, reference));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.PolygonOffsetFactorCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.PolygonOffsetFactorCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.PolygonOffsetFactorCase.prototype.constructor = es3fFloatStateQueryTests.PolygonOffsetFactorCase;

	es3fFloatStateQueryTests.PolygonOffsetFactorCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.POLYGON_OFFSET_FACTOR, 0.0));

		/** @type {Array<number>} */ var fixedTests = [0.0, 0.5, -0.5, 1.5];

		for (var ndx = 0; ndx < fixedTests.length; ++ndx) {
			gl.polygonOffset(fixedTests[ndx], 0);
			this.check(glsStateQuery.verify(gl.POLYGON_OFFSET_FACTOR, fixedTests[ndx]));
		}

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var reference = rnd.getFloat(-64000, 64000);

			gl.polygonOffset(reference, 0);
			this.check(glsStateQuery.verify(gl.POLYGON_OFFSET_FACTOR, reference));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.PolygonOffsetUnitsCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.PolygonOffsetUnitsCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.PolygonOffsetUnitsCase.prototype.constructor = es3fFloatStateQueryTests.PolygonOffsetUnitsCase;

	es3fFloatStateQueryTests.PolygonOffsetUnitsCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.POLYGON_OFFSET_UNITS, 0.0));

		/** @type {Array<number>} */ var fixedTests = [0.0, 0.5, -0.5, 1.5];

		for (var ndx = 0; ndx < fixedTests.length; ++ndx) {
			gl.polygonOffset(0, fixedTests[ndx]);
			this.check(glsStateQuery.verify(gl.POLYGON_OFFSET_UNITS, fixedTests[ndx]));
		}

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var reference = rnd.getFloat(-64000, 64000);

			gl.polygonOffset(0, reference);
			this.check(glsStateQuery.verify(gl.POLYGON_OFFSET_UNITS, reference));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.SampleCoverageCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.SampleCoverageCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.SampleCoverageCase.prototype.constructor = es3fFloatStateQueryTests.SampleCoverageCase;

	es3fFloatStateQueryTests.SampleCoverageCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.SAMPLE_COVERAGE_VALUE, 1.0));

		/** @type {Array<number>} */ var fixedTests = [0.0, 0.5, 0.45, 0.55];

		for (var ndx = 0; ndx < fixedTests.length; ++ndx) {
			gl.sampleCoverage(fixedTests[ndx], false);
			this.check(glsStateQuery.verify(gl.SAMPLE_COVERAGE_VALUE, fixedTests[ndx]));
		}

		/** @type {Array<number>} */ var clampTests = [-1.0, -1.5, 1.45, 3.55];

		for (var ndx = 0; ndx < clampTests.length; ++ndx) {
			gl.sampleCoverage(clampTests[ndx], false);
			this.check(glsStateQuery.verify(gl.SAMPLE_COVERAGE_VALUE, deMath.clamp(clampTests[ndx], 0.0, 1.0)));
		}

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var reference	= rnd.getFloat(0, 1);
			/** @type {boolean} */ var invert = rnd.getBool() ? true : false;

			gl.sampleCoverage(reference, invert);
			this.check(glsStateQuery.verify(gl.SAMPLE_COVERAGE_VALUE, reference));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.BlendColorCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.BlendColorCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.BlendColorCase.prototype.constructor = es3fFloatStateQueryTests.BlendColorCase;

	es3fFloatStateQueryTests.BlendColorCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.BLEND_COLOR, new Float32Array([0, 0, 0, 0])));

		/** @type {Array<Float32Array>} */ var fixedTests = [
			new Float32Array([0.5, 1.0, 0.5, 1.0]),
			new Float32Array([0.0, 0.5, 0.0, 0.5]),
			new Float32Array([0.0, 0.0, 0.0, 0.0]),
			new Float32Array([1.0, 1.0, 1.0, 1.0])
		];
		for (var ndx = 0; ndx < fixedTests.length; ++ndx) {
			gl.blendColor(fixedTests[ndx][0], fixedTests[ndx][1], fixedTests[ndx][2], fixedTests[ndx][3]);
			this.check(glsStateQuery.verify(gl.BLEND_COLOR, fixedTests[ndx]));
		}

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var r = rnd.getFloat(0, 1);
			/** @type {number} */ var g = rnd.getFloat(0, 1);
			/** @type {number} */ var b = rnd.getFloat(0, 1);
			/** @type {number} */ var a = rnd.getFloat(0, 1);

			gl.blendColor(r, g, b, a);
			this.check(glsStateQuery.verify(gl.BLEND_COLOR, new Float32Array([r, g, b, a])));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.ColorClearCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.ColorClearCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.ColorClearCase.prototype.constructor = es3fFloatStateQueryTests.ColorClearCase;

	es3fFloatStateQueryTests.ColorClearCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		// [dag] In the C++ dEQP code, initial color clear value check is temorarily removed. (until the framework does not alter it)
		this.check(glsStateQuery.verify(gl.COLOR_CLEAR_VALUE, new Float32Array([0, 0, 0, 0])));

		/** @type {Array<Float32Array>} */ var fixedTests = [
			new Float32Array([0.5, 1.0, 0.5, 1.0]),
			new Float32Array([0.0, 0.5, 0.0, 0.5]),
			new Float32Array([0.0, 0.0, 0.0, 0.0]),
			new Float32Array([1.0, 1.0, 1.0, 1.0])
		];
		for (var ndx = 0; ndx < fixedTests.length; ++ndx) {
			gl.clearColor(fixedTests[ndx][0], fixedTests[ndx][1], fixedTests[ndx][2], fixedTests[ndx][3]);
			this.check(glsStateQuery.verify(gl.COLOR_CLEAR_VALUE, fixedTests[ndx]));
		}

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var r = rnd.getFloat(0, 1);
			/** @type {number} */ var g = rnd.getFloat(0, 1);
			/** @type {number} */ var b = rnd.getFloat(0, 1);
			/** @type {number} */ var a = rnd.getFloat(0, 1);

			gl.clearColor(r, g, b, a);
			this.check(glsStateQuery.verify(gl.COLOR_CLEAR_VALUE, new Float32Array([r, g, b, a])));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.DepthClearCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.DepthClearCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.DepthClearCase.prototype.constructor = es3fFloatStateQueryTests.DepthClearCase;

	es3fFloatStateQueryTests.DepthClearCase.prototype.test = function() {
		/** @type {deRandom.Random} */ var rnd = new deRandom.Random(0xabcdef);

		this.check(glsStateQuery.verify(gl.DEPTH_CLEAR_VALUE, 1));

		/** @type {number} */ var numIterations = 120;
		for (var i = 0; i < numIterations; ++i) {
			/** @type {number} */ var ref = rnd.getFloat(0, 1);

			gl.clearDepth(ref);
			this.check(glsStateQuery.verify(gl.DEPTH_CLEAR_VALUE, ref));
		}
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.MaxTextureLODBiasCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.MaxTextureLODBiasCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.MaxTextureLODBiasCase.prototype.constructor = es3fFloatStateQueryTests.MaxTextureLODBiasCase;

	es3fFloatStateQueryTests.MaxTextureLODBiasCase.prototype.test = function() {
		this.check(glsStateQuery.verifyGreaterOrEqual(gl.MAX_TEXTURE_LOD_BIAS, 2.0));
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.AliasedPointSizeRangeCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.AliasedPointSizeRangeCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.AliasedPointSizeRangeCase.prototype.constructor = es3fFloatStateQueryTests.AliasedPointSizeRangeCase;

	es3fFloatStateQueryTests.AliasedPointSizeRangeCase.prototype.test = function() {
		var pointSizeRange = /** @type {Float32Array} */ (gl.getParameter(gl.ALIASED_POINT_SIZE_RANGE));
		/** @type {Float32Array} */ var reference = new Float32Array([1, 1]);
		this.check(pointSizeRange[0] <= reference[0] && pointSizeRange[1] >= reference[1]);
	};

	/**
	 * @constructor
	 * @extends {es3fApiCase.ApiCase}
	 * @param {string} name
	 * @param {string} description
	 */
	es3fFloatStateQueryTests.AliasedLineWidthRangeCase = function(name, description) {
		es3fApiCase.ApiCase.call(this, name, description, gl);
	};

	es3fFloatStateQueryTests.AliasedLineWidthRangeCase.prototype = Object.create(es3fApiCase.ApiCase.prototype);
	es3fFloatStateQueryTests.AliasedLineWidthRangeCase.prototype.constructor = es3fFloatStateQueryTests.AliasedLineWidthRangeCase;

	es3fFloatStateQueryTests.AliasedLineWidthRangeCase.prototype.test = function() {
		var lineWidthRange = /** @type {Float32Array} */ (gl.getParameter(gl.ALIASED_LINE_WIDTH_RANGE));
		/** @type {Float32Array} */ var reference = new Float32Array([1, 1]);
		this.check(lineWidthRange[0] <= reference[0] && lineWidthRange[1] >= reference[1]);
	};

    /**
    * @constructor
    * @extends {tcuTestCase.DeqpTest}
    */
    es3fFloatStateQueryTests.FloatStateQueryTests = function() {
        tcuTestCase.DeqpTest.call(this, 'floats', 'Float Values');
    };

    es3fFloatStateQueryTests.FloatStateQueryTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fFloatStateQueryTests.FloatStateQueryTests.prototype.constructor = es3fFloatStateQueryTests.FloatStateQueryTests;

    es3fFloatStateQueryTests.FloatStateQueryTests.prototype.init = function() {
		this.addChild(new es3fFloatStateQueryTests.DepthRangeCase('depth_range', 'DEPTH_RANGE'));
		this.addChild(new es3fFloatStateQueryTests.LineWidthCase('line_width', 'LINE_WIDTH'));
		this.addChild(new es3fFloatStateQueryTests.PolygonOffsetFactorCase('polygon_offset_factor', 'POLYGON_OFFSET_FACTOR'));
		this.addChild(new es3fFloatStateQueryTests.PolygonOffsetUnitsCase('polygon_offset_units', 'POLYGON_OFFSET_UNITS'));
		this.addChild(new es3fFloatStateQueryTests.SampleCoverageCase('sample_coverage_value', 'SAMPLE_COVERAGE_VALUE'));
		this.addChild(new es3fFloatStateQueryTests.BlendColorCase('blend_color', 'BLEND_COLOR'));
		this.addChild(new es3fFloatStateQueryTests.ColorClearCase('color_clear_value', 'COLOR_CLEAR_VALUE'));
		this.addChild(new es3fFloatStateQueryTests.DepthClearCase('depth_clear_value', 'DEPTH_CLEAR_VALUE'));
		this.addChild(new es3fFloatStateQueryTests.MaxTextureLODBiasCase('max_texture_lod_bias', 'MAX_TEXTURE_LOD_BIAS'));
		this.addChild(new es3fFloatStateQueryTests.AliasedPointSizeRangeCase('aliased_point_size_range', 'ALIASED_POINT_SIZE_RANGE'));
		this.addChild(new es3fFloatStateQueryTests.AliasedLineWidthRangeCase('aliased_line_width_range', 'ALIASED_LINE_WIDTH_RANGE'));

    };

    /**
    * Run test
    * @param {WebGL2RenderingContext} context
    */
    es3fFloatStateQueryTests.run = function(context) {
    	gl = context;
    	//Set up Test Root parameters
    	var state = tcuTestCase.runner;
    	state.setRoot(new es3fFloatStateQueryTests.FloatStateQueryTests());

    	//Set up name and description of this test series.
    	setCurrentTestName(state.testCases.fullName());
    	description(state.testCases.getDescription());

    	try {
    		//Run test cases
    		tcuTestCase.runTestCases();
    	}
    	catch (err) {
    		testFailedOptions('Failed to es3fFloatStateQueryTests.run tests', false);
    		tcuTestCase.runner.terminate();
    	}
    };

});
