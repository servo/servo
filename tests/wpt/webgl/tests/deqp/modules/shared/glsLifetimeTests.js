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
goog.provide('modules.shared.glsLifetimeTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuStringTemplate');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.opengl.gluShaderProgram');
goog.require('modules.shared.glsTextureTestUtil');

goog.scope(function() {
var glsLifetimeTests = modules.shared.glsLifetimeTests;
var tcuStringTemplate = framework.common.tcuStringTemplate;
var tcuSurface = framework.common.tcuSurface;
var deRandom = framework.delibs.debase.deRandom;
var glsTextureTestUtil = modules.shared.glsTextureTestUtil;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuTestCase = framework.common.tcuTestCase;
var tcuImageCompare = framework.common.tcuImageCompare;

/** @const */ var VIEWPORT_SIZE = 128;
/** @const */ var FRAMEBUFFER_SIZE = 128;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

/** @const */ var s_vertexShaderSrc =
    '#version 100\n' +
    'attribute vec2 pos;\n' +
    'void main()\n' +
    '{\n' +
    ' gl_Position = vec4(pos.xy, 0.0, 1.0);\n' +
    '}\n';

/** @const */ var s_fragmentShaderSrc =
   '#version 100\n' +
    'void main()\n' +
    '{\n' +
    ' gl_FragColor = vec4(1.0);\n' +
    '}\n';

/**
 * @constructor
 * @extends {gluShaderProgram.Shader}
 * @param {gluShaderProgram.shaderType} type
 * @param {string} src
 */
glsLifetimeTests.CheckedShader = function(type, src) {
    gluShaderProgram.Shader.call(this, gl, type);
    this.setSources(src);
    this.compile();
    assertMsgOptions(this.getCompileStatus() === true, 'Failed to compile shader', false, true);
};

setParentClass(glsLifetimeTests.CheckedShader, gluShaderProgram.Shader);

/**
 * @constructor
 * @extends {gluShaderProgram.Program}
 * @param {WebGLShader} vtxShader
 * @param {WebGLShader} fragShader
 */
glsLifetimeTests.CheckedProgram = function(vtxShader, fragShader) {
    gluShaderProgram.Program.call(this, gl);
    this.attachShader(vtxShader);
    this.attachShader(fragShader);
    this.link();
    assertMsgOptions(this.info.linkOk === true, 'Failed to link program', false, true);
};

setParentClass(glsLifetimeTests.CheckedProgram, gluShaderProgram.Program);

/**
 * @constructor
 */
glsLifetimeTests.Binder = function() {
};

/**
 * @param {WebGLObject} obj
 */
glsLifetimeTests.Binder.prototype.bind = function(obj) { throw new Error('Virtual function'); };

/**
 * @return {WebGLObject}
 */
glsLifetimeTests.Binder.prototype.getBinding = function() { throw new Error('Virtual function'); };

/**
 * @constructor
 * @extends {glsLifetimeTests.Binder}
 * @param {?function(number, ?)} bindFunc
 * @param {number} bindTarget
 * @param {number} bindingParam
 */
glsLifetimeTests.SimpleBinder = function(bindFunc, bindTarget, bindingParam) {
    glsLifetimeTests.Binder.call(this);
    this.m_bindFunc = bindFunc;
    this.m_bindTarget = bindTarget;
    this.m_bindingParam = bindingParam;
};

setParentClass(glsLifetimeTests.SimpleBinder, glsLifetimeTests.Binder);

glsLifetimeTests.SimpleBinder.prototype.bind = function(obj) {
    this.m_bindFunc.call(gl, this.m_bindTarget, obj);
};

glsLifetimeTests.SimpleBinder.prototype.getBinding = function() {
    return /** @type {WebGLObject} */ (gl.getParameter(this.m_bindingParam));
};

/**
 * @constructor
 */
glsLifetimeTests.Type = function() {
};

/**
 * Create a type
 * @return {WebGLObject}
 */
glsLifetimeTests.Type.prototype.gen = function() { throw new Error('Virtual function'); };

/**
 * Destroy a type
 * @param {WebGLObject} obj
 */
glsLifetimeTests.Type.prototype.release = function(obj) { throw new Error('Virtual function'); };

/**
 * Is object valid
 * @param {WebGLObject} obj
 */
glsLifetimeTests.Type.prototype.exists = function(obj) { throw new Error('Virtual function'); };

/**
 * Is object flagged for deletion
 * @param {WebGLObject} obj
 */
glsLifetimeTests.Type.prototype.isDeleteFlagged = function(obj) { return false; };

/**
 * @return {glsLifetimeTests.Binder}
 */
glsLifetimeTests.Type.prototype.binder = function() { return null; };

/**
 * @return {string}
 */
glsLifetimeTests.Type.prototype.getName = function() { throw new Error('Virtual function'); };

/**
 * Is the object unbound automatically when it is deleted?
 * @return {boolean}
 */
glsLifetimeTests.Type.prototype.nameLingers = function() { return false; };

/**
 * Does 'create' creates the object fully?
 * If not, the object is created at bound time
 * @return {boolean}
 */
glsLifetimeTests.Type.prototype.genCreates = function() { return false; };

/**
 * @constructor
 * @extends {glsLifetimeTests.Type}
 * @param {string} name
 * @param {function(): WebGLObject} genFunc
 * @param {function(?)} deleteFunc
 * @param {function(?): boolean} existsFunc
 * @param {glsLifetimeTests.Binder} binder
 * @param {boolean=} genCreates
 */
glsLifetimeTests.SimpleType = function(name, genFunc, deleteFunc, existsFunc, binder, genCreates) {
    glsLifetimeTests.Type.call(this);
    this.m_getName = name;
    this.m_genFunc = genFunc;
    this.m_deleteFunc = deleteFunc;
    this.m_existsFunc = existsFunc;
    this.m_binder = binder;
    this.m_genCreates = genCreates || false;
};

setParentClass(glsLifetimeTests.SimpleType, glsLifetimeTests.Type);

glsLifetimeTests.SimpleType.prototype.gen = function() { return this.m_genFunc.call(gl); };

glsLifetimeTests.SimpleType.prototype.release = function(obj) { return this.m_deleteFunc.call(gl, obj); };

glsLifetimeTests.SimpleType.prototype.exists = function(obj) { return this.m_existsFunc.call(gl, obj); };

glsLifetimeTests.SimpleType.prototype.binder = function() { return this.m_binder; };

glsLifetimeTests.SimpleType.prototype.getName = function() { return this.m_getName; };

glsLifetimeTests.SimpleType.prototype.genCreates = function() { return this.m_genCreates; };

/**
 * @constructor
 * @extends {glsLifetimeTests.Type}
 */
glsLifetimeTests.ProgramType = function() {
    glsLifetimeTests.Type.call(this);
};

setParentClass(glsLifetimeTests.ProgramType, glsLifetimeTests.Type);

glsLifetimeTests.ProgramType.prototype.gen = function() { return gl.createProgram(); };

glsLifetimeTests.ProgramType.prototype.release = function(obj) { return gl.deleteProgram(/** @type {WebGLProgram} */ (obj)); };

glsLifetimeTests.ProgramType.prototype.exists = function(obj) { return gl.isProgram(/** @type {WebGLProgram} */ (obj)); };

glsLifetimeTests.ProgramType.prototype.getName = function() { return 'program'; };

glsLifetimeTests.ProgramType.prototype.genCreates = function() { return true; };

glsLifetimeTests.ProgramType.prototype.nameLingers = function() { return true; };

glsLifetimeTests.ProgramType.prototype.isDeleteFlagged = function(obj) { return gl.getProgramParameter(/** @type {WebGLProgram} */ (obj), gl.DELETE_STATUS); };

/**
 * @constructor
 * @extends {glsLifetimeTests.Type}
 */
glsLifetimeTests.ShaderType = function() {
    glsLifetimeTests.Type.call(this);
};

setParentClass(glsLifetimeTests.ShaderType, glsLifetimeTests.Type);

glsLifetimeTests.ShaderType.prototype.gen = function() { return gl.createShader(gl.FRAGMENT_SHADER); };

glsLifetimeTests.ShaderType.prototype.release = function(obj) { return gl.deleteShader(/** @type {WebGLShader} */ (obj)); };

glsLifetimeTests.ShaderType.prototype.exists = function(obj) { return gl.isShader(/** @type {WebGLShader} */ (obj)); };

glsLifetimeTests.ShaderType.prototype.getName = function() { return 'shader'; };

glsLifetimeTests.ShaderType.prototype.genCreates = function() { return true; };

glsLifetimeTests.ShaderType.prototype.nameLingers = function() { return true; };

glsLifetimeTests.ShaderType.prototype.isDeleteFlagged = function(obj) { return gl.getShaderParameter(/** @type {WebGLShader} */ (obj), gl.DELETE_STATUS); };

/**
 * @constructor
 * @param {glsLifetimeTests.Type} elementType
 * @param {glsLifetimeTests.Type} containerType
 */
glsLifetimeTests.Attacher = function(elementType, containerType) {
    this.m_elementType = elementType;
    this.m_containerType = containerType;
};

/**
 * @param {number} seed
 * @param {WebGLObject} obj
 */
glsLifetimeTests.Attacher.prototype.initAttachment = function(seed, obj) { throw new Error('Virtual function'); };

/**
 * @param {WebGLObject} element
 * @param {WebGLObject} target
 */
glsLifetimeTests.Attacher.prototype.attach = function(element, target) { throw new Error('Virtual function'); };

/**
 * @param {WebGLObject} element
 * @param {WebGLObject} target
 */
glsLifetimeTests.Attacher.prototype.detach = function(element, target) { throw new Error('Virtual function'); };
glsLifetimeTests.Attacher.prototype.canAttachDeleted = function() { return true; };

/**
 * @return {glsLifetimeTests.Type}
 */
glsLifetimeTests.Attacher.prototype.getElementType = function() { return this.m_elementType; };

/**
 * @return {glsLifetimeTests.Type}
 */
glsLifetimeTests.Attacher.prototype.getContainerType = function() { return this.m_containerType; };

/**
 * @constructor
 */
glsLifetimeTests.InputAttacher = function(attacher) {
    this.m_attacher = attacher;
};

glsLifetimeTests.InputAttacher.prototype.getAttacher = function() { return this.m_attacher; };

/**
 * @param {WebGLObject} container
 * @param {tcuSurface.Surface} dst
 */
glsLifetimeTests.InputAttacher.prototype.drawContainer = function(container, dst) { throw new Error('Virtual function'); };

/**
 * @constructor
 */
glsLifetimeTests.OutputAttacher = function(attacher) {
    this.m_attacher = attacher;
};

glsLifetimeTests.OutputAttacher.prototype.getAttacher = function() { return this.m_attacher; };

/**
 * @param {number} seed
 * @param {WebGLObject} container
 */
glsLifetimeTests.OutputAttacher.prototype.setupContainer = function(seed, container) { throw new Error('Virtual function'); };

/**
 * @param {WebGLObject} attachment
 * @param {tcuSurface.Surface} dst
 */
glsLifetimeTests.OutputAttacher.prototype.drawAttachment = function(attachment, dst) { throw new Error('Virtual function'); };

/**
 * @constructor
 */
glsLifetimeTests.Types = function() {
    /** @type {Array<glsLifetimeTests.Type>} */ this.m_types = [];
    /** @type {Array<glsLifetimeTests.Attacher>} */ this.m_attachers = [];
    /** @type {Array<glsLifetimeTests.InputAttacher>} */ this.m_inAttachers = [];
    /** @type {Array<glsLifetimeTests.OutputAttacher>} */ this.m_outAttachers = [];
};

/**
 * @return {glsLifetimeTests.ProgramType}
 */
glsLifetimeTests.Types.prototype.getProgramType = function() { throw new Error('Virtual function'); };

glsLifetimeTests.Types.prototype.getTypes = function() { return this.m_types; };

glsLifetimeTests.Types.prototype.getAttachers = function() { return this.m_attachers; };

glsLifetimeTests.Types.prototype.getInputAttachers = function() { return this.m_inAttachers; };

glsLifetimeTests.Types.prototype.getOutputAttachers = function() { return this.m_outAttachers; };

/**
 * @param {number} seed
 * @param {WebGLFramebuffer} fbo
 */
glsLifetimeTests.setupFbo = function(seed, fbo) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

    if (seed == 0) {
        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clear(gl.COLOR_BUFFER_BIT);
    } else {
        var rnd = new deRandom.Random(seed);
        var width = rnd.getInt(0, FRAMEBUFFER_SIZE);
        var height = rnd.getInt(0, FRAMEBUFFER_SIZE);
        var x = rnd.getInt(0, FRAMEBUFFER_SIZE - width);
        var y = rnd.getInt(0, FRAMEBUFFER_SIZE - height);
        var r1 = rnd.getFloat();
        var g1 = rnd.getFloat();
        var b1 = rnd.getFloat();
        var a1 = rnd.getFloat();
        var r2 = rnd.getFloat();
        var g2 = rnd.getFloat();
        var b2 = rnd.getFloat();
        var a2 = rnd.getFloat();

        gl.clearColor(r1, g1, b1, a1);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.scissor(x, y, width, height);
        gl.enable(gl.SCISSOR_TEST);
        gl.clearColor(r2, g2, b2, a2);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.disable(gl.SCISSOR_TEST);
    }

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
};

/**
 * @param {{x: number, y:number, width: number, height: number}} rect
 * @param {tcuSurface.Surface} dst
 */
glsLifetimeTests.readRectangle = function(rect, dst) {
    dst.readViewport(gl, rect);
};

/**
 * @param {WebGLFramebuffer} fbo
 * @param {tcuSurface.Surface} dst
 */
glsLifetimeTests.drawFbo = function(fbo, dst) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    dst.readViewport(gl, [0, 0, FRAMEBUFFER_SIZE, FRAMEBUFFER_SIZE]);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
};

