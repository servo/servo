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
goog.provide('functional.gles3.es3fUniformApiTests');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTexture');
goog.require('framework.opengl.gluVarType');

goog.scope(function() {

    var es3fUniformApiTests = functional.gles3.es3fUniformApiTests;
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

    /** @type {WebGL2RenderingContext} */ var gl;

    /** @typedef {function(gluShaderUtil.DataType): boolean} */
    es3fUniformApiTests.dataTypePredicate;

    /** @type {number} */ es3fUniformApiTests.MAX_RENDER_WIDTH = 32;
    /** @type {number} */ es3fUniformApiTests.MAX_RENDER_HEIGHT = 32;
    /** @type {number} */ es3fUniformApiTests.MAX_NUM_SAMPLER_UNIFORMS = 16;

    /** @type {Array<gluShaderUtil.DataType>} */ es3fUniformApiTests.s_testDataTypes = [
    gluShaderUtil.DataType.FLOAT,
    gluShaderUtil.DataType.FLOAT_VEC2,
    gluShaderUtil.DataType.FLOAT_VEC3,
    gluShaderUtil.DataType.FLOAT_VEC4,
    gluShaderUtil.DataType.FLOAT_MAT2,
    gluShaderUtil.DataType.FLOAT_MAT2X3,
    gluShaderUtil.DataType.FLOAT_MAT2X4,
    gluShaderUtil.DataType.FLOAT_MAT3X2,
    gluShaderUtil.DataType.FLOAT_MAT3,
    gluShaderUtil.DataType.FLOAT_MAT3X4,
    gluShaderUtil.DataType.FLOAT_MAT4X2,
    gluShaderUtil.DataType.FLOAT_MAT4X3,
    gluShaderUtil.DataType.FLOAT_MAT4,

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

    gluShaderUtil.DataType.SAMPLER_2D,
    gluShaderUtil.DataType.SAMPLER_CUBE
        // \note We don't test all sampler types here.
    ];

    /**
     * Returns a substring from the beginning to the last occurence of the
     * specified character
     * @param {string} str The string in which to search
     * @param {string} c A single character
     * @return {string}
     */
    es3fUniformApiTests.beforeLast = function(str, c) {
        return str.substring(0, str.lastIndexOf(c));
    };

    /**
     * es3fUniformApiTests.fillWithColor
     * @param {tcuTexture.PixelBufferAccess} access ,
     * @param {Array<number>} color Array of four color components.
     */
    es3fUniformApiTests.fillWithColor = function(access, color) {
        for (var z = 0; z < access.getDepth(); z++)
        for (var y = 0; y < access.getHeight(); y++)
        for (var x = 0; x < access.getWidth(); x++)
            access.setPixel(color, x, y, z);
    };

    /**
     * @param {gluShaderUtil.DataType} type
     * @return {number}
     */
    es3fUniformApiTests.getSamplerNumLookupDimensions = function(type) {
        switch (type) {
            case gluShaderUtil.DataType.SAMPLER_2D:
            case gluShaderUtil.DataType.INT_SAMPLER_2D:
            case gluShaderUtil.DataType.UINT_SAMPLER_2D:
                return 2;

            case gluShaderUtil.DataType.SAMPLER_3D:
            case gluShaderUtil.DataType.INT_SAMPLER_3D:
            case gluShaderUtil.DataType.UINT_SAMPLER_3D:
            case gluShaderUtil.DataType.SAMPLER_2D_SHADOW:
            case gluShaderUtil.DataType.SAMPLER_2D_ARRAY:
            case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY:
            case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY:
            case gluShaderUtil.DataType.SAMPLER_CUBE:
            case gluShaderUtil.DataType.INT_SAMPLER_CUBE:
            case gluShaderUtil.DataType.UINT_SAMPLER_CUBE:
                return 3;

            case gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW:
            case gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW:
                return 4;

            default:
                throw new Error('es3fUniformApiTests.getSamplerNumLookupDimensions - Invalid type');
        }
    };

   /**
    * @param {gluShaderUtil.DataType} type
    * @return {gluShaderUtil.DataType}
    */
    es3fUniformApiTests.getSamplerLookupReturnType = function(type) {
        switch (type) {
            case gluShaderUtil.DataType.SAMPLER_2D:
            case gluShaderUtil.DataType.SAMPLER_CUBE:
            case gluShaderUtil.DataType.SAMPLER_2D_ARRAY:
            case gluShaderUtil.DataType.SAMPLER_3D:
                return gluShaderUtil.DataType.FLOAT_VEC4;

            case gluShaderUtil.DataType.UINT_SAMPLER_2D:
            case gluShaderUtil.DataType.UINT_SAMPLER_CUBE:
            case gluShaderUtil.DataType.UINT_SAMPLER_2D_ARRAY:
            case gluShaderUtil.DataType.UINT_SAMPLER_3D:
                return gluShaderUtil.DataType.UINT_VEC4;

            case gluShaderUtil.DataType.INT_SAMPLER_2D:
            case gluShaderUtil.DataType.INT_SAMPLER_CUBE:
            case gluShaderUtil.DataType.INT_SAMPLER_2D_ARRAY:
            case gluShaderUtil.DataType.INT_SAMPLER_3D:
                return gluShaderUtil.DataType.INT_VEC4;

            case gluShaderUtil.DataType.SAMPLER_2D_SHADOW:
            case gluShaderUtil.DataType.SAMPLER_CUBE_SHADOW:
            case gluShaderUtil.DataType.SAMPLER_2D_ARRAY_SHADOW:
                return gluShaderUtil.DataType.FLOAT;

            default:
                throw new Error('es3fUniformApiTests.getSamplerLookupReturnType - Invalid type');
        }
    };

    /**
     * @param {gluShaderUtil.DataType} T DataType to compare the type. Used to be a template param
     * @param {gluShaderUtil.DataType} t
     * @return {boolean}
     */
    es3fUniformApiTests.dataTypeEquals = function(T, t) {
        return t == T;
    };

    /**
     * @param {number} N Row number. Used to be a template parameter
     * @param {gluShaderUtil.DataType} t
     * @return {boolean}
     */
    es3fUniformApiTests.dataTypeIsMatrixWithNRows = function(N, t) {
        return gluShaderUtil.isDataTypeMatrix(t) && gluShaderUtil.getDataTypeMatrixNumRows(t) == N;
    };

   /**
    * @param {gluVarType.VarType} type
    * @param {es3fUniformApiTests.dataTypePredicate} predicate
    * @return {boolean}
    */
    es3fUniformApiTests.typeContainsMatchingBasicType = function(type, predicate) {
        if (type.isBasicType())
            return predicate(type.getBasicType());
        else if (type.isArrayType())
            return es3fUniformApiTests.typeContainsMatchingBasicType(type.getElementType(), predicate);
        else {
            assertMsgOptions(type.isStructType(), 'es3fUniformApiTests.typeContainsMatchingBasicType - not a struct type', false, true);
            /** @type {gluVarType.StructType} */ var structType = type.getStruct();
            for (var i = 0; i < structType.getSize(); i++)
                if (es3fUniformApiTests.typeContainsMatchingBasicType(structType.getMember(i).getType(), predicate))
                    return true;
            return false;
        }
    };

    /**
     * @param {Array<gluShaderUtil.DataType>} dst
     * @param {gluVarType.VarType} type
     */
    es3fUniformApiTests.getDistinctSamplerTypes = function(dst, type) {
        if (type.isBasicType()) {
            /** @type {gluShaderUtil.DataType} */ var basicType = type.getBasicType();
            if (gluShaderUtil.isDataTypeSampler(basicType) && dst.indexOf(basicType) == -1)
                dst.push(basicType);
        } else if (type.isArrayType())
            es3fUniformApiTests.getDistinctSamplerTypes(dst, type.getElementType());
        else {
            assertMsgOptions(type.isStructType(), 'es3fUniformApiTests.getDistinctSamplerTypes - not a struct type', false, true);
            /** @type {gluVarType.StructType} */ var structType = type.getStruct();
            for (var i = 0; i < structType.getSize(); i++)
                es3fUniformApiTests.getDistinctSamplerTypes(dst, structType.getMember(i).getType());
        }
    };

    /**
     * @param {gluVarType.VarType} type
     * @return {number}
     */
    es3fUniformApiTests.getNumSamplersInType = function(type) {
        if (type.isBasicType())
            return gluShaderUtil.isDataTypeSampler(type.getBasicType()) ? 1 : 0;
        else if (type.isArrayType())
            return es3fUniformApiTests.getNumSamplersInType(type.getElementType()) * type.getArraySize();
        else {
            assertMsgOptions(type.isStructType(), 'es3fUniformApiTests.getNumSamplersInType - not a struct type', false, true);
            /** @type {gluVarType.StructType} */ var structType = type.getStruct();
            /** @type {number} */ var sum = 0;
            for (var i = 0; i < structType.getSize(); i++)
                sum += es3fUniformApiTests.getNumSamplersInType(structType.getMember(i).getType());
            return sum;
        }
    };

    /** @typedef { {type: gluVarType.VarType, ndx: number}} */
    es3fUniformApiTests.VarTypeWithIndex;

    /**
     * @param {number} maxDepth
     * @param {number} curStructIdx Out parameter, instead returning it in the VarTypeWithIndex structure.
     * @param {Array<gluVarType.StructType>} structTypesDst
     * @param {deRandom.Random} rnd
     * @return {es3fUniformApiTests.VarTypeWithIndex}
     */
    es3fUniformApiTests.generateRandomType = function(maxDepth, curStructIdx, structTypesDst, rnd) {
        /** @type {boolean} */ var isStruct = maxDepth > 0 && rnd.getFloat() < 0.2;
        /** @type {boolean} */ var isArray = rnd.getFloat() < 0.3;

        if (isStruct) {
            /** @type {number} */ var numMembers = rnd.getInt(1, 5);
            /** @type {gluVarType.StructType} */ var structType = gluVarType.newStructType('structType' + curStructIdx++);

            for (var i = 0; i < numMembers; i++) {
                /** @type {es3fUniformApiTests.VarTypeWithIndex} */ var typeWithIndex = es3fUniformApiTests.generateRandomType(maxDepth - 1, curStructIdx, structTypesDst, rnd);
                curStructIdx = typeWithIndex.ndx;
                structType.addMember('m' + i, typeWithIndex.type);
            }

            structTypesDst.push(structType);
            return (isArray ? {
                type: gluVarType.newTypeArray(gluVarType.newTypeStruct(structType), rnd.getInt(1, 5)),
                ndx: curStructIdx
            }
            : {
                type: gluVarType.newTypeStruct(structType),
                ndx: curStructIdx
            });
        } else {
            /** @type {gluShaderUtil.DataType} */ var basicType = es3fUniformApiTests.s_testDataTypes[rnd.getInt(0, es3fUniformApiTests.s_testDataTypes.length - 1)];
            /** @type {gluShaderUtil.precision} */ var precision;
            if (!gluShaderUtil.isDataTypeBoolOrBVec(basicType))
                precision = gluShaderUtil.precision.PRECISION_MEDIUMP;
            return (isArray ? {
                type: gluVarType.newTypeArray(gluVarType.newTypeBasic(basicType, precision), rnd.getInt(1, 5)),
                ndx: curStructIdx
            }
            : {
                type: gluVarType.newTypeBasic(basicType, precision),
                ndx: curStructIdx
            });
        }
    };

    /**
     * es3fUniformApiTests.SamplerV structure
     * @constructor
     */
    es3fUniformApiTests.SamplerV = function() {
        this.samplerV = {
            /** @type {number} */ unit: 0,
            /** @type {Array<number>} */ fillColor: []
        };
    };

    /**
     * es3fUniformApiTests.VarValue class. may contain different types.
     * @constructor
     */
    es3fUniformApiTests.VarValue = function() {
        /** @type {gluShaderUtil.DataType} */ this.type;
        /** @type {Array<number | boolean> | es3fUniformApiTests.SamplerV} */ this.val = [];
    };

    /**
     * @enum {number}
     */
    es3fUniformApiTests.CaseShaderType = {
        VERTEX: 0,
        FRAGMENT: 1,
        BOTH: 2
    };

    /**
     * es3fUniformApiTests.Uniform struct.
     * @param {string} name_
     * @param {gluVarType.VarType} type_
     * @constructor
     */
    es3fUniformApiTests.Uniform = function(name_, type_) {
        /** @type {string} */ this.name = name_;
        /** @type {gluVarType.VarType} */ this.type = type_;
    };

    // A set of uniforms, along with related struct types.
    /**
     * class es3fUniformApiTests.UniformCollection
     * @constructor
     */
    es3fUniformApiTests.UniformCollection = function() {
        /** @type {Array<es3fUniformApiTests.Uniform>} */ this.m_uniforms = [];
        /** @type {Array<gluVarType.StructType>} */ this.m_structTypes = [];
    };

    /**
     * @return {number}
     */
    es3fUniformApiTests.UniformCollection.prototype.getNumUniforms = function() {return this.m_uniforms.length;};

    /**
     * @return {number}
     */
    es3fUniformApiTests.UniformCollection.prototype.getNumStructTypes = function() {return this.m_structTypes.length;};

    /**
     * @param {number} ndx
     * @return {es3fUniformApiTests.Uniform}
     */
    es3fUniformApiTests.UniformCollection.prototype.getUniform = function(ndx) {return this.m_uniforms[ndx];};

    /**
     * @param {number} ndx
     * @return {gluVarType.StructType}
     */
    es3fUniformApiTests.UniformCollection.prototype.getStructType = function(ndx) {return this.m_structTypes[ndx];};

    /**
     * @param {es3fUniformApiTests.Uniform} uniform
     */
    es3fUniformApiTests.UniformCollection.prototype.addUniform = function(uniform) {this.m_uniforms.push(uniform);};

    /**
     * @param {gluVarType.StructType} type
     */
    es3fUniformApiTests.UniformCollection.prototype.addStructType = function(type) {this.m_structTypes.push(type);};

    // Add the contents of m_uniforms and m_structTypes to receiver, and remove them from this one.
    // \note receiver takes ownership of the struct types.
    /**
     * @param {es3fUniformApiTests.UniformCollection} receiver
     */
    es3fUniformApiTests.UniformCollection.prototype.moveContents = function(receiver) {
        for (var i = 0; i < this.m_uniforms.length; i++)
            receiver.addUniform(this.m_uniforms[i]);
        this.m_uniforms.length = 0;

        for (var i = 0; i < this.m_structTypes.length; i++)
            receiver.addStructType(this.m_structTypes[i]);
        this.m_structTypes.length = 0;
    };

    /**
     * @param {es3fUniformApiTests.dataTypePredicate} predicate
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCollection.prototype.containsMatchingBasicType = function(predicate) {
        for (var i = 0; i < this.m_uniforms.length; i++)
            if (es3fUniformApiTests.typeContainsMatchingBasicType(this.m_uniforms[i].type, predicate))
                return true;
        return false;
    };

    /**
     * @return {Array<gluShaderUtil.DataType>}
     */
    es3fUniformApiTests.UniformCollection.prototype.getSamplerTypes = function() {
        /** @type {Array<gluShaderUtil.DataType>} */ var samplerTypes = [];
        for (var i = 0; i < this.m_uniforms.length; i++)
            es3fUniformApiTests.getDistinctSamplerTypes(samplerTypes, this.m_uniforms[i].type);
        return samplerTypes;
    };

    /**
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCollection.prototype.containsSeveralSamplerTypes = function() {
        return this.getSamplerTypes().length > 1;
    };

    /**
     * @return {number}
     */
    es3fUniformApiTests.UniformCollection.prototype.getNumSamplers = function() {
        var sum = 0;
        for (var i = 0; i < this.m_uniforms.length; i++)
            sum += es3fUniformApiTests.getNumSamplersInType(this.m_uniforms[i].type);
        return sum;
    };

    /**
     * @param {gluShaderUtil.DataType} type
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.basic = function(type, nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();
        /** @type {gluShaderUtil.precision} */ var prec;
        if (!gluShaderUtil.isDataTypeBoolOrBVec(type))
            prec = gluShaderUtil.precision.PRECISION_MEDIUMP;
        res.m_uniforms.push(new es3fUniformApiTests.Uniform('u_var' + nameSuffix, gluVarType.newTypeBasic(type, prec)));
        return res;
    };

    /**
     * @param {gluShaderUtil.DataType} type
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.basicArray = function(type, nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();
        /** @type {gluShaderUtil.precision} */ var prec;
        if (!gluShaderUtil.isDataTypeBoolOrBVec(type))
            prec = gluShaderUtil.precision.PRECISION_MEDIUMP;
        res.m_uniforms.push(new es3fUniformApiTests.Uniform('u_var' + nameSuffix, gluVarType.newTypeArray(gluVarType.newTypeBasic(type, prec), 3)));
        return res;
    };

    /**
     * @param {gluShaderUtil.DataType} type0
     * @param {gluShaderUtil.DataType} type1
     * @param {boolean} containsArrays
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.basicStruct = function(type0, type1, containsArrays, nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();
        /** @type {gluShaderUtil.precision} */ var prec0;
        if (!gluShaderUtil.isDataTypeBoolOrBVec(type0))
            prec0 = gluShaderUtil.precision.PRECISION_MEDIUMP;
        /** @type {gluShaderUtil.precision} */ var prec1;
        if (!gluShaderUtil.isDataTypeBoolOrBVec(type1))
            prec1 = gluShaderUtil.precision.PRECISION_MEDIUMP;

        /** @type {gluVarType.StructType} */ var structType = gluVarType.newStructType('structType' + nameSuffix);
        structType.addMember('m0', gluVarType.newTypeBasic(type0, prec0));
        structType.addMember('m1', gluVarType.newTypeBasic(type1, prec1));
        if (containsArrays) {
            structType.addMember('m2', gluVarType.newTypeArray(gluVarType.newTypeBasic(type0, prec0), 3));
            structType.addMember('m3', gluVarType.newTypeArray(gluVarType.newTypeBasic(type1, prec1), 3));
        }

        res.addStructType(structType);
        res.addUniform(new es3fUniformApiTests.Uniform('u_var' + nameSuffix, gluVarType.newTypeStruct(structType)));

        return res;
    };

    /**
     * @param {gluShaderUtil.DataType} type0
     * @param {gluShaderUtil.DataType} type1
     * @param {boolean} containsArrays
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.structInArray = function(type0, type1, containsArrays, nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = es3fUniformApiTests.UniformCollection.basicStruct(type0, type1, containsArrays, nameSuffix);
        res.getUniform(0).type = gluVarType.newTypeArray(res.getUniform(0).type, 3);
        return res;
    };

    /**
     * @param {gluShaderUtil.DataType} type0
     * @param {gluShaderUtil.DataType} type1
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.nestedArraysStructs = function(type0, type1, nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();
        /** @type {gluShaderUtil.precision} */ var prec0;
        if (!gluShaderUtil.isDataTypeBoolOrBVec(type0))
            prec0 = gluShaderUtil.precision.PRECISION_MEDIUMP;
        /** @type {gluShaderUtil.precision} */ var prec1;
        if (!gluShaderUtil.isDataTypeBoolOrBVec(type1))
            prec1 = gluShaderUtil.precision.PRECISION_MEDIUMP;
        /** @type {gluVarType.StructType} */ var structType = gluVarType.newStructType('structType' + nameSuffix);
        /** @type {gluVarType.StructType} */ var subStructType = gluVarType.newStructType('subStructType' + nameSuffix);
        /** @type {gluVarType.StructType} */ var subSubStructType = gluVarType.newStructType('subSubStructType' + nameSuffix);

        subSubStructType.addMember('mss0', gluVarType.newTypeBasic(type0, prec0));
        subSubStructType.addMember('mss1', gluVarType.newTypeBasic(type1, prec1));

        subStructType.addMember('ms0', gluVarType.newTypeBasic(type1, prec1));
        subStructType.addMember('ms1', gluVarType.newTypeArray(gluVarType.newTypeBasic(type0, prec0), 2));
        subStructType.addMember('ms2', gluVarType.newTypeArray(gluVarType.newTypeStruct(subSubStructType), 2));

        structType.addMember('m0', gluVarType.newTypeBasic(type0, prec0));
        structType.addMember('m1', gluVarType.newTypeStruct(subStructType));
        structType.addMember('m2', gluVarType.newTypeBasic(type1, prec1));

        res.addStructType(subSubStructType);
        res.addStructType(subStructType);
        res.addStructType(structType);

        res.addUniform(new es3fUniformApiTests.Uniform('u_var' + nameSuffix, gluVarType.newTypeStruct(structType)));

        return res;
    };

    /**
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.multipleBasic = function(nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {Array<gluShaderUtil.DataType>} */ var types = [gluShaderUtil.DataType.FLOAT, gluShaderUtil.DataType.INT_VEC3, gluShaderUtil.DataType.UINT_VEC4, gluShaderUtil.DataType.FLOAT_MAT3, gluShaderUtil.DataType.BOOL_VEC2];
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();

        for (var i = 0; i < types.length; i++) {
            /** @type {es3fUniformApiTests.UniformCollection} */ var sub = es3fUniformApiTests.UniformCollection.basic(types[i], '_' + i + nameSuffix);
            sub.moveContents(res);
        }

        return res;
    };

    /**
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.multipleBasicArray = function(nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {Array<gluShaderUtil.DataType>} */ var types = [gluShaderUtil.DataType.FLOAT, gluShaderUtil.DataType.INT_VEC3, gluShaderUtil.DataType.BOOL_VEC2];
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();

        for (var i = 0; i < types.length; i++) {
            /** @type {es3fUniformApiTests.UniformCollection} */ var sub = es3fUniformApiTests.UniformCollection.basicArray(types[i], '_' + i + nameSuffix);
            sub.moveContents(res);
        }

        return res;
    };

    /**
     * @param {string=} nameSuffix
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.multipleNestedArraysStructs = function(nameSuffix) {
        if (nameSuffix === undefined) nameSuffix = '';
        /** @type {Array<gluShaderUtil.DataType>} */ var types0 = [gluShaderUtil.DataType.FLOAT, gluShaderUtil.DataType.INT, gluShaderUtil.DataType.BOOL_VEC4];
        /** @type {Array<gluShaderUtil.DataType>} */ var types1 = [gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.DataType.INT_VEC4, gluShaderUtil.DataType.BOOL];
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();

        assertMsgOptions(types0.length == types1.length, 'es3fUniformApiTests.UniformCollection.multipleNestedArraysStructs - lengths are not the same', false, true);

        for (var i = 0; i < types0.length; i++) {
            /** @type {es3fUniformApiTests.UniformCollection} */ var sub = es3fUniformApiTests.UniformCollection.nestedArraysStructs(types0[i], types1[i], '_' + i + nameSuffix);
            sub.moveContents(res);
        }

        return res;
    };

    /**
     * @param {number} seed
     * @return {es3fUniformApiTests.UniformCollection}
     */
    es3fUniformApiTests.UniformCollection.random = function(seed) {
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(seed);
        /** @type {number} */ var numUniforms = rnd.getInt(1, 5);
        /** @type {number} */ var structIdx = 0;
        /** @type {es3fUniformApiTests.UniformCollection} */ var res = new es3fUniformApiTests.UniformCollection();

        for (var i = 0; i < numUniforms; i++) {
            /** @type {Array<gluVarType.StructType>} */ var structTypes = [];
            /** @type {es3fUniformApiTests.Uniform} */ var uniform = new es3fUniformApiTests.Uniform('u_var' + i, new gluVarType.VarType());

            // \note Discard uniforms that would cause number of samplers to exceed es3fUniformApiTests.MAX_NUM_SAMPLER_UNIFORMS.
            do {
                var temp = es3fUniformApiTests.generateRandomType(3, structIdx, structTypes, rnd);
                structIdx = temp.ndx;
                uniform.type = temp.type;
            } while (res.getNumSamplers() + es3fUniformApiTests.getNumSamplersInType(uniform.type) > es3fUniformApiTests.MAX_NUM_SAMPLER_UNIFORMS);

            res.addUniform(uniform);
            for (var j = 0; j < structTypes.length; j++)
                res.addStructType(structTypes[j]);
        }

        return res;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} sampler
     * @return {es3fUniformApiTests.VarValue}
     */
    es3fUniformApiTests.getSamplerFillValue = function(sampler) {
        assertMsgOptions(gluShaderUtil.isDataTypeSampler(sampler.type), 'es3fUniformApiTests.getSamplerFillValue - not a sampler type', false, true);

        /** @type {es3fUniformApiTests.VarValue} */ var result = new es3fUniformApiTests.VarValue();
        result.type = es3fUniformApiTests.getSamplerLookupReturnType(sampler.type);

        switch (result.type) {
            case gluShaderUtil.DataType.FLOAT_VEC4:
                for (var i = 0; i < 4; i++)
                    result.val[i] = sampler.val.samplerV.fillColor[i];
                break;
            case gluShaderUtil.DataType.UINT_VEC4:
                for (var i = 0; i < 4; i++)
                    result.val[i] = sampler.val.samplerV.fillColor[i];
                break;
            case gluShaderUtil.DataType.INT_VEC4:
                for (var i = 0; i < 4; i++)
                    result.val[i] = sampler.val.samplerV.fillColor[i];
                break;
            case gluShaderUtil.DataType.FLOAT:
                result.val[0] = sampler.val.samplerV.fillColor[0];
                break;
            default:
                throw new Error('es3fUniformApiTests.getSamplerFillValue - Invalid type');
        }

        return result;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} sampler
     * @return {es3fUniformApiTests.VarValue}
     */
    es3fUniformApiTests.getSamplerUnitValue = function(sampler) {
        assertMsgOptions(gluShaderUtil.isDataTypeSampler(sampler.type), 'es3fUniformApiTests.getSamplerUnitValue - not a sampler type', false, true);

        /** @type {es3fUniformApiTests.VarValue} */ var result = new es3fUniformApiTests.VarValue();
        result.type = gluShaderUtil.DataType.INT;
        result.val[0] = sampler.val.samplerV.unit;

        return result;
    };

    /**
     * @param {gluShaderUtil.DataType} original
     * @return {gluShaderUtil.DataType}
     */
    es3fUniformApiTests.getDataTypeTransposedMatrix = function(original) {
        return gluShaderUtil.getDataTypeMatrix(gluShaderUtil.getDataTypeMatrixNumRows(original), gluShaderUtil.getDataTypeMatrixNumColumns(original));
    };

    /**
     * @param {es3fUniformApiTests.VarValue} original
     * @return {es3fUniformApiTests.VarValue}
     */
    es3fUniformApiTests.getTransposeMatrix = function(original) {
        assertMsgOptions(gluShaderUtil.isDataTypeMatrix(original.type), 'es3fUniformApiTests.getTransposeMatrix - not a matrix', false, true);

        /** @type {number} */ var rows = gluShaderUtil.getDataTypeMatrixNumRows(original.type);
        /** @type {number} */ var cols = gluShaderUtil.getDataTypeMatrixNumColumns(original.type);
        /** @type {es3fUniformApiTests.VarValue} */ var result = new es3fUniformApiTests.VarValue();
        result.type = es3fUniformApiTests.getDataTypeTransposedMatrix(original.type);

        for (var i = 0; i < rows; i++)
        for (var j = 0; j < cols; j++)
            result.val[i * cols + j] = original.val[j * rows + i];

        return result;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} value
     * @return {string}
     */
    es3fUniformApiTests.shaderVarValueStr = function(value) {
        /** @type {number} */ var numElems = gluShaderUtil.getDataTypeScalarSize(value.type);
        /** @type {string} */ var result = '';

        if (numElems > 1)
            result += gluShaderUtil.getDataTypeName(value.type) + '(';

        for (var i = 0; i < numElems; i++) {
            if (i > 0)
                result += ', ';

            if (gluShaderUtil.isDataTypeFloatOrVec(value.type) || gluShaderUtil.isDataTypeMatrix(value.type))
                result += value.val[i].toFixed(2);
            else if (gluShaderUtil.isDataTypeIntOrIVec((value.type)))
                result += value.val[i];
            else if (gluShaderUtil.isDataTypeUintOrUVec((value.type)))
                result += value.val[i] + 'u';
            else if (gluShaderUtil.isDataTypeBoolOrBVec((value.type)))
                result += value.val[i] ? 'true' : 'false';
            else if (gluShaderUtil.isDataTypeSampler((value.type)))
                result += es3fUniformApiTests.shaderVarValueStr(es3fUniformApiTests.getSamplerFillValue(value));
            else
                throw new Error('es3fUniformApiTests.shaderVarValueStr - invalid type');
        }

        if (numElems > 1)
            result += ')';

        return result;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} value
     * @return {string}
     */
    es3fUniformApiTests.apiVarValueStr = function(value) {
        /** @type {number} */ var numElems = gluShaderUtil.getDataTypeScalarSize(value.type);
        /** @type {string} */ var result = '';

        if (numElems > 1)
            result += '(';

        for (var i = 0; i < numElems; i++) {
            if (i > 0)
                result += ', ';

            if (gluShaderUtil.isDataTypeFloatOrVec(value.type) || gluShaderUtil.isDataTypeMatrix(value.type))
                result += value.val[i].toFixed(2);
            else if (gluShaderUtil.isDataTypeIntOrIVec(value.type) ||
            gluShaderUtil.isDataTypeUintOrUVec(value.type))
                result += value.val[i];
            else if (gluShaderUtil.isDataTypeBoolOrBVec(value.type))
                result += value.val[i] ? 'true' : 'false';
            else if (gluShaderUtil.isDataTypeSampler(value.type))
                result += value.val.samplerV.unit;
            else
                throw new Error('es3fUniformApiTests.apiVarValueStr - Invalid type');
        }

        if (numElems > 1)
            result += ')';

        return result;
    };

    // samplerUnit used if type is a sampler type. \note Samplers' unit numbers are not randomized.
    /**
     * @param {gluShaderUtil.DataType} type
     * @param {deRandom.Random} rnd
     * @param {number=} samplerUnit
     * @return {es3fUniformApiTests.VarValue}
     */
    es3fUniformApiTests.generateRandomVarValue = function(type, rnd, samplerUnit) {
        if (samplerUnit === undefined) samplerUnit = -1;
        /** @type {number} */ var numElems = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {es3fUniformApiTests.VarValue} */ var result = new es3fUniformApiTests.VarValue();
        result.type = type;

        assertMsgOptions(
            (samplerUnit >= 0) == (gluShaderUtil.isDataTypeSampler(type)),
            'es3fUniformApiTests.generateRandomVarValue - sampler units do not match type', false, true
        );

        if (gluShaderUtil.isDataTypeFloatOrVec(type) || gluShaderUtil.isDataTypeMatrix(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = rnd.getFloat(-10.0, 10.0);
        } else if (gluShaderUtil.isDataTypeIntOrIVec(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = rnd.getInt(-10, 10);
        } else if (gluShaderUtil.isDataTypeUintOrUVec(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = rnd.getInt(0, 10);
        } else if (gluShaderUtil.isDataTypeBoolOrBVec(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = rnd.getBool();
        } else if (gluShaderUtil.isDataTypeSampler(type)) {
            /** @type {gluShaderUtil.DataType} */ var texResultType = es3fUniformApiTests.getSamplerLookupReturnType(type);
            /** @type {gluShaderUtil.DataType} */ var texResultScalarType = gluShaderUtil.getDataTypeScalarTypeAsDataType(texResultType);
            /** @type {number} */ var texResultNumDims = gluShaderUtil.getDataTypeScalarSize(texResultType);

            result.val = new es3fUniformApiTests.SamplerV();
            result.val.samplerV.unit = samplerUnit;

            for (var i = 0; i < texResultNumDims; i++) {
                switch (texResultScalarType) {
                    case gluShaderUtil.DataType.FLOAT: result.val.samplerV.fillColor[i] = rnd.getFloat(0.0, 1.0); break;
                    case gluShaderUtil.DataType.INT: result.val.samplerV.fillColor[i] = rnd.getInt(-10, 10); break;
                    case gluShaderUtil.DataType.UINT: result.val.samplerV.fillColor[i] = rnd.getInt(0, 10); break;
                    default:
                        throw new Error('es3fUniformApiTests.generateRandomVarValue - Invalid scalar type');
                }
            }
        } else
            throw new Error('es3fUniformApiTests.generateRandomVarValue - Invalid type');

        return result;
    };

    /**
     * @param {gluShaderUtil.DataType} type
     * @return {es3fUniformApiTests.VarValue}
     */
    es3fUniformApiTests.generateZeroVarValue = function(type) {
        /** @type {number} */ var numElems = gluShaderUtil.getDataTypeScalarSize(type);
        /** @type {es3fUniformApiTests.VarValue} */ var result = new es3fUniformApiTests.VarValue();
        result.type = type;

        if (gluShaderUtil.isDataTypeFloatOrVec(type) || gluShaderUtil.isDataTypeMatrix(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = 0.0;
        } else if (gluShaderUtil.isDataTypeIntOrIVec(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = 0;
        } else if (gluShaderUtil.isDataTypeUintOrUVec(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = 0;
        } else if (gluShaderUtil.isDataTypeBoolOrBVec(type)) {
            for (var i = 0; i < numElems; i++)
                result.val[i] = false;
        } else if (gluShaderUtil.isDataTypeSampler(type)) {
            /** @type {gluShaderUtil.DataType} */ var texResultType = es3fUniformApiTests.getSamplerLookupReturnType(type);
            /** @type {gluShaderUtil.DataType} */ var texResultScalarType = gluShaderUtil.getDataTypeScalarTypeAsDataType(texResultType);
            /** @type {number} */ var texResultNumDims = gluShaderUtil.getDataTypeScalarSize(texResultType);

            result.val = new es3fUniformApiTests.SamplerV();
            result.val.samplerV.unit = 0;

            for (var i = 0; i < texResultNumDims; i++) {
                switch (texResultScalarType) {
                    case gluShaderUtil.DataType.FLOAT: result.val.samplerV.fillColor[i] = 0.12 * i; break;
                    case gluShaderUtil.DataType.INT: result.val.samplerV.fillColor[i] = -2 + i; break;
                    case gluShaderUtil.DataType.UINT: result.val.samplerV.fillColor[i] = 4 + i; break;
                    default:
                        throw new Error('es3fUniformApiTests.generateZeroVarValue - Invalid scalar type');
                }
            }
        } else
            throw new Error('es3fUniformApiTests.generateZeroVarValue - Invalid type');

        return result;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} a
     * @param {es3fUniformApiTests.VarValue} b
     * @return {boolean}
     */
    es3fUniformApiTests.apiVarValueEquals = function(a, b) {
        /** @type {number} */ var size = gluShaderUtil.getDataTypeScalarSize(a.type);
        /** @type {number} */ var floatThreshold = 0.05;

        assertMsgOptions(a.type == b.type, 'es3fUniformApiTests.apiVarValueEquals - types are different', false, true);

        if (gluShaderUtil.isDataTypeFloatOrVec(a.type) || gluShaderUtil.isDataTypeMatrix(a.type)) {
            for (var i = 0; i < size; i++)
                if (Math.abs(a.val[i] - b.val[i]) >= floatThreshold)
                    return false;
        } else if (gluShaderUtil.isDataTypeIntOrIVec(a.type)) {
            for (var i = 0; i < size; i++)
                if (a.val[i] != b.val[i])
                    return false;
        } else if (gluShaderUtil.isDataTypeUintOrUVec(a.type)) {
            for (var i = 0; i < size; i++)
                if (a.val[i] != b.val[i])
                    return false;
        } else if (gluShaderUtil.isDataTypeBoolOrBVec(a.type)) {
            for (var i = 0; i < size; i++)
                if (a.val[i] != b.val[i])
                    return false;
        } else if (gluShaderUtil.isDataTypeSampler(a.type)) {
            if (a.val.samplerV.unit != b.val.samplerV.unit)
                return false;
        } else
            throw new Error('es3fUniformApiTests.apiVarValueEquals - Invalid type');

        return true;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} boolValue
     * @param {gluShaderUtil.DataType} targetScalarType
     * @param {deRandom.Random} rnd
     * @return {es3fUniformApiTests.VarValue}
     */
    es3fUniformApiTests.getRandomBoolRepresentation = function(boolValue, targetScalarType, rnd) {
        assertMsgOptions(
            gluShaderUtil.isDataTypeBoolOrBVec(boolValue.type),
            'es3fUniformApiTests.getRandomBoolRepresentation - Data type not boolean or boolean vector',
            false,
            true
        );

        /** @type {number} */ var size = gluShaderUtil.getDataTypeScalarSize(boolValue.type);
        /** @type {gluShaderUtil.DataType} */ var targetType = size == 1 ? targetScalarType : gluShaderUtil.getDataTypeVector(targetScalarType, size);
        /** @type {es3fUniformApiTests.VarValue} */ var result = new es3fUniformApiTests.VarValue();
        result.type = targetType;

        switch (targetScalarType) {
            case gluShaderUtil.DataType.INT:
                for (var i = 0; i < size; i++) {
                    if (boolValue.val[i]) {
                        result.val[i] = rnd.getInt(-10, 10);
                        if (result.val[i] == 0)
                            result.val[i] = 1;
                    } else
                        result.val[i] = 0;
                }
                break;

            case gluShaderUtil.DataType.UINT:
                for (var i = 0; i < size; i++) {
                    if (boolValue.val[i])
                        result.val[i] = rnd.getInt(1, 10);
                    else
                        result.val[i] = 0;
                }
                break;

            case gluShaderUtil.DataType.FLOAT:
                for (var i = 0; i < size; i++) {
                    if (boolValue.val[i]) {
                        result.val[i] = rnd.getFloat(-10.0, 10.0);
                        if (result.val[i] == 0.0)
                            result.val[i] = 1.0;
                    } else
                        result.val[i] = 0;
                }
                break;

            default:
                throw new Error('es3fUniformApiTests.getRandomBoolRepresentation - Invalid type');
        }

        return result;
    };

    /**
     * @param {es3fUniformApiTests.CaseShaderType} type
     * @return {?string}
     */
    es3fUniformApiTests.getCaseShaderTypeName = function(type) {
        switch (type) {
            case es3fUniformApiTests.CaseShaderType.VERTEX: return 'vertex';
            case es3fUniformApiTests.CaseShaderType.FRAGMENT: return 'fragment';
            case es3fUniformApiTests.CaseShaderType.BOTH: return 'both';
            default:
                throw new Error('es3fUniformApiTests.getCaseShaderTypeName - Invalid shader type');
        }
    };

    /**
     * @param {number} seed
     * @return {number}
     */
    es3fUniformApiTests.randomCaseShaderType = function(seed) {
        return (new deRandom.Random(seed)).getInt(0, Object.keys(es3fUniformApiTests.CaseShaderType).length - 1);
    };

    //es3fUniformApiTests.UniformCase definitions

    /**
     * es3fUniformApiTests.Feature - Implemented as a function to create an object without unwanted properties.
     * @constructor
     */
    es3fUniformApiTests.Feature = function() {
        // ARRAYUSAGE_ONLY_MIDDLE_INDEX: only middle index of each array is used in shader. If not given, use all indices.
        this.ARRAYUSAGE_ONLY_MIDDLE_INDEX = false;

        // UNIFORMFUNC_VALUE: use pass-by-value versions of uniform assignment funcs, e.g. glUniform1f(), where possible. If not given, use pass-by-pointer versions.
        this.UNIFORMFUNC_VALUE = false;

        // MATRIXMODE_ROWMAJOR: pass matrices to GL in row major form. If not given, use column major.
        this.MATRIXMODE_ROWMAJOR = false;

        // ARRAYASSIGN: how basic-type arrays are assigned with glUniform*(). If none given, assign each element of an array separately.
        this.ARRAYASSIGN_FULL = false; //!< Assign all elements of an array with one glUniform*().
        this.ARRAYASSIGN_BLOCKS_OF_TWO = false; //!< Assign two elements per one glUniform*().

        // UNIFORMUSAGE_EVERY_OTHER: use about half of the uniforms. If not given, use all uniforms (except that some array indices may be omitted according to ARRAYUSAGE).
        this.UNIFORMUSAGE_EVERY_OTHER = false;

        // BOOLEANAPITYPE: type used to pass booleans to and from GL api. If none given, use float.
        this.BOOLEANAPITYPE_INT = false;
        this.BOOLEANAPITYPE_UINT = false;

        // UNIFORMVALUE_ZERO: use zero-valued uniforms. If not given, use random uniform values.
        this.UNIFORMVALUE_ZERO = false;

        // ARRAY_FIRST_ELEM_NAME_NO_INDEX: in certain API functions, when referring to the first element of an array, use just the array name without [0] at the end.
        this.ARRAY_FIRST_ELEM_NAME_NO_INDEX = false;
    };

    // A basic uniform is a uniform (possibly struct or array member) whose type is a basic type (e.g. float, ivec4, sampler2d).
    /**
     * @constructor
     * @param {string} name_
     * @param {gluShaderUtil.DataType} type_
     * @param {boolean} isUsedInShader_
     * @param {es3fUniformApiTests.VarValue} finalValue_
     * @param {string=} rootName_
     * @param {number=} elemNdx_
     * @param {number=} rootSize_
     */
    es3fUniformApiTests.BasicUniform = function(name_, type_, isUsedInShader_, finalValue_, rootName_, elemNdx_, rootSize_) {
        /** @type {string} */ this.name = name_;
        /** @type {gluShaderUtil.DataType} */ this.type = type_;
        /** @type {boolean} */ this.isUsedInShader = isUsedInShader_;
        /** @type {es3fUniformApiTests.VarValue} */ this.finalValue = finalValue_; //!< The value we ultimately want to set for this uniform.

        /** @type {string} */ this.rootName = rootName_ === undefined ? name_ : rootName_; //!< If this is a member of a basic-typed array, rootName is the name of that array with "[0]" appended. Otherwise it equals name.
        /** @type {number} */ this.elemNdx = elemNdx_ === undefined ? -1 : elemNdx_; //!< If this is a member of a basic-typed array, elemNdx is the index in that array. Otherwise -1.
        /** @type {number} */ this.rootSize = rootSize_ === undefined ? 1 : rootSize_; //!< If this is a member of a basic-typed array, rootSize is the size of that array. Otherwise 1.
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} vec
     * @param {string} name
     * @return {es3fUniformApiTests.BasicUniform}
     */
    es3fUniformApiTests.BasicUniform.findWithName = function(vec, name) {
        for (var i = 0; i < vec.length; i++) {
            if (vec[i].name == name)
                return vec[i];
        }
        return null;
    };

    // Reference values for info that is expected to be reported by glGetActiveUniform() or glGetActiveUniforms().
    /**
     * @constructor
     * @param {string} name_
     * @param {gluShaderUtil.DataType} type_
     * @param {boolean} used
     */
    es3fUniformApiTests.BasicUniformReportRef = function(name_, type_, used) {
        /** @type {string} */ this.name = name_;
        // \note minSize and maxSize are for arrays and can be distinct since implementations are allowed, but not required, to trim the inactive end indices of arrays.
        /** @type {number} */ this.minSize = 1;
        /** @type {number} */ this.maxSize = 1;
        /** @type {gluShaderUtil.DataType} */ this.type = type_;
        /** @type {boolean} */ this.isUsedInShader = used;
    };

    /**
     * To be used after constructor
     * @param {number} minS
     * @param {number} maxS
     * @return {es3fUniformApiTests.BasicUniformReportRef}
     */
    es3fUniformApiTests.BasicUniformReportRef.prototype.constructor_A = function(minS, maxS) {
        this.minSize = minS;
        this.maxSize = maxS;

        assertMsgOptions(
            this.minSize <= this.maxSize,
            'es3fUniformApiTests.BasicUniformReportRef.prototype.constructor_A - min size not smaller or equal than max size',
            false,
            true
        );

        return this;
    };

    // Info that is actually reported by glGetActiveUniform() or glGetActiveUniforms().
    /**
     * @constructor
     * @param {string} name_
     * @param {number} nameLength_
     * @param {number} size_
     * @param {gluShaderUtil.DataType} type_
     * @param {number} index_
     */
    es3fUniformApiTests.BasicUniformReportGL = function(name_, nameLength_, size_, type_, index_) {
        this.name = name_;
        this.nameLength = nameLength_;
        this.size = size_;
        this.type = type_;
        this.index = index_;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniformReportGL>} vec
     * @param {string} name
     * @return {es3fUniformApiTests.BasicUniformReportGL}
     */
    es3fUniformApiTests.BasicUniformReportGL.findWithName = function(vec, name) {
        for (var i = 0; i < vec.length; i++) {
            if (vec[i].name == name)
                return vec[i];
        }
        return null;
    };

    /**
     * es3fUniformApiTests.UniformCase class, inherits from TestCase class
     * @constructor
     * @param {string} name
     * @param {string} description
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fUniformApiTests.UniformCase = function(name, description) { // \note Randomizes caseType, uniformCollection and features.
        tcuTestCase.DeqpTest.call(this, name, description);

        /** @type {es3fUniformApiTests.Feature} */ this.m_features;
        /** @type {es3fUniformApiTests.UniformCollection} (SharedPtr) */ this.m_uniformCollection;

        /** @type {number} */ this.m_caseShaderType = 0;

        /** @type {Array<gluTexture.Texture2D>} */ this.m_textures2d = [];
        /** @type {Array<gluTexture.TextureCube>} */ this.m_texturesCube = [];
        /** @type {Array<number>} */ this.m_filledTextureUnits = [];
    };

    es3fUniformApiTests.UniformCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    /** es3fUniformApiTests.UniformCase prototype restore */
    es3fUniformApiTests.UniformCase.prototype.constructor = es3fUniformApiTests.UniformCase;

    /**
     * es3fUniformApiTests.UniformCase newC. Creates a es3fUniformApiTests.UniformCase. Use after constructor.
     * @param {number} seed
     * @return {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformCase.prototype.newC = function(seed) {
        this.m_features = this.randomFeatures(seed);
        this.m_uniformCollection = es3fUniformApiTests.UniformCollection.random(seed);
        this.m_caseShaderType = es3fUniformApiTests.randomCaseShaderType(seed);

        return this;
    };

    /**
     * es3fUniformApiTests.UniformCase new_B (static). Creates a es3fUniformApiTests.UniformCase
     * @param {string} name
     * @param {string} description
     * @param {number} seed
     * @return {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformCase.new_C = function(name, description, seed) {
        var uniformCase = new es3fUniformApiTests.UniformCase(name, description).newC(seed);

        return uniformCase;
    };

    /**
     * es3fUniformApiTests.UniformCase new_B. Creates a es3fUniformApiTests.UniformCase. Use after constructor.
     * @param {es3fUniformApiTests.CaseShaderType} caseShaderType
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection (SharedPtr)
     * @param {es3fUniformApiTests.Feature} features
     * @return {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformCase.prototype.newB = function(caseShaderType, uniformCollection, features) {
        this.m_caseShaderType = caseShaderType;
        this.m_uniformCollection = uniformCollection;
        this.m_features = features;

        return this;
    };

    /**
     * es3fUniformApiTests.UniformCase new_B (static). Creates a es3fUniformApiTests.UniformCase
     * @param {string} name
     * @param {string} description
     * @param {es3fUniformApiTests.CaseShaderType} caseShaderType
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection (SharedPtr)
     * @param {es3fUniformApiTests.Feature} features
     * @return {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformCase.new_B = function(name, description, caseShaderType, uniformCollection, features) {
        var uniformCase = new es3fUniformApiTests.UniformCase(name, description).newB(caseShaderType, uniformCollection, features);

        return uniformCase;
    };

    /**
     * es3fUniformApiTests.UniformCase new_A. Creates a es3fUniformApiTests.UniformCase. Use after constructor.
     * @param {es3fUniformApiTests.CaseShaderType} caseShaderType
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection (SharedPtr)
     * @return {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformCase.prototype.newA = function(caseShaderType, uniformCollection) {
       this.m_caseShaderType = caseShaderType;
       this.m_uniformCollection = uniformCollection;
       this.m_features = null;

       return this;
    };

    /**
     * es3fUniformApiTests.UniformCase new_A (static). Creates a es3fUniformApiTests.UniformCase
     * @param {string} name
     * @param {string} description
     * @param {es3fUniformApiTests.CaseShaderType} caseShaderType
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection (SharedPtr)
     * @return {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformCase.new_A = function(name, description, caseShaderType, uniformCollection) {
        var uniformCase = new es3fUniformApiTests.UniformCase(name, description).newA(caseShaderType, uniformCollection);

        return uniformCase;
    };

    /**
     * @param {number} seed
     * @return {es3fUniformApiTests.Feature}
     */
    es3fUniformApiTests.UniformCase.prototype.randomFeatures = function(seed) {
        /** @type {es3fUniformApiTests.Feature} */ var result = new es3fUniformApiTests.Feature();

        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(seed);

        result.ARRAYUSAGE_ONLY_MIDDLE_INDEX = rnd.getBool();
        result.UNIFORMFUNC_VALUE = rnd.getBool();
        result.MATRIXMODE_ROWMAJOR = rnd.getBool();
        result.ARRAYASSIGN_FULL = rnd.getBool();
        result.ARRAYASSIGN_BLOCKS_OF_TWO = !result.ARRAYASSIGN_FULL;
        result.UNIFORMUSAGE_EVERY_OTHER = rnd.getBool();
        result.BOOLEANAPITYPE_INT = rnd.getBool();
        result.BOOLEANAPITYPE_UINT = !result.BOOLEANAPITYPE_INT;
        result.UNIFORMVALUE_ZERO = rnd.getBool();

        return result;
    };

    /**
     * Initialize the es3fUniformApiTests.UniformCase
     */
    es3fUniformApiTests.UniformCase.prototype.init = function() {
        /** @type {number} */ var numSamplerUniforms = this.m_uniformCollection.getNumSamplers();
        /** @type {number} */ var vertexTexUnitsRequired = this.m_caseShaderType != es3fUniformApiTests.CaseShaderType.FRAGMENT ? numSamplerUniforms : 0;
        /** @type {number} */ var fragmentTexUnitsRequired = this.m_caseShaderType != es3fUniformApiTests.CaseShaderType.VERTEX ? numSamplerUniforms : 0;
        /** @type {number} */ var combinedTexUnitsRequired = vertexTexUnitsRequired + fragmentTexUnitsRequired;
        var vertexTexUnitsSupported = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS));
        var fragmentTexUnitsSupported = /** @type {number} */ (gl.getParameter(gl.MAX_TEXTURE_IMAGE_UNITS));
        var combinedTexUnitsSupported = /** @type {number} */ (gl.getParameter(gl.MAX_COMBINED_TEXTURE_IMAGE_UNITS));

        assertMsgOptions(
            numSamplerUniforms <= es3fUniformApiTests.MAX_NUM_SAMPLER_UNIFORMS,
            'es3fUniformApiTests.UniformCase.prototype.init - sampler uniforms exceed MAX_NUM_SAMPLER_UNIFORMS',
            false,
            true
        );

        if (vertexTexUnitsRequired > vertexTexUnitsSupported)
            testFailedOptions('' + vertexTexUnitsRequired + ' vertex texture units required, ' + vertexTexUnitsSupported + ' supported', true);
        if (fragmentTexUnitsRequired > fragmentTexUnitsSupported)
            testFailedOptions('' + fragmentTexUnitsRequired + ' fragment texture units required, ' + fragmentTexUnitsSupported + ' supported', true);
        if (combinedTexUnitsRequired > combinedTexUnitsSupported)
            testFailedOptions('' + combinedTexUnitsRequired + ' combined texture units required, ' + combinedTexUnitsSupported + ' supported', true);
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniformsDst
     * @param {Array<es3fUniformApiTests.BasicUniformReportRef>} basicUniformReportsDst
     * @param {gluVarType.VarType} varType
     * @param {string} varName
     * @param {boolean} isParentActive
     * @param {number} samplerUnitCounter
     * @param {deRandom.Random} rnd
     * @return {number} Used to be output parameter. Sampler unit count
     */
    es3fUniformApiTests.UniformCase.prototype.generateBasicUniforms = function(basicUniformsDst, basicUniformReportsDst, varType, varName, isParentActive, samplerUnitCounter, rnd) {
        /** @type {es3fUniformApiTests.VarValue} */ var value;

        if (varType.isBasicType()) {
            /** @type {boolean} */ var isActive = isParentActive && (this.m_features.UNIFORMUSAGE_EVERY_OTHER ? basicUniformsDst.length % 2 == 0 : true);
            /** @type {gluShaderUtil.DataType} */ var type = varType.getBasicType();
            value = this.m_features.UNIFORMVALUE_ZERO ? es3fUniformApiTests.generateZeroVarValue(type) :
            gluShaderUtil.isDataTypeSampler(type) ? es3fUniformApiTests.generateRandomVarValue(type, rnd, samplerUnitCounter++) :
                                                es3fUniformApiTests.generateRandomVarValue(varType.getBasicType(), rnd);

            basicUniformsDst.push(new es3fUniformApiTests.BasicUniform(varName, varType.getBasicType(), isActive, value));
            basicUniformReportsDst.push(new es3fUniformApiTests.BasicUniformReportRef(varName, varType.getBasicType(), isActive));
        } else if (varType.isArrayType()) {
            /** @type {number} */ var size = varType.getArraySize();
            /** @type {string} */ var arrayRootName = '' + varName + '[0]';
            /** @type {Array<boolean>} */ var isElemActive = [];

            for (var elemNdx = 0; elemNdx < varType.getArraySize(); elemNdx++) {
                /** @type {string} */ var indexedName = '' + varName + '[' + elemNdx + ']';
                /** @type {boolean} */ var isCurElemActive = isParentActive &&
                                                  (this.m_features.UNIFORMUSAGE_EVERY_OTHER ? basicUniformsDst.length % 2 == 0 : true) &&
                                                  (this.m_features.ARRAYUSAGE_ONLY_MIDDLE_INDEX ? elemNdx == Math.floor(size / 2) : true);

                isElemActive.push(isCurElemActive);

                if (varType.getElementType().isBasicType()) {
                    // \note We don't want separate entries in basicUniformReportsDst for elements of basic-type arrays.
                    /** @type {gluShaderUtil.DataType} */ var elemBasicType = varType.getElementType().getBasicType();
                    value = this.m_features.UNIFORMVALUE_ZERO ? es3fUniformApiTests.generateZeroVarValue(elemBasicType) :
                    gluShaderUtil.isDataTypeSampler(elemBasicType) ? es3fUniformApiTests.generateRandomVarValue(elemBasicType, rnd, samplerUnitCounter++) :
                                                        es3fUniformApiTests.generateRandomVarValue(elemBasicType, rnd);

                    basicUniformsDst.push(new es3fUniformApiTests.BasicUniform(indexedName, elemBasicType, isCurElemActive, value, arrayRootName, elemNdx, size));
                } else
                    samplerUnitCounter = this.generateBasicUniforms(basicUniformsDst, basicUniformReportsDst, varType.getElementType(), indexedName, isCurElemActive, samplerUnitCounter, rnd);
            }

            if (varType.getElementType().isBasicType()) {
                /** @type {number} */ var minSize;
                for (minSize = varType.getArraySize(); minSize > 0 && !isElemActive[minSize - 1]; minSize--) {}

                basicUniformReportsDst.push(new es3fUniformApiTests.BasicUniformReportRef(arrayRootName, varType.getElementType().getBasicType(), isParentActive && minSize > 0).constructor_A(minSize, size));
            }
        } else {
            assertMsgOptions(
                varType.isStructType(),
                'es3fUniformApiTests.UniformCase.prototype.generateBasicUniforms - not a struct type',
                false,
                true
            );

            /** @type {gluVarType.StructType} */ var structType = varType.getStruct();

            for (var i = 0; i < structType.getSize(); i++) {
                /** @type {gluVarType.StructMember} */ var member = structType.getMember(i);
                /** @type {string} */ var memberFullName = '' + varName + '.' + member.getName();

                samplerUnitCounter = this.generateBasicUniforms(basicUniformsDst, basicUniformReportsDst, member.getType(), memberFullName, isParentActive, samplerUnitCounter, rnd);
            }
        }

        return samplerUnitCounter;
    };

    /**
     * @param {string} dst
     * @return {string}
     */
    es3fUniformApiTests.UniformCase.prototype.writeUniformDefinitions = function(dst) {
        for (var i = 0; i < this.m_uniformCollection.getNumStructTypes(); i++)
            dst += gluVarType.declareStructType(this.m_uniformCollection.getStructType(i), 0) + ';\n';

        for (var i = 0; i < this.m_uniformCollection.getNumUniforms(); i++)
            dst += 'uniform ' + gluVarType.declareVariable(this.m_uniformCollection.getUniform(i).type, this.m_uniformCollection.getUniform(i).name, 0) + ';\n';

        dst += '\n';

        var compareFuncs = [{
                requiringTypes: [gluShaderUtil.isDataTypeFloatOrVec, gluShaderUtil.isDataTypeMatrix],
                definition: 'mediump float compare_float (mediump float a, mediump float b) { return abs(a - b) < 0.05 ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_VEC2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeIsMatrixWithNRows(2, t);}
                ],
                definition: 'mediump float compare_vec2 (mediump vec2 a, mediump vec2 b) { return compare_float(a.x, b.x)*compare_float(a.y, b.y); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_VEC3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeIsMatrixWithNRows(3, t);}
                ],
                definition: 'mediump float compare_vec3 (mediump vec3 a, mediump vec3 b) { return compare_float(a.x, b.x)*compare_float(a.y, b.y)*compare_float(a.z, b.z); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_VEC4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeIsMatrixWithNRows(4, t);}],
                definition: 'mediump float compare_vec4 (mediump vec4 a, mediump vec4 b) { return compare_float(a.x, b.x)*compare_float(a.y, b.y)*compare_float(a.z, b.z)*compare_float(a.w, b.w); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat2 (mediump mat2 a, mediump mat2 b) { return compare_vec2(a[0], b[0])*compare_vec2(a[1], b[1]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT2X3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat2x3 (mediump mat2x3 a, mediump mat2x3 b) { return compare_vec3(a[0], b[0])*compare_vec3(a[1], b[1]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT2X4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat2x4 (mediump mat2x4 a, mediump mat2x4 b) { return compare_vec4(a[0], b[0])*compare_vec4(a[1], b[1]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT3X2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat3x2 (mediump mat3x2 a, mediump mat3x2 b) { return compare_vec2(a[0], b[0])*compare_vec2(a[1], b[1])*compare_vec2(a[2], b[2]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat3 (mediump mat3 a, mediump mat3 b) { return compare_vec3(a[0], b[0])*compare_vec3(a[1], b[1])*compare_vec3(a[2], b[2]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT3X4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat3x4 (mediump mat3x4 a, mediump mat3x4 b) { return compare_vec4(a[0], b[0])*compare_vec4(a[1], b[1])*compare_vec4(a[2], b[2]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT4X2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat4x2 (mediump mat4x2 a, mediump mat4x2 b) { return compare_vec2(a[0], b[0])*compare_vec2(a[1], b[1])*compare_vec2(a[2], b[2])*compare_vec2(a[3], b[3]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT4X3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat4x3 (mediump mat4x3 a, mediump mat4x3 b) { return compare_vec3(a[0], b[0])*compare_vec3(a[1], b[1])*compare_vec3(a[2], b[2])*compare_vec3(a[3], b[3]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.FLOAT_MAT4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_mat4 (mediump mat4 a, mediump mat4 b) { return compare_vec4(a[0], b[0])*compare_vec4(a[1], b[1])*compare_vec4(a[2], b[2])*compare_vec4(a[3], b[3]); }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INT, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_int (mediump int a, mediump int b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INT_VEC2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_ivec2 (mediump ivec2 a, mediump ivec2 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INT_VEC3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_ivec3 (mediump ivec3 a, mediump ivec3 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INT_VEC4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_ivec4 (mediump ivec4 a, mediump ivec4 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.UINT, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_uint (mediump uint a, mediump uint b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.UINT_VEC2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_uvec2 (mediump uvec2 a, mediump uvec2 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.UINT_VEC3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_uvec3 (mediump uvec3 a, mediump uvec3 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.UINT_VEC4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_uvec4 (mediump uvec4 a, mediump uvec4 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.BOOL, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_bool (bool a, bool b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.BOOL_VEC2, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_bvec2 (bvec2 a, bvec2 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.BOOL_VEC3, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_bvec3 (bvec3 a, bvec3 b) { return a == b ? 1.0 : 0.0; }'
            },{
                requiringTypes: [
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.BOOL_VEC4, t);},
                    function(t) {return es3fUniformApiTests.dataTypeEquals(gluShaderUtil.DataType.INVALID, t);}
                ],
                definition: 'mediump float compare_bvec4 (bvec4 a, bvec4 b) { return a == b ? 1.0 : 0.0; }'
            }
        ];

        /** @type {Array<gluShaderUtil.DataType>} */ var samplerTypes = this.m_uniformCollection.getSamplerTypes();

        for (var compFuncNdx = 0; compFuncNdx < compareFuncs.length; compFuncNdx++) {
            /** @type {Array<es3fUniformApiTests.dataTypePredicate>} */ var typeReq = compareFuncs[compFuncNdx].requiringTypes;
            /** @type {boolean} */ var containsTypeSampler = false;

            for (var i = 0; i < samplerTypes.length; i++) {
                if (gluShaderUtil.isDataTypeSampler(samplerTypes[i])) {
                    /** @type {gluShaderUtil.DataType} */ var retType = es3fUniformApiTests.getSamplerLookupReturnType(samplerTypes[i]);
                    if (typeReq[0](retType) || typeReq[1](retType)) {
                        containsTypeSampler = true;
                        break;
                    }
                }
            }

            if (containsTypeSampler || this.m_uniformCollection.containsMatchingBasicType(typeReq[0]) || this.m_uniformCollection.containsMatchingBasicType(typeReq[1]))
                dst += compareFuncs[compFuncNdx].definition + '\n';
        }

        return dst;
    };

    /**
     * @param {string} dst
     * @param {es3fUniformApiTests.BasicUniform} uniform
     * @return {string} Used to write the string in the output parameter
     */
    es3fUniformApiTests.UniformCase.prototype.writeUniformCompareExpr = function(dst, uniform) {
        if (gluShaderUtil.isDataTypeSampler(uniform.type))
            dst += 'compare_' + gluShaderUtil.getDataTypeName(es3fUniformApiTests.getSamplerLookupReturnType(uniform.type)) + '(texture(' + uniform.name + ', vec' + es3fUniformApiTests.getSamplerNumLookupDimensions(uniform.type) + '(0.0))'; //WebGL2.0
        else
            dst += 'compare_' + gluShaderUtil.getDataTypeName(uniform.type) + '(' + uniform.name;

        dst += ', ' + es3fUniformApiTests.shaderVarValueStr(uniform.finalValue) + ')';

        return dst;
    };

    /**
     * @param {string} dst
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {string} variableName
     * @return {string} Used to write the string in the output parameter
     */
    es3fUniformApiTests.UniformCase.prototype.writeUniformComparisons = function(dst, basicUniforms, variableName) {
        for (var i = 0; i < basicUniforms.length; i++) {
            /** @type {es3fUniformApiTests.BasicUniform} */ var unif = basicUniforms[i];

            if (unif.isUsedInShader) {
                dst += '\t' + variableName + ' *= ';
                dst = this.writeUniformCompareExpr(dst, basicUniforms[i]);
                dst += ';\n';
            } else
                dst += '\t// UNUSED: ' + basicUniforms[i].name + '\n';
        }

        return dst;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @return {string}
     */
    es3fUniformApiTests.UniformCase.prototype.generateVertexSource = function(basicUniforms) {
        /** @type {boolean} */ var isVertexCase = this.m_caseShaderType == es3fUniformApiTests.CaseShaderType.VERTEX || this.m_caseShaderType == es3fUniformApiTests.CaseShaderType.BOTH;
        /** @type {string} */ var result = '';

        result += '#version 300 es\n' +
                  'in highp vec4 a_position;\n' +
                  'out mediump float v_vtxOut;\n' +
                  '\n';

        if (isVertexCase)
            result = this.writeUniformDefinitions(result);

        result += '\n' +
                  'void main (void)\n' +
                  ' {\n' +
                  ' gl_Position = a_position;\n' +
                  ' v_vtxOut = 1.0;\n';

        if (isVertexCase)
            result = this.writeUniformComparisons(result, basicUniforms, 'v_vtxOut');

        result += '}\n';

        return result;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @return {string}
     */
    es3fUniformApiTests.UniformCase.prototype.generateFragmentSource = function(basicUniforms) {
        /**@type {boolean} */ var isFragmentCase = this.m_caseShaderType == es3fUniformApiTests.CaseShaderType.FRAGMENT || this.m_caseShaderType == es3fUniformApiTests.CaseShaderType.BOTH;
        /**@type {string} */ var result = '';

        result += '#version 300 es\n' +
                  'in mediump float v_vtxOut;\n' +
                  '\n';

        if (isFragmentCase)
            result = this.writeUniformDefinitions(result);

        result += '\n' +
                  'layout(location = 0) out mediump vec4 dEQP_FragColor;\n' +
                  '\n' +
                  'void main (void)\n' +
                  ' {\n' +
                  ' mediump float result = v_vtxOut;\n';

        if (isFragmentCase)
            result = this.writeUniformComparisons(result, basicUniforms, 'result');

        result += ' dEQP_FragColor = vec4(result, result, result, 1.0);\n' +
                  '}\n';

        return result;
    };

    /**
     * @param {es3fUniformApiTests.VarValue} value
     */
    es3fUniformApiTests.UniformCase.prototype.setupTexture = function(value) {
        // \note No handling for samplers other than 2D or cube.

        assertMsgOptions(
            es3fUniformApiTests.getSamplerLookupReturnType(value.type) == gluShaderUtil.DataType.FLOAT_VEC4,
            'es3fUniformApiTests.UniformCase.prototype.setupTexture - sampler return type should be vec4f', false, true
        );

        /** @type {number} */ var width = 32;
        /** @type {number} */ var height = 32;
        /** @type {Array<number>} */ var color = value.val.samplerV.fillColor;
        /** @type {tcuTexture.TextureCube} */ var refTexture;
        /** @type {gluTexture.TextureCube} */ var texture;

        if (value.type == gluShaderUtil.DataType.SAMPLER_2D) {
            texture = gluTexture.texture2DFromFormat(gl, gl.RGBA, gl.UNSIGNED_BYTE, width, height);
            refTexture = texture.getRefTexture();
            this.m_textures2d.push(texture);

            refTexture.allocLevel(0);
            es3fUniformApiTests.fillWithColor(refTexture.getLevel(0), color);

           gl.activeTexture(gl.TEXTURE0 + value.val.samplerV.unit);
            this.m_filledTextureUnits.push(value.val.samplerV.unit);
            texture.upload();
           gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
           gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
           gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
           gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        } else if (value.type == gluShaderUtil.DataType.SAMPLER_CUBE) {
            assertMsgOptions(width == height, 'es3fUniformApiTests.UniformCase.prototype.setupTexture - non square texture', false, true);

            texture = gluTexture.cubeFromFormat(gl, gl.RGBA, gl.UNSIGNED_BYTE, width);
            refTexture = texture.getRefTexture();
            this.m_texturesCube.push(texture);

            for (var face in tcuTexture.CubeFace) {
                refTexture.allocLevel(tcuTexture.CubeFace[face], 0);
                es3fUniformApiTests.fillWithColor(refTexture.getLevelFace(0, tcuTexture.CubeFace[face]), color);
            }

           gl.activeTexture(gl.TEXTURE0 + value.val.samplerV.unit);
            this.m_filledTextureUnits.push(value.val.samplerV.unit);
            texture.upload();
           gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
           gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
           gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
           gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
        } else
            throw new Error('es3fUniformApiTests.UniformCase.prototype.setupTexture - Invalid sampler type');
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniformReportGL>} basicUniformReportsDst
     * @param {Array<es3fUniformApiTests.BasicUniformReportRef>} basicUniformReportsRef
     * @param {WebGLProgram} programGL
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.getActiveUniformsOneByOne = function(basicUniformReportsDst, basicUniformReportsRef, programGL) {
        /** @type {WebGLProgram} */ var numActiveUniforms;
        /** @type {boolean} */ var success = true;

       numActiveUniforms = /** @type {WebGLProgram} */ (gl.getProgramParameter(programGL, gl.ACTIVE_UNIFORMS));
        bufferedLogToConsole('// Number of active uniforms reported: ' + numActiveUniforms);

        for (var unifNdx = 0; unifNdx < numActiveUniforms; unifNdx++) {
            /** @type {number} (GLint)*/ var reportedSize = -1;
            /** @type {number} (GLenum)*/ var reportedTypeGL = gl.NONE;
            /** @type {gluShaderUtil.DataType} */ var reportedType;
            /** @type {string} */ var reportedNameStr;
            /** @type {WebGLActiveInfo} */ var activeInfo;

           activeInfo = gl.getActiveUniform(programGL, unifNdx);

            reportedNameStr = activeInfo.name;
            reportedTypeGL = activeInfo.type;
            reportedSize = activeInfo.size;

            reportedType = gluShaderUtil.getDataTypeFromGLType(reportedTypeGL);

            checkMessage(reportedType !== undefined, 'Invalid uniform type');

            bufferedLogToConsole('// Got name = ' + reportedNameStr + ', size = ' + reportedSize + ', type = ' + gluShaderUtil.getDataTypeName(reportedType));

            // Ignore built-in uniforms.
            if (reportedNameStr.indexOf('gl_') == -1) {
                /** @type {number} */ var referenceNdx;
                for (referenceNdx = 0; referenceNdx < basicUniformReportsRef.length; referenceNdx++) {
                    if (basicUniformReportsRef[referenceNdx].name == reportedNameStr)
                        break;
                }

                if (referenceNdx >= basicUniformReportsRef.length) {
                    bufferedLogToConsole('// FAILURE: invalid non-built-in uniform name reported');
                    success = false;
                } else {
                    /** @type {es3fUniformApiTests.BasicUniformReportRef} */ var reference = basicUniformReportsRef[referenceNdx];

                    assertMsgOptions(
                        reference.type !== undefined,
                        'es3fUniformApiTests.UniformCase.prototype.getActiveUniformsOneByOne - type is undefined',
                        false,
                        true
                    );
                    assertMsgOptions(
                        reference.minSize >= 1 || (reference.minSize == 0 && !reference.isUsedInShader),
                        'es3fUniformApiTests.UniformCase.prototype.getActiveUniformsOneByOne - uniform min size does not match usage in shader',
                        false,
                        true
                    );
                    assertMsgOptions(
                        reference.minSize <= reference.maxSize,
                        'es3fUniformApiTests.UniformCase.prototype.getActiveUniformsOneByOne - uniform min size bigger than max size',
                        false,
                        true
                    );

                    if (es3fUniformApiTests.BasicUniformReportGL.findWithName(basicUniformReportsDst, reportedNameStr) !== null) {
                        bufferedLogToConsole('// FAILURE: same uniform name reported twice');
                        success = false;
                    }

                    basicUniformReportsDst.push(new es3fUniformApiTests.BasicUniformReportGL(reportedNameStr, reportedNameStr.length, reportedSize, reportedType, unifNdx));

                    if (reportedType != reference.type) {
                        bufferedLogToConsole('// FAILURE: wrong type reported, should be ' + gluShaderUtil.getDataTypeName(reference.type));
                        success = false;
                    }
                    if (reportedSize < reference.minSize || reportedSize > reference.maxSize) {
                        bufferedLogToConsole('// FAILURE: wrong size reported, should be ' +
                            (reference.minSize == reference.maxSize ? reference.minSize : 'in the range [' + reference.minSize + ', ' + reference.maxSize + ']'));

                        success = false;
                    }
                }
            }
        }

        for (var i = 0; i < basicUniformReportsRef.length; i++) {
            /** @type {es3fUniformApiTests.BasicUniformReportRef} */ var expected = basicUniformReportsRef[i];
            if (expected.isUsedInShader && es3fUniformApiTests.BasicUniformReportGL.findWithName(basicUniformReportsDst, expected.name) === null) {
                bufferedLogToConsole('// FAILURE: uniform with name ' + expected.name + ' was not reported by GL');
                success = false;
            }
        }

        return success;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniformReportGL>} basicUniformReportsDst
     * @param {Array<es3fUniformApiTests.BasicUniformReportRef>} basicUniformReportsRef
     * @param {WebGLProgram} programGL
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.getActiveUniforms = function(basicUniformReportsDst, basicUniformReportsRef, programGL) {
        /** @type {Array<string>} */ var queryNames = new Array(basicUniformReportsRef.length);
        /** @type {Array<string>} */ var queryNamesC = new Array(basicUniformReportsRef.length);
        /** @type {Array<number>} (GLuint) */ var uniformIndices = new Array(basicUniformReportsRef.length);
        /** @type {Array<number>} */ var validUniformIndices = []; // This shall have the same contents, and in same order, as uniformIndices, but with gl.INVALID_INDEX entries removed.
        /** @type {boolean} */ var success = true;

        for (var i = 0; i < basicUniformReportsRef.length; i++) {
            /** @type {string} */ var name = basicUniformReportsRef[i].name;
            queryNames[i] = this.m_features.ARRAY_FIRST_ELEM_NAME_NO_INDEX && name[name.length - 1] == ']' ? es3fUniformApiTests.beforeLast(name, '[') : name;
            queryNamesC[i] = queryNames[i];
        }

       uniformIndices = gl.getUniformIndices(programGL, queryNamesC);

        for (var i = 0; i < uniformIndices.length; i++) {
            if (uniformIndices[i] != gl.INVALID_INDEX)
                validUniformIndices.push(uniformIndices[i]);
            else {
                if (basicUniformReportsRef[i].isUsedInShader) {
                    bufferedLogToConsole('// FAILURE: uniform with name ' + basicUniformReportsRef[i].name + ' received gl.INVALID_INDEX');
                    success = false;
                }
            }
        }

        if (validUniformIndices.length > 0) {
            /** @type {Array<string>} */ var uniformNameBuf = new Array(validUniformIndices.length);
            /** @type {Array<number>} (GLint) */ var uniformSizeBuf = new Array(validUniformIndices.length);
            /** @type {Array<number>} (GLint) */ var uniformTypeBuf = new Array(validUniformIndices.length);

           uniformSizeBuf = gl.getActiveUniforms(programGL, validUniformIndices, gl.UNIFORM_SIZE);
           uniformTypeBuf = gl.getActiveUniforms(programGL, validUniformIndices, gl.UNIFORM_TYPE);

            /** @type {number} */ var validNdx = -1; // Keeps the corresponding index to validUniformIndices while unifNdx is the index to uniformIndices.
            for (var unifNdx = 0; unifNdx < uniformIndices.length; unifNdx++) {
                if (uniformIndices[unifNdx] == gl.INVALID_INDEX)
                    continue;

                validNdx++;

                /** @type {es3fUniformApiTests.BasicUniformReportRef} */ var reference = basicUniformReportsRef[unifNdx];
                /** @type {number} */ var reportedIndex = validUniformIndices[validNdx];
                /** @type {number} */ var reportedNameLength = reference.name.length;
                /** @type {number} */ var reportedSize = uniformSizeBuf[validNdx];
                /** @type {gluShaderUtil.DataType} */ var reportedType = gluShaderUtil.getDataTypeFromGLType(uniformTypeBuf[validNdx]);
                /** @type {string} */ var reportedNameStr = reference.name;

                bufferedLogToConsole('// Got name size = ' + reportedSize +
                    ', type = ' + gluShaderUtil.getDataTypeName(reportedType) +
                    ' for the uniform at index ' + reportedIndex + ' (' + reference.name + ')');

                assertMsgOptions(
                    reference.type !== undefined,
                    'es3fUniformApiTests.UniformCase.prototype.getActiveUniforms - type is undefined',
                    false,
                    true
                );
                assertMsgOptions(
                    reference.minSize >= 1 || (reference.minSize == 0 && !reference.isUsedInShader),
                    'es3fUniformApiTests.UniformCase.prototype.getActiveUniforms - uniform min size does not match usage in shader',
                    false,
                    true
                );
                assertMsgOptions(
                    reference.minSize <= reference.maxSize,
                    'es3fUniformApiTests.UniformCase.prototype.getActiveUniforms - uniform min size bigger than max size',
                    false,
                    true
                );

                if (es3fUniformApiTests.BasicUniformReportGL.findWithName(basicUniformReportsDst, reportedNameStr) !== null) {
                    bufferedLogToConsole('// FAILURE: same uniform name reported twice');
                    success = false;
                }
                basicUniformReportsDst.push(new es3fUniformApiTests.BasicUniformReportGL(reference.name, reportedNameLength, reportedSize, reportedType, reportedIndex));

                if (reportedType != reference.type) {
                    bufferedLogToConsole('// FAILURE: wrong type reported, should be ' + gluShaderUtil.getDataTypeName(reference.type));
                    success = false;
                }

                if (reportedSize < reference.minSize || reportedSize > reference.maxSize) {
                    bufferedLogToConsole('// FAILURE: wrong size reported, should be ' +
                        (reference.minSize == reference.maxSize ? reference.minSize : 'in the range [' + reference.minSize + ', ' + reference.maxSize + ']'));

                    success = false;
                }
            }
        }

        return success;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniformReportGL>} uniformResults
     * @param {Array<es3fUniformApiTests.BasicUniformReportGL>} uniformsResults
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.uniformVsUniformsComparison = function(uniformResults, uniformsResults) {
        /** @type {boolean} */ var success = true;
        /** @type {es3fUniformApiTests.BasicUniformReportGL} */ var uniformsResult;

        for (var uniformResultNdx = 0; uniformResultNdx < uniformResults.length; uniformResultNdx++) {
            /** @type {es3fUniformApiTests.BasicUniformReportGL} */ var uniformResult = uniformResults[uniformResultNdx];
            /** @type {string} */ var uniformName = uniformResult.name;
            uniformsResult = es3fUniformApiTests.BasicUniformReportGL.findWithName(uniformsResults, uniformName);

            if (uniformsResult !== null) {
                bufferedLogToConsole('// Checking uniform ' + uniformName);

                if (uniformResult.index != uniformsResult.index) {
                    bufferedLogToConsole('// FAILURE: glGetActiveUniform() and glGetUniformIndices() gave different indices for uniform ' + uniformName);
                    success = false;
                }
                if (uniformResult.nameLength != uniformsResult.nameLength) {
                    bufferedLogToConsole('// FAILURE: glGetActiveUniform() and glGetActiveUniforms() gave incompatible name lengths for uniform ' + uniformName);
                    success = false;
                }
                if (uniformResult.size != uniformsResult.size) {
                    bufferedLogToConsole('// FAILURE: glGetActiveUniform() and glGetActiveUniforms() gave different sizes for uniform ' + uniformName);
                    success = false;
                }
                if (uniformResult.type != uniformsResult.type) {
                    bufferedLogToConsole('// FAILURE: glGetActiveUniform() and glGetActiveUniforms() gave different types for uniform ' + uniformName);
                    success = false;
                }
            } else {
                bufferedLogToConsole('// FAILURE: uniform ' + uniformName + ' was reported active by glGetActiveUniform() but not by glGetUniformIndices()');
                success = false;
            }
        }

        for (var uniformsResultNdx = 0; uniformsResultNdx < uniformsResults.length; uniformsResultNdx++) {
            uniformsResult = uniformsResults[uniformsResultNdx];
            /** @type {string} */ var uniformsName = uniformsResult.name;
            /** @type {es3fUniformApiTests.BasicUniformReportGL} */ var uniformsResultIt = es3fUniformApiTests.BasicUniformReportGL.findWithName(uniformsResults, uniformsName);

            if (uniformsResultIt === null) {
                bufferedLogToConsole('// FAILURE: uniform ' + uniformsName + ' was reported active by glGetUniformIndices() but not by glGetActiveUniform()');
                success = false;
            }
        }

        return success;
    };

    /**
     * @param {Array<es3fUniformApiTests.VarValue>} valuesDst
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {WebGLProgram} programGL
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.getUniforms = function(valuesDst, basicUniforms, programGL) {
        /** @type {boolean} */ var success = true;

        for (var unifNdx = 0; unifNdx < basicUniforms.length; unifNdx++) {
            /** @type {es3fUniformApiTests.BasicUniform} */ var uniform = basicUniforms[unifNdx];
            /** @type {string} */ var queryName = this.m_features.ARRAY_FIRST_ELEM_NAME_NO_INDEX && uniform.elemNdx == 0 ? es3fUniformApiTests.beforeLast(uniform.name, '[') : uniform.name;
            /** @type {WebGLUniformLocation} */ var location = gl.getUniformLocation(programGL, queryName);
            /** @type {number} */ var size = gluShaderUtil.getDataTypeScalarSize(uniform.type);
            /** @type {es3fUniformApiTests.VarValue} */ var value = new es3fUniformApiTests.VarValue();

            if (!location) {
                value.type = gluShaderUtil.DataType.INVALID;
                valuesDst.push(value);
                if (uniform.isUsedInShader) {
                    bufferedLogToConsole('// FAILURE: ' + uniform.name + ' was used in shader, but has location -1');
                    success = false;
                }
                continue;
            }

            value.type = uniform.type;

            var result = /** @type {number} */ (gl.getUniform(programGL, location));

            if (gluShaderUtil.isDataTypeSampler(uniform.type)) {
                value.val = new es3fUniformApiTests.SamplerV();
                value.val.samplerV.unit = result;
            } else
                value.val = /** @type {Array<number>} */ (result.length === undefined ? [result] : result);

            valuesDst.push(value);

            bufferedLogToConsole('// Got ' + uniform.name + ' value ' + es3fUniformApiTests.apiVarValueStr(value));
        }

        return success;
    };

    /**
     * @param {Array<es3fUniformApiTests.VarValue>} values
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.checkUniformDefaultValues = function(values, basicUniforms) {
        /** @type {boolean} */ var success = true;

        assertMsgOptions(
            values.length == basicUniforms.length,
            'es3fUniformApiTests.UniformCase.prototype.checkUniformDefaultValues - lengths do not match',
            false,
            true
        );

        for (var unifNdx = 0; unifNdx < basicUniforms.length; unifNdx++) {
            /** @type {es3fUniformApiTests.BasicUniform} */ var uniform = basicUniforms[unifNdx];
            /** @type {es3fUniformApiTests.VarValue} */ var unifValue = values[unifNdx];
            /** @type {number} */ var valSize = gluShaderUtil.getDataTypeScalarSize(uniform.type);

            bufferedLogToConsole('// Checking uniform ' + uniform.name);

            if (unifValue.type == gluShaderUtil.DataType.INVALID) // This happens when glGetUniformLocation() returned -1.
                continue;

            var CHECK_UNIFORM = function(ZERO) {
                do {
                    for (var i = 0; i < valSize; i++) {
                        if (unifValue.val[i] != ZERO) {
                            bufferedLogToConsole('// FAILURE: uniform ' + uniform.name + ' has non-zero initial value');
                            success = false;
                        }
                    }
                } while (false);
            };

            if (gluShaderUtil.isDataTypeFloatOrVec(uniform.type) || gluShaderUtil.isDataTypeMatrix(uniform.type))
                CHECK_UNIFORM(0.0);
            else if (gluShaderUtil.isDataTypeIntOrIVec(uniform.type))
                CHECK_UNIFORM(0);
            else if (gluShaderUtil.isDataTypeUintOrUVec(uniform.type))
                CHECK_UNIFORM(0);
            else if (gluShaderUtil.isDataTypeBoolOrBVec(uniform.type))
                CHECK_UNIFORM(false);
            else if (gluShaderUtil.isDataTypeSampler(uniform.type)) {
                if (unifValue.val.samplerV.unit != 0) {
                    bufferedLogToConsole('// FAILURE: uniform ' + uniform.name + ' has non-zero initial value');
                    success = false;
                }
            } else
                throw new Error('es3fUniformApiTests.UniformCase.prototype.checkUniformDefaultValues - invalid uniform type');
        }

        return success;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {WebGLProgram} programGL
     * @param {deRandom.Random} rnd
     */
    es3fUniformApiTests.UniformCase.prototype.assignUniforms = function(basicUniforms, programGL, rnd) {
        /** @type {boolean} */ var transpose = false; //No support to transpose uniform matrices in WebGL, must always be false. (this.m_features.MATRIXMODE_ROWMAJOR) != 0;
        /** @type {boolean} (GLboolean) */ var transposeGL = transpose;
        /** @type {gluShaderUtil.DataType} */ var boolApiType = this.m_features.BOOLEANAPITYPE_INT ? gluShaderUtil.DataType.INT :
                                                this.m_features.BOOLEANAPITYPE_UINT ? gluShaderUtil.DataType.UINT :
                                                gluShaderUtil.DataType.FLOAT;

        for (var unifNdx = 0; unifNdx < basicUniforms.length; unifNdx++) {
            /** @type {es3fUniformApiTests.BasicUniform} */ var uniform = basicUniforms[unifNdx];
            /** @type {boolean} */ var isArrayMember = uniform.elemNdx >= 0;
            /** @type {string} */ var queryName = this.m_features.ARRAY_FIRST_ELEM_NAME_NO_INDEX && uniform.elemNdx == 0 ? es3fUniformApiTests.beforeLast(uniform.name, '[') : uniform.name;
            /** @type {number} */ var numValuesToAssign = !isArrayMember ? 1 :
                                                        this.m_features.ARRAYASSIGN_FULL ? (uniform.elemNdx == 0 ? uniform.rootSize : 0) :
                                                        this.m_features.ARRAYASSIGN_BLOCKS_OF_TWO ? (uniform.elemNdx % 2 == 0 ? 2 : 0) :
                                                        /* Default: assign array elements separately */ 1;

            assertMsgOptions(
                numValuesToAssign >= 0,
                'es3fUniformApiTests.UniformCase.prototype.assignUniforms - number of values to assign not a positive integer',
                false,
                true
            );
            assertMsgOptions(
                numValuesToAssign == 1 || isArrayMember,
                'es3fUniformApiTests.UniformCase.prototype.assignUniforms - not an array member and number of values to assign not 1',
                false,
                true
            );

            if (numValuesToAssign == 0) {
                bufferedLogToConsole('// es3fUniformApiTests.Uniform ' + uniform.name + ' is covered by another glUniform*v() call to the same array');
                continue;
            }

            /** @type {WebGLUniformLocation} */ var location = gl.getUniformLocation(programGL, queryName);
            /** @type {number} */ var typeSize = gluShaderUtil.getDataTypeScalarSize(uniform.type);
            /** @type {boolean} */ var assignByValue = this.m_features.UNIFORMFUNC_VALUE && !gluShaderUtil.isDataTypeMatrix(uniform.type) && numValuesToAssign == 1;
            /** @type {Array<es3fUniformApiTests.VarValue>} */ var valuesToAssign = [];
            /** @type {Array<number>} */ var buffer;

            for (var i = 0; i < numValuesToAssign; i++) {
                /** @type {string} */ var curName = isArrayMember ? es3fUniformApiTests.beforeLast(uniform.rootName, '[') + '[' + (uniform.elemNdx + i) + ']' : uniform.name;
                /** @type {es3fUniformApiTests.VarValue} */ var unifValue = new es3fUniformApiTests.VarValue();

                if (isArrayMember) {
                    /** @type {es3fUniformApiTests.BasicUniform} */ var elemUnif = es3fUniformApiTests.BasicUniform.findWithName(basicUniforms, curName);
                    if (elemUnif === null)
                        continue;
                    unifValue = elemUnif.finalValue;
                } else
                    unifValue = uniform.finalValue;

                /** @type {es3fUniformApiTests.VarValue} */ var apiValue = gluShaderUtil.isDataTypeBoolOrBVec(unifValue.type) ? es3fUniformApiTests.getRandomBoolRepresentation(unifValue, boolApiType, rnd) :
                gluShaderUtil.isDataTypeSampler(unifValue.type) ? es3fUniformApiTests.getSamplerUnitValue(unifValue) :
                                        unifValue;

                valuesToAssign.push(gluShaderUtil.isDataTypeMatrix(apiValue.type) && transpose ? es3fUniformApiTests.getTransposeMatrix(apiValue) : apiValue);

                if (gluShaderUtil.isDataTypeBoolOrBVec(uniform.type))
                    bufferedLogToConsole('// Using type ' + gluShaderUtil.getDataTypeName(boolApiType) + ' to set boolean value ' + es3fUniformApiTests.apiVarValueStr(unifValue) + ' for ' + curName);
                else if (gluShaderUtil.isDataTypeSampler(uniform.type))
                    bufferedLogToConsole('// Texture for the sampler uniform ' + curName + ' will be filled with color ' + es3fUniformApiTests.apiVarValueStr(es3fUniformApiTests.getSamplerFillValue(uniform.finalValue)));
            }

            assertMsgOptions(
                valuesToAssign.length > 0,
                'es3fUniformApiTests.UniformCase.prototype.assignUniforms - values quantity less than one',
                false,
                true
            );

            if (gluShaderUtil.isDataTypeFloatOrVec(valuesToAssign[0].type)) {
                if (assignByValue) {
                    switch (typeSize) {
                        case 1: gl.uniform1f(location, valuesToAssign[0].val[0]); break;
                        case 2: gl.uniform2f(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1]); break;
                        case 3: gl.uniform3f(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1], valuesToAssign[0].val[2]); break;
                        case 4: gl.uniform4f(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1], valuesToAssign[0].val[2], valuesToAssign[0].val[3]); break;
                        default:
                            throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid type size');
                    }
                } else {
                    buffer = new Array(valuesToAssign.length * typeSize);
                    for (var i = 0; i < buffer.length; i++)
                        buffer[i] = valuesToAssign[Math.floor(i / typeSize)].val[i % typeSize];

                    switch (typeSize) {
                        case 1: gl.uniform1fv(location, buffer); break;
                        case 2: gl.uniform2fv(location, buffer); break;
                        case 3: gl.uniform3fv(location, buffer); break;
                        case 4: gl.uniform4fv(location, buffer); break;
                        default:
                            throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid type size');
                    }
                }
            } else if (gluShaderUtil.isDataTypeMatrix(valuesToAssign[0].type)) {
                assertMsgOptions(
                    !assignByValue,
                    'es3fUniformApiTests.UniformCase.prototype.assignUniforms - assigning by value in matrix type',
                    false, true
                );

                buffer = new Array(valuesToAssign.length * typeSize);
                for (var i = 0; i < buffer.length; i++)
                    buffer[i] = valuesToAssign[Math.floor(i / typeSize)].val[i % typeSize];

                switch (uniform.type) {
                    case gluShaderUtil.DataType.FLOAT_MAT2: gl.uniformMatrix2fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT3: gl.uniformMatrix3fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT4: gl.uniformMatrix4fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT2X3: gl.uniformMatrix2x3fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT2X4: gl.uniformMatrix2x4fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT3X2: gl.uniformMatrix3x2fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT3X4: gl.uniformMatrix3x4fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT4X2: gl.uniformMatrix4x2fv(location, transposeGL, new Float32Array(buffer)); break;
                    case gluShaderUtil.DataType.FLOAT_MAT4X3: gl.uniformMatrix4x3fv(location, transposeGL, new Float32Array(buffer)); break;
                    default:
                        throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid uniform type');
                }
            } else if (gluShaderUtil.isDataTypeIntOrIVec(valuesToAssign[0].type)) {
                if (assignByValue) {
                    switch (typeSize) {
                        case 1: gl.uniform1i(location, valuesToAssign[0].val[0]); break;
                        case 2: gl.uniform2i(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1]); break;
                        case 3: gl.uniform3i(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1], valuesToAssign[0].val[2]); break;
                        case 4: gl.uniform4i(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1], valuesToAssign[0].val[2], valuesToAssign[0].val[3]); break;
                        default:
                            throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid type size');
                    }
                } else {
                    buffer = new Array(valuesToAssign.length * typeSize);
                    for (var i = 0; i < buffer.length; i++)
                        buffer[i] = valuesToAssign[Math.floor(i / typeSize)].val[i % typeSize];

                    switch (typeSize) {
                        case 1: gl.uniform1iv(location, buffer); break;
                        case 2: gl.uniform2iv(location, buffer); break;
                        case 3: gl.uniform3iv(location, buffer); break;
                        case 4: gl.uniform4iv(location, buffer); break;
                        default:
                            throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid type size');
                    }
                }
            } else if (gluShaderUtil.isDataTypeUintOrUVec(valuesToAssign[0].type)) {
                if (assignByValue) {
                    switch (typeSize) {
                        case 1: gl.uniform1ui(location, valuesToAssign[0].val[0]); break;
                        case 2: gl.uniform2ui(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1]); break;
                        case 3: gl.uniform3ui(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1], valuesToAssign[0].val[2]); break;
                        case 4: gl.uniform4ui(location, valuesToAssign[0].val[0], valuesToAssign[0].val[1], valuesToAssign[0].val[2], valuesToAssign[0].val[3]); break;
                        default:
                            throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid type size');
                    }
                } else {
                    buffer = new Array(valuesToAssign.length * typeSize);
                    for (var i = 0; i < buffer.length; i++)
                        buffer[i] = valuesToAssign[Math.floor(i / typeSize)].val[i % typeSize];

                    switch (typeSize) {
                        case 1: gl.uniform1uiv(location, buffer); break;
                        case 2: gl.uniform2uiv(location, buffer); break;
                        case 3: gl.uniform3uiv(location, buffer); break;
                        case 4: gl.uniform4uiv(location, buffer); break;
                        default:
                            throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid type size');
                    }
                }
            } else if (gluShaderUtil.isDataTypeSampler(valuesToAssign[0].type)) {
                if (assignByValue)
                   gl.uniform1i(location, uniform.finalValue.val.samplerV.unit);
                else {
                    var unit = /** @type {Array<number>} */ (uniform.finalValue.val);
                   gl.uniform1iv(location, unit);
                }
            } else
                throw new Error('es3fUniformApiTests.UniformCase.prototype.assignUniforms - Invalid uniform type');
        }
    };

    /**
     * @param {Array<es3fUniformApiTests.VarValue>} values
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.compareUniformValues = function(values, basicUniforms) {
        /** @type {boolean} */ var success = true;

        for (var unifNdx = 0; unifNdx < basicUniforms.length; unifNdx++) {
            /** @type {es3fUniformApiTests.BasicUniform} */ var uniform = basicUniforms[unifNdx];
            /** @type {es3fUniformApiTests.VarValue} */ var unifValue = values[unifNdx];

            bufferedLogToConsole('// Checking uniform ' + uniform.name);

            if (unifValue.type == gluShaderUtil.DataType.INVALID) // This happens when glGetUniformLocation() returned -1.
                continue;

            if (!es3fUniformApiTests.apiVarValueEquals(unifValue, uniform.finalValue)) {
                bufferedLogToConsole('// FAILURE: value obtained with glGetUniform*() for uniform ' + uniform.name + ' differs from value set with glUniform*()');
                success = false;
            }
        }

        return success;
    };

    /** @const @type {number} */ es3fUniformApiTests.VIEWPORT_WIDTH = 128;
    /** @const @type {number} */ es3fUniformApiTests.VIEWPORT_HEIGHT = 128;

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {gluShaderProgram.ShaderProgram} program
     * @param {deRandom.Random} rnd
     * @return {boolean}
     */
    es3fUniformApiTests.UniformCase.prototype.renderTest = function(basicUniforms, program, rnd) {
        //const tcu::RenderTarget& renderTarget = m_context.getRenderTarget();
        /** @const */ var viewportW = Math.min(gl.canvas.width, es3fUniformApiTests.VIEWPORT_WIDTH);
        /** @const */ var viewportH = Math.min(gl.canvas.height, es3fUniformApiTests.VIEWPORT_HEIGHT);
        /** @const */ var viewportX = rnd.getInt(0, gl.canvas.width - viewportW);
        /** @const */ var viewportY = rnd.getInt(0, gl.canvas.height - viewportH);
        /** @type {tcuSurface.Surface} */ var renderedImg = new tcuSurface.Surface(viewportW, viewportH);

        // Assert that no two samplers of different types have the same texture unit - this is an error in GL.
        for (var i = 0; i < basicUniforms.length; i++) {
            if (gluShaderUtil.isDataTypeSampler(basicUniforms[i].type)) {
                for (var j = 0; j < i; j++) {
                    if (gluShaderUtil.isDataTypeSampler(basicUniforms[j].type) && basicUniforms[i].type != basicUniforms[j].type)
                        assertMsgOptions(
                            basicUniforms[i].finalValue.val.samplerV.unit != basicUniforms[j].finalValue.val.samplerV.unit,
                            'es3fUniformApiTests.UniformCase.prototype.renderTest - sampler units have the same texture unit',
                            false, true
                        );
                }
            }
        }

        for (var i = 0; i < basicUniforms.length; i++) {
            if (gluShaderUtil.isDataTypeSampler(basicUniforms[i].type) && this.m_filledTextureUnits.indexOf(basicUniforms[i].finalValue.val) == -1) {
                bufferedLogToConsole('// Filling texture at unit ' + es3fUniformApiTests.apiVarValueStr(basicUniforms[i].finalValue) + ' with color ' + es3fUniformApiTests.shaderVarValueStr(basicUniforms[i].finalValue));
                this.setupTexture(basicUniforms[i].finalValue);
            }
        }

       gl.viewport(viewportX, viewportY, viewportW, viewportH);

        /** @type {Float32Array} */ var position = new Float32Array([
            -1.0, -1.0, 0.0, 1.0,
            -1.0, 1.0, 0.0, 1.0,
            1.0, -1.0, 0.0, 1.0,
            1.0, 1.0, 0.0, 1.0
        ]);

        /** @type {Uint16Array} */
        var indices = new Uint16Array([0, 1, 2, 2, 1, 3]);

        /** @type {number} */ var posLoc = gl.getAttribLocation(program.getProgram(), 'a_position');
        gl.enableVertexAttribArray(posLoc);

        var gl_position_buffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, gl_position_buffer);
        gl.bufferData(gl.ARRAY_BUFFER, position, gl.STATIC_DRAW);
        gl.vertexAttribPointer(posLoc, 4, gl.FLOAT, false, 0, 0);

        var gl_index_buffer = gl.createBuffer();
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, gl_index_buffer);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);

        gl.drawElements(gl.TRIANGLES, indices.length, gl.UNSIGNED_SHORT, 0);

        renderedImg.readViewport(gl, [viewportX, viewportY, viewportW, viewportH]);

        /** @type {number} */ var numFailedPixels = 0;
        var whitePixel = new gluDrawUtil.Pixel([255.0, 255.0, 255.0, 255.0]);
        for (var y = 0; y < renderedImg.getHeight(); y++) {
            for (var x = 0; x < renderedImg.getWidth(); x++) {
                var currentPixel = new gluDrawUtil.Pixel(renderedImg.getPixel(x, y));
                if (!whitePixel.equals(currentPixel))
                    numFailedPixels += 1;
            }
        }

        if (numFailedPixels > 0) {
            //TODO: log << TestLog::Image("RenderedImage", "Rendered image", renderedImg);
            bufferedLogToConsole('FAILURE: image comparison failed, got ' + numFailedPixels + ' non-white pixels');
            return false;
        } else {
            bufferedLogToConsole('Success: got all-white pixels (all uniforms have correct values)');
            return true;
        }
    };

    /**
     * @return {tcuTestCase.IterateResult}
     */
    es3fUniformApiTests.UniformCase.prototype.iterate = function() {
        /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name) ^ deRandom.getBaseSeed());
        /** @type {Array<es3fUniformApiTests.BasicUniform>} */ var basicUniforms = [];
        /** @type {Array<es3fUniformApiTests.BasicUniformReportRef>} */ var basicUniformReportsRef = [];

        /** @type {number} */ var samplerUnitCounter = 0;
        for (var i = 0; i < this.m_uniformCollection.getNumUniforms(); i++)
            samplerUnitCounter = this.generateBasicUniforms(basicUniforms, basicUniformReportsRef, this.m_uniformCollection.getUniform(i).type, this.m_uniformCollection.getUniform(i).name, true, samplerUnitCounter, rnd);

        /** @type {string} */ var vertexSource = this.generateVertexSource(basicUniforms);
        /** @type {string} */ var fragmentSource = this.generateFragmentSource(basicUniforms);
        /** @type {gluShaderProgram.ShaderProgram} */ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vertexSource, fragmentSource));

        bufferedLogToConsole(program.getProgramInfo().infoLog);

        if (!program.isOk()) {
            testFailedOptions('Compile failed', false);
            return tcuTestCase.IterateResult.STOP;
        }

       gl.useProgram(program.getProgram());

        /** @type {boolean} */ var success = this.test(basicUniforms, basicUniformReportsRef, program, rnd);
        assertMsgOptions(success, '', true, false);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @enum {number}
     */
    es3fUniformApiTests.CaseType = {
        UNIFORM: 0, //!< Check info returned by glGetActiveUniform().
        INDICES_UNIFORMSIV: 1, //!< Check info returned by glGetUniformIndices() + glGetActiveUniforms(). TODO: Check 'IV' part
        CONSISTENCY: 2 //!< Query info with both above methods, and check consistency.
    };

    /**
     * es3fUniformApiTests.UniformInfoQueryCase class
     * @constructor
     * @param {string} name
     * @param {string} description
     * @param {es3fUniformApiTests.CaseShaderType} shaderType
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection
     * @param {es3fUniformApiTests.CaseType} caseType
     * @param {es3fUniformApiTests.Feature} additionalFeatures
     * @extends {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformInfoQueryCase = function(name, description, shaderType, uniformCollection, caseType, additionalFeatures) {
        es3fUniformApiTests.UniformCase.call(this, name, description);
        this.newB(shaderType, uniformCollection, additionalFeatures);
        /** @type {es3fUniformApiTests.CaseType} */ this.m_caseType = caseType;
    };

    es3fUniformApiTests.UniformInfoQueryCase.prototype = Object.create(es3fUniformApiTests.UniformCase.prototype);
    /** Constructor restore */
    es3fUniformApiTests.UniformInfoQueryCase.prototype.constructor = es3fUniformApiTests.UniformInfoQueryCase;

    /**
     * @param {es3fUniformApiTests.CaseType} caseType
     * @return {?string}
     */
    es3fUniformApiTests.UniformInfoQueryCase.getCaseTypeName = function(caseType) {
        switch (caseType) {
            case es3fUniformApiTests.CaseType.UNIFORM: return 'active_uniform';
            case es3fUniformApiTests.CaseType.INDICES_UNIFORMSIV: return 'indices_active_uniformsiv';
            case es3fUniformApiTests.CaseType.CONSISTENCY: return 'consistency';
            default:
                throw new Error('Invalid type');
        }
    };

    /**
     * @param {es3fUniformApiTests.CaseType} caseType
     * @return {?string}
     */
    es3fUniformApiTests.UniformInfoQueryCase.getCaseTypeDescription = function(caseType) {
       switch (caseType) {
           case es3fUniformApiTests.CaseType.UNIFORM: return 'Test glGetActiveUniform()';
           case es3fUniformApiTests.CaseType.INDICES_UNIFORMSIV: return 'Test glGetUniformIndices() along with glGetActiveUniforms()';
           case es3fUniformApiTests.CaseType.CONSISTENCY: return 'Check consistency between results from glGetActiveUniform() and glGetUniformIndices() + glGetActiveUniforms()';
           default:
               throw new Error('Invalid type');
       }
    };

    // \note Although this is only used in UniformApiTest::es3fUniformApiTests.init, it needs to be defined here as it's used as a template argument.
    /**
     * @constructor
     * @param {?string} name
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection_
     */
    es3fUniformApiTests.UniformCollectionCase = function(name, uniformCollection_) {
        /** @type {string} */ this.namePrefix = name ? name + '_' : '';
        /** @type {es3fUniformApiTests.UniformCollection} (SharedPtr) */ this.uniformCollection = uniformCollection_;
    };

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {Array<es3fUniformApiTests.BasicUniformReportRef>} basicUniformReportsRef
     * @param {gluShaderProgram.ShaderProgram} program
     * @param {deRandom.Random} rnd
     * @return {boolean}
     */
    es3fUniformApiTests.UniformInfoQueryCase.prototype.test = function(basicUniforms, basicUniformReportsRef, program, rnd) {
        /** @type {WebGLProgram} */ var programGL = program.getProgram();
        /** @type {Array<es3fUniformApiTests.BasicUniformReportGL>} */ var basicUniformReportsUniform = [];
        /** @type {Array<es3fUniformApiTests.BasicUniformReportGL>} */ var basicUniformReportsUniforms = [];
        /** @type {boolean} */ var success;

        if (this.m_caseType == es3fUniformApiTests.CaseType.UNIFORM || this.m_caseType == es3fUniformApiTests.CaseType.CONSISTENCY) {
            success = false;

            //TODO:: const ScopedLogSection section(log, "InfoGetActiveUniform", "es3fUniformApiTests.Uniform information queries with glGetActiveUniform()");
            success = this.getActiveUniformsOneByOne(basicUniformReportsUniform, basicUniformReportsRef, programGL);

            if (!success) {
                if (this.m_caseType == es3fUniformApiTests.CaseType.UNIFORM)
                    return false;
                else {
                    assertMsgOptions(
                        this.m_caseType == es3fUniformApiTests.CaseType.CONSISTENCY,
                        'es3fUniformApiTests.UniformInfoQueryCase.prototype.test - case type is not consistency',
                        false,
                        true
                    );
                    bufferedLogToConsole('// Note: this is a consistency case, so ignoring above failure(s)');
                }
            }
        }

        if (this.m_caseType == es3fUniformApiTests.CaseType.INDICES_UNIFORMSIV || this.m_caseType == es3fUniformApiTests.CaseType.CONSISTENCY) {
            success = false;

            //TODO: const ScopedLogSection section(log, "InfoGetActiveUniforms", "es3fUniformApiTests.Uniform information queries with glGetUniformIndices() and glGetActiveUniforms()");
            success = this.getActiveUniforms(basicUniformReportsUniforms, basicUniformReportsRef, programGL);

            if (!success) {
                if (this.m_caseType == es3fUniformApiTests.CaseType.INDICES_UNIFORMSIV)
                    return false;
                else {
                    assertMsgOptions(
                        this.m_caseType == es3fUniformApiTests.CaseType.CONSISTENCY,
                        'es3fUniformApiTests.UniformInfoQueryCase.prototype.test - case type is not consistency',
                        false,
                        true
                    );
                    bufferedLogToConsole('// Note: this is a consistency case, so ignoring above failure(s)');
                }
            }
        }

        if (this.m_caseType == es3fUniformApiTests.CaseType.CONSISTENCY) {
            success = false;

            //TODO: const ScopedLogSection section(log, "CompareUniformVsUniforms", "Comparison of results from glGetActiveUniform() and glGetActiveUniforms()");
            success = this.uniformVsUniformsComparison(basicUniformReportsUniform, basicUniformReportsUniforms);

            if (!success)
                return false;
        }

        return true;
    };

    /**
     * @enum {number}
     */
    es3fUniformApiTests.ValueToCheck = {
        INITIAL: 0, //!< Verify the initial values of the uniforms (i.e. check that they're zero).
        ASSIGNED: 1 //!< Assign values to uniforms with glUniform*(), and check those.
    };

    /**
     * @enum {number}
     */
    es3fUniformApiTests.CheckMethod = {
        GET_UNIFORM: 0, //!< Check values with glGetUniform*().
        RENDER: 1 //!< Check values by rendering with the value-checking shader.
    };

    /**
     * @enum {number}
     */
    es3fUniformApiTests.AssignMethod = {
        POINTER: 0,
        VALUE: 1
    };

    /**
     * es3fUniformApiTests.UniformValueCase test class
     * @constructor
     * @param {string} name
     * @param {string} description
     * @param {es3fUniformApiTests.CaseShaderType} shaderType
     * @param {es3fUniformApiTests.UniformCollection} uniformCollection (SharedPtr)
     * @param {es3fUniformApiTests.ValueToCheck} valueToCheck
     * @param {es3fUniformApiTests.CheckMethod} checkMethod
     * @param {?es3fUniformApiTests.AssignMethod} assignMethod
     * @param {es3fUniformApiTests.Feature} additionalFeatures
     * @extends {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.UniformValueCase = function(name, description, shaderType, uniformCollection, valueToCheck, checkMethod, assignMethod, additionalFeatures) {
        es3fUniformApiTests.UniformCase.call(this, name, description);

        additionalFeatures.UNIFORMVALUE_ZERO |= valueToCheck == es3fUniformApiTests.ValueToCheck.INITIAL;
        additionalFeatures.UNIFORMFUNC_VALUE |= assignMethod == es3fUniformApiTests.AssignMethod.VALUE;
        this.newB(shaderType, uniformCollection, additionalFeatures);

        this.m_valueToCheck = valueToCheck;
        this.m_checkMethod = checkMethod;

        assertMsgOptions(
            !(assignMethod === undefined && valueToCheck == es3fUniformApiTests.ValueToCheck.ASSIGNED),
            'es3fUniformApiTests.UniformValueCase - assign method is undefined when value to check requires it',
            false,
            true
        );
    };

    es3fUniformApiTests.UniformValueCase.prototype = Object.create(es3fUniformApiTests.UniformCase.prototype);
    /** Constructor restore */
    es3fUniformApiTests.UniformValueCase.prototype.constructor = es3fUniformApiTests.UniformValueCase;

    /**
     * @param {es3fUniformApiTests.ValueToCheck} valueToCheck
     * @return {?string}
     */
    es3fUniformApiTests.UniformValueCase.getValueToCheckName = function(valueToCheck) {
        switch (valueToCheck) {
            case es3fUniformApiTests.ValueToCheck.INITIAL: return 'initial';
            case es3fUniformApiTests.ValueToCheck.ASSIGNED: return 'assigned';
            default: throw new Error('es3fUniformApiTests.UniformValueCase.getValueToCheckName - Invalid value to check option');
        }
    };

    /**
     * @param {es3fUniformApiTests.ValueToCheck} valueToCheck
     * @return {?string}
     */
    es3fUniformApiTests.UniformValueCase.getValueToCheckDescription = function(valueToCheck) {
        switch (valueToCheck) {
            case es3fUniformApiTests.ValueToCheck.INITIAL: return 'Check initial uniform values (zeros)';
            case es3fUniformApiTests.ValueToCheck.ASSIGNED: return 'Check assigned uniform values';
            default: throw new Error('es3fUniformApiTests.UniformValueCase.getValueToCheckDescription - Invalid value to check option');
        }
    };

    /**
     * @param {es3fUniformApiTests.CheckMethod} checkMethod
     * @return {?string}
     */
    es3fUniformApiTests.UniformValueCase.getCheckMethodName = function(checkMethod) {
        switch (checkMethod) {
            case es3fUniformApiTests.CheckMethod.GET_UNIFORM: return 'get_uniform';
            case es3fUniformApiTests.CheckMethod.RENDER: return 'render';
            default: throw new Error('es3fUniformApiTests.UniformValueCase.getCheckMethodName - Invalid check method');
        }
    };

    /**
     * @param {es3fUniformApiTests.CheckMethod} checkMethod
     * @return {?string}
     */
    es3fUniformApiTests.UniformValueCase.getCheckMethodDescription = function(checkMethod) {
        switch (checkMethod) {
            case es3fUniformApiTests.CheckMethod.GET_UNIFORM: return 'Verify values with glGetUniform*()';
            case es3fUniformApiTests.CheckMethod.RENDER: return 'Verify values by rendering';
            default: throw new Error('es3fUniformApiTests.UniformValueCase.getCheckMethodDescription - Invalid check method');
        }
    };

    /**
     * @param {es3fUniformApiTests.AssignMethod} assignMethod
     * @return {?string}
     */
    es3fUniformApiTests.UniformValueCase.getAssignMethodName = function(assignMethod) {
        switch (assignMethod) {
            case es3fUniformApiTests.AssignMethod.POINTER: return 'by_pointer';
            case es3fUniformApiTests.AssignMethod.VALUE: return 'by_value';
            default: throw new Error('es3fUniformApiTests.UniformValueCase.getAssignMethodName - Invalid assign method');
        }
    };

    /**
     * @param {es3fUniformApiTests.AssignMethod} assignMethod
     * @return {?string}
     */
    es3fUniformApiTests.UniformValueCase.getAssignMethodDescription = function(assignMethod) {
        switch (assignMethod) {
            case es3fUniformApiTests.AssignMethod.POINTER: return 'Assign values by-pointer';
            case es3fUniformApiTests.AssignMethod.VALUE: return 'Assign values by-value';
            default: throw new Error('es3fUniformApiTests.UniformValueCase.getAssignMethodDescription - Invalid assign method');
        }
    };

    /**
     * es3fUniformApiTests.UniformValueCase test function
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {Array<es3fUniformApiTests.BasicUniformReportRef>} basicUniformReportsRef
     * @param {gluShaderProgram.ShaderProgram} program
     * @param {deRandom.Random} rnd
     * @return {boolean}
     */
    es3fUniformApiTests.UniformValueCase.prototype.test = function(basicUniforms, basicUniformReportsRef, program, rnd) {
        /** @type {WebGLProgram} */ var programGL = program.getProgram();

        if (this.m_valueToCheck == es3fUniformApiTests.ValueToCheck.ASSIGNED) {
            //TODO: const ScopedLogSection section(log, "UniformAssign", "es3fUniformApiTests.Uniform value assignments");
            this.assignUniforms(basicUniforms, programGL, rnd);
        } else
            assertMsgOptions(
                this.m_valueToCheck == es3fUniformApiTests.ValueToCheck.INITIAL,
                'es3fUniformApiTests.UniformValueCase.prototype.test - value to check not initial',
                false, true
            );

        /** @type {boolean}*/ var success;

        if (this.m_checkMethod == es3fUniformApiTests.CheckMethod.GET_UNIFORM) {
            /** @type {Array<es3fUniformApiTests.VarValue>} */ var values = [];

            //TODO: const ScopedLogSection section(log, "GetUniforms", "es3fUniformApiTests.Uniform value query");
            success = this.getUniforms(values, basicUniforms, program.getProgram());

            if (!success)
                return false;

            if (this.m_valueToCheck == es3fUniformApiTests.ValueToCheck.ASSIGNED) {
                //TODO: const ScopedLogSection section(log, "ValueCheck", "Verify that the reported values match the assigned values");
                success = this.compareUniformValues(values, basicUniforms);

                if (!success)
                    return false;
            } else {
                assertMsgOptions(
                    this.m_valueToCheck == es3fUniformApiTests.ValueToCheck.INITIAL,
                    'es3fUniformApiTests.UniformValueCase.prototype.test - value to check not initial',
                    false, true
                );

                //TODO: const ScopedLogSection section(log, "ValueCheck", "Verify that the uniforms have correct initial values (zeros)");
                success = this.checkUniformDefaultValues(values, basicUniforms);

                if (!success)
                    return false;
            }
        } else {
            assertMsgOptions(
                this.m_checkMethod == es3fUniformApiTests.CheckMethod.RENDER,
                'es3fUniformApiTests.UniformValueCase.prototype.test - check method different than RENDER',
                false, true
            );

            //TODO: const ScopedLogSection section(log, "RenderTest", "Render test");
            success = this.renderTest(basicUniforms, program, rnd);

            if (!success)
                return false;
        }

        return true;
    };

    /**
     * es3fUniformApiTests.RandomUniformCase test class
     * @constructor
     * @param {string} name
     * @param {string} description
     * @param {number} seed
     * @extends {es3fUniformApiTests.UniformCase}
     */
    es3fUniformApiTests.RandomUniformCase = function(name, description, seed) {
        es3fUniformApiTests.UniformCase.call(this, name, description);
        this.newC(seed ^ deRandom.getBaseSeed());
    };

    es3fUniformApiTests.RandomUniformCase.prototype = Object.create(es3fUniformApiTests.UniformCase.prototype);
    /** Constructor restore */
    es3fUniformApiTests.RandomUniformCase.prototype.constructor = es3fUniformApiTests.RandomUniformCase;

    /**
     * @param {Array<es3fUniformApiTests.BasicUniform>} basicUniforms
     * @param {Array<es3fUniformApiTests.BasicUniformReportRef>} basicUniformReportsRef
     * @param {gluShaderProgram.ShaderProgram} program
     * @param {deRandom.Random} rnd
     * @return {boolean}
     */
    es3fUniformApiTests.RandomUniformCase.prototype.test = function(basicUniforms, basicUniformReportsRef, program, rnd) {
        // \note Different sampler types may not be bound to same unit when rendering.
        /** @type {boolean}*/ var renderingPossible = !this.m_features.UNIFORMVALUE_ZERO || !this.m_uniformCollection.containsSeveralSamplerTypes();

        /** @type {boolean} */ var performGetActiveUniforms = rnd.getBool();
        /** @type {boolean} */ var performGetActiveUniformsiv = rnd.getBool();
        /** @type {boolean} */ var performUniformVsUniformsivComparison = performGetActiveUniforms && performGetActiveUniformsiv && rnd.getBool();
        /** @type {boolean} */ var performGetUniforms = rnd.getBool();
        /** @type {boolean} */ var performCheckUniformDefaultValues = performGetUniforms && rnd.getBool();
        /** @type {boolean} */ var performAssignUniforms = rnd.getBool();
        /** @type {boolean} */ var performCompareUniformValues = performGetUniforms && performAssignUniforms && rnd.getBool();
        /** @type {boolean} */ var performRenderTest = renderingPossible && performAssignUniforms && rnd.getBool();
        /** @type {WebGLProgram} */ var programGL = program.getProgram();

        if (!(performGetActiveUniforms || performGetActiveUniformsiv || performUniformVsUniformsivComparison || performGetUniforms || performCheckUniformDefaultValues || performAssignUniforms || performCompareUniformValues || performRenderTest))
            performGetActiveUniforms = true; // Do something at least.

        var PERFORM_AND_CHECK = function(CALL, SECTION_NAME, SECTION_DESCRIPTION) {
            //TODO: const ScopedLogSection section(log, (SECTION_NAME), (SECTION_DESCRIPTION));
            /** @type {boolean} */ var success = CALL();
            if (!success)
                return false;
        };

        /** @type {Array<es3fUniformApiTests.BasicUniformReportGL>} */ var reportsUniform = [];
        /** @type {Array<es3fUniformApiTests.BasicUniformReportGL>} */ var reportsUniformsiv = [];

        var current = this; //To use "this" in anonymous function.

        if (performGetActiveUniforms)
            PERFORM_AND_CHECK(function() {current.getActiveUniformsOneByOne(reportsUniform, basicUniformReportsRef, programGL);}, 'InfoGetActiveUniform', 'es3fUniformApiTests.Uniform information queries with glGetActiveUniform()');

        if (performGetActiveUniformsiv)
            PERFORM_AND_CHECK(function() {current.getActiveUniforms(reportsUniformsiv, basicUniformReportsRef, programGL);}, 'InfoGetActiveUniformsiv', 'es3fUniformApiTests.Uniform information queries with glGetIndices() and glGetActiveUniformsiv()');

        if (performUniformVsUniformsivComparison)
            PERFORM_AND_CHECK(function() {current.uniformVsUniformsComparison(reportsUniform, reportsUniformsiv);}, 'CompareUniformVsUniformsiv', 'Comparison of results from glGetActiveUniform() and glGetActiveUniformsiv()');

        /** @type {Array<es3fUniformApiTests.VarValue>} */ var uniformDefaultValues = [];

        if (performGetUniforms)
            PERFORM_AND_CHECK(function() {current.getUniforms(uniformDefaultValues, basicUniforms, programGL);}, 'GetUniformDefaults', 'es3fUniformApiTests.Uniform default value query');

        if (performCheckUniformDefaultValues)
            PERFORM_AND_CHECK(function() {current.checkUniformDefaultValues(uniformDefaultValues, basicUniforms);}, 'DefaultValueCheck', 'Verify that the uniforms have correct initial values (zeros)');

        /** @type {Array<es3fUniformApiTests.VarValue>} */ var uniformValues = [];

        if (performAssignUniforms) {
            //TODO: const ScopedLogSection section(log, "UniformAssign", "es3fUniformApiTests.Uniform value assignments");
            this.assignUniforms(basicUniforms, programGL, rnd);
        }

        if (performCompareUniformValues) {
            PERFORM_AND_CHECK(function() {current.getUniforms(uniformValues, basicUniforms, programGL);}, 'GetUniforms', 'es3fUniformApiTests.Uniform value query');
            PERFORM_AND_CHECK(function() {current.compareUniformValues(uniformValues, basicUniforms);}, 'ValueCheck', 'Verify that the reported values match the assigned values');
        }

        if (performRenderTest)
            PERFORM_AND_CHECK(function() {current.renderTest(basicUniforms, program, rnd);}, 'RenderTest', 'Render test');

        return true;
    };

    /**
     * Initializes the tests to be performed.
     */
    es3fUniformApiTests.init = function() {
        var state = tcuTestCase.runner;
        var testGroup = state.testCases;

        // Generate sets of UniformCollections that are used by several cases.
        /**
         * @enum
         */
        var UniformCollections = {
            BASIC: 0,
            BASIC_ARRAY: 1,
            BASIC_STRUCT: 2,
            STRUCT_IN_ARRAY: 3,
            ARRAY_IN_STRUCT: 4,
            NESTED_STRUCTS_ARRAYS: 5,
            MULTIPLE_BASIC: 6,
            MULTIPLE_BASIC_ARRAY: 7,
            MULTIPLE_NESTED_STRUCTS_ARRAYS: 8
        };

        /**
         * @constructor
         */
        var UniformCollectionGroup = function() {
            /** @type {string} */ this.name = '';
            /** @type {Array<es3fUniformApiTests.UniformCollectionCase>} */ this.cases = [];
        };

        /** @type {Array<UniformCollectionGroup>} */ var defaultUniformCollections = new Array(Object.keys(UniformCollections).length);

        /** @type {string} */ var name;

        //Initialize
        for (var i = 0; i < defaultUniformCollections.length; i++) defaultUniformCollections[i] = new UniformCollectionGroup();

        defaultUniformCollections[UniformCollections.BASIC].name = 'basic';
        defaultUniformCollections[UniformCollections.BASIC_ARRAY].name = 'basic_array';
        defaultUniformCollections[UniformCollections.BASIC_STRUCT].name = 'basic_struct';
        defaultUniformCollections[UniformCollections.STRUCT_IN_ARRAY].name = 'struct_in_array';
        defaultUniformCollections[UniformCollections.ARRAY_IN_STRUCT].name = 'array_in_struct';
        defaultUniformCollections[UniformCollections.NESTED_STRUCTS_ARRAYS].name = 'nested_structs_arrays';
        defaultUniformCollections[UniformCollections.MULTIPLE_BASIC].name = 'multiple_basic';
        defaultUniformCollections[UniformCollections.MULTIPLE_BASIC_ARRAY].name = 'multiple_basic_array';
        defaultUniformCollections[UniformCollections.MULTIPLE_NESTED_STRUCTS_ARRAYS].name = 'multiple_nested_structs_arrays';

        for (var dataTypeNdx = 0; dataTypeNdx < es3fUniformApiTests.s_testDataTypes.length; dataTypeNdx++) {
            /** @type {gluShaderUtil.DataType} */ var dataType = es3fUniformApiTests.s_testDataTypes[dataTypeNdx];
            /** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(dataType);

            defaultUniformCollections[UniformCollections.BASIC].cases.push(new es3fUniformApiTests.UniformCollectionCase(typeName, es3fUniformApiTests.UniformCollection.basic(dataType)));

            if (gluShaderUtil.isDataTypeScalar(dataType) ||
                (gluShaderUtil.isDataTypeVector(dataType) && gluShaderUtil.getDataTypeScalarSize(dataType) == 4) ||
                dataType == gluShaderUtil.DataType.FLOAT_MAT4 ||
                dataType == gluShaderUtil.DataType.SAMPLER_2D)
                defaultUniformCollections[UniformCollections.BASIC_ARRAY].cases.push(new es3fUniformApiTests.UniformCollectionCase(typeName, es3fUniformApiTests.UniformCollection.basicArray(dataType)));

            if (gluShaderUtil.isDataTypeScalar(dataType) ||
                dataType == gluShaderUtil.DataType.FLOAT_MAT4 ||
                dataType == gluShaderUtil.DataType.SAMPLER_2D) {
                /** @type {gluShaderUtil.DataType} */ var secondDataType;
                if (gluShaderUtil.isDataTypeScalar(dataType))
                    secondDataType = gluShaderUtil.getDataTypeVector(dataType, 4);
                else if (dataType == gluShaderUtil.DataType.FLOAT_MAT4)
                    secondDataType = gluShaderUtil.DataType.FLOAT_MAT2;
                else if (dataType == gluShaderUtil.DataType.SAMPLER_2D)
                    secondDataType = gluShaderUtil.DataType.SAMPLER_CUBE;

                assertMsgOptions(
                    secondDataType !== undefined,
                    'es3fUniformApiTests.init - second data type undefined',
                    false, true
                );

                /** @type {string} */ var secondTypeName = gluShaderUtil.getDataTypeName(secondDataType);
                name = typeName + '_' + secondTypeName;

                defaultUniformCollections[UniformCollections.BASIC_STRUCT].cases.push(new es3fUniformApiTests.UniformCollectionCase(name, es3fUniformApiTests.UniformCollection.basicStruct(dataType, secondDataType, false)));
                defaultUniformCollections[UniformCollections.ARRAY_IN_STRUCT].cases.push(new es3fUniformApiTests.UniformCollectionCase(name, es3fUniformApiTests.UniformCollection.basicStruct(dataType, secondDataType, true)));
                defaultUniformCollections[UniformCollections.STRUCT_IN_ARRAY].cases.push(new es3fUniformApiTests.UniformCollectionCase(name, es3fUniformApiTests.UniformCollection.structInArray(dataType, secondDataType, false)));
                defaultUniformCollections[UniformCollections.NESTED_STRUCTS_ARRAYS].cases.push(new es3fUniformApiTests.UniformCollectionCase(name, es3fUniformApiTests.UniformCollection.nestedArraysStructs(dataType, secondDataType)));
            }
        }
        defaultUniformCollections[UniformCollections.MULTIPLE_BASIC].cases.push(new es3fUniformApiTests.UniformCollectionCase(null, es3fUniformApiTests.UniformCollection.multipleBasic()));
        defaultUniformCollections[UniformCollections.MULTIPLE_BASIC_ARRAY].cases.push(new es3fUniformApiTests.UniformCollectionCase(null, es3fUniformApiTests.UniformCollection.multipleBasicArray()));
        defaultUniformCollections[UniformCollections.MULTIPLE_NESTED_STRUCTS_ARRAYS].cases.push(new es3fUniformApiTests.UniformCollectionCase(null, es3fUniformApiTests.UniformCollection.multipleNestedArraysStructs()));

        // Info-query cases (check info returned by e.g. glGetActiveUniforms()).

        // info_query
        /** @type {tcuTestCase.DeqpTest} */
        var infoQueryGroup = tcuTestCase.newTest('info_query', 'Test uniform info querying functions');
        testGroup.addChild(infoQueryGroup);

        /** @type {UniformCollectionGroup} */ var collectionGroup;
        /** @type {es3fUniformApiTests.UniformCollectionCase} */ var collectionCase;
        /** @type {es3fUniformApiTests.UniformCollection} (SharedPtr) */ var uniformCollection;
        /** @type {es3fUniformApiTests.Feature} */ var features;
        /** @type {tcuTestCase.DeqpTest} */ var collectionTestGroup;
        /** @type {string} */ var collName;
        /** @type {es3fUniformApiTests.CheckMethod} */ var checkMethod;
        /** @type {tcuTestCase.DeqpTest} */ var checkMethodGroup;
        /** @type {string} */ var collectionGroupName;
        /** @type {boolean} */ var containsBooleans;
        /** @type {boolean} */ var varyBoolApiType;
        /** @type {number} */ var numBoolVariations;
        /** @type {es3fUniformApiTests.Feature} */ var booleanTypeFeat;
        /** @type {string} */ var booleanTypeName;
        /** @type {tcuTestCase.DeqpTest} */ var unusedUniformsGroup;

        /** @type {Array<string>} */ var shaderTypes = Object.keys(es3fUniformApiTests.CaseShaderType);

        for (var caseTypeI in es3fUniformApiTests.CaseType) {
            /** @type {es3fUniformApiTests.CaseType} */ var caseType = es3fUniformApiTests.CaseType[caseTypeI];
            /** @type {tcuTestCase.DeqpTest} */
            var caseTypeGroup = tcuTestCase.newTest(es3fUniformApiTests.UniformInfoQueryCase.getCaseTypeName(caseType), es3fUniformApiTests.UniformInfoQueryCase.getCaseTypeDescription(caseType));
            infoQueryGroup.addChild(caseTypeGroup);

            for (var collectionGroupNdx = 0; collectionGroupNdx < Object.keys(UniformCollections).length; collectionGroupNdx++) {
                var numArrayFirstElemNameCases = caseType == es3fUniformApiTests.CaseType.INDICES_UNIFORMSIV && collectionGroupNdx == UniformCollections.BASIC_ARRAY ? 2 : 1;

                for (var referToFirstArrayElemWithoutIndexI = 0; referToFirstArrayElemWithoutIndexI < numArrayFirstElemNameCases; referToFirstArrayElemWithoutIndexI++) {
                    collectionGroup = defaultUniformCollections[collectionGroupNdx];
                    collectionGroupName = collectionGroup.name + (referToFirstArrayElemWithoutIndexI == 0 ? '' : '_first_elem_without_brackets');
                    collectionTestGroup = tcuTestCase.newTest(collectionGroupName, '');
                    caseTypeGroup.addChild(collectionTestGroup);

                    for (var collectionNdx = 0; collectionNdx < collectionGroup.cases.length; collectionNdx++) {
                        collectionCase = collectionGroup.cases[collectionNdx];

                        for (var i = 0; i < shaderTypes.length; i++) {
                            name = collectionCase.namePrefix + es3fUniformApiTests.getCaseShaderTypeName(es3fUniformApiTests.CaseShaderType[shaderTypes[i]]);
                            uniformCollection = collectionCase.uniformCollection;

                            features = new es3fUniformApiTests.Feature();
                            features.ARRAY_FIRST_ELEM_NAME_NO_INDEX = referToFirstArrayElemWithoutIndexI != 0;

                            collectionTestGroup.addChild(new es3fUniformApiTests.UniformInfoQueryCase(name, '', es3fUniformApiTests.CaseShaderType[shaderTypes[i]], uniformCollection, caseType, features));
                        }
                    }
                }
            }

            // Info-querying cases when unused uniforms are present.

            unusedUniformsGroup = tcuTestCase.newTest('unused_uniforms', 'Test with unused uniforms');
            caseTypeGroup.addChild(unusedUniformsGroup);

            collectionGroup = defaultUniformCollections[UniformCollections.ARRAY_IN_STRUCT];

            for (var collectionNdx = 0; collectionNdx < collectionGroup.cases.length; collectionNdx++) {
                collectionCase = collectionGroup.cases[collectionNdx];
                collName = collectionCase.namePrefix;
                uniformCollection = collectionCase.uniformCollection;

                for (var i = 0; i < shaderTypes.length; i++) {
                    name = collName + es3fUniformApiTests.getCaseShaderTypeName(es3fUniformApiTests.CaseShaderType[shaderTypes[i]]);

                    features = new es3fUniformApiTests.Feature();
                    features.UNIFORMUSAGE_EVERY_OTHER = true;
                    features.ARRAYUSAGE_ONLY_MIDDLE_INDEX = true;

                    unusedUniformsGroup.addChild(new es3fUniformApiTests.UniformInfoQueryCase(name, '', es3fUniformApiTests.CaseShaderType[shaderTypes[i]], uniformCollection, caseType, features));
                }
            }
        }

        // Cases testing uniform values.

        // Cases checking uniforms' initial values (all must be zeros), with glGetUniform*() or by rendering.

        /** @type {tcuTestCase.DeqpTest} */ var initialValuesGroup = tcuTestCase.newTest(
            'value.' + es3fUniformApiTests.UniformValueCase.getValueToCheckName(es3fUniformApiTests.ValueToCheck.INITIAL),
            es3fUniformApiTests.UniformValueCase.getValueToCheckDescription(es3fUniformApiTests.ValueToCheck.INITIAL));
        testGroup.addChild(initialValuesGroup);

        for (var checkMethodI in es3fUniformApiTests.CheckMethod) {
            checkMethod = es3fUniformApiTests.CheckMethod[checkMethodI];
            checkMethodGroup = tcuTestCase.newTest(es3fUniformApiTests.UniformValueCase.getCheckMethodName(checkMethod), es3fUniformApiTests.UniformValueCase.getCheckMethodDescription(checkMethod));
            initialValuesGroup.addChild(checkMethodGroup);

            for (var collectionGroupNdx = 0; collectionGroupNdx < Object.keys(UniformCollections).length; collectionGroupNdx++) {
                collectionGroup = defaultUniformCollections[collectionGroupNdx];
                collectionTestGroup = tcuTestCase.newTest(collectionGroup.name, '');
                checkMethodGroup.addChild(collectionTestGroup);

                for (var collectionNdx = 0; collectionNdx < collectionGroup.cases.length; collectionNdx++) {
                    collectionCase = collectionGroup.cases[collectionNdx];
                    collName = collectionCase.namePrefix;
                    uniformCollection = collectionCase.uniformCollection;
                    containsBooleans = uniformCollection.containsMatchingBasicType(gluShaderUtil.isDataTypeBoolOrBVec);
                    varyBoolApiType = checkMethod == es3fUniformApiTests.CheckMethod.GET_UNIFORM && containsBooleans &&
                                                                (collectionGroupNdx == UniformCollections.BASIC || collectionGroupNdx == UniformCollections.BASIC_ARRAY);
                    numBoolVariations = varyBoolApiType ? 3 : 1;

                    if (checkMethod == es3fUniformApiTests.CheckMethod.RENDER && uniformCollection.containsSeveralSamplerTypes())
                        continue; // \note Samplers' initial API values (i.e. their texture units) are 0, and no two samplers of different types shall have same unit when rendering.

                    for (var booleanTypeI = 0; booleanTypeI < numBoolVariations; booleanTypeI++) {
                        booleanTypeFeat = new es3fUniformApiTests.Feature();
                        booleanTypeFeat.BOOLEANAPITYPE_INT = booleanTypeI == 1;
                        booleanTypeFeat.BOOLEANAPITYPE_UINT = booleanTypeI == 2;

                        booleanTypeName = booleanTypeI == 1 ? 'int' :
                                                            booleanTypeI == 2 ? 'uint' :
                                                            'float';
                        /** @type {string} */ var nameWithApiType = varyBoolApiType ? collName + 'api_' + booleanTypeName + '_' : collName;

                        for (var i = 0; i < shaderTypes.length; i++) {
                            name = nameWithApiType + es3fUniformApiTests.getCaseShaderTypeName(es3fUniformApiTests.CaseShaderType[shaderTypes[i]]);
                            collectionTestGroup.addChild(new es3fUniformApiTests.UniformValueCase(name, '', es3fUniformApiTests.CaseShaderType[shaderTypes[i]], uniformCollection,
                                                                                es3fUniformApiTests.ValueToCheck.INITIAL, checkMethod, null, booleanTypeFeat));
                        }
                    }
                }
            }
        }

        // Cases that first assign values to each uniform, then check the values with glGetUniform*() or by rendering.

        /** @type {tcuTestCase.DeqpTest} */ var assignedValuesGroup = tcuTestCase.newTest(
            'value.' + es3fUniformApiTests.UniformValueCase.getValueToCheckName(es3fUniformApiTests.ValueToCheck.ASSIGNED),
            es3fUniformApiTests.UniformValueCase.getValueToCheckDescription(es3fUniformApiTests.ValueToCheck.ASSIGNED));
        testGroup.addChild(assignedValuesGroup);

        for (var assignMethodI in es3fUniformApiTests.AssignMethod) {
            /** @type {es3fUniformApiTests.AssignMethod} */ var assignMethod = es3fUniformApiTests.AssignMethod[assignMethodI];
            /** @type {tcuTestCase.DeqpTest} */ var assignMethodGroup = tcuTestCase.newTest(es3fUniformApiTests.UniformValueCase.getAssignMethodName(assignMethod), es3fUniformApiTests.UniformValueCase.getAssignMethodDescription(assignMethod));
            assignedValuesGroup.addChild(assignMethodGroup);

            for (var checkMethodI in es3fUniformApiTests.CheckMethod) {
                checkMethod = es3fUniformApiTests.CheckMethod[checkMethodI];
                checkMethodGroup = tcuTestCase.newTest(es3fUniformApiTests.UniformValueCase.getCheckMethodName(checkMethod), es3fUniformApiTests.UniformValueCase.getCheckMethodDescription(checkMethod));
                assignMethodGroup.addChild(checkMethodGroup);

                for (var collectionGroupNdx = 0; collectionGroupNdx < Object.keys(UniformCollections).length; collectionGroupNdx++) {
                    /** @type {number} */ var numArrayFirstElemNameCases = checkMethod == es3fUniformApiTests.CheckMethod.GET_UNIFORM && collectionGroupNdx == UniformCollections.BASIC_ARRAY ? 2 : 1;

                    for (var referToFirstArrayElemWithoutIndexI = 0; referToFirstArrayElemWithoutIndexI < numArrayFirstElemNameCases; referToFirstArrayElemWithoutIndexI++) {
                        collectionGroup = defaultUniformCollections[collectionGroupNdx];
                        collectionGroupName = collectionGroup.name + (referToFirstArrayElemWithoutIndexI == 0 ? '' : '_first_elem_without_brackets');
                        collectionTestGroup = tcuTestCase.newTest(collectionGroupName, '');
                        checkMethodGroup.addChild(collectionTestGroup);

                        for (var collectionNdx = 0; collectionNdx < collectionGroup.cases.length; collectionNdx++) {
                            collectionCase = collectionGroup.cases[collectionNdx];
                            collName = collectionCase.namePrefix;
                            uniformCollection = collectionCase.uniformCollection;
                            containsBooleans = uniformCollection.containsMatchingBasicType(gluShaderUtil.isDataTypeBoolOrBVec);
                            varyBoolApiType = checkMethod == es3fUniformApiTests.CheckMethod.GET_UNIFORM && containsBooleans &&
                                                                            (collectionGroupNdx == UniformCollections.BASIC || collectionGroupNdx == UniformCollections.BASIC_ARRAY);
                            numBoolVariations = varyBoolApiType ? 3 : 1;
                            /** @type {boolean} */ var containsMatrices = uniformCollection.containsMatchingBasicType(gluShaderUtil.isDataTypeMatrix);
                            /** @type {boolean} */ var varyMatrixMode = containsMatrices &&
                                                                            (collectionGroupNdx == UniformCollections.BASIC || collectionGroupNdx == UniformCollections.BASIC_ARRAY);
                            /** @type {number} */ var numMatVariations = varyMatrixMode ? 2 : 1;

                            if (containsMatrices && assignMethod != es3fUniformApiTests.AssignMethod.POINTER)
                                continue;

                            for (var booleanTypeI = 0; booleanTypeI < numBoolVariations; booleanTypeI++) {
                                booleanTypeFeat = new es3fUniformApiTests.Feature();
                                booleanTypeFeat.BOOLEANAPITYPE_INT = booleanTypeI == 1;
                                booleanTypeFeat.BOOLEANAPITYPE_UINT = booleanTypeI == 2;

                                booleanTypeName = booleanTypeI == 1 ? 'int' :
                                                                        booleanTypeI == 2 ? 'uint' :
                                                                        'float';
                                /** @type {string} */ var nameWithBoolType = varyBoolApiType ? collName + 'api_' + booleanTypeName + '_' : collName;

                                for (var matrixTypeI = 0; matrixTypeI < numMatVariations; matrixTypeI++) {
                                    /** @type {string} */ var nameWithMatrixType = nameWithBoolType + (matrixTypeI == 1 ? 'row_major_' : '');

                                    for (var i = 0; i < shaderTypes.length; i++) {
                                        name = nameWithMatrixType + es3fUniformApiTests.getCaseShaderTypeName(es3fUniformApiTests.CaseShaderType[shaderTypes[i]]);

                                        booleanTypeFeat.ARRAY_FIRST_ELEM_NAME_NO_INDEX = referToFirstArrayElemWithoutIndexI != 0;
                                        booleanTypeFeat.MATRIXMODE_ROWMAJOR = matrixTypeI == 1;

                                        collectionTestGroup.addChild(new es3fUniformApiTests.UniformValueCase(name, '', es3fUniformApiTests.CaseShaderType[shaderTypes[i]], uniformCollection,
                                                                                            es3fUniformApiTests.ValueToCheck.ASSIGNED, checkMethod, assignMethod, booleanTypeFeat));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cases assign multiple basic-array elements with one glUniform*v() (i.e. the count parameter is bigger than 1).

        /** @type {es3fUniformApiTests.Feature} */ var arrayAssignFullMode = new es3fUniformApiTests.Feature();
        arrayAssignFullMode.ARRAYASSIGN_FULL = true;

        /** @type {es3fUniformApiTests.Feature} */ var arrayAssignBlocksOfTwo = new es3fUniformApiTests.Feature();
        arrayAssignFullMode.ARRAYASSIGN_BLOCKS_OF_TWO = true;

        var arrayAssignGroups =
        [{arrayAssignMode: arrayAssignFullMode, name: 'basic_array_assign_full', description: 'Assign entire basic-type arrays per glUniform*v() call'}, {arrayAssignMode: arrayAssignBlocksOfTwo, name: 'basic_array_assign_partial', description: 'Assign two elements of a basic-type array per glUniform*v() call'}
        ];

        for (var arrayAssignGroupNdx = 0; arrayAssignGroupNdx < arrayAssignGroups.length; arrayAssignGroupNdx++) {
            /** @type {es3fUniformApiTests.Feature} */ var arrayAssignMode = arrayAssignGroups[arrayAssignGroupNdx].arrayAssignMode;
            /** @type {string} */ var groupName = arrayAssignGroups[arrayAssignGroupNdx].name;
            /** @type {string} */ var groupDesc = arrayAssignGroups[arrayAssignGroupNdx].description;

            /** @type {tcuTestCase.DeqpTest} */ var curArrayAssignGroup = tcuTestCase.newTest(groupName, groupDesc);
            assignedValuesGroup.addChild(curArrayAssignGroup);

            /** @type {Array<number>} */ var basicArrayCollectionGroups = [UniformCollections.BASIC_ARRAY, UniformCollections.ARRAY_IN_STRUCT, UniformCollections.MULTIPLE_BASIC_ARRAY];

            for (var collectionGroupNdx = 0; collectionGroupNdx < basicArrayCollectionGroups.length; collectionGroupNdx++) {
                collectionGroup = defaultUniformCollections[basicArrayCollectionGroups[collectionGroupNdx]];
                collectionTestGroup = tcuTestCase.newTest(collectionGroup.name, '');
                curArrayAssignGroup.addChild(collectionTestGroup);

                for (var collectionNdx = 0; collectionNdx < collectionGroup.cases.length; collectionNdx++) {
                    collectionCase = collectionGroup.cases[collectionNdx];
                    collName = collectionCase.namePrefix;
                    uniformCollection = collectionCase.uniformCollection;

                    for (var i = 0; i < shaderTypes.length; i++) {
                        name = collName + es3fUniformApiTests.getCaseShaderTypeName(es3fUniformApiTests.CaseShaderType[shaderTypes[i]]);
                        collectionTestGroup.addChild(new es3fUniformApiTests.UniformValueCase(name, '', es3fUniformApiTests.CaseShaderType[shaderTypes[i]], uniformCollection,
                                                                            es3fUniformApiTests.ValueToCheck.ASSIGNED, es3fUniformApiTests.CheckMethod.GET_UNIFORM, es3fUniformApiTests.AssignMethod.POINTER,
                                                                            arrayAssignMode));
                    }
                }
            }
        }

        // Value checking cases when unused uniforms are present.

        unusedUniformsGroup = tcuTestCase.newTest('unused_uniforms', 'Test with unused uniforms');
        assignedValuesGroup.addChild(unusedUniformsGroup);

        collectionGroup = defaultUniformCollections[UniformCollections.ARRAY_IN_STRUCT];

        for (var collectionNdx = 0; collectionNdx < collectionGroup.cases.length; collectionNdx++) {
            collectionCase = collectionGroup.cases[collectionNdx];
            collName = collectionCase.namePrefix;
            uniformCollection = collectionCase.uniformCollection;

            for (var i = 0; i < shaderTypes.length; i++) {
                name = collName + es3fUniformApiTests.getCaseShaderTypeName(es3fUniformApiTests.CaseShaderType[shaderTypes[i]]);

                features = new es3fUniformApiTests.Feature();
                features.ARRAYUSAGE_ONLY_MIDDLE_INDEX = true;
                features.UNIFORMUSAGE_EVERY_OTHER = true;

                unusedUniformsGroup.addChild(new es3fUniformApiTests.UniformValueCase(name, '', es3fUniformApiTests.CaseShaderType[shaderTypes[i]], uniformCollection,
                                                                    es3fUniformApiTests.ValueToCheck.ASSIGNED, es3fUniformApiTests.CheckMethod.GET_UNIFORM, es3fUniformApiTests.AssignMethod.POINTER,
                                                                    features));
            }
        }

        // Random cases.

        /** @type {number} */ var numRandomCases = 100;
        /** @type {tcuTestCase.DeqpTest} */ var randomGroup = tcuTestCase.newTest('random', 'Random cases');
        testGroup.addChild(randomGroup);

        for (var ndx = 0; ndx < numRandomCases; ndx++)
            randomGroup.addChild(new es3fUniformApiTests.RandomUniformCase('' + ndx, '', ndx));
    };

    /**
     * Create and execute the test cases
     * @param {WebGL2RenderingContext} context
     */
    es3fUniformApiTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'uniform_api';
        var testDescription = 'es3fUniformApiTests.Uniform API Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fUniformApiTests.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fUniformApiTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
