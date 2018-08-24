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
goog.provide('functional.gles3.es3fLifetimeTests');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderProgram');
goog.require('modules.shared.glsLifetimeTests');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {
var es3fLifetimeTests = functional.gles3.es3fLifetimeTests;
var glsLifetimeTests = modules.shared.glsLifetimeTests;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var deRandom = framework.delibs.debase.deRandom;
var tcuSurface = framework.common.tcuSurface;
var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
var tcuTestCase = framework.common.tcuTestCase;

/** @const */ var VIEWPORT_SIZE = 128;
/** @const */ var NUM_COMPONENTS = 4;
/** @const */ var NUM_VERTICES = 3;

/** @type {WebGL2RenderingContext} */ var gl;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/**
 * @constructor
 * @extends {gluShaderProgram.ShaderProgram}
 */
es3fLifetimeTests.ScaleProgram = function() {
    gluShaderProgram.ShaderProgram.call(this, gl, this.getSources());
    assertMsgOptions(this.isOk(), 'Program creation failed', false, true);
    this.m_scaleLoc = gl.getUniformLocation(this.getProgram(), 'scale');
    this.m_posLoc = gl.getAttribLocation(this.getProgram(), 'pos');
};

setParentClass(es3fLifetimeTests.ScaleProgram, gluShaderProgram.ShaderProgram);

/**
 * @param {WebGLVertexArrayObject} vao
 * @param {number} scale
 * @param {boolean} tf
 * @param {tcuSurface.Surface} dst
 */
es3fLifetimeTests.ScaleProgram.prototype.draw = function(vao, scale, tf, dst) {
    es3fLifetimeTests.ScaleProgram.seed = es3fLifetimeTests.ScaleProgram.seed || 0;
    ++es3fLifetimeTests.ScaleProgram.seed;

    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), VIEWPORT_SIZE, VIEWPORT_SIZE, es3fLifetimeTests.ScaleProgram.seed);
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    gl.bindVertexArray(vao);
    gl.enableVertexAttribArray(this.m_posLoc);
    gl.useProgram(this.getProgram());

    gl.uniform1f(this.m_scaleLoc, scale);

    if (tf)
        gl.beginTransformFeedback(gl.TRIANGLES);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
    if (tf)
        gl.endTransformFeedback();

    if (dst)
        glsLifetimeTests.readRectangle(viewport, dst);

    gl.bindVertexArray(null);

};

/**
 * @param {WebGLBuffer} buffer
 * @param {WebGLVertexArrayObject} vao
 */
es3fLifetimeTests.ScaleProgram.prototype.setPos = function(buffer, vao) {
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bindVertexArray(vao);
    if (buffer) {
        gl.vertexAttribPointer(this.m_posLoc, NUM_COMPONENTS, gl.FLOAT, false, 0, 0);
    } else {
        var name = gl.getVertexAttrib(this.m_posLoc, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING);
        gl.deleteBuffer(name);
    }
    gl.bindVertexArray(null);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
};

/**
 * @private
 */
es3fLifetimeTests.ScaleProgram.prototype.getSources = function() {
/** @const */ var s_vertexShaderSrc =
    '#version 100\n' +
    'attribute vec4 pos;\n' +
    'uniform float scale;\n' +
    'void main ()\n' +
    '{\n' +
    ' gl_Position = vec4(scale * pos.xy, pos.zw);\n' +
    '}';

/** @const */ var s_fragmentShaderSrc =
    '#version 100\n' +
    'void main ()\n' +
    '{\n' +
    ' gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);\n' +
    '}';
    var sources = new gluShaderProgram.ProgramSources();
    sources.add(new gluShaderProgram.VertexSource(s_vertexShaderSrc));
    sources.add(new gluShaderProgram.FragmentSource(s_fragmentShaderSrc));
    sources.add(new gluShaderProgram.TransformFeedbackMode(gl.INTERLEAVED_ATTRIBS));
    sources.add(new gluShaderProgram.TransformFeedbackVarying('gl_Position'));
    return sources;
};

/**
 * @constructor
 * @extends {glsLifetimeTests.SimpleBinder}
 */
es3fLifetimeTests.VertexArrayBinder = function() {
    glsLifetimeTests.SimpleBinder.call(this, null, gl.NONE, gl.VERTEX_ARRAY_BINDING);
};

setParentClass(es3fLifetimeTests.VertexArrayBinder, glsLifetimeTests.SimpleBinder);

es3fLifetimeTests.VertexArrayBinder.prototype.bind = function(obj) {
    var vao = /** @type {WebGLVertexArrayObject} */ (obj);
    gl.bindVertexArray(vao);
};

/**
 * @constructor
 * @extends {glsLifetimeTests.Binder}
 */
