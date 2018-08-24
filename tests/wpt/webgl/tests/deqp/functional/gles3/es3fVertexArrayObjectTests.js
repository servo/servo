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
goog.provide('functional.gles3.es3fVertexArrayObjectTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluShaderProgram');

goog.scope(function() {
var es3fVertexArrayObjectTests = functional.gles3.es3fVertexArrayObjectTests;
var tcuTestCase = framework.common.tcuTestCase;
var deRandom = framework.delibs.debase.deRandom;
var deString = framework.delibs.debase.deString;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuSurface = framework.common.tcuSurface;
var tcuImageCompare = framework.common.tcuImageCompare;
var deUtil = framework.delibs.debase.deUtil;

/**
 * @constructor
 */
es3fVertexArrayObjectTests.Attribute = function() {
    this.enabled = false;
    this.size = 1;
    this.stride = 0;
    this.type = gl.FLOAT;
    this.integer = false;
    this.divisor = 0;
    this.offset = 0;
    this.normalized = false;
    this.bufferNdx = 0;
};

/**
 * @constructor
 * @struct
 */
es3fVertexArrayObjectTests.VertexArrayState = function() {
    this.attributes = [];
    this.elementArrayBuffer = 0;
};

/**
 * @constructor
 * @struct
 */
es3fVertexArrayObjectTests.BufferSpec = function(count, size, componentCount, stride, offset, type, intRangeMin, intRangeMax, floatRangeMin, floatRangeMax) {
    this.count = count;
    this.size = size;
    this.componentCount = componentCount;
    this.stride = stride;
    this.offset = offset;

    this.type = type;

    this.intRangeMin = intRangeMin;
    this.intRangeMax = intRangeMax;

    this.floatRangeMin = floatRangeMin;
    this.floatRangeMax = floatRangeMax;
};

/**
 * @constructor
 */
es3fVertexArrayObjectTests.Spec = function() {
    this.count = -1;
    this.instances = -1;
    this.useDrawElements = false;
    this.indexType = gl.NONE;
    this.indexOffset = -1;
    this.indexRangeMin = -1;
    this.indexRangeMax = -1;
    this.indexCount = -1;
    this.buffers = [];
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 * @param {es3fVertexArrayObjectTests.Spec} spec
 * @param {string} name
 * @param {string} description
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest = function(spec, name, description) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_spec = spec;
    this.m_random = new deRandom.Random(deString.deStringHash(name));
    /** @type Array<WebGLBuffer>} */ this.m_buffers = [];
    // mapping 0 -> null object
    this.m_buffers.push(null);
};

es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.constructor = es3fVertexArrayObjectTests.VertexArrayObjectTest;

es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.init = function() {
    this.m_vaoProgram = this.createProgram(this.m_spec.vao);
    // m_log << tcu::TestLog::Message << "Program used with Vertex Array Object" << tcu::TestLog::EndMessage;
    // m_log << *m_vaoProgram;
    this.m_stateProgram = this.createProgram(this.m_spec.state);
    // m_log << tcu::TestLog::Message << "Program used with Vertex Array State" << tcu::TestLog::EndMessage;
    // m_log << *m_stateProgram;

    if (!this.m_vaoProgram.isOk() || !this.m_stateProgram.isOk())
        testFailedOptions('Failed to compile shaders', true);
};

/**
 * @param {number} target GL target
 * @param {number} index Index of the buffer to bind
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.bindBuffer = function(target, index) {
    if (typeof this.m_buffers[index] === 'undefined') {
        var data = this.createRandomBufferData(this.m_spec.buffers[index - 1]);
        var buffer = gl.createBuffer();
        this.m_buffers[index] = buffer;

        gl.bindBuffer(target, buffer);
        gl.bufferData(target, data, gl.DYNAMIC_DRAW);
        gl.bindBuffer(target, null);
    }

    gl.bindBuffer(target, this.m_buffers[index]);
};

/**
 * @param {es3fVertexArrayObjectTests.BufferSpec} buffer
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.createRandomBufferData = function(buffer) {
    var typedArray;
    switch (buffer.type) {
        case gl.FLOAT: typedArray = Float32Array; break;
        case gl.INT: typedArray = Int32Array; break;
        case gl.UNSIGNED_INT: typedArray = Uint32Array; break;
        case gl.SHORT: typedArray = Int16Array; break;
        case gl.UNSIGNED_SHORT: typedArray = Uint16Array; break;
        case gl.BYTE: typedArray = Int8Array; break;
        case gl.UNSIGNED_BYTE: typedArray = Uint8Array; break;
        default:
            throw new Error('Invalid type: ' + buffer.type);
    }

    var raw = new ArrayBuffer(buffer.size);
    var stride;

    if (buffer.stride != 0) {
        stride = buffer.stride;
    } else {
        switch (buffer.type) {
            case gl.FLOAT: stride = buffer.componentCount * 4; break;
            case gl.INT: stride = buffer.componentCount * 4; break;
            case gl.UNSIGNED_INT: stride = buffer.componentCount * 4; break;
            case gl.SHORT: stride = buffer.componentCount * 2; break;
            case gl.UNSIGNED_SHORT: stride = buffer.componentCount * 2; break;
            case gl.BYTE: stride = buffer.componentCount * 1; break;
            case gl.UNSIGNED_BYTE: stride = buffer.componentCount * 1; break;
        }
    }

    var offset = 0;

    for (var pos = 0; pos < buffer.count; pos++) {
        var data = new typedArray(raw, offset, buffer.componentCount);
        for (var componentNdx = 0; componentNdx < buffer.componentCount; componentNdx++) {
            switch (buffer.type) {
                case gl.FLOAT: {
                    data[componentNdx] = this.m_random.getFloat(buffer.floatRangeMin, buffer.floatRangeMax);
                    break;
                }
                default: {
                    data[componentNdx] = this.m_random.getInt(buffer.intRangeMin, buffer.intRangeMax);
                }
            }
        }

        offset += stride;
    }

    return new typedArray(raw);
};

/**
 * @param {es3fVertexArrayObjectTests.VertexArrayState} state
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.createProgram = function(state) {
    var vtx = '';
    var value = '';

    vtx += '#version 300 es\n';

    for (var attribNdx = 0; attribNdx < state.attributes.length; attribNdx++) {
        if (state.attributes[attribNdx].integer)
            vtx += 'layout(location = ' + attribNdx + ') in mediump ivec4 a_attrib' + attribNdx + ';\n';
        else
            vtx += 'layout(location = ' + attribNdx + ') in mediump vec4 a_attrib' + attribNdx + ';\n';

        if (state.attributes[attribNdx].integer) {
            var scale = 0.0;

            // TODO: Should it be state.attributes[attribNdx].type?
            switch (state.attributes[attribNdx].type) {
                case gl.SHORT: scale = (1.0 / ((1 << 14) - 1)); break;
                case gl.UNSIGNED_SHORT: scale = (1.0 / ((1 << 15) - 1)); break;
                case gl.INT: scale = (1.0 / ((1 << 30) - 1)); break;
                case gl.UNSIGNED_INT: scale = (1.0 / ((1 << 31) - 1)); break;
                case gl.BYTE: scale = (1.0 / ((1 << 6) - 1)); break;
                case gl.UNSIGNED_BYTE: scale = (1.0 / ((1 << 7) - 1)); break;

                default:
                    throw new Error('Invalid type: ' + state.attributes[0].type);
            }
            value += (attribNdx != 0 ? ' + ' : '') + scale + ' * vec4(a_attrib' + attribNdx + ')';
        } else if (state.attributes[attribNdx].type != gl.FLOAT && !state.attributes[attribNdx].normalized) {
            var scale = 0.0;

            switch (state.attributes[attribNdx].type) {
                case gl.SHORT: scale = (0.5 / ((1 << 14) - 1)); break;
                case gl.UNSIGNED_SHORT: scale = (0.5 / ((1 << 15) - 1)); break;
                case gl.INT: scale = (0.5 / ((1 << 30) - 1)); break;
                case gl.UNSIGNED_INT: scale = (0.5 / ((1 << 31) - 1)); break;
                case gl.BYTE: scale = (0.5 / ((1 << 6) - 1)); break;
                case gl.UNSIGNED_BYTE: scale = (0.5 / ((1 << 7) - 1)); break;

                default:
                    throw new Error('Invalid type: ' + state.attributes[0].type);
            }
            value += (attribNdx != 0 ? ' + ' : '') + scale + ' * a_attrib' + attribNdx;
        } else
            value += (attribNdx != 0 ? ' + ' : '') + 'a_attrib' + attribNdx;
    }

    vtx +=
        'out mediump vec4 v_value;\n' +
        'void main (void)\n' +
        '{\n' +
        '\tv_value = ' + value + ';\n';

    if (state.attributes[0].integer) {
        var scale = 0.0;

        switch (state.attributes[0].type) {
            case gl.SHORT: scale = (1.0 / ((1 << 14) - 1)); break;
            case gl.UNSIGNED_SHORT: scale = (1.0 / ((1 << 15) - 1)); break;
            case gl.INT: scale = (1.0 / ((1 << 30) - 1)); break;
            case gl.UNSIGNED_INT: scale = (1.0 / ((1 << 31) - 1)); break;
            case gl.BYTE: scale = (1.0 / ((1 << 6) - 1)); break;
            case gl.UNSIGNED_BYTE: scale = (1.0 / ((1 << 7) - 1)); break;

            default:
                throw new Error('Invalid type: ' + state.attributes[0].type);
        }

        vtx +=
            '\tgl_Position = vec4(' + scale + ' * ' + 'vec3(a_attrib0.xyz), 1.0);\n' +
            '}';
    } else {
        if (state.attributes[0].normalized || state.attributes[0].type == gl.FLOAT) {
            vtx +=
                '\tgl_Position = vec4(a_attrib0.xyz, 1.0);\n' +
                '}';
        } else {
            var scale = 0.0;

            switch (state.attributes[0].type) {
                case gl.SHORT: scale = (1.0 / ((1 << 14) - 1)); break;
                case gl.UNSIGNED_SHORT: scale = (1.0 / ((1 << 15) - 1)); break;
                case gl.INT: scale = (1.0 / ((1 << 30) - 1)); break;
                case gl.UNSIGNED_INT: scale = (1.0 / ((1 << 31) - 1)); break;
                case gl.BYTE: scale = (1.0 / ((1 << 6) - 1)); break;
                case gl.UNSIGNED_BYTE: scale = (1.0 / ((1 << 7) - 1)); break;

                default:
                    throw new Error('Invalid type: ' + state.attributes[0].type);
            }

            scale *= 0.5;

            vtx +=
                '\tgl_Position = vec4(' + scale + ' * ' + 'a_attrib0.xyz, 1.0);\n' +
                '}';
        }
    }

    var fragmentShader =
        '#version 300 es\n' +
        'in mediump vec4 v_value;\n' +
        'layout(location = 0) out mediump vec4 fragColor;\n' +
        'void main (void)\n' +
        '{\n' +
        '\tfragColor = vec4(v_value.xyz, 1.0);\n' +
        '}';

    return new gluShaderProgram.ShaderProgram(gl, gluShaderProgram.makeVtxFragSources(vtx, fragmentShader));
};

/**
 * @param {es3fVertexArrayObjectTests.VertexArrayState} state
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.setState = function(state) {
    this.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, state.elementArrayBuffer);

    for (var attribNdx = 0; attribNdx < state.attributes.length; attribNdx++) {
        this.bindBuffer(gl.ARRAY_BUFFER, state.attributes[attribNdx].bufferNdx);
        if (state.attributes[attribNdx].enabled)
            gl.enableVertexAttribArray(attribNdx);
        else
            gl.disableVertexAttribArray(attribNdx);

        if (state.attributes[attribNdx].integer)
            gl.vertexAttribIPointer(attribNdx, state.attributes[attribNdx].size, state.attributes[attribNdx].type, state.attributes[attribNdx].stride, state.attributes[attribNdx].offset);
        else
            gl.vertexAttribPointer(attribNdx, state.attributes[attribNdx].size, state.attributes[attribNdx].type, state.attributes[attribNdx].normalized, state.attributes[attribNdx].stride, state.attributes[attribNdx].offset);

        gl.vertexAttribDivisor(attribNdx, state.attributes[attribNdx].divisor);
    }
};

/**
 * @param {es3fVertexArrayObjectTests.VertexArrayState} state
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.makeDrawCall = function(state) {
    gl.clearColor(0.7, 0.7, 0.7, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    var spec = this.m_spec;

    if (spec.useDrawElements) {
        if (spec.instances == 0)
            gl.drawElements(gl.TRIANGLES, spec.count, spec.indexType, spec.indexOffset);
        else
            gl.drawElementsInstanced(gl.TRIANGLES, spec.count, spec.indexType, spec.indexOffset, spec.instances);
    } else {
        if (spec.instances == 0)
            gl.drawArrays(gl.TRIANGLES, 0, spec.count);
        else
            gl.drawArraysInstanced(gl.TRIANGLES, 0, spec.count, spec.instances);
    }
};

/**
 * @param {tcuSurface.Surface} vaoResult
 * @param {tcuSurface.Surface} defaultResult
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.render = function(vaoResult, defaultResult) {
   var vao = gl.createVertexArray();

    gl.bindVertexArray(vao);
    this.setState(this.m_spec.vao);
    gl.bindVertexArray(null);

    this.setState(this.m_spec.state);

    gl.bindVertexArray(vao);
    gl.useProgram(this.m_vaoProgram.getProgram());
    this.makeDrawCall(this.m_spec.vao);
    vaoResult.readViewport();
    this.setState(this.m_spec.vao);
    gl.bindVertexArray(null);

    gl.useProgram(this.m_stateProgram.getProgram());
    this.makeDrawCall(this.m_spec.state);
    defaultResult.readViewport();

    gl.deleteVertexArray(vao);
};

/**
 * @param {tcuSurface.Surface} vaoRef
 * @param {tcuSurface.Surface} defaultRef
 */
es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.genReferences = function(vaoRef, defaultRef) {
    this.setState(this.m_spec.vao);
    gl.useProgram(this.m_vaoProgram.getProgram());
    this.makeDrawCall(this.m_spec.vao);
    vaoRef.readViewport();

    this.setState(this.m_spec.state);
    gl.useProgram(this.m_stateProgram.getProgram());
    this.makeDrawCall(this.m_spec.state);
    defaultRef.readViewport();
};

es3fVertexArrayObjectTests.VertexArrayObjectTest.prototype.iterate = function() {
    var vaoReference = new tcuSurface.Surface();
    var stateReference = new tcuSurface.Surface();
    var vaoResult = new tcuSurface.Surface();
    var stateResult = new tcuSurface.Surface();

    var isOk;

    // logVertexArrayState(m_log, m_spec.vao, "Vertex Array Object State");
    // logVertexArrayState(m_log, m_spec.state, "OpenGL Vertex Array State");
    this.genReferences(stateReference, vaoReference);
    this.render(stateResult, vaoResult);

    isOk = tcuImageCompare.pixelThresholdCompare('Results', 'Comparison result from rendering with Vertex Array State', stateReference, stateResult, [0, 0, 0, 0]);
    isOk = isOk && tcuImageCompare.pixelThresholdCompare('Results', 'Comparison result from rendering with Vertex Array Object', vaoReference, vaoResult, [0, 0, 0, 0]);

    if (!isOk)
        testFailedOptions('Result comparison failed', false);
    else
        testPassedOptions('Pass', true);

    return tcuTestCase.IterateResult.STOP;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fVertexArrayObjectTests.VertexArrayObjectTests = function() {
    tcuTestCase.DeqpTest.call(this, 'vertex_array_objects', 'Vertex array object test cases');
};

es3fVertexArrayObjectTests.VertexArrayObjectTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fVertexArrayObjectTests.VertexArrayObjectTests.prototype.constructor = es3fVertexArrayObjectTests.VertexArrayObjectTests;

es3fVertexArrayObjectTests.VertexArrayObjectTests.prototype.init = function() {
    var floatCoordBuffer48_1 = new es3fVertexArrayObjectTests.BufferSpec(48, 384, 2, 0, 0, gl.FLOAT, 0, 0, -1.0, 1.0);
    var floatCoordBuffer48_2 = new es3fVertexArrayObjectTests.BufferSpec(48, 384, 2, 0, 0, gl.FLOAT, 0, 0, -1.0, 1.0);

    var shortCoordBuffer48 = new es3fVertexArrayObjectTests.BufferSpec(48, 192, 2, 0, 0, gl.SHORT, -32768, 32768, 0.0, 0.0);

    var spec;
    var state;
    // Different buffer
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.FLOAT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = false;

    state.elementArrayBuffer = 0;

    spec.buffers.push(floatCoordBuffer48_1);
    spec.buffers.push(floatCoordBuffer48_2);

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 48;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.state.attributes[0].bufferNdx = 1;
    spec.vao.attributes[0].bufferNdx = 2;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_buffer', 'diff_buffer'));

    // Different size
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.FLOAT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = false;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(floatCoordBuffer48_1);

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.state.attributes[0].size = 2;
    spec.vao.attributes[0].size = 3;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_size', 'diff_size'));

    // Different stride
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.SHORT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = true;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(shortCoordBuffer48);

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.vao.attributes[0].stride = 2;
    spec.state.attributes[0].stride = 4;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_stride', 'diff_stride'));

    // Different types
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.SHORT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = true;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(shortCoordBuffer48);

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.vao.attributes[0].type = gl.SHORT;
    spec.state.attributes[0].type = gl.BYTE;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_type', 'diff_type'));

    // Different "integer"
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.BYTE;
    state.attributes[0].integer = true;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = false;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(shortCoordBuffer48);

    spec.useDrawElements = false;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.instances = 0;
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.state.attributes[0].integer = false;
    spec.vao.attributes[0].integer = true;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_integer', 'diff_integer'));

    // Different divisor
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());
    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.SHORT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = true;
    state.attributes[0].bufferNdx = 1;

    state.attributes[1].enabled = true;
    state.attributes[1].size = 4;
    state.attributes[1].stride = 0;
    state.attributes[1].type = gl.FLOAT;
    state.attributes[1].integer = false;
    state.attributes[1].divisor = 0;
    state.attributes[1].offset = 0;
    state.attributes[1].normalized = false;
    state.attributes[1].bufferNdx = 2;

    state.elementArrayBuffer = 0;

    spec.buffers.push(shortCoordBuffer48);
    spec.buffers.push(floatCoordBuffer48_1);

    spec.useDrawElements = false;
    spec.instances = 10;
    spec.count = 12;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.vao.attributes[1].divisor = 3;
    spec.state.attributes[1].divisor = 2;

    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_divisor', 'diff_divisor'));

    // Different offset
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.SHORT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = true;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(shortCoordBuffer48);

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.vao.attributes[0].offset = 2;
    spec.state.attributes[0].offset = 4;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_offset', 'diff_offset'));

    // Different normalize
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.SHORT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = true;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(shortCoordBuffer48);

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 48;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;

    spec.vao.attributes[0].normalized = true;
    spec.state.attributes[0].normalized = false;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_normalize', 'diff_normalize'));

    // DrawElements with buffer
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    state.attributes.push(new es3fVertexArrayObjectTests.Attribute());

    state.attributes[0].enabled = true;
    state.attributes[0].size = 2;
    state.attributes[0].stride = 0;
    state.attributes[0].type = gl.FLOAT;
    state.attributes[0].integer = false;
    state.attributes[0].divisor = 0;
    state.attributes[0].offset = 0;
    state.attributes[0].normalized = true;
    state.attributes[0].bufferNdx = 1;

    state.elementArrayBuffer = 0;

    spec.buffers.push(floatCoordBuffer48_1);

    var indexBuffer = new es3fVertexArrayObjectTests.BufferSpec(24, 192, 1, 0, 0, gl.UNSIGNED_SHORT, 0, 47, 0.0, 0.0);
    spec.buffers.push(indexBuffer);
    spec.buffers.push(indexBuffer);

    spec.useDrawElements = true;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.instances = 0;
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 48;
    spec.indexType = gl.UNSIGNED_SHORT;
    spec.indexCount = 24;

    spec.state.elementArrayBuffer = 3;
    spec.vao.elementArrayBuffer = 2;
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'diff_indices', 'diff_indices'));

    var attribCount = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
    var random = new deRandom.Random(attribCount);
    spec = new es3fVertexArrayObjectTests.Spec();

    state = new es3fVertexArrayObjectTests.VertexArrayState();

    spec.useDrawElements = false;
    spec.instances = 0;
    spec.count = 24;
    spec.vao = state;
    spec.state = deUtil.clone(state);
    spec.indexOffset = 0;
    spec.indexRangeMin = 0;
    spec.indexRangeMax = 0;
    spec.indexType = gl.NONE;
    spec.indexCount = 0;
    spec.vao.elementArrayBuffer = 0;
    spec.state.elementArrayBuffer = 0;

    // Use all attributes
    for (var attribNdx = 0; attribNdx < attribCount; attribNdx++) {
        spec.buffers.push(shortCoordBuffer48);

        spec.state.attributes.push(new es3fVertexArrayObjectTests.Attribute());
        spec.state.attributes[attribNdx].enabled = (random.getInt(0, 4) == 0) ? false : true;
        spec.state.attributes[attribNdx].size = random.getInt(2, 4);
        spec.state.attributes[attribNdx].stride = 2 * random.getInt(1, 3);
        spec.state.attributes[attribNdx].type = gl.SHORT;
        spec.state.attributes[attribNdx].integer = random.getBool();
        spec.state.attributes[attribNdx].divisor = random.getInt(0, 1);
        spec.state.attributes[attribNdx].offset = 2 * random.getInt(0, 2);
        spec.state.attributes[attribNdx].normalized = random.getBool();
        spec.state.attributes[attribNdx].bufferNdx = attribNdx + 1;

        if (attribNdx == 0) {
            spec.state.attributes[attribNdx].divisor = 0;
            spec.state.attributes[attribNdx].enabled = true;
            spec.state.attributes[attribNdx].size = 2;
        }

        spec.vao.attributes.push(new es3fVertexArrayObjectTests.Attribute());
        spec.vao.attributes[attribNdx].enabled = (random.getInt(0, 4) == 0) ? false : true;
        spec.vao.attributes[attribNdx].size = random.getInt(2, 4);
        spec.vao.attributes[attribNdx].stride = 2 * random.getInt(1, 3);
        spec.vao.attributes[attribNdx].type = gl.SHORT;
        spec.vao.attributes[attribNdx].integer = random.getBool();
        spec.vao.attributes[attribNdx].divisor = random.getInt(0, 1);
        spec.vao.attributes[attribNdx].offset = 2 * random.getInt(0, 2);
        spec.vao.attributes[attribNdx].normalized = random.getBool();
        spec.vao.attributes[attribNdx].bufferNdx = attribCount - attribNdx;

        if (attribNdx == 0) {
            spec.vao.attributes[attribNdx].divisor = 0;
            spec.vao.attributes[attribNdx].enabled = true;
            spec.vao.attributes[attribNdx].size = 2;
        }

    }
    this.addChild(new es3fVertexArrayObjectTests.VertexArrayObjectTest(spec, 'all_attributes', 'all_attributes'));

};

/**
 * Run test
 * @param {WebGL2RenderingContext} context
 */
es3fVertexArrayObjectTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fVertexArrayObjectTests.VertexArrayObjectTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fVertexArrayObjectTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});
