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
goog.provide('modules.shared.glsUniformBlockCase');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {

var glsUniformBlockCase = modules.shared.glsUniformBlockCase;
var tcuTestCase = framework.common.tcuTestCase;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var gluDrawUtil = framework.opengl.gluDrawUtil;
var deUtil = framework.delibs.debase.deUtil;
var deMath = framework.delibs.debase.deMath;
var deRandom = framework.delibs.debase.deRandom;
var deString = framework.delibs.debase.deString;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

var littleEndian = (function() {
    var buffer = new ArrayBuffer(2);
    new DataView(buffer).setInt16(0, 256, true /* littleEndian */);
    // Int16Array uses the platform's endianness.
    return new Int16Array(buffer)[0] === 256;
})();

/**
 * Class to implement some pointers functionality.
 * @constructor
 */
glsUniformBlockCase.BlockPointers = function() {
    /** @type {ArrayBuffer} */ this.data; //!< Data (vector<deUint8>).
    /** @type {Array<number>} */ this.offsets = []; //!< Reference block pointers (map<int, void*>).
    /** @type {Array<number>} */ this.sizes = [];
};

/**
 * push - Adds an offset/size pair to the collection
 * @param {number} offset Offset of the element to refer.
 * @param {number} size Size of the referred element.
 */
glsUniformBlockCase.BlockPointers.prototype.push = function(offset, size) {
    this.offsets.push(offset);
    this.sizes.push(size);
};

/**
 * find - Finds and maps the data at the given offset, and returns a Uint8Array
 * @param {number} index of the element to find.
 * @return {Uint8Array}
 */
glsUniformBlockCase.BlockPointers.prototype.find = function(index) {
    return new Uint8Array(this.data, this.offsets[index], this.sizes[index]);
};

/**
 * resize - Replaces resize of a vector in C++. Sets the size of the data buffer.
 * NOTE: In this case however, if you resize, the data is lost.
 * @param {number} newsize The new size of the data buffer.
 */
glsUniformBlockCase.BlockPointers.prototype.resize = function(newsize) {
    this.data = new ArrayBuffer(newsize);
};

/**
 * glsUniformBlockCase.isSupportedGLSLVersion
 * @param {gluShaderUtil.GLSLVersion} version
 * @return {boolean}
 */
glsUniformBlockCase.isSupportedGLSLVersion = function(version) {
    return version >= gluShaderUtil.GLSLVersion.V300_ES;
};

/**
 * @enum {number}
 */
glsUniformBlockCase.UniformFlags = {
    PRECISION_LOW: (1 << 0),
    PRECISION_MEDIUM: (1 << 1),
    PRECISION_HIGH: (1 << 2),

    LAYOUT_SHARED: (1 << 3),
    LAYOUT_PACKED: (1 << 4),
    LAYOUT_STD140: (1 << 5),
    LAYOUT_ROW_MAJOR: (1 << 6),
    LAYOUT_COLUMN_MAJOR: (1 << 7), //!< \note Lack of both flags means column-major matrix.

    DECLARE_VERTEX: (1 << 8),
    DECLARE_FRAGMENT: (1 << 9),

    UNUSED_VERTEX: (1 << 10), //!< glsUniformBlockCase.Uniform or struct member is not read in vertex shader.
    UNUSED_FRAGMENT: (1 << 11) //!< glsUniformBlockCase.Uniform or struct member is not read in fragment shader.
};

/** @const */ glsUniformBlockCase.UniformFlags.PRECISION_MASK = glsUniformBlockCase.UniformFlags.PRECISION_LOW | glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM | glsUniformBlockCase.UniformFlags.PRECISION_HIGH;
/** @const */ glsUniformBlockCase.UniformFlags.LAYOUT_MASK = glsUniformBlockCase.UniformFlags.LAYOUT_SHARED | glsUniformBlockCase.UniformFlags.LAYOUT_PACKED | glsUniformBlockCase.UniformFlags.LAYOUT_STD140 | glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR | glsUniformBlockCase.UniformFlags.LAYOUT_COLUMN_MAJOR;
/** @const */ glsUniformBlockCase.UniformFlags.DECLARE_BOTH = glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT;
/** @const */ glsUniformBlockCase.UniformFlags.UNUSED_BOTH = glsUniformBlockCase.UniformFlags.UNUSED_VERTEX | glsUniformBlockCase.UniformFlags.UNUSED_FRAGMENT;

/**
* glsUniformBlockCase.VarType types enum
* @enum {number}
*/
glsUniformBlockCase.Type = {
    TYPE_BASIC: 0,
    TYPE_ARRAY: 1,
    TYPE_STRUCT: 2
};

glsUniformBlockCase.Type.TYPE_LAST = Object.keys(glsUniformBlockCase.Type).length;

/**
* glsUniformBlockCase.TypeArray struct (nothing to do with JS's TypedArrays)
* @param {glsUniformBlockCase.VarType} elementType
* @param {number} arraySize
* @constructor
*/
glsUniformBlockCase.TypeArray = function(elementType, arraySize) {
    /** @type {glsUniformBlockCase.VarType} */ this.elementType = elementType;
    /** @type {number} */ this.size = arraySize;
};

/**
 * glsUniformBlockCase.VarType class
 * @constructor
 */
glsUniformBlockCase.VarType = function() {
    /** @type {glsUniformBlockCase.Type} */ this.m_type;
    /** @type {number} */ this.m_flags = 0;

    /*
     * m_data used to be a 'Data' union in C++. Using a var is enough here.
     * it will contain any necessary value.
     */

    /** @type {(gluShaderUtil.DataType|glsUniformBlockCase.TypeArray|glsUniformBlockCase.StructType)} */
    this.m_data;
};

/**
* Creates a basic type glsUniformBlockCase.VarType. Use this after the constructor call.
* @param {gluShaderUtil.DataType} basicType
* @param {number} flags
* @return {glsUniformBlockCase.VarType} The currently modified object
*/
glsUniformBlockCase.VarType.prototype.VarTypeBasic = function(basicType, flags) {
    this.m_type = glsUniformBlockCase.Type.TYPE_BASIC;
    this.m_flags = flags;
    this.m_data = basicType;

    return this;
};

/**
* Creates an array type glsUniformBlockCase.VarType. Use this after the constructor call.
* @param {glsUniformBlockCase.VarType} elementType
* @param {number} arraySize
* @return {glsUniformBlockCase.VarType} The currently modified object
*/
glsUniformBlockCase.VarType.prototype.VarTypeArray = function(elementType, arraySize) {
    this.m_type = glsUniformBlockCase.Type.TYPE_ARRAY;
    this.m_flags = 0;
    this.m_data = new glsUniformBlockCase.TypeArray(elementType, arraySize);

    return this;
};

/**
* Creates a struct type glsUniformBlockCase.VarType. Use this after the constructor call.
* @param {glsUniformBlockCase.StructType} structPtr
* @return {glsUniformBlockCase.VarType} The currently modified object
*/
glsUniformBlockCase.VarType.prototype.VarTypeStruct = function(structPtr) {
    this.m_type = glsUniformBlockCase.Type.TYPE_STRUCT;
    this.m_flags = 0;
    this.m_data = structPtr;

    return this;
};

/** isBasicType
* @return {boolean} true if the glsUniformBlockCase.VarType represents a basic type.
**/
glsUniformBlockCase.VarType.prototype.isBasicType = function() {
    return this.m_type == glsUniformBlockCase.Type.TYPE_BASIC;
};

/** isArrayType
* @return {boolean} true if the glsUniformBlockCase.VarType represents an array.
**/
glsUniformBlockCase.VarType.prototype.isArrayType = function() {
    return this.m_type == glsUniformBlockCase.Type.TYPE_ARRAY;
};

/** isStructType
* @return {boolean} true if the glsUniformBlockCase.VarType represents a struct.
**/
glsUniformBlockCase.VarType.prototype.isStructType = function() {
    return this.m_type == glsUniformBlockCase.Type.TYPE_STRUCT;
};

/** getFlags
* @return {number} returns the flags of the glsUniformBlockCase.VarType.
**/
glsUniformBlockCase.VarType.prototype.getFlags = function() {
    return this.m_flags;
};

/** getBasicType
* @return {gluShaderUtil.DataType} returns the basic data type of the glsUniformBlockCase.VarType.
**/
glsUniformBlockCase.VarType.prototype.getBasicType = function() {
    return /** @type {gluShaderUtil.DataType} */ (this.m_data);
};

/** getElementType
* @return {glsUniformBlockCase.VarType} returns the glsUniformBlockCase.VarType of the element in case of an Array.
**/
glsUniformBlockCase.VarType.prototype.getElementType = function() {
    return this.m_data.elementType;
};

/** getArraySize
* (not to be confused with a javascript array)
* @return {number} returns the size of the array in case it is an array.
**/
glsUniformBlockCase.VarType.prototype.getArraySize = function() {
    return this.m_data.size;
};

/** getStruct
* @return {glsUniformBlockCase.StructType} returns the structure when it is a glsUniformBlockCase.StructType.
**/
glsUniformBlockCase.VarType.prototype.getStruct = function() {
    return /** @type {glsUniformBlockCase.StructType} */ (this.m_data);
};

/**
 * Creates a basic type glsUniformBlockCase.VarType.
 * @param {gluShaderUtil.DataType} basicType
 * @param {number} flags
 * @return {glsUniformBlockCase.VarType}
 */
glsUniformBlockCase.newVarTypeBasic = function(basicType, flags) {
    return new glsUniformBlockCase.VarType().VarTypeBasic(basicType, flags);
};

/**
* Creates an array type glsUniformBlockCase.VarType.
* @param {glsUniformBlockCase.VarType} elementType
* @param {number} arraySize
* @return {glsUniformBlockCase.VarType}
*/
glsUniformBlockCase.newVarTypeArray = function(elementType, arraySize) {
    return new glsUniformBlockCase.VarType().VarTypeArray(elementType, arraySize);
};

/**
* Creates a struct type glsUniformBlockCase.VarType.
* @param {glsUniformBlockCase.StructType} structPtr
* @return {glsUniformBlockCase.VarType}
*/
glsUniformBlockCase.newVarTypeStruct = function(structPtr) {
    return new glsUniformBlockCase.VarType().VarTypeStruct(structPtr);
};

/** glsUniformBlockCase.StructMember
 * in the JSDoc annotations or if a number would do.
 * @constructor
**/
glsUniformBlockCase.StructMember = function() {
    /** @type {string} */ this.m_name;
    /** @type {glsUniformBlockCase.VarType} */ this.m_type;
    /** @type {number} */ this.m_flags = 0;
};

/**
 * Creates a glsUniformBlockCase.StructMember. Use this after the constructor call.
 * @param {string} name
 * @param {glsUniformBlockCase.VarType} type
 * @param {number} flags
 * @return {glsUniformBlockCase.StructMember} The currently modified object
 */
glsUniformBlockCase.StructMember.prototype.Constructor = function(name, type, flags) {
    this.m_type = type;
    this.m_name = name;
    this.m_flags = flags;

    return this;
};

/** getName
* @return {string} the name of the member
**/
glsUniformBlockCase.StructMember.prototype.getName = function() { return this.m_name; };

/** getType
* @return {glsUniformBlockCase.VarType} the type of the member
**/
glsUniformBlockCase.StructMember.prototype.getType = function() { return this.m_type; };

/** getFlags
* @return {number} the flags in the member
**/
glsUniformBlockCase.StructMember.prototype.getFlags = function() { return this.m_flags; };

/**
 * Creates a glsUniformBlockCase.StructMember with name, type and flags.
 * @param {string} name
 * @param {glsUniformBlockCase.VarType} type
 * @return {glsUniformBlockCase.StructMember}
 */
 glsUniformBlockCase.newStructMember = function(name, type, flags) {
     return new glsUniformBlockCase.StructMember().Constructor(name, type, flags);
 };

/**
 * glsUniformBlockCase.StructType
 * @constructor
 */
glsUniformBlockCase.StructType = function() {
    /** @type {string}*/ this.m_typeName;
    /** @type {Array<glsUniformBlockCase.StructMember>} */ this.m_members = [];
};

/**
 * glsUniformBlockCase.StructType - Constructor with type name
 * @param {string} typeName
 * @return {glsUniformBlockCase.StructType} The currently modified object.
 */
glsUniformBlockCase.StructType.prototype.Constructor = function(typeName) {
    /** @type {string}*/ this.m_typeName = typeName;
    return this;
};

/** getTypeName
* @return {string}
**/
glsUniformBlockCase.StructType.prototype.getTypeName = function() {
    return this.m_typeName;
};

/*
 * Instead of iterators, we'll add
 * a getter for a specific element (getMember),
 * and current members amount (getSize).
 */

/** getMember
* @param {number} memberNdx The index of the member to retrieve.
* @return {glsUniformBlockCase.StructMember}
**/
glsUniformBlockCase.StructType.prototype.getMember = function(memberNdx) {
    if (memberNdx >= 0 && memberNdx < this.m_members.length)
        return this.m_members[memberNdx];
    else {
        throw new Error("Invalid member index for glsUniformBlockCase.StructType's members");
    }
};

/** getSize
* @return {number} The size of the m_members array.
**/
glsUniformBlockCase.StructType.prototype.getSize = function() {
    return this.m_members.length;
};

/** addMember
* @param {string} member_name
* @param {glsUniformBlockCase.VarType} member_type
* @param {number=} member_flags
**/
glsUniformBlockCase.StructType.prototype.addMember = function(member_name, member_type, member_flags) {
    var member = glsUniformBlockCase.newStructMember(member_name, member_type, member_flags);

    this.m_members.push(member);
};

/**
 * Creates a glsUniformBlockCase.StructType.
 * @param {string} name
 * @return {glsUniformBlockCase.StructType}
 */
glsUniformBlockCase.newStructType = function(name) {
    return new glsUniformBlockCase.StructType().Constructor(name);
};

/** glsUniformBlockCase.Uniform
 * @param {string} name
 * @param {glsUniformBlockCase.VarType} type
 * @param {number=} flags
 * @constructor
**/
glsUniformBlockCase.Uniform = function(name, type, flags) {
    /** @type {string} */ this.m_name = name;
    /** @type {glsUniformBlockCase.VarType} */ this.m_type = type;
    /** @type {number} */ this.m_flags = (typeof flags === 'undefined') ? 0 : flags;
};

/** getName
 * @return {string}
 */
glsUniformBlockCase.Uniform.prototype.getName = function() {
    return this.m_name;
};

/** getType
 * @return {glsUniformBlockCase.VarType}
 */
glsUniformBlockCase.Uniform.prototype.getType = function() {
    return this.m_type;
};

/** getFlags
* @return {number}
**/
glsUniformBlockCase.Uniform.prototype.getFlags = function() {
    return this.m_flags;
};

/** glsUniformBlockCase.UniformBlock
 * @param {string} blockName
 * @constructor
**/
glsUniformBlockCase.UniformBlock = function(blockName) {
    /** @type {string} */ this.m_blockName = blockName;
    /** @type {string} */ this.m_instanceName;
    /** @type {Array<glsUniformBlockCase.Uniform>} */ this.m_uniforms = [];
    /** @type {number} */ this.m_arraySize = 0; //!< Array size or 0 if not interface block array.
    /** @type {number} */ this.m_flags = 0;
};

/** getBlockName
* @return {string}
**/
glsUniformBlockCase.UniformBlock.prototype.getBlockName = function() {
    return this.m_blockName;
};

/** getInstanceName
* @return {string}
**/
glsUniformBlockCase.UniformBlock.prototype.getInstanceName = function() {
    return this.m_instanceName;
};

/** isArray
* @return {boolean}
**/
glsUniformBlockCase.UniformBlock.prototype.isArray = function() {
    return this.m_arraySize > 0;
};

/** getArraySize
* @return {number}
**/
glsUniformBlockCase.UniformBlock.prototype.getArraySize = function() {
    return this.m_arraySize;
};

/** getFlags
* @return {number}
**/
glsUniformBlockCase.UniformBlock.prototype.getFlags = function() {
    return this.m_flags;
};

/** setInstanceName
* @param {string} name
**/
glsUniformBlockCase.UniformBlock.prototype.setInstanceName = function(name) {
    this.m_instanceName = name;
};

/** setFlags
* @param {number} flags
**/
glsUniformBlockCase.UniformBlock.prototype.setFlags = function(flags) {
    this.m_flags = flags;
};

/** setArraySize
* @param {number} arraySize
**/
glsUniformBlockCase.UniformBlock.prototype.setArraySize = function(arraySize) {
    this.m_arraySize = arraySize;
};

/** addUniform
* @param {glsUniformBlockCase.Uniform} uniform
**/
glsUniformBlockCase.UniformBlock.prototype.addUniform = function(uniform) {
    this.m_uniforms.push(uniform);
};

/*
 * Using uniform getter (getUniform),
 * and uniform array size getter (countUniforms)
 * instead of iterators.
*/

/**
 * getUniform
 * @param {number} index
 * @return {glsUniformBlockCase.Uniform}
 */
glsUniformBlockCase.UniformBlock.prototype.getUniform = function(index) {
    if (index >= 0 && index < this.m_uniforms.length)
        return this.m_uniforms[index];
    else {
        throw new Error("Invalid uniform index for glsUniformBlockCase.UniformBlock's uniforms");
    }
};

/**
 * countUniforms
 * @return {number}
 */
glsUniformBlockCase.UniformBlock.prototype.countUniforms = function() {
    return this.m_uniforms.length;
};

/**
 * glsUniformBlockCase.ShaderInterface
 * @constructor
 */
glsUniformBlockCase.ShaderInterface = function() {
    /** @type {Array<glsUniformBlockCase.StructType>} */ this.m_structs = [];
    /** @type {Array<glsUniformBlockCase.UniformBlock>} */ this.m_uniformBlocks = [];
};

/** allocStruct
* @param {string} name
* @return {glsUniformBlockCase.StructType}
**/
glsUniformBlockCase.ShaderInterface.prototype.allocStruct = function(name) {
    //m_structs.reserve(m_structs.length + 1);
    this.m_structs.push(glsUniformBlockCase.newStructType(name));
    return this.m_structs[this.m_structs.length - 1];
};

/** findStruct
* @param {string} name
* @return {glsUniformBlockCase.StructType}
**/
glsUniformBlockCase.ShaderInterface.prototype.findStruct = function(name) {
    for (var pos = 0; pos < this.m_structs.length; pos++) {
        if (this.m_structs[pos].getTypeName() == name)
            return this.m_structs[pos];
    }
    return null;
};

/** getNamedStructs
* @param {Array<glsUniformBlockCase.StructType>} structs
**/
glsUniformBlockCase.ShaderInterface.prototype.getNamedStructs = function(structs) {
    for (var pos = 0; pos < this.m_structs.length; pos++) {
        if (this.m_structs[pos].getTypeName() != undefined)
            structs.push(this.m_structs[pos]);
    }
};

/** allocBlock
* @param {string} name
* @return {glsUniformBlockCase.UniformBlock}
**/
glsUniformBlockCase.ShaderInterface.prototype.allocBlock = function(name) {
    this.m_uniformBlocks.push(new glsUniformBlockCase.UniformBlock(name));
    return this.m_uniformBlocks[this.m_uniformBlocks.length - 1];
};

/** getNumUniformBlocks
* @return {number}
**/
glsUniformBlockCase.ShaderInterface.prototype.getNumUniformBlocks = function() {
    return this.m_uniformBlocks.length;
};

/** getUniformBlock
* @param {number} ndx
* @return {glsUniformBlockCase.UniformBlock}
**/
glsUniformBlockCase.ShaderInterface.prototype.getUniformBlock = function(ndx) {
    return this.m_uniformBlocks[ndx];
};

/**
 * @constructor
 */
glsUniformBlockCase.BlockLayoutEntry = function() {
    return {
    /** @type {number} */ size: 0,
    /** @type {string} */ name: '',
    /** @type {Array<number>} */ activeUniformIndices: []
    };
};

/**
 * @constructor
 */
glsUniformBlockCase.UniformLayoutEntry = function() {
    return {
    /** @type {string} */ name: '',
    /** @type {gluShaderUtil.DataType} */ type: gluShaderUtil.DataType.INVALID,
    /** @type {number} */ size: 0,
    /** @type {number} */ blockNdx: -1,
    /** @type {number} */ offset: -1,
    /** @type {number} */ arrayStride: -1,
    /** @type {number} */ matrixStride: -1,
    /** @type {boolean} */ isRowMajor: false
    };
};

/**
 * @constructor
 */
glsUniformBlockCase.UniformLayout = function() {
    /** @type {Array<glsUniformBlockCase.BlockLayoutEntry>}*/ this.blocks = [];
    /** @type {Array<glsUniformBlockCase.UniformLayoutEntry>}*/ this.uniforms = [];
};

/** getUniformIndex, returns a uniform index number in the layout,
 * given the uniform's name.
 * @param {string} name
 * @return {number} uniform's index
 */
glsUniformBlockCase.UniformLayout.prototype.getUniformIndex = function(name) {
    for (var ndx = 0; ndx < this.uniforms.length; ndx++) {
        if (this.uniforms[ndx].name == name)
            return ndx;
    }
    return -1;
};

/** getBlockIndex, returns a block index number in the layout,
 * given the block's name.
 * @param {string} name the name of the block
 * @return {number} block's index
 */
glsUniformBlockCase.UniformLayout.prototype.getBlockIndex = function(name) {
    for (var ndx = 0; ndx < this.blocks.length; ndx++) {
        if (this.blocks[ndx].name == name)
            return ndx;
    }
    return -1;
};

/**
 * @enum {number}
 */
glsUniformBlockCase.BufferMode = {
    BUFFERMODE_SINGLE: 0, //!< Single buffer shared between uniform blocks.
    BUFFERMODE_PER_BLOCK: 1 //!< Per-block buffers
};

glsUniformBlockCase.BufferMode.BUFFERMODE_LAST = Object.keys(glsUniformBlockCase.BufferMode).length;

/**
 * glsUniformBlockCase.PrecisionFlagsFmt
 * @param {number} flags
 * @return {string}
 */
glsUniformBlockCase.PrecisionFlagsFmt = function(flags) {
    // Precision.
    DE_ASSERT(deMath.dePop32(flags & (glsUniformBlockCase.UniformFlags.PRECISION_LOW | glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM | glsUniformBlockCase.UniformFlags.PRECISION_HIGH)) <= 1);
    var str = '';
    str += (flags & glsUniformBlockCase.UniformFlags.PRECISION_LOW ? 'lowp' :
            flags & glsUniformBlockCase.UniformFlags.PRECISION_MEDIUM ? 'mediump' :
            flags & glsUniformBlockCase.UniformFlags.PRECISION_HIGH ? 'highp' : '');

    return str;
};

/**
 * glsUniformBlockCase.LayoutFlagsFmt
 * @param {number} flags_
 * @return {string}
 */
glsUniformBlockCase.LayoutFlagsFmt = function(flags_) {
    var str = '';
    var bitDesc =
    [{ bit: glsUniformBlockCase.UniformFlags.LAYOUT_SHARED, token: 'shared' }, { bit: glsUniformBlockCase.UniformFlags.LAYOUT_PACKED, token: 'packed' }, { bit: glsUniformBlockCase.UniformFlags.LAYOUT_STD140, token: 'std140' }, { bit: glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR, token: 'row_major' }, { bit: glsUniformBlockCase.UniformFlags.LAYOUT_COLUMN_MAJOR, token: 'column_major' }
    ];

    /** @type {number} */ var remBits = flags_;
    for (var descNdx = 0; descNdx < bitDesc.length; descNdx++) {
        if (remBits & bitDesc[descNdx].bit) {
            if (remBits != flags_)
                str += ', ';
            str += bitDesc[descNdx].token;
            remBits &= (~bitDesc[descNdx].bit) & 0xFFFFFFFF; //0xFFFFFFFF truncate to 32 bit value
        }
    }
    DE_ASSERT(remBits == 0);

    return str;
};

/**
 * @constructor
 */
glsUniformBlockCase.UniformBufferManager = function(renderCtx) {
    this.m_renderCtx = renderCtx;
    /** @type {Array<number>} */ this.m_buffers = [];
};

/**
 * allocBuffer
 * @return {WebGLBuffer}
 */
glsUniformBlockCase.UniformBufferManager.prototype.allocBuffer = function() {
    /** @type {WebGLBuffer} */ var buf = this.m_renderCtx.createBuffer();

    this.m_buffers.push(buf);

    return buf;
};

/**
 * @param {string} name
 * @param {string} description
 * @param {glsUniformBlockCase.BufferMode} bufferMode
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
glsUniformBlockCase.UniformBlockCase = function(name, description, bufferMode) {
    tcuTestCase.DeqpTest.call(this, name, description);
    /** @type {string} */ this.m_name = name;
    /** @type {string} */ this.m_description = description;
    /** @type {glsUniformBlockCase.BufferMode} */ this.m_bufferMode = bufferMode;
    /** @type {glsUniformBlockCase.ShaderInterface} */ this.m_interface = new glsUniformBlockCase.ShaderInterface();
};

