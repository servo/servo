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
goog.provide('functional.gles3.es3fTextureStateQuery');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('functional.gles3.es3fApiCase');
goog.require('modules.shared.glsStateQuery');

goog.scope(function() {
var es3fTextureStateQuery = functional.gles3.es3fTextureStateQuery;
var tcuTestCase = framework.common.tcuTestCase;
var glsStateQuery = modules.shared.glsStateQuery;
var es3fApiCase = functional.gles3.es3fApiCase;
var deRandom = framework.delibs.debase.deRandom;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 */
es3fTextureStateQuery.TextureCase = function(name, description, textureTarget) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    /** @type {WebGLTexture} */ this.m_texture;
    this.m_textureTarget = textureTarget;
};

setParentClass(es3fTextureStateQuery.TextureCase, es3fApiCase.ApiCase);

es3fTextureStateQuery.TextureCase.prototype.testTexture = function() {
    throw new Error('Virtual function. Please override.');
};

es3fTextureStateQuery.TextureCase.prototype.test = function() {
    this.m_texture = gl.createTexture();
    gl.bindTexture(this.m_textureTarget, this.m_texture);

    this.testTexture();

    gl.bindTexture(this.m_textureTarget, null);
    gl.deleteTexture(this.m_texture);
};

/**
 * @constructor
 * @extends {es3fTextureStateQuery.TextureCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 */
es3fTextureStateQuery.IsTextureCase = function(name, description, textureTarget) {
    es3fTextureStateQuery.TextureCase.call(this, name, description, textureTarget);
};

setParentClass(es3fTextureStateQuery.IsTextureCase, es3fTextureStateQuery.TextureCase);

es3fTextureStateQuery.IsTextureCase.prototype.testTexture = function() {
    this.check(glsStateQuery.compare(gl.isTexture(this.m_texture), true), 'gl.isTexture() should have returned true');
};

/**
 * @constructor
 * @extends {es3fTextureStateQuery.TextureCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 * @param {number} valueName
 * @param {number} initialValue
 * @param {Array<number>} valueRange
 */
es3fTextureStateQuery.TextureParamCase = function(name, description, textureTarget, valueName, initialValue, valueRange) {
    es3fTextureStateQuery.TextureCase.call(this, name, description, textureTarget);
    this.m_valueName = valueName;
    this.m_initialValue = initialValue;
    this.m_valueRange = valueRange;
};

setParentClass(es3fTextureStateQuery.TextureParamCase, es3fTextureStateQuery.TextureCase);

es3fTextureStateQuery.TextureParamCase.prototype.testTexture = function() {
    this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_valueName, this.m_initialValue));

    for (var ndx = 0; ndx < this.m_valueRange.length; ++ndx) {
        gl.texParameteri(this.m_textureTarget, this.m_valueName, this.m_valueRange[ndx]);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_valueName, this.m_valueRange[ndx]));
    }

    //check unit conversions with float

    for (var ndx = 0; ndx < this.m_valueRange.length; ++ndx) {
        gl.texParameterf(this.m_textureTarget, this.m_valueName, this.m_valueRange[ndx]);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_valueName, this.m_valueRange[ndx]));
    }
};

/**
 * @constructor
 * @extends {es3fTextureStateQuery.TextureCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 * @param {number} lodTarget
 * @param {number} initialValue
 */
es3fTextureStateQuery.TextureLODCase = function(name, description, textureTarget, lodTarget, initialValue) {
    es3fTextureStateQuery.TextureCase.call(this, name, description, textureTarget);
    this.m_lodTarget = lodTarget;
    this.m_initialValue = initialValue;
};

setParentClass(es3fTextureStateQuery.TextureLODCase, es3fTextureStateQuery.TextureCase);

es3fTextureStateQuery.TextureLODCase.prototype.testTexture = function() {
    var rnd = new deRandom.Random(0xabcdef);

    this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_lodTarget, this.m_initialValue));

    var numIterations = 60;
    for (var ndx = 0; ndx < numIterations; ++ndx) {
        var ref = rnd.getFloat(-64000, 64000);

        gl.texParameterf(this.m_textureTarget, this.m_lodTarget, ref);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_lodTarget, ref));
    }

    // check unit conversions with int

    for (var ndx = 0; ndx < numIterations; ++ndx) {
        var ref = rnd.getInt(-64000, 64000);

        gl.texParameteri(this.m_textureTarget, this.m_lodTarget, ref);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_lodTarget, ref));
    }
};

/**
 * @constructor
 * @extends {es3fTextureStateQuery.TextureCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 * @param {number} levelTarget
 * @param {number} initialValue
 */
es3fTextureStateQuery.TextureLevelCase = function(name, description, textureTarget, levelTarget, initialValue) {
    es3fTextureStateQuery.TextureCase.call(this, name, description, textureTarget);
    this.m_levelTarget = levelTarget;
    this.m_initialValue = initialValue;
};

setParentClass(es3fTextureStateQuery.TextureLevelCase, es3fTextureStateQuery.TextureCase);

