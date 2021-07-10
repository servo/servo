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
goog.provide('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {

var gluDrawUtil = framework.opengl.gluDrawUtil;
var gluShaderProgram = framework.opengl.gluShaderProgram;

/**
 * Description of a vertex array binding
 * @constructor
 * @param {number} type GL gluDrawUtil.Type of data
 * @param {(number|undefined)} location Binding location
 * @param {number} components Number of components per vertex
 * @param {number} elements Number of elements in the array
 * @param {Array<number>} data Source data
 * @param {number=} stride
 * @param {number=} offset
 */
gluDrawUtil.VertexArrayBinding = function(type, location, components, elements, data, stride, offset) {
    this.type = type;
    this.location = location === undefined ? -1 : location;
    this.components = components;
    this.elements = elements;
    this.data = data;
    /** @type {?string} */ this.name = null;
    this.stride = stride || 0;
    this.offset = offset || 0;
};

/**
 * Description of a vertex array binding
 * @param {gluDrawUtil.BindingPoint} binding
 * @param {gluDrawUtil.VertexArrayPointer} pointer
 * @param {number=} dataType GL Data Type
 * @return {gluDrawUtil.VertexArrayBinding}
 */
gluDrawUtil.vabFromBindingPointAndArrayPointer = function(binding, pointer, dataType) {
    var type = dataType === undefined ? gl.FLOAT : dataType;
    var location = binding.location;
    var components = pointer.numComponents;
    var elements = pointer.numElements;
    var data = pointer.data;
    var vaBinding = new gluDrawUtil.VertexArrayBinding(type, location, components, elements, data);
    vaBinding.componentType = pointer.componentType;
    vaBinding.name = binding.name;
    vaBinding.convert = pointer.convert;
    vaBinding.stride = pointer.stride;
    return vaBinding;
};

/**
 * ! Lower named bindings to locations and eliminate bindings that are not used by program.
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {WebGLProgram} program
 * @param {Array} inputArray - Array with the named binding locations
 * @param {Array=} outputArray - Array with the lowered locations
 * @return {Array} outputArray
 */
gluDrawUtil.namedBindingsToProgramLocations = function(gl, program, inputArray, outputArray) {
    outputArray = outputArray || [];

    for (var i = 0; i < inputArray.length; i++) {
        var cur = inputArray[i];
        if (cur.name) {
            //assert(binding.location >= 0);
            var location = gl.getAttribLocation(program, cur.name);
            if (location >= 0) {
                if (cur.location >= 0)
                    location += cur.location;
                // Add binding.location as an offset to accomodate matrices.
                outputArray.push(new gluDrawUtil.VertexArrayBinding(cur.type, location, cur.components, cur.elements, cur.data, cur.stride, cur.offset));
            }
        } else {
            outputArray.push(cur);
        }
    }

    return outputArray;
};

/**
 * Creates vertex buffer, binds it and draws elements
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {WebGLProgram} program ID, vertexProgramID
 * @param {Array<gluDrawUtil.VertexArrayBinding>} vertexArrays
 * @param {gluDrawUtil.PrimitiveList} primitives to gluDrawUtil.draw
 * @param { {beforeDrawCall:function(), afterDrawCall:function()}=} callback
 */
gluDrawUtil.draw = function(gl, program, vertexArrays, primitives, callback) {
    /** TODO: finish implementation */
    /** @type {Array<WebGLBuffer>} */ var objects = [];

    // Lower bindings to locations
    vertexArrays = gluDrawUtil.namedBindingsToProgramLocations(gl, program, vertexArrays);

    for (var i = 0; i < vertexArrays.length; i++) {
        /** @type {WebGLBuffer} */ var buffer = gluDrawUtil.vertexBuffer(gl, vertexArrays[i]);
        objects.push(buffer);
    }

    if (primitives.indices) {
        /** @type {WebGLBuffer} */ var elemBuffer = gluDrawUtil.indexBuffer(gl, primitives);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, elemBuffer);

        if (callback)
            callback.beforeDrawCall();

        gluDrawUtil.drawIndexed(gl, primitives, 0);

        if (callback)
            callback.afterDrawCall();

        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
    } else {
        if (callback)
            callback.beforeDrawCall();

        gl.drawArrays(gluDrawUtil.getPrimitiveGLType(gl, primitives.type), 0, primitives.numElements);

        if (callback)
            callback.afterDrawCall();
    }

  assertMsgOptions(gl.getError() === gl.NO_ERROR, 'drawArrays', false, true);
    for (var i = 0; i < vertexArrays.length; i++) {
        gl.disableVertexAttribArray(vertexArrays[i].location);
    }
  gl.bindBuffer(gl.ARRAY_BUFFER, null);
};