glsUniformBlockCase.UniformBlockCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
glsUniformBlockCase.UniformBlockCase.prototype.constructor = glsUniformBlockCase.UniformBlockCase;

/**
 * glsUniformBlockCase.getDataTypeByteSize
 * @param {gluShaderUtil.DataType} type
 * @return {number}
 */
glsUniformBlockCase.getDataTypeByteSize = function(type) {
    return gluShaderUtil.getDataTypeScalarSize(type) * deMath.INT32_SIZE;
};

/**
 * glsUniformBlockCase.getDataTypeByteAlignment
 * @param {gluShaderUtil.DataType} type
 * @return {number}
 */
glsUniformBlockCase.getDataTypeByteAlignment = function(type) {
    switch (type) {
        case gluShaderUtil.DataType.FLOAT:
        case gluShaderUtil.DataType.INT:
        case gluShaderUtil.DataType.UINT:
        case gluShaderUtil.DataType.BOOL: return 1 * deMath.INT32_SIZE;

        case gluShaderUtil.DataType.FLOAT_VEC2:
        case gluShaderUtil.DataType.INT_VEC2:
        case gluShaderUtil.DataType.UINT_VEC2:
        case gluShaderUtil.DataType.BOOL_VEC2: return 2 * deMath.INT32_SIZE;

        case gluShaderUtil.DataType.FLOAT_VEC3:
        case gluShaderUtil.DataType.INT_VEC3:
        case gluShaderUtil.DataType.UINT_VEC3:
        case gluShaderUtil.DataType.BOOL_VEC3: // Fall-through to vec4

        case gluShaderUtil.DataType.FLOAT_VEC4:
        case gluShaderUtil.DataType.INT_VEC4:
        case gluShaderUtil.DataType.UINT_VEC4:
        case gluShaderUtil.DataType.BOOL_VEC4: return 4 * deMath.INT32_SIZE;

        default:
            DE_ASSERT(false);
            return 0;
    }
};

