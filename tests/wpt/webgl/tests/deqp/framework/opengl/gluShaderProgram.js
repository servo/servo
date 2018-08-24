/*-------------------------------------------------------------------------
 * drawElements Quality gluShaderProgram.Program OpenGL ES Utilities
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
goog.provide('framework.opengl.gluShaderProgram');

goog.scope(function() {

var gluShaderProgram = framework.opengl.gluShaderProgram;

/**
 * gluShaderProgram.Shader type enum
 * @enum {number}
 */
gluShaderProgram.shaderType = {
    VERTEX: 0,
    FRAGMENT: 1
};

/**
 * gluShaderProgram.Shader type enum name
 * @param {gluShaderProgram.shaderType} shaderType
 * @return {string}
 */
gluShaderProgram.getShaderTypeName = function(shaderType) {
    var s_names =
    [
        'vertex',
        'fragment'
    ];

    return s_names[shaderType];
};

/**
 * Get GL shader type from gluShaderProgram.shaderType
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluShaderProgram.shaderType} type gluShaderProgram.Shader Type
 * @return {number} GL shader type
 */
gluShaderProgram.getGLShaderType = function(gl, type) {
    var _glShaderType;
    switch (type) {
    case gluShaderProgram.shaderType.VERTEX: _glShaderType = gl.VERTEX_SHADER; break;
    case gluShaderProgram.shaderType.FRAGMENT: _glShaderType = gl.FRAGMENT_SHADER; break;
    default:
        throw new Error('Unknown shader type ' + type);
    }
    return _glShaderType;
};

/**
 * Declares shader information
 * @constructor
 * @param {gluShaderProgram.shaderType} type
 * @param {string=} source
 */
gluShaderProgram.ShaderInfo = function(type, source) {
    this.type = type; /** gluShaderProgram.Shader type. */
    this.source = source; /** gluShaderProgram.Shader source. */
    this.infoLog; /** Compile info log. */
    this.compileOk = false; /** Did compilation succeed? */
    this.compileTimeUs = 0; /** Compile time in microseconds (us). */
};

/**
 * Generates vertex shader info from source
 * @param {string} source
 * @return {gluShaderProgram.ShaderInfo} vertex shader info
 */
gluShaderProgram.genVertexSource = function(source) {
/** @type {gluShaderProgram.ShaderInfo} */ var shader = new gluShaderProgram.ShaderInfo(gluShaderProgram.shaderType.VERTEX, source);
    return shader;
};

/**
 * Generates fragment shader info from source
 * @param {string} source
 * @return {gluShaderProgram.ShaderInfo} fragment shader info
 */
gluShaderProgram.genFragmentSource = function(source) {
/** @type {gluShaderProgram.ShaderInfo} */ var shader = new gluShaderProgram.ShaderInfo(gluShaderProgram.shaderType.FRAGMENT, source);
    return shader;
};

/**
 * Generates shader from WebGL context and type
 * @constructor
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluShaderProgram.shaderType} type gluShaderProgram.Shader Type
 */
gluShaderProgram.Shader = function(gl, type) {
    this.gl = gl;
    this.info = new gluShaderProgram.ShaderInfo(type); /** Client-side clone of state for debug / perf reasons. */
    this.shader = gl.createShader(gluShaderProgram.getGLShaderType(gl, type));
    assertMsgOptions(gl.getError() == gl.NO_ERROR, 'gl.createShader()', false, true);

    this.setSources = function(source) {
        this.gl.shaderSource(this.shader, source);
        assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'glshaderSource()', false, true);
        this.info.source = source;
    };

    this.getCompileStatus = function() {
        return this.info.compileOk;
    };

    this.compile = function() {
        this.info.compileOk = false;
        this.info.compileTimeUs = 0;
        this.info.infoLog = '';

        /** @type {Date} */ var compileStart = new Date();
        this.gl.compileShader(this.shader);
        /** @type {Date} */ var compileEnd = new Date();
        this.info.compileTimeUs = 1000 * (compileEnd.getTime() - compileStart.getTime());

        assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.compileShader()', false, true);

        var compileStatus = this.gl.getShaderParameter(this.shader, this.gl.COMPILE_STATUS);
        assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'glGetShaderParameter()', false, true);

        this.info.compileOk = compileStatus;
        this.info.infoLog = this.gl.getShaderInfoLog(this.shader);
    };

    this.getShader = function() {
        return this.shader;
    };

    this.destroy = function() {
        this.gl.deleteShader(this.shader);
    };

};
/**
 * Creates gluShaderProgram.ProgramInfo
 * @constructor
 */
gluShaderProgram.ProgramInfo = function() {
    /** @type {string} */  this.infoLog = ''; /** Link info log. */
    /** @type {boolean} */ this.linkOk = false; /** Did link succeed? */
    /** @type {number} */ this.linkTimeUs = 0; /** Link time in microseconds (us). */
};

/**
 * Creates program.
 * Inner methods: attach shaders, bind attributes location, link program and transform Feedback Varyings
 * @constructor
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {WebGLProgram=} programID
 */