/**
 * @constructor
 * @extends {glsLifetimeTests.Attacher}
 */
glsLifetimeTests.FboAttacher = function(elementType, containerType) {
    glsLifetimeTests.Attacher.call(this, elementType, containerType);
};

setParentClass(glsLifetimeTests.FboAttacher, glsLifetimeTests.Attacher);

glsLifetimeTests.FboAttacher.prototype.initStorage = function() { throw new Error('Virtual function'); };

glsLifetimeTests.FboAttacher.prototype.initAttachment = function(seed, element) {
    var binder = this.getElementType().binder();
    var fbo = gl.createFramebuffer();

    binder.bind(element);
    this.initStorage();
    binder.bind(null);

    this.attach(element, fbo);
    glsLifetimeTests.setupFbo(seed, fbo);
    this.detach(element, fbo);

    gl.deleteFramebuffer(fbo);

    bufferedLogToConsole('Drew to ' + this.getElementType().getName() + ' ' + element + ' with seed ' + seed + '.');
};

/**
 * @constructor
 * @extends {glsLifetimeTests.InputAttacher}
 */
glsLifetimeTests.FboInputAttacher = function(attacher) {
    glsLifetimeTests.InputAttacher.call(this, attacher);
};

setParentClass(glsLifetimeTests.FboInputAttacher, glsLifetimeTests.InputAttacher);