/**
 * glsUniformBlockCase.getDataTypeArrayStride
 * @param {gluShaderUtil.DataType} type
 * @return {number}
 */
glsUniformBlockCase.getDataTypeArrayStride = function(type) {
    DE_ASSERT(!gluShaderUtil.isDataTypeMatrix(type));

    /** @type {number} */ var baseStride = glsUniformBlockCase.getDataTypeByteSize(type);
    /** @type {number} */ var vec4Alignment = deMath.INT32_SIZE * 4;

    DE_ASSERT(baseStride <= vec4Alignment);
    return Math.max(baseStride, vec4Alignment); // Really? See rule 4.
};

/**
 * glsUniformBlockCase.deRoundUp32 Rounds up 'a' in case the
 * relationship with 'b' has a decimal part.
 * @param {number} a
 * @param {number} b
 * @return {number}
 */
glsUniformBlockCase.deRoundUp32 = function(a, b) {
    var d = Math.trunc(a / b);
    return d * b == a ? a : (d + 1) * b;
};

/**
 * glsUniformBlockCase.computeStd140BaseAlignment
 * @param {glsUniformBlockCase.VarType} type
 * @return {number}
 */
glsUniformBlockCase.computeStd140BaseAlignment = function(type) {
    /** @type {number} */ var vec4Alignment = deMath.INT32_SIZE * 4;

    if (type.isBasicType()) {
        /** @type {gluShaderUtil.DataType} */ var basicType = type.getBasicType();

        if (gluShaderUtil.isDataTypeMatrix(basicType)) {
            /** @type {boolean} */ var isRowMajor = !!(type.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR);
            /** @type {number} */ var vecSize = isRowMajor ? gluShaderUtil.getDataTypeMatrixNumColumns(basicType) :
            gluShaderUtil.getDataTypeMatrixNumRows(basicType);

            return glsUniformBlockCase.getDataTypeArrayStride(gluShaderUtil.getDataTypeFloatVec(vecSize));
        } else
            return glsUniformBlockCase.getDataTypeByteAlignment(basicType);
    } else if (type.isArrayType()) {
        /** @type {number} */ var elemAlignment = glsUniformBlockCase.computeStd140BaseAlignment(type.getElementType());

        // Round up to alignment of vec4
        return glsUniformBlockCase.deRoundUp32(elemAlignment, vec4Alignment);
    } else {
        DE_ASSERT(type.isStructType());

        /** @type {number} */ var maxBaseAlignment = 0;

        for (var memberNdx = 0; memberNdx < type.getStruct().getSize(); memberNdx++) {
            /** @type {glsUniformBlockCase.StructMember} */ var memberIter = type.getStruct().getMember(memberNdx);
            maxBaseAlignment = Math.max(maxBaseAlignment, glsUniformBlockCase.computeStd140BaseAlignment(memberIter.getType()));
        }

        return glsUniformBlockCase.deRoundUp32(maxBaseAlignment, vec4Alignment);
    }
};

/**
 * mergeLayoutflags
 * @param {number} prevFlags
 * @param {number} newFlags
 * @return {number}
 */
glsUniformBlockCase.mergeLayoutFlags = function(prevFlags, newFlags) {
    /** @type {number} */ var packingMask = glsUniformBlockCase.UniformFlags.LAYOUT_PACKED | glsUniformBlockCase.UniformFlags.LAYOUT_SHARED | glsUniformBlockCase.UniformFlags.LAYOUT_STD140;
    /** @type {number} */ var matrixMask = glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR | glsUniformBlockCase.UniformFlags.LAYOUT_COLUMN_MAJOR;

    /** @type {number} */ var mergedFlags = 0;

    mergedFlags |= ((newFlags & packingMask) ? newFlags : prevFlags) & packingMask;
    mergedFlags |= ((newFlags & matrixMask) ? newFlags : prevFlags) & matrixMask;

    return mergedFlags;
};

/**
 * glsUniformBlockCase.computeStd140Layout_B
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {number} curOffset
 * @param {number} curBlockNdx
 * @param {string} curPrefix
 * @param {glsUniformBlockCase.VarType} type
 * @param {number} layoutFlags
 * @return {number} //This is what would return in the curOffset output parameter in the original C++ project.
 */
glsUniformBlockCase.computeStd140Layout_B = function(layout, curOffset, curBlockNdx, curPrefix, type, layoutFlags) {
    /** @type {number} */ var baseAlignment = glsUniformBlockCase.computeStd140BaseAlignment(type);
    /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var entry;
    /** @type {number} */ var stride;
    /** @type {gluShaderUtil.DataType} */ var elemBasicType;
    /** @type {boolean} */ var isRowMajor;
    /** @type {number} */ var vecSize;
    /** @type {number} */ var numVecs;

    curOffset = deMath.deAlign32(curOffset, baseAlignment);

    if (type.isBasicType()) {
        /** @type {gluShaderUtil.DataType} */ var basicType = type.getBasicType();
        entry = new glsUniformBlockCase.UniformLayoutEntry();

        entry.name = curPrefix;
        entry.type = basicType;
        entry.size = 1;
        entry.arrayStride = 0;
        entry.matrixStride = 0;
        entry.blockNdx = curBlockNdx;

        if (gluShaderUtil.isDataTypeMatrix(basicType)) {
            // Array of vectors as specified in rules 5 & 7.
            isRowMajor = !!(layoutFlags & glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR);
            vecSize = isRowMajor ? gluShaderUtil.getDataTypeMatrixNumColumns(basicType) :
            gluShaderUtil.getDataTypeMatrixNumRows(basicType);
            numVecs = isRowMajor ? gluShaderUtil.getDataTypeMatrixNumRows(basicType) :
            gluShaderUtil.getDataTypeMatrixNumColumns(basicType);
            stride = glsUniformBlockCase.getDataTypeArrayStride(gluShaderUtil.getDataTypeFloatVec(vecSize));

            entry.offset = curOffset;
            entry.matrixStride = stride;
            entry.isRowMajor = isRowMajor;

            curOffset += numVecs * stride;
        } else {
            // Scalar or vector.
            entry.offset = curOffset;

            curOffset += glsUniformBlockCase.getDataTypeByteSize(basicType);
        }

        layout.uniforms.push(entry);
    } else if (type.isArrayType()) {
        /** @type {glsUniformBlockCase.VarType} */ var elemType = type.getElementType();

        if (elemType.isBasicType() && !gluShaderUtil.isDataTypeMatrix(elemType.getBasicType())) {
            // Array of scalars or vectors.
            elemBasicType = elemType.getBasicType();
            entry = new glsUniformBlockCase.UniformLayoutEntry();
            stride = glsUniformBlockCase.getDataTypeArrayStride(elemBasicType);

            entry.name = curPrefix + '[0]'; // Array uniforms are always postfixed with [0]
            entry.type = elemBasicType;
            entry.blockNdx = curBlockNdx;
            entry.offset = curOffset;
            entry.size = type.getArraySize();
            entry.arrayStride = stride;
            entry.matrixStride = 0;

            curOffset += stride * type.getArraySize();

            layout.uniforms.push(entry);
        } else if (elemType.isBasicType() && gluShaderUtil.isDataTypeMatrix(elemType.getBasicType())) {
            // Array of matrices.
            elemBasicType = elemType.getBasicType();
            isRowMajor = !!(layoutFlags & glsUniformBlockCase.UniformFlags.LAYOUT_ROW_MAJOR);
            vecSize = isRowMajor ? gluShaderUtil.getDataTypeMatrixNumColumns(elemBasicType) :
            gluShaderUtil.getDataTypeMatrixNumRows(elemBasicType);
            numVecs = isRowMajor ? gluShaderUtil.getDataTypeMatrixNumRows(elemBasicType) :
            gluShaderUtil.getDataTypeMatrixNumColumns(elemBasicType);
            stride = glsUniformBlockCase.getDataTypeArrayStride(gluShaderUtil.getDataTypeFloatVec(vecSize));
            entry = new glsUniformBlockCase.UniformLayoutEntry();

            entry.name = curPrefix + '[0]'; // Array uniforms are always postfixed with [0]
            entry.type = elemBasicType;
            entry.blockNdx = curBlockNdx;
            entry.offset = curOffset;
            entry.size = type.getArraySize();
            entry.arrayStride = stride * numVecs;
            entry.matrixStride = stride;
            entry.isRowMajor = isRowMajor;

            curOffset += numVecs * type.getArraySize() * stride;

            layout.uniforms.push(entry);
        } else {
            DE_ASSERT(elemType.isStructType() || elemType.isArrayType());

            for (var elemNdx = 0; elemNdx < type.getArraySize(); elemNdx++)
                curOffset = glsUniformBlockCase.computeStd140Layout_B(layout, curOffset, curBlockNdx, curPrefix + '[' + elemNdx + ']', type.getElementType(), layoutFlags);
        }
    } else {
        DE_ASSERT(type.isStructType());

        for (var memberNdx = 0; memberNdx < type.getStruct().getSize(); memberNdx++) {
            /** @type {glsUniformBlockCase.StructMember} */ var memberIter = type.getStruct().getMember(memberNdx);
            curOffset = glsUniformBlockCase.computeStd140Layout_B(layout, curOffset, curBlockNdx, curPrefix + '.' + memberIter.getName(), memberIter.getType(), layoutFlags);
        }

        curOffset = deMath.deAlign32(curOffset, baseAlignment);
    }

    return curOffset;
};

/**
 * glsUniformBlockCase.computeStd140Layout
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {glsUniformBlockCase.ShaderInterface} sinterface
 */
glsUniformBlockCase.computeStd140Layout = function(layout, sinterface) {
    // \todo [2012-01-23 pyry] Uniforms in default block.

    /** @type {number} */ var numUniformBlocks = sinterface.getNumUniformBlocks();

    for (var blockNdx = 0; blockNdx < numUniformBlocks; blockNdx++) {
        /** @type {glsUniformBlockCase.UniformBlock} */ var block = sinterface.getUniformBlock(blockNdx);
        /** @type {boolean} */ var hasInstanceName = block.getInstanceName() !== undefined;
        /** @type {string} */ var blockPrefix = hasInstanceName ? (block.getBlockName() + '.') : '';
        /** @type {number} */ var curOffset = 0;
        /** @type {number} */ var activeBlockNdx = layout.blocks.length;
        /** @type {number} */ var firstUniformNdx = layout.uniforms.length;

        for (var ubNdx = 0; ubNdx < block.countUniforms(); ubNdx++) {
            /** @type {glsUniformBlockCase.Uniform} */ var uniform = block.getUniform(ubNdx);
            curOffset = glsUniformBlockCase.computeStd140Layout_B(layout, curOffset, activeBlockNdx, blockPrefix + uniform.getName(), uniform.getType(), glsUniformBlockCase.mergeLayoutFlags(block.getFlags(), uniform.getFlags()));
        }

        /** @type {number} */ var uniformIndicesEnd = layout.uniforms.length;
        /** @type {number} */ var blockSize = curOffset;
        /** @type {number} */ var numInstances = block.isArray() ? block.getArraySize() : 1;

        // Create block layout entries for each instance.
        for (var instanceNdx = 0; instanceNdx < numInstances; instanceNdx++) {
            // Allocate entry for instance.
            layout.blocks.push(new glsUniformBlockCase.BlockLayoutEntry());
            /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var blockEntry = layout.blocks[layout.blocks.length - 1];

            blockEntry.name = block.getBlockName();
            blockEntry.size = blockSize;

            // Compute active uniform set for block.
            for (var uniformNdx = firstUniformNdx; uniformNdx < uniformIndicesEnd; uniformNdx++)
                blockEntry.activeUniformIndices.push(uniformNdx);

            if (block.isArray())
                blockEntry.name += '[' + instanceNdx + ']';
        }
    }
};

/**
 * glsUniformBlockCase.generateValue - Value generator
 * @param {glsUniformBlockCase.UniformLayoutEntry} entry
 * @param {Uint8Array} basePtr
 * @param {deRandom.Random} rnd
 */