gluShaderProgram.Program = function(gl, programID) {
    this.gl = gl;
    this.program = programID || null;
    this.info = new gluShaderProgram.ProgramInfo();

    if (!programID) {
        this.program = gl.createProgram();
        assertMsgOptions(gl.getError() == gl.NO_ERROR, 'gl.createProgram()', false, true);
    }
};

/**
 * @return {WebGLProgram}
 */
gluShaderProgram.Program.prototype.getProgram = function() { return this.program; };

/**
 * @return {gluShaderProgram.ProgramInfo}
 */
gluShaderProgram.Program.prototype.getInfo = function() { return this.info; };

/**
 * @param {WebGLShader} shader
 */
gluShaderProgram.Program.prototype.attachShader = function(shader) {
    this.gl.attachShader(this.program, shader);
    assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.attachShader()', false, true);
};

/**
 * @param {WebGLShader} shader
 */
gluShaderProgram.Program.prototype.detachShader = function(shader) {
    this.gl.detachShader(this.program, shader);
    assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.detachShader()', false, true);
};

/**
 * @param {number} location
 * @param {string} name
 */
gluShaderProgram.Program.prototype.bindAttribLocation = function(location, name) {
    this.gl.bindAttribLocation(this.program, location, name);
    assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.bindAttribLocation()', false, true);
};

gluShaderProgram.Program.prototype.link = function() {
    this.info.linkOk = false;
    this.info.linkTimeUs = 0;
    this.info.infoLog = '';

    /** @type {Date} */ var linkStart = new Date();
    this.gl.linkProgram(this.program);
    /** @type {Date} */ var linkEnd = new Date();
    this.info.linkTimeUs = 1000 * (linkEnd.getTime() - linkStart.getTime());

    assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.linkProgram()', false, true);

    var linkStatus = this.gl.getProgramParameter(this.program, this.gl.LINK_STATUS);
    assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.getProgramParameter()', false, true);
    this.info.linkOk = linkStatus;
    this.info.infoLog = this.gl.getProgramInfoLog(this.program);
    if (!this.info.linkOk)
        bufferedLogToConsole("program linking: " + this.info.infoLog);
};

/**
 * return {boolean}
 */
gluShaderProgram.Program.prototype.getLinkStatus = function() {
    return this.info.linkOk;
};

/**
 * @param {Array<string>} varyings
 * @param {number} bufferMode
 */
gluShaderProgram.Program.prototype.transformFeedbackVaryings = function(varyings, bufferMode) {
    this.gl.transformFeedbackVaryings(this.program, varyings, bufferMode);
    assertMsgOptions(this.gl.getError() == this.gl.NO_ERROR, 'gl.transformFeedbackVaryings()', false, true);
};

/**
 * Assigns gl WebGL context and programSources. Declares array of shaders and new program()
 * @constructor
 * @param {WebGL2RenderingContext} gl WebGL context
 * @param {gluShaderProgram.ProgramSources} programSources
 */
gluShaderProgram.ShaderProgram = function(gl, programSources) {
    this.gl = gl;
    this.programSources = programSources;
    this.shaders = [];
    this.program = new gluShaderProgram.Program(gl);

    /** @type {boolean} */ this.shadersOK = true;

    for (var i = 0; i < programSources.sources.length; i++) {
    /** @type {gluShaderProgram.Shader} */ var shader = new gluShaderProgram.Shader(gl, programSources.sources[i].type);
        shader.setSources(programSources.sources[i].source);
        shader.compile();
        this.shaders.push(shader);
        this.shadersOK = this.shadersOK && shader.getCompileStatus();
        if (!shader.getCompileStatus()) {
            bufferedLogToConsole('gluShaderProgram.Shader:\n' + programSources.sources[i].source);
            bufferedLogToConsole('Compile status: ' + shader.getCompileStatus());
            bufferedLogToConsole('Shader infoLog: ' + shader.info.infoLog);
        }
    }

    if (this.shadersOK) {
        for (var i = 0; i < this.shaders.length; i++)
            this.program.attachShader(this.shaders[i].getShader());

        for (var attrib in programSources.attribLocationBindings)
            this.program.bindAttribLocation(programSources.attribLocationBindings[attrib], attrib);

        if (programSources.transformFeedbackBufferMode)
            if (programSources.transformFeedbackBufferMode === gl.NONE)
                assertMsgOptions(programSources.transformFeedbackVaryings.length === 0, 'Transform feedback sanity check', false, true);
            else
                this.program.transformFeedbackVaryings(programSources.transformFeedbackVaryings, programSources.transformFeedbackBufferMode);

        /* TODO: GLES 3.1: set separable flag */

        this.program.link();

    }
};

/**
 * return {WebGLProgram}
 */
gluShaderProgram.ShaderProgram.prototype.getProgram = function() {
    return this.program.program;
    };

/**
 * @return {gluShaderProgram.ProgramInfo}
 */
gluShaderProgram.ShaderProgram.prototype.getProgramInfo = function() {
    return this.program.info;
};

gluShaderProgram.ShaderProgram.prototype.isOk = function() {
    return this.shadersOK && this.program.getLinkStatus();
};

