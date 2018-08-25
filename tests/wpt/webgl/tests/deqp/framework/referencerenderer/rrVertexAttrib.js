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
goog.provide('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.common.tcuFloat');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.referencerenderer.rrGenericVector');

goog.scope(function() {

var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
var deMath = framework.delibs.debase.deMath;
var tcuFloat = framework.common.tcuFloat;
var rrGenericVector = framework.referencerenderer.rrGenericVector;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * rrVertexAttrib.NormalOrder
     * @enum
     */
    rrVertexAttrib.NormalOrder = {
        T0: 0,
        T1: 1,
        T2: 2,
        T3: 3
    };

    /**
     * rrVertexAttrib.BGRAOrder
     * @enum
     */
    rrVertexAttrib.BGRAOrder = {
        T0: 2,
        T1: 1,
        T2: 0,
        T3: 3
    };

    /**
     * rrVertexAttrib.VertexAttribType enum
     * @enum
     */
    rrVertexAttrib.VertexAttribType = {
        // Can only be rrVertexAttrib.read as floats
        FLOAT: 0,
        HALF: 1,
        FIXED: 2,
        DOUBLE: 3,

        // Can only be rrVertexAttrib.read as floats, will be normalized
        NONPURE_UNORM8: 4,
        NONPURE_UNORM16: 5,
        NONPURE_UNORM32: 6,
        NONPURE_UNORM_2_10_10_10_REV: 7, //!< Packed format, only size = 4 is allowed

        // Clamped formats, GLES3-style conversion: max{c / (2^(b-1) - 1), -1 }
        NONPURE_SNORM8_CLAMP: 8,
        NONPURE_SNORM16_CLAMP: 9,
        NONPURE_SNORM32_CLAMP: 10,
        NONPURE_SNORM_2_10_10_10_REV_CLAMP: 11, //!< Packed format, only size = 4 is allowed

        // Scaled formats, GLES2-style conversion: (2c + 1) / (2^b - 1)
        NONPURE_SNORM8_SCALE: 12,
        NONPURE_SNORM16_SCALE: 13,
        NONPURE_SNORM32_SCALE: 14,
        NONPURE_SNORM_2_10_10_10_REV_SCALE: 15, //!< Packed format, only size = 4 is allowed

        // can only be rrVertexAttrib.read as float, will not be normalized
        NONPURE_UINT8: 16,
        NONPURE_UINT16: 17,
        NONPURE_UINT32: 18,

        NONPURE_INT8: 19,
        NONPURE_INT16: 20,
        NONPURE_INT32: 21,

        NONPURE_UINT_2_10_10_10_REV: 22, //!< Packed format, only size = 4 is allowed
        NONPURE_INT_2_10_10_10_REV: 23, //!< Packed format, only size = 4 is allowed

        // can only be rrVertexAttrib.read as integers
        PURE_UINT8: 24,
        PURE_UINT16: 25,
        PURE_UINT32: 26,

        PURE_INT8: 27,
        PURE_INT16: 28,
        PURE_INT32: 29,

        // reordered formats of gl.ARB_vertex_array_bgra
        NONPURE_UNORM8_BGRA: 30,
        NONPURE_UNORM_2_10_10_10_REV_BGRA: 31,
        NONPURE_SNORM_2_10_10_10_REV_CLAMP_BGRA: 32,
        NONPURE_SNORM_2_10_10_10_REV_SCALE_BGRA: 33,

        // can be rrVertexAttrib.read as anything
        DONT_CARE: 34 //!< Do not enforce type checking when reading GENERIC attribute. Used for current client side attributes.
    };

    /**
     * rrVertexAttrib.VertexAttrib class
     * @constructor
     */
    rrVertexAttrib.VertexAttrib = function() {
        /** @type {rrVertexAttrib.VertexAttribType} */ this.type = rrVertexAttrib.VertexAttribType.FLOAT;
        /** @type {number} */ this.size = 0;
        /** @type {number} */ this.stride = 0;
        /** @type {number} */ this.instanceDivisor = 0;
        /** @type {number} */ this.offset = 0; //Added this property to compensate functionality (not in original dEQP).
        /** @type {ArrayBuffer} */ this.pointer = null;
        /** @type {Array<number>|rrGenericVector.GenericVec4} */ this.generic; //!< Generic attribute, used if pointer is null.
    };

    /**
     * @param {rrVertexAttrib.VertexAttribType} type
     * @return {number}
     */
    rrVertexAttrib.getComponentSize = function(type) {
        switch (type) {
            case rrVertexAttrib.VertexAttribType.FLOAT: return 4;
            case rrVertexAttrib.VertexAttribType.HALF: return 2;
            case rrVertexAttrib.VertexAttribType.FIXED: return 4;
            case rrVertexAttrib.VertexAttribType.DOUBLE: return 8; //sizeof(double);
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM8: return 1;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM16: return 2;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM32: return 4;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM8_CLAMP: return 1;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM16_CLAMP: return 2;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM32_CLAMP: return 4;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM8_SCALE: return 1;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM16_SCALE: return 2;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM32_SCALE: return 4;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT8: return 1;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT16: return 2;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT32: return 4;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT8: return 1;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT16: return 2;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT32: return 4;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT_2_10_10_10_REV: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT_2_10_10_10_REV: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.PURE_UINT8: return 1;
            case rrVertexAttrib.VertexAttribType.PURE_UINT16: return 2;
            case rrVertexAttrib.VertexAttribType.PURE_UINT32: return 4;
            case rrVertexAttrib.VertexAttribType.PURE_INT8: return 1;
            case rrVertexAttrib.VertexAttribType.PURE_INT16: return 2;
            case rrVertexAttrib.VertexAttribType.PURE_INT32: return 4;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM8_BGRA: return 1;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV_BGRA: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP_BGRA: return 1; //sizeof(deUint32)/4;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE_BGRA: return 1; //sizeof(deUint32)/4;
            default:
                throw new Error('rrVertexAttrib.getComponentSize - Invalid type');
        }
    };

    /**
     * rrVertexAttrib.isValidVertexAttrib function
     * @param {rrVertexAttrib.VertexAttrib} vertexAttrib
     * @return {boolean}
     */
    rrVertexAttrib.isValidVertexAttrib = function(vertexAttrib) {
        // Trivial range checks.
        if (!deMath.deInBounds32(vertexAttrib.type, 0, Object.keys(rrVertexAttrib.VertexAttribType).length) ||
            !deMath.deInRange32(vertexAttrib.size, 0, 4) ||
            vertexAttrib.instanceDivisor < 0)
            return false;

        // Generic attributes
        if (!vertexAttrib.pointer && vertexAttrib.type != rrVertexAttrib.VertexAttribType.DONT_CARE)
            return false;

        // Packed formats
        if ((vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_INT_2_10_10_10_REV ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_UINT_2_10_10_10_REV ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV_BGRA ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP_BGRA ||
            vertexAttrib.type == rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE_BGRA) &&
            vertexAttrib.size != 4)
            return false;

        return true;
    };

    /**
     * rrVertexAttrib.readVertexAttrib function
     * @param {rrVertexAttrib.VertexAttrib} vertexAttrib
     * @param {number} instanceNdx
     * @param {number} vertexNdx
     * @param {rrGenericVector.GenericVecType} genericType
     * @return {goog.NumberArray}
     */
    rrVertexAttrib.readVertexAttrib = function(vertexAttrib, instanceNdx, vertexNdx, genericType) {
        DE_ASSERT(rrVertexAttrib.isValidVertexAttrib(vertexAttrib));
        /** @type {goog.NumberArray} */ var dst;

        var arrayType = null;
        switch (genericType) {
            case rrGenericVector.GenericVecType.INT32:
                arrayType = Int32Array;
                break;
            case rrGenericVector.GenericVecType.UINT32:
                arrayType = Uint32Array;
                break;
            case rrGenericVector.GenericVecType.FLOAT:
                arrayType = Float32Array;
                break;
        }

        if (vertexAttrib.pointer) {
            /** @type {number} */ var elementNdx = (vertexAttrib.instanceDivisor != 0) ? (instanceNdx / vertexAttrib.instanceDivisor) : vertexNdx;
            /** @type {number} */ var compSize = rrVertexAttrib.getComponentSize(vertexAttrib.type);
            /** @type {number} */ var stride = (vertexAttrib.stride != 0) ? (vertexAttrib.stride) : (vertexAttrib.size * compSize);
            /** @type {number} */ var byteOffset = vertexAttrib.offset + (elementNdx * stride);

            dst = [0, 0, 0, 1]; // defaults

            if (arrayType != null) {
                dst = new arrayType(dst);
            }

            rrVertexAttrib.read(dst, vertexAttrib.type, vertexAttrib.size, new Uint8Array(vertexAttrib.pointer, byteOffset));
        } else {
            dst = new arrayType(/** @type {Array<number>} */ vertexAttrib.generic.data);
        }

        return dst;
    };

    /**
     * rrVertexAttrib.readHalf
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    rrVertexAttrib.readHalf = function(dst, size, ptr) {
        var arraysize16 = 2; //2 bytes

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arraysize16)); //Small buffer copy (max. 8 bytes)
        var aligned = new Uint16Array(ptrclone.buffer);

        //Reinterpret aligned's values into the dst vector.
        dst[0] = tcuFloat.newFloat32From16(aligned[0]).getValue();
        if (size >= 2) dst[1] = tcuFloat.newFloat32From16(aligned[1]).getValue();
        if (size >= 3) dst[2] = tcuFloat.newFloat32From16(aligned[2]).getValue();
        if (size >= 4) dst[3] = tcuFloat.newFloat32From16(aligned[3]).getValue();
    };

    /**
     * rrVertexAttrib.readFixed
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    /*rrVertexAttrib.readFixed = function(dst, size, ptr) {
        var arraysize32 = 4; //4 bytes

        //Reinterpret ptr as a uint16 array,
        //assuming original ptr is 8-bits per element
        var aligned = new Int32Array(ptr.buffer).subarray(
            ptr.byteOffset / arraysize32,
            (ptr.byteOffset + ptr.byteLength) / arraysize32);

        //Reinterpret aligned's values into the dst vector.
        dst[0] = aligned[0] / (1 << 16);
        if (size >= 2) dst[1] = aligned[1] / (1 << 16);
        if (size >= 3) dst[2] = aligned[2] / (1 << 16);
        if (size >= 4) dst[3] = aligned[3] / (1 << 16);
    };*/

    /**
     * TODO: Check 64 bit numbers are handled ok
     * rrVertexAttrib.readDouble
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    /*rrVertexAttrib.readDouble = function(dst, size, ptr) {
        var arraysize64 = 8; //8 bytes

        //Reinterpret 'ptr' into 'aligned' as a float64 array,
        //assuming original ptr is 8-bits per element.
        var aligned = new Float64Array(ptr.buffer).subarray(
            ptr.byteOffset / arraysize64,
            (ptr.byteOffset + ptr.byteLength) / arraysize64);

        //Reinterpret aligned's values into the dst vector.
        dst[0] = aligned[0];
        if (size >= 2) dst[1] = aligned[1];
        if (size >= 3) dst[2] = aligned[2];
        if (size >= 4) dst[3] = aligned[3];
    };*/

    /**
     * extendSign
     * @param {number} integerLen
     * @param {number} integer_ (deUint32)
     * @return {number} (deInt32)
     */
    rrVertexAttrib.extendSign = function(integerLen, integer_) {
        return new Int32Array([
            deMath.binaryOp(
                0 -
                deMath.shiftLeft(
                    deMath.binaryOp(
                        integer_,
                        deMath.shiftLeft(
                            1,
                            (integerLen - 1)
                        ),
                        deMath.BinaryOp.AND
                    ),
                    1
                ) ,

integer_,
                deMath.BinaryOp.OR
            )
        ])[0];
    };

    /**
     * rrVertexAttrib.readUint2101010Rev
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    rrVertexAttrib.readUint2101010Rev = function(dst, size, ptr) {
        var arraysize32 = 4; //4 bytes

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arraysize32)); //Small buffer copy (max. 16 bytes)
        var aligned = new Uint32Array(ptrclone.buffer)[0];

        dst[0] = deMath.binaryOp(deMath.shiftRight(aligned, 0), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND);
        if (size >= 2) dst[1] = deMath.binaryOp(deMath.shiftRight(aligned, 10), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND);
        if (size >= 3) dst[2] = deMath.binaryOp(deMath.shiftRight(aligned, 20), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND);
        if (size >= 4) dst[3] = deMath.binaryOp(deMath.shiftRight(aligned, 30), deMath.shiftLeft(1, 2) - 1, deMath.BinaryOp.AND);
    };

    /**
     * rrVertexAttrib.readInt2101010Rev
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    rrVertexAttrib.readInt2101010Rev = function(dst, size, ptr) {
        var arraysize32 = 4; //4 bytes

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arraysize32)); //Small buffer copy (max. 16 bytes)
        var aligned = new Uint32Array(ptrclone.buffer)[0];

        dst[0] = rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 0), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND));
        if (size >= 2) dst[1] = rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 10), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND));
        if (size >= 3) dst[2] = rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 20), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND));
        if (size >= 4) dst[3] = rrVertexAttrib.extendSign(2, deMath.binaryOp(deMath.shiftRight(aligned, 30), deMath.shiftLeft(1, 2) - 1, deMath.BinaryOp.AND));
    };

    /**
     * rrVertexAttrib.readUnorm2101010RevOrder
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {Object<rrVertexAttrib.NormalOrder|rrVertexAttrib.BGRAOrder>} order
     */
    rrVertexAttrib.readUnorm2101010RevOrder = function(dst, size, ptr, order) {
        var arraysize32 = 4; //4 bytes

        //Left shift within 32-bit range as 32-bit int.
        var range10 = new Uint32Array([deMath.shiftLeft(1, 10) - 1])[0];
        var range2 = new Uint32Array([deMath.shiftLeft(1, 2) - 1])[0];

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arraysize32)); //Small buffer copy (max. 16 bytes)
        var aligned = new Uint32Array(ptrclone.buffer)[0];

        dst[order.T0] = deMath.binaryOp(deMath.shiftRight(aligned, 0), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND) / range10;
        if (size >= 2) dst[order.T1] = deMath.binaryOp(deMath.shiftRight(aligned, 10), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND) / range10;
        if (size >= 3) dst[order.T2] = deMath.binaryOp(deMath.shiftRight(aligned, 20), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND) / range10;
        if (size >= 4) dst[order.T3] = deMath.binaryOp(deMath.shiftRight(aligned, 30), deMath.shiftLeft(1, 2) - 1, deMath.BinaryOp.AND) / range2;
    };

    /**
     * rrVertexAttrib.readSnorm2101010RevClampOrder
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {Object<rrVertexAttrib.NormalOrder|rrVertexAttrib.BGRAOrder>} order
     */
    rrVertexAttrib.readSnorm2101010RevClampOrder = function(dst, size, ptr, order) {
        var arraysize32 = 4; //4 bytes

        //Left shift within 32-bit range as 32-bit int.
        var range10 = new Uint32Array([deMath.shiftLeft(1, 10 - 1) - 1])[0];
        var range2 = new Uint32Array([deMath.shiftLeft(1, 2 - 1) - 1])[0];

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arraysize32)); //Small buffer copy (max. 16 bytes)
        var aligned = new Uint32Array(ptrclone.buffer)[0];

        dst[order.T0] = Math.max(-1.0, new Float32Array([rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 0), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND))])[0] / range10);
        if (size >= 2) dst[order.T1] = Math.max(-1.0, new Float32Array([rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 10), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND))])[0] / range10);
        if (size >= 3) dst[order.T2] = Math.max(-1.0, new Float32Array([rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 20), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND))])[0] / range10);
        if (size >= 4) dst[order.T3] = Math.max(-1.0, new Float32Array([rrVertexAttrib.extendSign(2, deMath.binaryOp(deMath.shiftRight(aligned, 30), deMath.shiftLeft(1, 2) - 1, deMath.BinaryOp.AND))])[0] / range2);
    };

    /**
     * rrVertexAttrib.readSnorm2101010RevScaleOrder
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {Object<rrVertexAttrib.NormalOrder|rrVertexAttrib.BGRAOrder>} order
     */
    rrVertexAttrib.readSnorm2101010RevScaleOrder = function(dst, size, ptr, order) {
        var arraysize32 = 4; //4 bytes

        //Left shift within 32-bit range as 32-bit int.
        var range10 = new Uint32Array([deMath.shiftLeft(1, 10) - 1])[0];
        var range2 = new Uint32Array([deMath.shiftLeft(1, 2) - 1])[0];

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arraysize32)); //Small buffer copy (max. 16 bytes)
        var aligned = new Uint32Array(ptrclone.buffer)[0];

        dst[order.T0] = new Float32Array([rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 0), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND)) * 2.0 + 1.0])[0] / range10;
        if (size >= 2) dst[order.T1] = new Float32Array([rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 10), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND)) * 2.0 + 1.0])[0] / range10;
        if (size >= 3) dst[order.T2] = new Float32Array([rrVertexAttrib.extendSign(10, deMath.binaryOp(deMath.shiftRight(aligned, 20), deMath.shiftLeft(1, 10) - 1, deMath.BinaryOp.AND)) * 2.0 + 1.0])[0] / range10;
        if (size >= 4) dst[order.T3] = new Float32Array([rrVertexAttrib.extendSign(2, deMath.binaryOp(deMath.shiftRight(aligned, 30), deMath.shiftLeft(1, 2) - 1, deMath.BinaryOp.AND)) * 2.0 + 1.0])[0] / range2;
    };

    /**
     * rrVertexAttrib.readUnormOrder
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {Object<rrVertexAttrib.NormalOrder|rrVertexAttrib.BGRAOrder>} order
     * @param readAsTypeArray
     */
    rrVertexAttrib.readUnormOrder = function(dst, size, ptr, order, readAsTypeArray) {
        var arrayelementsize = readAsTypeArray.BYTES_PER_ELEMENT;

        //Left shift within 32-bit range as 32-bit float.
        var range = new Float32Array([deMath.shiftLeft(1, arrayelementsize * 8) - 1])[0];

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arrayelementsize)); //Small buffer copy (max. 16 bytes)
        var aligned = new readAsTypeArray(ptrclone.buffer);

        //Reinterpret aligned's values into the dst vector.
        dst[order.T0] = aligned[0] / range;
        if (size >= 2) dst[order.T1] = aligned[1] / range;
        if (size >= 3) dst[order.T2] = aligned[2] / range;
        if (size >= 4) dst[order.T3] = aligned[3] / range;
    };

    /**
     * rrVertexAttrib.readSnormClamp
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {function(new:ArrayBufferView,(Array<number>|ArrayBuffer|ArrayBufferView|null|number), number=, number=)} readAsTypeArray
     */
    rrVertexAttrib.readSnormClamp = function(dst, size, ptr, readAsTypeArray) {
        var arrayelementsize = readAsTypeArray.BYTES_PER_ELEMENT;

        //Left shift within 32-bit range as 32-bit float.
        var range = new Float32Array([deMath.shiftLeft(1, arrayelementsize * 8 - 1) - 1])[0];

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arrayelementsize)); //Small buffer copy (max. 16 bytes)
        var aligned = new readAsTypeArray(ptrclone.buffer);

        //Reinterpret aligned's values into the dst vector.
        dst[0] = Math.max(-1, aligned[0] / range);
        if (size >= 2) dst[1] = Math.max(-1, aligned[1] / range);
        if (size >= 3) dst[2] = Math.max(-1, aligned[2] / range);
        if (size >= 4) dst[3] = Math.max(-1, aligned[3] / range);
    };

    /**
     * rrVertexAttrib.readOrder
     * @param {goog.NumberArray} dst
     * @param {number} size
     * @param {Uint8Array} ptr
     * @param {Object<rrVertexAttrib.NormalOrder|rrVertexAttrib.BGRAOrder>} order NormalOrder or BGRAOrder
     * @param readAsTypeArray Typed Array type
     */
    rrVertexAttrib.readOrder = function(dst, size, ptr, order, readAsTypeArray) {
        var arrayelementsize = readAsTypeArray.BYTES_PER_ELEMENT;

        var ptrclone = new Uint8Array(ptr.subarray(0, size * arrayelementsize)); //Small buffer copy (max. 16 bytes)
        var aligned = new readAsTypeArray(ptrclone.buffer);

        //Reinterpret aligned's values into the dst vector.
        //(automatic in JS typed arrays).
        dst[order.T0] = aligned[0];
        if (size >= 2) dst[order.T1] = aligned[1];
        if (size >= 3) dst[order.T2] = aligned[2];
        if (size >= 4) dst[order.T3] = aligned[3];
    };

    /**
     * TODO: Implement readSNormScale.
     * @param {goog.NumberArray} dst
     * @param {rrVertexAttrib.VertexAttribType} type
     * @param {number} size
     * @param {Uint8Array} ptr
     */
    rrVertexAttrib.read = function(dst, type, size, ptr) {
        var order;

        switch (type) {
            case rrVertexAttrib.VertexAttribType.FLOAT:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Float32Array);
                break;
            case rrVertexAttrib.VertexAttribType.HALF:
                rrVertexAttrib.readHalf(dst, size, ptr);
                break;
            /*case rrVertexAttrib.VertexAttribType.FIXED:
                rrVertexAttrib.readFixed(dst, size, ptr);
                break;
            case rrVertexAttrib.VertexAttribType.DOUBLE:
                rrVertexAttrib.readDouble(dst, size, ptr);
                break;*/
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM8:
                rrVertexAttrib.readUnormOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint8Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM16:
                rrVertexAttrib.readUnormOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint16Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM32:
                rrVertexAttrib.readUnormOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint32Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV:
                rrVertexAttrib.readUnorm2101010RevOrder(dst, size, ptr, rrVertexAttrib.NormalOrder);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM8_CLAMP: //Int8
                rrVertexAttrib.readSnormClamp(dst, size, ptr, Int8Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM16_CLAMP: //Int16
                rrVertexAttrib.readSnormClamp(dst, size, ptr, Int16Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM32_CLAMP: //Int32
                rrVertexAttrib.readSnormClamp(dst, size, ptr, Int32Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP:
                rrVertexAttrib.readSnorm2101010RevClampOrder(dst, size, ptr, rrVertexAttrib.NormalOrder);
                break;
            /*case rrVertexAttrib.VertexAttribType.NONPURE_SNORM8_SCALE: //Int8
                rrVertexAttrib.readSnormScale(dst, size, ptr, Int8Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM16_SCALE: //Int16
                rrVertexAttrib.readSnormScale(dst, size, ptr, Int16Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM32_SCALE: //Int32
                rrVertexAttrib.readSnormScale(dst, size, ptr, Int32Array);
                break;*/
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE:
                rrVertexAttrib.readSnorm2101010RevScaleOrder(dst, size, ptr, rrVertexAttrib.NormalOrder);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT_2_10_10_10_REV:
                rrVertexAttrib.readUint2101010Rev(dst, size, ptr);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT_2_10_10_10_REV:
                rrVertexAttrib.readInt2101010Rev(dst, size, ptr);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM8_BGRA:
                rrVertexAttrib.readUnormOrder(dst, size, ptr, rrVertexAttrib.BGRAOrder, Uint8Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UNORM_2_10_10_10_REV_BGRA:
                rrVertexAttrib.readUnorm2101010RevOrder(dst, size, ptr, rrVertexAttrib.BGRAOrder);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_CLAMP_BGRA:
                rrVertexAttrib.readSnorm2101010RevClampOrder(dst, size, ptr, rrVertexAttrib.BGRAOrder);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_SNORM_2_10_10_10_REV_SCALE_BGRA:
                rrVertexAttrib.readSnorm2101010RevScaleOrder(dst, size, ptr, rrVertexAttrib.BGRAOrder);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT8:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint8Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT16:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint16Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_UINT32:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint32Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT8:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Int8Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT16:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Int16Array);
                break;
            case rrVertexAttrib.VertexAttribType.NONPURE_INT32:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Int32Array);
                break;
            case rrVertexAttrib.VertexAttribType.PURE_UINT8:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint8Array);
                break;
            case rrVertexAttrib.VertexAttribType.PURE_UINT16:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint16Array);
                break;
            case rrVertexAttrib.VertexAttribType.PURE_UINT32:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Uint32Array);
                break;
            case rrVertexAttrib.VertexAttribType.PURE_INT8:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Int8Array);
                break;
            case rrVertexAttrib.VertexAttribType.PURE_INT16:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Int16Array);
                break;
            case rrVertexAttrib.VertexAttribType.PURE_INT32:
                rrVertexAttrib.readOrder(dst, size, ptr, rrVertexAttrib.NormalOrder, Int32Array);
                break;

            default:
                throw new Error('rrVertexAttrib.read - Invalid type');
        }
    };

});
