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
goog.provide('functional.gles3.es3fShaderApiTests');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluShaderProgram');
goog.require('functional.gles3.es3fApiCase');

goog.scope(function() {
var es3fShaderApiTests = functional.gles3.es3fShaderApiTests;
var tcuTestCase = framework.common.tcuTestCase;
var es3fApiCase = functional.gles3.es3fApiCase;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var deRandom = framework.delibs.debase.deRandom;
var deString = framework.delibs.debase.deString;

/** @type {WebGL2RenderingContext} */ var gl;

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

var getSimpleShaderSource = function(shaderType) {
    var simpleVertexShaderSource =
        '#version 300 es\n' +
        'void main (void)\n' +
        '{\n' +
        ' gl_Position = vec4(0.0);\n' +
        '}\n';

    var simpleFragmentShaderSource =
        '#version 300 es\n' +
        'layout(location = 0) out mediump vec4 o_fragColor;\n' +
        'void main (void)\n' +
        '{\n' +
        ' o_fragColor = vec4(0.0);\n' +
        '}\n';

    switch (shaderType) {
        case gluShaderProgram.shaderType.VERTEX:
            return simpleVertexShaderSource;
        case gluShaderProgram.shaderType.FRAGMENT:
            return simpleFragmentShaderSource;
        default:
            throw new Error('Invalid shader type');
    }
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.CreateShaderCase = function(name, description, shaderType) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_shaderType = shaderType;
};

setParentClass(es3fShaderApiTests.CreateShaderCase, es3fApiCase.ApiCase);

es3fShaderApiTests.CreateShaderCase.prototype.test = function() {
    var shaderObject = gl.createShader(gluShaderProgram.getGLShaderType(gl, this.m_shaderType));
    this.check(shaderObject != null);
    gl.deleteShader(shaderObject);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.CompileShaderCase = function(name, description, shaderType) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_shaderType = shaderType;
};

setParentClass(es3fShaderApiTests.CompileShaderCase, es3fApiCase.ApiCase);

es3fShaderApiTests.CompileShaderCase.prototype.checkCompileStatus = function(shader) {
    var status = /** @type {boolean} */ (gl.getShaderParameter(shader, gl.COMPILE_STATUS));
    return status;
};

es3fShaderApiTests.CompileShaderCase.prototype.test = function() {
    var shaderObject = gl.createShader(gluShaderProgram.getGLShaderType(gl, this.m_shaderType));
    var shaderSource = getSimpleShaderSource(this.m_shaderType);

    this.check(shaderObject != null);

    gl.shaderSource(shaderObject, shaderSource);
    gl.compileShader(shaderObject);

    this.check(this.checkCompileStatus(shaderObject));

    gl.deleteShader(shaderObject);
};

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ShaderSourceReplaceCase = function(name, description, shaderType) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_shaderType = shaderType;
};

setParentClass(es3fShaderApiTests.ShaderSourceReplaceCase, es3fApiCase.ApiCase);

es3fShaderApiTests.ShaderSourceReplaceCase.prototype.generateFirstSource = function() {
    return getSimpleShaderSource(this.m_shaderType);
};

es3fShaderApiTests.ShaderSourceReplaceCase.prototype.generateSecondSource = function() {
    var source = '#version 300 es\n' +
                 'precision mediump float;\n';

    if (this.m_shaderType == gluShaderProgram.shaderType.FRAGMENT)
        source += 'layout(location = 0) out mediump vec4 o_fragColor;\n';

    source += 'void main()\n'+
            '{\n'+
            ' float variable = 1.0f;\n';

    if (this.m_shaderType == gluShaderProgram.shaderType.VERTEX) source += ' gl_Position = vec4(variable);\n';
    else if (this.m_shaderType == gluShaderProgram.shaderType.FRAGMENT) source += ' o_fragColor = vec4(variable);\n';

    source += '}\n';

    return source;
};

es3fShaderApiTests.ShaderSourceReplaceCase.prototype.test = function() {
    var shaderObject = gl.createShader(gluShaderProgram.getGLShaderType(gl, this.m_shaderType));
    var firstSource = this.generateFirstSource();
    var secondSource = this.generateSecondSource();

    this.check(shaderObject != null);

    gl.shaderSource(shaderObject, firstSource);
    this.check(firstSource == gl.getShaderSource(shaderObject));

    gl.shaderSource(shaderObject, secondSource);
    this.check(secondSource == gl.getShaderSource(shaderObject));

    gl.deleteShader(shaderObject);
};

/**
 * @constructor
 */
es3fShaderApiTests.SourceGenerator = function() {};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 * @return {string}
 */
es3fShaderApiTests.SourceGenerator.prototype.next = function(shaderType) {
    throw new Error('Virtual function. Please override');
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 * @return {boolean}
 */
es3fShaderApiTests.SourceGenerator.prototype.finished = function(shaderType) {
    throw new Error('Virtual function. Please override');
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.SourceGenerator}
 */
es3fShaderApiTests.ConstantShaderGenerator = function(rnd) {
    es3fShaderApiTests.SourceGenerator.call(this);
    this.m_rnd = rnd;
};

setParentClass(es3fShaderApiTests.ConstantShaderGenerator, es3fShaderApiTests.SourceGenerator);

es3fShaderApiTests.SourceGenerator.prototype.next = function(shaderType) {
    var value = this.m_rnd.getFloat(0.0, 1.0);
    var outputName = (shaderType == gluShaderProgram.shaderType.VERTEX) ? 'gl_Position' : 'o_fragColor';

    var out = '#version 300 es\n';

    if (shaderType == gluShaderProgram.shaderType.FRAGMENT)
        out += 'layout(location = 0) out mediump vec4 o_fragColor;\n';

    out += 'void main (void)\n';
    out += '{\n';
    out += ' ' + outputName + ' = vec4(' + value + ');\n';
    out += '}\n';

    return out;
};

es3fShaderApiTests.SourceGenerator.prototype.finished = function(shaderType) {
    return false;
};

// Shader allocation utility

/**
 * @constructor
 * @param {es3fShaderApiTests.SourceGenerator} generator
 */
es3fShaderApiTests.ShaderAllocator = function(generator) {
    this.m_srcGen = generator;
    this.m_shaders = {};
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ShaderAllocator.prototype.createShader = function(shaderType) {
    var shader = new gluShaderProgram.Shader(gl, shaderType);
    this.m_shaders[shaderType] = shader;
    this.setSource(shaderType);
    return shader;
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ShaderAllocator.prototype.deleteShader = function(shaderType) {
    this.m_shaders[shaderType].destroy();
    this.m_shaders[shaderType] = null;
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ShaderAllocator.prototype.setSource = function(shaderType) {
    var source = this.m_srcGen.next(shaderType);
    this.m_shaders[shaderType].setSources(source);
};

/**
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ShaderAllocator.prototype.get = function(shaderType) {
    return this.m_shaders[shaderType];
};

// Base class for simple program API tests

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderApiTests.SimpleProgramCase = function(name, description) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_vertShader = null;
    this.m_fragShader = null;
    this.m_program = null;
};

setParentClass(es3fShaderApiTests.SimpleProgramCase, es3fApiCase.ApiCase);

es3fShaderApiTests.SimpleProgramCase.prototype.compileShaders = function() {
    var vertSource = getSimpleShaderSource(gluShaderProgram.shaderType.VERTEX);
    var fragSource = getSimpleShaderSource(gluShaderProgram.shaderType.FRAGMENT);

    var vertShader = gl.createShader(gl.VERTEX_SHADER);
    var fragShader = gl.createShader(gl.FRAGMENT_SHADER);

    this.check(vertShader != null);
    this.check(fragShader != null);

    gl.shaderSource(vertShader, vertSource);
    gl.compileShader(vertShader);

    gl.shaderSource(fragShader, fragSource);
    gl.compileShader(fragShader);

    this.m_vertShader = vertShader;
    this.m_fragShader = fragShader;
};

es3fShaderApiTests.SimpleProgramCase.prototype.linkProgram = function() {
    var program = gl.createProgram();

    this.check(program != null);

    gl.attachShader(program, this.m_vertShader);
    gl.attachShader(program, this.m_fragShader);

    gl.linkProgram(program);

    this.m_program = program;
};

es3fShaderApiTests.SimpleProgramCase.prototype.cleanup = function() {
    gl.deleteShader(this.m_vertShader);
    gl.deleteShader(this.m_fragShader);
    gl.deleteProgram(this.m_program);
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.SimpleProgramCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderApiTests.DeleteShaderCase = function(name, description) {
    es3fShaderApiTests.SimpleProgramCase.call(this, name, description);
};

setParentClass(es3fShaderApiTests.DeleteShaderCase, es3fShaderApiTests.SimpleProgramCase);

es3fShaderApiTests.DeleteShaderCase.prototype.checkDeleteStatus = function(shader) {
    var status = /** @type {boolean} */ (gl.getShaderParameter(shader, gl.DELETE_STATUS));
    return status;
};

es3fShaderApiTests.DeleteShaderCase.prototype.deleteShaders = function() {
    gl.deleteShader(this.m_vertShader);
    gl.deleteShader(this.m_fragShader);
};

es3fShaderApiTests.DeleteShaderCase.prototype.test = function() {
    this.compileShaders();
    this.linkProgram();

    this.deleteShaders();

    this.check(this.checkDeleteStatus(this.m_vertShader) && this.checkDeleteStatus(this.m_fragShader));

    gl.deleteProgram(this.m_program);

    this.check(!(gl.isShader(this.m_vertShader) || gl.isShader(this.m_fragShader)));
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.SimpleProgramCase}
 * @param {string} name
 * @param {string} description
 */
es3fShaderApiTests.LinkVertexFragmentCase = function(name, description) {
    es3fShaderApiTests.SimpleProgramCase.call(this, name, description);
};

setParentClass(es3fShaderApiTests.LinkVertexFragmentCase, es3fShaderApiTests.SimpleProgramCase);

es3fShaderApiTests.LinkVertexFragmentCase.prototype.checkLinkStatus = function(program) {
    var status = /** @type {boolean} */ (gl.getProgramParameter(program, gl.LINK_STATUS));
    return status;
};

es3fShaderApiTests.LinkVertexFragmentCase.prototype.test = function() {
    this.compileShaders();
    this.linkProgram();

    this.check(this.checkLinkStatus(this.m_program), 'Fail, expected LINK_STATUS to be TRUE.');

    this.cleanup();
};

// Base class for program state persistence cases

/**
 * @constructor
 * @extends {es3fApiCase.ApiCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateCase = function(name, description, shaderType) {
    es3fApiCase.ApiCase.call(this, name, description, gl);
    this.m_shaderType = shaderType;
    this.m_rnd = new deRandom.Random(deString.deStringHash(name) ^ 0x713de0ca);
};

setParentClass(es3fShaderApiTests.ProgramStateCase, es3fApiCase.ApiCase);

/**
 * @param {gluShaderProgram.Program} program
 * @param {es3fShaderApiTests.ShaderAllocator} shaders
 */
es3fShaderApiTests.ProgramStateCase.prototype.buildProgram = function(program, shaders) {
    var vertShader = shaders.createShader(gluShaderProgram.shaderType.VERTEX);
    var fragShader = shaders.createShader(gluShaderProgram.shaderType.FRAGMENT);

    vertShader.compile();
    fragShader.compile();

    program.attachShader(vertShader.getShader());
    program.attachShader(fragShader.getShader());
    program.link();
};

/**
 * @param {gluShaderProgram.Program} program
 * @param {gluShaderProgram.ProgramInfo} reference
 */
es3fShaderApiTests.ProgramStateCase.prototype.verify = function(program, reference) {
    var programInfo = program.getInfo();
    this.check(programInfo.linkOk, 'Fail, link status may only change as a result of linking');

    this.check(programInfo.linkTimeUs == reference.linkTimeUs, 'Fail, reported link time changed.');

    this.check(programInfo.infoLog == reference.infoLog, 'Fail, program infolog changed.');
};

es3fShaderApiTests.ProgramStateCase.prototype.test = function() {
    var sourceGen = new es3fShaderApiTests.ConstantShaderGenerator(this.m_rnd);

    var shaders = new es3fShaderApiTests.ShaderAllocator(sourceGen);
    var program = new gluShaderProgram.Program(gl);

    this.buildProgram(program, shaders);

    if (program.getLinkStatus()) {
        var programInfo = program.getInfo();

        this.executeForProgram(program, shaders);

        this.verify(program, programInfo);

    } else{
        this.check(false, "Fail, couldn't link program.");
    }

};

/**
 * @constructor
 * @extends {es3fShaderApiTests.ProgramStateCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateDetachShaderCase = function(name, description, shaderType) {
    es3fShaderApiTests.ProgramStateCase.call(this, name, description, shaderType);
};

setParentClass(es3fShaderApiTests.ProgramStateDetachShaderCase, es3fShaderApiTests.ProgramStateCase);

es3fShaderApiTests.ProgramStateDetachShaderCase.prototype.executeForProgram = function(program, shaders) {
    var caseShader = shaders.get(this.m_shaderType);
    program.detachShader(caseShader.getShader());
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.ProgramStateCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateReattachShaderCase = function(name, description, shaderType) {
    es3fShaderApiTests.ProgramStateCase.call(this, name, description, shaderType);
};

setParentClass(es3fShaderApiTests.ProgramStateReattachShaderCase, es3fShaderApiTests.ProgramStateCase);

es3fShaderApiTests.ProgramStateReattachShaderCase.prototype.executeForProgram = function(program, shaders) {
    var caseShader = shaders.get(this.m_shaderType);
    program.detachShader(caseShader.getShader());
    program.attachShader(caseShader.getShader());
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.ProgramStateCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateDeleteShaderCase = function(name, description, shaderType) {
    es3fShaderApiTests.ProgramStateCase.call(this, name, description, shaderType);
};

setParentClass(es3fShaderApiTests.ProgramStateDeleteShaderCase, es3fShaderApiTests.ProgramStateCase);

es3fShaderApiTests.ProgramStateDeleteShaderCase.prototype.executeForProgram = function(program, shaders) {
    var caseShader = shaders.get(this.m_shaderType);
    program.detachShader(caseShader.getShader());
    shaders.deleteShader(this.m_shaderType);
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.ProgramStateCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateReplaceShaderCase = function(name, description, shaderType) {
    es3fShaderApiTests.ProgramStateCase.call(this, name, description, shaderType);
};

setParentClass(es3fShaderApiTests.ProgramStateReplaceShaderCase, es3fShaderApiTests.ProgramStateCase);

es3fShaderApiTests.ProgramStateReplaceShaderCase.prototype.executeForProgram = function(program, shaders) {
    var caseShader = shaders.get(this.m_shaderType);
    program.detachShader(caseShader.getShader());
    shaders.deleteShader(this.m_shaderType);
    program.attachShader(shaders.createShader(this.m_shaderType).getShader());
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.ProgramStateCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateRecompileShaderCase = function(name, description, shaderType) {
    es3fShaderApiTests.ProgramStateCase.call(this, name, description, shaderType);
};

setParentClass(es3fShaderApiTests.ProgramStateRecompileShaderCase, es3fShaderApiTests.ProgramStateCase);

es3fShaderApiTests.ProgramStateRecompileShaderCase.prototype.executeForProgram = function(program, shaders) {
    var caseShader = shaders.get(this.m_shaderType);
    caseShader.compile();
};

/**
 * @constructor
 * @extends {es3fShaderApiTests.ProgramStateCase}
 * @param {string} name
 * @param {string} description
 * @param {gluShaderProgram.shaderType} shaderType
 */
es3fShaderApiTests.ProgramStateReplaceSourceCase = function(name, description, shaderType) {
    es3fShaderApiTests.ProgramStateCase.call(this, name, description, shaderType);
};

setParentClass(es3fShaderApiTests.ProgramStateReplaceSourceCase, es3fShaderApiTests.ProgramStateCase);

es3fShaderApiTests.ProgramStateReplaceSourceCase.prototype.executeForProgram = function(program, shaders) {
    var caseShader = shaders.get(this.m_shaderType);
    shaders.setSource(this.m_shaderType);
    caseShader.compile();
};

/**
* @constructor
* @extends {tcuTestCase.DeqpTest}
*/
es3fShaderApiTests.ShaderApiTests = function() {
    tcuTestCase.DeqpTest.call(this, 'shader_api', 'Shader API Cases');
};

es3fShaderApiTests.ShaderApiTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
es3fShaderApiTests.ShaderApiTests.prototype.constructor = es3fShaderApiTests.ShaderApiTests;

es3fShaderApiTests.ShaderApiTests.prototype.init = function() {
    // create and delete shaders
    var createDeleteGroup = new tcuTestCase.DeqpTest('create_delete', 'glCreateShader() tests');
    this.addChild(createDeleteGroup);

    createDeleteGroup.addChild(new es3fShaderApiTests.CreateShaderCase('create_vertex_shader', 'Create vertex shader object', gluShaderProgram.shaderType.VERTEX));
    createDeleteGroup.addChild(new es3fShaderApiTests.CreateShaderCase('create_fragment_shader', 'Create fragment shader object', gluShaderProgram.shaderType.FRAGMENT));

    createDeleteGroup.addChild(new es3fShaderApiTests.DeleteShaderCase('delete_vertex_fragment', 'Delete vertex shader and fragment shader'));

    // compile and link
    var compileLinkGroup = new tcuTestCase.DeqpTest('compile_link', 'Compile and link tests');
    this.addChild(compileLinkGroup);

    compileLinkGroup.addChild(new es3fShaderApiTests.CompileShaderCase('compile_vertex_shader', 'Compile vertex shader', gluShaderProgram.shaderType.VERTEX));
    compileLinkGroup.addChild(new es3fShaderApiTests.CompileShaderCase('compile_fragment_shader', 'Compile fragment shader', gluShaderProgram.shaderType.FRAGMENT));

    compileLinkGroup.addChild(new es3fShaderApiTests.LinkVertexFragmentCase('link_vertex_fragment', 'Link vertex and fragment shaders'));

    // shader source
    var shaderSourceGroup = new tcuTestCase.DeqpTest('shader_source', 'glShaderSource() tests');
    this.addChild(shaderSourceGroup);
    shaderSourceGroup.addChild(new es3fShaderApiTests.ShaderSourceReplaceCase('replace_source_vertex', 'Replace source code of vertex shader', gluShaderProgram.shaderType.VERTEX));
    shaderSourceGroup.addChild(new es3fShaderApiTests.ShaderSourceReplaceCase('replace_source_fragment', 'Replace source code of fragment shader', gluShaderProgram.shaderType.FRAGMENT));

    // link status and infolog
    var linkStatusGroup = new tcuTestCase.DeqpTest('program_state', 'Program state persistence tests');
    this.addChild(linkStatusGroup);

    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateDetachShaderCase('detach_shader_vertex', 'detach vertex shader', gluShaderProgram.shaderType.VERTEX));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateReattachShaderCase('reattach_shader_vertex', 'reattach vertex shader', gluShaderProgram.shaderType.VERTEX));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateDeleteShaderCase('delete_shader_vertex', 'delete vertex shader', gluShaderProgram.shaderType.VERTEX));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateReplaceShaderCase('replace_shader_vertex', 'replace vertex shader object', gluShaderProgram.shaderType.VERTEX));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateRecompileShaderCase('recompile_shader_vertex', 'recompile vertex shader', gluShaderProgram.shaderType.VERTEX));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateReplaceSourceCase('replace_source_vertex', 'replace vertex shader source', gluShaderProgram.shaderType.VERTEX));

    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateDetachShaderCase('detach_shader_fragment', 'detach fragment shader', gluShaderProgram.shaderType.FRAGMENT));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateReattachShaderCase('reattach_shader_fragment', 'reattach fragment shader', gluShaderProgram.shaderType.FRAGMENT));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateDeleteShaderCase('delete_shader_fragment', 'delete fragment shader', gluShaderProgram.shaderType.FRAGMENT));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateReplaceShaderCase('replace_shader_fragment', 'replace fragment shader object', gluShaderProgram.shaderType.FRAGMENT));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateRecompileShaderCase('recompile_shader_fragment', 'recompile fragment shader', gluShaderProgram.shaderType.FRAGMENT));
    linkStatusGroup.addChild(new es3fShaderApiTests.ProgramStateReplaceSourceCase('replace_source_fragment', 'replace fragment shader source', gluShaderProgram.shaderType.FRAGMENT));

};

/**
* Run test
* @param {WebGL2RenderingContext} context
*/
es3fShaderApiTests.run = function(context) {
    gl = context;
    //Set up Test Root parameters
    var state = tcuTestCase.runner;
    state.setRoot(new es3fShaderApiTests.ShaderApiTests());

    //Set up name and description of this test series.
    setCurrentTestName(state.testCases.fullName());
    description(state.testCases.getDescription());

    try {
        //Run test cases
        tcuTestCase.runTestCases();
    }
    catch (err) {
        testFailedOptions('Failed to es3fShaderApiTests.run tests', false);
        tcuTestCase.runner.terminate();
    }
};

});