es3fLifetimeTests.SamplerBinder = function() {
   glsLifetimeTests.Binder.call(this);
};

setParentClass(es3fLifetimeTests.SamplerBinder, glsLifetimeTests.Binder);

es3fLifetimeTests.SamplerBinder.prototype.bind = function(obj) {
    var sampler = /** @type {WebGLSampler} */ (obj);
    gl.bindSampler(0, sampler);
};
es3fLifetimeTests.SamplerBinder.prototype.getBinding = function() { return /** @type {WebGLSampler} */ (gl.getParameter(gl.SAMPLER_BINDING)); };

/**
 * @constructor
 * @extends {glsLifetimeTests.Binder}
 */
es3fLifetimeTests.QueryBinder = function() {
   glsLifetimeTests.Binder.call(this);
};

setParentClass(es3fLifetimeTests.QueryBinder, glsLifetimeTests.Binder);

es3fLifetimeTests.QueryBinder.prototype.bind = function(obj) {
    var query = /** @type {WebGLQuery} */ (obj);
    if (query)
        gl.beginQuery(gl.ANY_SAMPLES_PASSED, query);
    else
        gl.endQuery(gl.ANY_SAMPLES_PASSED);
};

es3fLifetimeTests.QueryBinder.prototype.getBinding = function() { return null; };

/**
 * @constructor
 * @extends {glsLifetimeTests.Attacher}
 * @param {glsLifetimeTests.Type} elementType
 * @param {glsLifetimeTests.Type} varrType
 * @param {es3fLifetimeTests.ScaleProgram} program
 */
es3fLifetimeTests.BufferVAOAttacher = function(elementType, varrType, program) {
    glsLifetimeTests.Attacher.call(this, elementType, varrType);
    this.m_program = program;
};

setParentClass(es3fLifetimeTests.BufferVAOAttacher, glsLifetimeTests.Attacher);

/**
 * @return {es3fLifetimeTests.ScaleProgram}
 */
es3fLifetimeTests.BufferVAOAttacher.prototype.getProgram = function() { return this.m_program; };

/**
 * @param {number} seed
 * @param {number} usage
 * @param {WebGLBuffer} buffer
 */
es3fLifetimeTests.initBuffer = function(seed, usage, buffer) {
    /** @const */ var s_varrData = [
    -1.0, 0.0, 0.0, 1.0,
     1.0, 1.0, 0.0, 1.0,
     0.0, -1.0, 0.0, 1.0
    ];
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    if (seed == 0)
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(s_varrData), usage);
    else {
        var rnd = new deRandom.Random(seed);
        var data = [];

        for (var ndx = 0; ndx < NUM_VERTICES; ndx++) {
            data.push(2 * (rnd.getFloat() - 0.5));
            data.push(2 * (rnd.getFloat() - 0.5));
            data.push(0);
            data.push(1);
        }
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(data), usage);
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
};

es3fLifetimeTests.BufferVAOAttacher.prototype.initAttachment = function(seed, obj) {
    var buffer = /** @type {WebGLBuffer} */ (obj);
    es3fLifetimeTests.initBuffer(seed, gl.STATIC_DRAW, buffer);
    bufferedLogToConsole('Initialized buffer ' + buffer + ' from seed ' + seed);
};

es3fLifetimeTests.BufferVAOAttacher.prototype.attach = function(element, target) {
    var buffer = /** @type {WebGLBuffer} */ (element);
    var vao = /** @type {WebGLVertexArrayObject} */ (target);

    this.m_program.setPos(buffer, vao);
    bufferedLogToConsole('Set the `pos` attribute in VAO ' + vao + ' to buffer ' + buffer);
};

es3fLifetimeTests.BufferVAOAttacher.prototype.detach = function(element, target) {
    var vao = /** @type {WebGLVertexArrayObject} */ (target);
    this.attach(null, vao);
};

es3fLifetimeTests.BufferVAOAttacher.prototype.getAttachment = function(target) {
    var vao = /** @type {WebGLVertexArrayObject} */ (target);
    gl.bindVertexArray(vao);
    var name = gl.getVertexAttrib(this.m_posLoc, gl.VERTEX_ATTRIB_ARRAY_BUFFER_BINDING);
    gl.bindVertexArray(null);
    return name;
};

/**
 * @constructor
 * @extends {glsLifetimeTests.InputAttacher}
 * @param {es3fLifetimeTests.BufferVAOAttacher} attacher
 */
es3fLifetimeTests.BufferVAOInputAttacher = function(attacher) {
    glsLifetimeTests.InputAttacher.call(this, attacher);
    this.m_program = attacher.getProgram();
};