glsLifetimeTests.FboInputAttacher.prototype.drawContainer = function(obj, dst) {
    var fbo = /** @type {WebGLFramebuffer} */ (obj);
    glsLifetimeTests.drawFbo(fbo, dst);
    bufferedLogToConsole('Read pixels from framebuffer ' + fbo + ' to output image.');
};

/**
 * @constructor
 * @extends {glsLifetimeTests.OutputAttacher}
 */
glsLifetimeTests.FboOutputAttacher = function(attacher) {
    glsLifetimeTests.OutputAttacher.call(this, attacher);
};

setParentClass(glsLifetimeTests.FboOutputAttacher, glsLifetimeTests.OutputAttacher);

glsLifetimeTests.FboOutputAttacher.prototype.setupContainer = function(seed, fbo) {
    glsLifetimeTests.setupFbo(seed, /** @type {WebGLFramebuffer} */ (fbo));
   bufferedLogToConsole('Drew to framebuffer ' + fbo + ' with seed ' + seed + '.');
};

glsLifetimeTests.FboOutputAttacher.prototype.drawAttachment = function(element, dst) {
    var fbo = gl.createFramebuffer();
    this.m_attacher.attach(element, fbo);
    glsLifetimeTests.drawFbo(fbo, dst);
    this.m_attacher.detach(element, fbo);
    gl.deleteFramebuffer(fbo);
    bufferedLogToConsole('Read pixels from ' + this.m_attacher.getElementType().getName() + ' ' + element + ' to output image.');
};