/**
 * Creates vertex buffer, binds it and draws elements
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluDrawUtil.PrimitiveList} primitives Primitives to gluDrawUtil.draw
 * @param {number} offset
 */
gluDrawUtil.drawIndexed = function(gl, primitives, offset) {
/** @type {number} */ var mode = gluDrawUtil.getPrimitiveGLType(gl, primitives.type);
    /** TODO: C++ implementation supports different index types, we use only int16.
        Could it cause any issues?

        deUint32 indexGLType = getIndexGLType(primitives.indexType);
    */

    gl.drawElements(mode, primitives.indices.length, gl.UNSIGNED_SHORT, offset);
};

/**
 * Enums for primitive types
 * @enum
 */
gluDrawUtil.primitiveType = {
    TRIANGLES: 0,
    TRIANGLE_STRIP: 1,
    TRIANGLE_FAN: 2,

    LINES: 3,
    LINE_STRIP: 4,
    LINE_LOOP: 5,

    POINTS: 6,

    PATCHES: 7
};

/**
 * get GL type from primitive type
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluDrawUtil.primitiveType} type gluDrawUtil.primitiveType
 * @return {number} GL primitive type
 */
gluDrawUtil.getPrimitiveGLType = function(gl, type) {
    switch (type) {
        case gluDrawUtil.primitiveType.TRIANGLES: return gl.TRIANGLES;
        case gluDrawUtil.primitiveType.TRIANGLE_STRIP: return gl.TRIANGLE_STRIP;
        case gluDrawUtil.primitiveType.TRIANGLE_FAN: return gl.TRIANGLE_FAN;
        case gluDrawUtil.primitiveType.LINES: return gl.LINES;
        case gluDrawUtil.primitiveType.LINE_STRIP: return gl.LINE_STRIP;
        case gluDrawUtil.primitiveType.LINE_LOOP: return gl.LINE_LOOP;
        case gluDrawUtil.primitiveType.POINTS: return gl.POINTS;
//        case gluDrawUtil.primitiveType.PATCHES: return gl.PATCHES;
        default:
            throw new Error('Unknown primitive type ' + type);
    }
};

/**
 * Calls gluDrawUtil.newPrimitiveListFromIndices() to create primitive list for Points
 * @param {number} numElements
 */
gluDrawUtil.pointsFromElements = function(numElements) {
    return new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.POINTS, numElements);
};

/**
 * Calls gluDrawUtil.newPrimitiveListFromIndices() to create primitive list for Triangles
 * @param {Array<number>} indices
 */
gluDrawUtil.triangles = function(indices) {
    return gluDrawUtil.newPrimitiveListFromIndices(gluDrawUtil.primitiveType.TRIANGLES, indices);
};

/**
 * Calls gluDrawUtil.newPrimitiveListFromIndices() to create primitive list for Patches
 * @param {Array<number>} indices
 */
gluDrawUtil.patches = function(indices) {
    return gluDrawUtil.newPrimitiveListFromIndices(gluDrawUtil.primitiveType.PATCHES, indices);
};

/**
 * Creates primitive list for Triangles or Patches, depending on type
 * @param {gluDrawUtil.primitiveType} type gluDrawUtil.primitiveType
 * @param {number} numElements
 * @constructor
 */
gluDrawUtil.PrimitiveList = function(type, numElements) {
    this.type = type;
    this.indices = 0;
    this.numElements = numElements;
};

/**
 * @param {gluDrawUtil.primitiveType} type
 * @param {Array<number>} indices
 * @return {gluDrawUtil.PrimitiveList}
 */
gluDrawUtil.newPrimitiveListFromIndices = function(type, indices) {
    /** @type {gluDrawUtil.PrimitiveList} */ var primitiveList = new gluDrawUtil.PrimitiveList(type, 0);
    primitiveList.indices = indices;
    return primitiveList;
};

/**
 * Create Element Array Buffer
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluDrawUtil.PrimitiveList} primitives to construct the buffer from
 * @return {WebGLBuffer} indexObject buffer with elements
 */
gluDrawUtil.indexBuffer = function(gl, primitives) {
    /** @type {WebGLBuffer} */ var indexObject = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexObject);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'bindBuffer', false, true);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(primitives.indices), gl.STATIC_DRAW);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'bufferData', false, true);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
    return indexObject;
};

/**
 * Create Array Buffer
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluDrawUtil.VertexArrayBinding} vertexArray primitives, Array buffer descriptor
 * @return {WebGLBuffer} buffer of vertices
 */