glsUniformBlockCase.generateValue = function(entry, basePtr, rnd) {
    /** @type {gluShaderUtil.DataType}*/ var scalarType = gluShaderUtil.getDataTypeScalarTypeAsDataType(entry.type); //Using a more appropriate function in this case.
    /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(entry.type);
    /** @type {boolean} */ var isMatrix = gluShaderUtil.isDataTypeMatrix(entry.type);
    /** @type {number} */ var numVecs = isMatrix ? (entry.isRowMajor ? gluShaderUtil.getDataTypeMatrixNumRows(entry.type) : gluShaderUtil.getDataTypeMatrixNumColumns(entry.type)) : 1;
    /** @type {number} */ var vecSize = scalarSize / numVecs;
    /** @type {boolean} */ var isArray = entry.size > 1;
    /** @type {number} */ var compSize = deMath.INT32_SIZE;

    DE_ASSERT(scalarSize % numVecs == 0);

    for (var elemNdx = 0; elemNdx < entry.size; elemNdx++) {
        /** @type {Uint8Array} */ var elemPtr = basePtr.subarray(entry.offset + (isArray ? elemNdx * entry.arrayStride : 0));

        for (var vecNdx = 0; vecNdx < numVecs; vecNdx++) {
            /** @type {Uint8Array} */ var vecPtr = elemPtr.subarray(isMatrix ? vecNdx * entry.matrixStride : 0);

            for (var compNdx = 0; compNdx < vecSize; compNdx++) {
                /** @type {Uint8Array} */ var compPtr = vecPtr.subarray(compSize * compNdx);
                /** @type {number} */ var _random;

                //Copy the random data byte per byte
                var _size = glsUniformBlockCase.getDataTypeByteSize(scalarType);

                var nbuffer = new ArrayBuffer(_size);
                var nview = new DataView(nbuffer);

                switch (scalarType) {
                    case gluShaderUtil.DataType.FLOAT:
                        _random = rnd.getInt(-9, 9);
                        nview.setFloat32(0, _random, littleEndian);
                        break;
                    case gluShaderUtil.DataType.INT:
                        _random = rnd.getInt(-9, 9);
                        nview.setInt32(0, _random, littleEndian);
                        break;
                    case gluShaderUtil.DataType.UINT:
                        _random = rnd.getInt(0, 9);
                        nview.setUint32(0, _random, littleEndian);
                        break;
                    // \note Random bit pattern is used for true values. Spec states that all non-zero values are
                    //       interpreted as true but some implementations fail this.
                    case gluShaderUtil.DataType.BOOL:
                        _random = rnd.getBool() ? 1 : 0;
                        nview.setUint32(0, _random, littleEndian);
                        break;
                    default:
                        DE_ASSERT(false);
                }

                for (var i = 0; i < _size; i++) {
                    compPtr[i] = nview.getUint8(i);
                }
            }
        }
    }
};

/**
 * glsUniformBlockCase.generateValues
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {glsUniformBlockCase.BlockPointers} blockPointers
 * @param {number} seed
 */
glsUniformBlockCase.generateValues = function(layout, blockPointers, seed) {
    /** @type  {deRandom.Random} */ var rnd = new deRandom.Random(seed);
    /** @type  {number} */ var numBlocks = layout.blocks.length;

    for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
        /** @type {Uint8Array} */ var basePtr = blockPointers.find(blockNdx);
        /** @type  {number} */ var numEntries = layout.blocks[blockNdx].activeUniformIndices.length;

        for (var entryNdx = 0; entryNdx < numEntries; entryNdx++) {
            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var entry = layout.uniforms[layout.blocks[blockNdx].activeUniformIndices[entryNdx]];
            glsUniformBlockCase.generateValue(entry, basePtr, rnd);
        }
    }
};

// Shader generator.

/**
 * glsUniformBlockCase.getCompareFuncForType
 * @param {gluShaderUtil.DataType} type
 * @return {string}
 */