gluShaderProgram.containerTypes = {
    ATTRIB_LOCATION_BINDING: 0,
    TRANSFORM_FEEDBACK_MODE: 1,
    TRANSFORM_FEEDBACK_VARYING: 2,
    TRANSFORM_FEEDBACK_VARYINGS: 3,
    SHADER_SOURCE: 4,
    PROGRAM_SEPARABLE: 5,
    PROGRAM_SOURCES: 6,

    CONTAINER_TYPE_LAST: 7,
    ATTACHABLE_BEGIN: 0, // ATTRIB_LOCATION_BINDING
    ATTACHABLE_END: 5 + 1 // PROGRAM_SEPARABLE + 1
};

/**
 * @constructor
 */
gluShaderProgram.AttribLocationBinding = function(name, location) {
    this.name = name;
    this.location = location;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.ATTRIB_LOCATION_BINDING;
    };
};

/**
 * @constructor
 */
gluShaderProgram.TransformFeedbackMode = function(mode) {
    this.mode = mode;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.TRANSFORM_FEEDBACK_MODE;
    };
};

/**
 * @constructor
 * @param {string} name
 */
gluShaderProgram.TransformFeedbackVarying = function(name) {
    /** @type {string} */ this.name = name;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.TRANSFORM_FEEDBACK_VARYING;
    };
};

/**
 * @constructor
 * @param {Array<string>} array
 */
gluShaderProgram.TransformFeedbackVaryings = function(array) {
    /** @type {Array<string>} */ this.array = array;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.TRANSFORM_FEEDBACK_VARYINGS;
    };
};

/**
 * @constructor
 */
gluShaderProgram.ProgramSeparable = function(separable) {
    this.separable = separable;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.PROGRAM_SEPARABLE;
    };
};

/**
 * @constructor
 */
gluShaderProgram.VertexSource = function(str) {
    this.shaderType = gluShaderProgram.shaderType.VERTEX;
    this.source = str;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.SHADER_SOURCE;
    };
};

/**
 * @constructor
 */
gluShaderProgram.FragmentSource = function(str) {
    this.shaderType = gluShaderProgram.shaderType.FRAGMENT;
    this.source = str;

    this.getContainerType = function() {
        return gluShaderProgram.containerTypes.SHADER_SOURCE;
    };
};

/**
 * Create gluShaderProgram.ProgramSources.
 * @constructor
 */
gluShaderProgram.ProgramSources = function() {
    /** @type {Array<gluShaderProgram.ShaderInfo>} */ this.sources = [];
    this.attribLocationBindings = [];
    /** @type {Array<string>} */ this.transformFeedbackVaryings = [];
    this.transformFeedbackBufferMode = 0;
    this.separable = false;
};

gluShaderProgram.ProgramSources.prototype.getContainerType = function() {
    return gluShaderProgram.containerTypes.PROGRAM_SOURCES;
};

gluShaderProgram.ProgramSources.prototype.add = function(item) {
    var type = undefined;
    if (typeof(item.getContainerType) == 'function') {
        type = item.getContainerType();
        if (
            typeof(type) != 'number' ||
            type < gluShaderProgram.containerTypes.ATTACHABLE_BEGIN ||
            type >= gluShaderProgram.containerTypes.ATTACHABLE_END
        ) {
            type = undefined;
        }
    }

    switch (type) {
        case gluShaderProgram.containerTypes.ATTRIB_LOCATION_BINDING:
            this.attribLocationBindings.push(item);
            break;

        case gluShaderProgram.containerTypes.TRANSFORM_FEEDBACK_MODE:
            this.transformFeedbackBufferMode = item.mode;
            break;

        case gluShaderProgram.containerTypes.TRANSFORM_FEEDBACK_VARYING:
            this.transformFeedbackVaryings.push(item.name);
            break;

        case gluShaderProgram.containerTypes.TRANSFORM_FEEDBACK_VARYINGS:
            this.transformFeedbackVaryings = this.transformFeedbackVaryings.concat(item.array);
            break;

        case gluShaderProgram.containerTypes.SHADER_SOURCE:
            this.sources.push(new gluShaderProgram.ShaderInfo(item.shaderType, item.source));
            break;

        case gluShaderProgram.containerTypes.PROGRAM_SEPARABLE:
            this.separable = item.separable;
            break;

        default:
            throw new Error('Type \"' + type + '\" cannot be added to gluShaderProgram.ProgramSources.');
            break;
    }

    return this;
};

/**
 * //! Helper for constructing vertex-fragment source pair.
 * @param {string} vertexSrc
 * @param {string} fragmentSrc
 * @return {gluShaderProgram.ProgramSources}
 */
gluShaderProgram.makeVtxFragSources = function(vertexSrc, fragmentSrc) {
    /** @type  {gluShaderProgram.ProgramSources} */ var sources = new gluShaderProgram.ProgramSources();
    sources.sources.push(gluShaderProgram.genVertexSource(vertexSrc));
    sources.sources.push(gluShaderProgram.genFragmentSource(fragmentSrc));
    return sources;
};

});