/**
 * @constructor
 * @extends {glsLifetimeTests.FboAttacher}
 */
glsLifetimeTests.TextureFboAttacher = function(elementType, containerType) {
    glsLifetimeTests.FboAttacher.call(this, elementType, containerType);
};

setParentClass(glsLifetimeTests.TextureFboAttacher, glsLifetimeTests.FboAttacher);

glsLifetimeTests.TextureFboAttacher.prototype.initStorage = function() {
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, FRAMEBUFFER_SIZE, FRAMEBUFFER_SIZE, 0,
                     gl.RGBA, gl.UNSIGNED_SHORT_4_4_4_4, null);

};

glsLifetimeTests.TextureFboAttacher.prototype.attach = function(element, target) {
    var texture = /** @type {WebGLTexture} */ (element);
    var fbo = /** @type {WebGLFramebuffer} */ (target);
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                              gl.TEXTURE_2D, texture, 0);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
};

glsLifetimeTests.TextureFboAttacher.prototype.detach = function(texture, target) {
    var fbo = /** @type {WebGLFramebuffer} */ (target);
    this.attach(null, fbo);
};

glsLifetimeTests.getFboAttachment = function(fbo, requiredType) {
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    var type = gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                                               gl.FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE);
    var name = gl.getFramebufferAttachmentParameter(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0,
                                               gl.FRAMEBUFFER_ATTACHMENT_OBJECT_NAME);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);

    var ret = type == requiredType ? name : null;
    return ret;
};

glsLifetimeTests.TextureFboAttacher.prototype.getAttachment = function(fbo) {
    return glsLifetimeTests.getFboAttachment(fbo, gl.TEXTURE);
};

/**
 * @constructor
 * @extends {glsLifetimeTests.FboAttacher}
 */
glsLifetimeTests.RboFboAttacher = function(elementType, containerType) {
    glsLifetimeTests.FboAttacher.call(this, elementType, containerType);
};

setParentClass(glsLifetimeTests.RboFboAttacher, glsLifetimeTests.FboAttacher);

glsLifetimeTests.RboFboAttacher.prototype.initStorage = function() {
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA4, FRAMEBUFFER_SIZE, FRAMEBUFFER_SIZE);

};

glsLifetimeTests.RboFboAttacher.prototype.attach = function(element, target) {
    var rbo = /** @type {WebGLRenderbuffer} */ (element);
    var fbo = /** @type {WebGLFramebuffer} */ (target);
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo);
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
};

glsLifetimeTests.RboFboAttacher.prototype.detach = function(rbo, target) {
    var fbo = /** @type {WebGLFramebuffer} */ (target);
    this.attach(null, fbo);
};

glsLifetimeTests.RboFboAttacher.prototype.getAttachment = function(fbo) {
    return glsLifetimeTests.getFboAttachment(fbo, gl.RENDERBUFFER);
};

/**
 * @constructor
 * @extends {glsLifetimeTests.Attacher}
 */
glsLifetimeTests.ShaderProgramAttacher = function(elementType, containerType) {
    glsLifetimeTests.Attacher.call(this, elementType, containerType);
};

setParentClass(glsLifetimeTests.ShaderProgramAttacher, glsLifetimeTests.Attacher);

glsLifetimeTests.ShaderProgramAttacher.prototype.initAttachment = function(seed, obj) {
    var shader = /** @type {WebGLShader} */ (obj);
    var s_fragmentShaderTemplate =
    '#version 100\n' +
    'void main()\n' +
    '{\n' +
    ' gl_FragColor = vec4(${RED}, ${GREEN}, ${BLUE}, 1.0);\n' +
    '}';

    var rnd = new deRandom.Random(seed);
    var params = [];
    params['RED'] = rnd.getFloat().toString(10);
    params['GREEN'] = rnd.getFloat().toString(10);
    params['BLUE'] = rnd.getFloat().toString(10);

    var source = tcuStringTemplate.specialize(s_fragmentShaderTemplate, params);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    var compileStatus = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
    assertMsgOptions(compileStatus === true, 'Failed to compile shader: ' + source, false, true);
};

glsLifetimeTests.ShaderProgramAttacher.prototype.attach = function(element, target) {
    var shader = /** @type {WebGLShader} */ (element);
    var program = /** @type {WebGLProgram} */ (target);
    gl.attachShader(program, shader);
};

glsLifetimeTests.ShaderProgramAttacher.prototype.detach = function(element, target) {
    var shader = /** @type {WebGLShader} */ (element);
    var program = /** @type {WebGLProgram} */ (target);
    gl.detachShader(program, shader);
};

glsLifetimeTests.ShaderProgramAttacher.prototype.getAttachment = function(program) {
    var shaders = gl.getAttachedShaders(program);
    for (var i = 0; i < shaders.length; i++) {
        var shader = shaders[i];
        var type = gl.getShaderParameter(shader, gl.SHADER_TYPE);
        if (type === gl.FRAGMENT_SHADER)
            return shader;
    }
    return null;
};

/**
 * @constructor
 * @extends {glsLifetimeTests.InputAttacher}
 */
glsLifetimeTests.ShaderProgramInputAttacher = function(attacher) {
    glsLifetimeTests.InputAttacher.call(this, attacher);
};

