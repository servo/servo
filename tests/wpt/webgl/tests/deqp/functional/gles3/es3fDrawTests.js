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
goog.provide('functional.gles3.es3fDrawTests');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuRGBA');
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
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');
goog.require('modules.shared.glsDrawTests');

goog.scope(function() {

    var es3fDrawTests = functional.gles3.es3fDrawTests;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluTexture = framework.opengl.gluTexture;
    var gluVarType = framework.opengl.gluVarType;
    var tcuLogImage = framework.common.tcuLogImage;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuTexture = framework.common.tcuTexture;
    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var deRandom = framework.delibs.debase.deRandom;
    var deUtil = framework.delibs.debase.deUtil;
    var glsDrawTests = modules.shared.glsDrawTests;
    var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;
    var rrShadingContext = framework.referencerenderer.rrShadingContext;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
    var rrVertexPacket = framework.referencerenderer.rrVertexPacket;

    /** @type {WebGL2RenderingContext}*/ var gl;

    /**
     * @enum
     */
    es3fDrawTests.TestIterationType = {
        DRAW_COUNT: 0, // !< test with 2, 6, and 26 primitives
        INSTANCE_COUNT: 1, // !< test with 2, 4, and 12 instances
        INDEX_RANGE: 2
    };

    /**
     * @param {glsDrawTests.DrawTest} test
     * @param {glsDrawTests.DrawTestSpec} baseSpec
     * @param {?es3fDrawTests.TestIterationType} type
     */
    es3fDrawTests.addTestIterations = function(test, baseSpec, type) {
        var spec = /** @type {glsDrawTests.DrawTestSpec} */ (deUtil.clone(baseSpec));

        if (type == es3fDrawTests.TestIterationType.DRAW_COUNT) {
            spec.primitiveCount = 1;
            test.addIteration(spec, 'draw count = ' + spec.primitiveCount);

            spec.primitiveCount = 5;
            test.addIteration(spec, 'draw count = ' + spec.primitiveCount);

            spec.primitiveCount = 25;
            test.addIteration(spec, 'draw count = ' + spec.primitiveCount);
        } else if (type == es3fDrawTests.TestIterationType.INSTANCE_COUNT) {
            spec.instanceCount = 1;
            test.addIteration(spec, 'instance count = ' + spec.instanceCount);

            spec.instanceCount = 4;
            test.addIteration(spec, 'instance count = ' + spec.instanceCount);

            spec.instanceCount = 11;
            test.addIteration(spec, 'instance count = ' + spec.instanceCount);
        } else if (type == es3fDrawTests.TestIterationType.INDEX_RANGE) {
            spec.indexMin = 0;
            spec.indexMax = 23;
            test.addIteration(spec, 'index range = [' + spec.indexMin + ', ' + spec.indexMax + ']');

            spec.indexMin = 23;
            spec.indexMax = 40;
            test.addIteration(spec, 'index range = [' + spec.indexMin + ', ' + spec.indexMax + ']');

            // Only makes sense with points
            if (spec.primitive == glsDrawTests.DrawTestSpec.Primitive.POINTS) {
                spec.indexMin = 5;
                spec.indexMax = 5;
                test.addIteration(spec, 'index range = [' + spec.indexMin + ', ' + spec.indexMax + ']');
            }
        } else
            throw new Error('Invalid test iteration type');
    };

    /**
     * @param {glsDrawTests.DrawTestSpec} spec
     * @param {?glsDrawTests.DrawTestSpec.DrawMethod} method
     */
    es3fDrawTests.genBasicSpec = function(spec, method) {
        //spec.apiType = glu::ApiType::es(3,0);
        spec.primitive = glsDrawTests.DrawTestSpec.Primitive.TRIANGLES;
        spec.primitiveCount = 6;
        spec.drawMethod = method;
        spec.indexType = null;
        spec.indexPointerOffset = 0;
        spec.indexStorage = null;
        spec.first = 0;
        spec.indexMin = 0;
        spec.indexMax = 0;
        spec.instanceCount = 1;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());

        spec.attribs[0].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[0].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[0].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[0].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[0].componentCount = 4;
        spec.attribs[0].offset = 0;
        spec.attribs[0].stride = 0;
        spec.attribs[0].normalize = false;
        spec.attribs[0].instanceDivisor = 0;
        spec.attribs[0].useDefaultAttribute = false;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());

        spec.attribs[1].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[1].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[1].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[1].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[1].componentCount = 2;
        spec.attribs[1].offset = 0;
        spec.attribs[1].stride = 0;
        spec.attribs[1].normalize = false;
        spec.attribs[1].instanceDivisor = 0;
        spec.attribs[1].useDefaultAttribute = false;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} descr
     * @param {?glsDrawTests.DrawTestSpec.DrawMethod} drawMethod
     * @param {?glsDrawTests.DrawTestSpec.Primitive} primitive
     * @param {?glsDrawTests.DrawTestSpec.IndexType} indexType
     * @param {?glsDrawTests.DrawTestSpec.Storage} indexStorage
     */
    es3fDrawTests.AttributeGroup = function(name, descr, drawMethod, primitive, indexType, indexStorage) {
        tcuTestCase.DeqpTest.call(this, name, descr);
        this.m_method = drawMethod;
        this.m_primitive = primitive;
        this.m_indexType = indexType;
        this.m_indexStorage = indexStorage;
        this.makeExecutable();
    };

    es3fDrawTests.AttributeGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.AttributeGroup.prototype.constructor = es3fDrawTests.AttributeGroup;

    es3fDrawTests.AttributeGroup.prototype.init = function() {
        // select test type
        /** @type {boolean} */ var instanced = this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS_INSTANCED ||
            this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_INSTANCED;
        /** @type {boolean} */ var ranged = this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED;
        /** @type {es3fDrawTests.TestIterationType} */ var testType = instanced ? es3fDrawTests.TestIterationType.INSTANCE_COUNT :
            (ranged ? es3fDrawTests.TestIterationType.INDEX_RANGE : es3fDrawTests.TestIterationType.DRAW_COUNT);

        // Single attribute
        /** @type {glsDrawTests.DrawTest} */ var test = new glsDrawTests.DrawTest(null, 'single_attribute', 'Single attribute array.');
        /** @type {glsDrawTests.DrawTestSpec} */ var spec = new glsDrawTests.DrawTestSpec();

        //spec.apiType = glu::ApiType::es(3,0);
        spec.primitive = this.m_primitive;
        spec.primitiveCount = 5;
        spec.drawMethod = this.m_method;
        spec.indexType = this.m_indexType;
        spec.indexPointerOffset = 0;
        spec.indexStorage = this.m_indexStorage;
        spec.first = 0;
        spec.indexMin = 0;
        spec.indexMax = 0;
        spec.instanceCount = 1;

        spec.attribs.length = 0;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[0].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[0].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[0].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[0].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[0].componentCount = 2;
        spec.attribs[0].offset = 0;
        spec.attribs[0].stride = 0;
        spec.attribs[0].normalize = false;
        spec.attribs[0].instanceDivisor = 0;
        spec.attribs[0].useDefaultAttribute = false;

        es3fDrawTests.addTestIterations(test, spec, testType);

        this.addChild(test);

        // Multiple attribute

        test = new glsDrawTests.DrawTest(null, 'multiple_attributes', 'Multiple attribute arrays.');
        spec.primitive = this.m_primitive;
        spec.primitiveCount = 5;
        spec.drawMethod = this.m_method;
        spec.indexType = this.m_indexType;
        spec.indexPointerOffset = 0;
        spec.indexStorage = this.m_indexStorage;
        spec.first = 0;
        spec.indexMin = 0;
        spec.indexMax = 0;
        spec.instanceCount = 1;

        spec.attribs.length = 0;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[0].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[0].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[0].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[0].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[0].componentCount = 4;
        spec.attribs[0].offset = 0;
        spec.attribs[0].stride = 0;
        spec.attribs[0].normalize = false;
        spec.attribs[0].instanceDivisor = 0;
        spec.attribs[0].useDefaultAttribute = false;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[1].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[1].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[1].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[1].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[1].componentCount = 2;
        spec.attribs[1].offset = 0;
        spec.attribs[1].stride = 0;
        spec.attribs[1].normalize = false;
        spec.attribs[1].instanceDivisor = 0;
        spec.attribs[1].useDefaultAttribute = false;

        es3fDrawTests.addTestIterations(test, spec, testType);

        this.addChild(test);

        // Multiple attribute, second one divided

        test = new glsDrawTests.DrawTest(null, 'instanced_attributes', 'Instanced attribute array.');

        //spec.apiType = glu::ApiType::es(3,0);
        spec.primitive = this.m_primitive;
        spec.primitiveCount = 5;
        spec.drawMethod = this.m_method;
        spec.indexType = this.m_indexType;
        spec.indexPointerOffset = 0;
        spec.indexStorage = this.m_indexStorage;
        spec.first = 0;
        spec.indexMin = 0;
        spec.indexMax = 0;
        spec.instanceCount = 1;

        spec.attribs.length = 0;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[0].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[0].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[0].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[0].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[0].componentCount = 4;
        spec.attribs[0].offset = 0;
        spec.attribs[0].stride = 0;
        spec.attribs[0].normalize = false;
        spec.attribs[0].instanceDivisor = 0;
        spec.attribs[0].useDefaultAttribute = false;

        // Add another position component so the instances wont be drawn on each other
        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[1].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[1].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[1].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[1].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[1].componentCount = 2;
        spec.attribs[1].offset = 0;
        spec.attribs[1].stride = 0;
        spec.attribs[1].normalize = false;
        spec.attribs[1].instanceDivisor = 1;
        spec.attribs[1].useDefaultAttribute = false;
        spec.attribs[1].additionalPositionAttribute = true;

        // Instanced color
        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[2].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[2].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[2].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[2].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[2].componentCount = 3;
        spec.attribs[2].offset = 0;
        spec.attribs[2].stride = 0;
        spec.attribs[2].normalize = false;
        spec.attribs[2].instanceDivisor = 1;
        spec.attribs[2].useDefaultAttribute = false;

        es3fDrawTests.addTestIterations(test, spec, testType);

        this.addChild(test);

        // Multiple attribute, second one default
        test = new glsDrawTests.DrawTest(null, 'default_attribute', 'Attribute specified with glVertexAttrib*.');

        //spec.apiType = glu::ApiType::es(3,0);
        spec.primitive = this.m_primitive;
        spec.primitiveCount = 5;
        spec.drawMethod = this.m_method;
        spec.indexType = this.m_indexType;
        spec.indexPointerOffset = 0;
        spec.indexStorage = this.m_indexStorage;
        spec.first = 0;
        spec.indexMin = 0;
        spec.indexMax = 17; // \note addTestIterations is not called for the spec, so we must ensure [indexMin, indexMax] is a good range
        spec.instanceCount = 1;

        spec.attribs.length = 0;

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        spec.attribs[0].inputType = glsDrawTests.DrawTestSpec.InputType.FLOAT;
        spec.attribs[0].outputType = glsDrawTests.DrawTestSpec.OutputType.VEC2;
        spec.attribs[0].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
        spec.attribs[0].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
        spec.attribs[0].componentCount = 2;
        spec.attribs[0].offset = 0;
        spec.attribs[0].stride = 0;
        spec.attribs[0].normalize = false;
        spec.attribs[0].instanceDivisor = 0;
        spec.attribs[0].useDefaultAttribute = false;

        /** @type {Array<{input:?glsDrawTests.DrawTestSpec.InputType, output:?glsDrawTests.DrawTestSpec.OutputType, componentCount:number}>} */ var iopairs = [
            {input: glsDrawTests.DrawTestSpec.InputType.FLOAT, output: glsDrawTests.DrawTestSpec.OutputType.VEC2, componentCount: 4},
            {input: glsDrawTests.DrawTestSpec.InputType.FLOAT, output: glsDrawTests.DrawTestSpec.OutputType.VEC4, componentCount: 2},
            {input: glsDrawTests.DrawTestSpec.InputType.INT, output: glsDrawTests.DrawTestSpec.OutputType.IVEC3, componentCount: 4},
            {input: glsDrawTests.DrawTestSpec.InputType.UNSIGNED_INT, output: glsDrawTests.DrawTestSpec.OutputType.UVEC2, componentCount: 4}
        ];

        spec.attribs.push(new glsDrawTests.DrawTestSpec.AttributeSpec());
        for (var ioNdx = 0; ioNdx < iopairs.length; ++ioNdx) {
            /** @type {string} */ var desc = glsDrawTests.DrawTestSpec.inputTypeToString(iopairs[ioNdx].input) + iopairs[ioNdx].componentCount + ' to ' + glsDrawTests.DrawTestSpec.outputTypeToString(iopairs[ioNdx].output);

            spec.attribs[1].inputType = iopairs[ioNdx].input;
            spec.attribs[1].outputType = iopairs[ioNdx].output;
            spec.attribs[1].storage = glsDrawTests.DrawTestSpec.Storage.BUFFER;
            spec.attribs[1].usage = glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW;
            spec.attribs[1].componentCount = iopairs[ioNdx].componentCount;
            spec.attribs[1].offset = 0;
            spec.attribs[1].stride = 0;
            spec.attribs[1].normalize = false;
            spec.attribs[1].instanceDivisor = 0;
            spec.attribs[1].useDefaultAttribute = true;

            test.addIteration(spec, desc);
        }

        this.addChild(test);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} descr
     * @param {?glsDrawTests.DrawTestSpec.DrawMethod} drawMethod
     */
    es3fDrawTests.IndexGroup = function(name, descr, drawMethod) {
        tcuTestCase.DeqpTest.call(this, name, descr);
        /** @type {?glsDrawTests.DrawTestSpec.DrawMethod} */ this.m_method = drawMethod;
        this.makeExecutable();
    };

    es3fDrawTests.IndexGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.IndexGroup.prototype.constructor = es3fDrawTests.IndexGroup;

    es3fDrawTests.IndexGroup.prototype.init = function() {
        /** @type {Array<{storage: ?glsDrawTests.DrawTestSpec.Storage, type: ?glsDrawTests.DrawTestSpec.IndexType, aligned: boolean, offsets: Array<number>}>} */ var tests = [
            {storage: glsDrawTests.DrawTestSpec.Storage.BUFFER, type: glsDrawTests.DrawTestSpec.IndexType.BYTE, aligned: true, offsets: [0, 1, -1]},
            {storage: glsDrawTests.DrawTestSpec.Storage.BUFFER, type: glsDrawTests.DrawTestSpec.IndexType.SHORT, aligned: true, offsets: [0, 2, -1]},
            {storage: glsDrawTests.DrawTestSpec.Storage.BUFFER, type: glsDrawTests.DrawTestSpec.IndexType.INT, aligned: true, offsets: [0, 4, -1]},

            {storage: glsDrawTests.DrawTestSpec.Storage.BUFFER, type: glsDrawTests.DrawTestSpec.IndexType.SHORT, aligned: false, offsets: [1, 3, -1]},
            {storage: glsDrawTests.DrawTestSpec.Storage.BUFFER, type: glsDrawTests.DrawTestSpec.IndexType.INT, aligned: false, offsets: [2, 3, -1]}
        ];

        /** @type {glsDrawTests.DrawTestSpec} */ var spec = new glsDrawTests.DrawTestSpec();
        es3fDrawTests.genBasicSpec(spec, this.m_method);

        /** @type {tcuTestCase.DeqpTest} */ var bufferGroup = new tcuTestCase.DeqpTest('buffer', 'buffer');
        /** @type {tcuTestCase.DeqpTest} */ var unalignedBufferGroup = new tcuTestCase.DeqpTest('unaligned_buffer', 'unaligned buffer');

        this.addChild(bufferGroup);
        this.addChild(unalignedBufferGroup);

        for (var testNdx = 0; testNdx < tests.length; ++testNdx) {
            /** @type {{storage: ?glsDrawTests.DrawTestSpec.Storage, type: ?glsDrawTests.DrawTestSpec.IndexType, aligned: boolean, offsets: Array<number>}} */
            var indexTest = tests[testNdx];
            /** @type {tcuTestCase.DeqpTest} */ var group = indexTest.aligned ? bufferGroup : unalignedBufferGroup;

            /** @type {string} */ var name = 'index_' + glsDrawTests.DrawTestSpec.indexTypeToString(indexTest.type);
            /** @type {string} */ var desc = 'index ' + glsDrawTests.DrawTestSpec.indexTypeToString(indexTest.type) + ' in ' + glsDrawTests.DrawTestSpec.storageToString(indexTest.storage);
            /** @type {glsDrawTests.DrawTest} */ var test = new glsDrawTests.DrawTest(null, name, desc);

            spec.indexType = indexTest.type;
            spec.indexStorage = indexTest.storage;

            for (var iterationNdx = 0; iterationNdx < indexTest.offsets.length && indexTest.offsets[iterationNdx] != -1; ++iterationNdx) {
                /** @type {string} */ var iterationDesc = 'offset ' + indexTest.offsets[iterationNdx];
                spec.indexPointerOffset = indexTest.offsets[iterationNdx];
                test.addIteration(spec, iterationDesc);
            }

            if (spec.isCompatibilityTest() != glsDrawTests.DrawTestSpec.CompatibilityTestType.UNALIGNED_OFFSET &&
                spec.isCompatibilityTest() != glsDrawTests.DrawTestSpec.CompatibilityTestType.UNALIGNED_STRIDE)
                group.addChild(test);
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} descr
     * @param {?glsDrawTests.DrawTestSpec.DrawMethod} drawMethod
     */
    es3fDrawTests.FirstGroup = function(name, descr, drawMethod) {
        tcuTestCase.DeqpTest.call(this, name, descr);
        /** @type {?glsDrawTests.DrawTestSpec.DrawMethod} */ this.m_method = drawMethod;
        this.makeExecutable();
    };

    es3fDrawTests.FirstGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.FirstGroup.prototype.constructor = es3fDrawTests.FirstGroup;

    /**
     * init
     */
    es3fDrawTests.FirstGroup.prototype.init = function() {
        var firsts =
        [
            1, 3, 17
        ];

        /** @type {glsDrawTests.DrawTestSpec} */ var spec = new glsDrawTests.DrawTestSpec();
        es3fDrawTests.genBasicSpec(spec, this.m_method);

        for (var firstNdx = 0; firstNdx < firsts.length; ++firstNdx) {
            var name = 'first_' + firsts[firstNdx];
            var desc = 'first ' + firsts[firstNdx];
            /** @type {glsDrawTests.DrawTest} */ var test = new glsDrawTests.DrawTest(null, name, desc);

            spec.first = firsts[firstNdx];

            es3fDrawTests.addTestIterations(test, spec, es3fDrawTests.TestIterationType.DRAW_COUNT);

            this.addChild(test);
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} descr
     * @param {?glsDrawTests.DrawTestSpec.DrawMethod} drawMethod
     */
    es3fDrawTests.MethodGroup = function(name, descr, drawMethod) {
        tcuTestCase.DeqpTest.call(this, name, descr);
        /** @type {?glsDrawTests.DrawTestSpec.DrawMethod} */ this.m_method = drawMethod;
        this.makeExecutable();
    };

    es3fDrawTests.MethodGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.MethodGroup.prototype.constructor = es3fDrawTests.MethodGroup;

    /**
     * init
     */
    es3fDrawTests.MethodGroup.prototype.init = function() {
        var indexed = (this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS) || (this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_INSTANCED) || (this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED);
        var hasFirst = (this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS) || (this.m_method == glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS_INSTANCED);

        var primitive =
        [
            glsDrawTests.DrawTestSpec.Primitive.POINTS,
            glsDrawTests.DrawTestSpec.Primitive.TRIANGLES,
            glsDrawTests.DrawTestSpec.Primitive.TRIANGLE_FAN,
            glsDrawTests.DrawTestSpec.Primitive.TRIANGLE_STRIP,
            glsDrawTests.DrawTestSpec.Primitive.LINES,
            glsDrawTests.DrawTestSpec.Primitive.LINE_STRIP,
            glsDrawTests.DrawTestSpec.Primitive.LINE_LOOP
        ];

        if (hasFirst) {
            // First-tests
            this.addChild(new es3fDrawTests.FirstGroup('first', 'First tests', this.m_method));
        }

        if (indexed) {
            // Index-tests
            if (this.m_method != glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED)
                this.addChild(new es3fDrawTests.IndexGroup('indices', 'Index tests', this.m_method));
        }

        for (var ndx = 0; ndx < primitive.length; ++ndx) {
            var name = glsDrawTests.DrawTestSpec.primitiveToString(primitive[ndx]);
            var desc = glsDrawTests.DrawTestSpec.primitiveToString(primitive[ndx]);

            this.addChild(new es3fDrawTests.AttributeGroup(name, desc, this.m_method, primitive[ndx], glsDrawTests.DrawTestSpec.IndexType.SHORT, glsDrawTests.DrawTestSpec.Storage.BUFFER));
        }
    };

    /**
     * es3fDrawTests.GridProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     */
    es3fDrawTests.GridProgram = function() {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */ var decl = new sglrShaderProgram.ShaderProgramDeclaration();

        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_offset', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_color', rrGenericVector.GenericVecType.FLOAT));

        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(rrGenericVector.GenericVecType.FLOAT));

        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in highp vec4 a_offset;\n' +
            'in highp vec4 a_color;\n' +
            'out mediump vec4 v_color;\n' +
            'void main(void)\n' +
            '{\n' +
            ' gl_Position = a_position + a_offset;\n' +
            ' v_color = a_color;\n' +
            '}\n'
        ));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
            '#version 300 es\n' +
            'layout(location = 0) out mediump vec4 dEQP_FragColor;\n' +
            'in mediump vec4 v_color;\n' +
            'void main(void)\n' +
            '{\n' +
            ' dEQP_FragColor = v_color;\n' +
            '}\n'
        ));

        sglrShaderProgram.ShaderProgram.call(this, decl);
    };

    es3fDrawTests.GridProgram.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fDrawTests.GridProgram.prototype.constructor = es3fDrawTests.GridProgram;

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fDrawTests.GridProgram.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var ndx = 0; ndx < numPackets; ++ndx) {
            packets[ndx].position = deMath.add(
                rrVertexAttrib.readVertexAttrib(inputs[0], packets[ndx].instanceNdx, packets[ndx].vertexNdx, rrGenericVector.GenericVecType.FLOAT),
                rrVertexAttrib.readVertexAttrib(inputs[1], packets[ndx].instanceNdx, packets[ndx].vertexNdx, rrGenericVector.GenericVecType.FLOAT)
            );
            packets[ndx].outputs[0] = rrVertexAttrib.readVertexAttrib(inputs[2], packets[ndx].instanceNdx, packets[ndx].vertexNdx, rrGenericVector.GenericVecType.FLOAT);
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packets
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fDrawTests.GridProgram.prototype.shadeFragments = function(packets, context) {
        for (var packetNdx = 0; packetNdx < packets.length; ++packetNdx)
        for (var fragNdx = 0; fragNdx < 4; ++fragNdx)
            packets[packetNdx].value = rrShadingContext.readTriangleVarying(packets[packetNdx], context, fragNdx);
    };

    /**
     * InstancedGridRenderTest
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} gridSide
     * @param {boolean} useIndices
     */
    es3fDrawTests.InstancedGridRenderTest = function(name, desc, gridSide, useIndices) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_gridSide = gridSide;
        this.m_useIndices = useIndices;
    };

    es3fDrawTests.InstancedGridRenderTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.InstancedGridRenderTest.prototype.constructor = es3fDrawTests.InstancedGridRenderTest;

    /**
     * iterate
     * @return {tcuTestCase.IterateResult}
     */
    es3fDrawTests.InstancedGridRenderTest.prototype.iterate = function() {
        var renderTargetWidth = Math.min(1024, gl.canvas.width);
        var renderTargetHeight = Math.min(1024, gl.canvas.height);

        /** @type {sglrGLContext.GLContext} */ var ctx = new sglrGLContext.GLContext(gl);
        /** @type {tcuSurface.Surface} */ var surface = new tcuSurface.Surface(renderTargetWidth, renderTargetHeight);
        /** @type {es3fDrawTests.GridProgram} */ var program = new es3fDrawTests.GridProgram();

        // render

        this.renderTo(ctx, program, surface);

        // verify image

        if (this.verifyImage(surface))
            testPassed('');
        else
            testFailed('Incorrect rendering result');
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @param {sglrGLContext.GLContext} ctx
     * @param {sglrShaderProgram.ShaderProgram} program
     * @param {tcuSurface.Surface} dst
     */
    es3fDrawTests.InstancedGridRenderTest.prototype.renderTo = function(ctx, program, dst) {
        var green = [0, 1, 0, 1];
        var yellow = [1, 1, 0, 1];

        /** @type {WebGLBuffer} */ var positionBuf = null;
        /** @type {WebGLBuffer} */ var offsetBuf = null;
        /** @type {WebGLBuffer} */ var colorBuf = null;
        /** @type {WebGLBuffer} */ var indexBuf = null;
        /** @type {WebGLProgram} */ var programID = ctx.createProgram(program);
        /** @type {number} */ var posLocation = ctx.getAttribLocation(/** @type {WebGLProgram} */ (programID), 'a_position');
        /** @type {number} */ var offsetLocation = ctx.getAttribLocation(/** @type {WebGLProgram} */ (programID), 'a_offset');
        /** @type {number} */ var colorLocation = ctx.getAttribLocation(/** @type {WebGLProgram} */ (programID), 'a_color');

        var cellW = 2.0 / this.m_gridSide;
        var cellH = 2.0 / this.m_gridSide;
        var vertexPositions = new Float32Array([
            0, 0, 0, 1,
            cellW, 0, 0, 1,
            0, cellH, 0, 1,

            0, cellH, 0, 1,
            cellW, 0, 0, 1,
            cellW, cellH, 0, 1
        ]);

        var indices = new Uint16Array([
            0, 4, 3,
            2, 1, 5
        ]);

        var offsets = [];
        for (var x = 0; x < this.m_gridSide; ++x)
        for (var y = 0; y < this.m_gridSide; ++y) {
            offsets.push(x * cellW - 1.0);
            offsets.push(y * cellW - 1.0);
            offsets.push(0, 0);
        }
        offsets = new Float32Array(offsets);

        var colors = [];
        for (var x = 0; x < this.m_gridSide; ++x)
        for (var y = 0; y < this.m_gridSide; ++y) {
            var colorToPush = ((x + y) % 2 == 0) ? (green) : (yellow);
            colors.push(colorToPush[0]);
            colors.push(colorToPush[1]);
            colors.push(colorToPush[2]);
            colors.push(colorToPush[3]);
        }
        colors = new Float32Array(colors);

        positionBuf = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, positionBuf);
        ctx.bufferData(gl.ARRAY_BUFFER, vertexPositions, gl.STATIC_DRAW);
        ctx.vertexAttribPointer(posLocation, 4, gl.FLOAT, false, 0, 0);
        ctx.vertexAttribDivisor(posLocation, 0);
        ctx.enableVertexAttribArray(posLocation);

        offsetBuf = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, offsetBuf);
        ctx.bufferData(gl.ARRAY_BUFFER, offsets, gl.STATIC_DRAW);
        ctx.vertexAttribPointer(offsetLocation, 4, gl.FLOAT, false, 0, 0);
        ctx.vertexAttribDivisor(offsetLocation, 1);
        ctx.enableVertexAttribArray(offsetLocation);

        colorBuf = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, colorBuf);
        ctx.bufferData(gl.ARRAY_BUFFER, colors, gl.STATIC_DRAW);
        ctx.vertexAttribPointer(colorLocation, 4, gl.FLOAT, false, 0, 0);
        ctx.vertexAttribDivisor(colorLocation, 1);
        ctx.enableVertexAttribArray(colorLocation);

        if (this.m_useIndices) {
            indexBuf = ctx.createBuffer();
            ctx.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuf);
            ctx.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
        }

        ctx.clearColor(0, 0, 0, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT);

        ctx.viewport(0, 0, dst.getWidth(), dst.getHeight());

        ctx.useProgram(programID);
        if (this.m_useIndices)
            ctx.drawElementsInstanced(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0, this.m_gridSide * this.m_gridSide);
        else
            ctx.drawArraysInstanced(gl.TRIANGLES, 0, 6, this.m_gridSide * this.m_gridSide);
        ctx.useProgram(null);

        if (this.m_useIndices)
            ctx.deleteBuffer(indexBuf);
        ctx.deleteBuffer(colorBuf);
        ctx.deleteBuffer(offsetBuf);
        ctx.deleteBuffer(positionBuf);
        ctx.deleteProgram(programID);

        ctx.finish();
        dst.readViewport(ctx, [0 , 0, dst.getWidth(), dst.getHeight()]);
    };

    /**
     * @param {tcuSurface.Surface} image
     * @return {boolean}
     */
    es3fDrawTests.InstancedGridRenderTest.prototype.verifyImage = function(image) {
        // \note the green/yellow pattern is only for clarity. The test will only verify that all instances were drawn by looking for anything non-green/yellow.

        var green = [0, 255, 0, 255];
        var yellow = [255, 255, 0, 255];
        var colorThreshold = 20;

        /** @type {tcuSurface.Surface} */ var error = new tcuSurface.Surface(image.getWidth(), image.getHeight());
        var isOk = true;

        for (var y = 1; y < image.getHeight() - 1; y++)
        for (var x = 1; x < image.getWidth() - 1; x++) {
            /** @type {tcuRGBA.RGBA} */ var pixel = new tcuRGBA.RGBA(image.getPixel(x, y));
            var pixelOk = true;

            // Any pixel with !(G ~= 255) is faulty (not a linear combinations of green and yellow)
            if (Math.abs(pixel.getGreen() - 255) > colorThreshold)
                pixelOk = false;

            // Any pixel with !(B ~= 0) is faulty (not a linear combinations of green and yellow)
            if (Math.abs(pixel.getBlue() - 0) > colorThreshold)
                pixelOk = false;

            error.setPixel(x, y, pixelOk ? [0, 255, 0, 255] : [255, 0, 0, 255]);
            isOk = isOk && pixelOk;
        }

        if (!isOk) {
            bufferedLogToConsole('Image verification failed.');
            debug('Verfication result');
            tcuLogImage.logImageWithInfo(image.getAccess(), 'Result');
            tcuLogImage.logImageWithInfo(error.getAccess(), 'Error mask');
        } else {
            debug('Verfication result');
        }

        return isOk;
    };

    /**
     * InstancingGroup
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fDrawTests.InstancingGroup = function(name, descr) {
        tcuTestCase.DeqpTest.call(this, name, descr);
        this.makeExecutable();
    };

    es3fDrawTests.InstancingGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.InstancingGroup.prototype.constructor = es3fDrawTests.InstancingGroup;

    /**
     * init
     */
    es3fDrawTests.InstancingGroup.prototype.init = function() {
        var gridWidths = [
            2,
            5,
            10,
            32,
            100
        ];

        // drawArrays
        for (var ndx = 0; ndx < gridWidths.length; ++ndx) {
            var name = 'draw_arrays_instanced_grid_' + gridWidths[ndx] + 'x' + gridWidths[ndx];
            var desc = 'DrawArraysInstanced, Grid size ' + gridWidths[ndx] + 'x' + gridWidths[ndx];

            this.addChild(new es3fDrawTests.InstancedGridRenderTest(name, desc, gridWidths[ndx], false));
        }

        // drawElements
        for (var ndx = 0; ndx < gridWidths.length; ++ndx) {
            var name = 'draw_elements_instanced_grid_' + gridWidths[ndx] + 'x' + gridWidths[ndx];
            var desc = 'DrawElementsInstanced, Grid size ' + gridWidths[ndx] + 'x' + gridWidths[ndx];

            this.addChild(new es3fDrawTests.InstancedGridRenderTest(name, desc, gridWidths[ndx], true));
        }
    };

    /**
     * @constructor
     * @param {number} size
     */
    es3fDrawTests.UniformWeightArray = function(size) {
        this.weights = [];

        for (var i = 0; i < size; ++i)
            this.weights[i] = 1.0;
    };

    /**
     * RandomGroup
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} descr
     */
    es3fDrawTests.RandomGroup = function(name, descr) {
        tcuTestCase.DeqpTest.call(this, name, descr);
        this.makeExecutable();
    };

    es3fDrawTests.RandomGroup.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.RandomGroup.prototype.constructor = es3fDrawTests.RandomGroup;

    /**
     * init
     */
    es3fDrawTests.RandomGroup.prototype.init = function() {
        /** @type {number} */ var numAttempts = 300;

        /** @type {Array<number>} */ var attribCounts = [1, 2, 5];
        /** @type {Array<number>} */ var attribWeights = [30, 10, 1];
        /** @type {Array<number>} */ var primitiveCounts = [2, 6, 64];
        /** @type {Array<number>} */ var primitiveCountWeights = [20, 10, 1];
        /** @type {Array<number>} */ var indexOffsets = [0, 7, 13];
        /** @type {Array<number>} */ var indexOffsetWeights = [20, 20, 1];
        /** @type {Array<number>} */ var firsts = [0, 6, 12];
        /** @type {Array<number>} */ var firstWeights = [20, 20, 1];
        /** @type {Array<number>} */ var instanceCounts = [1, 2, 16, 17];
        /** @type {Array<number>} */ var instanceWeights = [20, 10, 5, 1];
        /** @type {Array<number>} */ var indexMins = [0, 1, 3, 9];
        /** @type {Array<number>} */ var indexMaxs = [5, 9, 129, 257];
        /** @type {Array<number>} */ var indexWeights = [50, 50, 50, 50];
        /** @type {Array<number>} */ var offsets = [0, 1, 5, 12];
        /** @type {Array<number>} */ var offsetWeights = [50, 10, 10, 10];
        /** @type {Array<number>} */ var strides = [0, 7, 16, 17];
        /** @type {Array<number>} */ var strideWeights = [50, 10, 10, 10];
        /** @type {Array<number>} */ var instanceDivisors = [0, 1, 3, 129];
        /** @type {Array<number>} */ var instanceDivisorWeights = [70, 30, 10, 10];

        /** @type {Array<glsDrawTests.DrawTestSpec.Primitive>} */ var primitives = [
            glsDrawTests.DrawTestSpec.Primitive.POINTS,
            glsDrawTests.DrawTestSpec.Primitive.TRIANGLES,
            glsDrawTests.DrawTestSpec.Primitive.TRIANGLE_FAN,
            glsDrawTests.DrawTestSpec.Primitive.TRIANGLE_STRIP,
            glsDrawTests.DrawTestSpec.Primitive.LINES,
            glsDrawTests.DrawTestSpec.Primitive.LINE_STRIP,
            glsDrawTests.DrawTestSpec.Primitive.LINE_LOOP
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var primitiveWeights = new es3fDrawTests.UniformWeightArray(primitives.length);

        /** @type {Array<glsDrawTests.DrawTestSpec.DrawMethod>} */ var drawMethods = [
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS_INSTANCED,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_INSTANCED
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var drawMethodWeights = new es3fDrawTests.UniformWeightArray(drawMethods.length);

        /** @type {Array<glsDrawTests.DrawTestSpec.IndexType>} */ var indexTypes = [
            glsDrawTests.DrawTestSpec.IndexType.BYTE,
            glsDrawTests.DrawTestSpec.IndexType.SHORT,
            glsDrawTests.DrawTestSpec.IndexType.INT
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var indexTypeWeights = new es3fDrawTests.UniformWeightArray(indexTypes.length);

        /** @type {Array<glsDrawTests.DrawTestSpec.Storage>} */ var storages = [
            //glsDrawTests.DrawTestSpec.Storage.USER,
            glsDrawTests.DrawTestSpec.Storage.BUFFER
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var storageWeights = new es3fDrawTests.UniformWeightArray(storages.length);

        /** @type {Array<glsDrawTests.DrawTestSpec.InputType>} */ var inputTypes = [
            glsDrawTests.DrawTestSpec.InputType.FLOAT,
            //glsDrawTests.DrawTestSpec.InputType.FIXED,
            glsDrawTests.DrawTestSpec.InputType.BYTE,
            glsDrawTests.DrawTestSpec.InputType.SHORT,
            glsDrawTests.DrawTestSpec.InputType.UNSIGNED_BYTE,
            glsDrawTests.DrawTestSpec.InputType.UNSIGNED_SHORT,
            glsDrawTests.DrawTestSpec.InputType.INT,
            glsDrawTests.DrawTestSpec.InputType.UNSIGNED_INT,
            glsDrawTests.DrawTestSpec.InputType.HALF,
            glsDrawTests.DrawTestSpec.InputType.UNSIGNED_INT_2_10_10_10,
            glsDrawTests.DrawTestSpec.InputType.INT_2_10_10_10
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var inputTypeWeights = new es3fDrawTests.UniformWeightArray(inputTypes.length);

        /** @type {Array<glsDrawTests.DrawTestSpec.OutputType>} */ var outputTypes = [
            glsDrawTests.DrawTestSpec.OutputType.FLOAT,
            glsDrawTests.DrawTestSpec.OutputType.VEC2,
            glsDrawTests.DrawTestSpec.OutputType.VEC3,
            glsDrawTests.DrawTestSpec.OutputType.VEC4,
            glsDrawTests.DrawTestSpec.OutputType.INT,
            glsDrawTests.DrawTestSpec.OutputType.UINT,
            glsDrawTests.DrawTestSpec.OutputType.IVEC2,
            glsDrawTests.DrawTestSpec.OutputType.IVEC3,
            glsDrawTests.DrawTestSpec.OutputType.IVEC4,
            glsDrawTests.DrawTestSpec.OutputType.UVEC2,
            glsDrawTests.DrawTestSpec.OutputType.UVEC3,
            glsDrawTests.DrawTestSpec.OutputType.UVEC4
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var outputTypeWeights = new es3fDrawTests.UniformWeightArray(outputTypes.length);

        /** @type {Array<glsDrawTests.DrawTestSpec.Usage>} */ var usages = [
            glsDrawTests.DrawTestSpec.Usage.DYNAMIC_DRAW,
            glsDrawTests.DrawTestSpec.Usage.STATIC_DRAW,
            glsDrawTests.DrawTestSpec.Usage.STREAM_DRAW,
            glsDrawTests.DrawTestSpec.Usage.STREAM_READ,
            glsDrawTests.DrawTestSpec.Usage.STREAM_COPY,
            glsDrawTests.DrawTestSpec.Usage.STATIC_READ,
            glsDrawTests.DrawTestSpec.Usage.STATIC_COPY,
            glsDrawTests.DrawTestSpec.Usage.DYNAMIC_READ,
            glsDrawTests.DrawTestSpec.Usage.DYNAMIC_COPY
        ];
        /** @type {es3fDrawTests.UniformWeightArray} */ var usageWeights = new es3fDrawTests.UniformWeightArray(usages.length);

        /** @type {Array<number>} */ var insertedHashes = []; //'set' structure
        /** @type {number} */ var insertedCount = 0;

        for (var ndx = 0; ndx < numAttempts; ++ndx) {
            /** @type {deRandom.Random} */ var random = new deRandom.Random(0xc551393 + ndx); // random does not depend on previous cases

            /** @type {number} */ var attributeCount = random.chooseWeighted(attribCounts, attribWeights);
            /** @type {glsDrawTests.DrawTestSpec} */ var spec = new glsDrawTests.DrawTestSpec();

            //spec.apiType = glu::ApiType::es(3,0);
            spec.primitive = /** @type {glsDrawTests.DrawTestSpec.Primitive} */ (random.chooseWeighted(primitives, primitiveWeights.weights));
            spec.primitiveCount = random.chooseWeighted(primitiveCounts, primitiveCountWeights);
            spec.drawMethod = /** @type {glsDrawTests.DrawTestSpec.DrawMethod} */ (random.chooseWeighted(drawMethods, drawMethodWeights.weights));
            spec.indexType = /** @type {glsDrawTests.DrawTestSpec.IndexType} */ (random.chooseWeighted(indexTypes, indexTypeWeights.weights));
            spec.indexPointerOffset = random.chooseWeighted(indexOffsets, indexOffsetWeights);
            spec.indexStorage = /** @type {glsDrawTests.DrawTestSpec.Storage} */ (random.chooseWeighted(storages, storageWeights.weights));
            spec.first = random.chooseWeighted(firsts, firstWeights);
            spec.indexMin = random.chooseWeighted(indexMins, indexWeights);
            spec.indexMax = random.chooseWeighted(indexMaxs, indexWeights);
            spec.instanceCount = random.chooseWeighted(instanceCounts, instanceWeights);

            // check spec is legal
            if (!spec.valid())
                continue;

            var hasZeroDivisor = false;
            for (var attrNdx = 0; attrNdx < attributeCount;) {
                /** @type {boolean} */ var valid;
                /** @type {glsDrawTests.DrawTestSpec.AttributeSpec} */ var attribSpec = new glsDrawTests.DrawTestSpec.AttributeSpec();

                attribSpec.inputType = /** @type {glsDrawTests.DrawTestSpec.InputType} */ (random.chooseWeighted(inputTypes, inputTypeWeights.weights));
                attribSpec.outputType = /** @type {glsDrawTests.DrawTestSpec.OutputType} */ (random.chooseWeighted(outputTypes, outputTypeWeights.weights));
                attribSpec.storage = /** @type {glsDrawTests.DrawTestSpec.Storage} */ (random.chooseWeighted(storages, storageWeights.weights));
                attribSpec.usage = /** @type {glsDrawTests.DrawTestSpec.Usage} */ (random.chooseWeighted(usages, usageWeights.weights));
                attribSpec.componentCount = random.getInt(1, 4);
                attribSpec.offset = random.chooseWeighted(offsets, offsetWeights);
                attribSpec.stride = random.chooseWeighted(strides, strideWeights);
                attribSpec.normalize = random.getBool();
                attribSpec.instanceDivisor = random.chooseWeighted(instanceDivisors, instanceDivisorWeights);
                attribSpec.useDefaultAttribute = random.getBool();

                // check spec is legal
                valid = attribSpec.valid(/*spec.apiType*/);

                // we do not want interleaved elements. (Might result in some weird floating point values)
                if (attribSpec.stride && attribSpec.componentCount * glsDrawTests.DrawTestSpec.inputTypeSize(attribSpec.inputType) > attribSpec.stride)
                    valid = false;

                // try again if not valid
                if (valid) {
                    spec.attribs.push(attribSpec);
                    ++attrNdx;
                    if (attribSpec.instanceDivisor == 0)
                        hasZeroDivisor = true;
                }
            }

            // Do not collapse all vertex positions to a single positions
            if (spec.primitive != glsDrawTests.DrawTestSpec.Primitive.POINTS) {
                spec.attribs[0].instanceDivisor = 0;
                hasZeroDivisor = true;
            }

            // There should be at least one enabled vertex attribute array that has a divisor of zero in WebGL.
            // This limitation is added to keep compatible with D3D. It differs from the feature in gles.
            // See the section <Enabled Attribute> in WebGL spec: https://www.khronos.org/registry/webgl/specs/latest/2.0/#5.6
            if (hasZeroDivisor == false)
                continue;

            // Is render result meaningful?
            // Only one vertex
            if (spec.drawMethod == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED && spec.indexMin == spec.indexMax && spec.primitive != glsDrawTests.DrawTestSpec.Primitive.POINTS)
                continue;
            if (spec.attribs[0].useDefaultAttribute && spec.primitive != glsDrawTests.DrawTestSpec.Primitive.POINTS)
                continue;

            // Triangle only on one axis
            if (spec.primitive == glsDrawTests.DrawTestSpec.Primitive.TRIANGLES || spec.primitive == glsDrawTests.DrawTestSpec.Primitive.TRIANGLE_FAN || spec.primitive == glsDrawTests.DrawTestSpec.Primitive.TRIANGLE_STRIP) {
                if (spec.attribs[0].componentCount == 1)
                    continue;
                if (spec.attribs[0].outputType == glsDrawTests.DrawTestSpec.OutputType.FLOAT || spec.attribs[0].outputType == glsDrawTests.DrawTestSpec.OutputType.INT || spec.attribs[0].outputType == glsDrawTests.DrawTestSpec.OutputType.UINT)
                    continue;
                if (spec.drawMethod == glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED && (spec.indexMax - spec.indexMin) < 2)
                    continue;
            }

            // Add case
            /** @type {number} */ var hash = spec.hash();
            for (var attrNdx = 0; attrNdx < attributeCount; ++attrNdx)
                hash = deMath.binaryOp(deMath.shiftLeft(hash, 2), spec.attribs[attrNdx].hash(), deMath.BinaryOp.XOR);

            if (insertedHashes.indexOf(hash) == -1) {
                // Only properly aligned
                if (spec.isCompatibilityTest() != glsDrawTests.DrawTestSpec.CompatibilityTestType.UNALIGNED_OFFSET &&
                    spec.isCompatibilityTest() != glsDrawTests.DrawTestSpec.CompatibilityTestType.UNALIGNED_STRIDE) {
                    this.addChild(new glsDrawTests.DrawTest(spec, insertedCount + '', spec.getDesc()));
                }
                deUtil.dePushUniqueToArray(insertedHashes, hash);

                ++insertedCount;
            }
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fDrawTests.DrawTest = function() {
        tcuTestCase.DeqpTest.call(this, 'draw', 'Drawing tests');
        this.makeExecutable();
    };

    es3fDrawTests.DrawTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fDrawTests.DrawTest.prototype.constructor = es3fDrawTests.DrawTest;

    /**
     * init
     */
    es3fDrawTests.DrawTest.prototype.init = function() {
        // Basic
        /** @type {Array<glsDrawTests.DrawTestSpec.DrawMethod>} */ var basicMethods = [
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWARRAYS_INSTANCED,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_INSTANCED,
            glsDrawTests.DrawTestSpec.DrawMethod.DRAWELEMENTS_RANGED
        ];

        for (var ndx = 0; ndx < basicMethods.length; ++ndx) {
            var name = glsDrawTests.DrawTestSpec.drawMethodToString(basicMethods[ndx]);
            var desc = glsDrawTests.DrawTestSpec.drawMethodToString(basicMethods[ndx]);

            this.addChild(new es3fDrawTests.MethodGroup(name, desc, basicMethods[ndx]));
        }

        // extreme instancing

        this.addChild(new es3fDrawTests.InstancingGroup('instancing', 'draw tests with a large instance count.'));

        // Random

        this.addChild(new es3fDrawTests.RandomGroup('random', 'random draw commands.'));
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     */
    es3fDrawTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;

        var rootTest = new es3fDrawTests.DrawTest();
        state.setRoot(rootTest);

        //Set up name and description of this test series.
        setCurrentTestName(rootTest.fullName());
        description(rootTest.getDescription());

        try {
            if (range) {
                state.setRange(range);
            }
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to run draw tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