setParentClass(es3fLifetimeTests.BufferVAOInputAttacher, glsLifetimeTests.InputAttacher);

es3fLifetimeTests.BufferVAOInputAttacher.prototype.drawContainer = function(obj, dst) {
    var vao = /** @type {WebGLVertexArrayObject} */ (obj);
    this.m_program.draw(vao, 1.0, false, dst);
    bufferedLogToConsole('Drew an output image with VAO ' + vao);
};

/**
 * @constructor
 * @extends {glsLifetimeTests.Attacher}
 * @param {glsLifetimeTests.Type} elementType
 * @param {glsLifetimeTests.Type} tfType
*/
es3fLifetimeTests.BufferTfAttacher = function(elementType, tfType) {
    glsLifetimeTests.Attacher.call(this, elementType, tfType);
};

setParentClass(es3fLifetimeTests.BufferTfAttacher, glsLifetimeTests.Attacher);

es3fLifetimeTests.BufferTfAttacher.prototype.initAttachment = function(seed, obj) {
    var buffer = /** @type {WebGLBuffer} */ (obj);
    es3fLifetimeTests.initBuffer(seed, gl.DYNAMIC_READ, buffer);
    bufferedLogToConsole('Initialized buffer ' + buffer + ' from seed ' + seed);
};

es3fLifetimeTests.BufferTfAttacher.prototype.attach = function(element, target) {
    var buffer = /** @type {WebGLBuffer} */ (element);
    var tf = /** @type {WebGLTransformFeedback} */ (target);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buffer);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
};

es3fLifetimeTests.BufferTfAttacher.prototype.detach = function(element, target) {
    var buffer = /** @type {WebGLBuffer} */ (element);
    var tf = /** @type {WebGLTransformFeedback} */ (target);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, null);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);

};

es3fLifetimeTests.BufferTfAttacher.prototype.getAttachment = function(target) {
    var tf = /** @type {WebGLTransformFeedback} */ (target);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    var name = gl.getIndexedParameter(gl.TRANSFORM_FEEDBACK_BUFFER_BINDING, 0);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
    return name;
};

/**
 * @constructor
 * @extends {glsLifetimeTests.OutputAttacher}
 */
es3fLifetimeTests.BufferTfOutputAttacher = function(attacher, program) {
    glsLifetimeTests.OutputAttacher.call(this, attacher);
    this.m_program = program;
};

setParentClass(es3fLifetimeTests.BufferTfOutputAttacher, glsLifetimeTests.OutputAttacher);

es3fLifetimeTests.BufferTfOutputAttacher.prototype.setupContainer = function(seed, obj) {
    var tf = /** @type {WebGLTransformFeedback} */ (obj);
    var posBuf = gl.createBuffer();
    var vao = gl.createVertexArray();

    es3fLifetimeTests.initBuffer(seed, gl.STATIC_DRAW, posBuf);
    this.m_program.setPos(posBuf, vao);

    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    this.m_program.draw(vao, -1.0, true, null);
    bufferedLogToConsole('Drew an image with seed ' + seed + ' with transform feedback to ' + tf);
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
    gl.deleteVertexArray(vao);
    gl.deleteBuffer(posBuf);
};

es3fLifetimeTests.BufferTfOutputAttacher.prototype.drawAttachment = function(buffer, dst) {
    var vao = gl.createVertexArray();

    this.m_program.setPos(buffer, vao);
    this.m_program.draw(vao, 1.0, false, dst);
    bufferedLogToConsole('Drew output image with vertices from buffer ' + buffer);
    gl.deleteVertexArray(vao);
};

/**
 * @constructor
 * @extends {glsLifetimeTests.ES2Types}
 */
