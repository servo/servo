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
goog.provide('functional.gles3.es3fVertexArrayTests');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluVarType');
goog.require('modules.shared.glsVertexArrayTests');

goog.scope(function() {

    var es3fVertexArrayTests = functional.gles3.es3fVertexArrayTests;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluTexture = framework.opengl.gluTexture;
    var gluVarType = framework.opengl.gluVarType;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var deRandom = framework.delibs.debase.deRandom;
    var deUtil = framework.delibs.debase.deUtil;
    var glsVertexArrayTests = modules.shared.glsVertexArrayTests;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * es3fVertexArrayTests.SingleVertexArrayUsageGroup
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsVertexArrayTests.deArray.Usage} usage
     */
    es3fVertexArrayTests.SingleVertexArrayUsageGroup = function(usage) {
        tcuTestCase.DeqpTest.call(
            this,
            "single_attribute.usages." + glsVertexArrayTests.deArray.usageTypeToString(usage),
            glsVertexArrayTests.deArray.usageTypeToString(usage)
        );
        this.makeExecutable();
        this.m_usage = usage;
    };

    es3fVertexArrayTests.SingleVertexArrayUsageGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayUsageGroup.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayUsageGroup;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayUsageGroup.prototype.init = function() {
        /** @type {Array<number>} */ var counts = [1, 256];
        /** @type {Array<number>} */ var strides = [0, -1, 17, 32]; // Treat negative value as sizeof input. Same as 0, but done outside of GL.
        /** @type {Array<glsVertexArrayTests.deArray.InputType>} */ var inputTypes = [
            glsVertexArrayTests.deArray.InputType.FLOAT,
            /*glsVertexArrayTests.deArray.InputType.FIXED,*/
            glsVertexArrayTests.deArray.InputType.SHORT,
            glsVertexArrayTests.deArray.InputType.BYTE
        ];

        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++) {
            for (var countNdx = 0; countNdx < counts.length; countNdx++) {
                for (var strideNdx = 0; strideNdx < strides.length; strideNdx++) {
                    /** @type {number} */ var stride = (strides[strideNdx] < 0 ? glsVertexArrayTests.deArray.inputTypeSize(inputTypes[inputTypeNdx]) * 2 : strides[strideNdx]);
                    /** @type {boolean} */ var aligned = (stride % glsVertexArrayTests.deArray.inputTypeSize(inputTypes[inputTypeNdx])) == 0;
                    /** @type {string} */ var name = 'stride' + stride + '_' + glsVertexArrayTests.deArray.inputTypeToString(inputTypes[inputTypeNdx]) + '_quads' + counts[countNdx];

                    var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                        inputTypes[inputTypeNdx],
                        glsVertexArrayTests.deArray.OutputType.VEC2,
                        glsVertexArrayTests.deArray.Storage.BUFFER,
                        this.m_usage,
                        2,
                        0,
                        stride,
                        false,
                        glsVertexArrayTests.GLValue.getMinValue(inputTypes[inputTypeNdx]),
                                                                                                glsVertexArrayTests.GLValue.getMaxValue(inputTypes[inputTypeNdx])
                    );

                    var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();
                    spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
                    spec.drawCount = counts[countNdx];
                    spec.first = 0;
                    spec.arrays.push(arraySpec);

                    if (aligned)
                        this.addChild(
                            new glsVertexArrayTests.MultiVertexArrayTest(
                                spec, name, name
                            )
                        );
                }
            }
        }
    };

    /**
     * es3fVertexArrayTests.SingleVertexArrayStrideGroup
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    es3fVertexArrayTests.SingleVertexArrayStrideGroup = function(type) {
        tcuTestCase.DeqpTest.call(this, glsVertexArrayTests.deArray.inputTypeToString(type), glsVertexArrayTests.deArray.inputTypeToString(type));
        this.makeExecutable();
        this.m_type = type;
    };

    es3fVertexArrayTests.SingleVertexArrayStrideGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayStrideGroup.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayStrideGroup;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayStrideGroup.prototype.init = function() {
        /** @type {Array<glsVertexArrayTests.deArray.Storage>} */ var storages = [
            // User storage not supported in WebGL - glsVertexArrayTests.deArray.Storage.USER,
            glsVertexArrayTests.deArray.Storage.BUFFER
        ];
        var counts = [1, 256];
        var strides = [/*0,*/ -1, 17, 32]; // Treat negative value as sizeof input. Same as 0, but done outside of GL.

        for (var storageNdx = 0; storageNdx < storages.length; storageNdx++) {
            for (var componentCount = 2; componentCount < 5; componentCount++) {
                for (var countNdx = 0; countNdx < counts.length; countNdx++) {
                    for (var strideNdx = 0; strideNdx < strides.length; strideNdx++) {
                        /** @type {boolean} */ var packed = this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 || this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10;
                        /** @type {number} */ var stride = (strides[strideNdx] < 0) ? ((packed) ? (16) : (glsVertexArrayTests.deArray.inputTypeSize(this.m_type) * componentCount)) : (strides[strideNdx]);
                        /** @type {number} */ var alignment = (packed) ? (glsVertexArrayTests.deArray.inputTypeSize(this.m_type) * componentCount) : (glsVertexArrayTests.deArray.inputTypeSize(this.m_type));
                        /** @type {boolean} */ var bufferUnaligned = (storages[storageNdx] == glsVertexArrayTests.deArray.Storage.BUFFER) && (stride % alignment) != 0;

                        /** @type {string} */ var name = glsVertexArrayTests.deArray.storageToString(storages[storageNdx]) + '_stride' + stride + '_components' + componentCount + '_quads' + counts[countNdx];

                        if ((this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10) && componentCount != 4)
                            continue;

                        /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec} */ var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                            this.m_type,
                            glsVertexArrayTests.deArray.OutputType.VEC4,
                            storages[storageNdx],
                            glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                            componentCount,
                            0,
                            stride,
                            false,
                            glsVertexArrayTests.GLValue.getMinValue(this.m_type),
                            glsVertexArrayTests.GLValue.getMaxValue(this.m_type)
                        );

                        var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();

                        spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
                        spec.drawCount = counts[countNdx];
                        spec.first = 0;
                        spec.arrays.push(arraySpec);

                        if (!bufferUnaligned)
                            this.addChild(
                                new glsVertexArrayTests.MultiVertexArrayTest(
                                    spec, name, name
                                )
                            );
                    }
                }
            }
        }
    };

    /**
     * es3fVertexArrayTests.SingleVertexArrayStrideTests
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.SingleVertexArrayStrideTests = function() {
        tcuTestCase.DeqpTest.call(this, 'single_attribute.strides', 'Single stride vertex atribute');
        this.makeExecutable();
    };

    es3fVertexArrayTests.SingleVertexArrayStrideTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayStrideTests.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayStrideTests;

    es3fVertexArrayTests.SingleVertexArrayStrideTests.prototype.init = function() {
        /** @type {Array<glsVertexArrayTests.deArray.InputType>} */ var inputTypes = [
            glsVertexArrayTests.deArray.InputType.FLOAT,
            glsVertexArrayTests.deArray.InputType.SHORT,
            glsVertexArrayTests.deArray.InputType.BYTE,
            /*glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE,
            glsVertexArrayTests.deArray.InputType.FIXED,*/
            glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];

        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++)
            this.addChild(
                new es3fVertexArrayTests.SingleVertexArrayStrideGroup(
                    inputTypes[inputTypeNdx]
                )
            );
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    es3fVertexArrayTests.SingleVertexArrayFirstGroup = function(type) {
        tcuTestCase.DeqpTest.call(
            this,
            glsVertexArrayTests.deArray.inputTypeToString(type),
            glsVertexArrayTests.deArray.inputTypeToString(type)
        );
        this.makeExecutable();

        this.m_type = type;
    };

    es3fVertexArrayTests.SingleVertexArrayFirstGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayFirstGroup.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayFirstGroup;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayFirstGroup.prototype.init = function() {
        var counts = [5, 256];
        var firsts = [6, 24];
        var offsets = [1, 16, 17];
        var strides = [/*0,*/ -1, 17, 32]; // Tread negative value as sizeof input. Same as 0, but done outside of GL.

        for (var offsetNdx = 0; offsetNdx < offsets.length; offsetNdx++) {
            for (var countNdx = 0; countNdx < counts.length; countNdx++) {
                for (var strideNdx = 0; strideNdx < strides.length; strideNdx++) {
                    for (var firstNdx = 0; firstNdx < firsts.length; firstNdx++) {
                        var packed = this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10;
                        var componentCount = packed ? 4 : 2;
                        var stride = strides[strideNdx] < 0 ?
                            (packed ? 8 : (glsVertexArrayTests.deArray.inputTypeSize(this.m_type) * componentCount)) :
                            (strides[strideNdx]);
                        var alignment = packed ?
                            (glsVertexArrayTests.deArray.inputTypeSize(this.m_type) * componentCount) :
                            (glsVertexArrayTests.deArray.inputTypeSize(this.m_type));
                        var aligned = ((stride % alignment) == 0) &&
                            ((offsets[offsetNdx] % alignment) == 0);
                        var name = 'first' + firsts[firstNdx] + '_offset' + offsets[offsetNdx] + '_stride' + stride + '_quads' + counts[countNdx];

                        var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                            this.m_type,
                            glsVertexArrayTests.deArray.OutputType.VEC2,
                            glsVertexArrayTests.deArray.Storage.BUFFER,
                            glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                            componentCount,
                            offsets[offsetNdx],
                            stride,
                            false,
                            glsVertexArrayTests.GLValue.getMinValue(this.m_type),
                            glsVertexArrayTests.GLValue.getMaxValue(this.m_type)
                        );

                        var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();
                        spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
                        spec.drawCount = counts[countNdx];
                        spec.first = firsts[firstNdx];
                        spec.arrays.push(arraySpec);

                        if (aligned)
                            this.addChild(
                                new glsVertexArrayTests.MultiVertexArrayTest(
                                    spec, name, name
                                )
                            );
                    }
                }
            }
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.SingleVertexArrayFirstTests = function() {
        tcuTestCase.DeqpTest.call(this, 'single_attribute.first', 'Single vertex attribute, different first values to drawArrays');
        this.makeExecutable();
    };

    es3fVertexArrayTests.SingleVertexArrayFirstTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayFirstTests.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayFirstTests;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayFirstTests.prototype.init = function() {
        // Test offset with different input types, component counts and storage, Usage(?)
        var inputTypes = [
            glsVertexArrayTests.deArray.InputType.FLOAT,
            glsVertexArrayTests.deArray.InputType.BYTE,
            glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];

        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++) {
            this.addChild(
                new es3fVertexArrayTests.SingleVertexArrayFirstGroup(
                    inputTypes[inputTypeNdx]
                )
            );
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    es3fVertexArrayTests.SingleVertexArrayOffsetGroup = function(type) {
        tcuTestCase.DeqpTest.call(
            this,
            glsVertexArrayTests.deArray.inputTypeToString(type),
            glsVertexArrayTests.deArray.inputTypeToString(type)
        );
        this.makeExecutable();
        this.m_type = type;
    };

    es3fVertexArrayTests.SingleVertexArrayOffsetGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayOffsetGroup.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayOffsetGroup;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayOffsetGroup.prototype.init = function() {
        var counts = [1, 256];
        var offsets = [1, 4, 17, 32];
        var strides = [/*0,*/ -1, 17, 32]; // Tread negative value as sizeof input. Same as 0, but done outside of GL.

        for (var offsetNdx = 0; offsetNdx < offsets.length; offsetNdx++) {
            for (var countNdx = 0; countNdx < counts.length; countNdx++) {
                for (var strideNdx = 0; strideNdx < strides.length; strideNdx++) {
                    var packed = this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 ||
                        this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10;
                    var componentCount = packed ? 4 : 2;
                    var stride = (
                        strides[strideNdx] < 0 ?
                        glsVertexArrayTests.deArray.inputTypeSize(
                            this.m_type
                        ) * componentCount :
                        strides[strideNdx]
                    );
                    var alignment = packed ?
                        glsVertexArrayTests.deArray.inputTypeSize(this.m_type) * componentCount :
                        glsVertexArrayTests.deArray.inputTypeSize(this.m_type);

                    var aligned = ((stride % alignment) == 0) &&
                        ((offsets[offsetNdx] % alignment) == 0);
                    var name = 'offset' + offsets[offsetNdx] +
                        '_stride' + stride + '_quads' +
                        counts[countNdx];

                    /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec} */ var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                        this.m_type,
                        glsVertexArrayTests.deArray.OutputType.VEC2,
                        glsVertexArrayTests.deArray.Storage.BUFFER,
                        glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                        componentCount,
                        offsets[offsetNdx],
                        stride,
                        false,
                        glsVertexArrayTests.GLValue.getMinValue(this.m_type),
                        glsVertexArrayTests.GLValue.getMaxValue(this.m_type)
                    );

                    var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();
                    spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
                    spec.drawCount = counts[countNdx];
                    spec.first = 0;
                    spec.arrays.push(arraySpec);

                    if (aligned)
                        this.addChild(
                            new glsVertexArrayTests.MultiVertexArrayTest(
                                spec, name, name
                            )
                        );
                }
            }
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.SingleVertexArrayOffsetTests = function() {
        tcuTestCase.DeqpTest.call(this, 'single_attribute.offset', 'Single vertex atribute offset element');
        this.makeExecutable();
    };

    es3fVertexArrayTests.SingleVertexArrayOffsetTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayOffsetTests.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayOffsetTests;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayOffsetTests.prototype.init = function() {
        // Test offset with different input types, component counts and storage, Usage(?)
        var inputTypes = [
            glsVertexArrayTests.deArray.InputType.FLOAT,
            glsVertexArrayTests.deArray.InputType.BYTE,
            glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];

        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++) {
            this.addChild(
                new es3fVertexArrayTests.SingleVertexArrayOffsetGroup(
                    inputTypes[inputTypeNdx]
                )
            );
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    es3fVertexArrayTests.SingleVertexArrayNormalizeGroup = function(type) {
        tcuTestCase.DeqpTest.call(
            this,
            glsVertexArrayTests.deArray.inputTypeToString(type),
            glsVertexArrayTests.deArray.inputTypeToString(type)
        );
        this.makeExecutable();
        this.m_type = type;
    };

    es3fVertexArrayTests.SingleVertexArrayNormalizeGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayNormalizeGroup.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayNormalizeGroup;

    /**
     * init for SingleVertexArrayNormalizeGroup
     */
    es3fVertexArrayTests.SingleVertexArrayNormalizeGroup.prototype.init = function() {
        var counts = [1, 256];

        for (var componentCount = 2; componentCount < 5; componentCount++) {
            for (var countNdx = 0; countNdx < counts.length; countNdx++) {
                if (
                    (
                        this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 ||
                        this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
                    ) && componentCount != 4
                )
                    continue;

                var name = 'components' + componentCount.toString() + '_quads' + counts[countNdx].toString();

                var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                    this.m_type,
                    glsVertexArrayTests.deArray.OutputType.VEC4,
                    glsVertexArrayTests.deArray.Storage.BUFFER, //No USER Storage support in WebGL2
                    glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                    componentCount,
                    0,
                    0,
                    true,
                    glsVertexArrayTests.GLValue.getMinValue(this.m_type),
                    glsVertexArrayTests.GLValue.getMaxValue(this.m_type)
                );

                var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();
                spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
                spec.drawCount = counts[countNdx];
                spec.first = 0;
                spec.arrays.push(arraySpec);

                this.addChild(
                    new glsVertexArrayTests.MultiVertexArrayTest(
                        spec, name, name
                    )
                );
            }
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.SingleVertexArrayNormalizeTests = function() {
        tcuTestCase.DeqpTest.call(this, 'single_attribute.normalize', 'Single normalize vertex atribute');
        this.makeExecutable();
    };

    es3fVertexArrayTests.SingleVertexArrayNormalizeTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayNormalizeTests.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayNormalizeTests;

    /**
     * init
     */
    es3fVertexArrayTests.SingleVertexArrayNormalizeTests.prototype.init = function() {
        // Test normalization with different input types, component counts and storage
        /** @type {Array<glsVertexArrayTests.deArray.InputType>} */ var inputTypes = [
            glsVertexArrayTests.deArray.InputType.FLOAT,
            glsVertexArrayTests.deArray.InputType.SHORT,
            glsVertexArrayTests.deArray.InputType.BYTE,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE,
            //glsVertexArrayTests.deArray.InputType.FIXED,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_INT,
            glsVertexArrayTests.deArray.InputType.INT,
            glsVertexArrayTests.deArray.InputType.HALF,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10,
            glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];

        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++) {
            this.addChild(
                new es3fVertexArrayTests.SingleVertexArrayNormalizeGroup(
                    inputTypes[inputTypeNdx]
                )
            );
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    es3fVertexArrayTests.SingleVertexArrayOutputTypeGroup = function(type) {
        tcuTestCase.DeqpTest.call(
            this,
            "single_attribute.output_types." + glsVertexArrayTests.deArray.inputTypeToString(type),
            glsVertexArrayTests.deArray.inputTypeToString(type)
        );
        this.makeExecutable();
        this.m_type = type;
    };

    es3fVertexArrayTests.SingleVertexArrayOutputTypeGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.SingleVertexArrayOutputTypeGroup.prototype.constructor = es3fVertexArrayTests.SingleVertexArrayOutputTypeGroup;

    es3fVertexArrayTests.SingleVertexArrayOutputTypeGroup.prototype.init = function() {
        var outputTypes = [
            glsVertexArrayTests.deArray.OutputType.VEC2,
            glsVertexArrayTests.deArray.OutputType.VEC3,
            glsVertexArrayTests.deArray.OutputType.VEC4,
            glsVertexArrayTests.deArray.OutputType.IVEC2,
            glsVertexArrayTests.deArray.OutputType.IVEC3,
            glsVertexArrayTests.deArray.OutputType.IVEC4,
            glsVertexArrayTests.deArray.OutputType.UVEC2,
            glsVertexArrayTests.deArray.OutputType.UVEC3,
            glsVertexArrayTests.deArray.OutputType.UVEC4
        ];
        var storages = [glsVertexArrayTests.deArray.Storage.BUFFER]; //No USER storage support in WebGL2
        var counts = [1, 256];

        for (var outputTypeNdx = 0; outputTypeNdx < outputTypes.length; outputTypeNdx++) {
            for (var storageNdx = 0; storageNdx < storages.length; storageNdx++) {
                for (var componentCount = 2; componentCount < 5; componentCount++) {
                    for (var countNdx = 0; countNdx < counts.length; countNdx++) {
                        var name = 'components' + componentCount + '_' +
                            glsVertexArrayTests.deArray.outputTypeToString(
                                outputTypes[outputTypeNdx]
                            ) +
                            '_quads' + counts[countNdx];

                        var inputIsSignedInteger =
                            this.m_type == glsVertexArrayTests.deArray.InputType.INT ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.SHORT ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.BYTE;

                        var inputIsUnignedInteger =
                            this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE;

                        var outputIsSignedInteger =
                            outputTypes[outputTypeNdx] == glsVertexArrayTests.deArray.OutputType.IVEC2 ||
                            outputTypes[outputTypeNdx] == glsVertexArrayTests.deArray.OutputType.IVEC3 ||
                            outputTypes[outputTypeNdx] == glsVertexArrayTests.deArray.OutputType.IVEC4;

                        var outputIsUnsignedInteger =
                            outputTypes[outputTypeNdx] == glsVertexArrayTests.deArray.OutputType.UVEC2 ||
                            outputTypes[outputTypeNdx] == glsVertexArrayTests.deArray.OutputType.UVEC3 ||
                            outputTypes[outputTypeNdx] == glsVertexArrayTests.deArray.OutputType.UVEC4;

                        // If input type is float type and output type is int type skip
                        if ((this.m_type == glsVertexArrayTests.deArray.InputType.FLOAT ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.HALF) &&
                            (outputTypes[outputTypeNdx] >= glsVertexArrayTests.deArray.OutputType.INT))
                            continue;

                        if ((this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10) &&
                            (outputTypes[outputTypeNdx] >= glsVertexArrayTests.deArray.OutputType.INT))
                            continue;

                        if ((this.m_type == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 ||
                            this.m_type == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10) &&
                            componentCount != 4)
                            continue;

                        // Loading signed data as unsigned causes undefined values and vice versa
                        if (inputIsSignedInteger && outputIsUnsignedInteger)
                            continue;
                        if (inputIsUnignedInteger && outputIsSignedInteger)
                            continue;

                        var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                            this.m_type,
                            outputTypes[outputTypeNdx],
                            storages[storageNdx],
                            glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                            componentCount,
                            0,
                            0,
                            false,
                            glsVertexArrayTests.GLValue.getMinValue(this.m_type),
                            glsVertexArrayTests.GLValue.getMaxValue(this.m_type)
                        );

                        var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();
                        spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
                        spec.drawCount = counts[countNdx];
                        spec.first = 0;
                        spec.arrays.push(arraySpec);

                        this.addChild(
                            new glsVertexArrayTests.MultiVertexArrayTest(
                                spec, name, name
                            )
                        );
                    }
                }
            }
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.MultiVertexArrayCountTests = function() {
        tcuTestCase.DeqpTest.call(this, 'multiple_attributes.attribute_count', 'Attribute counts');
        this.makeExecutable();
    };

    es3fVertexArrayTests.MultiVertexArrayCountTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.MultiVertexArrayCountTests.prototype.constructor = es3fVertexArrayTests.MultiVertexArrayCountTests;

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @return {string}
     */
    es3fVertexArrayTests.MultiVertexArrayCountTests.prototype.getTestName = function(spec) {
        var name = '';
        name += spec.arrays.length;

        return name;
    };

    es3fVertexArrayTests.MultiVertexArrayCountTests.prototype.init = function() {
        // Test attribute counts
        var arrayCounts = [2, 3, 4, 5, 6, 7, 8];

        for (var arrayCountNdx = 0; arrayCountNdx < arrayCounts.length; arrayCountNdx++) {
            var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();

            spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
            spec.drawCount = 256;
            spec.first = 0;

            for (var arrayNdx = 0; arrayNdx < arrayCounts[arrayCountNdx]; arrayNdx++) {
                var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                    glsVertexArrayTests.deArray.InputType.FLOAT,
                    glsVertexArrayTests.deArray.OutputType.VEC2,
                    glsVertexArrayTests.deArray.Storage.BUFFER, // No USER storage support in WebGL2
                    glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                    2,
                    0,
                    0,
                    false,
                    glsVertexArrayTests.GLValue.getMinValue(glsVertexArrayTests.deArray.InputType.FLOAT),
                    glsVertexArrayTests.GLValue.getMaxValue(glsVertexArrayTests.deArray.InputType.FLOAT)
                );
                spec.arrays.push(arraySpec);
            }

            var name = this.getTestName(spec);
            var desc = this.getTestName(spec);

            this.addChild(
                new glsVertexArrayTests.MultiVertexArrayTest(
                    spec, name, desc
                )
            );
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.MultiVertexArrayStorageTests = function() {
        tcuTestCase.DeqpTest.call(this, 'multiple_attributes.storage', 'Attribute storages');
        this.makeExecutable();
    };

    es3fVertexArrayTests.MultiVertexArrayStorageTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.MultiVertexArrayStorageTests.prototype.constructor = es3fVertexArrayTests.MultiVertexArrayStorageTests;

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @return {string}
     */
    es3fVertexArrayTests.MultiVertexArrayStorageTests.prototype.getTestName = function(spec) {
        var name = '';
        name += spec.arrays.length;

        for (var arrayNdx = 0; arrayNdx < spec.arrays.length; arrayNdx++)
            name += '_' + glsVertexArrayTests.deArray.storageToString(spec.arrays[arrayNdx].storage);

        return name;
    };

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @param {number} depth
     */
    es3fVertexArrayTests.MultiVertexArrayStorageTests.prototype.addStorageCases = function(spec, depth) {
        if (depth == 0) {
            // Skip trivial case, used elsewhere
            var ok = false;
            for (var arrayNdx = 0; arrayNdx < spec.arrays.length; arrayNdx++) {
                if (spec.arrays[arrayNdx].storage != glsVertexArrayTests.deArray.Storage.USER) {
                    ok = true;
                    break;
                }
            }

            if (!ok)
                return;

            var name = this.getTestName(spec);
            var desc = this.getTestName(spec);

            this.addChild(
                new glsVertexArrayTests.MultiVertexArrayTest(
                    spec, name, desc
                )
            );
            return;
        }

        var storages = [
            //glsVertexArrayTests.deArray.Storage.USER, Not supported in WebGL 2.0
            glsVertexArrayTests.deArray.Storage.BUFFER
        ];

        for (var storageNdx = 0; storageNdx < storages.length; storageNdx++) {
            var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                glsVertexArrayTests.deArray.InputType.FLOAT,
                glsVertexArrayTests.deArray.OutputType.VEC2,
                storages[storageNdx],
                glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                2,
                0,
                0,
                false,
                glsVertexArrayTests.GLValue.getMinValue(glsVertexArrayTests.deArray.InputType.FLOAT),
                glsVertexArrayTests.GLValue.getMaxValue(glsVertexArrayTests.deArray.InputType.FLOAT)
            );

            var _spec = spec;
            _spec.arrays.push(arraySpec);
            this.addStorageCases(_spec, depth - 1);
        }
    };

    /**
     * init
     */
    es3fVertexArrayTests.MultiVertexArrayStorageTests.prototype.init = function() {
        // Test different storages
        var arrayCounts = [3];

        var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();

        spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
        spec.drawCount = 256;
        spec.first = 0;

        for (var arrayCountNdx = 0; arrayCountNdx < arrayCounts.length; arrayCountNdx++)
            this.addStorageCases(spec, arrayCounts[arrayCountNdx]);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.MultiVertexArrayStrideTests = function() {
        tcuTestCase.DeqpTest.call(this, 'multiple_attributes.stride', 'Strides');
        this.makeExecutable();
    };

    es3fVertexArrayTests.MultiVertexArrayStrideTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.MultiVertexArrayStrideTests.prototype.constructor = es3fVertexArrayTests.MultiVertexArrayStrideTests;

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @return {string}
     */
    es3fVertexArrayTests.MultiVertexArrayStrideTests.prototype.getTestName = function(spec) {
        var name = '';

        name += spec.arrays.length;

        for (var arrayNdx = 0; arrayNdx < spec.arrays.length; arrayNdx++) {
            name += '_' +
            glsVertexArrayTests.deArray.inputTypeToString(spec.arrays[arrayNdx].inputType) +
            spec.arrays[arrayNdx].componentCount + '_' +
            spec.arrays[arrayNdx].stride;
        }

        return name;
    };

    /**
     * init
     */
    es3fVertexArrayTests.MultiVertexArrayStrideTests.prototype.init = function() {
        // Test different strides, with multiple arrays, input types??
        var arrayCounts = [3];

        var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();

        spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
        spec.drawCount = 256;
        spec.first = 0;

        for (var arrayCountNdx = 0; arrayCountNdx < arrayCounts.length; arrayCountNdx++)
            this.addStrideCases(spec, arrayCounts[arrayCountNdx]);
    };

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @param {number} depth
     */
    es3fVertexArrayTests.MultiVertexArrayStrideTests.prototype.addStrideCases = function(spec, depth) {
        if (depth == 0) {
            var name = this.getTestName(spec);
            var desc = this.getTestName(spec);
            this.addChild(
                new glsVertexArrayTests.MultiVertexArrayTest(
                    spec, name, desc
                )
            );
            return;
        }

        var strides = [0, -1, 17, 32];
        var inputType = glsVertexArrayTests.deArray.InputType.FLOAT;

        for (var strideNdx = 0; strideNdx < strides.length; strideNdx++) {
            var componentCount = 2;
            var stride = strides[strideNdx] >= 0 ? strides[strideNdx] : componentCount * glsVertexArrayTests.deArray.inputTypeSize(glsVertexArrayTests.deArray.InputType.FLOAT);
            var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                inputType,
                glsVertexArrayTests.deArray.OutputType.VEC2,
                glsVertexArrayTests.deArray.Storage.BUFFER, //USER storage not supported in WebGL 2.0
                glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                componentCount,
                0,
                stride,
                false,
                glsVertexArrayTests.GLValue.getMinValue(glsVertexArrayTests.deArray.InputType.FLOAT),
                glsVertexArrayTests.GLValue.getMaxValue(glsVertexArrayTests.deArray.InputType.FLOAT)
            );

            /** @type {boolean} */ var aligned = (stride % glsVertexArrayTests.deArray.inputTypeSize(inputType)) == 0;
            if (aligned) {
                var _spec = /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec} */ (deUtil.clone(spec)); //To assign spec by value;
                _spec.arrays.push(arraySpec);
                this.addStrideCases(_spec, depth - 1);
            }
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.MultiVertexArrayOutputTests = function() {
        tcuTestCase.DeqpTest.call(this, 'multiple_attributes.input_types', 'input types');
        this.makeExecutable();
    };

    es3fVertexArrayTests.MultiVertexArrayOutputTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.MultiVertexArrayOutputTests.prototype.constructor = es3fVertexArrayTests.MultiVertexArrayOutputTests;

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @return {string}
     */
    es3fVertexArrayTests.MultiVertexArrayOutputTests.prototype.getTestName = function(spec) {
        var name = '';

        name += spec.arrays.length;

        for (var arrayNdx = 0; arrayNdx < spec.arrays.length; arrayNdx++) {
            name += '_' +
            glsVertexArrayTests.deArray.inputTypeToString(spec.arrays[arrayNdx].inputType) +
            spec.arrays[arrayNdx].componentCount + '_' +
            glsVertexArrayTests.deArray.outputTypeToString(spec.arrays[arrayNdx].outputType);
        }

        return name;
    };

    /**
     * init
     */
    es3fVertexArrayTests.MultiVertexArrayOutputTests.prototype.init = function() {
        // Test different input types, with multiple arrays
        var arrayCounts = [3];

        var spec = new glsVertexArrayTests.MultiVertexArrayTest.Spec();

        spec.primitive = glsVertexArrayTests.deArray.Primitive.TRIANGLES;
        spec.drawCount = 256;
        spec.first = 0;

        for (var arrayCountNdx = 0; arrayCountNdx < arrayCounts.length; arrayCountNdx++)
            this.addInputTypeCases(spec, arrayCounts[arrayCountNdx]);
    };

    /**
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @param {number} depth
     */
    es3fVertexArrayTests.MultiVertexArrayOutputTests.prototype.addInputTypeCases = function(spec, depth) {
        if (depth == 0) {
            var name = this.getTestName(spec);
            var desc = this.getTestName(spec);
            this.addChild(
                new glsVertexArrayTests.MultiVertexArrayTest(
                    spec, name, desc
                )
            );
            return;
        }

        var inputTypes = [
            glsVertexArrayTests.deArray.InputType.BYTE,
            glsVertexArrayTests.deArray.InputType.SHORT,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT
        ];

        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++) {
            var arraySpec = new glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec(
                inputTypes[inputTypeNdx],
                glsVertexArrayTests.deArray.OutputType.VEC2,
                glsVertexArrayTests.deArray.Storage.BUFFER, //USER storage not supported in WebGL 2.0
                glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
                2,
                0,
                0,
                false,
                glsVertexArrayTests.GLValue.getMinValue(inputTypes[inputTypeNdx]),
                glsVertexArrayTests.GLValue.getMaxValue(inputTypes[inputTypeNdx])
            );

            var _spec = /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec} */ (deUtil.clone(spec));
            _spec.arrays.push(arraySpec);
            this.addInputTypeCases(_spec, depth - 1);
        }
    };

    /**
     * es3fVertexArrayTests.VertexArrayTestGroup
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fVertexArrayTests.VertexArrayTestGroup = function() {
        tcuTestCase.DeqpTest.call(this, 'vertex_arrays', 'Vertex array and array tests');
        this.makeExecutable();
    };

    es3fVertexArrayTests.VertexArrayTestGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fVertexArrayTests.VertexArrayTestGroup.prototype.constructor = es3fVertexArrayTests.VertexArrayTestGroup;

    /**
     * init
     */
    es3fVertexArrayTests.VertexArrayTestGroup.prototype.init = function() {
        this.addChild(new es3fVertexArrayTests.SingleVertexArrayStrideTests());
        this.addChild(new es3fVertexArrayTests.SingleVertexArrayNormalizeTests());

        // Test output types with different input types, component counts and storage, Usage?, Precision?, float?
        var inputTypes = [
            glsVertexArrayTests.deArray.InputType.FLOAT,
            glsVertexArrayTests.deArray.InputType.SHORT,
            glsVertexArrayTests.deArray.InputType.BYTE,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_INT,
            glsVertexArrayTests.deArray.InputType.INT,
            glsVertexArrayTests.deArray.InputType.HALF,
            glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10,
            glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];
        for (var inputTypeNdx = 0; inputTypeNdx < inputTypes.length; inputTypeNdx++) {
            this.addChild(new es3fVertexArrayTests.SingleVertexArrayOutputTypeGroup(inputTypes[inputTypeNdx]));
        }

        /** @type {Array<glsVertexArrayTests.deArray.Usage>} */ var usages = [
            glsVertexArrayTests.deArray.Usage.STATIC_DRAW,
            glsVertexArrayTests.deArray.Usage.STREAM_DRAW,
            glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW,
            glsVertexArrayTests.deArray.Usage.STATIC_COPY,
            glsVertexArrayTests.deArray.Usage.STREAM_COPY,
            glsVertexArrayTests.deArray.Usage.DYNAMIC_COPY,
            glsVertexArrayTests.deArray.Usage.STATIC_READ,
            glsVertexArrayTests.deArray.Usage.STREAM_READ,
            glsVertexArrayTests.deArray.Usage.DYNAMIC_READ
        ];
        for (var usageNdx = 0; usageNdx < usages.length; usageNdx++) {
            this.addChild(new es3fVertexArrayTests.SingleVertexArrayUsageGroup(usages[usageNdx]));
        }

        this.addChild(new es3fVertexArrayTests.SingleVertexArrayOffsetTests());
        this.addChild(new es3fVertexArrayTests.SingleVertexArrayFirstTests());

        this.addChild(new es3fVertexArrayTests.MultiVertexArrayCountTests());
        this.addChild(new es3fVertexArrayTests.MultiVertexArrayStorageTests());
        this.addChild(new es3fVertexArrayTests.MultiVertexArrayStrideTests());
        this.addChild(new es3fVertexArrayTests.MultiVertexArrayOutputTests());
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     */
    es3fVertexArrayTests.run = function(context, range) {
        gl = context;
        //Set up root Test
        var state = tcuTestCase.runner;

        var test = new es3fVertexArrayTests.VertexArrayTestGroup();
        var testName = test.fullName();
        var testDescription = test.getDescription();
        state.testCases = test;
        state.testName = testName;

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        } catch (err) {
            testFailedOptions('Failed to es3fVertexArrayTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