gluDrawUtil.vertexBuffer = function(gl, vertexArray) {
    /** @type {goog.TypedArray} */ var typedArray;
    switch (vertexArray.type) {
        case gl.BYTE: typedArray = new Int8Array(vertexArray.data); break;
        case gl.UNSIGNED_BYTE: typedArray = new Uint8Array(vertexArray.data); break;
        case gl.SHORT: typedArray = new Int16Array(vertexArray.data); break;
        case gl.UNSIGNED_SHORT: typedArray = new Uint16Array(vertexArray.data); break;
        case gl.INT: typedArray = new Int32Array(vertexArray.data); break;
        case gl.UNSIGNED_INT: typedArray = new Uint32Array(vertexArray.data); break;
        default: typedArray = new Float32Array(vertexArray.data); break;
    }

    /** @type {WebGLBuffer} */ var buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'bindBuffer', false, true);
    gl.bufferData(gl.ARRAY_BUFFER, typedArray, gl.STATIC_DRAW);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'bufferData', false, true);
    gl.enableVertexAttribArray(vertexArray.location);
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'enableVertexAttribArray', false, true);
    if (vertexArray.type === gl.FLOAT) {
        gl.vertexAttribPointer(vertexArray.location, vertexArray.components, vertexArray.type, false, vertexArray.stride, vertexArray.offset);
    } else {
        gl.vertexAttribIPointer(vertexArray.location, vertexArray.components, vertexArray.type, vertexArray.stride, vertexArray.offset);
    }
    assertMsgOptions(gl.getError() === gl.NO_ERROR, 'vertexAttribPointer', false, true);
    return buffer;
};

/**
 * @param {Array<number>} rgba
 * @constructor
 */
gluDrawUtil.Pixel = function(rgba) {
    this.rgba = rgba;
};

gluDrawUtil.Pixel.prototype.getRed = function() {
    return this.rgba[0];
};
gluDrawUtil.Pixel.prototype.getGreen = function() {
    return this.rgba[1];
};
gluDrawUtil.Pixel.prototype.getBlue = function() {
    return this.rgba[2];
};
gluDrawUtil.Pixel.prototype.getAlpha = function() {
    return this.rgba[3];
};
gluDrawUtil.Pixel.prototype.equals = function(otherPixel) {
    return this.rgba[0] == otherPixel.rgba[0] &&
           this.rgba[1] == otherPixel.rgba[1] &&
           this.rgba[2] == otherPixel.rgba[2] &&
           this.rgba[3] == otherPixel.rgba[3];
};

/**
 * @constructor
 */
gluDrawUtil.Surface = function() {
};

gluDrawUtil.Surface.prototype.readSurface = function(gl, x, y, width, height) {
    this.buffer = new Uint8Array(width * height * 4);
    gl.readPixels(x, y, width, height, gl.RGBA, gl.UNSIGNED_BYTE, this.buffer);
    this.x = x;
    this.y = y;
    this.width = width;
    this.height = height;
    return this.buffer;
};

gluDrawUtil.Surface.prototype.getPixel = function(x, y) {
    /** @type {number} */ var base = (x + y * this.width) * 4;
    /** @type {Array<number>} */
    var rgba = [
        this.buffer[base],
        this.buffer[base + 1],
        this.buffer[base + 2],
        this.buffer[base + 3]
        ];
    return new gluDrawUtil.Pixel(rgba);
};

gluDrawUtil.Surface.prototype.getPixelUintRGB8 = function(x, y) {
    /** @type {number} */ var base = (x + y * this.width) * 4;
    /** @type {number} */
    return (this.buffer[base] << 16) +
        (this.buffer[base + 1] << 8) +
        this.buffer[base + 2];
};

/**
 * @enum
 */
gluDrawUtil.VertexComponentType = {
    // Standard types: all conversion types apply.
    VTX_COMP_UNSIGNED_INT8: 0,
    VTX_COMP_UNSIGNED_INT16: 1,
    VTX_COMP_UNSIGNED_INT32: 2,
    VTX_COMP_SIGNED_INT8: 3,
    VTX_COMP_SIGNED_INT16: 4,
    VTX_COMP_SIGNED_INT32: 5,

    // Special types: only CONVERT_NONE is allowed.
    VTX_COMP_FIXED: 6,
    VTX_COMP_HALF_FLOAT: 7,
    VTX_COMP_FLOAT: 8
};

/**
 * @enum
 */
gluDrawUtil.VertexComponentConversion = {
    VTX_COMP_CONVERT_NONE: 0, //!< No conversion: integer types, or floating-point values.
    VTX_COMP_CONVERT_NORMALIZE_TO_FLOAT: 1, //!< Normalize integers to range [0,1] or [-1,1] depending on type.
    VTX_COMP_CONVERT_CAST_TO_FLOAT: 2 //!< Convert to floating-point directly.
};