setParentClass(glsLifetimeTests.ShaderProgramInputAttacher, glsLifetimeTests.InputAttacher);

glsLifetimeTests.ShaderProgramInputAttacher.prototype.drawContainer = function(container, dst) {
    var program = /** @type {WebGLProgram} */ (container);
    var s_vertices = [-1.0, 0.0, 1.0, 1.0, 0.0, -1.0];
    glsLifetimeTests.ShaderProgramInputAttacher.seed = glsLifetimeTests.ShaderProgramInputAttacher.seed || 0;
    var vtxShader = new glsLifetimeTests.CheckedShader(gluShaderProgram.shaderType.VERTEX, s_vertexShaderSrc);
    var viewport = new glsTextureTestUtil.RandomViewport(document.getElementById('canvas'), VIEWPORT_SIZE, VIEWPORT_SIZE, glsLifetimeTests.ShaderProgramInputAttacher.seed);

    gl.attachShader(program, vtxShader.getShader());
    gl.linkProgram(program);

    var linkStatus = gl.getProgramParameter(program, gl.LINK_STATUS);
    assertMsgOptions(linkStatus === true, 'Program link failed', false, true);

    bufferedLogToConsole('Attached a temporary vertex shader and linked program ' + program);

    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);

    bufferedLogToConsole('Positioned viewport randomly');

    gl.useProgram(program);

    var posLoc = gl.getAttribLocation(program, 'pos');
    assertMsgOptions(posLoc >= 0, 'Could not find pos attribute', false, true);

    var buf = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(s_vertices), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);

    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 3);

    gl.disableVertexAttribArray(posLoc);
    gl.deleteBuffer(buf);
    bufferedLogToConsole('Drew a fixed triangle');

    gl.useProgram(null);

    glsLifetimeTests.readRectangle(viewport, dst);
    bufferedLogToConsole('Copied viewport to output image');

    gl.detachShader(program, vtxShader.getShader());
    bufferedLogToConsole('Removed temporary vertex shader');
};

/**
 * @constructor
 * @extends {glsLifetimeTests.Types}
 */
glsLifetimeTests.ES2Types = function() {
    glsLifetimeTests.Types.call(this);
    this.m_bufferBind = new glsLifetimeTests.SimpleBinder(gl.bindBuffer, gl.ARRAY_BUFFER, gl.ARRAY_BUFFER_BINDING);
    this.m_bufferType = new glsLifetimeTests.SimpleType('buffer', gl.createBuffer, gl.deleteBuffer, gl.isBuffer, this.m_bufferBind);
    this.m_textureBind = new glsLifetimeTests.SimpleBinder(gl.bindTexture, gl.TEXTURE_2D, gl.TEXTURE_BINDING_2D);
    this.m_textureType = new glsLifetimeTests.SimpleType('texture', gl.createTexture, gl.deleteTexture, gl.isTexture, this.m_textureBind);
    this.m_rboBind = new glsLifetimeTests.SimpleBinder(gl.bindRenderbuffer, gl.RENDERBUFFER, gl.RENDERBUFFER_BINDING);
    this.m_rboType = new glsLifetimeTests.SimpleType('renderbuffer', gl.createRenderbuffer, gl.deleteRenderbuffer, gl.isRenderbuffer, this.m_rboBind);
    this.m_fboBind = new glsLifetimeTests.SimpleBinder(gl.bindFramebuffer, gl.FRAMEBUFFER, gl.FRAMEBUFFER_BINDING);
    this.m_fboType = new glsLifetimeTests.SimpleType('framebuffer', gl.createFramebuffer, gl.deleteFramebuffer, gl.isFramebuffer, this.m_fboBind);
    this.m_shaderType = new glsLifetimeTests.ShaderType();
    this.m_programType = new glsLifetimeTests.ProgramType();
    this.m_texFboAtt = new glsLifetimeTests.TextureFboAttacher(this.m_textureType, this.m_fboType);
    this.m_texFboInAtt = new glsLifetimeTests.FboInputAttacher(this.m_texFboAtt);
    this.m_texFboOutAtt = new glsLifetimeTests.FboOutputAttacher(this.m_texFboAtt);
    this.m_rboFboAtt = new glsLifetimeTests.RboFboAttacher(this.m_rboType, this.m_fboType);
    this.m_rboFboInAtt = new glsLifetimeTests.FboInputAttacher(this.m_rboFboAtt);
    this.m_rboFboOutAtt = new glsLifetimeTests.FboOutputAttacher(this.m_rboFboAtt);
    this.m_shaderAtt = new glsLifetimeTests.ShaderProgramAttacher(this.m_shaderType, this.m_programType);
    this.m_shaderInAtt = new glsLifetimeTests.ShaderProgramInputAttacher(this.m_shaderAtt);

    this.m_types.push(this.m_bufferType, this.m_textureType, this.m_rboType, this.m_fboType, this.m_shaderType, this.m_programType);
    this.m_attachers.push(this.m_texFboAtt, this.m_rboFboAtt, this.m_shaderAtt);
    this.m_inAttachers.push(this.m_texFboInAtt, this.m_rboFboInAtt, this.m_shaderInAtt);
    this.m_outAttachers.push(this.m_texFboOutAtt, this.m_rboFboOutAtt);
};

setParentClass(glsLifetimeTests.ES2Types, glsLifetimeTests.Types);

glsLifetimeTests.ES2Types.prototype.getProgramType = function() { return this.m_programType; };

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 * @param {string} name
 * @param {string} description
 * @param {glsLifetimeTests.Type} type
 * @param {function()} test
 */