glsUniformBlockCase.getCompareFuncForType = function(type) {
    switch (type) {
        case gluShaderUtil.DataType.FLOAT: return 'mediump float compare_float (highp float a, highp float b) { return abs(a - b) < 0.05 ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.FLOAT_VEC2: return 'mediump float compare_vec2 (highp vec2 a, highp vec2 b) { return compare_float(a.x, b.x)*compare_float(a.y, b.y); }\n';
        case gluShaderUtil.DataType.FLOAT_VEC3: return 'mediump float compare_vec3 (highp vec3 a, highp vec3 b) { return compare_float(a.x, b.x)*compare_float(a.y, b.y)*compare_float(a.z, b.z); }\n';
        case gluShaderUtil.DataType.FLOAT_VEC4: return 'mediump float compare_vec4 (highp vec4 a, highp vec4 b) { return compare_float(a.x, b.x)*compare_float(a.y, b.y)*compare_float(a.z, b.z)*compare_float(a.w, b.w); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT2: return 'mediump float compare_mat2 (highp mat2 a, highp mat2 b) { return compare_vec2(a[0], b[0])*compare_vec2(a[1], b[1]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT2X3: return 'mediump float compare_mat2x3 (highp mat2x3 a, highp mat2x3 b) { return compare_vec3(a[0], b[0])*compare_vec3(a[1], b[1]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT2X4: return 'mediump float compare_mat2x4 (highp mat2x4 a, highp mat2x4 b) { return compare_vec4(a[0], b[0])*compare_vec4(a[1], b[1]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT3X2: return 'mediump float compare_mat3x2 (highp mat3x2 a, highp mat3x2 b) { return compare_vec2(a[0], b[0])*compare_vec2(a[1], b[1])*compare_vec2(a[2], b[2]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT3: return 'mediump float compare_mat3 (highp mat3 a, highp mat3 b) { return compare_vec3(a[0], b[0])*compare_vec3(a[1], b[1])*compare_vec3(a[2], b[2]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT3X4: return 'mediump float compare_mat3x4 (highp mat3x4 a, highp mat3x4 b) { return compare_vec4(a[0], b[0])*compare_vec4(a[1], b[1])*compare_vec4(a[2], b[2]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT4X2: return 'mediump float compare_mat4x2 (highp mat4x2 a, highp mat4x2 b) { return compare_vec2(a[0], b[0])*compare_vec2(a[1], b[1])*compare_vec2(a[2], b[2])*compare_vec2(a[3], b[3]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT4X3: return 'mediump float compare_mat4x3 (highp mat4x3 a, highp mat4x3 b) { return compare_vec3(a[0], b[0])*compare_vec3(a[1], b[1])*compare_vec3(a[2], b[2])*compare_vec3(a[3], b[3]); }\n';
        case gluShaderUtil.DataType.FLOAT_MAT4: return 'mediump float compare_mat4 (highp mat4 a, highp mat4 b) { return compare_vec4(a[0], b[0])*compare_vec4(a[1], b[1])*compare_vec4(a[2], b[2])*compare_vec4(a[3], b[3]); }\n';
        case gluShaderUtil.DataType.INT: return 'mediump float compare_int (highp int a, highp int b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.INT_VEC2: return 'mediump float compare_ivec2 (highp ivec2 a, highp ivec2 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.INT_VEC3: return 'mediump float compare_ivec3 (highp ivec3 a, highp ivec3 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.INT_VEC4: return 'mediump float compare_ivec4 (highp ivec4 a, highp ivec4 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.UINT: return 'mediump float compare_uint (highp uint a, highp uint b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.UINT_VEC2: return 'mediump float compare_uvec2 (highp uvec2 a, highp uvec2 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.UINT_VEC3: return 'mediump float compare_uvec3 (highp uvec3 a, highp uvec3 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.UINT_VEC4: return 'mediump float compare_uvec4 (highp uvec4 a, highp uvec4 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.BOOL: return 'mediump float compare_bool (bool a, bool b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.BOOL_VEC2: return 'mediump float compare_bvec2 (bvec2 a, bvec2 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.BOOL_VEC3: return 'mediump float compare_bvec3 (bvec3 a, bvec3 b) { return a == b ? 1.0 : 0.0; }\n';
        case gluShaderUtil.DataType.BOOL_VEC4: return 'mediump float compare_bvec4 (bvec4 a, bvec4 b) { return a == b ? 1.0 : 0.0; }\n';
        default:
            throw new Error('Type "' + type + '" not supported.');

    }
};

/**
 * glsUniformBlockCase.getCompareDependencies
 * @param {Array<gluShaderUtil.DataType>} compareFuncs Should contain unique elements
 * @param {gluShaderUtil.DataType} basicType
 */
glsUniformBlockCase.getCompareDependencies = function(compareFuncs, basicType) {
    switch (basicType) {
        case gluShaderUtil.DataType.FLOAT_VEC2:
        case gluShaderUtil.DataType.FLOAT_VEC3:
        case gluShaderUtil.DataType.FLOAT_VEC4:
            deUtil.dePushUniqueToArray(compareFuncs, gluShaderUtil.DataType.FLOAT);
            deUtil.dePushUniqueToArray(compareFuncs, basicType);
            break;

        case gluShaderUtil.DataType.FLOAT_MAT2:
        case gluShaderUtil.DataType.FLOAT_MAT2X3:
        case gluShaderUtil.DataType.FLOAT_MAT2X4:
        case gluShaderUtil.DataType.FLOAT_MAT3X2:
        case gluShaderUtil.DataType.FLOAT_MAT3:
        case gluShaderUtil.DataType.FLOAT_MAT3X4:
        case gluShaderUtil.DataType.FLOAT_MAT4X2:
        case gluShaderUtil.DataType.FLOAT_MAT4X3:
        case gluShaderUtil.DataType.FLOAT_MAT4:
            deUtil.dePushUniqueToArray(compareFuncs, gluShaderUtil.DataType.FLOAT);
            deUtil.dePushUniqueToArray(compareFuncs, gluShaderUtil.getDataTypeFloatVec(gluShaderUtil.getDataTypeMatrixNumRows(basicType)));
            deUtil.dePushUniqueToArray(compareFuncs, basicType);
            break;

        default:
            deUtil.dePushUniqueToArray(compareFuncs, basicType);
            break;
    }
};

/**
 * glsUniformBlockCase.collectUniqueBasicTypes_B
 * @param {Array<gluShaderUtil.DataType>} basicTypes Should contain unique elements
 * @param {glsUniformBlockCase.VarType} type
 */
glsUniformBlockCase.collectUniqueBasicTypes_B = function(basicTypes, type) {
    if (type.isStructType()) {
        /** @type {glsUniformBlockCase.StructType} */ var stype = type.getStruct();
        for (var memberNdx = 0; memberNdx < stype.getSize(); memberNdx++)
            glsUniformBlockCase.collectUniqueBasicTypes_B(basicTypes, stype.getMember(memberNdx).getType());
    } else if (type.isArrayType())
        glsUniformBlockCase.collectUniqueBasicTypes_B(basicTypes, type.getElementType());
    else {
        DE_ASSERT(type.isBasicType());
        deUtil.dePushUniqueToArray(basicTypes, type.getBasicType());
    }
};

/**
 * glsUniformBlockCase.collectUniqueBasicTypes_A
 * @param {Array<gluShaderUtil.DataType>} basicTypes Should contain unique elements
 * @param {glsUniformBlockCase.UniformBlock} uniformBlock
 */
glsUniformBlockCase.collectUniqueBasicTypes_A = function(basicTypes, uniformBlock) {
    for (var uniformNdx = 0; uniformNdx < uniformBlock.countUniforms(); uniformNdx++)
        glsUniformBlockCase.collectUniqueBasicTypes_B(basicTypes, uniformBlock.getUniform(uniformNdx).getType());
};

/**
 * glsUniformBlockCase.collectUniqueBasicTypes
 * @param {Array<gluShaderUtil.DataType>} basicTypes Should contain unique elements
 * @param {glsUniformBlockCase.ShaderInterface} sinterface
 */
glsUniformBlockCase.collectUniqueBasicTypes = function(basicTypes, sinterface) {
    for (var ndx = 0; ndx < sinterface.getNumUniformBlocks(); ++ndx)
        glsUniformBlockCase.collectUniqueBasicTypes_A(basicTypes, sinterface.getUniformBlock(ndx));
};

/**
 * glsUniformBlockCase.collectUniqueBasicTypes
 * @return {string} Was originally an output parameter. As it is a basic type, we have to return it instead.
 * @param {glsUniformBlockCase.ShaderInterface} sinterface
 */
glsUniformBlockCase.generateCompareFuncs = function(sinterface) {
    /** @type {string} */ var str = '';
    /** @type {Array<gluShaderUtil.DataType>} */ var types = []; //Will contain unique elements.
    /** @type {Array<gluShaderUtil.DataType>} */ var compareFuncs = []; //Will contain unique elements.

    // Collect unique basic types
    glsUniformBlockCase.collectUniqueBasicTypes(types, sinterface);

    // Set of compare functions required
    for (var typeNdx = 0; typeNdx < types.length; typeNdx++)
        glsUniformBlockCase.getCompareDependencies(compareFuncs, types[typeNdx]);

    for (var type in gluShaderUtil.DataType) {
        if (compareFuncs.indexOf(gluShaderUtil.DataType[type]) > -1)
            str += glsUniformBlockCase.getCompareFuncForType(gluShaderUtil.DataType[type]);
    }

    return str;
};

/**
 * glsUniformBlockCase.Indent - Prints level_ number of tab chars
 * @param {number} level_
 * @return {string}
 */
glsUniformBlockCase.Indent = function(level_) {
    var str = '';
    for (var i = 0; i < level_; i++)
        str += '\t';

    return str;
};

/**
 * glsUniformBlockCase.generateDeclaration_C
 * @return {string} src
 * @param {glsUniformBlockCase.StructType} structType
 * @param {number} indentLevel
 */
glsUniformBlockCase.generateDeclaration_C = function(structType, indentLevel) {
    /** @type {string} */ var src = '';

    DE_ASSERT(structType.getTypeName() !== undefined);
    src += glsUniformBlockCase.generateFullDeclaration(structType, indentLevel);
    src += ';\n';

    return src;
};

/**
 * glsUniformBlockCase.generateFullDeclaration
 * @return {string} src
 * @param {glsUniformBlockCase.StructType} structType
 * @param {number} indentLevel
 */
glsUniformBlockCase.generateFullDeclaration = function(structType, indentLevel) {
    var src = 'struct';
    if (structType.getTypeName())
        src += ' ' + structType.getTypeName();
    src += '\n' + glsUniformBlockCase.Indent(indentLevel) + ' {\n';

    for (var memberNdx = 0; memberNdx < structType.getSize(); memberNdx++) {
        src += glsUniformBlockCase.Indent(indentLevel + 1);
        /** @type {glsUniformBlockCase.StructMember} */ var memberIter = structType.getMember(memberNdx);
        src += glsUniformBlockCase.generateDeclaration_B(memberIter.getType(), memberIter.getName(), indentLevel + 1, memberIter.getFlags() & glsUniformBlockCase.UniformFlags.UNUSED_BOTH);
    }

    src += glsUniformBlockCase.Indent(indentLevel) + '}';

    return src;
};

/**
 * glsUniformBlockCase.generateLocalDeclaration
 * @return {string} src
 * @param {glsUniformBlockCase.StructType} structType
 * @param {number} indentLevel
 */
glsUniformBlockCase.generateLocalDeclaration = function(structType, indentLevel) {
    /** @type {string} */ var src = '';

    if (structType.getTypeName() === undefined)
        src += glsUniformBlockCase.generateFullDeclaration(structType, indentLevel);
    else
        src += structType.getTypeName();

    return src;
};

/**
 * glsUniformBlockCase.generateDeclaration_B
 * @return {string} src
 * @param {glsUniformBlockCase.VarType} type
 * @param {string} name
 * @param {number} indentLevel
 * @param {number} unusedHints
 */
glsUniformBlockCase.generateDeclaration_B = function(type, name, indentLevel, unusedHints) {
    /** @type {string} */ var src = '';
    /** @type {number} */ var flags = type.getFlags();

    if ((flags & glsUniformBlockCase.UniformFlags.LAYOUT_MASK) != 0)
        src += 'layout(' + glsUniformBlockCase.LayoutFlagsFmt(flags & glsUniformBlockCase.UniformFlags.LAYOUT_MASK) + ') ';

    if ((flags & glsUniformBlockCase.UniformFlags.PRECISION_MASK) != 0)
        src += glsUniformBlockCase.PrecisionFlagsFmt(flags & glsUniformBlockCase.UniformFlags.PRECISION_MASK) + ' ';

    if (type.isBasicType())
        src += gluShaderUtil.getDataTypeName(type.getBasicType()) + ' ' + name;
    else if (type.isArrayType()) {
        /** @type {Array<number>} */ var arraySizes = [];
        /** @type {glsUniformBlockCase.VarType} */ var curType = type;
        while (curType.isArrayType()) {
            arraySizes.push(curType.getArraySize());
            curType = curType.getElementType();
        }

        if (curType.isBasicType()) {
            if ((curType.getFlags() & glsUniformBlockCase.UniformFlags.PRECISION_MASK) != 0)
                src += glsUniformBlockCase.PrecisionFlagsFmt(curType.getFlags() & glsUniformBlockCase.UniformFlags.PRECISION_MASK) + ' ';
            src += gluShaderUtil.getDataTypeName(curType.getBasicType());
        } else {
            DE_ASSERT(curType.isStructType());
            src += glsUniformBlockCase.generateLocalDeclaration(curType.getStruct(), indentLevel + 1);
        }

        src += ' ' + name;

        for (var sizeNdx = 0; sizeNdx < arraySizes.length; sizeNdx++)
            src += '[' + arraySizes[sizeNdx] + ']';
    } else {
        src += glsUniformBlockCase.generateLocalDeclaration(type.getStruct(), indentLevel + 1);
        src += ' ' + name;
    }

    src += ';';

    // Print out unused hints.
    if (unusedHints != 0)
        src += ' // unused in ' + (unusedHints == glsUniformBlockCase.UniformFlags.UNUSED_BOTH ? 'both shaders' :
                                    unusedHints == glsUniformBlockCase.UniformFlags.UNUSED_VERTEX ? 'vertex shader' :
                                    unusedHints == glsUniformBlockCase.UniformFlags.UNUSED_FRAGMENT ? 'fragment shader' : '???');

    src += '\n';

    return src;
};

/**
 * glsUniformBlockCase.generateDeclaration_A
 * @return {string} src
 * @param {glsUniformBlockCase.Uniform} uniform
 * @param {number} indentLevel
 */
glsUniformBlockCase.generateDeclaration_A = function(uniform, indentLevel) {
    /** @type {string} */ var src = '';

    if ((uniform.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_MASK) != 0)
        src += 'layout(' + glsUniformBlockCase.LayoutFlagsFmt(uniform.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_MASK) + ') ';

    src += glsUniformBlockCase.generateDeclaration_B(uniform.getType(), uniform.getName(), indentLevel, uniform.getFlags() & glsUniformBlockCase.UniformFlags.UNUSED_BOTH);

    return src;
};

/**
 * glsUniformBlockCase.generateDeclaration
 * @return {string} src
 * @param {glsUniformBlockCase.UniformBlock} block
 */
glsUniformBlockCase.generateDeclaration = function(block) {
    /** @type {string} */ var src = '';

    if ((block.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_MASK) != 0)
        src += 'layout(' + glsUniformBlockCase.LayoutFlagsFmt(block.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_MASK) + ') ';

    src += 'uniform ' + block.getBlockName();
    src += '\n {\n';

    for (var uniformNdx = 0; uniformNdx < block.countUniforms(); uniformNdx++) {
        src += glsUniformBlockCase.Indent(1);
        src += glsUniformBlockCase.generateDeclaration_A(block.getUniform(uniformNdx), 1 /* indent level */);
    }

    src += '}';

    if (block.getInstanceName() !== undefined) {
        src += ' ' + block.getInstanceName();
        if (block.isArray())
            src += '[' + block.getArraySize() + ']';
    } else
        DE_ASSERT(!block.isArray());

    src += ';\n';

    return src;
};

/**
 * glsUniformBlockCase.newArrayBufferFromView - Creates a new buffer copying data from a given view
 * @param {goog.NumberArray} view
 * @return {ArrayBuffer} The newly created buffer
 */
glsUniformBlockCase.newArrayBufferFromView = function(view) {
    var buffer = new ArrayBuffer(view.length * view.BYTES_PER_ELEMENT);
    var copyview;
    switch (view.BYTES_PER_ELEMENT) {
        case 1:
            copyview = new Uint8Array(buffer); break;
        case 2:
            copyview = new Uint16Array(buffer); break;
        case 4:
            copyview = new Uint32Array(buffer); break;
        default:
            assertMsgOptions(false, 'Unexpected value for BYTES_PER_ELEMENT in view', false, true);
    }
    for (var i = 0; i < view.length; i++)
        copyview[i] = view[i];

    return buffer;
};

/**
 * glsUniformBlockCase.generateValueSrc
 * @return {string} Used to be an output parameter in C++ project
 * @param {glsUniformBlockCase.UniformLayoutEntry} entry
 * @param {Uint8Array} basePtr
 * @param {number} elementNdx
 */
glsUniformBlockCase.generateValueSrc = function(entry, basePtr, elementNdx) {
    /** @type {string} */ var src = '';
    /** @type {gluShaderUtil.DataType} */ var scalarType = gluShaderUtil.getDataTypeScalarTypeAsDataType(entry.type);
    /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(entry.type);
    /** @type {boolean} */ var isArray = entry.size > 1;
    /** @type {Uint8Array} */ var elemPtr = basePtr.subarray(entry.offset + (isArray ? elementNdx * entry.arrayStride : 0));
    /** @type {number} */ var compSize = deMath.INT32_SIZE;
    /** @type {Uint8Array} */ var compPtr;
    if (scalarSize > 1)
        src += gluShaderUtil.getDataTypeName(entry.type) + '(';

    if (gluShaderUtil.isDataTypeMatrix(entry.type)) {
        /** @type {number} */ var numRows = gluShaderUtil.getDataTypeMatrixNumRows(entry.type);
        /** @type {number} */ var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(entry.type);

        DE_ASSERT(scalarType == gluShaderUtil.DataType.FLOAT);

        // Constructed in column-wise order.
        for (var colNdx = 0; colNdx < numCols; colNdx++) {
            for (var rowNdx = 0; rowNdx < numRows; rowNdx++) {
                compPtr = elemPtr.subarray(entry.isRowMajor ? rowNdx * entry.matrixStride + colNdx * compSize :
                                                                      colNdx * entry.matrixStride + rowNdx * compSize);

                if (colNdx > 0 || rowNdx > 0)
                    src += ', ';

                var newbuffer = new Uint8Array(compPtr.subarray(0, 4)).buffer;
                var newview = new DataView(newbuffer);
                src += parseFloat(newview.getFloat32(0, littleEndian)).toFixed(1);
            }
        }
    } else {
        for (var scalarNdx = 0; scalarNdx < scalarSize; scalarNdx++) {
            compPtr = elemPtr.subarray(scalarNdx * compSize);

            if (scalarNdx > 0)
                src += ', ';

            var newbuffer = glsUniformBlockCase.newArrayBufferFromView(compPtr.subarray(0, 4));
            var newview = new DataView(newbuffer);

            switch (scalarType) {
                case gluShaderUtil.DataType.FLOAT: src += parseFloat(newview.getFloat32(0, littleEndian) * 100 / 100).toFixed(1); break;
                case gluShaderUtil.DataType.INT: src += newview.getInt32(0, littleEndian); break;
                case gluShaderUtil.DataType.UINT: src += newview.getUint32(0, littleEndian) + 'u'; break;
                case gluShaderUtil.DataType.BOOL: src += (newview.getUint32(0, littleEndian) != 0 ? 'true' : 'false'); break;
                default:
                    DE_ASSERT(false);
            }
        }
    }

    if (scalarSize > 1)
        src += ')';

    return src;
};

/**
 * glsUniformBlockCase.generateCompareSrc_A
 * @return {string} Used to be an output parameter in C++ project
 * @param {string} resultVar
 * @param {glsUniformBlockCase.VarType} type
 * @param {string} srcName
 * @param {string} apiName
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {Uint8Array} basePtr
 * @param {number} unusedMask
 */
glsUniformBlockCase.generateCompareSrc_A = function(resultVar, type, srcName, apiName, layout, basePtr, unusedMask) {
    /** @type {string} */ var src = '';
    /** @type {string} */ var op;
    /** @type {glsUniformBlockCase.VarType|gluShaderUtil.DataType} */ var elementType;

    if (type.isBasicType() || (type.isArrayType() && type.getElementType().isBasicType())) {
        // Basic type or array of basic types.
        /** @type {boolean} */ var isArray = type.isArrayType();
        elementType = isArray ? type.getElementType().getBasicType() : type.getBasicType();
        /** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(elementType);
        /** @type {string} */ var fullApiName = apiName + (isArray ? '[0]' : ''); // Arrays are always postfixed with [0]
        /** @type {number} */ var uniformNdx = layout.getUniformIndex(fullApiName);
        /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var entry = layout.uniforms[uniformNdx];

        if (isArray) {
            for (var elemNdx = 0; elemNdx < type.getArraySize(); elemNdx++) {
                src += '\tresult *= compare_' + typeName + '(' + srcName + '[' + elemNdx + '], ';
                src += glsUniformBlockCase.generateValueSrc(entry, basePtr, elemNdx);
                src += ');\n';
            }
        } else {
            src += '\tresult *= compare_' + typeName + '(' + srcName + ', ';
            src += glsUniformBlockCase.generateValueSrc(entry, basePtr, 0);
            src += ');\n';
        }
    } else if (type.isArrayType()) {
        elementType = type.getElementType();

        for (var elementNdx = 0; elementNdx < type.getArraySize(); elementNdx++) {
            op = '[' + elementNdx + ']';
            src += glsUniformBlockCase.generateCompareSrc_A(resultVar, elementType, srcName + op, apiName + op, layout, basePtr, unusedMask);
        }
    } else {
        DE_ASSERT(type.isStructType());

        /** @type {glsUniformBlockCase.StructType} */ var stype = type.getStruct();
        for (var memberNdx = 0; memberNdx < stype.getSize(); memberNdx++) {
            /** @type {glsUniformBlockCase.StructMember} */ var memberIter = stype.getMember(memberNdx);
            if (memberIter.getFlags() & unusedMask)
                continue; // Skip member.

            op = '.' + memberIter.getName();
            src += glsUniformBlockCase.generateCompareSrc_A(resultVar, memberIter.getType(), srcName + op, apiName + op, layout, basePtr, unusedMask);
        }
    }

    return src;
};

/**
 * glsUniformBlockCase.generateCompareSrc
 * @return {string} Used to be an output parameter in C++ project
 * @param {string} resultVar
 * @param {glsUniformBlockCase.ShaderInterface} sinterface
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {glsUniformBlockCase.BlockPointers} blockPointers
 * @param {boolean} isVertex
 */
glsUniformBlockCase.generateCompareSrc = function(resultVar, sinterface, layout, blockPointers, isVertex) {
    /** @type {string} */ var src = '';
    /** @type {number} */ var unusedMask = isVertex ? glsUniformBlockCase.UniformFlags.UNUSED_VERTEX : glsUniformBlockCase.UniformFlags.UNUSED_FRAGMENT;

    for (var blockNdx = 0; blockNdx < sinterface.getNumUniformBlocks(); blockNdx++) {
        /** @type {glsUniformBlockCase.UniformBlock} */ var block = sinterface.getUniformBlock(blockNdx);

        if ((block.getFlags() & (isVertex ? glsUniformBlockCase.UniformFlags.DECLARE_VERTEX : glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT)) == 0)
            continue; // Skip.

        /** @type {boolean} */ var hasInstanceName = block.getInstanceName() !== undefined;
        /** @type {boolean} */ var isArray = block.isArray();
        /** @type {number} */ var numInstances = isArray ? block.getArraySize() : 1;
        /** @type {string} */ var apiPrefix = hasInstanceName ? block.getBlockName() + '.' : '';

        DE_ASSERT(!isArray || hasInstanceName);

        for (var instanceNdx = 0; instanceNdx < numInstances; instanceNdx++) {
            /** @type {string} */ var instancePostfix = isArray ? '[' + instanceNdx + ']' : '';
            /** @type {string} */ var blockInstanceName = block.getBlockName() + instancePostfix;
            /** @type {string} */ var srcPrefix = hasInstanceName ? block.getInstanceName() + instancePostfix + '.' : '';
            /** @type {number} */ var activeBlockNdx = layout.getBlockIndex(blockInstanceName);
            /** @type {Uint8Array} */ var basePtr = blockPointers.find(activeBlockNdx);

            for (var uniformNdx = 0; uniformNdx < block.countUniforms(); uniformNdx++) {
                /** @type {glsUniformBlockCase.Uniform} */ var uniform = block.getUniform(uniformNdx);

                if (uniform.getFlags() & unusedMask)
                    continue; // Don't read from that uniform.

                src += glsUniformBlockCase.generateCompareSrc_A(resultVar, uniform.getType(), srcPrefix + uniform.getName(), apiPrefix + uniform.getName(), layout, basePtr, unusedMask);
            }
        }
    }

    return src;
};

/**
 * glsUniformBlockCase.generateVertexShader
 * @return {string} src
 * @param {glsUniformBlockCase.ShaderInterface} sinterface
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {glsUniformBlockCase.BlockPointers} blockPointers
 */
glsUniformBlockCase.generateVertexShader = function(sinterface, layout, blockPointers) {
    /** @type {string} */ var src = '';

    DE_ASSERT(glsUniformBlockCase.isSupportedGLSLVersion(gluShaderUtil.getGLSLVersion(gl)));

    src += gluShaderUtil.getGLSLVersionDeclaration(gluShaderUtil.getGLSLVersion(gl)) + '\n';
    src += 'in highp vec4 a_position;\n';
    src += 'out mediump float v_vtxResult;\n';
    src += '\n';

    /** @type {Array<glsUniformBlockCase.StructType>} */ var namedStructs = [];
    sinterface.getNamedStructs(namedStructs);
    for (var structNdx = 0; structNdx < namedStructs.length; structNdx++)
        src += glsUniformBlockCase.generateDeclaration_C(namedStructs[structNdx], 0);

    for (var blockNdx = 0; blockNdx < sinterface.getNumUniformBlocks(); blockNdx++) {
        /** @type {glsUniformBlockCase.UniformBlock} */ var block = sinterface.getUniformBlock(blockNdx);
        if (block.getFlags() & glsUniformBlockCase.UniformFlags.DECLARE_VERTEX)
            src += glsUniformBlockCase.generateDeclaration(block);
    }

    // Comparison utilities.
    src += '\n';
    src += glsUniformBlockCase.generateCompareFuncs(sinterface);

    src += '\n' +
           'void main (void)\n' +
           ' {\n' +
           ' gl_Position = a_position;\n' +
           ' mediump float result = 1.0;\n';

    // Value compare.
    src += glsUniformBlockCase.generateCompareSrc('result', sinterface, layout, blockPointers, true);

    src += ' v_vtxResult = result;\n' +
           '}\n';

    return src;
};

/**
 * glsUniformBlockCase.generateFragmentShader
 * @return {string} Used to be an output parameter
 * @param {glsUniformBlockCase.ShaderInterface} sinterface
 * @param {glsUniformBlockCase.UniformLayout} layout
 * @param {glsUniformBlockCase.BlockPointers} blockPointers
 */
glsUniformBlockCase.generateFragmentShader = function(sinterface, layout, blockPointers) {
    /** @type {string} */ var src = '';
    DE_ASSERT(glsUniformBlockCase.isSupportedGLSLVersion(gluShaderUtil.getGLSLVersion(gl)));

    src += gluShaderUtil.getGLSLVersionDeclaration(gluShaderUtil.getGLSLVersion(gl)) + '\n';
    src += 'in mediump float v_vtxResult;\n';
    src += 'layout(location = 0) out mediump vec4 dEQP_FragColor;\n';
    src += '\n';

    /** @type {Array<glsUniformBlockCase.StructType>} */ var namedStructs = [];
    sinterface.getNamedStructs(namedStructs);
    for (var structNdx = 0; structNdx < namedStructs.length; structNdx++)
        src += glsUniformBlockCase.generateDeclaration_C(namedStructs[structNdx], 0);

    for (var blockNdx = 0; blockNdx < sinterface.getNumUniformBlocks(); blockNdx++) {
        /** @type {glsUniformBlockCase.UniformBlock} */ var block = sinterface.getUniformBlock(blockNdx);
        if (block.getFlags() & glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT)
            src += glsUniformBlockCase.generateDeclaration(block);
    }

    // Comparison utilities.
    src += '\n';
    src += glsUniformBlockCase.generateCompareFuncs(sinterface);

    src += '\n' +
           'void main (void)\n' +
           ' {\n' +
           ' mediump float result = 1.0;\n';

    // Value compare.
    src += glsUniformBlockCase.generateCompareSrc('result', sinterface, layout, blockPointers, false);

    src += ' dEQP_FragColor = vec4(1.0, v_vtxResult, result, 1.0);\n' +
           '}\n';

    return src;
};

/**
 * TODO: test glsUniformBlockCase.getGLUniformLayout Gets the uniform blocks and uniforms in the program.
 * @param {WebGL2RenderingContext} gl
 * @param {glsUniformBlockCase.UniformLayout} layout To store the layout described in program.
 * @param {WebGLProgram} program id
 */
glsUniformBlockCase.getGLUniformLayout = function(gl, layout, program) {
    /** @type {number} */ var numActiveUniforms = 0;
    /** @type {number} */ var numActiveBlocks = 0;

    numActiveUniforms = /** @type {number} */ (gl.getProgramParameter(program, gl.ACTIVE_UNIFORMS)); // ACTIVE_UNIFORM* returns GLInt
    numActiveBlocks = /** @type {number} */ (gl.getProgramParameter(program, gl.ACTIVE_UNIFORM_BLOCKS));

    /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var entryBlock;
    /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var entryUniform;
    /** @type {number} */ var size;
    /** @type {number} */ var nameLen;
    /** @type {string} */ var nameBuf;
    /** @type {number} */ var numBlockUniforms;

    // Block entries.
    //No need to allocate these beforehand: layout.blocks.resize(numActiveBlocks);
    for (var blockNdx = 0; blockNdx < numActiveBlocks; blockNdx++) {
        entryBlock = new glsUniformBlockCase.BlockLayoutEntry();

        size = /** @type {number} */ (gl.getActiveUniformBlockParameter(program, blockNdx, gl.UNIFORM_BLOCK_DATA_SIZE));
        // nameLen not used so this line is removed.
        // nameLen = gl.getActiveUniformBlockParameter(program, blockNdx, gl.UNIFORM_BLOCK_NAME_LENGTH); // TODO: UNIFORM_BLOCK_NAME_LENGTH is removed in WebGL2
        numBlockUniforms = /** @type {number} */ (gl.getActiveUniformBlockParameter(program, blockNdx, gl.UNIFORM_BLOCK_ACTIVE_UNIFORMS));

        nameBuf = gl.getActiveUniformBlockName(program, blockNdx);

        entryBlock.name = nameBuf;
        entryBlock.size = size;
        //entry.activeUniformIndices.resize(numBlockUniforms);

        if (numBlockUniforms > 0)
            entryBlock.activeUniformIndices = gl.getActiveUniformBlockParameter(program, blockNdx, gl.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES);

        layout.blocks.push(entryBlock); //Pushing the block into the array here.
    }

    if (numActiveUniforms > 0) {
        // glsUniformBlockCase.Uniform entries.
        /** @type {Array<number>} */ var uniformIndices = [];
        for (var i = 0; i < numActiveUniforms; i++)
            uniformIndices.push(i);

        /** @type {Array<number>} */ var types = [];
        /** @type {Array<number>} */ var sizes = [];
        /** @type {Array<number>} */ var nameLengths = [];
        /** @type {Array<number>} */ var blockIndices = [];
        /** @type {Array<number>} */ var offsets = [];
        /** @type {Array<number>} */ var arrayStrides = [];
        /** @type {Array<number>} */ var matrixStrides = [];
        /** @type {Array<number>} */ var rowMajorFlags = [];

        // Execute queries.
        types = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_TYPE);
        sizes = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_SIZE);
        // Remove this: nameLengths = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_NAME_LENGTH);
        blockIndices = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_BLOCK_INDEX);
        offsets = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_OFFSET);
        arrayStrides = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_ARRAY_STRIDE);
        matrixStrides = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_MATRIX_STRIDE);
        rowMajorFlags = gl.getActiveUniforms(program, uniformIndices, gl.UNIFORM_IS_ROW_MAJOR);

        // Translate to LayoutEntries
        // No resize needed. Will push them: layout.uniforms.resize(numActiveUniforms);
        for (var uniformNdx = 0; uniformNdx < numActiveUniforms; uniformNdx++) {
            entryUniform = new glsUniformBlockCase.UniformLayoutEntry();

            // Remove this: nameLen = 0;
            size = 0;
            /** @type {number} */ var type = gl.NONE;

            var uniform = gl.getActiveUniform(program, uniformNdx);

            nameBuf = uniform.name;
            // Remove this: nameLen = nameBuf.length;
            size = uniform.size;
            type = uniform.type;

            // Remove this: nameLen != nameLengths[uniformNdx] ||
            if (size != sizes[uniformNdx] ||
                type != types[uniformNdx])
                testFailedOptions("Values returned by gl.getActiveUniform() don't match with values queried with gl.getActiveUniforms().", true);

            entryUniform.name = nameBuf;
            entryUniform.type = gluShaderUtil.getDataTypeFromGLType(types[uniformNdx]);
            entryUniform.size = sizes[uniformNdx];
            entryUniform.blockNdx = blockIndices[uniformNdx];
            entryUniform.offset = offsets[uniformNdx];
            entryUniform.arrayStride = arrayStrides[uniformNdx];
            entryUniform.matrixStride = matrixStrides[uniformNdx];
            entryUniform.isRowMajor = rowMajorFlags[uniformNdx] != false;

            layout.uniforms.push(entryUniform); //Pushing this uniform in the end.
        }
    }
};

/**
 * glsUniformBlockCase.copyUniformData_A - Copies a source uniform buffer segment to a destination uniform buffer segment.
 * @param {glsUniformBlockCase.UniformLayoutEntry} dstEntry
 * @param {Uint8Array} dstBlockPtr
 * @param {glsUniformBlockCase.UniformLayoutEntry} srcEntry
 * @param {Uint8Array} srcBlockPtr
 */
glsUniformBlockCase.copyUniformData_A = function(dstEntry, dstBlockPtr, srcEntry, srcBlockPtr) {
    /** @type {Uint8Array} */ var dstBasePtr = dstBlockPtr.subarray(dstEntry.offset);
    /** @type {Uint8Array} */ var srcBasePtr = srcBlockPtr.subarray(srcEntry.offset);

    DE_ASSERT(dstEntry.size <= srcEntry.size);
    DE_ASSERT(dstEntry.type == srcEntry.type);

    /** @type {number} */ var scalarSize = gluShaderUtil.getDataTypeScalarSize(dstEntry.type);
    /** @type {boolean} */ var isMatrix = gluShaderUtil.isDataTypeMatrix(dstEntry.type);
    /** @type {number} */ var compSize = deMath.INT32_SIZE;

    for (var elementNdx = 0; elementNdx < dstEntry.size; elementNdx++) {
        /** @type {Uint8Array} */ var dstElemPtr = dstBasePtr.subarray(elementNdx * dstEntry.arrayStride);
        /** @type {Uint8Array} */ var srcElemPtr = srcBasePtr.subarray(elementNdx * srcEntry.arrayStride);

        if (isMatrix) {
            /** @type {number} */ var numRows = gluShaderUtil.getDataTypeMatrixNumRows(dstEntry.type);
            /** @type {number} */ var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(dstEntry.type);

            for (var colNdx = 0; colNdx < numCols; colNdx++) {
                for (var rowNdx = 0; rowNdx < numRows; rowNdx++) {
                    var srcoffset = dstEntry.isRowMajor ? rowNdx * dstEntry.matrixStride + colNdx * compSize :
                                    colNdx * dstEntry.matrixStride + rowNdx * compSize;
                    /** @type {Uint8Array} */ var dstCompPtr = dstElemPtr.subarray(srcoffset, srcoffset + compSize);
                    var dstoffset = srcEntry.isRowMajor ? rowNdx * srcEntry.matrixStride + colNdx * compSize :
                                    colNdx * srcEntry.matrixStride + rowNdx * compSize;
                    /** @type {Uint8Array} */ var srcCompPtr = srcElemPtr.subarray(dstoffset, dstoffset + compSize);

                    //Copy byte per byte
                    for (var i = 0; i < compSize; i++)
                        dstCompPtr[i] = srcCompPtr[i];
                }
            }
        } else
            //Copy byte per byte
            for (var i = 0; i < scalarSize * compSize; i++)
                dstElemPtr[i] = srcElemPtr[i];
    }
};

/**
 * glsUniformBlockCase.copyUniformData - Copies a source uniform buffer to a destination uniform buffer.
 * @param {glsUniformBlockCase.UniformLayout} dstLayout
 * @param {glsUniformBlockCase.BlockPointers} dstBlockPointers
 * @param {glsUniformBlockCase.UniformLayout} srcLayout
 * @param {glsUniformBlockCase.BlockPointers} srcBlockPointers
 */
glsUniformBlockCase.copyUniformData = function(dstLayout, dstBlockPointers, srcLayout, srcBlockPointers) {
    // \note Src layout is used as reference in case of activeUniforms happens to be incorrect in dstLayout blocks.
    /** @type {number} */ var numBlocks = srcLayout.blocks.length;

    for (var srcBlockNdx = 0; srcBlockNdx < numBlocks; srcBlockNdx++) {
        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var srcBlock = srcLayout.blocks[srcBlockNdx];
        /** @type {Uint8Array} */ var srcBlockPtr = srcBlockPointers.find(srcBlockNdx);
        /** @type {number} */ var dstBlockNdx = dstLayout.getBlockIndex(srcBlock.name);
        /** @type {Uint8Array} */ var dstBlockPtr = dstBlockNdx >= 0 ? dstBlockPointers.find(dstBlockNdx) : null;

        if (dstBlockNdx < 0)
            continue;

        for (var srcUniformNdx = 0; srcUniformNdx < srcBlock.activeUniformIndices.length; srcUniformNdx++) {
            /** @type {number} */ var srcUniformNdxIter = srcBlock.activeUniformIndices[srcUniformNdx];
            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var srcEntry = srcLayout.uniforms[srcUniformNdxIter];
            /** @type {number} */ var dstUniformNdx = dstLayout.getUniformIndex(srcEntry.name);

            if (dstUniformNdx < 0)
                continue;

            glsUniformBlockCase.copyUniformData_A(dstLayout.uniforms[dstUniformNdx], dstBlockPtr, srcEntry, srcBlockPtr);
        }
    }
};

 /**
  * TODO: Test with an actual WebGL 2.0 context
  * iterate - The actual execution of the test.
  * @return {tcuTestCase.IterateResult}
  */
 glsUniformBlockCase.UniformBlockCase.prototype.iterate = function() {
    /** @type {glsUniformBlockCase.UniformLayout} */ var refLayout = new glsUniformBlockCase.UniformLayout(); //!< std140 layout.
    /** @type {glsUniformBlockCase.BlockPointers} */ var blockPointers = new glsUniformBlockCase.BlockPointers();

    // Compute reference layout.
    glsUniformBlockCase.computeStd140Layout(refLayout, this.m_interface);

    // Assign storage for reference values.
    /** @type {number} */ var totalSize = 0;
    for (var blockNdx = 0; blockNdx < refLayout.blocks.length; blockNdx++) {
        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var blockIter = refLayout.blocks[blockNdx];
        totalSize += blockIter.size;
    }
    blockPointers.resize(totalSize);

    // Pointers for each block.
    var curOffset = 0;
    for (var blockNdx = 0; blockNdx < refLayout.blocks.length; blockNdx++) {
        var size = refLayout.blocks[blockNdx].size;
        blockPointers.push(curOffset, size);
        curOffset += size;
    }

    // Generate values.
    glsUniformBlockCase.generateValues(refLayout, blockPointers, 1 /* seed */);

    // Generate shaders and build program.
    /** @type {string} */ var vtxSrc = glsUniformBlockCase.generateVertexShader(this.m_interface, refLayout, blockPointers);
    /** @type {string} */ var fragSrc = glsUniformBlockCase.generateFragmentShader(this.m_interface, refLayout, blockPointers);

    /** @type {gluShaderProgram.ShaderProgram}*/ var program = new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vtxSrc, fragSrc));
    bufferedLogToConsole(program.getProgramInfo().infoLog);

    if (!program.isOk()) {
        // Compile failed.
        testFailedOptions('Compile failed', false);
        return tcuTestCase.IterateResult.STOP;
    }

    // Query layout from GL.
    /** @type {glsUniformBlockCase.UniformLayout} */ var glLayout = new glsUniformBlockCase.UniformLayout();
    glsUniformBlockCase.getGLUniformLayout(gl, glLayout, program.getProgram());

    // Print layout to log.
    bufferedLogToConsole('Active glsUniformBlockCase.Uniform Blocks');
    for (var blockNdx = 0; blockNdx < glLayout.blocks.length; blockNdx++)
        bufferedLogToConsole(blockNdx + ': ' + glLayout.blocks[blockNdx]);

    bufferedLogToConsole('Active Uniforms');
     for (var uniformNdx = 0; uniformNdx < glLayout.uniforms.length; uniformNdx++)
         bufferedLogToConsole(uniformNdx + ': ' + glLayout.uniforms[uniformNdx]);

    // Check that we can even try rendering with given layout.
    if (!this.checkLayoutIndices(glLayout) || !this.checkLayoutBounds(glLayout) || !this.compareTypes(refLayout, glLayout)) {
        testFailedOptions('Invalid layout', false);
        return tcuTestCase.IterateResult.STOP; // It is not safe to use the given layout.
    }

    // Verify all std140 blocks.
    if (!this.compareStd140Blocks(refLayout, glLayout))
        testFailedOptions('Invalid std140 layout', false);

    // Verify all shared blocks - all uniforms should be active, and certain properties match.
    if (!this.compareSharedBlocks(refLayout, glLayout))
        testFailedOptions('Invalid shared layout', false);

    // Check consistency with index queries
    if (!this.checkIndexQueries(program.getProgram(), glLayout))
        testFailedOptions('Inconsintent block index query results', false);

    // Use program.
    gl.useProgram(program.getProgram());

    /** @type {number} */ var binding;
    /** @type {WebGLBuffer} */ var buffer;

    // Assign binding points to all active uniform blocks.
    for (var blockNdx = 0; blockNdx < glLayout.blocks.length; blockNdx++) {
        binding = blockNdx; // \todo [2012-01-25 pyry] Randomize order?
        gl.uniformBlockBinding(program.getProgram(), blockNdx, binding);
    }

    /** @type {number} */ var numBlocks;
    /** @type {glsUniformBlockCase.BlockPointers} */ var glBlockPointers;

    // Allocate buffers, write data and bind to targets.
    /** @type {glsUniformBlockCase.UniformBufferManager} */ var bufferManager = new glsUniformBlockCase.UniformBufferManager(gl);
    if (this.m_bufferMode == glsUniformBlockCase.BufferMode.BUFFERMODE_PER_BLOCK) {
        numBlocks = glLayout.blocks.length;
        glBlockPointers = new glsUniformBlockCase.BlockPointers();

        var totalsize = 0;
        for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++)
            totalsize += glLayout.blocks[blockNdx].size;

        glBlockPointers.resize(totalsize);

        var offset = 0;
        for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
            glBlockPointers.push(offset, glLayout.blocks[blockNdx].size);
            offset += glLayout.blocks[blockNdx].size;
        }

        glsUniformBlockCase.copyUniformData(glLayout, glBlockPointers, refLayout, blockPointers);

        for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
            buffer = bufferManager.allocBuffer();
            binding = blockNdx;
            gl.bindBuffer(gl.UNIFORM_BUFFER, buffer);
            gl.bufferData(gl.UNIFORM_BUFFER, glBlockPointers.find(blockNdx) /*(glw::GLsizeiptr)glData[blockNdx].size(), &glData[blockNdx][0]*/, gl.STATIC_DRAW);
            gl.bindBufferBase(gl.UNIFORM_BUFFER, binding, buffer);
        }
    } else {
        DE_ASSERT(this.m_bufferMode == glsUniformBlockCase.BufferMode.BUFFERMODE_SINGLE);

        totalSize = 0;
        curOffset = 0;
        numBlocks = glLayout.blocks.length;
        /** @type {number} */ var bindingAlignment = 0;
        glBlockPointers = new glsUniformBlockCase.BlockPointers();

        bindingAlignment = /** @type {number} */ (gl.getParameter(gl.UNIFORM_BUFFER_OFFSET_ALIGNMENT));

        // Compute total size and offsets.
        curOffset = 0;
        for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
            if (bindingAlignment > 0)
                curOffset = glsUniformBlockCase.deRoundUp32(curOffset, bindingAlignment);
            glBlockPointers.push(curOffset, glLayout.blocks[blockNdx].size);
            curOffset += glLayout.blocks[blockNdx].size;
        }
        totalSize = curOffset;
        glBlockPointers.resize(totalSize);

        // Copy to gl format.
        glsUniformBlockCase.copyUniformData(glLayout, glBlockPointers, refLayout, blockPointers);

        // Allocate buffer and upload data.
        buffer = bufferManager.allocBuffer();
        gl.bindBuffer(gl.UNIFORM_BUFFER, buffer);
        if (glBlockPointers.data.byteLength > 0 /*!glData.empty()*/)
            gl.bufferData(gl.UNIFORM_BUFFER, glBlockPointers.find(blockNdx) /*(glw::GLsizeiptr)glData.size(), &glData[0]*/, gl.STATIC_DRAW);

        // Bind ranges to binding points.
        for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
            binding = blockNdx;
            gl.bindBufferRange(gl.UNIFORM_BUFFER, binding, buffer, glBlockPointers.offsets[blockNdx], glLayout.blocks[blockNdx].size);
        }
    }

    /** @type {boolean} */ var renderOk = this.render(program);
    if (!renderOk)
        testFailedOptions('Image compare failed', false);
    else
        assertMsgOptions(renderOk, '', true, false);

    return tcuTestCase.IterateResult.STOP;
};