es3fLifetimeTests.ES3Types = function() {
    glsLifetimeTests.ES2Types.call(this);
    this.m_program = new es3fLifetimeTests.ScaleProgram();
    this.m_queryBind = new es3fLifetimeTests.QueryBinder();
    this.m_queryType = new glsLifetimeTests.SimpleType('query', gl.createQuery, gl.deleteQuery, gl.isQuery, this.m_queryBind);
    this.m_tfBind = new glsLifetimeTests.SimpleBinder(gl.bindTransformFeedback, gl.TRANSFORM_FEEDBACK,
                     gl.TRANSFORM_FEEDBACK_BINDING);
    this.m_tfType = new glsLifetimeTests.SimpleType('transform_feedback', gl.createTransformFeedback, gl.deleteTransformFeedback, gl.isTransformFeedback, this.m_tfBind);
    this.m_varrBind = new es3fLifetimeTests.VertexArrayBinder();
    this.m_varrType = new glsLifetimeTests.SimpleType('vertex_array', gl.createVertexArray, gl.deleteVertexArray, gl.isVertexArray, this.m_varrBind);
    this.m_samplerBind = new es3fLifetimeTests.SamplerBinder();
    this.m_samplerType = new glsLifetimeTests.SimpleType('sampler', gl.createSampler, gl.deleteSampler, gl.isSampler, this.m_samplerBind, true);
    this.m_bufVarrAtt = new es3fLifetimeTests.BufferVAOAttacher(this.m_bufferType, this.m_varrType, this.m_program);
    this.m_bufVarrInAtt = new es3fLifetimeTests.BufferVAOInputAttacher(this.m_bufVarrAtt);
    this.m_bufTfAtt = new es3fLifetimeTests.BufferTfAttacher(this.m_bufferType, this.m_tfType);
    this.m_bufTfOutAtt = new es3fLifetimeTests.BufferTfOutputAttacher(this.m_bufTfAtt, this.m_program);

    this.m_types.push(this.m_queryType, this.m_tfType, this.m_varrType, this.m_samplerType);
    this.m_attachers.push(this.m_bufVarrAtt, this.m_bufTfAtt);
    this.m_inAttachers.push(this.m_bufVarrInAtt);
    this.m_outAttachers.push(this.m_bufTfOutAtt);
};

setParentClass(es3fLifetimeTests.ES3Types, glsLifetimeTests.ES2Types);

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 */
es3fLifetimeTests.TfDeleteActiveTest = function(name, description) {
    tcuTestCase.DeqpTest.call(this, name, description);
};

setParentClass(es3fLifetimeTests.TfDeleteActiveTest, tcuTestCase.DeqpTest);

es3fLifetimeTests.TfDeleteActiveTest.prototype.iterate = function() {
/** @const */ var s_xfbVertexSource =
    '#version 300 es\n' +
    'void main ()\n' +
    '{\n' +
    ' gl_Position = vec4(float(gl_VertexID) / 2.0, float(gl_VertexID % 2) / 2.0, 0.0, 1.0);\n' +
    '}\n';

/** @const */  var s_xfbFragmentSource =
    '#version 300 es\n' +
    'layout(location=0) out mediump vec4 dEQP_FragColor;\n' +
    'void main (void)\n' +
    '{\n' +
    ' dEQP_FragColor = vec4(1.0, 1.0, 0.0, 1.0);\n' +
    '}\n';
    var buf = gl.createBuffer();

    var sources = new gluShaderProgram.ProgramSources();
    sources.add(new gluShaderProgram.VertexSource(s_xfbVertexSource));
    sources.add(new gluShaderProgram.FragmentSource(s_xfbFragmentSource));
    sources.add(new gluShaderProgram.TransformFeedbackMode(gl.SEPARATE_ATTRIBS));
    sources.add(new gluShaderProgram.TransformFeedbackVarying('gl_Position'));
    var program = new gluShaderProgram.ShaderProgram(gl, sources);
    if (!program.isOk()) {
        bufferedLogToConsole(program.getProgramInfo().infoLog);
        testFailedOptions('failed to build program', true);
    }
    gl.useProgram(program.getProgram());

    var tf = gl.createTransformFeedback();
    gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, tf);
    gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, 0, buf);
    gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, 48, gl.STATIC_DRAW);

    gl.beginTransformFeedback(gl.TRIANGLES);
    var errCode = gl.NONE;
    gl.deleteTransformFeedback(tf);
    errCode = gl.getError();
    assertMsgOptions(errCode == gl.INVALID_OPERATION,
        'Deleting active transform feedback must produce INVALID_OPERATION', false, true);
    gl.endTransformFeedback();
    gl.deleteTransformFeedback(tf);
    testPassed();
    return tcuTestCase.IterateResult.STOP;
};

es3fLifetimeTests.genTestCases = function() {
    var state = tcuTestCase.runner;
    state.setRoot(tcuTestCase.newTest('lifetime', 'Top level'));

    var types = new es3fLifetimeTests.ES3Types();
    glsLifetimeTests.addTestCases(state.testCases, types);
    /* TODO: Add TfDeleteActiveTest test */
    var deleteActiveGroup = tcuTestCase.newTest('delete_active', 'Delete active object');
    state.testCases.addChild(deleteActiveGroup);
    deleteActiveGroup.addChild(
        new es3fLifetimeTests.TfDeleteActiveTest('transform_feedback', 'Transform Feedback'));
};

/**
 * Create and execute the test cases
 */
es3fLifetimeTests.run = function(context) {
    gl = context;
    try {
        es3fLifetimeTests.genTestCases();
        tcuTestCase.runner.runCallback(tcuTestCase.runTestCases);
    } catch (err) {
        bufferedLogToConsole(err);
        tcuTestCase.runner.terminate();
    }

};

});
