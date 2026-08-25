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
goog.provide('functional.gles3.es3fUniformBlockTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderUtil');
goog.require('modules.shared.glsRandomUniformBlockCase');
goog.require('modules.shared.glsUniformBlockCase');

goog.scope(function() {

    var es3fUniformBlockTests = functional.gles3.es3fUniformBlockTests;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var glsUniformBlockCase = modules.shared.glsUniformBlockCase;
    var glsRandomUniformBlockCase = modules.shared.glsRandomUniformBlockCase;
    var tcuTestCase = framework.common.tcuTestCase;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;

    /**
     * es3fUniformBlockTests.createRandomCaseGroup
     * @param {tcuTestCase.DeqpTest} parentGroup
     * @param {string} groupName
     * @param {string} description
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} features
     * @param {number} numCases
     * @param {number} baseSeed
     */
    es3fUniformBlockTests.createRandomCaseGroup = function(parentGroup, groupName, description, bufferMode, features, numCases, baseSeed) {
        /** @type {tcuTestCase.DeqpTest} */
        var group = tcuTestCase.newTest(groupName, description);
        parentGroup.addChild(group);

        baseSeed += deRandom.getBaseSeed();

        for (var ndx = 0; ndx < numCases; ndx++)
            group.addChild(new glsRandomUniformBlockCase.RandomUniformBlockCase('' + ndx, '', bufferMode, features, ndx + baseSeed));
    };

    /**
     * es3fUniformBlockTests.BlockBasicTypeCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {glsUniformBlockCase.VarType} type The type of the block
     * @param {number} layoutFlags
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockBasicTypeCase = function(name, description, type, layoutFlags, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK);
        /** @type {glsUniformBlockCase.UniformBlock}*/ var block = this.m_interface.allocBlock('Block');
        block.addUniform(new glsUniformBlockCase.Uniform('var', type, 0));
        block.setFlags(layoutFlags);

        if (numInstances > 0) {
            block.setArraySize(numInstances);
            block.setInstanceName('block');
        }
    };

    es3fUniformBlockTests.BlockBasicTypeCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockBasicTypeCase.prototype.constructor = es3fUniformBlockTests.BlockBasicTypeCase;

    /**
     * es3fUniformBlockTests.createBlockBasicTypeCases
     * @param {tcuTestCase.DeqpTest} group
     * @param {string} name
     * @param {glsUniformBlockCase.VarType} type
     * @param {number} layoutFlags
     * @param {number=} numInstances
     */
    es3fUniformBlockTests.createBlockBasicTypeCases = function(group, name, type, layoutFlags, numInstances) {
        numInstances = (numInstances === undefined) ? 0 : numInstances;
        group.addChild(new es3fUniformBlockTests.BlockBasicTypeCase(name + '_vertex', '', type, layoutFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, numInstances));
        group.addChild(new es3fUniformBlockTests.BlockBasicTypeCase(name + '_fragment', '', type, layoutFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, numInstances));

        //alert(group.spec[0].m_instance);
        if (!(layoutFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
            group.addChild(new es3fUniformBlockTests.BlockBasicTypeCase(name + '_both', '', type, layoutFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, numInstances));
    };

    /**
     * es3fUniformBlockTests.BlockSingleStructCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {number} layoutFlags
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockSingleStructCase = function(name, description, layoutFlags, bufferMode, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, bufferMode);
        this.m_layoutFlags = layoutFlags;
        this.m_numInstances = numInstances;
    };

    es3fUniformBlockTests.BlockSingleStructCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockSingleStructCase.prototype.constructor = es3fUniformBlockTests.BlockSingleStructCase;

    es3fUniformBlockTests.BlockSingleStructCase.prototype.init = function() {
        /**@type {glsUniformBlockCase.StructType}*/ var typeS = this.m_interface.allocStruct('S');
        typeS.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC3, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), glsUniformBlockCase.UniformFlags.UNUSED_BOTH); // First member is unused.
        typeS.addMember('b', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM), 4));
        typeS.addMember('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_HIGH));

        /** @type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.allocBlock('Block');
        block.addUniform(new glsUniformBlockCase.Uniform('s', glsUniformBlockCase.newVarTypeStruct(typeS), 0));
        block.setFlags(this.m_layoutFlags);

        if (this.m_numInstances > 0) {
            block.setInstanceName('block');
            block.setArraySize(this.m_numInstances);
        }
    };

    /**
     * es3fUniformBlockTests.BlockSingleStructArrayCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {number} layoutFlags
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockSingleStructArrayCase = function(name, description, layoutFlags, bufferMode, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, bufferMode);
        this.m_layoutFlags = layoutFlags;
        this.m_numInstances = numInstances;
    };

    es3fUniformBlockTests.BlockSingleStructArrayCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockSingleStructArrayCase.prototype.constructor = es3fUniformBlockTests.BlockSingleStructArrayCase;

    es3fUniformBlockTests.BlockSingleStructArrayCase.prototype.init = function() {
        /**@type {glsUniformBlockCase.StructType}*/ var typeS = this.m_interface.allocStruct('S');
        typeS.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC3, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), glsUniformBlockCase.UniformFlags.UNUSED_BOTH); // First member is unused.
        typeS.addMember('b', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM), 4));
        typeS.addMember('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_HIGH));

        /** @type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.allocBlock('Block');
        block.addUniform(new glsUniformBlockCase.Uniform('u', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.UINT, glsUniformBlockCase.UniformFlags.PRECISION_LOW)));
        block.addUniform(new glsUniformBlockCase.Uniform('s', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeStruct(typeS), 3)));
        block.addUniform(new glsUniformBlockCase.Uniform('v', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM)));
        block.setFlags(this.m_layoutFlags);

        if (this.m_numInstances > 0) {
            block.setInstanceName('block');
            block.setArraySize(this.m_numInstances);
        }
    };

    /**
     * es3fUniformBlockTests.BlockSingleNestedStructCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {number} layoutFlags
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockSingleNestedStructCase = function(name, description, layoutFlags, bufferMode, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, bufferMode);
        this.m_layoutFlags = layoutFlags;
        this.m_numInstances = numInstances;
    };

    es3fUniformBlockTests.BlockSingleNestedStructCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockSingleNestedStructCase.prototype.constructor = es3fUniformBlockTests.BlockSingleNestedStructCase;

    es3fUniformBlockTests.BlockSingleNestedStructCase.prototype.init = function() {
        /**@type {glsUniformBlockCase.StructType}*/ var typeS = this.m_interface.allocStruct('S');
        typeS.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC3, glsUniformBlockCase.UniformFlags.PRECISION_HIGH));
        typeS.addMember('b', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM), 4));
        typeS.addMember('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), glsUniformBlockCase.UniformFlags.UNUSED_BOTH);

        /**@type {glsUniformBlockCase.StructType}*/ var typeT = this.m_interface.allocStruct('T');
        typeT.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM));
        typeT.addMember('b', glsUniformBlockCase.newVarTypeStruct(typeS));

        /** @type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.allocBlock('Block');
        block.addUniform(new glsUniformBlockCase.Uniform('s', glsUniformBlockCase.newVarTypeStruct(typeS), 0));
        block.addUniform(new glsUniformBlockCase.Uniform('v', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC2, glsUniformBlockCase.UniformFlags.PRECISION_LOW), glsUniformBlockCase.UniformFlags.UNUSED_BOTH));
        block.addUniform(new glsUniformBlockCase.Uniform('t', glsUniformBlockCase.newVarTypeStruct(typeT), 0));
        block.addUniform(new glsUniformBlockCase.Uniform('u', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.UINT, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), 0));
        block.setFlags(this.m_layoutFlags);

        if (this.m_numInstances > 0) {
            block.setInstanceName('block');
            block.setArraySize(this.m_numInstances);
        }
    };

    /**
     * es3fUniformBlockTests.BlockSingleNestedStructArrayCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {number} layoutFlags
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockSingleNestedStructArrayCase = function(name, description, layoutFlags, bufferMode, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, bufferMode);
        this.m_layoutFlags = layoutFlags;
        this.m_numInstances = numInstances;
    };

    es3fUniformBlockTests.BlockSingleNestedStructArrayCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockSingleNestedStructArrayCase.prototype.constructor = es3fUniformBlockTests.BlockSingleNestedStructArrayCase;

    es3fUniformBlockTests.BlockSingleNestedStructArrayCase.prototype.init = function() {
        /**@type {glsUniformBlockCase.StructType}*/ var typeS = this.m_interface.allocStruct('S');
        typeS.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC3, glsUniformBlockCase.UniformFlags.PRECISION_HIGH));
        typeS.addMember('b', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC2, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM), 4));
        typeS.addMember('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), glsUniformBlockCase.UniformFlags.UNUSED_BOTH);

        /**@type {glsUniformBlockCase.StructType}*/ var typeT = this.m_interface.allocStruct('T');
        typeT.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM));
        typeT.addMember('b', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeStruct(typeS), 3));

        /** @type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.allocBlock('Block');
        block.addUniform(new glsUniformBlockCase.Uniform('s', glsUniformBlockCase.newVarTypeStruct(typeS), 0));
        block.addUniform(new glsUniformBlockCase.Uniform('v', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC2, glsUniformBlockCase.UniformFlags.PRECISION_LOW), glsUniformBlockCase.UniformFlags.UNUSED_BOTH));
        block.addUniform(new glsUniformBlockCase.Uniform('t', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeStruct(typeT), 2), 0));
        block.addUniform(new glsUniformBlockCase.Uniform('u', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.UINT, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), 0));
        block.setFlags(this.m_layoutFlags);

        if (this.m_numInstances > 0) {
            block.setInstanceName('block');
            block.setArraySize(this.m_numInstances);
        }
    };

    /**
     * es3fUniformBlockTests.BlockMultiBasicTypesCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {number} flagsA
     * @param {number} flagsB
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockMultiBasicTypesCase = function(name, description, flagsA, flagsB, bufferMode, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, bufferMode);
        this.m_flagsA = flagsA;
        this.m_flagsB = flagsB;
        this.m_numInstances = numInstances;
    };

    es3fUniformBlockTests.BlockMultiBasicTypesCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockMultiBasicTypesCase.prototype.constructor = es3fUniformBlockTests.BlockMultiBasicTypesCase;

    es3fUniformBlockTests.BlockMultiBasicTypesCase.prototype.init = function() {
        /** @type {glsUniformBlockCase.UniformBlock} */ var blockA = this.m_interface.allocBlock('BlockA');
        blockA.addUniform(new glsUniformBlockCase.Uniform('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT, glsUniformBlockCase.UniformFlags.PRECISION_HIGH)));
        blockA.addUniform(new glsUniformBlockCase.Uniform('b', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.UINT_VEC3, glsUniformBlockCase.UniformFlags.PRECISION_LOW), glsUniformBlockCase.UniformFlags.UNUSED_BOTH));
        blockA.addUniform(new glsUniformBlockCase.Uniform('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT2, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM)));
        blockA.setInstanceName('blockA');
        blockA.setFlags(this.m_flagsA);

        /** @type {glsUniformBlockCase.UniformBlock} */ var blockB = this.m_interface.allocBlock('BlockB');
        blockB.addUniform(new glsUniformBlockCase.Uniform('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM)));
        blockB.addUniform(new glsUniformBlockCase.Uniform('b', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC2, glsUniformBlockCase.UniformFlags.PRECISION_LOW)));
        blockB.addUniform(new glsUniformBlockCase.Uniform('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), glsUniformBlockCase.UniformFlags.UNUSED_BOTH));
        blockB.addUniform(new glsUniformBlockCase.Uniform('d', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.BOOL, 0)));
        blockB.setInstanceName('blockB');
        blockB.setFlags(this.m_flagsB);

        if (this.m_numInstances > 0) {
            blockA.setArraySize(this.m_numInstances);
            blockB.setArraySize(this.m_numInstances);
        }
    };

    /**
     * es3fUniformBlockTests.BlockMultiNestedStructCase constructor
     * @param {string} name The name of the test
     * @param {string} description The description of the test
     * @param {number} flagsA
     * @param {number} flagsB
     * @param {glsUniformBlockCase.BufferMode} bufferMode
     * @param {number} numInstances
     * @constructor
     * @extends {glsUniformBlockCase.UniformBlockCase}
     */
    es3fUniformBlockTests.BlockMultiNestedStructCase = function(name, description, flagsA, flagsB, bufferMode, numInstances) {
        glsUniformBlockCase.UniformBlockCase.call(this, name, description, bufferMode);
        this.m_flagsA = flagsA;
        this.m_flagsB = flagsB;
        this.m_numInstances = numInstances;
    };

    es3fUniformBlockTests.BlockMultiNestedStructCase.prototype = Object.create(glsUniformBlockCase.UniformBlockCase.prototype);
    es3fUniformBlockTests.BlockMultiNestedStructCase.prototype.constructor = es3fUniformBlockTests.BlockMultiNestedStructCase;

    es3fUniformBlockTests.BlockMultiNestedStructCase.prototype.init = function() {
        /**@type {glsUniformBlockCase.StructType}*/ var typeS = this.m_interface.allocStruct('S');
        typeS.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, glsUniformBlockCase.UniformFlags.PRECISION_LOW));
        typeS.addMember('b', glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.INT_VEC2, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM), 4));
        typeS.addMember('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, glsUniformBlockCase.UniformFlags.PRECISION_HIGH));

        /**@type {glsUniformBlockCase.StructType}*/ var typeT = this.m_interface.allocStruct('T');
        typeT.addMember('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.UINT, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM), glsUniformBlockCase.UniformFlags.UNUSED_BOTH);
        typeT.addMember('b', glsUniformBlockCase.newVarTypeStruct(typeS));
        typeT.addMember('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.BOOL_VEC4, 0));

        /** @type {glsUniformBlockCase.UniformBlock} */ var blockA = this.m_interface.allocBlock('BlockA');
        blockA.addUniform(new glsUniformBlockCase.Uniform('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT, glsUniformBlockCase.UniformFlags.PRECISION_HIGH)));
        blockA.addUniform(new glsUniformBlockCase.Uniform('b', glsUniformBlockCase.newVarTypeStruct(typeS)));
        blockA.addUniform(new glsUniformBlockCase.Uniform('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.UINT_VEC3, glsUniformBlockCase.UniformFlags.PRECISION_LOW), glsUniformBlockCase.UniformFlags.UNUSED_BOTH));
        blockA.setInstanceName('blockA');
        blockA.setFlags(this.m_flagsA);

        /** @type {glsUniformBlockCase.UniformBlock} */ var blockB = this.m_interface.allocBlock('BlockB');
        blockB.addUniform(new glsUniformBlockCase.Uniform('a', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.FLOAT_MAT2, glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM)));
        blockB.addUniform(new glsUniformBlockCase.Uniform('b', glsUniformBlockCase.newVarTypeStruct(typeT)));
        blockB.addUniform(new glsUniformBlockCase.Uniform('c', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.BOOL_VEC4, 0), glsUniformBlockCase.UniformFlags.UNUSED_BOTH));
        blockB.addUniform(new glsUniformBlockCase.Uniform('d', glsUniformBlockCase.newVarTypeBasic(gluShaderUtil.DataType.BOOL, 0)));
        blockB.setInstanceName('blockB');
        blockB.setFlags(this.m_flagsB);

        if (this.m_numInstances > 0) {
            blockA.setArraySize(this.m_numInstances);
            blockB.setArraySize(this.m_numInstances);
        }
    };

    /**
     * Creates the test hierarchy to be executed.
     **/
    es3fUniformBlockTests.init = function() {
        /** @const @type {tcuTestCase.DeqpTest} */ var testGroup = tcuTestCase.runner.testCases;

        /** @type {Array<gluShaderUtil.DataType>} */
        var basicTypes = [
            gluShaderUtil.DataType.FLOAT,
            gluShaderUtil.DataType.FLOAT_VEC2,
            gluShaderUtil.DataType.FLOAT_VEC3,
            gluShaderUtil.DataType.FLOAT_VEC4,
            gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT_VEC2,
            gluShaderUtil.DataType.INT_VEC3,
            gluShaderUtil.DataType.INT_VEC4,
            gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT_VEC2,
            gluShaderUtil.DataType.UINT_VEC3,
            gluShaderUtil.DataType.UINT_VEC4,
            gluShaderUtil.DataType.BOOL,
            gluShaderUtil.DataType.BOOL_VEC2,
            gluShaderUtil.DataType.BOOL_VEC3,
            gluShaderUtil.DataType.BOOL_VEC4,
            gluShaderUtil.DataType.FLOAT_MAT2,
            gluShaderUtil.DataType.FLOAT_MAT3,
            gluShaderUtil.DataType.FLOAT_MAT4,
            gluShaderUtil.DataType.FLOAT_MAT2X3,
            gluShaderUtil.DataType.FLOAT_MAT2X4,
            gluShaderUtil.DataType.FLOAT_MAT3X2,
            gluShaderUtil.DataType.FLOAT_MAT3X4,
            gluShaderUtil.DataType.FLOAT_MAT4X2,
            gluShaderUtil.DataType.FLOAT_MAT4X3
        ];

        /** @type {Array<Object.<string, glsUniformBlockCase.UniformFlags>>} */
        var precisionFlags = [{ name: 'lowp', flags: glsUniformBlockCase.UniformFlags.PRECISION_LOW }, { name: 'mediump', flags: glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM }, { name: 'highp', flags: glsUniformBlockCase.UniformFlags.PRECISION_HIGH }
        ];

        /** @type {Array<Object.<string, glsUniformBlockCase.UniformFlags>>} */
        var layoutFlags = [ /* { name: 'shared', flags: glsUniformBlockCase.UniformFlags.LAYOUT_SHARED }, */
            /* { name: 'packed', flags: glsUniformBlockCase.UniformFlags.LAYOUT_PACKED }, */ { name: 'std140', flags: glsUniformBlockCase.UniformFlags.LAYOUT_STD140 }
        ];

        /** @type {Array<Object.<string, glsUniformBlockCase.UniformFlags>>} */
        var matrixFlags = [{ name: 'row_major', flags: glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR }, { name: 'column_major', flags: glsUniformBlockCase.UniformFlags.LAYOUT_COLUMN_MAJOR }
        ];

        /** @type {Array<Object.<string, glsUniformBlockCase.UniformFlags>>} */
        var bufferModes = [{ name: 'per_block_buffer', mode: glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK }, { name: 'single_buffer', mode: glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE }
        ];

        // ubo.single_basic_type
        /** @type {tcuTestCase.DeqpTest} */
        var singleBasicTypeGroup = tcuTestCase.newTest('single_basic_type', 'Single basic variable in single buffer');

        testGroup.addChild(singleBasicTypeGroup);

        /** @type {tcuTestCase.DeqpTest} */
        var layoutGroup;
        /** @type {gluShaderUtil.DataType} */
        var type;
        /** @type {string} */
        var typeName;
        /** @type {tcuTestCase.DeqpTest} */
        var modeGroup;
        /** @type {string} */
        var baseName;
        /** @type {number} */
        var baseFlags;

        for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {

            layoutGroup = tcuTestCase.newTest(layoutFlags[layoutFlagNdx].name, '', null);
            singleBasicTypeGroup.addChild(layoutGroup);

            for (var basicTypeNdx = 0; basicTypeNdx < basicTypes.length; basicTypeNdx++) {
                type = basicTypes[basicTypeNdx];
                typeName = gluShaderUtil.getDataTypeName(type);

                if (gluShaderUtil.isDataTypeBoolOrBVec(type))
                    es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, typeName, glsUniformBlockCase.newVarTypeBasic(type, 0), layoutFlags[layoutFlagNdx].flags);
                else {
                    for (var precNdx = 0; precNdx < precisionFlags.length; precNdx++)
                        es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, precisionFlags[precNdx].name + '_' + typeName,
                        glsUniformBlockCase.newVarTypeBasic(type, precisionFlags[precNdx].flags), layoutFlags[layoutFlagNdx].flags);
                }

                if (gluShaderUtil.isDataTypeMatrix(type)) {
                    for (var matFlagNdx = 0; matFlagNdx < matrixFlags.length; matFlagNdx++) {
                        for (var precNdx = 0; precNdx < precisionFlags.length; precNdx++)
                            es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, matrixFlags[matFlagNdx].name + '_' + precisionFlags[precNdx].name + '_' + typeName,
                            glsUniformBlockCase.newVarTypeBasic(type, precisionFlags[precNdx].flags), layoutFlags[layoutFlagNdx].flags | matrixFlags[matFlagNdx].flags);
                    }
                }
            }
        }
        bufferedLogToConsole('ubo.single_basic_type: Tests created');

        // ubo.single_basic_array
        /** @type {tcuTestCase.DeqpTest} */
        var singleBasicArrayGroup = tcuTestCase.newTest('single_basic_array', 'Single basic array variable in single buffer');
        testGroup.addChild(singleBasicArrayGroup);

        for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
            layoutGroup = tcuTestCase.newTest(layoutFlags[layoutFlagNdx].name, '', null);
            singleBasicArrayGroup.addChild(layoutGroup);

            for (var basicTypeNdx = 0; basicTypeNdx < basicTypes.length; basicTypeNdx++) {
                type = basicTypes[basicTypeNdx];
                typeName = gluShaderUtil.getDataTypeName(type);
                /** @type {number} */ var arraySize = 3;

                es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, typeName,
                    glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(type, gluShaderUtil.isDataTypeBoolOrBVec(type) ? 0 : glsUniformBlockCase.UniformFlags.PRECISION_HIGH), arraySize),
                    layoutFlags[layoutFlagNdx].flags);

                if (gluShaderUtil.isDataTypeMatrix(type)) {
                    for (var matFlagNdx = 0; matFlagNdx < matrixFlags.length; matFlagNdx++)
                        es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, matrixFlags[matFlagNdx].name + '_' + typeName,
                        glsUniformBlockCase.newVarTypeArray(glsUniformBlockCase.newVarTypeBasic(type, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), arraySize),
                            layoutFlags[layoutFlagNdx].flags | matrixFlags[matFlagNdx].flags);
                }
            }
        }
        bufferedLogToConsole('ubo.single_basic_array: Tests created');

        // ubo.single_struct
        /** @type {tcuTestCase.DeqpTest} */
        var singleStructGroup = tcuTestCase.newTest('single_struct', 'Single struct in uniform block');
        testGroup.addChild(singleStructGroup);

        for (var modeNdx = 0; modeNdx < bufferModes.length; modeNdx++) {
            modeGroup = tcuTestCase.newTest(bufferModes[modeNdx].name, '');
            singleStructGroup.addChild(modeGroup);

            for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
                for (var isArray = 0; isArray < 2; isArray++) {
                    baseName = layoutFlags[layoutFlagNdx].name;
                    baseFlags = layoutFlags[layoutFlagNdx].flags;

                    if (bufferModes[modeNdx].mode == glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE && isArray == 0)
                        continue; // Doesn't make sense to add this variant.

                    if (isArray)
                        baseName += '_instance_array';

                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleStructCase(baseName + '_vertex', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleStructCase(baseName + '_fragment', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    if (!(baseFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
                        modeGroup.addChild(new es3fUniformBlockTests.BlockSingleStructCase(baseName + '_both', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                }
            }
        }
        bufferedLogToConsole('ubo.single_struct: Tests created');

        // ubo.single_struct_array
        /** @type {tcuTestCase.DeqpTest} */
        var singleStructArrayGroup = tcuTestCase.newTest('single_struct_array', 'Struct array in one uniform block');
        testGroup.addChild(singleStructArrayGroup);

        for (var modeNdx = 0; modeNdx < bufferModes.length; modeNdx++) {
            modeGroup = tcuTestCase.newTest(bufferModes[modeNdx].name, '');
            singleStructArrayGroup.addChild(modeGroup);

            for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
                for (var isArray = 0; isArray < 2; isArray++) {
                    baseName = layoutFlags[layoutFlagNdx].name;
                    baseFlags = layoutFlags[layoutFlagNdx].flags;

                    if (bufferModes[modeNdx].mode == glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE && isArray == 0)
                        continue; // Doesn't make sense to add this variant.

                    if (isArray)
                        baseName += '_instance_array';

                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleStructArrayCase(baseName + '_vertex', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleStructArrayCase(baseName + '_fragment', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    if (!(baseFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
                        modeGroup.addChild(new es3fUniformBlockTests.BlockSingleStructArrayCase(baseName + '_both', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                }
            }
        }
        bufferedLogToConsole('ubo.single_struct_array: Tests created');

        // ubo.single_nested_struct
        /** @type {tcuTestCase.DeqpTest} */
        var singleNestedStructGroup = tcuTestCase.newTest('single_nested_struct', 'Nested struct in one uniform block');
        testGroup.addChild(singleNestedStructGroup);

        for (var modeNdx = 0; modeNdx < bufferModes.length; modeNdx++) {
            modeGroup = tcuTestCase.newTest(bufferModes[modeNdx].name, '');
            singleNestedStructGroup.addChild(modeGroup);

            for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
                for (var isArray = 0; isArray < 2; isArray++) {
                    baseName = layoutFlags[layoutFlagNdx].name;
                    baseFlags = layoutFlags[layoutFlagNdx].flags;

                    if (bufferModes[modeNdx].mode == glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE && isArray == 0)
                        continue; // Doesn't make sense to add this variant.

                    if (isArray)
                        baseName += '_instance_array';

                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleNestedStructCase(baseName + '_vertex', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleNestedStructCase(baseName + '_fragment', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    if (!(baseFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
                        modeGroup.addChild(new es3fUniformBlockTests.BlockSingleNestedStructCase(baseName + '_both', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                }
            }
        }
        bufferedLogToConsole('ubo.single_nested_struct: Tests created');

        // ubo.single_nested_struct_array
        /** @type {tcuTestCase.DeqpTest} */
        var singleNestedStructArrayGroup = tcuTestCase.newTest('single_nested_struct_array', 'Nested struct array in one uniform block');
        testGroup.addChild(singleNestedStructArrayGroup);

        for (var modeNdx = 0; modeNdx < bufferModes.length; modeNdx++) {
            modeGroup = tcuTestCase.newTest(bufferModes[modeNdx].name, '');
            singleNestedStructArrayGroup.addChild(modeGroup);

            for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
                for (var isArray = 0; isArray < 2; isArray++) {
                    baseName = layoutFlags[layoutFlagNdx].name;
                    baseFlags = layoutFlags[layoutFlagNdx].flags;

                    if (bufferModes[modeNdx].mode == glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE && isArray == 0)
                        continue; // Doesn't make sense to add this variant.

                    if (isArray)
                        baseName += '_instance_array';

                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleNestedStructArrayCase(baseName + '_vertex', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                    modeGroup.addChild(new es3fUniformBlockTests.BlockSingleNestedStructArrayCase(baseName + '_fragment', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    if (!(baseFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
                        modeGroup.addChild(new es3fUniformBlockTests.BlockSingleNestedStructArrayCase(baseName + '_both', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                }
            }
        }
        bufferedLogToConsole('ubo.single_nested_struct_array: Tests created');

        // ubo.instance_array_basic_type
        /** @type {tcuTestCase.DeqpTest} */
        var instanceArrayBasicTypeGroup = tcuTestCase.newTest('instance_array_basic_type', 'Single basic variable in instance array');
        testGroup.addChild(instanceArrayBasicTypeGroup);

        for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
            layoutGroup = tcuTestCase.newTest(layoutFlags[layoutFlagNdx].name, '');
            instanceArrayBasicTypeGroup.addChild(layoutGroup);

            for (var basicTypeNdx = 0; basicTypeNdx < basicTypes.length; basicTypeNdx++) {
                type = basicTypes[basicTypeNdx];
                typeName = gluShaderUtil.getDataTypeName(type);
                /** @type {number} */ var numInstances = 3;

                es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, typeName,
                    glsUniformBlockCase.newVarTypeBasic(type, gluShaderUtil.isDataTypeBoolOrBVec(type) ? 0 : glsUniformBlockCase.UniformFlags.PRECISION_HIGH),
                    layoutFlags[layoutFlagNdx].flags, numInstances);

                if (gluShaderUtil.isDataTypeMatrix(type)) {
                    for (var matFlagNdx = 0; matFlagNdx < matrixFlags.length; matFlagNdx++)
                        es3fUniformBlockTests.createBlockBasicTypeCases(layoutGroup, matrixFlags[matFlagNdx].name + '_' + typeName,
                        glsUniformBlockCase.newVarTypeBasic(type, glsUniformBlockCase.UniformFlags.PRECISION_HIGH), layoutFlags[layoutFlagNdx].flags | matrixFlags[matFlagNdx].flags,
                            numInstances);
                }
            }
        }
        bufferedLogToConsole('ubo.instance_array_basic_type: Tests created');

        // ubo.multi_basic_types
        /** @type {tcuTestCase.DeqpTest} */
        var multiBasicTypesGroup = tcuTestCase.newTest('multi_basic_types', 'Multiple buffers with basic types');
        testGroup.addChild(multiBasicTypesGroup);

        for (var modeNdx = 0; modeNdx < bufferModes.length; modeNdx++) {
            modeGroup = tcuTestCase.newTest(bufferModes[modeNdx].name, '');
            multiBasicTypesGroup.addChild(modeGroup);

            for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
                for (var isArray = 0; isArray < 2; isArray++) {
                    baseName = layoutFlags[layoutFlagNdx].name;
                    baseFlags = layoutFlags[layoutFlagNdx].flags;

                    if (isArray)
                        baseName += '_instance_array';

                    modeGroup.addChild(new es3fUniformBlockTests.BlockMultiBasicTypesCase(baseName + '_vertex', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                    modeGroup.addChild(new es3fUniformBlockTests.BlockMultiBasicTypesCase(baseName + '_fragment', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    if (!(baseFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
                        modeGroup.addChild(new es3fUniformBlockTests.BlockMultiBasicTypesCase(baseName + '_both', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    modeGroup.addChild(new es3fUniformBlockTests.BlockMultiBasicTypesCase(baseName + '_mixed', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                }
            }
        }
        bufferedLogToConsole('ubo.multi_basic_types: Tests created');

        // ubo.multi_nested_struct
        /** @type {tcuTestCase.DeqpTest} */
        var multiNestedStructGroup = tcuTestCase.newTest('multi_nested_struct', 'Multiple buffers with basic types');
        testGroup.addChild(multiNestedStructGroup);

        for (var modeNdx = 0; modeNdx < bufferModes.length; modeNdx++) {
            modeGroup = tcuTestCase.newTest(bufferModes[modeNdx].name, '');
            multiNestedStructGroup.addChild(modeGroup);

            for (var layoutFlagNdx = 0; layoutFlagNdx < layoutFlags.length; layoutFlagNdx++) {
                for (var isArray = 0; isArray < 2; isArray++) {
                    baseName = layoutFlags[layoutFlagNdx].name;
                    baseFlags = layoutFlags[layoutFlagNdx].flags;

                    if (isArray)
                        baseName += '_instance_array';

                    modeGroup.addChild(new es3fUniformBlockTests.BlockMultiNestedStructCase(baseName + '_vertex', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                    modeGroup.addChild(new es3fUniformBlockTests.BlockMultiNestedStructCase(baseName + '_fragment', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    if (!(baseFlags & glsUniformBlockCase.UniformFlags.LAYOUT_PACKED))
                        modeGroup.addChild(new es3fUniformBlockTests.BlockMultiNestedStructCase(baseName + '_both', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));

                    modeGroup.addChild(new es3fUniformBlockTests.BlockMultiNestedStructCase(baseName + '_mixed', '', baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_VERTEX, baseFlags | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT, bufferModes[modeNdx].mode, isArray ? 3 : 0));
                }
            }
        }
        bufferedLogToConsole('ubo.multi_nested_struct: Tests created');

        /* ubo.random */
         /** @type {number} */ var allShaders = glsRandomUniformBlockCase.FeatureBits.FEATURE_VERTEX_BLOCKS | glsRandomUniformBlockCase.FeatureBits.FEATURE_FRAGMENT_BLOCKS | glsRandomUniformBlockCase.FeatureBits.FEATURE_SHARED_BLOCKS;
         /** @type {number} */ var allLayouts = glsRandomUniformBlockCase.FeatureBits.FEATURE_STD140_LAYOUT;
         /** @type {number} */ var allBasicTypes = glsRandomUniformBlockCase.FeatureBits.FEATURE_VECTORS | glsRandomUniformBlockCase.FeatureBits.FEATURE_MATRICES;
         /** @type {number} */ var unused = glsRandomUniformBlockCase.FeatureBits.FEATURE_UNUSED_MEMBERS | glsRandomUniformBlockCase.FeatureBits.FEATURE_UNUSED_UNIFORMS;
         /** @type {number} */ var matFlags = glsRandomUniformBlockCase.FeatureBits.FEATURE_MATRIX_LAYOUT;
         /** @type {number} */ var allFeatures = (~glsRandomUniformBlockCase.FeatureBits.FEATURE_ARRAYS_OF_ARRAYS & 0xFFFF);

         /** @type {tcuTestCase.DeqpTest} */
         var randomGroup = tcuTestCase.newTest('random', 'Random Uniform Block cases');
         testGroup.addChild(randomGroup);

         // Basic types.
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'scalar_types', 'Scalar types only, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused, 25, 0);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'vector_types', 'Scalar and vector types only, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | glsRandomUniformBlockCase.FeatureBits.FEATURE_VECTORS, 25, 25);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'basic_types', 'All basic types, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags, 25, 50);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'basic_arrays', 'Arrays, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags | glsRandomUniformBlockCase.FeatureBits.FEATURE_ARRAYS, 25, 50);

         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'basic_instance_arrays', 'Basic instance arrays, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags | glsRandomUniformBlockCase.FeatureBits.FEATURE_INSTANCE_ARRAYS, 25, 75);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'nested_structs', 'Nested structs, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags | glsRandomUniformBlockCase.FeatureBits.FEATURE_STRUCTS, 25, 100);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'nested_structs_arrays', 'Nested structs, arrays, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags | glsRandomUniformBlockCase.FeatureBits.FEATURE_STRUCTS | glsRandomUniformBlockCase.FeatureBits.FEATURE_ARRAYS, 25, 150);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'nested_structs_instance_arrays', 'Nested structs, instance arrays, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags | glsRandomUniformBlockCase.FeatureBits.FEATURE_STRUCTS | glsRandomUniformBlockCase.FeatureBits.FEATURE_INSTANCE_ARRAYS, 25, 125);
         es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'nested_structs_arrays_instance_arrays', 'Nested structs, instance arrays, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allShaders | allLayouts | unused | allBasicTypes | matFlags | glsRandomUniformBlockCase.FeatureBits.FEATURE_STRUCTS | glsRandomUniformBlockCase.FeatureBits.FEATURE_ARRAYS | glsRandomUniformBlockCase.FeatureBits.FEATURE_INSTANCE_ARRAYS, 25, 175);

         // Disabled: WebGL does not support shared or packed uniform buffers.
         //es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'all_per_block_buffers', 'All random features, per-block buffers', glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK, allFeatures, 50, 200);
         //es3fUniformBlockTests.createRandomCaseGroup(randomGroup, 'all_shared_buffer', 'All random features, shared buffer', glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE, allFeatures, 50, 250);
         bufferedLogToConsole('ubo.random: Tests created');
    };

    /**
     * Create and execute the test cases
     */
    es3fUniformBlockTests.run = function(range) {
        //Set up Test Root parameters
        var testName = 'ubo';
        var testDescription = 'Uniform Block Tests';
        var state = tcuTestCase.runner;

        state.setRoot(tcuTestCase.newTest(testName, testDescription, null));

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fUniformBlockTests.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fUniformBlockTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