es3fTextureStateQuery.TextureLevelCase.prototype.testTexture = function() {
    var rnd = new deRandom.Random(0xabcdef);

    this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_levelTarget, this.m_initialValue));

    var numIterations = 60;
    for (var ndx = 0; ndx < numIterations; ++ndx) {
        var ref = rnd.getInt(0, 64000);

        gl.texParameteri(this.m_textureTarget, this.m_levelTarget, ref);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_levelTarget, ref));
    }

    // check unit conversions with float
    var nonSignificantOffsets = [-0.45, -0.25, 0, 0.45]; // offsets O so that for any integers z in Z, o in O roundToClosestInt(z+o)==z

    for (var ndx = 0; ndx < numIterations; ++ndx) {
        var ref = rnd.getInt(0, 64000);

        for (var i = 0; i < nonSignificantOffsets.length; i++) {
            gl.texParameterf(this.m_textureTarget, this.m_levelTarget, ref + nonSignificantOffsets[i]);
            this.check(glsStateQuery.verifyTexture(this.m_textureTarget, this.m_levelTarget, ref));
        }
    }
};

/**
 * @constructor
 * @extends {es3fTextureStateQuery.TextureCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 */
es3fTextureStateQuery.TextureImmutableLevelsCase = function(name, description, textureTarget) {
    es3fTextureStateQuery.TextureCase.call(this, name, description, textureTarget);
};

setParentClass(es3fTextureStateQuery.TextureImmutableLevelsCase, es3fTextureStateQuery.TextureCase);

es3fTextureStateQuery.TextureImmutableLevelsCase.prototype.testTexture = function() {
    this.check(glsStateQuery.verifyTexture(this.m_textureTarget, gl.TEXTURE_IMMUTABLE_LEVELS, 0));
    for (var level = 1; level <= 8; ++level) {
        var textureID = gl.createTexture();
        gl.bindTexture(this.m_textureTarget, textureID);

        if (this.m_textureTarget == gl.TEXTURE_2D_ARRAY || this.m_textureTarget == gl.TEXTURE_3D)
            gl.texStorage3D(this.m_textureTarget, level, gl.RGB8, 256, 256, 256);
        else
            gl.texStorage2D(this.m_textureTarget, level, gl.RGB8, 256, 256);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, gl.TEXTURE_IMMUTABLE_LEVELS, level));
        gl.deleteTexture(textureID);
    }
};

/**
 * @constructor
 * @extends {es3fTextureStateQuery.TextureCase}
 * @param {string} name
 * @param {string} description
 * @param {number} textureTarget
 */
es3fTextureStateQuery.TextureImmutableFormatCase = function(name, description, textureTarget) {
    es3fTextureStateQuery.TextureCase.call(this, name, description, textureTarget);
};

setParentClass(es3fTextureStateQuery.TextureImmutableFormatCase, es3fTextureStateQuery.TextureCase);