/**
* compareStd140Blocks
* @param {glsUniformBlockCase.UniformLayout} refLayout
* @param {glsUniformBlockCase.UniformLayout} cmpLayout
**/
glsUniformBlockCase.UniformBlockCase.prototype.compareStd140Blocks = function(refLayout, cmpLayout) {
    /**@type {boolean} */ var isOk = true;
    /**@type {number} */ var numBlocks = this.m_interface.getNumUniformBlocks();

    for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
        /**@type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.getUniformBlock(blockNdx);
        /**@type {boolean} */ var isArray = block.isArray();
        /**@type {string} */ var instanceName = block.getBlockName() + (isArray ? '[0]' : '');
        /**@type {number} */ var refBlockNdx = refLayout.getBlockIndex(instanceName);
        /**@type {number} */ var cmpBlockNdx = cmpLayout.getBlockIndex(instanceName);
        /**@type {boolean} */ var isUsed = (block.getFlags() & (glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT)) != 0;

        if ((block.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_STD140) == 0)
            continue; // Not std140 layout.

        DE_ASSERT(refBlockNdx >= 0);

        if (cmpBlockNdx < 0) {
            // Not found, should it?
            if (isUsed) {
                bufferedLogToConsole("Error: glsUniformBlockCase.Uniform block '" + instanceName + "' not found");
                isOk = false;
            }

            continue; // Skip block.
        }

        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var refBlockLayout = refLayout.blocks[refBlockNdx];
        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var cmpBlockLayout = cmpLayout.blocks[cmpBlockNdx];

        // \todo [2012-01-24 pyry] Verify that activeUniformIndices is correct.
        // \todo [2012-01-24 pyry] Verify all instances.
        if (refBlockLayout.activeUniformIndices.length != cmpBlockLayout.activeUniformIndices.length) {
            bufferedLogToConsole("Error: Number of active uniforms differ in block '" + instanceName +
                "' (expected " + refBlockLayout.activeUniformIndices.length +
                ', got ' + cmpBlockLayout.activeUniformIndices.length +
                ')');
            isOk = false;
        }

        for (var ndx = 0; ndx < refBlockLayout.activeUniformIndices.length; ndx++) {
            /** @type {number} */ var ndxIter = refBlockLayout.activeUniformIndices[ndx];
            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var refEntry = refLayout.uniforms[ndxIter];
            /** @type {number} */ var cmpEntryNdx = cmpLayout.getUniformIndex(refEntry.name);

            if (cmpEntryNdx < 0) {
                bufferedLogToConsole("Error: glsUniformBlockCase.Uniform '" + refEntry.name + "' not found");
                isOk = false;
                continue;
            }

            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var cmpEntry = cmpLayout.uniforms[cmpEntryNdx];

            if (refEntry.type != cmpEntry.type ||
                refEntry.size != cmpEntry.size ||
                refEntry.offset != cmpEntry.offset ||
                refEntry.arrayStride != cmpEntry.arrayStride ||
                refEntry.matrixStride != cmpEntry.matrixStride ||
                refEntry.isRowMajor != cmpEntry.isRowMajor) {
                bufferedLogToConsole("Error: Layout mismatch in '" + refEntry.name + "':\n" +
                ' expected: type = ' + gluShaderUtil.getDataTypeName(refEntry.type) + ', size = ' + refEntry.size + ', row major = ' + (refEntry.isRowMajor ? 'true' : 'false') + '\n' +
                ' got: type = ' + gluShaderUtil.getDataTypeName(cmpEntry.type) + ', size = ' + cmpEntry.size + ', row major = ' + (cmpEntry.isRowMajor ? 'true' : 'false'));
                isOk = false;
            }
        }
    }

    return isOk;
};

