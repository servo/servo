/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL (ES) Module
 * -----------------------------------------------
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
 *//*!
 * \file
 * \brief Attribute location tests
 *//*--------------------------------------------------------------------*/

'use strict';
goog.provide('modules.shared.glsAttributeLocationTests');
goog.require('framework.common.tcuStringTemplate');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluShaderUtil');

goog.scope(function() {

    var glsAttributeLocationTests = modules.shared.glsAttributeLocationTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var tcuStringTemplate = framework.common.tcuStringTemplate;

      /**
        * @param {Array<number>} bindings
        * @param {string} attrib
        * @return {number}
        */
        glsAttributeLocationTests.getBoundLocation = function(bindings, attrib) {
                return (bindings[attrib] === undefined ? glsAttributeLocationTests.LocationEnum.UNDEF : bindings[attrib]);
        };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @param {Array<number>} bindings
     * @return {boolean}
     */
    glsAttributeLocationTests.hasAttributeAliasing = function(attributes, bindings) {
        /** @type {Array<boolean>} */ var reservedSpaces = [];

        /** @type {number} */ var location;
        /** @type {number} */ var size;

        for (var attribNdx = 0; attribNdx < attributes.length; attribNdx++) {
              location = glsAttributeLocationTests.getBoundLocation(bindings, attributes[attribNdx].getName());
                size = attributes[attribNdx].getType().getLocationSize();

                if (location != glsAttributeLocationTests.LocationEnum.UNDEF) {

                    for (var i = 0; i < size; i++) {
                          if (reservedSpaces[location + i])
                                return true;
                            reservedSpaces[location + i] = true;
                    }
              }
        }

          return false;
    };

    /**
     * @return {number}
     */
    glsAttributeLocationTests.getMaxAttributeLocations = function() {
        /** @type {number} */ var maxAttribs;
        maxAttribs = /** @type {number} */ (gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
        return maxAttribs;
    };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @return {string}
     */
    glsAttributeLocationTests.generateAttributeDefinitions = function(attributes) {
        /** @type {string} */ var src = '';

        for (var i = 0; i < attributes.length; i++) {
            if (attributes[i].getLayoutLocation() != glsAttributeLocationTests.LocationEnum.UNDEF)
                src += ('layout(location = ' + attributes[i].getLayoutLocation() + ') ');

            src += '${VTX_INPUT} mediump ';
            src += (attributes[i].getType().getName() + ' ');
            src += attributes[i].getName();
            src += (attributes[i].getArraySize() != glsAttributeLocationTests.ArrayEnum.NOT ?
                '[' + attributes[i].getArraySize() + ']' : '');
            src += ';\n';
        }

        return src;
    };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @return {string}
     */
    glsAttributeLocationTests.generateConditionUniformDefinitions = function(attributes) {
        /** @type {string} */ var src = '';
        /** @type {Array<string>} */ var conditions = [];

        for (var i = 0; i < attributes.length; i++) {
            if (attributes[i].getCondition().notEquals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.NEVER)) &&
                attributes[i].getCondition().notEquals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS)))
                if (conditions.indexOf(attributes[i].getCondition().getName()) == -1)
                    conditions.push(attributes[i].getCondition().getName());
        }

        for (var i = 0; i < conditions.length; i++)
            src += ('uniform mediump float u_' + conditions[i] + ';\n');

        return src;
    };

    /**
     * @param {glsAttributeLocationTests.Attribute} attrib
     * @param {number=} id
     * @return {string}
     */
    glsAttributeLocationTests.generateToVec4Expression = function(attrib, id) {
        /** @type {string} */ var src = '';
        id = id === undefined ? -1 : id;

        /** @type {string} */
        var variableName = (attrib.getName() + (attrib.getArraySize() != glsAttributeLocationTests.ArrayEnum.NOT ? '[' + id + ']' : ''));

        switch (attrib.getType().getGLTypeEnum()) {
            case gl.INT_VEC2:
            case gl.UNSIGNED_INT_VEC2:
            case gl.FLOAT_VEC2:
                src += ('vec4(' + variableName + '.xy, ' + variableName + '.yx)');
                break;

            case gl.INT_VEC3:
            case gl.UNSIGNED_INT_VEC3:
            case gl.FLOAT_VEC3:
                src += ('vec4(' + variableName + '.xyz, ' + variableName + '.x)');
                break;

            default:
                src += ('vec4(' + variableName + ')');
                break;
        }

        return src;
    };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @return {string}
     */
    glsAttributeLocationTests.generateOutputCode = function(attributes) {
        /** @type {string} */ var src = '';

        for (var i = 0; i < attributes.length; i++) {
            if (attributes[i].getCondition().equals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.NEVER))) {
                src += '\tif (0 != 0)\n\t{\n';

                if (attributes[i].getArraySize() == glsAttributeLocationTests.ArrayEnum.NOT)
                    src += ('\t\tcolor += ' + glsAttributeLocationTests.generateToVec4Expression(attributes[i]) + ';\n');
                else {
                    for (var j = 0; j < attributes[i].getArraySize(); i++)
                        src += ('\t\tcolor += ' + glsAttributeLocationTests.generateToVec4Expression(attributes[i], j) + ';\n');
                }

                src += '\t}\n';
            } else if (attributes[i].getCondition().equals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS))) {
                if (attributes[i].getArraySize() == glsAttributeLocationTests.ArrayEnum.NOT)
                    src += ('\tcolor += ' + glsAttributeLocationTests.generateToVec4Expression(attributes[i]) + ';\n');
                else {
                    for (var j = 0; j < attributes[i].getArraySize(); j++)
                        src += ('\tcolor += ' + glsAttributeLocationTests.generateToVec4Expression(attributes[i], j) + ';\n');
                }
            } else {
                src += ('\tif (u_' + attributes[i].getCondition().getName() + (attributes[i].getCondition().getNegate() ? ' != ' : ' == ') + '0.0)\n');
                src += '\t{\n';

                if (attributes[i].getArraySize() == glsAttributeLocationTests.ArrayEnum.NOT)
                    src += ('\t\tcolor += ' + glsAttributeLocationTests.generateToVec4Expression(attributes[i]) + ';\n');
                else {
                    for (var j = 0; j < attributes[i].getArraySize(); i++)
                        src += ('\t\tcolor += ' + glsAttributeLocationTests.generateToVec4Expression(attributes[i], j) + ';\n');
                }

                src += '\t}\n';
            }
        }

        return src;
    };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @return {string}
     */
    glsAttributeLocationTests.generateVertexShaderTemplate = function(attributes) {
        /** @type {string} */ var src = '';

        src = '${VERSION}\n' +
        '${VTX_OUTPUT} mediump vec4 v_color;\n' +
        glsAttributeLocationTests.generateAttributeDefinitions(attributes) +
        '\n' +
        glsAttributeLocationTests.generateConditionUniformDefinitions(attributes) +
        '\n' +
        'void main (void)\n' +
        '{\n' +
        '\tmediump vec4 color = vec4(0.0);\n' +
        '\n' +
        glsAttributeLocationTests.generateOutputCode(attributes) +
        '\n' +
        '\tv_color = color;\n' +
        '\tgl_Position = color;\n' +
        '}\n';

        return src;
    };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @param {boolean} attributeAliasing
     * @return {string}
     */
    glsAttributeLocationTests.createVertexShaderSource = function(attributes, attributeAliasing) {
        // \note On GLES only GLSL #version 100 supports aliasing
        /** @type {gluShaderUtil.GLSLVersion} */ var glslVersion = gluShaderUtil.getGLSLVersion(gl);
        glslVersion = attributeAliasing ? gluShaderUtil.GLSLVersion.V100_ES : glslVersion;
        /** @type {boolean} */ var usesInOutQualifiers = gluShaderUtil.glslVersionUsesInOutQualifiers(glslVersion);
        /** @type {string} */ var vertexShaderTemplate = glsAttributeLocationTests.generateVertexShaderTemplate(attributes);

        /** @type {Array<string>} */ var parameters = [];

        parameters['VERSION'] = gluShaderUtil.getGLSLVersionDeclaration(glslVersion);
        parameters['VTX_OUTPUT'] = usesInOutQualifiers ? 'out' : 'varying';
        parameters['VTX_INPUT'] = usesInOutQualifiers ? 'in' : 'attribute';
        parameters['FRAG_INPUT'] = usesInOutQualifiers ? 'in' : 'varying';
        parameters['FRAG_OUTPUT_VAR'] = usesInOutQualifiers ? 'dEQP_FragColor' : 'gl_FragColor';
        parameters['FRAG_OUTPUT_DECLARATION'] = usesInOutQualifiers ? 'layout(location=0) out mediump vec4 dEQP_FragColor;' : '';

        return tcuStringTemplate.specialize(vertexShaderTemplate, parameters);
    };

    /**
     * @param {boolean} attributeAliasing
     * @return {string}
     */
    glsAttributeLocationTests.createFragmentShaderSource = function(attributeAliasing) {
        /** @type {string} */ var fragmentShaderSource = '';
        fragmentShaderSource = '${VERSION}\n' +
        '${FRAG_OUTPUT_DECLARATION}\n' +
        '${FRAG_INPUT} mediump vec4 v_color;\n' +
        'void main (void)\n' +
        '{\n' +
        '\t${FRAG_OUTPUT_VAR} = v_color;\n' +
        '}\n';

        // \note On GLES only GLSL #version 100 supports aliasing
        /** @type {gluShaderUtil.GLSLVersion} */ var glslVersion = gluShaderUtil.getGLSLVersion(gl);
        glslVersion = attributeAliasing ? gluShaderUtil.GLSLVersion.V100_ES : glslVersion;
        /** @type {boolean} */ var usesInOutQualifiers = gluShaderUtil.glslVersionUsesInOutQualifiers(glslVersion);

        /** @type {Array<string>} */ var parameters = [];

        parameters['VERSION'] = gluShaderUtil.getGLSLVersionDeclaration(glslVersion);
        parameters['VTX_OUTPUT'] = usesInOutQualifiers ? 'out' : 'varying';
        parameters['VTX_INPUT'] = usesInOutQualifiers ? 'in' : 'attribute';
        parameters['FRAG_INPUT'] = usesInOutQualifiers ? 'in' : 'varying';
        parameters['FRAG_OUTPUT_VAR'] = usesInOutQualifiers ? 'dEQP_FragColor' : 'gl_FragColor';
        parameters['FRAG_OUTPUT_DECLARATION'] = usesInOutQualifiers ? 'layout(location=0) out mediump vec4 dEQP_FragColor;' : '';

        return tcuStringTemplate.specialize(fragmentShaderSource, parameters);
    };

    glsAttributeLocationTests.logProgram = function(program) {
        var programLinkOk = /** @type {boolean} */ (gl.getProgramParameter(program, gl.LINK_STATUS));
        /**@type{string} */ var programInfoLog = gl.getProgramInfoLog(program);
        /**@type{string} */ var log = 'Program Link Info: ' + programInfoLog +
        'Link result: ' + (programLinkOk ? 'Ok' : 'Fail');

        bufferedLogToConsole(log);
    };

    glsAttributeLocationTests.logAttributes = function(attributes) {
        /**@type{string} */ var log;
        for (var i = 0; i < attributes.length; i++) {

            log = 'Type: ' + attributes[i].getType().getName() +
            ', Name: ' + attributes[i].getName() +
            (attributes[i].getLayoutLocation() != glsAttributeLocationTests.LocationEnum.UNDEF ? ', Layout location ' + attributes[i].getLayoutLocation() : '');

            bufferedLogToConsole(log);
        }
    };

    /**
     * @param {string} vertexShaderSource
     * @param {string} vertexShaderInfoLog
     * @param {boolean} vertexCompileOk
     * @param {string} fragmentShaderSource
     * @param {string} fragmentShaderInfoLog
     * @param {boolean} fragmentCompileOk
     */
    glsAttributeLocationTests.logShaders = function(vertexShaderSource, vertexShaderInfoLog, vertexCompileOk, fragmentShaderSource, fragmentShaderInfoLog, fragmentCompileOk) {

        /**@type{string} */ var log;
        log = '\nVertex Shader Info: ' +
        vertexShaderSource +
        '\nInfo Log: ' +
        vertexShaderInfoLog +
        '\nCompilation result: ' + (vertexCompileOk ? 'Ok' : 'Failed') +

        '\nFragment Shader Info: ' +
        fragmentShaderSource +
        '\nInfo Log: ' +
        fragmentShaderInfoLog +
        '\nCompilation result: ' + (fragmentCompileOk ? 'Ok' : 'Failed');

        bufferedLogToConsole(log);
    };

    /**
     * @param {WebGLProgram} program
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @return {boolean}
     */
    glsAttributeLocationTests.checkActiveAttribQuery = function(program, attributes) {
        /** @type {number} */ var activeAttribCount = 0;
        /** @type {Array<string>} */ var activeAttributes = [];
        /** @type {boolean} */ var isOk = true;
        /** @type {string} */ var log;

        activeAttribCount = /** @type {number} */ (gl.getProgramParameter(program, gl.ACTIVE_ATTRIBUTES));

        /** @type {glsAttributeLocationTests.Attribute} */ var attrib;
        /** @type {boolean} */ var isActive;
        /** @type {WebGLActiveInfo} */ var activeInfo;

        for (var activeAttribNdx = 0; activeAttribNdx < activeAttribCount; activeAttribNdx++) {

            activeInfo = gl.getActiveAttrib(program, activeAttribNdx);

            log = 'glGetActiveAttrib(program' +
            '\nindex= ' + activeAttribNdx +
            '\nsize= ' + activeInfo.size +
            '\ntype= ' + activeInfo.type +
            '\nname= ' + activeInfo.name;

            bufferedLogToConsole(log);

                /** @type {boolean} */ var found = false;

                for (var attribNdx = 0; attribNdx < attributes.length; attribNdx++) {
                    attrib = attributes[attribNdx];

                    if (attrib.getName() == activeInfo.name) {
                        if (activeInfo.type != attrib.getType().getGLTypeEnum()) {

                            log = 'Error: Wrong type ' + attrib.getType().getGLTypeEnum() +
                            ' expected= ' + activeInfo.type;
                            bufferedLogToConsole(log);

                            isOk = false;
                        }

                        if (attrib.getArraySize() == glsAttributeLocationTests.ArrayEnum.NOT) {
                            if (activeInfo.size != 1) {

                                bufferedLogToConsole('Error: Wrong size ' + activeInfo.size + ' expected 1');
                                isOk = false;
                            }
                        } else {
                            if (activeInfo.size != attrib.getArraySize()) {
                                bufferedLogToConsole('Error: Wrong size ' + activeInfo.size + ' expected ' + attrib.getArraySize());

                                isOk = false;
                            }
                        }

                        found = true;
                        break;
                    }
                }

                if (!found) {
                    log = 'Error: Unknown attribute ' + activeInfo.name + ' returned= by glGetActiveAttrib().';
                    bufferedLogToConsole(log);

                    isOk = false;
                }

            activeAttributes.push(activeInfo.name);
        }

        for (var attribNdx = 0; attribNdx < attributes.length; attribNdx++) {
            attrib = attributes[attribNdx];
            isActive = attrib.getCondition().notEquals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.NEVER));

            if (isActive) {
                if (activeAttributes.indexOf(attrib.getName()) == -1) {

                    bufferedLogToConsole('Error: Active attribute ' + attrib.getName() + 'wasn\'t returned by glGetActiveAttrib().');
                    isOk = false;
                }
            } else {
                if (activeAttributes[attrib.getName()] === undefined)
                    bufferedLogToConsole('Note: Inactive attribute ' + attrib.getName() + 'was returned by glGetActiveAttrib().');
            }
        }

        return isOk;
    };

    /**
     * @param {WebGLProgram} program
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @param {Array<number>} bindings
     * @return {boolean}
     */
    glsAttributeLocationTests.checkAttribLocationQuery = function(program, attributes, bindings) {
        /** @type {boolean} */ var isOk = true;
        /** @type {string} */ var log;

        for (var attribNdx = 0; attribNdx < attributes.length; attribNdx++) {
            /** @type {glsAttributeLocationTests.Attribute} */ var attrib = attributes[attribNdx];
            /** @type {number} */ var expectedLocation = (attrib.getLayoutLocation() != glsAttributeLocationTests.LocationEnum.UNDEF ? attrib.getLayoutLocation() : glsAttributeLocationTests.getBoundLocation(bindings, attrib.getName()));
            var location = /** @type {number} */ (gl.getAttribLocation(program, attrib.getName()));

            if (attrib.getCondition().equals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.NEVER)) && location != -1)
                bufferedLogToConsole('Note: Inactive attribute with location.');

            if (attrib.getCondition().notEquals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.NEVER)) && expectedLocation != glsAttributeLocationTests.LocationEnum.UNDEF && expectedLocation != location)
            bufferedLogToConsole('Error: Invalid attribute location.');

            isOk = (attrib.getCondition().equals(glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.NEVER)) || expectedLocation == glsAttributeLocationTests.LocationEnum.UNDEF || expectedLocation == location);
        }

        return isOk;
    };

    /**
     * @param {WebGLProgram} program
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @param {Array<number>} bindings
     * @return {boolean}
     */
    glsAttributeLocationTests.checkQuery = function(program, attributes, bindings) {
        /** @type {boolean} */ var isOk = glsAttributeLocationTests.checkActiveAttribQuery(program, attributes);

        if (!glsAttributeLocationTests.checkAttribLocationQuery(program, attributes, bindings))
            isOk = false;

        return isOk;
    };

    /**
     * @param {WebGLProgram} program
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @param {boolean} attributeAliasing
     * @return {Object}
     */
    glsAttributeLocationTests.createAndAttachShaders = function(program, attributes, attributeAliasing) {
        /** @type {string} */ var vertexShaderSource = glsAttributeLocationTests.createVertexShaderSource(attributes, attributeAliasing);
        /** @type {string} */ var fragmentShaderSource = glsAttributeLocationTests.createFragmentShaderSource(attributeAliasing);

        /** @type {WebGLShader} */ var vertexShader = gl.createShader(gl.VERTEX_SHADER);
        /** @type {WebGLShader} */ var fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);

        gl.shaderSource(vertexShader, vertexShaderSource);
        gl.shaderSource(fragmentShader, fragmentShaderSource);

        gl.compileShader(vertexShader);
        gl.compileShader(fragmentShader);

        gl.attachShader(program, vertexShader);
        gl.attachShader(program, fragmentShader);

        var vertexShaderCompileOk = /** @type {boolean} */ (gl.getShaderParameter(vertexShader, gl.COMPILE_STATUS));
        var fragmentShaderCompileOk = /** @type {boolean} */ (gl.getShaderParameter(fragmentShader, gl.COMPILE_STATUS));

        // log shaders
        glsAttributeLocationTests.logShaders(vertexShaderSource, gl.getShaderInfoLog(vertexShader),
            vertexShaderCompileOk,
            fragmentShaderSource, gl.getShaderInfoLog(fragmentShader),
            fragmentShaderCompileOk);

        assertMsgOptions(vertexShaderCompileOk, 'vertex Shader compile failed', false, true);
        assertMsgOptions(fragmentShaderCompileOk, 'fragment Shader compile failed', false, true);

        gl.deleteShader(vertexShader);
        gl.deleteShader(fragmentShader);

        return {first: vertexShader, second: fragmentShader};

    };

    /**
     * @param {WebGLProgram} program
     * @param {Array<glsAttributeLocationTests.Bind>} binds
     */
    glsAttributeLocationTests.bindAttributes = function(program, binds) {
        for (var i = 0; i < binds.length; i++) {
            bufferedLogToConsole('Bind attribute: ' + binds[i].getAttributeName() + ' to ' + binds[i].getLocation());
            gl.bindAttribLocation(program, binds[i].getLocation(), binds[i].getAttributeName());
        }
    };

    /**
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     * @return {string}
     */
    glsAttributeLocationTests.generateTestName = function(type, arraySize) {
        return type.getName() + (arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? '_array_' + arraySize : '');
    };

    /**
     * @constructor
     * @param {string} name
     * @param {number} locationSize
     * @param {number} typeEnum
     */
    glsAttributeLocationTests.AttribType = function(name, locationSize, typeEnum) {
        /** @type {string} */ this.m_name = name;
        /** @type {number} */ this.m_locationSize = locationSize;
        /** @type {number} */ this.m_glTypeEnum = typeEnum;
    };

    /**
     * @return {string}
     */
    glsAttributeLocationTests.AttribType.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @return {number}
     */
    glsAttributeLocationTests.AttribType.prototype.getLocationSize = function() {
        return this.m_locationSize;
    };

    /**
     * @return {number}
     */
    glsAttributeLocationTests.AttribType.prototype.getGLTypeEnum = function() {
        return this.m_glTypeEnum;
    };

    /**
     * @enum {number}
     */
    glsAttributeLocationTests.ConstCond = {
        ALWAYS: 0,
        NEVER: 1
    };

    /**
     * @constructor
     * @param {string} name
     * @param {boolean=} negate
     */
    glsAttributeLocationTests.Cond = function(name, negate) {
        /** @type {boolean} */ this.m_negate = negate === undefined ? false : negate;
        /** @type {string} */ this.m_name = name;
    };

    /**
     * @param {glsAttributeLocationTests.ConstCond} cond
     * @return {glsAttributeLocationTests.Cond}
     */
    glsAttributeLocationTests.NewCondWithEnum = function(cond) {
        var condObj = new glsAttributeLocationTests.Cond('', false);
        condObj.m_name = '__always__';
        condObj.m_negate = (cond != glsAttributeLocationTests.ConstCond.NEVER);

        return condObj;
    };

    /**
     * @param {glsAttributeLocationTests.Cond} other
     * @return {boolean}
     */
    glsAttributeLocationTests.Cond.prototype.equals = function(other) {
        return (this.m_negate == other.m_negate && this.m_name == other.m_name);
    };

    /**
     * @param {glsAttributeLocationTests.Cond} other
     * @return {boolean}
     */
    glsAttributeLocationTests.Cond.prototype.notEquals = function(other) {
        return (!this.equals(other));
    };

    /**
     * @return {string}
     */
    glsAttributeLocationTests.Cond.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @return {boolean}
     */
    glsAttributeLocationTests.Cond.prototype.getNegate = function() {
        return this.m_negate;
    };

    /**
     * @enum {number}
     */
    glsAttributeLocationTests.LocationEnum = {
        UNDEF: -1
    };

    /**
     * @enum {number}
     */
    glsAttributeLocationTests.ArrayEnum = {
        NOT: -1
    };

    /**
     * @constructor
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {string} name
     * @param {number=} layoutLocation
     * @param {glsAttributeLocationTests.Cond=} cond
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.Attribute = function(type, name, layoutLocation, cond, arraySize) {
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        /** @type {string} */ this.m_name = name;
        /** @type {number} */ this.m_layoutLocation = layoutLocation === undefined ? glsAttributeLocationTests.LocationEnum.UNDEF : layoutLocation;
        /** @type {glsAttributeLocationTests.Cond} */ this.m_cond = cond === undefined ?
                                glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS) : cond;
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
    };

    /**
     * @return {glsAttributeLocationTests.AttribType}
     */
    glsAttributeLocationTests.Attribute.prototype.getType = function() {
        return this.m_type;
    };

    /**
     * @return {string}
     */
    glsAttributeLocationTests.Attribute.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @return {number}
     */
    glsAttributeLocationTests.Attribute.prototype.getLayoutLocation = function() {
        return this.m_layoutLocation;
    };

    /**
     * @return {glsAttributeLocationTests.Cond}
     */
    glsAttributeLocationTests.Attribute.prototype.getCondition = function() {
        return this.m_cond;
    };

    /**
     * @return {number}
     */
    glsAttributeLocationTests.Attribute.prototype.getArraySize = function() {
        return this.m_arraySize;
    };

    /**
     * @constructor
     * @param {string} attribute
     * @param {number} location
     */
    glsAttributeLocationTests.Bind = function(attribute, location) {
        /** @type {string} */ this.m_attribute = attribute;
        /** @type {number} */ this.m_location = location;
    };

    /**
     * @return {string}
     */
    glsAttributeLocationTests.Bind.prototype.getAttributeName = function() {
        return this.m_attribute;
    };

    /**
     * @return {number}
     */
    glsAttributeLocationTests.Bind.prototype.getLocation = function() {
        return this.m_location;
    };

    /**
     * @param {Array<glsAttributeLocationTests.Attribute>} attributes
     * @param {Array<glsAttributeLocationTests.Bind>} preAttachBind
     * @param {Array<glsAttributeLocationTests.Bind>} preLinkBind
     * @param {Array<glsAttributeLocationTests.Bind>} postLinkBind
     * @param {boolean} relink
     * @param {boolean=} reattach
     * @param {Array<glsAttributeLocationTests.Attribute>=} reattachAttributes
     */
    glsAttributeLocationTests.runTest = function(attributes, preAttachBind, preLinkBind, postLinkBind, relink, reattach, reattachAttributes) {
        reattach = reattach === undefined ? false : reattach;
        reattachAttributes = reattachAttributes === undefined ? [] : reattachAttributes;

        try {
            /** @type {boolean} */ var isOk = true;
            /** @type {Array<number>} */ var activeBindings = [];

            for (var bindNdx = 0; bindNdx < preAttachBind.length; bindNdx++)
                activeBindings[preAttachBind[bindNdx].getAttributeName()] = preAttachBind[bindNdx].getLocation();

            for (var bindNdx = 0; bindNdx < preLinkBind.length; bindNdx++)
                activeBindings[preLinkBind[bindNdx].getAttributeName()] = preLinkBind[bindNdx].getLocation();

            glsAttributeLocationTests.logAttributes(attributes);

            /** @type {WebGLProgram} */ var program = gl.createProgram();

            if (!preAttachBind.length == 0)
                glsAttributeLocationTests.bindAttributes(program, preAttachBind);

            /** @type {*} */ var shaders = glsAttributeLocationTests.createAndAttachShaders(program, attributes, glsAttributeLocationTests.hasAttributeAliasing(attributes, activeBindings));

            if (!preLinkBind.length == 0)
                glsAttributeLocationTests.bindAttributes(program, preLinkBind);

                gl.linkProgram(program);

                assertMsgOptions(gl.getProgramParameter(program, gl.LINK_STATUS) == true, 'link program failed', false, true);

                glsAttributeLocationTests.logProgram(program);

            if (!glsAttributeLocationTests.checkQuery(program, attributes, activeBindings))
                isOk = false;

            if (!postLinkBind.length == 0) {
                glsAttributeLocationTests.bindAttributes(program, postLinkBind);

                if (!glsAttributeLocationTests.checkQuery(program, attributes, activeBindings))
                    isOk = false;
            }

            if (relink) {
                gl.linkProgram(program);

                assertMsgOptions(gl.getProgramParameter(program, gl.LINK_STATUS) == true, 'link program failed', false, true);

                glsAttributeLocationTests.logProgram(program);

                for (var bindNdx = 0; bindNdx < postLinkBind.length; bindNdx++)
                    activeBindings[postLinkBind[bindNdx].getAttributeName()] = postLinkBind[bindNdx].getLocation();

                if (!glsAttributeLocationTests.checkQuery(program, attributes, activeBindings))
                    isOk = false;
            }

            if (reattach) {
                gl.detachShader(program, shaders.first);
                gl.detachShader(program, shaders.second);

                glsAttributeLocationTests.createAndAttachShaders(program, reattachAttributes, glsAttributeLocationTests.hasAttributeAliasing(reattachAttributes, activeBindings));

                gl.linkProgram(program);

                assertMsgOptions(gl.getProgramParameter(program, gl.LINK_STATUS) == true, 'link program failed', false, true);

                glsAttributeLocationTests.logProgram(program);

                if (!glsAttributeLocationTests.checkQuery(program, reattachAttributes, activeBindings))
                    isOk = false;
            }

            gl.deleteProgram(program);

            assertMsgOptions(isOk, '', true, true);

        } catch (e) {
            if (program)
                gl.deleteProgram(program);

            throw e;
        }
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.BindAttributeTest = function(type, arraySize) {
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.BindAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.BindAttributeTest.prototype.constructor = glsAttributeLocationTests.BindAttributeTest;

    glsAttributeLocationTests.BindAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_0', glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.BindMaxAttributesTest = function(type, arraySize) {
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.BindMaxAttributesTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.BindMaxAttributesTest.prototype.constructor = glsAttributeLocationTests.BindMaxAttributesTest;

    glsAttributeLocationTests.BindMaxAttributesTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {number} */ var ndx = 0;

        bufferedLogToConsole('MAX_VERTEX_ATTRIBS: ' + maxAttributes);

        for (var loc = maxAttributes - (arrayElementCount * this.m_type.getLocationSize()); loc >= 0; loc -= (arrayElementCount * this.m_type.getLocationSize())) {
            attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_' + ndx, glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));
            bindings.push(new glsAttributeLocationTests.Bind('a_' + ndx, loc));
            ndx++;
        }

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.BindHoleAttributeTest = function(type, arraySize) {
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.BindHoleAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.BindHoleAttributeTest.prototype.constructor = glsAttributeLocationTests.BindHoleAttributeTest;

    glsAttributeLocationTests.BindHoleAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0'));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 0));

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_1', glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));

        /** @type {number} */ var ndx = 2;
        for (var loc = 1 + this.m_type.getLocationSize() * arrayElementCount; loc < maxAttributes; loc++) {
            attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx));
            bindings.push(new glsAttributeLocationTests.Bind('a_' + ndx, loc));

            ndx++;
        }

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.PreAttachBindAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'pre_attach', 'pre_attach');
    };

    glsAttributeLocationTests.PreAttachBindAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.PreAttachBindAttributeTest.prototype.constructor = glsAttributeLocationTests.PreAttachBindAttributeTest;

    glsAttributeLocationTests.PreAttachBindAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {number} */ var ndx = 0;

        attributes.push(new glsAttributeLocationTests.Attribute(new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4), 'a_0'));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, bindings, noBindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.PreLinkBindAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'pre_link', 'pre_link');
    };

    glsAttributeLocationTests.PreLinkBindAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.PreLinkBindAttributeTest.prototype.constructor = glsAttributeLocationTests.PreLinkBindAttributeTest;

    glsAttributeLocationTests.PreLinkBindAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {number} */ var ndx = 0;

        attributes.push(new glsAttributeLocationTests.Attribute(new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4), 'a_0'));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, bindings, noBindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.PostLinkBindAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'post_link', 'post_link');
    };

    glsAttributeLocationTests.PostLinkBindAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.PostLinkBindAttributeTest.prototype.constructor = glsAttributeLocationTests.PostLinkBindAttributeTest;

    glsAttributeLocationTests.PostLinkBindAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4), 'a_0'));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, noBindings, noBindings, bindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.BindReattachAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'reattach', 'reattach');
    };

    glsAttributeLocationTests.BindReattachAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.BindReattachAttributeTest.prototype.constructor = glsAttributeLocationTests.BindReattachAttributeTest;

    glsAttributeLocationTests.BindReattachAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {glsAttributeLocationTests.AttribType} */ var vec2 = new glsAttributeLocationTests.AttribType('vec2', 1, gl.FLOAT_VEC2);

        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var reattachAttributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0'));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 1));
        bindings.push(new glsAttributeLocationTests.Bind('a_1', 1));

        reattachAttributes.push(new glsAttributeLocationTests.Attribute(vec2, 'a_1'));

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false, true, reattachAttributes);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.LocationAttributeTest = function(type, arraySize) {
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.LocationAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.LocationAttributeTest.prototype.constructor = glsAttributeLocationTests.LocationAttributeTest;

    glsAttributeLocationTests.LocationAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_0', 3, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));

        glsAttributeLocationTests.runTest(attributes, noBindings, noBindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.LocationMaxAttributesTest = function(type, arraySize) {
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.LocationMaxAttributesTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.LocationMaxAttributesTest.prototype.constructor = glsAttributeLocationTests.LocationMaxAttributesTest;

    glsAttributeLocationTests.LocationMaxAttributesTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {number} */ var ndx = 0;

        bufferedLogToConsole('MAX_VERTEX_ATTRIBS: ' + maxAttributes);

        for (var loc = maxAttributes - (arrayElementCount * this.m_type.getLocationSize()); loc >= 0; loc -= (arrayElementCount * this.m_type.getLocationSize())) {
            attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_' + ndx, loc, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));
            ndx++;
        }

        glsAttributeLocationTests.runTest(attributes, noBindings, noBindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.LocationHoleAttributeTest = function(type, arraySize) {
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.LocationHoleAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.LocationHoleAttributeTest.prototype.constructor = glsAttributeLocationTests.LocationHoleAttributeTest;

    glsAttributeLocationTests.LocationHoleAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0', 0));

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_1', glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));

        /** @type {number} */ var ndx = 2;
        for (var loc = 1 + this.m_type.getLocationSize() * arrayElementCount; loc < maxAttributes; loc++) {
            attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx, loc));
            ndx++;
        }

        glsAttributeLocationTests.runTest(attributes, noBindings, noBindings, noBindings, false);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.MixedAttributeTest = function(type, arraySize) {
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.MixedAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.MixedAttributeTest.prototype.constructor = glsAttributeLocationTests.MixedAttributeTest;

    glsAttributeLocationTests.MixedAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_0', 3, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 4));

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.MixedMaxAttributesTest = function(type, arraySize) {
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.MixedMaxAttributesTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.MixedMaxAttributesTest.prototype.constructor = glsAttributeLocationTests.MixedMaxAttributesTest;

    glsAttributeLocationTests.MixedMaxAttributesTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {number} */ var ndx = 0;

        bufferedLogToConsole('MAX_VERTEX_ATTRIBS: ' + maxAttributes);

        for (var loc = maxAttributes - (arrayElementCount * this.m_type.getLocationSize()); loc >= 0; loc -= (arrayElementCount * this.m_type.getLocationSize())) {
            if ((ndx % 2) != 0)
                attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_' + ndx, loc, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));
            else {
                attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_' + ndx, glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));
                bindings.push(new glsAttributeLocationTests.Bind('a_' + ndx, loc));
            }
            ndx++;
        }

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.MixedHoleAttributeTest = function(type, arraySize) {
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.MixedHoleAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.MixedHoleAttributeTest.prototype.constructor = glsAttributeLocationTests.MixedHoleAttributeTest;

    glsAttributeLocationTests.MixedHoleAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0'));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 0));

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_1', glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));

        /** @type {number} */ var ndx = 2;
        for (var loc = 1 + this.m_type.getLocationSize() * arrayElementCount; loc < maxAttributes; loc++) {
            if ((ndx % 2) != 0)
                attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx, loc));
            else {
                attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx, loc));
                bindings.push(new glsAttributeLocationTests.Bind('a_' + ndx, loc));
            }
            ndx++;
        }

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.BindRelinkAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'relink', 'relink');
    };

    glsAttributeLocationTests.BindRelinkAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.BindRelinkAttributeTest.prototype.constructor = glsAttributeLocationTests.BindRelinkAttributeTest;

    glsAttributeLocationTests.BindRelinkAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var preLinkBindings = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var postLinkBindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0'));
        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_1'));

        preLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 3));
        preLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 5));

        postLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 6));

        glsAttributeLocationTests.runTest(attributes, noBindings, preLinkBindings, postLinkBindings, true);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.BindRelinkHoleAttributeTest = function(type, arraySize) {
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.BindRelinkHoleAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.BindRelinkHoleAttributeTest.prototype.constructor = glsAttributeLocationTests.BindRelinkHoleAttributeTest;

    glsAttributeLocationTests.BindRelinkHoleAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var preLinkBindings = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var postLinkBindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0'));
        preLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 0));

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_1', glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));

        /** @type {number} */ var ndx = 2;
        for (var loc = 1 + this.m_type.getLocationSize() * arrayElementCount; loc < maxAttributes; loc++) {
            attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx));
            preLinkBindings.push(new glsAttributeLocationTests.Bind('a_' + ndx, loc));

            ndx++;
        }

        postLinkBindings.push(new glsAttributeLocationTests.Bind('a_2', 1));

        glsAttributeLocationTests.runTest(attributes, noBindings, preLinkBindings, postLinkBindings, true);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsAttributeLocationTests.AttribType} type
     * @param {number=} arraySize
     */
    glsAttributeLocationTests.MixedRelinkHoleAttributeTest = function(type, arraySize) {
        /** @type {glsAttributeLocationTests.AttribType} */ this.m_type = type;
        /** @type {number} */ this.m_arraySize = arraySize === undefined ? glsAttributeLocationTests.ArrayEnum.NOT : arraySize;
        tcuTestCase.DeqpTest.call(this, glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize), glsAttributeLocationTests.generateTestName(this.m_type, this.m_arraySize));
    };

    glsAttributeLocationTests.MixedRelinkHoleAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.MixedRelinkHoleAttributeTest.prototype.constructor = glsAttributeLocationTests.MixedRelinkHoleAttributeTest;

    glsAttributeLocationTests.MixedRelinkHoleAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {number} */ var maxAttributes = glsAttributeLocationTests.getMaxAttributeLocations();
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {number} */ var arrayElementCount = (this.m_arraySize != glsAttributeLocationTests.ArrayEnum.NOT ? this.m_arraySize : 1);

        /** @type {Array<glsAttributeLocationTests.Bind>} */ var preLinkBindings = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var postLinkBindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0'));
        preLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 0));

        attributes.push(new glsAttributeLocationTests.Attribute(this.m_type, 'a_1', glsAttributeLocationTests.LocationEnum.UNDEF, glsAttributeLocationTests.NewCondWithEnum(glsAttributeLocationTests.ConstCond.ALWAYS), this.m_arraySize));

        /** @type {number} */ var ndx = 2;
        for (var loc = 1 + this.m_type.getLocationSize() * arrayElementCount; loc < maxAttributes; loc++) {
            if ((ndx % 2) != 0)
                attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx, loc));
            else {
                attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_' + ndx));
                preLinkBindings.push(new glsAttributeLocationTests.Bind('a_' + ndx, loc));

            }
            ndx++;
        }

        postLinkBindings.push(new glsAttributeLocationTests.Bind('a_2', 1));

        glsAttributeLocationTests.runTest(attributes, noBindings, preLinkBindings, postLinkBindings, true);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.PreAttachMixedAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'pre_attach', 'pre_attach');
    };

    glsAttributeLocationTests.PreAttachMixedAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.PreAttachMixedAttributeTest.prototype.constructor = glsAttributeLocationTests.PreAttachMixedAttributeTest;

    glsAttributeLocationTests.PreAttachMixedAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4), 'a_0', 1));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, bindings, noBindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.PreLinkMixedAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'pre_link', 'pre_link');
    };

    glsAttributeLocationTests.PreLinkMixedAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.PreLinkMixedAttributeTest.prototype.constructor = glsAttributeLocationTests.PreLinkMixedAttributeTest;

    glsAttributeLocationTests.PreLinkMixedAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4), 'a_0', 1));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.PostLinkMixedAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'post_link', 'post_link');
    };

    glsAttributeLocationTests.PostLinkMixedAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.PostLinkMixedAttributeTest.prototype.constructor = glsAttributeLocationTests.PostLinkMixedAttributeTest;

    glsAttributeLocationTests.PostLinkMixedAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4), 'a_0', 1));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 3));

        glsAttributeLocationTests.runTest(attributes, noBindings, noBindings, bindings, false);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.MixedReattachAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'reattach', 'reattach');
    };

    glsAttributeLocationTests.MixedReattachAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.MixedReattachAttributeTest.prototype.constructor = glsAttributeLocationTests.MixedReattachAttributeTest;

    glsAttributeLocationTests.MixedReattachAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);
        /** @type {glsAttributeLocationTests.AttribType} */ var vec2 = new glsAttributeLocationTests.AttribType('vec2', 1, gl.FLOAT_VEC2);

        /** @type {Array<glsAttributeLocationTests.Bind>} */ var bindings = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var reattachAttributes = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0', 2));
        bindings.push(new glsAttributeLocationTests.Bind('a_0', 1));
        bindings.push(new glsAttributeLocationTests.Bind('a_1', 1));

        reattachAttributes.push(new glsAttributeLocationTests.Attribute(vec2, 'a_1'));

        glsAttributeLocationTests.runTest(attributes, noBindings, bindings, noBindings, false, true, reattachAttributes);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    glsAttributeLocationTests.MixedRelinkAttributeTest = function() {
        tcuTestCase.DeqpTest.call(this, 'relink', 'relink');
    };

    glsAttributeLocationTests.MixedRelinkAttributeTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsAttributeLocationTests.MixedRelinkAttributeTest.prototype.constructor = glsAttributeLocationTests.MixedRelinkAttributeTest;

    glsAttributeLocationTests.MixedRelinkAttributeTest.prototype.iterate = function() {
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var noBindings = [];
        /** @type {glsAttributeLocationTests.AttribType} */ var vec4 = new glsAttributeLocationTests.AttribType('vec4', 1, gl.FLOAT_VEC4);

        /** @type {Array<glsAttributeLocationTests.Attribute>} */ var attributes = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var preLinkBindings = [];
        /** @type {Array<glsAttributeLocationTests.Bind>} */ var postLinkBindings = [];

        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_0', 1));
        attributes.push(new glsAttributeLocationTests.Attribute(vec4, 'a_1'));

        preLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 3));
        preLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 5));

        postLinkBindings.push(new glsAttributeLocationTests.Bind('a_0', 6));

        glsAttributeLocationTests.runTest(attributes, noBindings, preLinkBindings, postLinkBindings, true);
        return tcuTestCase.IterateResult.STOP;
    };

});