es3fTextureStateQuery.TextureImmutableFormatCase.prototype.testTexture = function() {
    this.check(glsStateQuery.verifyTexture(this.m_textureTarget, gl.TEXTURE_IMMUTABLE_LEVELS, 0));
    var testSingleFormat = function(format) {
        var textureID = gl.createTexture();
        gl.bindTexture(this.m_textureTarget, textureID);

        if (this.m_textureTarget == gl.TEXTURE_2D_ARRAY || this.m_textureTarget == gl.TEXTURE_3D)
            gl.texStorage3D(this.m_textureTarget, 1, format, 32, 32, 32);
        else
            gl.texStorage2D(this.m_textureTarget, 1, format, 32, 32);

        this.check(glsStateQuery.verifyTexture(this.m_textureTarget, gl.TEXTURE_IMMUTABLE_FORMAT, 1));
        gl.deleteTexture(textureID);
    };

    var formats = [
        gl.RGBA32I, gl.RGBA32UI, gl.RGBA16I, gl.RGBA16UI, gl.RGBA8, gl.RGBA8I,
        gl.RGBA8UI, gl.SRGB8_ALPHA8, gl.RGB10_A2, gl.RGB10_A2UI, gl.RGBA4,
        gl.RGB5_A1, gl.RGB8, gl.RGB565, gl.RG32I, gl.RG32UI, gl.RG16I, gl.RG16UI,
        gl.RG8, gl.RG8I, gl.RG8UI, gl.R32I, gl.R32UI, gl.R16I, gl.R16UI, gl.R8,
        gl.R8I, gl.R8UI,

        gl.RGBA32F, gl.RGBA16F, gl.RGBA8_SNORM, gl.RGB32F,
        gl.RGB32I, gl.RGB32UI, gl.RGB16F, gl.RGB16I, gl.RGB16UI, gl.RGB8_SNORM,
        gl.RGB8I, gl.RGB8UI, gl.SRGB8, gl.R11F_G11F_B10F, gl.RGB9_E5, gl.RG32F,
        gl.RG16F, gl.RG8_SNORM, gl.R32F, gl.R16F, gl.R8_SNORM
    ];

    var non3dFormats = [
        gl.DEPTH_COMPONENT32F, gl.DEPTH_COMPONENT24, gl.DEPTH_COMPONENT16,
        gl.DEPTH32F_STENCIL8, gl.DEPTH24_STENCIL8
    ];

    for (var formatNdx = 0; formatNdx < formats.length; ++formatNdx)
        testSingleFormat.bind(this, formats[formatNdx]);

    if (this.m_textureTarget != gl.TEXTURE_3D)
        for (var formatNdx = 0; formatNdx < non3dFormats.length; ++formatNdx)
            testSingleFormat.bind(this, non3dFormats[formatNdx]);
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fTextureStateQuery.TextureStateQuery = function() {
    tcuTestCase.DeqpTest.call(this, 'texture', 'Texture State Query tests');
};

es3fTextureStateQuery.TextureStateQuery.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fTextureStateQuery.TextureStateQuery.prototype.constructor = es3fTextureStateQuery.TextureStateQuery;

es3fTextureStateQuery.TextureStateQuery.prototype.init = function() {
    var textureTargets = [
        ['texture_2d', gl.TEXTURE_2D],
        ['texture_3d', gl.TEXTURE_3D],
        ['texture_2d_array', gl.TEXTURE_2D_ARRAY],
        ['texture_cube_map', gl.TEXTURE_CUBE_MAP]
    ];

    var state = this;
    var wrapValues = [gl.CLAMP_TO_EDGE, gl.REPEAT, gl.MIRRORED_REPEAT];
    var magValues = [gl.NEAREST, gl.LINEAR];
    var minValues = [gl.NEAREST, gl.LINEAR, gl.NEAREST_MIPMAP_NEAREST, gl.NEAREST_MIPMAP_LINEAR, gl.LINEAR_MIPMAP_NEAREST, gl.LINEAR_MIPMAP_LINEAR];
    var modes = [gl.COMPARE_REF_TO_TEXTURE, gl.NONE];
    var compareFuncs = [gl.LEQUAL, gl.GEQUAL, gl.LESS, gl.GREATER, gl.EQUAL, gl.NOTEQUAL, gl.ALWAYS, gl.NEVER];
    textureTargets.forEach(function(elem) {
        var name = elem[0];
        var target = elem[1];
        state.addChild(new es3fTextureStateQuery.IsTextureCase(name + '_is_texture', 'IsTexture', target));
        state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_wrap_s' , 'TEXTURE_WRAP_S',
            target, gl.TEXTURE_WRAP_S, gl.REPEAT, wrapValues));
        if (target == gl.TEXTURE_2D ||
            target == gl.TEXTURE_3D ||
            target == gl.TEXTURE_CUBE_MAP)
            state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_wrap_t' , 'TEXTURE_WRAP_T',
                target, gl.TEXTURE_WRAP_T, gl.REPEAT, wrapValues));

        if (target == gl.TEXTURE_3D)
            state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_wrap_r' , 'TEXTURE_WRAP_R',
                target, gl.TEXTURE_WRAP_R, gl.REPEAT, wrapValues));

        state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_mag_filter' , 'TEXTURE_MAG_FILTER',
            target, gl.TEXTURE_MAG_FILTER, gl.LINEAR, magValues));
        state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_min_filter' , 'TEXTURE_MIN_FILTER',
            target, gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_LINEAR, minValues));
        state.addChild(new es3fTextureStateQuery.TextureLODCase(name + '_texture_min_lod' , 'TEXTURE_MIN_LOD', target, gl.TEXTURE_MIN_LOD, -1000));
        state.addChild(new es3fTextureStateQuery.TextureLODCase(name + '_texture_max_lod' , 'TEXTURE_MAX_LOD', target, gl.TEXTURE_MAX_LOD, 1000));
        state.addChild(new es3fTextureStateQuery.TextureLevelCase(name + '_texture_base_level' , 'TEXTURE_BASE_LEVEL', target, gl.TEXTURE_BASE_LEVEL, 0));
        state.addChild(new es3fTextureStateQuery.TextureLevelCase(name + '_texture_max_level' , 'TEXTURE_MAX_LEVEL', target, gl.TEXTURE_MAX_LEVEL, 1000));

        state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_compare_mode' , 'TEXTURE_COMPARE_MODE',
            target, gl.TEXTURE_COMPARE_MODE, gl.NONE, modes));
        state.addChild(new es3fTextureStateQuery.TextureParamCase(name + '_texture_compare_func' , 'TEXTURE_COMPARE_FUNC',
            target, gl.TEXTURE_COMPARE_FUNC, gl.LEQUAL, compareFuncs));

        state.addChild(new es3fTextureStateQuery.TextureImmutableLevelsCase(name + '_texture_immutable_levels', 'TEXTURE_IMMUTABLE_LEVELS', target));
        state.addChild(new es3fTextureStateQuery.TextureImmutableFormatCase(name + '_texture_immutable_format', 'TEXTURE_IMMUTABLE_FORMAT', target));
    });
};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fTextureStateQuery.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fTextureStateQuery.TextureStateQuery());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fTextureStateQuery.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});