/**
* compareSharedBlocks
* @param {glsUniformBlockCase.UniformLayout} refLayout
* @param {glsUniformBlockCase.UniformLayout} cmpLayout
**/
glsUniformBlockCase.UniformBlockCase.prototype.compareSharedBlocks = function(refLayout, cmpLayout) {
    /** @type {boolean} */ var isOk = true;
    /** @type {number} */ var numBlocks = this.m_interface.getNumUniformBlocks();

    for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
        /** @type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.getUniformBlock(blockNdx);
        /** @type {boolean} */ var isArray = block.isArray();
        /** @type {string} */ var instanceName = block.getBlockName() + (isArray ? '[0]' : '');
        /** @type {number} */ var refBlockNdx = refLayout.getBlockIndex(instanceName);
        /** @type {number} */ var cmpBlockNdx = cmpLayout.getBlockIndex(instanceName);
        /** @type {boolean} */ var isUsed = (block.getFlags() & (glsUniformBlockCase.UniformFlags.DECLARE_VERTEX | glsUniformBlockCase.UniformFlags.DECLARE_FRAGMENT)) != 0;

        if ((block.getFlags() & glsUniformBlockCase.UniformFlags.LAYOUT_SHARED) == 0)
            continue; // Not shared layout.

        DE_ASSERT(refBlockNdx >= 0);

        if (cmpBlockNdx < 0) {
            // Not found, should it?
            if (isUsed) {
                bufferedLogToConsole("Error: glsUniformBlockCase.Uniform block '" + instanceName + "' not found");
                isOk = false;
            }

            continue; // Skip block.
        }

        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var refBlockLayout = refLayout.blocks[refBlockNdx];
        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var cmpBlockLayout = cmpLayout.blocks[cmpBlockNdx];

        if (refBlockLayout.activeUniformIndices.length != cmpBlockLayout.activeUniformIndices.length) {
            bufferedLogToConsole("Error: Number of active uniforms differ in block '" + instanceName +
                "' (expected " + refBlockLayout.activeUniformIndices.length +
                ', got ' + cmpBlockLayout.activeUniformIndices.length +
                ')');
            isOk = false;
        }

        for (var ndx = 0; ndx < refBlockLayout.activeUniformIndices.length; ndx++) {
            /** @type {number} */ var ndxIter = refBlockLayout.activeUniformIndices[ndx];
            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var refEntry = refLayout.uniforms[ndxIter];
            /** @type {number} */ var cmpEntryNdx = cmpLayout.getUniformIndex(refEntry.name);

            if (cmpEntryNdx < 0) {
                bufferedLogToConsole("Error: glsUniformBlockCase.Uniform '" + refEntry.name + "' not found");
                isOk = false;
                continue;
            }

            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var cmpEntry = cmpLayout.uniforms[cmpEntryNdx];

            if (refEntry.type != cmpEntry.type ||
                refEntry.size != cmpEntry.size ||
                refEntry.isRowMajor != cmpEntry.isRowMajor) {
                bufferedLogToConsole("Error: Layout mismatch in '" + refEntry.name + "':\n" +
                ' expected: type = ' + gluShaderUtil.getDataTypeName(refEntry.type) + ', size = ' + refEntry.size + ', row major = ' + (refEntry.isRowMajor ? 'true' : 'false') + '\n' +
                ' got: type = ' + gluShaderUtil.getDataTypeName(cmpEntry.type) + ', size = ' + cmpEntry.size + ', row major = ' + (cmpEntry.isRowMajor ? 'true' : 'false'));
                isOk = false;
            }
        }
    }

    return isOk;
};