/**
 * gluDrawUtil.VertexArrayPointer
 * @constructor
 * @param {gluDrawUtil.VertexComponentType} componentType_
 * @param {gluDrawUtil.VertexComponentConversion} convert_
 * @param {number} numComponents_
 * @param {number} numElements_
 * @param {number} stride_
 * @const @param {Array<number>} data_
 */
gluDrawUtil.VertexArrayPointer = function(componentType_, convert_, numComponents_, numElements_, stride_, data_) {
    this.componentType = componentType_;
    this.convert = convert_;
    this.numComponents = numComponents_;
    this.numElements = numElements_;
    this.stride = stride_;
    this.data = data_;
};

/**
 * gluDrawUtil.BindingPoint
 * @constructor
 * @param {string} name
 * @param {number} location
 * @param {number=} offset
 */
gluDrawUtil.BindingPoint = function(name, location, offset) {
    /** @type {string} */ this.name = name;
    /** @type {number} */ this.location = location;
    /** @type {number} */ this.offset = offset || 0;
};

/**
 * bindingPointFromLocation
 * @param {number} location
 * return {gluDrawUtil.BindingPoint}
 */
gluDrawUtil.bindingPointFromLocation = function(location) {
    return new gluDrawUtil.BindingPoint('', location);
};

/**
 * bindingPointFromName
 * @param {string} name
 * @param {number=} location
 * return {gluDrawUtil.BindingPoint}
 */
gluDrawUtil.bindingPointFromName = function(name, location) {
    location = location === undefined ? -1 : location;
    return new gluDrawUtil.BindingPoint(name, location);
};

/**
 * @param {string} name
 * @param {number} numComponents
 * @param {number} numElements
 * @param {number} stride
 * @param {Array<number>} data
 * @return {gluDrawUtil.VertexArrayBinding}
 */
gluDrawUtil.newInt32VertexArrayBinding = function(name, numComponents, numElements, stride, data) {
    var bindingPoint = gluDrawUtil.bindingPointFromName(name);
    var arrayPointer = new gluDrawUtil.VertexArrayPointer(gluDrawUtil.VertexComponentType.VTX_COMP_SIGNED_INT32,
        gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE, numComponents, numElements, stride, data);
    return gluDrawUtil.vabFromBindingPointAndArrayPointer(bindingPoint, arrayPointer, gl.INT);
};

/**
 * @param {string} name
 * @param {number} numComponents
 * @param {number} numElements
 * @param {number} stride
 * @param {Array<number>} data
 * @return {gluDrawUtil.VertexArrayBinding}
 */
gluDrawUtil.newUint32VertexArrayBinding = function(name, numComponents, numElements, stride, data) {
    var bindingPoint = gluDrawUtil.bindingPointFromName(name);
    var arrayPointer = new gluDrawUtil.VertexArrayPointer(gluDrawUtil.VertexComponentType.VTX_COMP_UNSIGNED_INT32,
        gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE, numComponents, numElements, stride, data);
    return gluDrawUtil.vabFromBindingPointAndArrayPointer(bindingPoint, arrayPointer, gl.UNSIGNED_INT);
};

/**
 * @param {string} name
 * @param {number} numComponents
 * @param {number} numElements
 * @param {number} stride
 * @param {Array<number>} data
 * @return {gluDrawUtil.VertexArrayBinding}
 */
gluDrawUtil.newFloatVertexArrayBinding = function(name, numComponents, numElements, stride, data) {
    var bindingPoint = gluDrawUtil.bindingPointFromName(name);
    var arrayPointer = new gluDrawUtil.VertexArrayPointer(gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
        gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE, numComponents, numElements, stride, data);
    return gluDrawUtil.vabFromBindingPointAndArrayPointer(bindingPoint, arrayPointer);
};

/**
 * @param {string} name
 * @param {number} column
 * @param {number} rows
 * @param {number} numElements
 * @param {number} stride
 * @param {Array<number>} data
 * @return {gluDrawUtil.VertexArrayBinding}
 */
gluDrawUtil.newFloatColumnVertexArrayBinding = function(name, column, rows, numElements, stride, data) {
    var bindingPoint = gluDrawUtil.bindingPointFromName(name);
    bindingPoint.location = column;
    var arrayPointer = new gluDrawUtil.VertexArrayPointer(gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
        gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE, rows, numElements, stride, data);
    return gluDrawUtil.vabFromBindingPointAndArrayPointer(bindingPoint, arrayPointer);
};

});
