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
goog.provide('framework.opengl.gluVarType');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {

    var gluVarType = framework.opengl.gluVarType;
    var gluShaderUtil = framework.opengl.gluShaderUtil;

    /**
    * gluVarType.VarType types enum
    * @enum {number}
    */
    gluVarType.Type = {
       TYPE_BASIC: 0,
       TYPE_ARRAY: 1,
       TYPE_STRUCT: 2
    };

    /**
    * gluVarType.TypeArray struct
    * @param {gluVarType.VarType} elementType
    * @param {number} arraySize
    * @constructor
    */
    gluVarType.TypeArray = function(elementType, arraySize) {
       /** @type {gluVarType.VarType} */ this.elementType = gluVarType.newClone(elementType);
       /** @type {number} */ this.size = arraySize;
    };

    /**
     * gluVarType.VarType class
     * @constructor
     */
    gluVarType.VarType = function() {
        /**
         * @type {gluShaderUtil.precision}
         * @private
         */
        this.m_flags;

        /**
         * @type {number}
         * @private
         */
        this.m_type = -1;

        /**
         * m_data used to be a 'Data' union in C++. Using a var is enough here.
         * it will contain any necessary value.
         * case TYPE_BASIC: number
         * case TYPE_ARRAY: gluVarType.TypeArray
         * case TYPE_STRUCT: gluVarType.StructType
         * @private
         * @type {(number|gluVarType.TypeArray|gluVarType.StructType)}
        */
        this.m_data = null;
    };

    gluVarType.VarType.UNSIZED_ARRAY = -1;

    /**
     * Creates a basic type gluVarType.VarType. Use this after the constructor call.
     * @param {number} basicType
     * @param {gluShaderUtil.precision} flags
     * @return {gluVarType.VarType} The currently modified object
     */
    gluVarType.VarType.prototype.VarTypeBasic = function(basicType, flags) {
        this.m_type = gluVarType.Type.TYPE_BASIC;
        this.m_flags = flags;
        this.m_data = basicType;

        return this;
    };

    /**
     * Creates a basic type gluVarType.VarType with type boolean and undefined precision.
     * @param {number} basicType
     * @return {gluVarType.VarType} The currently modified object
     */
    gluVarType.VarType.prototype.VarTypeBoolean = function(basicType) {
        this.m_type = gluVarType.Type.TYPE_BASIC;
        this.m_data = basicType;

        return this;
    };

    /**
     * Creates an array type gluVarType.VarType. Use this after the constructor call.
     * @param {gluVarType.VarType} elementType
     * @param {number} arraySize
     * @return {gluVarType.VarType} The currently modified object
     */
    gluVarType.VarType.prototype.VarTypeArray = function(elementType, arraySize) {
        this.m_type = gluVarType.Type.TYPE_ARRAY;
        if (!(arraySize >= 0 || arraySize == gluVarType.VarType.UNSIZED_ARRAY))
            throw new Error('Illegal array size: ' + arraySize);
        this.m_data = new gluVarType.TypeArray(elementType, arraySize);

        return this;
    };

    /**
     * Creates a struct type gluVarType.VarType. Use this after the constructor call.
     * @param {gluVarType.StructType} structPtr
     * @return {gluVarType.VarType} The currently modified object
     */
    gluVarType.VarType.prototype.VarTypeStruct = function(structPtr) {
        this.m_type = gluVarType.Type.TYPE_STRUCT;
        this.m_data = structPtr;

        return this;
    };

    /**
     * Creates a gluVarType.VarType, the same type as the passed in object.
     * Use this after the constructor call.
     * @param {gluVarType.VarType} object
     * @return {gluVarType.VarType} The currently modified object
     */
    gluVarType.VarType.prototype.VarTypeClone = function(object) {

        this.m_type = object.m_type;

        switch (this.m_type) {
          case gluVarType.Type.TYPE_BASIC:
            this.m_flags = object.m_flags;
            this.m_data = object.m_data;
            break;
          case gluVarType.Type.TYPE_BASIC:
            this.m_data = new gluVarType.TypeArray(object.m_data.elementType, object.m_data.size);
            break;
          case gluVarType.Type.TYPE_STRUCT:
            this.m_data = object.m_data;
            break;
          default:
            throw new Error('unknown type: ' + this.m_type);
        }

        return this;
    };

    /** isBasicType
     * @return {boolean} true if the gluVarType.VarType represents a basic type.
     */
    gluVarType.VarType.prototype.isBasicType = function() {
        return this.m_type == gluVarType.Type.TYPE_BASIC;
    };

    /** isArrayType
     * @return {boolean} true if the gluVarType.VarType represents an array.
     */
    gluVarType.VarType.prototype.isArrayType = function() {
        return this.m_type == gluVarType.Type.TYPE_ARRAY;
    };

    /** isStructType
     * @return {boolean} true if the gluVarType.VarType represents a struct.
     */
    gluVarType.VarType.prototype.isStructType = function() {
        return this.m_type == gluVarType.Type.TYPE_STRUCT;
    };

    /** getFlags
     * @return {number} returns the flags of the gluVarType.VarType.
     */
    gluVarType.VarType.prototype.getFlags = function() {
        return this.m_flags;
    };

    /** getBasicType
     * @return {gluShaderUtil.DataType<number>} returns the basic data type of the gluVarType.VarType.
     */
    gluVarType.VarType.prototype.getBasicType = function() {
        if (!this.isBasicType())
            throw new Error('VarType is not a basic type.');
        return /** @type {gluShaderUtil.DataType<number>} */ (this.m_data);
    };

    /** getPrecision
     * @return {gluShaderUtil.precision} returns the precision flag.
     */
    gluVarType.VarType.prototype.getPrecision = function() {
        if (!this.isBasicType())
            throw new Error('VarType is not a basic type.');
        return this.m_flags;
     };

    /** getElementType
     * @return {gluVarType.VarType} returns the gluVarType.VarType of the element in case of an Array.
     */
    gluVarType.VarType.prototype.getElementType = function() {
        if (!this.isArrayType())
            throw new Error('VarType is not an array type.');
        return this.m_data.elementType;
    };

    /** getArraySize
     * (not to be confused with a javascript array)
     * @return {number} returns the size of the array in case it is an array.
     */
    gluVarType.VarType.prototype.getArraySize = function() {
        if (!this.isArrayType())
            throw new Error('VarType is not an array type.');
        return this.m_data.size;
    };

    /** getStruct
     * @return {gluVarType.StructType} returns the structure when it is a gluVarType.StructType.
     */
    gluVarType.VarType.prototype.getStruct = function() {
        if (!this.isStructType())
            throw new Error('VarType is not a struct type.');
        return /** @type {gluVarType.StructType} */ (this.m_data);
    };

    /**
     * getScalarSize
     * @return {number} size of the scalar
     */
    gluVarType.VarType.prototype.getScalarSize = function() {
        switch (this.m_type) {
            case gluVarType.Type.TYPE_BASIC: {
                return gluShaderUtil.getDataTypeScalarSize(/** @type {gluShaderUtil.DataType} */(this.getBasicType()));
            }

            // TODO: check implementation below: return m_data.array.elementType->getScalarSize()*m_data.array.size;
            case gluVarType.Type.TYPE_ARRAY: {
                var m_data = /** @type {gluVarType.TypeArray} */(this.m_data);
                return m_data.elementType.getScalarSize() * m_data.size;
            }

            case gluVarType.Type.TYPE_STRUCT: {
                var size = 0;

                var struct = /** @type {gluVarType.StructType} */ (this.m_data);

                // TODO: check loop conditions below
                // for (gluVarType.StructType::ConstIterator iter = m_data.structPtr->begin(); iter != m_data.structPtr->end(); iter++)
                for (var iter = 0; struct.m_members[iter] < struct.getSize(); iter++)
                    size += struct.getMember(iter).m_type.getScalarSize();
                return size;
            }

            default:
                // throw new Error('Unexpected type.');
                return 0;
        }
    };

    /**
    * is
    * @return {boolean} returns true if the current object is equivalent to other.
    */
    gluVarType.VarType.prototype.is = function(other) {
        if (this.m_type != other.m_type)
            return false;

        switch (this.m_type) {
            case gluVarType.Type.TYPE_BASIC:
                return this.m_data == other.m_data &&
                       this.m_flags == other.m_flags;

            case gluVarType.Type.TYPE_ARRAY:
                return this.m_data.elementType == other.m_data.elementType &&
                       this.m_data.size == other.m_data.size;

            case gluVarType.Type.TYPE_STRUCT:
                return this.m_data === other.m_data;

            default:
                // throw new Error('Unexpected type.');
                return false;
        }
    };

    /**
    * isnt
    * @return {boolean} returns true if the current object is not equivalent to other.
    */
    gluVarType.VarType.prototype.isnt = function(other) {
        return !(this.is(other));
    };

    /**
     * Creates a basic type gluVarType.VarType.
     * @param {gluShaderUtil.DataType} basicType
     * @param {framework.opengl.gluShaderUtil.precision=} flags
     * @return {gluVarType.VarType}
     */
    gluVarType.newTypeBasic = function(basicType, flags) {
        if (!gluShaderUtil.isDataTypeBoolOrBVec(basicType))
            return new gluVarType.VarType().VarTypeBasic(basicType, /** @type {framework.opengl.gluShaderUtil.precision}*/ (flags));
        else
            return new gluVarType.VarType().VarTypeBoolean(basicType);
    };

    /**
    * Creates an array type gluVarType.VarType.
    * @param {gluVarType.VarType} elementType
    * @param {number} arraySize
    * @return {gluVarType.VarType}
    */
    gluVarType.newTypeArray = function(elementType, arraySize) {
       return new gluVarType.VarType().VarTypeArray(elementType, arraySize);
    };

    /**
    * Creates a struct type gluVarType.VarType.
    * @param {gluVarType.StructType} structPtr
    * @return {gluVarType.VarType}
    */
    gluVarType.newTypeStruct = function(structPtr) {
        return new gluVarType.VarType().VarTypeStruct(structPtr);
    };

    /**
    * Creates a struct type gluVarType.VarType.
    * @param {gluVarType.VarType} object
    * @return {gluVarType.VarType}
    */
    gluVarType.newClone = function(object) {
        return new gluVarType.VarType().VarTypeClone(object);
    };

    /**
     * gluVarType.StructMember class
     * @constructor
     */
    gluVarType.StructMember = function() {
       /** @type {string} */ this.m_name;
       /** @type {gluVarType.VarType} */ this.m_type;
       /** @type {number} */ // this.m_flags = 0; // only in glsUniformBlockCase
    };

    /**
     * Creates a gluVarType.StructMember. Use this after the constructor call.
     * @param {string} name
     * @param {gluVarType.VarType} type
     * @return {gluVarType.StructMember} The currently modified object
     */
    gluVarType.StructMember.prototype.Constructor = function(name, type) {
        this.m_type = type;
        this.m_name = name;

        return this;
    };

    /** getName
     * @return {string} name of the gluVarType.StructMember object.
     */
    gluVarType.StructMember.prototype.getName = function() {
        return this.m_name;
    };

    /** getType
     * @return {gluVarType.VarType} type of the gluVarType.StructMember object.
     */
    gluVarType.StructMember.prototype.getType = function() {
        return this.m_type;
    };

    /**
     * Creates a gluVarType.StructMember.
     * @param {string} name
     * @param {gluVarType.VarType} type
     * @return {gluVarType.StructMember}
     */
    gluVarType.newStructMember = function(name, type) {
        return new gluVarType.StructMember().Constructor(name, type);
    };

    /**
     * gluVarType.StructType class
     * @constructor
     */
    gluVarType.StructType = function() {
        /** @type {string} */ this.m_typeName = '';
        /** @type {Array<gluVarType.StructMember>} */ this.m_members = [];
    };

    /**
     * Creates a gluVarType.StructType. Use this after the constructor call.
     * @param {string} name
     * @return {gluVarType.StructType} The currently modified object
     */
    gluVarType.StructType.prototype.Constructor = function(name) {
        /** @type {string}*/ this.m_typeName = this.setTypeName(name);
        return this;
    };

    /** hasTypeName
     * Checks if the gluVarType.StructType m_typeName is defined
     * @return {boolean}
     */
    gluVarType.StructType.prototype.hasTypeName = function() {
        return (this.m_typeName !== 'undefined');
    };

    /** setTypeName
     * @param {string} name
     * @return {string} returns gluVarType.StructType.m_typeName
     */
    gluVarType.StructType.prototype.setTypeName = function(name) {
        return this.m_typeName = name;
    };

    /** getTypeName
     * @return {string}
     */
    gluVarType.StructType.prototype.getTypeName = function() {
        return this.m_typeName;
    };

    /** getNumMembers
     * @return {number}
     */
    gluVarType.StructType.prototype.getNumMembers = function() {
        return this.m_members.length;
    };

    /** getMember
     * @param {number} memberNdx The index of the member to retrieve.
     * @return {gluVarType.StructMember}
     */
    gluVarType.StructType.prototype.getMember = function(memberNdx) {
        if (memberNdx >= 0 && memberNdx < this.m_members.length)
            return this.m_members[memberNdx];
        else {
            throw new Error('Invalid member index for StructTypes members');
        }
    };

    /** getSize
     * @return {number} The size of the m_members array.
     */
    gluVarType.StructType.prototype.getSize = function() {
        return this.m_members.length;
    };

    /** addMember
     * @param {string} name
     * @param {gluVarType.VarType} type
     */
    gluVarType.StructType.prototype.addMember = function(name, type) {
        var member = gluVarType.newStructMember(name, type);
        this.m_members.push(member);
    };

    /**
     * Creates a gluVarType.StructType.
     * @param {string} name
     * @return {gluVarType.StructType}
     */
    gluVarType.newStructType = function(name) {
        return new gluVarType.StructType().Constructor(name);
    };

    /**
     * @param {number} level
     * @return {string}
     */
    gluVarType.indent = function(level) {
        /** @type {string} */ var str = '';
        for (var i = 0; i < level; i++)
            str += '\t';
        return str;
    };

    /**
     * @param {gluVarType.VarType} varType
     * @param {string} name
     * @param {number=} level
     * @return {string}
     */
    gluVarType.declareVariable = function(varType, name, level) {
        /** @type {string} */ var str = '';
        /** @type {gluVarType.VarType} */ var type = varType;
        /** @type {gluVarType.VarType} */ var curType = type;
        /** @type {Array<number>} */ var arraySizes = [];

        // Handle arrays.
        while (curType.isArrayType()) {
            arraySizes.push(curType.getArraySize());
            curType = curType.getElementType();
        }

        if (curType.isBasicType()) {
            if (curType.getPrecision() !== undefined)
                str += gluShaderUtil.getPrecisionName(curType.getPrecision()) + ' ';
            str += gluShaderUtil.getDataTypeName(/** @type {gluShaderUtil.DataType} */(curType.getBasicType()));
        } else if (curType.isStructType()) {
            /** @type {gluVarType.StructType} */ var structPtr = curType.getStruct();

            if (structPtr.hasTypeName())
                str += structPtr.getTypeName();
            else
                str += gluVarType.declareStructType(structPtr, level); // Generate inline declaration.
        } else
            throw new Error('Unexpected Array typed VarType.');

        str += ' ' + name;

        // Print array sizes.
        for (var size = 0; size < arraySizes.length; size++) { //std::vector<int>::const_iterator sizeIter = arraySizes.begin(); sizeIter != arraySizes.end(); sizeIter++) {
            /** @type {number} */ var arrSize = arraySizes[size];
            if (arrSize == gluVarType.VarType.UNSIZED_ARRAY)
                str += '[]';
            else
                str += '[' + arrSize + ']';
        }

        return str;
    };

    /**
     * @param {gluVarType.StructType} structType
     * @param {number=} level
     * @return {string}
     */
    gluVarType.declareStructType = function(structType, level) {
        /** @type {string} */ var str = 'struct';
        level = level || 0;

        // gluVarType.Type name is optional.
        if (structType.hasTypeName())
            str += ' ' + structType.getTypeName();

        str += '\n' + gluVarType.indent(level) + ' {\n';

        for (var memberNdx = 0; memberNdx < structType.getSize(); memberNdx++) { //gluVarType.StructType::ConstIterator memberIter = decl.structPtr->begin(); memberIter != decl.structPtr->end(); memberIter++) {
            /** @type {gluVarType.StructMember} */ var memberIter = structType.getMember(memberNdx);
            str += gluVarType.indent(level + 1);
            str += gluVarType.declareVariable(memberIter.getType(), memberIter.getName(), level + 1) + ';\n';
        }

        str += gluVarType.indent(level) + '}';

        return str;
    };

    /**
     * @param {*} T
     * @param {number=} size
     * @param {gluShaderUtil.precision=} precision
     * @return {gluVarType.VarType}
     */
    gluVarType.getVarTypeOf = function(T, size, precision) {
        size = size || 1;
        precision = precision || gluShaderUtil.precision.PRECISION_LOWP;
        switch (size) {
            case 4: return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, precision);
            case 3: return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC3, precision);
            case 2: return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC2, precision);
        }
        switch (T) {
            case 'float' : return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT, precision);
            case 'vec4': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, precision);
            case 'vec3': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC3, precision);
            case 'vec2': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC2, precision);
            case 'mat2': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT2, precision);
            case 'mat2x3': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT2X3, precision);
            case 'mat2x4': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT2X4, precision);
            case 'mat3x2': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3X2, precision);
            case 'mat3': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3, precision);
            case 'mat3x4': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT3X4, precision);
            case 'mat4x2': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT4X2, precision);
            case 'mat4x3': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT4X3, precision);
            case 'mat4': return gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_MAT4, precision);
        }
        throw new Error('Invalid input type ' + T + ' or size ' + size);
    };

    /**
     * @enum
     */
    gluVarType.Storage = {
            STORAGE_IN: 0,
            STORAGE_OUT: 1,
            STORAGE_CONST: 2,
            STORAGE_UNIFORM: 3,
            STORAGE_BUFFER: 4
    };

    /**
     * @param {gluVarType.Storage} storage
     * @return {string}
     */
    gluVarType.getStorageName = function(storage) {
        switch (storage) {
            case gluVarType.Storage.STORAGE_IN: return 'in';
            case gluVarType.Storage.STORAGE_OUT: return 'out';
            case gluVarType.Storage.STORAGE_CONST: return 'const';
            case gluVarType.Storage.STORAGE_UNIFORM: return 'uniform';
            case gluVarType.Storage.STORAGE_BUFFER: return 'buffer';
            default:
                throw new Error('Unknown storage: ' + storage);
        }
    };

    /**
     * @enum
     */
    gluVarType.Interpolation = {
        INTERPOLATION_SMOOTH: 0,
        INTERPOLATION_FLAT: 1,
        INTERPOLATION_CENTROID: 2
    };

    /**
     * @param {gluVarType.Interpolation} interpolation
     * @return {string}
     */
    gluVarType.getInterpolationName = function(interpolation) {
        switch (interpolation) {
            case gluVarType.Interpolation.INTERPOLATION_SMOOTH: return 'smooth';
            case gluVarType.Interpolation.INTERPOLATION_FLAT: return 'flat';
            case gluVarType.Interpolation.INTERPOLATION_CENTROID: return 'centrod';
            default:
                throw new Error('Unknown interpolation: ' + interpolation);
        }
    };

    /**
     * @enum
     */
    gluVarType.FormatLayout = {
        FORMATLAYOUT_RGBA32F: 0,
        FORMATLAYOUT_RGBA16F: 1,
        FORMATLAYOUT_R32F: 2,
        FORMATLAYOUT_RGBA8: 3,
        FORMATLAYOUT_RGBA8_SNORM: 4,

        FORMATLAYOUT_RGBA32I: 5,
        FORMATLAYOUT_RGBA16I: 6,
        FORMATLAYOUT_RGBA8I: 7,
        FORMATLAYOUT_R32I: 8,

        FORMATLAYOUT_RGBA32UI: 9,
        FORMATLAYOUT_RGBA16UI: 10,
        FORMATLAYOUT_RGBA8UI: 11,
        FORMATLAYOUT_R32UI: 12
    };

    /**
     * @param {gluVarType.FormatLayout} layout
     * @return {string}
     */
    gluVarType.getFormatLayoutName = function(layout) {
        switch (layout) {
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA32F: return 'rgba32f';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA16F: return 'rgba16f';
            case gluVarType.FormatLayout.FORMATLAYOUT_R32F: return 'r32f';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA8: return 'rgba8';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA8_SNORM: return 'rgba8_snorm';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA32I: return 'rgba32i';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA16I: return 'rgba16i';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA8I: return 'rgba8i';
            case gluVarType.FormatLayout.FORMATLAYOUT_R32I: return 'r32i';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA32UI: return 'rgba32ui';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA16UI: return 'rgba16ui';
            case gluVarType.FormatLayout.FORMATLAYOUT_RGBA8UI: return 'rgba8ui';
            case gluVarType.FormatLayout.FORMATLAYOUT_R32UI: return 'r32ui';
            default:
                throw new Error('Unknown layout: ' + layout);
        }
    };

    /**
     * @enum
     */
    gluVarType.MatrixOrder = {
        MATRIXORDER_COLUMN_MAJOR: 0,
        MATRIXORDER_ROW_MAJOR: 1
    };

   /**
     * @param {gluVarType.MatrixOrder} qualifier
     * @return {string}
     */
    gluVarType.getMatrixOrderName = function(qualifier) {
        switch (qualifier) {
            case gluVarType.MatrixOrder.MATRIXORDER_COLUMN_MAJOR: return 'column_major';
            case gluVarType.MatrixOrder.MATRIXORDER_ROW_MAJOR: return 'row_major';
            default:
                throw new Error('Unknown qualifier: ' + qualifier);
        }
    };

    gluVarType.MemoryAccessQualifier = {
        MEMORYACCESSQUALIFIER_COHERENT_BIT: 0x01,
        MEMORYACCESSQUALIFIER_VOLATILE_BIT: 0x02,
        MEMORYACCESSQUALIFIER_RESTRICT_BIT: 0x04,
        MEMORYACCESSQUALIFIER_READONLY_BIT: 0x08,
        MEMORYACCESSQUALIFIER_WRITEONLY_BIT: 0x10
    };
    gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_MASK = (gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_WRITEONLY_BIT << 1) - 1;

    /**
     * @param {number} qualifier
     * @return {string}
     */
    gluVarType.getMemoryAccessQualifierName = function(qualifier) {
        switch (qualifier) {
            case gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_COHERENT_BIT: return 'coherent';
            case gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_VOLATILE_BIT: return 'volatile';
            case gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_RESTRICT_BIT: return 'restrict';
            case gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_READONLY_BIT: return 'readonly';
            case gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_WRITEONLY_BIT: return 'writeonly';
            default:
                throw new Error('Unknown qualifier: ' + qualifier);
        }
    };

    /**
     * @constructor
     * @param {number=} location
     * @param {number=} binding
     * @param {number=} offset
     * @param {gluVarType.FormatLayout=} format
     * @param {gluVarType.MatrixOrder=} matrixOrder
     */
    gluVarType.Layout = function(location, binding, offset, format, matrixOrder) {
        this.location = location;
        this.binding = binding;
        this.offset = offset;
        this.format = format;
        this.matrixOrder = matrixOrder;
    };

    gluVarType.Layout.prototype.toString = function() {
        var strings = [];
        var str = '';
        if (typeof this.location !== 'undefined')
            strings.push('location=' + this.location);
        if (typeof this.binding !== 'undefined')
            strings.push('binding=' + this.binding);
        if (typeof this.offset !== 'undefined')
            strings.push('offset=' + this.offset);
        if (typeof this.format !== 'undefined')
            strings.push(gluVarType.getFormatLayoutName(this.format));
        if (typeof this.matrixOrder !== 'undefined')
            strings.push(gluVarType.getMatrixOrderName(this.matrixOrder));

        if (strings.length > 0) {
            str += 'layout(' + strings[0];

            for (var i = 1; i < strings.length; i++)
                str += ', ' + strings[i];
            str += ')';
        }

        return str;
    };

    /**
     * @constructor
     * @param {gluVarType.VarType} varType
     * @param {string} name
     * @param {gluVarType.Storage=} storage
     * @param {gluVarType.Interpolation=} interpolation
     * @param {gluVarType.Layout=} layout
     * @param {number=} memoryAccessQualifierBits
     */
    gluVarType.VariableDeclaration = function(varType, name, storage, interpolation, layout, memoryAccessQualifierBits) {
        this.varType = varType;
        this.name = name;
        this.storage = storage;
        this.interpolation = interpolation;
        this.layout = layout;
        this.memoryAccessQualifierBits = memoryAccessQualifierBits || 0;
    };

    gluVarType.VariableDeclaration.prototype.toString = function() {
        var str = '';
        if (typeof this.layout !== 'undefined')
            str += this.layout.toString() + ' ';

        for (var bitNdx = 0; (1 << bitNdx) & gluVarType.MemoryAccessQualifier.MEMORYACCESSQUALIFIER_MASK; ++bitNdx)
            if (this.memoryAccessQualifierBits & (1 << bitNdx))
                str += gluVarType.getMemoryAccessQualifierName((1 << bitNdx)) + ' ';

        if (typeof this.interpolation !== 'undefined')
            str += gluVarType.getInterpolationName(this.interpolation) + ' ';

        if (typeof this.storage !== 'undefined')
            str += gluVarType.getStorageName(this.storage) + ' ';

        str += gluVarType.declareVariable(this.varType, this.name);

        return str;
    };

});