glsLifetimeTests.LifeTest = function(name, description, type, test) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_type = type;
    this.m_test = test;
};

setParentClass(glsLifetimeTests.LifeTest, tcuTestCase.DeqpTest);

glsLifetimeTests.LifeTest.prototype.iterate = function() {
    this.m_test();
    return tcuTestCase.IterateResult.STOP;
};

/**
 * @this {glsLifetimeTests.LifeTest}
 */
glsLifetimeTests.LifeTest.testGen = function() {
    var obj = this.m_type.gen();
    if (this.m_type.genCreates())
        assertMsgOptions(this.m_type.exists(obj), "create* should have created an object, but didn't", false, true);
    else
        assertMsgOptions(!this.m_type.exists(obj), 'create* should not have created an object, but did', false, true);
    this.m_type.release(obj);
    testPassed();
};

/**
 * @this {glsLifetimeTests.LifeTest}
 */
glsLifetimeTests.LifeTest.testDelete = function() {
    var obj = this.m_type.gen();
    this.m_type.release(obj);
    assertMsgOptions(!this.m_type.exists(obj), 'Object still exists after deletion', false, true);
    testPassed();
};

/**
 * @this {glsLifetimeTests.LifeTest}
 */
glsLifetimeTests.LifeTest.testBind = function() {
    var obj = this.m_type.gen();
    this.m_type.binder().bind(obj);
    var err = gl.getError();
    assertMsgOptions(err == gl.NONE, 'Bind failed', false, true);
    assertMsgOptions(this.m_type.exists(obj), 'Object does not exist after binding', false, true);
    this.m_type.binder().bind(null);
    this.m_type.release(obj);
    testPassed();
};

/**
 * @this {glsLifetimeTests.LifeTest}
 */
glsLifetimeTests.LifeTest.testDeleteBound = function() {
    var obj = this.m_type.gen();
    this.m_type.binder().bind(obj);
    this.m_type.release(obj);
    if (this.m_type.nameLingers()) {
        assertMsgOptions(gl.getError() == gl.NONE, 'Deleting bound object failed', false, true);
        assertMsgOptions(this.m_type.binder().getBinding() === obj, 'Deleting bound object did not retain binding', false, true);
        assertMsgOptions(this.m_type.exists(obj), 'Deleting bound object made its name invalid', false, true);
        assertMsgOptions(this.m_type.isDeleteFlagged(obj), 'Deleting bound object did not flag the object for deletion', false, true);
        this.m_type.binder().bind(null);
    } else {
        assertMsgOptions(gl.getError() == gl.NONE, 'Deleting bound object failed', false, true);
        assertMsgOptions(this.m_type.binder().getBinding() === null, 'Deleting bound object did not remove binding', false, true);
        assertMsgOptions(!this.m_type.exists(obj), 'Deleting bound object did not make its name invalid', false, true);
    }
    assertMsgOptions(this.m_type.binder().getBinding() === null, "Unbinding didn't remove binding", false, true);
    assertMsgOptions(!this.m_type.exists(obj), 'Name is still valid after deleting and unbinding', false, true);
    testPassed();
};

/**
 * @this {glsLifetimeTests.LifeTest}
 */
glsLifetimeTests.LifeTest.testDeleteUsed = function() {
    var vtxShader = new glsLifetimeTests.CheckedShader(gluShaderProgram.shaderType.VERTEX, s_vertexShaderSrc);
    var fragShader = new glsLifetimeTests.CheckedShader(gluShaderProgram.shaderType.FRAGMENT, s_fragmentShaderSrc);
    var program = new glsLifetimeTests.CheckedProgram(vtxShader.getShader(), fragShader.getShader());
    var programId = program.getProgram();
    bufferedLogToConsole('Created and linked program ' + programId);
    gl.useProgram(programId);

    gl.deleteProgram(programId);
    bufferedLogToConsole('Deleted program ' + programId);
    assertMsgOptions(gl.isProgram(programId), 'Deleted current program', false, true);
    var deleteFlagged = gl.getProgramParameter(programId, gl.DELETE_STATUS);
    assertMsgOptions(deleteFlagged == true, 'Program object was not flagged as deleted', false, true);
    gl.useProgram(null);
    assertMsgOptions(!gl.isProgram(programId), 'Deleted program name still valid after being made non-current', false, true);
    testPassed();
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 * @param {string} name
 * @param {string} description
 * @param {glsLifetimeTests.Attacher} attacher
 * @param {function()} test
 */
glsLifetimeTests.AttachmentTest = function(name, description, attacher, test) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_attacher = attacher;
    this.m_test = test;
};

setParentClass(glsLifetimeTests.AttachmentTest, tcuTestCase.DeqpTest);

glsLifetimeTests.AttachmentTest.prototype.iterate = function() {
    this.m_test();
    return tcuTestCase.IterateResult.STOP;
};

/**
 * @this {glsLifetimeTests.AttachmentTest}
 */
