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
goog.provide('modules.shared.glsVertexArrayTests');
goog.require('framework.common.tcuFloat');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');

goog.scope(function() {

    var glsVertexArrayTests = modules.shared.glsVertexArrayTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuRGBA = framework.common.tcuRGBA;
    var tcuFloat = framework.common.tcuFloat;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var tcuSurface = framework.common.tcuSurface;
    var tcuImageCompare = framework.common.tcuImageCompare;
    var tcuLogImage = framework.common.tcuLogImage;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;
    var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
    var deMath = framework.delibs.debase.deMath;
    var deRandom = framework.delibs.debase.deRandom;
    var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;
    var rrShadingContext = framework.referencerenderer.rrShadingContext;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
    var rrVertexPacket = framework.referencerenderer.rrVertexPacket;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * @interface
     */
    glsVertexArrayTests.deArray = function() {};

    /**
     * glsVertexArrayTests.deArray.Target enum
     * @enum
     */
    glsVertexArrayTests.deArray.Target = {
        ELEMENT_ARRAY: 0,
        ARRAY: 1
    };

    /**
     * glsVertexArrayTests.deArray.InputType enum
     * @enum
     */
    glsVertexArrayTests.deArray.InputType = {
        FLOAT: 0,
        /*FIXED: 1,
        DOUBLE: 2,*/

        BYTE: 1,
        SHORT: 2,

        UNSIGNED_BYTE: 3,
        UNSIGNED_SHORT: 4,

        INT: 5,
        UNSIGNED_INT: 6,
        HALF: 7,
        UNSIGNED_INT_2_10_10_10: 8,
        INT_2_10_10_10: 9
    };

    /**
     * glsVertexArrayTests.deArray.OutputType enum
     * @enum
     */
    glsVertexArrayTests.deArray.OutputType = {
        FLOAT: 0,
        VEC2: 1,
        VEC3: 2,
        VEC4: 3,

        INT: 4,
        UINT: 5,

        IVEC2: 6,
        IVEC3: 7,
        IVEC4: 8,

        UVEC2: 9,
        UVEC3: 10,
        UVEC4: 11
    };

    /**
     * glsVertexArrayTests.deArray.Usage enum
     * @enum
     */
    glsVertexArrayTests.deArray.Usage = {
        DYNAMIC_DRAW: 0,
        STATIC_DRAW: 1,
        STREAM_DRAW: 2,

        STREAM_READ: 3,
        STREAM_COPY: 4,

        STATIC_READ: 5,
        STATIC_COPY: 6,

        DYNAMIC_READ: 7,
        DYNAMIC_COPY: 8
    };

    /**
     * glsVertexArrayTests.deArray.Storage enum
     * @enum
     */
    glsVertexArrayTests.deArray.Storage = {
        USER: 0,
        BUFFER: 1
    };

    /**
     * glsVertexArrayTests.deArray.Primitive enum
     * @enum
     */
    glsVertexArrayTests.deArray.Primitive = {
        POINTS: 0,
        TRIANGLES: 1,
        TRIANGLE_FAN: 2,
        TRIANGLE_STRIP: 3
    };

    //glsVertexArrayTests.deArray static functions

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     * @return {string}
     */
    glsVertexArrayTests.deArray.targetToString = function(target) {
        DE_ASSERT(target < Object.keys(glsVertexArrayTests.deArray.Target).length);

        /** @type {Array<string>} */ var targets =
        [
            'element_array', // glsVertexArrayTests.deArray.Target.ELEMENT_ARRAY
            'array' // glsVertexArrayTests.deArray.Target.ARRAY
        ];
        DE_ASSERT(targets.length == Object.keys(glsVertexArrayTests.deArray.Target).length);

        return targets[target];
    };

    /**
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {string}
     */
    glsVertexArrayTests.deArray.inputTypeToString = function(type) {
        DE_ASSERT(type < Object.keys(glsVertexArrayTests.deArray.InputType).length);

        /** @type {Array<string>} */ var types =
        [
            'float', // glsVertexArrayTests.deArray.InputType.FLOAT

            'byte', // glsVertexArrayTests.deArray.InputType.BYTE
            'short', // glsVertexArrayTests.deArray.InputType.SHORT

            'unsigned_byte', // glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE
            'unsigned_short', // glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT

            'int', // glsVertexArrayTests.deArray.InputType.INT
            'unsigned_int', // glsVertexArrayTests.deArray.InputType.UNSIGNED_INT
            'half', // glsVertexArrayTests.deArray.InputType.HALF
            'unsigned_int2_10_10_10', // glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10
            'int2_10_10_10' // glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];
        DE_ASSERT(types.length == Object.keys(glsVertexArrayTests.deArray.InputType).length);

        return types[type];
    };

    /**
     * @param {glsVertexArrayTests.deArray.OutputType} type
     * @return {string}
     */
    glsVertexArrayTests.deArray.outputTypeToString = function(type) {
        DE_ASSERT(type < Object.keys(glsVertexArrayTests.deArray.OutputType).length);

        /** @type {Array<string>} */ var types =
        [
            'float', // glsVertexArrayTests.deArray.OutputType.FLOAT
            'vec2', // glsVertexArrayTests.deArray.OutputType.VEC2
            'vec3', // glsVertexArrayTests.deArray.OutputType.VEC3
            'vec4', // glsVertexArrayTests.deArray.OutputType.VEC4

            'int', // glsVertexArrayTests.deArray.OutputType.INT
            'uint', // glsVertexArrayTests.deArray.OutputType.UINT

            'ivec2', // glsVertexArrayTests.deArray.OutputType.IVEC2
            'ivec3', // glsVertexArrayTests.deArray.OutputType.IVEC3
            'ivec4', // glsVertexArrayTests.deArray.OutputType.IVEC4

            'uvec2', // glsVertexArrayTests.deArray.OutputType.UVEC2
            'uvec3', // glsVertexArrayTests.deArray.OutputType.UVEC3
            'uvec4' // glsVertexArrayTests.deArray.OutputType.UVEC4
        ];
        DE_ASSERT(types.length == Object.keys(glsVertexArrayTests.deArray.OutputType).length);

        return types[type];
    };

    /**
     * @param {glsVertexArrayTests.deArray.Usage} usage
     * @return {string}
     */
    glsVertexArrayTests.deArray.usageTypeToString = function(usage) {
        DE_ASSERT(usage < Object.keys(glsVertexArrayTests.deArray.Usage).length);

        /** @type {Array<string>} */ var usages =
        [
            'dynamic_draw', // glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW
            'static_draw', // glsVertexArrayTests.deArray.Usage.STATIC_DRAW
            'stream_draw', // glsVertexArrayTests.deArray.Usage.STREAM_DRAW

            'stream_read', // glsVertexArrayTests.deArray.Usage.STREAM_READ
            'stream_copy', // glsVertexArrayTests.deArray.Usage.STREAM_COPY

            'static_read', // glsVertexArrayTests.deArray.Usage.STATIC_READ
            'static_copy', // glsVertexArrayTests.deArray.Usage.STATIC_COPY

            'dynamic_read', // glsVertexArrayTests.deArray.Usage.DYNAMIC_READ
            'dynamic_copy' // glsVertexArrayTests.deArray.Usage.DYNAMIC_COPY
        ];
        DE_ASSERT(usages.length == Object.keys(glsVertexArrayTests.deArray.Usage).length);

        return usages[usage];
    };

    /**
     * @param {glsVertexArrayTests.deArray.Storage} storage
     * @return {string}
     */
    glsVertexArrayTests.deArray.storageToString = function(storage) {
        DE_ASSERT(storage < Object.keys(glsVertexArrayTests.deArray.Storage).length);

        /** @type {Array<string>} */ var storages =
        [
            'user_ptr', // glsVertexArrayTests.deArray.Storage.USER
            'buffer' // glsVertexArrayTests.deArray.Storage.BUFFER
        ];
        DE_ASSERT(storages.length == Object.keys(glsVertexArrayTests.deArray.Storage).length);

        return storages[storage];
    };

    /**
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @return {string}
     */
    glsVertexArrayTests.deArray.primitiveToString = function(primitive) {
        DE_ASSERT(primitive < Object.keys(glsVertexArrayTests.deArray.Primitive).length);

        /** @type {Array<string>} */ var primitives =
        [
            'points', // glsVertexArrayTests.deArray.Primitive.POINTS
            'triangles', // glsVertexArrayTests.deArray.Primitive.TRIANGLES
            'triangle_fan', // glsVertexArrayTests.deArray.Primitive.TRIANGLE_FAN
            'triangle_strip' // glsVertexArrayTests.deArray.Primitive.TRIANGLE_STRIP
        ];
        DE_ASSERT(primitives.length == Object.keys(glsVertexArrayTests.deArray.Primitive).length);

        return primitives[primitive];
    };

    /**
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {number}
     */
    glsVertexArrayTests.deArray.inputTypeSize = function(type) {
        DE_ASSERT(type < Object.keys(glsVertexArrayTests.deArray.InputType).length);

        /** @type {Array<number>} */ var size = [
            4, // glsVertexArrayTests.deArray.InputType.FLOAT

            1, // glsVertexArrayTests.deArray.InputType.BYTE
            2, // glsVertexArrayTests.deArray.InputType.SHORT

            1, // glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE
            2, // glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT

            4, // glsVertexArrayTests.deArray.InputType.INT
            4, // glsVertexArrayTests.deArray.InputType.UNSIGNED_INT
            2, // glsVertexArrayTests.deArray.InputType.HALF
            4 / 4, // glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10
            4 / 4 // glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];
        DE_ASSERT(size.length == Object.keys(glsVertexArrayTests.deArray.InputType).length);

        return size[type];
    };

    /**
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {boolean}
     */
    glsVertexArrayTests.inputTypeIsFloatType = function(type) {
        if (type == glsVertexArrayTests.deArray.InputType.FLOAT)
            return true;
        /*if (type == glsVertexArrayTests.deArray.InputType.FIXED)
            return true;
        if (type == glsVertexArrayTests.deArray.InputType.DOUBLE)
            return true;*/
        if (type == glsVertexArrayTests.deArray.InputType.HALF)
            return true;
        return false;
    };

    /**
     * @param {glsVertexArrayTests.deArray.OutputType} type
     * @return {boolean}
     */
    glsVertexArrayTests.outputTypeIsFloatType = function(type) {
        if (type == glsVertexArrayTests.deArray.OutputType.FLOAT ||
            type == glsVertexArrayTests.deArray.OutputType.VEC2 ||
            type == glsVertexArrayTests.deArray.OutputType.VEC3 ||
            type == glsVertexArrayTests.deArray.OutputType.VEC4)
            return true;

        return false;
    };

    //glsVertexArrayTests.deArray member functions (all virtual, since this is an interface)

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     * @param {number} size
     * @param {Uint8Array} data
     * @param {glsVertexArrayTests.deArray.Usage} usage
     */
    glsVertexArrayTests.deArray.prototype.data = function(target, size, data, usage) {};

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     * @param {number} offset
     * @param {number} size
     * @param {Uint8Array} data
     */
    glsVertexArrayTests.deArray.prototype.subdata = function(target, offset, size, data) {};

    /**
     * @param {number} attribNdx
     * @param {number} offset
     * @param {number} size
     * @param {glsVertexArrayTests.deArray.InputType} inType
     * @param {glsVertexArrayTests.deArray.OutputType} outType
     * @param {boolean} normalized
     * @param {number} stride
     */
    glsVertexArrayTests.deArray.prototype.bind = function(attribNdx, offset, size, inType, outType, normalized, stride) {};

    /**
     * unBind
     */
    glsVertexArrayTests.deArray.prototype.unBind = function() {};

    /**
     * @return {boolean}
     */
    glsVertexArrayTests.deArray.prototype.isBound = function() {};

    /**
     * @return {number}
     */
    glsVertexArrayTests.deArray.prototype.getComponentCount = function() {};

    /**
     * @return {glsVertexArrayTests.deArray.Target}
     */
    glsVertexArrayTests.deArray.prototype.getTarget = function() {};

    /**
     * @return {glsVertexArrayTests.deArray.InputType}
     */
    glsVertexArrayTests.deArray.prototype.getInputType = function() {};

    /**
     * @return {glsVertexArrayTests.deArray.OutputType}
     */
    glsVertexArrayTests.deArray.prototype.getOutputType = function() {};

    /**
     * @return {glsVertexArrayTests.deArray.Storage}
     */
    glsVertexArrayTests.deArray.prototype.getStorageType = function() {};

    /**
     * @return {boolean}
     */
    glsVertexArrayTests.deArray.prototype.getNormalized = function() {};

    /**
     * @return {number}
     */
    glsVertexArrayTests.deArray.prototype.getStride = function() {};

    /**
     * @return {number}
     */
    glsVertexArrayTests.deArray.prototype.getAttribNdx = function() {};

    /**
     * @param {number} attribNdx
     */
    glsVertexArrayTests.deArray.prototype.setAttribNdx = function(attribNdx) {};

    //glsVertexArrayTests.ContextArray class, implements glsVertexArrayTests.deArray interface

    /**
     * @constructor
     * @implements {glsVertexArrayTests.deArray}
     * @param {glsVertexArrayTests.deArray.Storage} storage
     * @param {sglrGLContext.GLContext | sglrReferenceContext.ReferenceContext} context
     */
    glsVertexArrayTests.ContextArray = function(storage, context) {
        /** @type {glsVertexArrayTests.deArray.Storage} */ this.m_storage = storage;
        /** @type {sglrGLContext.GLContext | sglrReferenceContext.ReferenceContext} */ this.m_ctx = context;
        /** @type {WebGLBuffer|sglrReferenceContext.DataBuffer|null} */ this.m_glBuffer = null;

        /** @type {boolean} */ this.m_bound = false;
        /** @type {number} */ this.m_attribNdx = 0;
        /** @type {number} */ this.m_size = 0;
        /** @type {Uint8Array} */ this.m_data = null;
        /** @type {number} */ this.m_componentCount = 1;
        /** @type {glsVertexArrayTests.deArray.Target} */ this.m_target = glsVertexArrayTests.deArray.Target.ARRAY;
        /** @type {glsVertexArrayTests.deArray.InputType} */ this.m_inputType = glsVertexArrayTests.deArray.InputType.FLOAT;
        /** @type {glsVertexArrayTests.deArray.OutputType} */ this.m_outputType = glsVertexArrayTests.deArray.OutputType.FLOAT;
        /** @type {boolean} */ this.m_normalize = false;
        /** @type {number} */ this.m_stride = 0;
        /** @type {number} */ this.m_offset = 0;

        if (this.m_storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
            this.m_glBuffer = this.m_ctx.createBuffer();
        }
    };

    // glsVertexArrayTests.ContextArray member functions

    /**
     * unBind
     */
    glsVertexArrayTests.ContextArray.prototype.unBind = function() { this.m_bound = false; };

    /**
     * @return {boolean}
     */
    glsVertexArrayTests.ContextArray.prototype.isBound = function() { return this.m_bound; };

    /**
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.prototype.getComponentCount = function() { return this.m_componentCount; };

    /**
     * @return {glsVertexArrayTests.deArray.Target}
     */
    glsVertexArrayTests.ContextArray.prototype.getTarget = function() { return this.m_target; };

    /**
     * @return {glsVertexArrayTests.deArray.InputType}
     */
    glsVertexArrayTests.ContextArray.prototype.getInputType = function() { return this.m_inputType; };

    /**
     * @return {glsVertexArrayTests.deArray.OutputType}
     */
    glsVertexArrayTests.ContextArray.prototype.getOutputType = function() { return this.m_outputType; };

    /**
     * @return {glsVertexArrayTests.deArray.Storage}
     */
    glsVertexArrayTests.ContextArray.prototype.getStorageType = function() { return this.m_storage; };

    /**
     * @return {boolean}
     */
    glsVertexArrayTests.ContextArray.prototype.getNormalized = function() { return this.m_normalize; };

    /**
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.prototype.getStride = function() { return this.m_stride; };

    /**
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.prototype.getAttribNdx = function() { return this.m_attribNdx; };

    /**
     * @param {number} attribNdx
     */
    glsVertexArrayTests.ContextArray.prototype.setAttribNdx = function(attribNdx) { this.m_attribNdx = attribNdx; };

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {glsVertexArrayTests.deArray.Usage} usage
     */
    glsVertexArrayTests.ContextArray.prototype.data = function(target, size, ptr, usage) {
        this.m_size = size;
        this.m_target = target;

        if (this.m_storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
            this.m_ctx.bindBuffer(glsVertexArrayTests.ContextArray.targetToGL(target), this.m_glBuffer);

            //No need for size param here, as opposed to GL ES.
            this.m_ctx.bufferData(glsVertexArrayTests.ContextArray.targetToGL(target), ptr, glsVertexArrayTests.ContextArray.usageToGL(usage));
        } else if (this.m_storage == glsVertexArrayTests.deArray.Storage.USER) {
            this.m_data = new Uint8Array(size);
            for (var i = 0; i < size; i++)
                this.m_data[i] = ptr[i];
        } else
            throw new Error('glsVertexArrayTests.ContextArray.prototype.data - Invalid storage type specified');
    };

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     * @param {number} offset
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    glsVertexArrayTests.ContextArray.prototype.subdata = function(target, offset, size, ptr) {
        this.m_target = target;

        if (this.m_storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
            this.m_ctx.bindBuffer(glsVertexArrayTests.ContextArray.targetToGL(target), this.m_glBuffer);

            this.m_ctx.bufferSubData(glsVertexArrayTests.ContextArray.targetToGL(target), offset, ptr);
        } else if (this.m_storage == glsVertexArrayTests.deArray.Storage.USER)
            for (var i = offset; i < size; i++)
                this.m_data[i] = ptr[i];
        else
            throw new Error('glsVertexArrayTests.ContextArray.prototype.subdata - Invalid storage type specified');
    };

    /**
     * @param {number} attribNdx
     * @param {number} offset
     * @param {number} size
     * @param {glsVertexArrayTests.deArray.InputType} inType
     * @param {glsVertexArrayTests.deArray.OutputType} outType
     * @param {boolean} normalized
     * @param {number} stride
     */
    glsVertexArrayTests.ContextArray.prototype.bind = function(attribNdx, offset, size, inType, outType, normalized, stride) {
        this.m_attribNdx = attribNdx;
        this.m_bound = true;
        this.m_componentCount = size;
        this.m_inputType = inType;
        this.m_outputType = outType;
        this.m_normalize = normalized;
        this.m_stride = stride;
        this.m_offset = offset;
    };

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     */
    glsVertexArrayTests.ContextArray.prototype.bindIndexArray = function(target) {
        if (this.m_storage == glsVertexArrayTests.deArray.Storage.USER) {
        } else if (this.m_storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
            this.m_ctx.bindBuffer(glsVertexArrayTests.ContextArray.targetToGL(target), this.m_glBuffer);
        }
    };

    /**
     * @param {number} loc
     */
    glsVertexArrayTests.ContextArray.prototype.glBind = function(loc) {
        if (this.m_storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
            this.m_ctx.bindBuffer(glsVertexArrayTests.ContextArray.targetToGL(this.m_target), this.m_glBuffer);

            if (!glsVertexArrayTests.inputTypeIsFloatType(this.m_inputType)) {
                // Input is not float type

                if (glsVertexArrayTests.outputTypeIsFloatType(this.m_outputType)) {
                    // Output type is float type
                    this.m_ctx.vertexAttribPointer(loc, this.m_componentCount, glsVertexArrayTests.ContextArray.inputTypeToGL(this.m_inputType), this.m_normalize, this.m_stride, this.m_offset);
                } else {
                    // Output type is int type
                    this.m_ctx.vertexAttribIPointer(loc, this.m_componentCount, glsVertexArrayTests.ContextArray.inputTypeToGL(this.m_inputType), this.m_stride, this.m_offset);
                }
            } else {
                // Input type is float type
                // Output type must be float type
                DE_ASSERT(this.m_outputType == glsVertexArrayTests.deArray.OutputType.FLOAT || this.m_outputType == glsVertexArrayTests.deArray.OutputType.VEC2 || this.m_outputType == glsVertexArrayTests.deArray.OutputType.VEC3 || this.m_outputType == glsVertexArrayTests.deArray.OutputType.VEC4);

                this.m_ctx.vertexAttribPointer(loc, this.m_componentCount, glsVertexArrayTests.ContextArray.inputTypeToGL(this.m_inputType), this.m_normalize, this.m_stride, this.m_offset);
            }

            this.m_ctx.bindBuffer(glsVertexArrayTests.ContextArray.targetToGL(this.m_target), null);
        } else if (this.m_storage == glsVertexArrayTests.deArray.Storage.USER) {
            this.m_ctx.bindBuffer(glsVertexArrayTests.ContextArray.targetToGL(this.m_target), null);

            if (!glsVertexArrayTests.inputTypeIsFloatType(this.m_inputType)) {
                // Input is not float type

                if (glsVertexArrayTests.outputTypeIsFloatType(this.m_outputType)) {
                    // Output type is float type
                    this.m_ctx.vertexAttribPointer(loc, this.m_componentCount, glsVertexArrayTests.ContextArray.inputTypeToGL(this.m_inputType), this.m_normalize, this.m_stride, this.m_data.subarray(this.m_offset));
                } else {
                    // Output type is int type
                    this.m_ctx.vertexAttribIPointer(loc, this.m_componentCount, glsVertexArrayTests.ContextArray.inputTypeToGL(this.m_inputType), this.m_stride, this.m_data.subarray(this.m_offset));
                }
            } else {
                // Input type is float type

                // Output type must be float type
                DE_ASSERT(this.m_outputType == glsVertexArrayTests.deArray.OutputType.FLOAT || this.m_outputType == glsVertexArrayTests.deArray.OutputType.VEC2 || this.m_outputType == glsVertexArrayTests.deArray.OutputType.VEC3 || this.m_outputType == glsVertexArrayTests.deArray.OutputType.VEC4);

                this.m_ctx.vertexAttribPointer(loc, this.m_componentCount, glsVertexArrayTests.ContextArray.inputTypeToGL(this.m_inputType), this.m_normalize, this.m_stride, this.m_data.subarray(this.m_offset));
            }
        } else
            throw new Error('glsVertexArrayTests.ContextArray.prototype.glBind - Invalid storage type specified');
    };

    //glsVertexArrayTests.ContextArray static functions

    /**
     * @param {glsVertexArrayTests.deArray.Target} target
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.targetToGL = function(target) {
        DE_ASSERT(target < Object.keys(glsVertexArrayTests.deArray.Target).length);

        /** @type {Array<number>} */ var targets =
        [
            gl.ELEMENT_ARRAY_BUFFER, // glsVertexArrayTests.deArray.Target.ELEMENT_ARRAY
            gl.ARRAY_BUFFER // glsVertexArrayTests.deArray.Target.ARRAY
        ];

        return targets[target];
    };

    /**
     * @param {glsVertexArrayTests.deArray.Usage} usage
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.usageToGL = function(usage) {
        DE_ASSERT(usage < Object.keys(glsVertexArrayTests.deArray.Usage).length);

        /** @type {Array<number>} */ var usages =
        [
            gl.DYNAMIC_DRAW, // glsVertexArrayTests.deArray.Usage.DYNAMIC_DRAW
            gl.STATIC_DRAW, // glsVertexArrayTests.deArray.Usage.STATIC_DRAW
            gl.STREAM_DRAW, // glsVertexArrayTests.deArray.Usage.STREAM_DRAW

            gl.STREAM_READ, // glsVertexArrayTests.deArray.Usage.STREAM_READ
            gl.STREAM_COPY, // glsVertexArrayTests.deArray.Usage.STREAM_COPY

            gl.STATIC_READ, // glsVertexArrayTests.deArray.Usage.STATIC_READ
            gl.STATIC_COPY, // glsVertexArrayTests.deArray.Usage.STATIC_COPY

            gl.DYNAMIC_READ, // glsVertexArrayTests.deArray.Usage.DYNAMIC_READ
            gl.DYNAMIC_COPY // glsVertexArrayTests.deArray.Usage.DYNAMIC_COPY
        ];
        DE_ASSERT(usages.length == Object.keys(glsVertexArrayTests.deArray.Usage).length);

        return usages[usage];
    };

    /**
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.inputTypeToGL = function(type) {
        DE_ASSERT(type < Object.keys(glsVertexArrayTests.deArray.InputType).length);

        /** @type {Array<number>} */ var types =
        [
            gl.FLOAT, // glsVertexArrayTests.deArray.InputType.FLOAT

            gl.BYTE, // glsVertexArrayTests.deArray.InputType.BYTE
            gl.SHORT, // glsVertexArrayTests.deArray.InputType.SHORT
            gl.UNSIGNED_BYTE, // glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE
            gl.UNSIGNED_SHORT, // glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT

            gl.INT, // glsVertexArrayTests.deArray.InputType.INT
            gl.UNSIGNED_INT, // glsVertexArrayTests.deArray.InputType.UNSIGNED_INT
            gl.HALF_FLOAT, // glsVertexArrayTests.deArray.InputType.HALF
            gl.UNSIGNED_INT_2_10_10_10_REV, // glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10
            gl.INT_2_10_10_10_REV // glsVertexArrayTests.deArray.InputType.INT_2_10_10_10
        ];
        DE_ASSERT(types.length == Object.keys(glsVertexArrayTests.deArray.InputType).length);

        return types[type];
    };

    /**
     * @param {glsVertexArrayTests.deArray.OutputType} type
     * @return {string}
     */
    glsVertexArrayTests.ContextArray.outputTypeToGLType = function(type) {
        DE_ASSERT(type < Object.keys(glsVertexArrayTests.deArray.OutputType).length);

        /** @type {Array<string>} */ var types =
        [
            'float', // glsVertexArrayTests.deArray.OutputType.FLOAT
            'vec2', // glsVertexArrayTests.deArray.OutputType.VEC2
            'vec3', // glsVertexArrayTests.deArray.OutputType.VEC3
            'vec4', // glsVertexArrayTests.deArray.OutputType.VEC4

            'int', // glsVertexArrayTests.deArray.OutputType.INT
            'uint', // glsVertexArrayTests.deArray.OutputType.UINT

            'ivec2', // glsVertexArrayTests.deArray.OutputType.IVEC2
            'ivec3', // glsVertexArrayTests.deArray.OutputType.IVEC3
            'ivec4', // glsVertexArrayTests.deArray.OutputType.IVEC4

            'uvec2', // glsVertexArrayTests.deArray.OutputType.UVEC2
            'uvec3', // glsVertexArrayTests.deArray.OutputType.UVEC3
            'uvec4' // glsVertexArrayTests.deArray.OutputType.UVEC4
        ];
        DE_ASSERT(types.length == Object.keys(glsVertexArrayTests.deArray.OutputType).length);

        return types[type];
    };

    /**
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @return {number}
     */
    glsVertexArrayTests.ContextArray.primitiveToGL = function(primitive) {
        /** @type {Array<number>} */ var primitives =
        [
            gl.POINTS, // glsVertexArrayTests.deArray.Primitive.POINTS
            gl.TRIANGLES, // glsVertexArrayTests.deArray.Primitive.TRIANGLES
            gl.TRIANGLE_FAN, // glsVertexArrayTests.deArray.Primitive.TRIANGLE_FAN
            gl.TRIANGLE_STRIP // glsVertexArrayTests.deArray.Primitive.TRIANGLE_STRIP
        ];
        DE_ASSERT(primitives.length == Object.keys(glsVertexArrayTests.deArray.Primitive).length);

        return primitives[primitive];
    };

    /**
     * @constructor
     * @param {sglrGLContext.GLContext | sglrReferenceContext.ReferenceContext} drawContext
     */
    glsVertexArrayTests.ContextArrayPack = function(drawContext) {
        /** @type {WebGLRenderingContextBase} */ this.m_renderCtx = gl;
        //TODO: Reference rasterizer implementation.
        /** @type {sglrGLContext.GLContext | sglrReferenceContext.ReferenceContext} */ this.m_ctx = drawContext;

        /** @type {Array<glsVertexArrayTests.ContextArray>} */ this.m_arrays = [];
        /** @type {sglrShaderProgram.ShaderProgram} */ this.m_program;
        /** @type {tcuSurface.Surface} */ this.m_screen = new tcuSurface.Surface(
            Math.min(512, canvas.width),
            Math.min(512, canvas.height)
        );
    };

    /**
     * @return {number}
     */
    glsVertexArrayTests.ContextArrayPack.prototype.getArrayCount = function() {
        return this.m_arrays.length;
    };

    /**
     * @param {glsVertexArrayTests.deArray.Storage} storage
     */
    glsVertexArrayTests.ContextArrayPack.prototype.newArray = function(storage) {
        this.m_arrays.push(new glsVertexArrayTests.ContextArray(storage, this.m_ctx));
    };

    /**
     * @param {number} i
     * @return {glsVertexArrayTests.ContextArray}
     */
    glsVertexArrayTests.ContextArrayPack.prototype.getArray = function(i) {
        return this.m_arrays[i];
    };

    /**
     * updateProgram
     */
    glsVertexArrayTests.ContextArrayPack.prototype.updateProgram = function() {
        this.m_program = new glsVertexArrayTests.ContextShaderProgram(this.m_renderCtx, this.m_arrays);
    };

    /**
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @param {number} firstVertex
     * @param {number} vertexCount
     * @param {boolean} useVao
     * @param {number} coordScale
     * @param {number} colorScale
     */
    glsVertexArrayTests.ContextArrayPack.prototype.render = function(primitive, firstVertex, vertexCount, useVao, coordScale, colorScale) {
        var program;
        /** @type {(WebGLVertexArrayObject|sglrReferenceContext.VertexArray|null)} */ var vaoID = null;

        this.updateProgram();

        this.m_ctx.viewport(0, 0, this.m_screen.getWidth(), this.m_screen.getHeight());
        this.m_ctx.clearColor(0.0, 0.0, 0.0, 1.0);
        this.m_ctx.clear(gl.COLOR_BUFFER_BIT);

        program = this.m_ctx.createProgram(this.m_program);

        this.m_ctx.useProgram(program);

        this.m_ctx.uniform1f(this.m_ctx.getUniformLocation(program, 'u_coordScale'), coordScale);
        this.m_ctx.uniform1f(this.m_ctx.getUniformLocation(program, 'u_colorScale'), colorScale);

        if (useVao) {
            vaoID = this.m_ctx.createVertexArray();
            this.m_ctx.bindVertexArray(vaoID);
        }

        /** @type {string} */ var attribName;
        /** @type {number} */ var loc;
        for (var arrayNdx = 0; arrayNdx < this.m_arrays.length; arrayNdx++) {
            if (this.m_arrays[arrayNdx].isBound()) {
                attribName = 'a_' + this.m_arrays[arrayNdx].getAttribNdx();
                loc = this.m_ctx.getAttribLocation(program, attribName);
                this.m_ctx.enableVertexAttribArray(loc);

                this.m_arrays[arrayNdx].glBind(loc);
            }
        }

        DE_ASSERT((firstVertex % 6) == 0);
        //this.m_ctx.drawArrays(glsVertexArrayTests.ContextArray.primitiveToGL(primitive), firstVertex, vertexCount - firstVertex);
        this.m_ctx.drawQuads(gl.TRIANGLES, firstVertex, vertexCount - firstVertex);

        for (var arrayNdx = 0; arrayNdx < this.m_arrays.length; arrayNdx++) {
            if (this.m_arrays[arrayNdx].isBound()) {
                attribName = 'a_' + this.m_arrays[arrayNdx].getAttribNdx();
                loc = this.m_ctx.getAttribLocation(program, attribName);

                this.m_ctx.disableVertexAttribArray(loc);
            }
        }

        if (useVao)
            vaoID = this.m_ctx.deleteVertexArray(vaoID);

        this.m_ctx.deleteProgram(program);
        this.m_ctx.useProgram(null);
        this.m_ctx.readPixels(0, 0, this.m_screen.getWidth(), this.m_screen.getHeight(), gl.RGBA, gl.UNSIGNED_BYTE, this.m_screen.getAccess().getDataPtr());
    };

    /**
     * @return {tcuSurface.Surface}
     */
    glsVertexArrayTests.ContextArrayPack.prototype.getSurface = function() { return this.m_screen; };

    /**
     * glsVertexArrayTests.ContextShaderProgram class
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {WebGLRenderingContextBase | sglrReferenceContext.ReferenceContext} ctx
     * @param {Array<glsVertexArrayTests.ContextArray>} arrays
     */
    glsVertexArrayTests.ContextShaderProgram = function(ctx, arrays) {
        sglrShaderProgram.ShaderProgram.call(this, this.createProgramDeclaration(ctx, arrays));
        this.m_componentCount = new Array(arrays.length);
        /** @type {Array<rrGenericVector.GenericVecType>} */ this.m_attrType = new Array(arrays.length);

        for (var arrayNdx = 0; arrayNdx < arrays.length; arrayNdx++) {
            this.m_componentCount[arrayNdx] = this.getComponentCount(arrays[arrayNdx].getOutputType());
            this.m_attrType[arrayNdx] = this.mapOutputType(arrays[arrayNdx].getOutputType());
        }
    };

    glsVertexArrayTests.ContextShaderProgram.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    glsVertexArrayTests.ContextShaderProgram.prototype.constructor = glsVertexArrayTests.ContextShaderProgram;

    /**
     * glsVertexArrayTests.calcShaderColorCoord function
     * @param {Array<number>} coord (2 elements)
     * @param {Array<number>} color (3 elements)
     * @param {goog.NumberArray} attribValue (4 elements)
     * @param {boolean} isCoordinate
     * @param {number} numComponents
     */
    glsVertexArrayTests.calcShaderColorCoord = function(coord, color, attribValue, isCoordinate, numComponents) {
        if (isCoordinate)
            switch (numComponents) {
                case 1:
                    coord[0] = attribValue[0];
                    coord[1] = attribValue[0];
                    break;
                case 2:
                    coord[0] = attribValue[0];
                    coord[1] = attribValue[1];
                    break;
                case 3:
                    coord[0] = attribValue[0] + attribValue[2];
                    coord[1] = attribValue[1];
                    break;
                case 4:
                    coord[0] = attribValue[0] + attribValue[2];
                    coord[1] = attribValue[1] + attribValue[3];
                    break;
                default:
                    throw new Error('glsVertexArrayTests.calcShaderColorCoord - Invalid number of components');
            } else {
            switch (numComponents) {
                case 1:
                    color[0] = color[0] * attribValue[0];
                    break;
                case 2:
                    color[0] = color[0] * attribValue[0];
                    color[1] = color[1] * attribValue[1];
                    break;
                case 3:
                    color[0] = color[0] * attribValue[0];
                    color[1] = color[1] * attribValue[1];
                    color[2] = color[2] * attribValue[2];
                    break;
                case 4:
                    color[0] = color[0] * attribValue[0] * attribValue[3];
                    color[1] = color[1] * attribValue[1] * attribValue[3];
                    color[2] = color[2] * attribValue[2] * attribValue[3];
                    break;
                default:
                    throw new Error('glsVertexArrayTests.calcShaderColorCoord - Invalid number of components');
            }
        }
    };

    /**
     * glsVertexArrayTests.ContextShaderProgram.shadeVertices
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.shadeVertices = function(inputs, packets, numPackets) {
        /** @type {number} */ var u_coordScale = this.getUniformByName('u_coordScale').value[0];
        /** @type {number} */ var u_colorScale = this.getUniformByName('u_colorScale').value[0];

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {number} */ var varyingLocColor = 0;

            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];

            // Calc output color
            /** @type {Array<number>} */ var coord = [1.0, 1.0];
            /** @type {Array<number>} */ var color = [1.0, 1.0, 1.0];

            for (var attribNdx = 0; attribNdx < this.m_attrType.length; attribNdx++) {
                /** @type {number} */ var numComponents = this.m_componentCount[attribNdx];

                glsVertexArrayTests.calcShaderColorCoord(coord, color, rrVertexAttrib.readVertexAttrib(inputs[attribNdx], packet.instanceNdx, packet.vertexNdx, this.m_attrType[attribNdx]), attribNdx == 0, numComponents);
            }

            // Transform position
            packet.position = [u_coordScale * coord[0], u_coordScale * coord[1], 1.0, 1.0];

            // Pass color to FS
            packet.outputs[varyingLocColor] = [u_colorScale * color[0], u_colorScale * color[1], u_colorScale * color[2], 1.0];
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packets
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.shadeFragments = function(packets, context) {
        var varyingLocColor = 0;

        // Normal shading
        for (var packetNdx = 0; packetNdx < packets.length; ++packetNdx)
            packets[packetNdx].value = rrShadingContext.readTriangleVarying(packets[packetNdx], context, varyingLocColor);
    };

    /**
     * @param {Array<glsVertexArrayTests.ContextArray>} arrays
     * @return string
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.genVertexSource = function(arrays) {
        var vertexShaderSrc = '';
        var params = [];

        params['VTX_IN'] = 'in';
        params['VTX_OUT'] = 'out';
        params['FRAG_IN'] = 'in';
        params['FRAG_COLOR'] = 'dEQP_FragColor';
        params['VTX_HDR'] = '#version 300 es\n';
        params['FRAG_HDR'] = '#version 300 es\nlayout(location = 0) out mediump vec4 dEQP_FragColor;\n';

        vertexShaderSrc += params['VTX_HDR'];

        for (var arrayNdx = 0; arrayNdx < arrays.length; arrayNdx++) {
            vertexShaderSrc += params['VTX_IN'] + ' highp ' + glsVertexArrayTests.ContextArray.outputTypeToGLType(arrays[arrayNdx].getOutputType()) + ' a_' + arrays[arrayNdx].getAttribNdx() + ';\n';
        }

        vertexShaderSrc +=
        'uniform highp float u_coordScale;\n' +
        'uniform highp float u_colorScale;\n' +
        params['VTX_OUT'] + ' mediump vec4 v_color;\n' +
        'void main(void)\n' +
        ' {\n' +
        '\tgl_PointSize = 1.0;\n' +
        '\thighp vec2 coord = vec2(1.0, 1.0);\n' +
        '\thighp vec3 color = vec3(1.0, 1.0, 1.0);\n';

        for (var arrayNdx = 0; arrayNdx < arrays.length; arrayNdx++) {
            if (arrays[arrayNdx].getAttribNdx() == 0) {
                switch (arrays[arrayNdx].getOutputType()) {
                    case (glsVertexArrayTests.deArray.OutputType.FLOAT):
                        vertexShaderSrc +=
                        '\tcoord = vec2(a_0);\n';
                        break;

                    case (glsVertexArrayTests.deArray.OutputType.VEC2):
                        vertexShaderSrc +=
                        '\tcoord = a_0.xy;\n';
                        break;

                    case (glsVertexArrayTests.deArray.OutputType.VEC3):
                        vertexShaderSrc +=
                        '\tcoord = a_0.xy;\n' +
                        '\tcoord.x = coord.x + a_0.z;\n';
                        break;

                    case (glsVertexArrayTests.deArray.OutputType.VEC4):
                        vertexShaderSrc +=
                        '\tcoord = a_0.xy;\n' +
                        '\tcoord += a_0.zw;\n';
                        break;

                    case (glsVertexArrayTests.deArray.OutputType.IVEC2):
                    case (glsVertexArrayTests.deArray.OutputType.UVEC2):
                        vertexShaderSrc +=
                        '\tcoord = vec2(a_0.xy);\n';
                        break;

                    case (glsVertexArrayTests.deArray.OutputType.IVEC3):
                    case (glsVertexArrayTests.deArray.OutputType.UVEC3):
                        vertexShaderSrc +=
                        '\tcoord = vec2(a_0.xy);\n' +
                        '\tcoord.x = coord.x + float(a_0.z);\n';
                        break;

                    case (glsVertexArrayTests.deArray.OutputType.IVEC4):
                    case (glsVertexArrayTests.deArray.OutputType.UVEC4):
                        vertexShaderSrc +=
                        '\tcoord = vec2(a_0.xy);\n' +
                        '\tcoord += vec2(a_0.zw);\n';
                        break;

                    default:
                        throw new Error('Invalid output type');
                        break;
                }
                continue;
            }

            switch (arrays[arrayNdx].getOutputType()) {
                case (glsVertexArrayTests.deArray.OutputType.FLOAT):
                    vertexShaderSrc +=
                    '\tcolor = color * a_' + arrays[arrayNdx].getAttribNdx() + ';\n';
                    break;

                case (glsVertexArrayTests.deArray.OutputType.VEC2):
                    vertexShaderSrc +=
                    '\tcolor.rg = color.rg * a_' + arrays[arrayNdx].getAttribNdx() + '.xy;\n';
                    break;

                case (glsVertexArrayTests.deArray.OutputType.VEC3):
                    vertexShaderSrc +=
                    '\tcolor = color.rgb * a_' + arrays[arrayNdx].getAttribNdx() + '.xyz;\n';
                    break;

                case (glsVertexArrayTests.deArray.OutputType.VEC4):
                    vertexShaderSrc +=
                    '\tcolor = color.rgb * a_' + arrays[arrayNdx].getAttribNdx() + '.xyz * a_' + arrays[arrayNdx].getAttribNdx() + '.w;\n';
                    break;

                default:
                    throw new Error('Invalid output type');
                    break;
            }
        }

        vertexShaderSrc +=
        '\tv_color = vec4(u_colorScale * color, 1.0);\n' +
        '\tgl_Position = vec4(u_coordScale * coord, 1.0, 1.0);\n' +
        '}\n';

        return vertexShaderSrc;
    };

    /**
     * @return {string}
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.genFragmentSource = function() {
        var params = [];

        params['VTX_IN'] = 'in';
        params['VTX_OUT'] = 'out';
        params['FRAG_IN'] = 'in';
        params['FRAG_COLOR'] = 'dEQP_FragColor';
        params['VTX_HDR'] = '#version 300 es\n';
        params['FRAG_HDR'] = '#version 300 es\nlayout(location = 0) out mediump vec4 dEQP_FragColor;\n';

        /* TODO: Check if glsl supported version check function is needed.*/

        var fragmentShaderSrc = params['FRAG_HDR'] +
        params['FRAG_IN'] + ' mediump vec4 v_color;\n' +
        'void main(void)\n' +
        ' {\n' +
        '\t' + params['FRAG_COLOR'] + ' = v_color;\n' +
        '}\n';

        return fragmentShaderSrc;
    };

    /**
     * @param {glsVertexArrayTests.deArray.OutputType} type
     * @return {rrGenericVector.GenericVecType}
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.mapOutputType = function(type) {
        switch (type) {
            case (glsVertexArrayTests.deArray.OutputType.FLOAT):
            case (glsVertexArrayTests.deArray.OutputType.VEC2):
            case (glsVertexArrayTests.deArray.OutputType.VEC3):
            case (glsVertexArrayTests.deArray.OutputType.VEC4):
                return rrGenericVector.GenericVecType.FLOAT;

            case (glsVertexArrayTests.deArray.OutputType.INT):
            case (glsVertexArrayTests.deArray.OutputType.IVEC2):
            case (glsVertexArrayTests.deArray.OutputType.IVEC3):
            case (glsVertexArrayTests.deArray.OutputType.IVEC4):
                return rrGenericVector.GenericVecType.INT32;

            case (glsVertexArrayTests.deArray.OutputType.UINT):
            case (glsVertexArrayTests.deArray.OutputType.UVEC2):
            case (glsVertexArrayTests.deArray.OutputType.UVEC3):
            case (glsVertexArrayTests.deArray.OutputType.UVEC4):
                return rrGenericVector.GenericVecType.UINT32;

            default:
                throw new Error('Invalid output type');
        }
    };

    /**
     * @param {glsVertexArrayTests.deArray.OutputType} type
     * @return {number}
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.getComponentCount = function(type) {
        switch (type) {
            case (glsVertexArrayTests.deArray.OutputType.FLOAT):
            case (glsVertexArrayTests.deArray.OutputType.INT):
            case (glsVertexArrayTests.deArray.OutputType.UINT):
                return 1;

            case (glsVertexArrayTests.deArray.OutputType.VEC2):
            case (glsVertexArrayTests.deArray.OutputType.IVEC2):
            case (glsVertexArrayTests.deArray.OutputType.UVEC2):
                return 2;

            case (glsVertexArrayTests.deArray.OutputType.VEC3):
            case (glsVertexArrayTests.deArray.OutputType.IVEC3):
            case (glsVertexArrayTests.deArray.OutputType.UVEC3):
                return 3;

            case (glsVertexArrayTests.deArray.OutputType.VEC4):
            case (glsVertexArrayTests.deArray.OutputType.IVEC4):
            case (glsVertexArrayTests.deArray.OutputType.UVEC4):
                return 4;

            default:
                throw new Error('Invalid output type');
        }
    };

    /**
     * @param {WebGLRenderingContextBase | sglrReferenceContext.ReferenceContext} ctx
     * @param {Array<glsVertexArrayTests.ContextArray>} arrays
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    glsVertexArrayTests.ContextShaderProgram.prototype.createProgramDeclaration = function(ctx, arrays) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */ var decl = new sglrShaderProgram.ShaderProgramDeclaration();

        for (var arrayNdx = 0; arrayNdx < arrays.length; arrayNdx++)
            decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_' + arrayNdx, this.mapOutputType(arrays[arrayNdx].getOutputType())));

        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(rrGenericVector.GenericVecType.FLOAT));

        decl.pushVertexSource(new sglrShaderProgram.VertexSource(this.genVertexSource(/*ctx,*/ arrays))); //TODO: Check if we need to review the support of a given GLSL version (we'd need the ctx)
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(this.genFragmentSource(/*ctx*/)));

        decl.pushUniform(new sglrShaderProgram.Uniform('u_coordScale', gluShaderUtil.DataType.FLOAT));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_colorScale', gluShaderUtil.DataType.FLOAT));

        return decl;
    };

    /**
     * glsVertexArrayTests.GLValue class
     * @constructor
     */
    glsVertexArrayTests.GLValue = function() {
        /** @type {goog.NumberArray} */ this.m_value = [0];
        /** @type {glsVertexArrayTests.deArray.InputType} */ this.m_type;
    };

    /**
     * @param {Uint8Array} dst
     * @param {glsVertexArrayTests.GLValue} val
     */
    glsVertexArrayTests.copyGLValueToArray = function(dst, val) {
        /** @type {Uint8Array} */ var val8 = new Uint8Array(val.m_value.buffer); // TODO: Fix encapsulation issue
        dst.set(val8);
    };

    /**
     * @param {Uint8Array} dst
     * @param {goog.NumberArray} src
     */
    glsVertexArrayTests.copyArray = function(dst, src) {
        /** @type {Uint8Array} */ var src8 = new Uint8Array(src.buffer).subarray(src.byteOffset, src.byteOffset + src.byteLength); // TODO: Fix encapsulation issue
        dst.set(src8);
    };

    /**
     * typeToTypedArray function. Determines which type of array will store the value, and stores it.
     * @param {number} value
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    glsVertexArrayTests.GLValue.typeToTypedArray = function(value, type) {
        var array;

        switch (type) {
            case glsVertexArrayTests.deArray.InputType.FLOAT:
                array = new Float32Array(1);
                break;
            /*case glsVertexArrayTests.deArray.InputType.FIXED:
                array = new Int32Array(1);
                break;
            case glsVertexArrayTests.deArray.InputType.DOUBLE:
                array = new Float32Array(1); // 64-bit?
                break;*/

            case glsVertexArrayTests.deArray.InputType.BYTE:
                array = new Int8Array(1);
                break;
            case glsVertexArrayTests.deArray.InputType.SHORT:
                array = new Int16Array(1);
                break;

            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
                array = new Uint8Array(1);
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
                array = new Uint16Array(1);
                break;

            case glsVertexArrayTests.deArray.InputType.INT:
                array = new Int32Array(1);
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT:
                array = new Uint32Array(1);
                break;
            case glsVertexArrayTests.deArray.InputType.HALF:
                array = new Uint16Array(1);
                value = glsVertexArrayTests.GLValue.floatToHalf(value);
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10:
                array = new Uint32Array(1);
                break;
            case glsVertexArrayTests.deArray.InputType.INT_2_10_10_10:
                array = new Int32Array(1);
                break;
            default:
                throw new Error('glsVertexArrayTests.GLValue.typeToTypedArray - Invalid InputType');
        }

        array[0] = value;
        return array;
    };

    /**
     * glsVertexArrayTests.GLValue.create
     * @param {number} value
     * @param {glsVertexArrayTests.deArray.InputType} type
     */
    glsVertexArrayTests.GLValue.create = function(value, type) {
        var v = new glsVertexArrayTests.GLValue();
        v.m_value = glsVertexArrayTests.GLValue.typeToTypedArray(value, type);
        v.m_type = type;
        return v;
    };

    /**
     * glsVertexArrayTests.GLValue.halfToFloat
     * @param {number} value
     * @return {number}
     */
    glsVertexArrayTests.GLValue.halfToFloat = function(value) {
        return tcuFloat.halfFloatToNumberNoDenorm(value);
    };

    /**
     * @param {number} f
     * @return {number}
     */
    glsVertexArrayTests.GLValue.floatToHalf = function(f) {
        // No denorm support.
        return tcuFloat.numberToHalfFloatNoDenorm(f);
    };

    /**
     * glsVertexArrayTests.GLValue.getMaxValue
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.getMaxValue = function(type) {
        var value;

        switch (type) {
            case glsVertexArrayTests.deArray.InputType.FLOAT:
                value = 127;
                break;
            /*case glsVertexArrayTests.deArray.InputType.FIXED:
                value = 32760;
                break;
            case glsVertexArrayTests.deArray.InputType.DOUBLE:
                value = 127;
                break;*/
            case glsVertexArrayTests.deArray.InputType.BYTE:
                value = 127;
                break;
            case glsVertexArrayTests.deArray.InputType.SHORT:
                value = 32760;
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
                value = 255;
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
                value = 65530;
                break;
            case glsVertexArrayTests.deArray.InputType.INT:
                value = 2147483647;
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT:
                value = 4294967295;
                break;
            case glsVertexArrayTests.deArray.InputType.HALF:
                value = 256;
                break;
            default: //Original code returns garbage-filled GLValues
                return new glsVertexArrayTests.GLValue();
        }

        return glsVertexArrayTests.GLValue.create(value, type);
    };

    /**
     * glsVertexArrayTests.GLValue.getMinValue
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.getMinValue = function(type) {
        var value;

        switch (type) {
            case glsVertexArrayTests.deArray.InputType.FLOAT:
                value = -127;
                break;
            /*case glsVertexArrayTests.deArray.InputType.FIXED:
                value = -32760;
                break;
            case glsVertexArrayTests.deArray.InputType.DOUBLE:
                value = -127;
                break;*/
            case glsVertexArrayTests.deArray.InputType.BYTE:
                value = -127;
                break;
            case glsVertexArrayTests.deArray.InputType.SHORT:
                value = -32760;
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
                value = 0;
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
                value = 0;
                break;
            case glsVertexArrayTests.deArray.InputType.INT:
                value = -2147483647;
                break;
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT:
                value = 0;
                break;
            case glsVertexArrayTests.deArray.InputType.HALF:
                value = -256;
                break;

            default: //Original code returns garbage-filled GLValues
                return new glsVertexArrayTests.GLValue();
        }

        return glsVertexArrayTests.GLValue.create(value, type);
    };

    /**
     * glsVertexArrayTests.GLValue.getRandom
     * @param {deRandom.Random} rnd
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.getRandom = function(rnd, min, max) {
        DE_ASSERT(min.getType() == max.getType());

        var minv = min.interpret();
        var maxv = max.interpret();
        var type = min.getType();
        var value;

        if (maxv < minv)
            return min;

        switch (type) {
            case glsVertexArrayTests.deArray.InputType.FLOAT:
            //case glsVertexArrayTests.deArray.InputType.DOUBLE:
            case glsVertexArrayTests.deArray.InputType.HALF: {
                return glsVertexArrayTests.GLValue.create(minv + rnd.getFloat() * (maxv - minv), type);
                break;
            }

            /*case glsVertexArrayTests.deArray.InputType.FIXED: {
                return minv == maxv ? min : glsVertexArrayTests.GLValue.create(minv + rnd.getInt() % (maxv - minv), type);
                break;
            }*/

            case glsVertexArrayTests.deArray.InputType.SHORT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
            case glsVertexArrayTests.deArray.InputType.BYTE:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
            case glsVertexArrayTests.deArray.InputType.INT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT: {
                return glsVertexArrayTests.GLValue.create(minv + rnd.getInt() % (maxv - minv), type);
                break;
            }

            default:
                throw new Error('glsVertexArrayTests.GLValue.getRandom - Invalid input type');
                break;
        }
    };

    // Minimum difference required between coordinates

    /**
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.minValue = function(type) {
        switch (type) {
            case glsVertexArrayTests.deArray.InputType.FLOAT:
            case glsVertexArrayTests.deArray.InputType.BYTE:
            case glsVertexArrayTests.deArray.InputType.HALF:
            //case glsVertexArrayTests.deArray.InputType.DOUBLE:
                return glsVertexArrayTests.GLValue.create(4, type);
            case glsVertexArrayTests.deArray.InputType.SHORT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
                return glsVertexArrayTests.GLValue.create(4 * 256, type);
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
                return glsVertexArrayTests.GLValue.create(4 * 2, type);
            /*case glsVertexArrayTests.deArray.InputType.FIXED:
                return glsVertexArrayTests.GLValue.create(4 * 512, type);*/
            case glsVertexArrayTests.deArray.InputType.INT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT:
                return glsVertexArrayTests.GLValue.create(4 * 16777216, type);

            default:
                throw new Error('glsVertexArrayTests.GLValue.minValue - Invalid input type');
        }
    };

    /**
     * @param {glsVertexArrayTests.GLValue} val
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.abs = function(val) {
        var type = val.getType();
        switch (type) {
            //case glsVertexArrayTests.deArray.InputType.FIXED:
            case glsVertexArrayTests.deArray.InputType.SHORT:
                return glsVertexArrayTests.GLValue.create(0x7FFF & val.getValue(), type);
            case glsVertexArrayTests.deArray.InputType.BYTE:
                return glsVertexArrayTests.GLValue.create(0x7F & val.getValue(), type);
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT:
                return val;
            case glsVertexArrayTests.deArray.InputType.FLOAT:
            case glsVertexArrayTests.deArray.InputType.HALF:
            //case glsVertexArrayTests.deArray.InputType.DOUBLE:
                return glsVertexArrayTests.GLValue.create(Math.abs(val.interpret()), type);
            case glsVertexArrayTests.deArray.InputType.INT:
                return glsVertexArrayTests.GLValue.create(0x7FFFFFFF & val.getValue(), type);
            default:
                throw new Error('glsVertexArrayTests.GLValue.abs - Invalid input type');
        }
    };

    /**
     * @return {glsVertexArrayTests.deArray.InputType}
     */
    glsVertexArrayTests.GLValue.prototype.getType = function() {
        return this.m_type;
    };

    /**
     * glsVertexArrayTests.GLValue.toFloat
     * @return {number}
     */
    glsVertexArrayTests.GLValue.prototype.toFloat = function() {
        return this.interpret();
    };

    /**
     * glsVertexArrayTests.GLValue.getValue
     * @return {number}
     */
    glsVertexArrayTests.GLValue.prototype.getValue = function() {
        return this.m_value[0];
    };

    /**
     * interpret function. Returns the m_value as a quantity so arithmetic operations can be performed on it
     * Only some types require this.
     * @return {number}
     */
    glsVertexArrayTests.GLValue.prototype.interpret = function() {
        if (this.m_type == glsVertexArrayTests.deArray.InputType.HALF)
            return glsVertexArrayTests.GLValue.halfToFloat(this.m_value[0]);
        /*else if (this.m_type == glsVertexArrayTests.deArray.InputType.FIXED) {
            var maxValue = 65536;
            return Math.floor((2 * this.m_value[0] + 1) / (maxValue - 1));
        }*/

        return this.m_value[0];
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.add = function(other) {
        return glsVertexArrayTests.GLValue.create(this.interpret() + other.interpret(), this.m_type);
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.mul = function(other) {
        return glsVertexArrayTests.GLValue.create(this.interpret() * other.interpret(), this.m_type);
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.div = function(other) {
        return glsVertexArrayTests.GLValue.create(this.interpret() / other.interpret(), this.m_type);
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.sub = function(other) {
        return glsVertexArrayTests.GLValue.create(this.interpret() - other.interpret(), this.m_type);
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.addToSelf = function(other) {
        this.m_value[0] = this.interpret() + other.interpret();
        return this;
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.subToSelf = function(other) {
        this.m_value[0] = this.interpret() - other.interpret();
        return this;
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.mulToSelf = function(other) {
        this.m_value[0] = this.interpret() * other.interpret();
        return this;
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {glsVertexArrayTests.GLValue}
     */
    glsVertexArrayTests.GLValue.prototype.divToSelf = function(other) {
        this.m_value[0] = this.interpret() / other.interpret();
        return this;
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {boolean}
     */
    glsVertexArrayTests.GLValue.prototype.equals = function(other) {
        return this.m_value[0] == other.getValue();
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {boolean}
     */
    glsVertexArrayTests.GLValue.prototype.lessThan = function(other) {
        return this.interpret() < other.interpret();
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {boolean}
     */
    glsVertexArrayTests.GLValue.prototype.greaterThan = function(other) {
        return this.interpret() > other.interpret();
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {boolean}
     */
    glsVertexArrayTests.GLValue.prototype.lessOrEqualThan = function(other) {
        return this.interpret() <= other.interpret();
    };

    /**
     * @param {glsVertexArrayTests.GLValue} other
     * @return {boolean}
     */
    glsVertexArrayTests.GLValue.prototype.greaterOrEqualThan = function(other) {
        return this.interpret() >= other.interpret();
    };

    /**
     * glsVertexArrayTests.RandomArrayGenerator class. Contains static methods only
     */
    glsVertexArrayTests.RandomArrayGenerator = function() {};

    /**
     * glsVertexArrayTests.RandomArrayGenerator.setData
     * @param {Uint8Array} data
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @param {deRandom.Random} rnd
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     */
    glsVertexArrayTests.RandomArrayGenerator.setData = function(data, type, rnd, min, max) {
        // Parameter type is not necessary, but we'll use it to assert the created glsVertexArrayTests.GLValue is of the correct type.
        /** @type {glsVertexArrayTests.GLValue} */ var value = glsVertexArrayTests.GLValue.getRandom(rnd, min, max);
        DE_ASSERT(value.getType() == type);

        glsVertexArrayTests.copyGLValueToArray(data, value);
    };

    /**
     * generateArray
     * @param {number} seed
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     * @param {number} count
     * @param {number} componentCount
     * @param {number} stride
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @return {ArrayBuffer}
     */
    glsVertexArrayTests.RandomArrayGenerator.generateArray = function(seed, min, max, count, componentCount, stride, type) {
        /** @type {ArrayBuffer} */ var data;
        /** @type {Uint8Array} */ var data8;

        var rnd = new deRandom.Random(seed);

        if (stride == 0)
            stride = componentCount * glsVertexArrayTests.deArray.inputTypeSize(type);

        data = new ArrayBuffer(stride * count);
        data8 = new Uint8Array(data);

        for (var vertexNdx = 0; vertexNdx < count; vertexNdx++) {
            for (var componentNdx = 0; componentNdx < componentCount; componentNdx++) {
                glsVertexArrayTests.RandomArrayGenerator.setData(data8.subarray(vertexNdx * stride + glsVertexArrayTests.deArray.inputTypeSize(type) * componentNdx), type, rnd, min, max);
            }
        }

        return data;
    };

    /* {
        static char*    generateQuads (int seed, int count, int componentCount, int offset, int stride, Array::Primitive primitive, Array::InputType type, glsVertexArrayTests.GLValue min, glsVertexArrayTests.GLValue max);
        static char*    generatePerQuad (int seed, int count, int componentCount, int stride, Array::Primitive primitive, Array::InputType type, glsVertexArrayTests.GLValue min, glsVertexArrayTests.GLValue max);

    private:
        template<typename T>
        static char*    createQuads (int seed, int count, int componentCount, int offset, int stride, Array::Primitive primitive, T min, T max);
        template<typename T>
        static char*    createPerQuads (int seed, int count, int componentCount, int stride, Array::Primitive primitive, T min, T max);
        static char*    createQuadsPacked (int seed, int count, int componentCount, int offset, int stride, Array::Primitive primitive);
    };*/

    /**
     * @param {number} seed
     * @param {number} count
     * @param {number} componentCount
     * @param {number} offset
     * @param {number} stride
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     * @param {number} scale Coordinate scaling factor
     * @return {ArrayBuffer}
     */
    glsVertexArrayTests.RandomArrayGenerator.generateQuads = function(seed, count, componentCount, offset, stride, primitive, type, min, max, scale) {
        /** @type {ArrayBuffer} */ var data;

        switch (type) {
            case glsVertexArrayTests.deArray.InputType.FLOAT:
            /*case glsVertexArrayTests.deArray.InputType.FIXED:
            case glsVertexArrayTests.deArray.InputType.DOUBLE:*/
            case glsVertexArrayTests.deArray.InputType.BYTE:
            case glsVertexArrayTests.deArray.InputType.SHORT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_BYTE:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_SHORT:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT:
            case glsVertexArrayTests.deArray.InputType.INT:
            case glsVertexArrayTests.deArray.InputType.HALF:
                data = glsVertexArrayTests.RandomArrayGenerator.createQuads(seed, count, componentCount, offset, stride, primitive, min, max, scale);
                break;

            case glsVertexArrayTests.deArray.InputType.INT_2_10_10_10:
            case glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10:
                data = glsVertexArrayTests.RandomArrayGenerator.createQuadsPacked(seed, count, componentCount, offset, stride, primitive);
                break;

            default:
                throw new Error('glsVertexArrayTests.RandomArrayGenerator.generateQuads - Invalid input type');
                break;
        }

        return data;
    };

    /**
     * @param {number} seed
     * @param {number} count
     * @param {number} componentCount
     * @param {number} offset
     * @param {number} stride
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @return {ArrayBuffer}
     */
    glsVertexArrayTests.RandomArrayGenerator.createQuadsPacked = function(seed, count, componentCount, offset, stride, primitive) {
        DE_ASSERT(componentCount == 4);

        /** @type {number} */ var quadStride = 0;

        if (stride == 0)
            stride = deMath.INT32_SIZE;

        switch (primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES:
                quadStride = stride * 6;
                break;

            default:
                throw new Error('glsVertexArrayTests.RandomArrayGenerator.createQuadsPacked - Invalid primitive');
                break;
        }

        /** @type {ArrayBuffer} */ var _data = new ArrayBuffer(offset + quadStride * (count - 1) + stride * 5 + componentCount * glsVertexArrayTests.deArray.inputTypeSize(glsVertexArrayTests.deArray.InputType.INT_2_10_10_10)); // last element must be fully in the array
        /** @type {Uint8Array} */ var resultData = new Uint8Array(_data).subarray(offset);

        /** @type {number} */ var max = 1024;
        /** @type {number} */ var min = 10;
        /** @type {number} */ var max2 = 4;

        var rnd = new deRandom.Random(seed);

        switch (primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES: {
                for (var quadNdx = 0; quadNdx < count; quadNdx++) {
                    /** @type {number} */ var x1 = min + rnd.getInt() % (max - min);
                    /** @type {number} */ var x2 = min + rnd.getInt() % (max - x1);

                    /** @type {number} */ var y1 = min + rnd.getInt() % (max - min);
                    /** @type {number} */ var y2 = min + rnd.getInt() % (max - y1);

                    /** @type {number} */ var z = min + rnd.getInt() % (max - min);
                    /** @type {number} */ var w = rnd.getInt() % max2;

                    /** @type {number} */ var val1 = (w << 30) | (z << 20) | (y1 << 10) | x1;
                    /** @type {number} */ var val2 = (w << 30) | (z << 20) | (y1 << 10) | x2;
                    /** @type {number} */ var val3 = (w << 30) | (z << 20) | (y2 << 10) | x1;

                    /** @type {number} */ var val4 = (w << 30) | (z << 20) | (y2 << 10) | x1;
                    /** @type {number} */ var val5 = (w << 30) | (z << 20) | (y1 << 10) | x2;
                    /** @type {number} */ var val6 = (w << 30) | (z << 20) | (y2 << 10) | x2;

                    glsVertexArrayTests.copyArray(resultData.subarray(quadNdx * quadStride + stride * 0), new Uint32Array([val1]));
                    glsVertexArrayTests.copyArray(resultData.subarray(quadNdx * quadStride + stride * 1), new Uint32Array([val2]));
                    glsVertexArrayTests.copyArray(resultData.subarray(quadNdx * quadStride + stride * 2), new Uint32Array([val3]));
                    glsVertexArrayTests.copyArray(resultData.subarray(quadNdx * quadStride + stride * 3), new Uint32Array([val4]));
                    glsVertexArrayTests.copyArray(resultData.subarray(quadNdx * quadStride + stride * 4), new Uint32Array([val5]));
                    glsVertexArrayTests.copyArray(resultData.subarray(quadNdx * quadStride + stride * 5), new Uint32Array([val6]));
                }

                break;
            }

            default:
                throw new Error('glsVertexArrayTests.RandomArrayGenerator.createQuadsPacked - Invalid primitive');
                break;
        }

        return _data;
    };

    /**
     * @param {number} seed
     * @param {number} count
     * @param {number} componentCount
     * @param {number} offset
     * @param {number} stride
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     * @param {number} scale Coordinate scaling factor
     * @return {ArrayBuffer}
     */
    glsVertexArrayTests.RandomArrayGenerator.createQuads = function(seed, count, componentCount, offset, stride, primitive, min, max, scale) {
        var componentStride = min.m_value.byteLength; //TODO: Fix encapsulation issue
        var quadStride = 0;
        var type = min.getType(); //Instead of using the template parameter.

        if (stride == 0)
            stride = componentCount * componentStride;
        DE_ASSERT(stride >= componentCount * componentStride);

        switch (primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES:
                quadStride = stride * 6;
                break;

            default:
                throw new Error('glsVertexArrayTests.RandomArrayGenerator.createQuads - Invalid primitive');
                break;
        }

        /** @type {ArrayBuffer} */ var _data = new ArrayBuffer(offset + quadStride * count);
        /** @type {Uint8Array} */ var resultData = new Uint8Array(_data).subarray(offset);

        var rnd = new deRandom.Random(seed);

        switch (primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES: {
                for (var quadNdx = 0; quadNdx < count; ++quadNdx) {
                    /** @type {glsVertexArrayTests.GLValue} */ var x1 = null;
                    /** @type {glsVertexArrayTests.GLValue} */ var x2 = null;
                    /** @type {glsVertexArrayTests.GLValue} */ var y1 = null;
                    /** @type {glsVertexArrayTests.GLValue} */ var y2 = null;
                    /** @type {glsVertexArrayTests.GLValue} */ var z = null;
                    /** @type {glsVertexArrayTests.GLValue} */ var w = null;

                    // attempt to find a good (i.e not extremely small) quad
                    for (var attemptNdx = 0; attemptNdx < 4; ++attemptNdx) {
                        x1 = glsVertexArrayTests.GLValue.getRandom(rnd, min, max);
                        x2 = glsVertexArrayTests.GLValue.getRandom(rnd, glsVertexArrayTests.GLValue.minValue(type), glsVertexArrayTests.GLValue.abs(max.sub(x1)));

                        y1 = glsVertexArrayTests.GLValue.getRandom(rnd, min, max);
                        y2 = glsVertexArrayTests.GLValue.getRandom(rnd, glsVertexArrayTests.GLValue.minValue(type), glsVertexArrayTests.GLValue.abs(max.sub(y1)));

                        z = (componentCount > 2) ? (glsVertexArrayTests.GLValue.getRandom(rnd, min, max)) : (glsVertexArrayTests.GLValue.create(0, type));
                        w = (componentCount > 3) ? (glsVertexArrayTests.GLValue.getRandom(rnd, min, max)) : (glsVertexArrayTests.GLValue.create(1, type));

                        // no additional components, all is good
                        if (componentCount <= 2)
                            break;

                        // The result quad is too thin?
                        if ((Math.abs(x2.interpret() + z.interpret()) < glsVertexArrayTests.GLValue.minValue(type).interpret()) ||
                            (Math.abs(y2.interpret() + w.interpret()) < glsVertexArrayTests.GLValue.minValue(type).interpret()))
                            continue;

                        // all ok
                        break;
                    }

                    x2 = x1.add(x2);
                    y2 = y1.add(y2);

                    /**
                     * Transform GL vertex coordinates so that after vertex shading the vertices will be rounded.
                     * We want to avoid quads that cover a pixel partially
                     */
                    var round = function(pos, scale, offset, range) {
                        // Perform the same transformation as the vertex shader
                        var val = (pos.interpret() + offset) * scale;
                        var half = range / 2;
                        val = val * half + half;
                        // Round it
                        val = Math.round(val);
                        // And reverse the vertex shading transformation
                        val = (val - half) / half;
                        val = val / scale - offset;
                        return glsVertexArrayTests.GLValue.create(val, pos.m_type);
                    };

                    var viewport = gl.getParameter(gl.VIEWPORT);
                    var voffset = 0;
                    if (componentCount > 2)
                        voffset = z.interpret();
                    x1 = round(x1, scale, voffset, viewport[2]);
                    x2 = round(x2, scale, voffset, viewport[2]);
                    voffset = 1;
                    if (componentCount > 3)
                        voffset = w.interpret();
                    y1 = round(y1, scale, voffset, viewport[3]);
                    y2 = round(y2, scale, voffset, viewport[3]);

                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride), x1);
                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + componentStride), y1);

                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride), x2);
                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride + componentStride), y1);

                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 2), x1);
                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 2 + componentStride), y2);

                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 3), x1);
                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 3 + componentStride), y2);

                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 4), x2);
                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 4 + componentStride), y1);

                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 5), x2);
                    glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * 5 + componentStride), y2);

                    if (componentCount > 2) {
                        for (var i = 0; i < 6; i++)
                            glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * i + componentStride * 2), z);
                    }

                    if (componentCount > 3) {
                        for (var i = 0; i < 6; i++)
                            glsVertexArrayTests.copyGLValueToArray(resultData.subarray(quadNdx * quadStride + stride * i + componentStride * 3), w);
                    }
                }

                break;
            }

            default:
                throw new Error('glsVertexArrayTests.RandomArrayGenerator.createQuads - Invalid primitive');
                break;
        }

        return _data;
    };

    /**
     * @param {number} seed
     * @param {number} count
     * @param {number} componentCount
     * @param {number} stride
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @param {glsVertexArrayTests.deArray.InputType} type
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     */
    glsVertexArrayTests.RandomArrayGenerator.generatePerQuad = function(seed, count, componentCount, stride, primitive, type, min, max) {
        /** @type {ArrayBuffer} */ var data = null;

        data = glsVertexArrayTests.RandomArrayGenerator.createPerQuads(seed, count, componentCount, stride, primitive, min, max);
        return data;
    };

    /**
     * @param {number} seed
     * @param {number} count
     * @param {number} componentCount
     * @param {number} stride
     * @param {glsVertexArrayTests.deArray.Primitive} primitive
     * @param {glsVertexArrayTests.GLValue} min
     * @param {glsVertexArrayTests.GLValue} max
     */
    glsVertexArrayTests.RandomArrayGenerator.createPerQuads = function(seed, count, componentCount, stride, primitive, min, max) {
        var rnd = new deRandom.Random(seed);

        var componentStride = min.m_value.byteLength; //TODO: Fix encapsulation issue.

        if (stride == 0)
            stride = componentStride * componentCount;

        var quadStride = 0;

        switch (primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES:
                quadStride = stride * 6;
                break;

            default:
                throw new Error('glsVertexArrayTests.RandomArrayGenerator.createPerQuads - Invalid primitive');
                break;
        }

        /** @type {ArrayBuffer} */ var data = new ArrayBuffer(count * quadStride);

        for (var quadNdx = 0; quadNdx < count; quadNdx++) {
            for (var componentNdx = 0; componentNdx < componentCount; componentNdx++) {
                /** @type {glsVertexArrayTests.GLValue} */ var val = glsVertexArrayTests.GLValue.getRandom(rnd, min, max);

                var data8 = new Uint8Array(data);
                glsVertexArrayTests.copyGLValueToArray(data8.subarray(quadNdx * quadStride + stride * 0 + componentStride * componentNdx), val);
                glsVertexArrayTests.copyGLValueToArray(data8.subarray(quadNdx * quadStride + stride * 1 + componentStride * componentNdx), val);
                glsVertexArrayTests.copyGLValueToArray(data8.subarray(quadNdx * quadStride + stride * 2 + componentStride * componentNdx), val);
                glsVertexArrayTests.copyGLValueToArray(data8.subarray(quadNdx * quadStride + stride * 3 + componentStride * componentNdx), val);
                glsVertexArrayTests.copyGLValueToArray(data8.subarray(quadNdx * quadStride + stride * 4 + componentStride * componentNdx), val);
                glsVertexArrayTests.copyGLValueToArray(data8.subarray(quadNdx * quadStride + stride * 5 + componentStride * componentNdx), val);
            }
        }

        return data;
    };

    /**
     * class glsVertexArrayTests.VertexArrayTest
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    glsVertexArrayTests.VertexArrayTest = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);

        var r = /** @type {number} */ (gl.getParameter(gl.RED_BITS));
        var g = /** @type {number} */ (gl.getParameter(gl.GREEN_BITS));
        var b = /** @type {number} */ (gl.getParameter(gl.BLUE_BITS));
        var a = /** @type {number} */ (gl.getParameter(gl.ALPHA_BITS));
        this.m_pixelformat = new tcuPixelFormat.PixelFormat(r, g, b, a);

        /** @type {sglrReferenceContext.ReferenceContextBuffers} */ this.m_refBuffers = null;
        /** @type {sglrReferenceContext.ReferenceContext} */ this.m_refContext = null;
        /** @type {sglrGLContext.GLContext} */ this.m_glesContext = null;
        /** @type {glsVertexArrayTests.ContextArrayPack} */ this.m_glArrayPack = null;
        /** @type {glsVertexArrayTests.ContextArrayPack} */ this.m_rrArrayPack = null;
        /** @type {boolean} */ this.m_isOk = false;
        /** @type {number} */ this.m_maxDiffRed = Math.ceil(256.0 * (2.0 / (1 << this.m_pixelformat.redBits)));
        /** @type {number} */ this.m_maxDiffGreen = Math.ceil(256.0 * (2.0 / (1 << this.m_pixelformat.greenBits)));
        /** @type {number} */ this.m_maxDiffBlue = Math.ceil(256.0 * (2.0 / (1 << this.m_pixelformat.blueBits)));
    };

    glsVertexArrayTests.VertexArrayTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsVertexArrayTests.VertexArrayTest.prototype.constructor = glsVertexArrayTests.VertexArrayTest;

    /**
     * init
     */
    glsVertexArrayTests.VertexArrayTest.prototype.init = function() {
        /** @type {number}*/ var renderTargetWidth = Math.min(512, canvas.width);
        /** @type {number}*/ var renderTargetHeight = Math.min(512, canvas.height);
        /** @type {sglrReferenceContext.ReferenceContextLimits} */ var limits = new sglrReferenceContext.ReferenceContextLimits(gl);

        this.m_glesContext = new sglrGLContext.GLContext(gl);
        this.m_refBuffers = new sglrReferenceContext.ReferenceContextBuffers(this.m_pixelformat, 0, 0, renderTargetWidth, renderTargetHeight);
        this.m_refContext = new sglrReferenceContext.ReferenceContext(limits, this.m_refBuffers.getColorbuffer(), this.m_refBuffers.getDepthbuffer(), this.m_refBuffers.getStencilbuffer());

        this.m_glArrayPack = new glsVertexArrayTests.ContextArrayPack(this.m_glesContext);
        this.m_rrArrayPack = new glsVertexArrayTests.ContextArrayPack(this.m_refContext);
    };

    /**
     * compare
     */
    glsVertexArrayTests.VertexArrayTest.prototype.compare = function() {
        /** @type {tcuSurface.Surface} */ var ref = this.m_rrArrayPack.getSurface();
        /** @type {tcuSurface.Surface} */ var screen = this.m_glArrayPack.getSurface();

        if (/** @type {number} */ (this.m_glesContext.getParameter(gl.SAMPLES)) > 1) {
            // \todo [mika] Improve compare when using multisampling
            bufferedLogToConsole('Warning: Comparison of result from multisample render targets are not as strict as without multisampling. Might produce false positives!');
            this.m_isOk = tcuImageCompare.fuzzyCompare('Compare Results', 'Compare Results', ref.getAccess(), screen.getAccess(), 1.5);
        } else {
            /** @type {tcuRGBA.RGBA} */ var threshold = tcuRGBA.newRGBAComponents(this.m_maxDiffRed, this.m_maxDiffGreen, this.m_maxDiffBlue, 255);
            /** @type {tcuSurface.Surface} */ var error = new tcuSurface.Surface(ref.getWidth(), ref.getHeight());

            this.m_isOk = true;

            for (var y = 1; y < ref.getHeight() - 1; y++) {
                for (var x = 1; x < ref.getWidth() - 1; x++) {
                    /** @type {tcuRGBA.RGBA} */ var refPixel = tcuRGBA.newRGBAFromArray(ref.getPixel(x, y));
                    /** @type {tcuRGBA.RGBA} */ var screenPixel = tcuRGBA.newRGBAFromArray(screen.getPixel(x, y));
                    /** @type {boolean} */ var isOkPixel = false;

                    // Don't do comparisons for this pixel if it belongs to a one-pixel-thin part (i.e. it doesn't have similar-color neighbors in both x and y directions) in both result and reference.
                    // This fixes some false negatives.
                    /** @type {boolean} */ var refThin = (
                        !tcuRGBA.compareThreshold(refPixel, tcuRGBA.newRGBAFromArray(ref.getPixel(x - 1, y)), threshold) &&
                        !tcuRGBA.compareThreshold(refPixel, tcuRGBA.newRGBAFromArray(ref.getPixel(x + 1, y)), threshold)
                    ) || (
                        !tcuRGBA.compareThreshold(refPixel, tcuRGBA.newRGBAFromArray(ref.getPixel(x, y - 1)), threshold) &&
                        !tcuRGBA.compareThreshold(refPixel, tcuRGBA.newRGBAFromArray(ref.getPixel(x, y + 1)), threshold)
                    );

                    /** @type {boolean} */ var screenThin = (
                        !tcuRGBA.compareThreshold(screenPixel, tcuRGBA.newRGBAFromArray(screen.getPixel(x - 1, y)), threshold) &&
                        !tcuRGBA.compareThreshold(screenPixel, tcuRGBA.newRGBAFromArray(screen.getPixel(x + 1, y)), threshold)
                    ) || (
                        !tcuRGBA.compareThreshold(screenPixel, tcuRGBA.newRGBAFromArray(screen.getPixel(x, y - 1)), threshold) &&
                        !tcuRGBA.compareThreshold(screenPixel, tcuRGBA.newRGBAFromArray(screen.getPixel(x, y + 1)), threshold)
                    );

                    if (refThin && screenThin)
                        isOkPixel = true;
                    else {
                        //NOTE: This will ignore lines less than three pixels wide, so
                        //even if there's a difference, the test will pass.
                        for (var dy = -1; dy < 2 && !isOkPixel; dy++) {
                            for (var dx = -1; dx < 2 && !isOkPixel; dx++) {
                                // Check reference pixel against screen pixel
                                /** @type {tcuRGBA.RGBA} */ var screenCmpPixel = tcuRGBA.newRGBAFromArray(screen.getPixel(x + dx, y + dy));
                                /** @type {number} (8-bit) */ var r = Math.abs(refPixel.getRed() - screenCmpPixel.getRed());
                                /** @type {number} (8-bit) */ var g = Math.abs(refPixel.getGreen() - screenCmpPixel.getGreen());
                                /** @type {number} (8-bit) */ var b = Math.abs(refPixel.getBlue() - screenCmpPixel.getBlue());

                                if (r <= this.m_maxDiffRed && g <= this.m_maxDiffGreen && b <= this.m_maxDiffBlue)
                                    isOkPixel = true;

                                // Check screen pixels against reference pixel
                                /** @type {tcuRGBA.RGBA} */ var refCmpPixel = tcuRGBA.newRGBAFromArray(ref.getPixel(x + dx, y + dy));
                                r = Math.abs(refCmpPixel.getRed() - screenPixel.getRed());
                                g = Math.abs(refCmpPixel.getGreen() - screenPixel.getGreen());
                                b = Math.abs(refCmpPixel.getBlue() - screenPixel.getBlue());

                                    if (r <= this.m_maxDiffRed && g <= this.m_maxDiffGreen && b <= this.m_maxDiffBlue)
                                        isOkPixel = true;
                            }
                        }
                    }

                    if (isOkPixel)
                        error.setPixel(x, y,
                            [tcuRGBA.newRGBAFromArray(screen.getPixel(x, y)).getRed(),
                            (tcuRGBA.newRGBAFromArray(screen.getPixel(x, y)).getGreen() + 255) / 2,
                            tcuRGBA.newRGBAFromArray(screen.getPixel(x, y)).getBlue(), 255]
                        );
                    else {
                        error.setPixel(x, y, [255, 0, 0, 255]);
                        this.m_isOk = false;
                    }
                }
            }

            if (!this.m_isOk) {
                debug('Image comparison failed, threshold = (' + this.m_maxDiffRed + ', ' + this.m_maxDiffGreen + ', ' + this.m_maxDiffBlue + ')');
                //log << TestLog::ImageSet("Compare result", "Result of rendering");
                tcuImageCompare.displayImages(screen.getAccess(), ref.getAccess(), error.getAccess());
            } else {
                //log << TestLog::ImageSet("Compare result", "Result of rendering")
                tcuLogImage.logImage('Result', '', screen.getAccess());
            }
        }
    };

    //TODO: Is this actually used? -> glsVertexArrayTests.VertexArrayTest& operator= (const glsVertexArrayTests.VertexArrayTest& other);

    /**
     * glsVertexArrayTests.MultiVertexArrayTest class
     * @constructor
     * @extends {glsVertexArrayTests.VertexArrayTest}
     * @param {glsVertexArrayTests.MultiVertexArrayTest.Spec} spec
     * @param {string} name
     * @param {string} desc
     */
    glsVertexArrayTests.MultiVertexArrayTest = function(spec, name, desc) {
        glsVertexArrayTests.VertexArrayTest.call(this, name, desc);

        /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec} */ this.m_spec = spec;
        /** @type {number} */ this.m_iteration = 0;
    };

    glsVertexArrayTests.MultiVertexArrayTest.prototype = Object.create(glsVertexArrayTests.VertexArrayTest.prototype);
    glsVertexArrayTests.MultiVertexArrayTest.prototype.constructor = glsVertexArrayTests.MultiVertexArrayTest;

    /**
     * glsVertexArrayTests.MultiVertexArrayTest.Spec class
     * @constructor
     */
    glsVertexArrayTests.MultiVertexArrayTest.Spec = function() {
        /** @type {glsVertexArrayTests.deArray.Primitive} */ this.primitive;
        /** @type {number} */ this.drawCount = 0;
        /** @type {number} */ this.first = 0;
        /** @type {Array<glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec>} */ this.arrays = [];
    };

    /**
     * glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec class
     * @constructor
     * @param {glsVertexArrayTests.deArray.InputType} inputType_
     * @param {glsVertexArrayTests.deArray.OutputType} outputType_
     * @param {glsVertexArrayTests.deArray.Storage} storage_
     * @param {glsVertexArrayTests.deArray.Usage} usage_
     * @param {number} componentCount_
     * @param {number} offset_
     * @param {number} stride_
     * @param {boolean} normalize_
     * @param {glsVertexArrayTests.GLValue} min_
     * @param {glsVertexArrayTests.GLValue} max_
     */
    glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec = function(inputType_, outputType_, storage_, usage_, componentCount_, offset_, stride_, normalize_, min_, max_) {
        this.inputType = inputType_;
        this.outputType = outputType_;
        this.storage = storage_;
        this.usage = usage_;
        this.componentCount = componentCount_;
        this.offset = offset_;
        /** @type {number} */ this.stride = stride_;
        this.normalize = normalize_;
        this.min = min_;
        this.max = max_;
    };

    /**
     * getName
     * @return {string}
     */
    glsVertexArrayTests.MultiVertexArrayTest.Spec.prototype.getName = function() {
        var name = '';

        for (var ndx = 0; ndx < this.arrays.length; ++ndx) {
            /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec} */ var array = this.arrays[ndx];

            if (this.arrays.length > 1)
                name += 'array' + ndx + '_';

            name += glsVertexArrayTests.deArray.storageToString(array.storage) + '_' +
            array.offset + '_' +
            array.stride + '_' +
            glsVertexArrayTests.deArray.inputTypeToString(array.inputType);

            if (array.inputType != glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 && array.inputType != glsVertexArrayTests.deArray.InputType.INT_2_10_10_10)
                name += array.componentCount;
            name += '_' +
            (array.normalize ? 'normalized_' : '') +
            glsVertexArrayTests.deArray.outputTypeToString(array.outputType) + '_' +
            glsVertexArrayTests.deArray.usageTypeToString(array.usage) + '_';
        }

        if (this.first)
            name += 'first' + this.first + '_';

        switch (this.primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES:
                name += 'quads_';
                break;
            case glsVertexArrayTests.deArray.Primitive.POINTS:
                name += 'points_';
                break;

            default:
                throw new Error('glsVertexArrayTests.MultiVertexArrayTest.Spec.getName - Invalid primitive type');
                break;
        }

        name += this.drawCount;

        return name;
    };

    /**
     * getName
     * @return {string}
     */
    glsVertexArrayTests.MultiVertexArrayTest.Spec.prototype.getDesc = function() {
        var desc = '';

        for (var ndx = 0; ndx < this.arrays.length; ++ndx) {
            /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec} */ var array = this.arrays[ndx];

            desc += 'Array ' + ndx + ': ' +
            'Storage in ' + glsVertexArrayTests.deArray.storageToString(array.storage) + ', ' +
            'stride ' + array.stride + ', ' +
            'input datatype ' + glsVertexArrayTests.deArray.inputTypeToString(array.inputType) + ', ' +
            'input component count ' + array.componentCount + ', ' +
            (array.normalize ? 'normalized, ' : '') +
            'used as ' + glsVertexArrayTests.deArray.outputTypeToString(array.outputType) + ', ';
        }

        desc += 'drawArrays(), ' +
        'first ' + this.first + ', ' +
        this.drawCount;

        switch (this.primitive) {
            case glsVertexArrayTests.deArray.Primitive.TRIANGLES:
                desc += 'quads ';
                break;
            case glsVertexArrayTests.deArray.Primitive.POINTS:
                desc += 'points';
                break;

            default:
                throw new Error('glsVertexArrayTests.MultiVertexArrayTest.Spec.getDesc - Invalid primitive type');
                break;
        }

        return desc;
    };

    /**
     * iterate
     * @return {tcuTestCase.IterateResult}
     */
    glsVertexArrayTests.MultiVertexArrayTest.prototype.iterate = function() {
        if (this.m_iteration == 0) {
            var primitiveSize = (this.m_spec.primitive == glsVertexArrayTests.deArray.Primitive.TRIANGLES) ? (6) : (1); // in non-indexed draw Triangles means rectangles
            var coordScale = 1.0;
            var colorScale = 1.0;
            var useVao = true; // WebGL, WebGL 2.0 - gl.getType().getProfile() == glu::PROFILE_CORE;

            // Log info
            bufferedLogToConsole(this.m_spec.getDesc());

            // Color and Coord scale

            // First array is always position
            /** @type {glsVertexArrayTests.MultiVertexArrayTest.Spec.ArraySpec} */ var arraySpec = this.m_spec.arrays[0];
            if (arraySpec.inputType == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10) {
                if (arraySpec.normalize)
                    coordScale = 1;
                else
                    coordScale = 1 / 1024;
            } else if (arraySpec.inputType == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10) {
                if (arraySpec.normalize)
                    coordScale = 1.0;
                else
                    coordScale = 1.0 / 512.0;
            } else
                coordScale = arraySpec.normalize && !glsVertexArrayTests.inputTypeIsFloatType(arraySpec.inputType) ? 1.0 : 0.9 / arraySpec.max.toFloat();

            if (arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.VEC3 || arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.VEC4 ||
                arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.IVEC3 || arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.IVEC4 ||
                arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.UVEC3 || arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.UVEC4)
                coordScale = coordScale * 0.5;

            // And other arrays are color-like
            for (var arrayNdx = 1; arrayNdx < this.m_spec.arrays.length; arrayNdx++) {
                arraySpec = this.m_spec.arrays[arrayNdx];

                colorScale *= (arraySpec.normalize && !glsVertexArrayTests.inputTypeIsFloatType(arraySpec.inputType) ? 1.0 : 1.0 / arraySpec.max.toFloat());
                if (arraySpec.outputType == glsVertexArrayTests.deArray.OutputType.VEC4)
                    colorScale *= (arraySpec.normalize && !glsVertexArrayTests.inputTypeIsFloatType(arraySpec.inputType) ? 1.0 : 1.0 / arraySpec.max.toFloat());
            }

            // Data

            for (var arrayNdx = 0; arrayNdx < this.m_spec.arrays.length; arrayNdx++) {
                arraySpec = this.m_spec.arrays[arrayNdx];
                /** @type {number} */ var seed = arraySpec.inputType + 10 * arraySpec.outputType + 100 * arraySpec.storage + 1000 * this.m_spec.primitive + 10000 * arraySpec.usage + this.m_spec.drawCount + 12 * arraySpec.componentCount + arraySpec.stride + arraySpec.normalize;
                /** @type {ArrayBuffer} */ var data = null;
                /** @type {number} */ var stride = arraySpec.stride == 0 ? arraySpec.componentCount * glsVertexArrayTests.deArray.inputTypeSize(arraySpec.inputType) : arraySpec.stride;
                /** @type {number} */ var bufferSize = arraySpec.offset + stride * (this.m_spec.drawCount * primitiveSize - 1) + arraySpec.componentCount * glsVertexArrayTests.deArray.inputTypeSize(arraySpec.inputType);

                switch (this.m_spec.primitive) {
                    //          case glsVertexArrayTests.deArray.Primitive.POINTS:
                    //              data = glsVertexArrayTests.RandomArrayGenerator.generateArray(seed, arraySpec.min, arraySpec.max, arraySpec.count, arraySpec.componentCount, arraySpec.stride, arraySpec.inputType);
                    //              break;
                    case glsVertexArrayTests.deArray.Primitive.TRIANGLES:
                        if (arrayNdx == 0) {
                            data = glsVertexArrayTests.RandomArrayGenerator.generateQuads(seed, this.m_spec.drawCount, arraySpec.componentCount, arraySpec.offset, arraySpec.stride, this.m_spec.primitive, arraySpec.inputType, arraySpec.min, arraySpec.max, coordScale);
                        } else {
                            DE_ASSERT(arraySpec.offset == 0); // \note [jarkko] it just hasn't been implemented
                            data = glsVertexArrayTests.RandomArrayGenerator.generatePerQuad(seed, this.m_spec.drawCount, arraySpec.componentCount, arraySpec.stride, this.m_spec.primitive, arraySpec.inputType, arraySpec.min, arraySpec.max);
                        }
                        break;

                    default:
                        throw new Error('glsVertexArrayTests.MultiVertexArrayTest.prototype.iterate - Invalid primitive type');
                        break;
                }

                this.m_glArrayPack.newArray(arraySpec.storage);
                this.m_rrArrayPack.newArray(arraySpec.storage);

                this.m_glArrayPack.getArray(arrayNdx).data(glsVertexArrayTests.deArray.Target.ARRAY, bufferSize, new Uint8Array(data), arraySpec.usage);
                this.m_rrArrayPack.getArray(arrayNdx).data(glsVertexArrayTests.deArray.Target.ARRAY, bufferSize, new Uint8Array(data), arraySpec.usage);

                this.m_glArrayPack.getArray(arrayNdx).bind(arrayNdx, arraySpec.offset, arraySpec.componentCount, arraySpec.inputType, arraySpec.outputType, arraySpec.normalize, arraySpec.stride);
                this.m_rrArrayPack.getArray(arrayNdx).bind(arrayNdx, arraySpec.offset, arraySpec.componentCount, arraySpec.inputType, arraySpec.outputType, arraySpec.normalize, arraySpec.stride);
            }

            try {
                this.m_glArrayPack.render(this.m_spec.primitive, this.m_spec.first, this.m_spec.drawCount * primitiveSize, useVao, coordScale, colorScale);
                this.m_rrArrayPack.render(this.m_spec.primitive, this.m_spec.first, this.m_spec.drawCount * primitiveSize, useVao, coordScale, colorScale);
            }
            catch (err) {
                // GL Errors are ok if the mode is not properly aligned

                bufferedLogToConsole('Got error: ' + err.message);

                if (this.isUnalignedBufferOffsetTest())
                    testFailedOptions('Failed to draw with unaligned buffers', false); // TODO: QP_TEST_RESULT_COMPATIBILITY_WARNING
                else if (this.isUnalignedBufferStrideTest())
                    testFailedOptions('Failed to draw with unaligned stride', false); // QP_TEST_RESULT_COMPATIBILITY_WARNING
                else
                    throw new Error(err.message);

                return tcuTestCase.IterateResult.STOP;
            }

            this.m_iteration++;
            return tcuTestCase.IterateResult.CONTINUE;
        } else if (this.m_iteration == 1) {
            this.compare();

            if (this.m_isOk) {
                testPassedOptions('', true);
            } else {
                if (this.isUnalignedBufferOffsetTest())
                    testFailedOptions('Failed to draw with unaligned buffers', false); // QP_TEST_RESULT_COMPATIBILITY_WARNING
                else if (this.isUnalignedBufferStrideTest())
                    testFailedOptions('Failed to draw with unaligned stride', false); // QP_TEST_RESULT_COMPATIBILITY_WARNING
                else
                    testFailedOptions('Image comparison failed', false);
            }

            this.m_iteration++;
            return tcuTestCase.IterateResult.STOP;
        } else {
            testFailedOptions('glsVertexArrayTests.MultiVertexArrayTest.iterate - Invalid iteration stage', false);
            return tcuTestCase.IterateResult.STOP;
        }
    };

    /**
     * isUnalignedBufferOffsetTest
     * @return {boolean}
     */
    glsVertexArrayTests.MultiVertexArrayTest.prototype.isUnalignedBufferOffsetTest = function() {
        // Buffer offsets should be data type size aligned
        for (var i = 0; i < this.m_spec.arrays.length; ++i) {
            if (this.m_spec.arrays[i].storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
                /** @type {boolean} */ var inputTypePacked = this.m_spec.arrays[i].inputType == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 || this.m_spec.arrays[i].inputType == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10;

                /** @type {number} */ var dataTypeSize = glsVertexArrayTests.deArray.inputTypeSize(this.m_spec.arrays[i].inputType);
                if (inputTypePacked)
                    dataTypeSize = 4;

                if (this.m_spec.arrays[i].offset % dataTypeSize != 0)
                    return true;
            }
        }
        return false;
    };

    /**
     * isUnalignedBufferStrideTest
     * @return {boolean}
     */
    glsVertexArrayTests.MultiVertexArrayTest.prototype.isUnalignedBufferStrideTest = function() {
        // Buffer strides should be data type size aligned
        for (var i = 0; i < this.m_spec.arrays.length; ++i) {
            if (this.m_spec.arrays[i].storage == glsVertexArrayTests.deArray.Storage.BUFFER) {
                /** @type {boolean} */ var inputTypePacked = this.m_spec.arrays[i].inputType == glsVertexArrayTests.deArray.InputType.UNSIGNED_INT_2_10_10_10 || this.m_spec.arrays[i].inputType == glsVertexArrayTests.deArray.InputType.INT_2_10_10_10;

                /** @type {number} */ var dataTypeSize = glsVertexArrayTests.deArray.inputTypeSize(this.m_spec.arrays[i].inputType);
                if (inputTypePacked)
                    dataTypeSize = 4;

                if (this.m_spec.arrays[i].stride % dataTypeSize != 0)
                    return true;
            }
        }
        return false;
    };

});