/** compareTypes
* @param {glsUniformBlockCase.UniformLayout} refLayout
* @param {glsUniformBlockCase.UniformLayout} cmpLayout
* @return {boolean} true if uniform types are the same
**/
glsUniformBlockCase.UniformBlockCase.prototype.compareTypes = function(refLayout, cmpLayout) {
    /** @type {boolean} */ var isOk = true;
    /** @type {number} */ var numBlocks = this.m_interface.getNumUniformBlocks();

    for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
        /** @type {glsUniformBlockCase.UniformBlock} */ var block = this.m_interface.getUniformBlock(blockNdx);
        /** @type {boolean} */ var isArray = block.isArray();
        /** @type {number} */ var numInstances = isArray ? block.getArraySize() : 1;

        for (var instanceNdx = 0; instanceNdx < numInstances; instanceNdx++) {
            /** @type {string} */ var instanceName;

            instanceName += block.getBlockName();
            if (isArray)
                instanceName = instanceName + '[' + instanceNdx + ']';

            /** @type {number} */ var cmpBlockNdx = cmpLayout.getBlockIndex(instanceName);

            if (cmpBlockNdx < 0)
                continue;

            /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var cmpBlockLayout = cmpLayout.blocks[cmpBlockNdx];

            for (var ndx = 0; ndx < cmpBlockLayout.activeUniformIndices.length; ndx++) {
                /** @type {number} */ var ndxIter = cmpBlockLayout.activeUniformIndices[ndx];
                /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var cmpEntry = cmpLayout.uniforms[ndxIter];
                /** @type {number} */ var refEntryNdx = refLayout.getUniformIndex(cmpEntry.name);

                if (refEntryNdx < 0) {
                    bufferedLogToConsole("Error: glsUniformBlockCase.Uniform '" + cmpEntry.name + "' not found in reference layout");
                    isOk = false;
                    continue;
                }

                /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var refEntry = refLayout.uniforms[refEntryNdx];

                // \todo [2012-11-26 pyry] Should we check other properties as well?
                if (refEntry.type != cmpEntry.type) {
                    bufferedLogToConsole("Error: glsUniformBlockCase.Uniform type mismatch in '" + refEntry.name + "':</br>" +
                        "' expected: '" + gluShaderUtil.getDataTypeName(refEntry.type) + "'</br>" +
                        "' got: '" + gluShaderUtil.getDataTypeName(cmpEntry.type) + "'");
                    isOk = false;
                }
            }
        }
    }

    return isOk;
};

/** checkLayoutIndices
* @param {glsUniformBlockCase.UniformLayout} layout Layout whose indices are to be checked
* @return {boolean} true if all is ok
**/
glsUniformBlockCase.UniformBlockCase.prototype.checkLayoutIndices = function(layout) {
    /** @type {number} */ var numUniforms = layout.uniforms.length;
    /** @type {number} */ var numBlocks = layout.blocks.length;
    /** @type {boolean} */ var isOk = true;

    // Check uniform block indices.
    for (var uniformNdx = 0; uniformNdx < numUniforms; uniformNdx++) {
        /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var uniform = layout.uniforms[uniformNdx];

        if (uniform.blockNdx < 0 || !deMath.deInBounds32(uniform.blockNdx, 0, numBlocks)) {
            bufferedLogToConsole("Error: Invalid block index in uniform '" + uniform.name + "'");
            isOk = false;
        }
    }

    // Check active uniforms.
    for (var blockNdx = 0; blockNdx < numBlocks; blockNdx++) {
        /** @type {glsUniformBlockCase.BlockLayoutEntry} */ var block = layout.blocks[blockNdx];

        for (var uniformNdx = 0; uniformNdx < block.activeUniformIndices.length; uniformNdx++) {
            /** @type {glsUniformBlockCase.UniformLayoutEntry} */ var activeUniformNdx = block.activeUniformIndices[uniformNdx];
            if (!deMath.deInBounds32(activeUniformNdx, 0, numUniforms)) {
                bufferedLogToConsole('Error: Invalid active uniform index ' + activeUniformNdx + " in block '" + block.name);
                isOk = false;
            }
        }
    }
    return isOk;
};

/** checkLayoutBounds
* @param {glsUniformBlockCase.UniformLayout} layout The uniform layout to check
* @return {boolean} true if all is within bounds
**/
glsUniformBlockCase.UniformBlockCase.prototype.checkLayoutBounds = function(layout) {
    /** @type {number} */ var numUniforms = layout.uniforms.length;
    /** @type {boolean}*/ var isOk = true;

    for (var uniformNdx = 0; uniformNdx < numUniforms; uniformNdx++) {
        /** @type {glsUniformBlockCase.UniformLayoutEntry}*/ var uniform = layout.uniforms[uniformNdx];

        if (uniform.blockNdx < 0)
            continue;

        /** @type {glsUniformBlockCase.BlockLayoutEntry}*/ var block = layout.blocks[uniform.blockNdx];
        /** @type {boolean}*/ var isMatrix = gluShaderUtil.isDataTypeMatrix(uniform.type);
        /** @type {number}*/ var numVecs = isMatrix ? (uniform.isRowMajor ? gluShaderUtil.getDataTypeMatrixNumRows(uniform.type) : gluShaderUtil.getDataTypeMatrixNumColumns(uniform.type)) : 1;
        /** @type {number}*/ var numComps = isMatrix ? (uniform.isRowMajor ? gluShaderUtil.getDataTypeMatrixNumColumns(uniform.type) : gluShaderUtil.getDataTypeMatrixNumRows(uniform.type)) : gluShaderUtil.getDataTypeScalarSize(uniform.type);
        /** @type {number}*/ var numElements = uniform.size;
        /** @type {number}*/ var compSize = deMath.INT32_SIZE;
        /** @type {number}*/ var vecSize = numComps * compSize;

        /** @type {number}*/ var minOffset = 0;
        /** @type {number}*/ var maxOffset = 0;

        // For negative strides.
        minOffset = Math.min(minOffset, (numVecs - 1) * uniform.matrixStride);
        minOffset = Math.min(minOffset, (numElements - 1) * uniform.arrayStride);
        minOffset = Math.min(minOffset, (numElements - 1) * uniform.arrayStride + (numVecs - 1) * uniform.matrixStride);

        maxOffset = Math.max(maxOffset, vecSize);
        maxOffset = Math.max(maxOffset, (numVecs - 1) * uniform.matrixStride + vecSize);
        maxOffset = Math.max(maxOffset, (numElements - 1) * uniform.arrayStride + vecSize);
        maxOffset = Math.max(maxOffset, (numElements - 1) * uniform.arrayStride + (numVecs - 1) * uniform.matrixStride + vecSize);

        if (uniform.offset + minOffset < 0 || uniform.offset + maxOffset > block.size) {
            bufferedLogToConsole("Error: glsUniformBlockCase.Uniform '" + uniform.name + "' out of block bounds");
            isOk = false;
        }
    }

    return isOk;
};

/** checkIndexQueries
* @param {WebGLProgram} program The shader program to be checked against
* @param {glsUniformBlockCase.UniformLayout} layout The layout to check
* @return {boolean} true if everything matches.
**/
glsUniformBlockCase.UniformBlockCase.prototype.checkIndexQueries = function(program, layout) {
    /** @type {boolean}*/ var allOk = true;

    // \note Spec mandates that uniform blocks are assigned consecutive locations from 0
    //       to ACTIVE_UNIFORM_BLOCKS. BlockLayoutEntries are stored in that order in glsUniformBlockCase.UniformLayout.
    for (var blockNdx = 0; blockNdx < layout.blocks.length; blockNdx++) {
        /** @const */ var block = layout.blocks[blockNdx];
        /** @const */ var queriedNdx = gl.getUniformBlockIndex(program, block.name);

        if (queriedNdx != blockNdx) {
            bufferedLogToConsole('ERROR: glGetUniformBlockIndex(' + block.name + ') returned ' + queriedNdx + ', expected ' + blockNdx + '!');
            allOk = false;
        }
    }

    return allOk;
};

/** @const @type {number} */ glsUniformBlockCase.VIEWPORT_WIDTH = 128;
/** @const @type {number} */ glsUniformBlockCase.VIEWPORT_HEIGHT = 128;

/** Renders a white square, and then tests all pixels are
* effectively white in the color buffer.
* @param {gluShaderProgram.ShaderProgram} program The shader program to use.
* @return {boolean} false if there was at least one incorrect pixel
**/
glsUniformBlockCase.UniformBlockCase.prototype.render = function(program) {
    // Compute viewport.
    /** @type {deRandom.Random} */ var rnd = new deRandom.Random(deString.deStringHash(this.name));
    /** @const */ var viewportW = Math.min(gl.canvas.width, glsUniformBlockCase.VIEWPORT_WIDTH);
    /** @const */ var viewportH = Math.min(gl.canvas.height, glsUniformBlockCase.VIEWPORT_HEIGHT);
    /** @const */ var viewportX = rnd.getInt(0, gl.canvas.width);
    /** @const */ var viewportY = rnd.getInt(0, gl.canvas.height);

    gl.clearColor(0.125, 0.25, 0.5, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

    //Draw
    var position = [
        -1.0, -1.0, 0.0, 1.0,
        -1.0, 1.0, 0.0, 1.0,
        1.0, -1.0, 0.0, 1.0,
        1.0, 1.0, 0.0, 1.0
        ];
    var indices = [0, 1, 2, 2, 1, 3];

    gl.viewport(viewportX, viewportY, viewportW, viewportH);

    // Access
    var posLoc = gl.getAttribLocation(program.getProgram(), 'a_position');
    var posArray = [new gluDrawUtil.VertexArrayBinding(gl.FLOAT, posLoc, 4, 4, position)];
    gluDrawUtil.draw(gl, program.getProgram(), posArray, gluDrawUtil.triangles(indices));

    // Verify that all pixels are white.
    var pixels = new gluDrawUtil.Surface();
    var numFailedPixels = 0;

    var readPixelsX = (viewportX + viewportW) > gl.canvas.width
        ? (gl.canvas.width - viewportX) : viewportW;
    var readPixelsY = (viewportY + viewportH) > gl.canvas.height
        ? (gl.canvas.height - viewportY) : viewportH;

    var buffer = pixels.readSurface(gl, viewportX, viewportY, readPixelsX, readPixelsY);

    var whitePixel = new gluDrawUtil.Pixel([255.0, 255.0, 255.0, 255.0]);
    for (var y = 0; y < readPixelsY; y++) {
        for (var x = 0; x < readPixelsX; x++) {
            if (!pixels.getPixel(x, y).equals(whitePixel))
                numFailedPixels += 1;
        }
    }

    if (numFailedPixels > 0) {
        bufferedLogToConsole('Image comparison failed, got ' + numFailedPixels + ' non-white pixels.');
    }

    return numFailedPixels == 0;
};

});