glsLifetimeTests.AttachmentTest.testDeletedNames = function() {
    var getAttachment = function(attacher, container) {
        var queriedAttachment = attacher.getAttachment(container);
        bufferedLogToConsole('Result of query for ' + attacher.getElementType().getName() +
                       ' attached to ' + attacher.getContainerType().getName() + ' ' +
                       container + ': ' + queriedAttachment);
        return queriedAttachment;
    };

    var elemType = this.m_attacher.getElementType();
    var containerType = this.m_attacher.getContainerType();
    var container = containerType.gen();

    var element = elemType.gen();
    this.m_attacher.initAttachment(0, element);
    this.m_attacher.attach(element, container);
    assertMsgOptions(getAttachment(this.m_attacher, container) == element,
                 'Attachment not returned by query even before deletion.', false, true);

    elemType.release(element);
    // "Such a container or other context may continue using the object, and
    // may still contain state identifying its name as being currently bound"
    //
    // We here interpret "may" to mean that whenever the container has a
    // deleted object attached to it, a query will return that object's former
    // name.
    assertMsgOptions(getAttachment(this.m_attacher, container) == element,
                 'Attachment name not returned by query after attachment was deleted.', false, true);

    if (elemType.nameLingers())
        assertMsgOptions(elemType.exists(element),
                     'Attached object name no longer valid after deletion.', false, true);
    else
        assertMsgOptions(!elemType.exists(element),
                     'Attached object name still valid after deletion.', false, true);

    this.m_attacher.detach(element, container);
    assertMsgOptions(getAttachment(this.m_attacher, container) == null,
                 'Attachment name returned by query even after detachment.', false, true);
    assertMsgOptions(!elemType.exists(element),
                 'Deleted attached object name still usable after detachment.', false, true);
    testPassed();
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 * @param {string} name
 * @param {string} description
 * @param {glsLifetimeTests.InputAttacher} attacher
 */
glsLifetimeTests.InputAttachmentTest = function(name, description, attacher) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_inputAttacher = attacher;
};

setParentClass(glsLifetimeTests.InputAttachmentTest, tcuTestCase.DeqpTest);

glsLifetimeTests.InputAttachmentTest.prototype.iterate = function() {
    var attacher = this.m_inputAttacher.getAttacher();
    var containerType = attacher.getContainerType();
    var elementType = attacher.getElementType();
    var container = containerType.gen();

    glsLifetimeTests.InputAttachmentTest.seed = glsLifetimeTests.InputAttachmentTest.seed || 0;
    ++glsLifetimeTests.InputAttachmentTest.seed;
    var rnd = new deRandom.Random(glsLifetimeTests.InputAttachmentTest.seed);
    var refSeed = rnd.getInt();
    var newSeed = rnd.getInt();

    var refSurface = new tcuSurface.Surface(VIEWPORT_SIZE, VIEWPORT_SIZE); // Surface from drawing with refSeed-seeded attachment
    var delSurface = new tcuSurface.Surface(VIEWPORT_SIZE, VIEWPORT_SIZE); // Surface from drawing with deleted refSeed attachment
    var newSurface = new tcuSurface.Surface(VIEWPORT_SIZE, VIEWPORT_SIZE); // Surface from drawing with newSeed-seeded attachment

    bufferedLogToConsole('Testing if writing to a newly created object modifies a deleted attachment');

    bufferedLogToConsole('Writing to an original attachment');
    var element = elementType.gen();

    attacher.initAttachment(refSeed, element);
    attacher.attach(element, container);
    this.m_inputAttacher.drawContainer(container, refSurface);
    // element gets deleted here
    bufferedLogToConsole('Deleting attachment');
    elementType.release(element);

    bufferedLogToConsole('Writing to a new attachment after deleting the original');
    var newElement = elementType.gen();

    attacher.initAttachment(newSeed, newElement);

    this.m_inputAttacher.drawContainer(container, delSurface);
    attacher.detach(element, container);

    attacher.attach(newElement, container);
    this.m_inputAttacher.drawContainer(container, newSurface);
    attacher.detach(newElement, container);
    var surfacesMatch = tcuImageCompare.pixelThresholdCompare(
        'Reading from deleted',
        'Comparison result from reading from a container with a deleted attachment ' +
        'before and after writing to a fresh object.',
        refSurface, delSurface, [0, 0, 0, 0]);

    /* TODO: Add logging images */
    // if (!surfacesMatch)
    //     log() << TestLog::Image("New attachment",
    //                             "Container state after attached to the fresh object",
    //                             newSurface);

    assertMsgOptions(surfacesMatch,
        'Writing to a fresh object modified the container with a deleted attachment.', false, true);

    testPassed();
    return tcuTestCase.IterateResult.STOP;
};

/**
 * @constructor
 * @extends {tcuTestCase.DeqpTest}
 * @param {string} name
 * @param {string} description
 * @param {glsLifetimeTests.OutputAttacher} attacher
 */
glsLifetimeTests.OutputAttachmentTest = function(name, description, attacher) {
    tcuTestCase.DeqpTest.call(this, name, description);
    this.m_outputAttacher = attacher;
};

setParentClass(glsLifetimeTests.OutputAttachmentTest, tcuTestCase.DeqpTest);

glsLifetimeTests.OutputAttachmentTest.prototype.iterate = function() {
    var attacher = this.m_outputAttacher.getAttacher();
    var containerType = attacher.getContainerType();
    var elementType = attacher.getElementType();
    var container = containerType.gen();
    glsLifetimeTests.InputAttachmentTest.seed = glsLifetimeTests.InputAttachmentTest.seed || 0;
    ++glsLifetimeTests.InputAttachmentTest.seed;
    var rnd = new deRandom.Random(glsLifetimeTests.InputAttachmentTest.seed);
    var refSeed = rnd.getInt();
    var newSeed = rnd.getInt();

    var refSurface = new tcuSurface.Surface(VIEWPORT_SIZE, VIEWPORT_SIZE); // Surface from drawing with refSeed-seeded attachment
    var delSurface = new tcuSurface.Surface(VIEWPORT_SIZE, VIEWPORT_SIZE); // Surface from drawing with deleted refSeed attachment
    var newSurface = new tcuSurface.Surface(VIEWPORT_SIZE, VIEWPORT_SIZE); // Surface from drawing with newSeed-seeded attachment

    bufferedLogToConsole('Testing if writing to a container with a deleted attachment ' +
          'modifies a newly created object');

    bufferedLogToConsole('Writing to a container with an existing attachment');
    var element = elementType.gen();

    attacher.initAttachment(0, element);
    attacher.attach(element, container);

    // For reference purposes, make note of what refSeed looks like.
    this.m_outputAttacher.setupContainer(refSeed, container);
    // Since in WebGL, buffer bound to TRANSFORM_FEEDBACK_BUFFER can not be bound to other targets.
    // Unfortunately, element will be bound again in drawAttachment() for drawing.
    // Detach element from container before drawing, then reattach it after drawing.
    attacher.detach(element, container);
    this.m_outputAttacher.drawAttachment(element, refSurface);
    attacher.attach(element, container);
    elementType.release(element);

    bufferedLogToConsole('Writing to a container after deletion of attachment');
    var newElement = elementType.gen();
    bufferedLogToConsole('Creating a new object ');

    bufferedLogToConsole('Recording state of new object before writing to container');
    attacher.initAttachment(newSeed, newElement);
    this.m_outputAttacher.drawAttachment(newElement, newSurface);

    bufferedLogToConsole('Writing to container');

    // Now re-write refSeed to the container.
    this.m_outputAttacher.setupContainer(refSeed, container);
    // Does it affect the newly created attachment object?
    this.m_outputAttacher.drawAttachment(newElement, delSurface);
    attacher.detach(element, container);

    var surfacesMatch = tcuImageCompare.pixelThresholdCompare(
        'Writing to deleted',
        'Comparison result from reading from a fresh object before and after ' +
        'writing to a container with a deleted attachment',
        newSurface, delSurface, [0, 0, 0, 0]);

    /* TODO: Add logging images */
    // if (!surfacesMatch)
    //     log() << TestLog::Image(
    //         "Original attachment",
    //         "Result of container modification on original attachment before deletion.",
    //         refSurface);

    assertMsgOptions(surfacesMatch,
                 'Writing to container with deleted attachment modified a new object.', false, true);

    testPassed();
    return tcuTestCase.IterateResult.STOP;
};

glsLifetimeTests.createLifeTestGroup = function(spec, types) {
    var group = tcuTestCase.newTest(spec.name, spec.name);

    for (var i = 0; i < types.length; i++) {
        var type = types[i];
        var name = type.getName();
        if (!spec.needBind || type.binder() != null)
            group.addChild(new glsLifetimeTests.LifeTest(name, name, type, spec.func));
    }

    return group;
};

/**
 * @param {tcuTestCase.DeqpTest} group
 * @param {glsLifetimeTests.Types} types
 */
glsLifetimeTests.addTestCases = function(group, types) {
    var attacherName = function(attacher) {
        return attacher.getElementType().getName() + '_' + attacher.getContainerType().getName();
    };

    var s_lifeTests = [
        /* Create */ { name: 'gen', func: glsLifetimeTests.LifeTest.testGen, needBind: false },
        /* Delete */ { name: 'delete', func: glsLifetimeTests.LifeTest.testDelete, needBind: false },
        /* Bind */ { name: 'bind', func: glsLifetimeTests.LifeTest.testBind, needBind: true },
        /* Delete bound */ { name: 'delete_bound', func: glsLifetimeTests.LifeTest.testDeleteBound, needBind: true }
    ];

    s_lifeTests.forEach(function(spec) {
        group.addChild(glsLifetimeTests.createLifeTestGroup(spec, types.getTypes()));
    });

    var delUsedGroup = tcuTestCase.newTest('delete_used', 'Delete current program');
    group.addChild(delUsedGroup);

    delUsedGroup.addChild(new glsLifetimeTests.LifeTest('program', 'program', types.getProgramType(),
                     glsLifetimeTests.LifeTest.testDeleteUsed));

    var attGroup = tcuTestCase.newTest('attach', 'Attachment tests');
    group.addChild(attGroup);

    var nameGroup = tcuTestCase.newTest('deleted_name', 'Name of deleted attachment');
    attGroup.addChild(nameGroup);

    var atts = types.getAttachers();
    for (var i = 0; i < atts.length; i++) {
        var att = atts[i];
        var name = attacherName(att);
        nameGroup.addChild(new glsLifetimeTests.AttachmentTest(name, name, att,
                                               glsLifetimeTests.AttachmentTest.testDeletedNames));
    }

    var inputGroup = tcuTestCase.newTest('deleted_input', 'Input from deleted attachment');
    attGroup.addChild(inputGroup);

    var inAtts = types.getInputAttachers();
    for (var i = 0; i < inAtts.length; i++) {
        var att = inAtts[i];
        var name = attacherName(att.getAttacher());
        inputGroup.addChild(new glsLifetimeTests.InputAttachmentTest(name, name, att));
    }

    var outputGroup = tcuTestCase.newTest('deleted_output', 'Output to deleted attachment');
    attGroup.addChild(outputGroup);

    var outAtts = types.getOutputAttachers();
    for (var i = 0; i < outAtts.length; i++) {
        var att = outAtts[i];
        var name = attacherName(att.getAttacher());
        outputGroup.addChild(new glsLifetimeTests.OutputAttachmentTest(name, name, att));
    }

};

});
