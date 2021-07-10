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
goog.provide('functional.gles3.es3fShaderIndexingTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuStringTemplate');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluTexture');
goog.require('modules.shared.glsShaderRenderCase');

goog.scope(function() {
    /** @type {?WebGL2RenderingContext} */ var gl;
    var es3fShaderIndexingTests = functional.gles3.es3fShaderIndexingTests;
    var deMath = framework.delibs.debase.deMath;
    var glsShaderRenderCase = modules.shared.glsShaderRenderCase;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTexture = framework.opengl.gluTexture;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuStringTemplate = framework.common.tcuStringTemplate;
    /**
     * @enum {number}
     */
    es3fShaderIndexingTests.IndexAccessType = {
        STATIC: 0,
        DYNAMIC: 1,
        STATIC_LOOP: 2,
        DYNAMIC_LOOP: 3
    };

    /**
     * @param {es3fShaderIndexingTests.IndexAccessType} accessType
     * @return {string}
     */
    es3fShaderIndexingTests.getIndexAccessTypeName = function(accessType) {
        /** @type {Array<string>} */ var s_names = [
            'static',
            'dynamic',
            'static_loop',
            'dynamic_loop'
        ];
        return s_names[accessType];
    };

    /**
     * @enum {number}
     */
    es3fShaderIndexingTests.VectorAccessType = {
        DIRECT: 0,
        COMPONENT: 1,
        SUBSCRIPT_STATIC: 2,
        SUBSCRIPT_DYNAMIC: 3,
        SUBSCRIPT_STATIC_LOOP: 4,
        SUBSCRIPT_DYNAMIC_LOOP: 5
    };

    /**
     * @param {es3fShaderIndexingTests.VectorAccessType} accessType
     * @return {string}
     */
    es3fShaderIndexingTests.getVectorAccessTypeName = function(accessType) {
        /** @type {Array<string>} */ var s_names = [
            'direct',
            'component',
            'static_subscript',
            'dynamic_subscript',
            'static_loop_subscript',
            'dynamic_loop_subscript'
        ];
        assertMsgOptions(deMath.deInBounds32(accessType, 0, s_names.length), 'Index out of bounds', false, true);
        return s_names[accessType];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayCoordsFloat = function(c) {
        c.color[0] = 1.875 * c.coords[0];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayCoordsVec2 = function(c) {
        var swizzled = deMath.swizzle(c.coords, [0, 1]);
        c.color[0] = 1.875 * swizzled[0];
        c.color[1] = 1.875 * swizzled[1];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayCoordsVec3 = function(c) {
        var swizzled = deMath.swizzle(c.coords, [0, 1, 2]);
        c.color[0] = 1.875 * swizzled[0];
        c.color[1] = 1.875 * swizzled[1];
        c.color[2] = 1.875 * swizzled[2];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayCoordsVec4 = function(c) {
        c.color = deMath.scale(c.coords, 1.875);
    };

    /**
     * @param {gluShaderUtil.DataType} dataType
     * @return {function(glsShaderRenderCase.ShaderEvalContext)}
     */
    es3fShaderIndexingTests.getArrayCoordsEvalFunc = function(dataType) {
        if (dataType === gluShaderUtil.DataType.FLOAT) return es3fShaderIndexingTests.evalArrayCoordsFloat;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC2) return es3fShaderIndexingTests.evalArrayCoordsVec2;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC3) return es3fShaderIndexingTests.evalArrayCoordsVec3;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC4) return es3fShaderIndexingTests.evalArrayCoordsVec4;
        else throw new Error('Invalid data type.');
    };


    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayUniformFloat = function(c) {
        c.color[0] = 1.875 * c.constCoords[0];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayUniformVec2 = function(c) {
        var swizzled = deMath.swizzle(c.constCoords, [0, 1]);
        c.color[0] = 1.875 * swizzled[0];
        c.color[1] = 1.875 * swizzled[1];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayUniformVec3 = function(c) {
        var swizzled = deMath.swizzle(c.constCoords, [0, 1, 2]);
        c.color[0] = 1.875 * swizzled[0];
        c.color[1] = 1.875 * swizzled[1];
        c.color[2] = 1.875 * swizzled[2];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalArrayUniformVec4 = function(c) {
        c.color = deMath.scale(c.constCoords, 1.875);
    };

    /**
     * @param {gluShaderUtil.DataType} dataType
     * @return {function(glsShaderRenderCase.ShaderEvalContext)}
     */
    es3fShaderIndexingTests.getArrayUniformEvalFunc = function(dataType) {
        if (dataType === gluShaderUtil.DataType.FLOAT) return es3fShaderIndexingTests.evalArrayUniformFloat;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC2) return es3fShaderIndexingTests.evalArrayUniformVec2;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC3) return es3fShaderIndexingTests.evalArrayUniformVec3;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC4) return es3fShaderIndexingTests.evalArrayUniformVec4;
        else throw new Error('Invalid data type.');
    };

    /**
     * @constructor
     * @extends {glsShaderRenderCase.ShaderRenderCase}
     * @param {string} name
     * @param {string} description
     * @param {boolean} isVertexCase
     * @param {gluShaderUtil.DataType} varType
     * @param {function(glsShaderRenderCase.ShaderEvalContext)} evalFunc
     * @param {string} vertShaderSource
     * @param {string} fragShaderSource
     */
    es3fShaderIndexingTests.ShaderIndexingCase = function(name, description, isVertexCase, varType, evalFunc, vertShaderSource, fragShaderSource) {
        glsShaderRenderCase.ShaderRenderCase.call(this, name, description, isVertexCase, evalFunc);
        /** @type {gluShaderUtil.DataType} */ this.m_varType = varType;
        /** @type {string} */ this.m_vertShaderSource    = vertShaderSource;
        /** @type {string} */ this.m_fragShaderSource    = fragShaderSource;
    };

    es3fShaderIndexingTests.ShaderIndexingCase.prototype = Object.create(glsShaderRenderCase.ShaderRenderCase.prototype);
    es3fShaderIndexingTests.ShaderIndexingCase.prototype.constructor = es3fShaderIndexingTests.ShaderIndexingCase;

    /**
     * @param {?WebGLProgram} programID
     * @param {Array<number>} constCoords
     */
    es3fShaderIndexingTests.ShaderIndexingCase.prototype.setupUniforms = function(programID, constCoords) {
        /** @type {(Array<number>|Float32Array)} */ var arr = [];
        /** @type {Array<number>} */ var array1d = [];
        /** @type {?WebGLUniformLocation} */ var arrLoc = gl.getUniformLocation(programID, 'u_arr');
        if (arrLoc != null) {
            if (this.m_varType === gluShaderUtil.DataType.FLOAT) {
                arr[0] = constCoords[0];
                arr[1] = constCoords[0] * 0.5;
                arr[2] = constCoords[0] * 0.25;
                arr[3] = constCoords[0] * 0.125;
                gl.uniform1fv(arrLoc, arr);
            }
            else if (this.m_varType === gluShaderUtil.DataType.FLOAT_VEC2) {
                arr[0] = deMath.swizzle(constCoords, [0, 1]);
                arr[1] = deMath.scale(deMath.swizzle(constCoords, [0, 1]), 0.5);
                arr[2] = deMath.scale(deMath.swizzle(constCoords, [0, 1]), 0.25);
                arr[3] = deMath.scale(deMath.swizzle(constCoords, [0, 1]), 0.125);
                for (var i = 0; i < arr.length; i++)
                    array1d = array1d.concat(arr[i]);
                gl.uniform2fv(arrLoc, array1d);
            }
            else if (this.m_varType === gluShaderUtil.DataType.FLOAT_VEC3) {
                arr[0] = deMath.swizzle(constCoords, [0, 1, 2]);
                arr[1] = deMath.scale(deMath.swizzle(constCoords, [0, 1, 2]), 0.5);
                arr[2] = deMath.scale(deMath.swizzle(constCoords, [0, 1, 2]), 0.25);
                arr[3] = deMath.scale(deMath.swizzle(constCoords, [0, 1, 2]), 0.125);
                for (var i = 0; i < arr.length; i++)
                    array1d = array1d.concat(arr[i]);
                gl.uniform3fv(arrLoc, array1d);
            }
            else if (this.m_varType === gluShaderUtil.DataType.FLOAT_VEC4) {
                arr[0] = deMath.swizzle(constCoords, [0,1,2,3]);
                arr[1] = deMath.scale(deMath.swizzle(constCoords, [0, 1, 2, 3]), 0.5);
                arr[2] = deMath.scale(deMath.swizzle(constCoords, [0, 1, 2, 3]), 0.25);
                arr[3] = deMath.scale(deMath.swizzle(constCoords, [0, 1, 2, 3]), 0.125);
                for (var i = 0; i < arr.length; i++)
                    array1d = array1d.concat(arr[i]);
                gl.uniform4fv(arrLoc, array1d);
            }
            else
                throw new Error('u_arr should not have location assigned in this test case');
        }
    };

    /**
     * @param  {string} caseName
     * @param  {string} description
     * @param  {gluShaderUtil.DataType} varType
     * @param  {es3fShaderIndexingTests.IndexAccessType} vertAccess
     * @param  {es3fShaderIndexingTests.IndexAccessType} fragAccess
     * @return {es3fShaderIndexingTests.ShaderIndexingCase}
     */
    es3fShaderIndexingTests.createVaryingArrayCase = function(caseName, description, varType, vertAccess, fragAccess) {
        /** @type {string} */ var vtx = '';
        vtx += '#version 300 es\n' +
               'in highp vec4 a_position;\n' +
               'in highp vec4 a_coords;\n';

        if (vertAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC)
            vtx += 'uniform mediump int ui_zero, ui_one, ui_two, ui_three;\n';
        else if (vertAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP)
            vtx += 'uniform mediump int ui_four;\n';

        vtx += 'out ${PRECISION} ${VAR_TYPE} var[${ARRAY_LEN}];\n' +
               '\n' +
               'void main()\n' +
               '{\n' +
               '    gl_Position = a_position;\n';

        if (vertAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            vtx += '    var[0] = ${VAR_TYPE}(a_coords);\n' +
                   '    var[1] = ${VAR_TYPE}(a_coords) * 0.5;\n' +
                   '    var[2] = ${VAR_TYPE}(a_coords) * 0.25;\n' +
                   '    var[3] = ${VAR_TYPE}(a_coords) * 0.125;\n';
        }
        else if (vertAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            vtx += '    var[ui_zero]  = ${VAR_TYPE}(a_coords);\n' +
                   '    var[ui_one]   = ${VAR_TYPE}(a_coords) * 0.5;\n' +
                   '    var[ui_two]   = ${VAR_TYPE}(a_coords) * 0.25;\n' +
                   '    var[ui_three] = ${VAR_TYPE}(a_coords) * 0.125;\n';
        }
        else if (vertAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            vtx += '	${PRECISION} ${VAR_TYPE} coords = ${VAR_TYPE}(a_coords);\n' +
                   '    for (int i = 0; i < 4; i++)\n' +
                   '    {\n' +
                   '		var[i] = ${VAR_TYPE}(coords);\n' +
                   '        coords = coords * 0.5;\n' +
                   '    }\n';
        }
        else {
            assertMsgOptions(vertAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'Not Dynamic_Loop', false, true);
            vtx += '	${PRECISION} ${VAR_TYPE} coords = ${VAR_TYPE}(a_coords);\n' +
                   '    for (int i = 0; i < ui_four; i++)\n' +
                   '    {\n' +
                   '		var[i] = ${VAR_TYPE}(coords);\n' +
                   '        coords = coords * 0.5;\n' +
                   '    }\n';
        }
        vtx += '}\n';

        /** @type {string} */ var frag = '';
        frag += '#version 300 es\n' +
                'precision mediump int;\n' +
                'layout(location = 0) out mediump vec4 o_color;\n';

        if (fragAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC)
            frag += 'uniform mediump int ui_zero, ui_one, ui_two, ui_three;\n';
        else if (fragAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP)
            frag += 'uniform int ui_four;\n';

        frag += 'in ${PRECISION} ${VAR_TYPE} var[${ARRAY_LEN}];\n' +
                '\n' +
                'void main()\n' +
                '{\n' +
                '	${PRECISION} ${VAR_TYPE} res = ${VAR_TYPE}(0.0);\n';

        if (fragAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            frag += '    res += var[0];\n' +
                    '    res += var[1];\n' +
                    '    res += var[2];\n' +
                    '    res += var[3];\n';
        }
        else if (fragAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            frag += '    res += var[ui_zero];\n' +
                    '    res += var[ui_one];\n' +
                    '    res += var[ui_two];\n' +
                    '    res += var[ui_three];\n';
        }
        else if (fragAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            frag += '    for (int i = 0; i < 4; i++)\n' +
                    '        res += var[i];\n';
        }
        else {
            assertMsgOptions(fragAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'Not Dynamic_Loop', false, true);
            frag += '    for (int i = 0; i < ui_four; i++)\n' +
                    '        res += var[i];\n';
        }
        frag += '	o_color = vec4(res${PADDING});\n' +
                '}\n';

        // Fill in shader templates.
        /** @type {Object} */ var params = {};
	    params['VAR_TYPE'] = gluShaderUtil.getDataTypeName(varType);
	    params['ARRAY_LEN'] = '4';
	    params['PRECISION'] = 'mediump';

        if (varType === gluShaderUtil.DataType.FLOAT)
            params['PADDING'] = ', 0.0, 0.0, 1.0';
        else if (varType === gluShaderUtil.DataType.FLOAT_VEC2)
            params['PADDING'] = ', 0.0, 1.0';
        else if (varType === gluShaderUtil.DataType.FLOAT_VEC3)
            params['PADDING'] = ', 1.0';
        else
            params['PADDING'] = '';

        /** @type {string} */ var vertexShaderSource = tcuStringTemplate.specialize(vtx, params);
    	/** @type {string} */ var fragmentShaderSource = tcuStringTemplate.specialize(frag, params);

        /** @type {function(glsShaderRenderCase.ShaderEvalContext)} */
        var evalFunc = es3fShaderIndexingTests.getArrayCoordsEvalFunc(varType);
        return new es3fShaderIndexingTests.ShaderIndexingCase(caseName, description, true, varType, evalFunc, vertexShaderSource, fragmentShaderSource);
    };

    /**
     * @param {string} caseName
     * @param {string} description
     * @param {boolean} isVertexCase
     * @param {gluShaderUtil.DataType} varType
     * @param {es3fShaderIndexingTests.IndexAccessType} readAccess
     * @return {es3fShaderIndexingTests.ShaderIndexingCase}
     */
    es3fShaderIndexingTests.createUniformArrayCase = function(caseName, description, isVertexCase, varType, readAccess) {
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        vtx += '#version 300 es\n';
        frag += '#version 300 es\n';

        vtx += 'in highp vec4 a_position;\n';
        vtx += 'in highp vec4 a_coords;\n';
        frag += 'layout(location = 0) out mediump vec4 o_color;\n';

        if (isVertexCase) {
            vtx += 'out mediump vec4 v_color;\n';
            frag += 'in mediump vec4 v_color;\n';
        }
        else {
            vtx += 'out mediump vec4 v_coords;\n';
            frag += 'in mediump vec4 v_coords;\n';
        }

        if (readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC)
            op += 'uniform mediump int ui_zero, ui_one, ui_two, ui_three;\n';
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP)
            op += 'uniform mediump int ui_four;\n';

        op += 'uniform ${PRECISION} ${VAR_TYPE} u_arr[${ARRAY_LEN}];\n';

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        vtx += '\n';
        vtx += 'void main()\n';
        vtx += '{\n';
        vtx += '    gl_Position = a_position;\n';

        frag += '\n';
        frag += 'void main()\n';
        frag += '{\n';

        // Read array.
        op += '	${PRECISION} ${VAR_TYPE} res = ${VAR_TYPE}(0.0);\n';
        if (readAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            op += '    res += u_arr[0];\n';
            op += '    res += u_arr[1];\n';
            op += '    res += u_arr[2];\n';
            op += '    res += u_arr[3];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            op += '    res += u_arr[ui_zero];\n';
            op += '    res += u_arr[ui_one];\n';
            op += '    res += u_arr[ui_two];\n';
            op += '    res += u_arr[ui_three];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            op += '    for (int i = 0; i < 4; i++)\n';
            op += '        res += u_arr[i];\n';
        }
        else {
            assertMsgOptions(readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'readAccess not supported.', false, true);
            op += '    for (int i = 0; i < ui_four; i++)\n';
            op += '        res += u_arr[i];\n';
        }

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        if (isVertexCase) {
            vtx += '    v_color = vec4(res${PADDING});\n';
            frag += '    o_color = v_color;\n';
        }
        else {
            vtx += '    v_coords = a_coords;\n';
            frag += '    o_color = vec4(res${PADDING});\n';
        }

        vtx += '}\n';
        frag += '}\n';

        // Fill in shader templates.
        /** @type {Object} */ var params = {};
	    params['VAR_TYPE'] = gluShaderUtil.getDataTypeName(varType);
	    params['ARRAY_LEN'] = '4';
	    params['PRECISION'] = 'mediump';

        if (varType === gluShaderUtil.DataType.FLOAT)
            params['PADDING'] = ', 0.0, 0.0, 1.0';
        else if (varType === gluShaderUtil.DataType.FLOAT_VEC2)
            params['PADDING'] = ', 0.0, 1.0';
        else if (varType === gluShaderUtil.DataType.FLOAT_VEC3)
            params['PADDING'] = ', 1.0';
        else
            params['PADDING'] = '';


        /** @type {string} */ var vertexShaderSource = tcuStringTemplate.specialize(vtx, params);
        /** @type {string} */ var fragmentShaderSource = tcuStringTemplate.specialize(frag, params);

        /** @type {function(glsShaderRenderCase.ShaderEvalContext)} */
        var evalFunc = es3fShaderIndexingTests.getArrayUniformEvalFunc(varType);
        return new es3fShaderIndexingTests.ShaderIndexingCase(caseName, description, isVertexCase, varType, evalFunc, vertexShaderSource, fragmentShaderSource);
    };

    /**
     * @param {string} caseName
     * @param {string} description
     * @param {boolean} isVertexCase
     * @param {gluShaderUtil.DataType} varType
     * @param {es3fShaderIndexingTests.IndexAccessType} writeAccess
     * @param {es3fShaderIndexingTests.IndexAccessType} readAccess
     * @return {es3fShaderIndexingTests.ShaderIndexingCase}
     */
    es3fShaderIndexingTests.createTmpArrayCase = function(caseName, description, isVertexCase, varType, writeAccess, readAccess)    {
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        vtx += '#version 300 es\n';
        frag += '#version 300 es\n';

        vtx += 'in highp vec4 a_position;\n' +
               'in highp vec4 a_coords;\n';
        frag += 'layout(location = 0) out mediump vec4 o_color;\n';

        if (isVertexCase) {
            vtx += 'out mediump vec4 v_color;\n';
            frag += 'in mediump vec4 v_color;\n';
        }
        else {
            vtx += 'out mediump vec4 v_coords;\n';
            frag += 'in mediump vec4 v_coords;\n';
        }

        if (writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC || readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC)
            op += 'uniform mediump int ui_zero, ui_one, ui_two, ui_three;\n';

        if (writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP || readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP)
            op += 'uniform mediump int ui_four;\n';

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        vtx += '\n' +
               'void main()\n' +
               '{\n' +
               '    gl_Position = a_position;\n';

        frag += '\n' +
                'void main()\n' +
                '{\n';

        // Write array.
        if (isVertexCase)
            op += '	${PRECISION} ${VAR_TYPE} coords = ${VAR_TYPE}(a_coords);\n';
        else
            op += '	${PRECISION} ${VAR_TYPE} coords = ${VAR_TYPE}(v_coords);\n';

        op += '	${PRECISION} ${VAR_TYPE} arr[${ARRAY_LEN}];\n';
        if (writeAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            op += '	arr[0] = ${VAR_TYPE}(coords);\n' +
    		      '	arr[1] = ${VAR_TYPE}(coords) * 0.5;\n' +
    		      '	arr[2] = ${VAR_TYPE}(coords) * 0.25;\n' +
    		      '	arr[3] = ${VAR_TYPE}(coords) * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
    		op += '	arr[ui_zero]  = ${VAR_TYPE}(coords);\n' +
    		      '	arr[ui_one]   = ${VAR_TYPE}(coords) * 0.5;\n' +
    		      '	arr[ui_two]   = ${VAR_TYPE}(coords) * 0.25;\n' +
    		      '	arr[ui_three] = ${VAR_TYPE}(coords) * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            op += '    for (int i = 0; i < 4; i++)\n' +
                  '    {\n' +
                  '        arr[i] = ${VAR_TYPE}(coords);\n' +
                  '        coords = coords * 0.5;\n' +
                  '    }\n';
        }
        else {
            assertMsgOptions(writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'writeAccess not supported', false, true);
            op += '    for (int i = 0; i < ui_four; i++)\n' +
                  '    {\n' +
                  '        arr[i] = ${VAR_TYPE}(coords);\n' +
                  '        coords = coords * 0.5;\n' +
                  '    }\n';
        }

        // Read array.
        op += '	${PRECISION} ${VAR_TYPE} res = ${VAR_TYPE}(0.0);\n';
        if (readAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            op += '    res += arr[0];\n' +
                  '    res += arr[1];\n' +
                  '    res += arr[2];\n' +
                  '    res += arr[3];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            op += '    res += arr[ui_zero];\n' +
                  '    res += arr[ui_one];\n' +
                  '    res += arr[ui_two];\n' +
                  '    res += arr[ui_three];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            op += '    for (int i = 0; i < 4; i++)\n' +
                  '        res += arr[i];\n';
        }
        else {
            assertMsgOptions(readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'readAccess not supported.', false, true);
            op += '    for (int i = 0; i < ui_four; i++)\n' +
                  '        res += arr[i];\n';
        }

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        if (isVertexCase) {
            vtx += '    v_color = vec4(res${PADDING});\n';
            frag += '    o_color = v_color;\n';
        }
        else {
            vtx += '    v_coords = a_coords;\n';
            frag += '    o_color = vec4(res${PADDING});\n';
        }

        vtx += '}\n';
        frag += '}\n';

        // Fill in shader templates.
        /** @type {Object} */ var params = {};
    	params["VAR_TYPE"] = gluShaderUtil.getDataTypeName(varType);
    	params["ARRAY_LEN"] = "4";
    	params["PRECISION"] = "mediump";

        if (varType === gluShaderUtil.DataType.FLOAT)
            params['PADDING'] = ', 0.0, 0.0, 1.0';
        else if (varType === gluShaderUtil.DataType.FLOAT_VEC2)
            params['PADDING'] = ', 0.0, 1.0';
        else if (varType === gluShaderUtil.DataType.FLOAT_VEC3)
            params['PADDING'] = ', 1.0';
        else
            params['PADDING'] = '';

    	/** @type {string} */ var vertexShaderSource = tcuStringTemplate.specialize(vtx, params);
    	/** @type {string} */ var fragmentShaderSource = tcuStringTemplate.specialize(frag, params);

        /** @type {function(glsShaderRenderCase.ShaderEvalContext)} */
        var evalFunc = es3fShaderIndexingTests.getArrayCoordsEvalFunc(varType);
        return new es3fShaderIndexingTests.ShaderIndexingCase(caseName, description, isVertexCase, varType, evalFunc, vertexShaderSource, fragmentShaderSource);
    };

    // VECTOR SUBSCRIPT.

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptVec2 = function(c) {
        c.color[0] = c.coords[0] + 0.5 * c.coords[1];
        c.color[1] = c.coords[0] + 0.5 * c.coords[1];
        c.color[2] = c.coords[0] + 0.5 * c.coords[1];
    };
    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptVec3 = function(c) {
        c.color[0] = c.coords[0] + 0.5 * c.coords[1] + 0.25 * c.coords[2];
        c.color[1] = c.coords[0] + 0.5 * c.coords[1] + 0.25 * c.coords[2];
        c.color[2] = c.coords[0] + 0.5 * c.coords[1] + 0.25 * c.coords[2];
    };
    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptVec4 = function(c) {
        c.color[0] = c.coords[0] + 0.5 * c.coords[1] + 0.25 * c.coords[2] + 0.125 * c.coords[3];
        c.color[1] = c.coords[0] + 0.5 * c.coords[1] + 0.25 * c.coords[2] + 0.125 * c.coords[3];
        c.color[2] = c.coords[0] + 0.5 * c.coords[1] + 0.25 * c.coords[2] + 0.125 * c.coords[3];
    };

    /**
     * @param {gluShaderUtil.DataType} dataType
     * @return {function(glsShaderRenderCase.ShaderEvalContext)}
     */
    es3fShaderIndexingTests.getVectorSubscriptEvalFunc = function(dataType) {
        if (dataType === gluShaderUtil.DataType.FLOAT_VEC2) return es3fShaderIndexingTests.evalSubscriptVec2;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC3) return es3fShaderIndexingTests.evalSubscriptVec3;
        else if (dataType === gluShaderUtil.DataType.FLOAT_VEC4) return es3fShaderIndexingTests.evalSubscriptVec4;
        else throw new Error('Invalid data type.');
    };

    /**
     * @param {string} caseName
     * @param {string} description
     * @param {boolean} isVertexCase
     * @param {gluShaderUtil.DataType} varType
     * @param {es3fShaderIndexingTests.VectorAccessType} writeAccess
     * @param {es3fShaderIndexingTests.VectorAccessType} readAccess
     * @return {es3fShaderIndexingTests.ShaderIndexingCase}
     */
    es3fShaderIndexingTests.createVectorSubscriptCase = function(caseName, description, isVertexCase, varType, writeAccess, readAccess) {
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '' ;

        /** @type {number} */ var vecLen = gluShaderUtil.getDataTypeScalarSize(varType);
        /** @type {string} */ var vecLenName = glsShaderRenderCase.getIntUniformName(vecLen);

        vtx += '#version 300 es\n';
        frag += '#version 300 es\n';

        vtx += 'in highp vec4 a_position;\n' +
               'in highp vec4 a_coords;\n';
        frag += 'layout(location = 0) out mediump vec4 o_color;\n';

        if (isVertexCase) {
            vtx += 'out mediump vec3 v_color;\n';
            frag += 'in mediump vec3 v_color;\n';
        }
        else {
            vtx += 'out mediump vec4 v_coords;\n';
            frag += 'in mediump vec4 v_coords;\n';
        }

        if (writeAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC ||
            readAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC){
            op += 'uniform mediump int ui_zero';
            if (vecLen >= 2) op += ', ui_one';
            if (vecLen >= 3) op += ', ui_two';
            if (vecLen >= 4) op += ', ui_three';
            op += ';\n';
        }

        if (writeAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC_LOOP ||
            readAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC_LOOP)
            op += 'uniform mediump int ' + vecLenName + ';\n';

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        vtx += '\n' +
               'void main()\n' +
               '{\n' +
               '    gl_Position = a_position;\n';

        frag += '\n' +
                'void main()\n' +
                '{\n';

        // Write vector.
        if (isVertexCase)
            op += '	${PRECISION} ${VAR_TYPE} coords = ${VAR_TYPE}(a_coords);\n';
        else
            op += '	${PRECISION} ${VAR_TYPE} coords = ${VAR_TYPE}(v_coords);\n';

        op += '	${PRECISION} ${VAR_TYPE} tmp;\n';
        if (writeAccess === es3fShaderIndexingTests.VectorAccessType.DIRECT)
            op += '    tmp = coords.${SWIZZLE} * vec4(1.0, 0.5, 0.25, 0.125).${SWIZZLE};\n';
        else if (writeAccess === es3fShaderIndexingTests.VectorAccessType.COMPONENT) {
            op += '    tmp.x = coords.x;\n';
            if (vecLen >= 2) op += '    tmp.y = coords.y * 0.5;\n';
            if (vecLen >= 3) op += '    tmp.z = coords.z * 0.25;\n';
            if (vecLen >= 4) op += '    tmp.w = coords.w * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_STATIC) {
            op += '    tmp[0] = coords.x;\n';
            if (vecLen >= 2) op += '    tmp[1] = coords.y * 0.5;\n';
            if (vecLen >= 3) op += '    tmp[2] = coords.z * 0.25;\n';
            if (vecLen >= 4) op += '    tmp[3] = coords.w * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC) {
            op += '    tmp[ui_zero]  = coords.x;\n';
            if (vecLen >= 2) op += '    tmp[ui_one]   = coords.y * 0.5;\n';
            if (vecLen >= 3) op += '    tmp[ui_two]   = coords.z * 0.25;\n';
            if (vecLen >= 4) op += '    tmp[ui_three] = coords.w * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_STATIC_LOOP) {
            op += '    for (int i = 0; i < ' + vecLen + '; i++)\n';
            op += '    {\n';
            op += '        tmp[i] = coords.x;\n';
            op += '        coords = coords.${ROT_SWIZZLE} * 0.5;\n';
            op += '    }\n';
        }
        else {
            assertMsgOptions(writeAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC_LOOP, 'writeAccess not supported.', false, true);
            op += '    for (int i = 0; i < ' + vecLenName + '; i++)\n';
            op += '    {\n';
            op += '        tmp[i] = coords.x;\n';
            op += '        coords = coords.${ROT_SWIZZLE} * 0.5;\n';
            op += '    }\n';
        }

        // Read vector.
        op += '	${PRECISION} float res = 0.0;\n';
        if (readAccess === es3fShaderIndexingTests.VectorAccessType.DIRECT)
            op += '	res = dot(tmp, ${VAR_TYPE}(1.0));\n';
        else if (readAccess === es3fShaderIndexingTests.VectorAccessType.COMPONENT) {
            op += '    res += tmp.x;\n';
            if (vecLen >= 2) op += '    res += tmp.y;\n';
            if (vecLen >= 3) op += '    res += tmp.z;\n';
            if (vecLen >= 4) op += '    res += tmp.w;\n';
        }
        else if (readAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_STATIC) {
            op += '    res += tmp[0];\n';
            if (vecLen >= 2) op += '    res += tmp[1];\n';
            if (vecLen >= 3) op += '    res += tmp[2];\n';
            if (vecLen >= 4) op += '    res += tmp[3];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC) {
            op += '    res += tmp[ui_zero];\n';
            if (vecLen >= 2) op += '    res += tmp[ui_one];\n';
            if (vecLen >= 3) op += '    res += tmp[ui_two];\n';
            if (vecLen >= 4) op += '    res += tmp[ui_three];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_STATIC_LOOP) {
            op += '    for (int i = 0; i < ' + vecLen + '; i++)\n';
            op += '        res += tmp[i];\n';
        }
        else {
            assertMsgOptions(readAccess === es3fShaderIndexingTests.VectorAccessType.SUBSCRIPT_DYNAMIC_LOOP, 'readAccess not supported', false, true);
            op += '    for (int i = 0; i < ' + vecLenName + '; i++)\n';
            op += '        res += tmp[i];\n';
        }

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        if (isVertexCase) {
            vtx += '    v_color = vec3(res);\n';
            frag += '    o_color = vec4(v_color.rgb, 1.0);\n';
        }
        else {
            vtx += '    v_coords = a_coords;\n';
            frag += '    o_color = vec4(vec3(res), 1.0);\n';
        }

        vtx += '}\n';
        frag += '}\n';

        // Fill in shader templates.
        /** @type {Array<string>} */ var s_swizzles = ['', 'x', 'xy', 'xyz', 'xyzw'];
        /** @type {Array<string>} */ var s_rotSwizzles = ['', 'x', 'yx', 'yzx', 'yzwx'];

        /** @type {Object} */ var params = {};
        params["VAR_TYPE"] = gluShaderUtil.getDataTypeName(varType);
	    params["PRECISION"] = "mediump";
	    params["SWIZZLE"] = s_swizzles[vecLen];
	    params["ROT_SWIZZLE"] = s_rotSwizzles[vecLen];

        /** @type {string} */ var vertexShaderSource = tcuStringTemplate.specialize(vtx, params);
        /** @type {string} */ var fragmentShaderSource = tcuStringTemplate.specialize(frag, params);

        /** @type {function(glsShaderRenderCase.ShaderEvalContext)} */
        var evalFunc = es3fShaderIndexingTests.getVectorSubscriptEvalFunc(varType);
        return new es3fShaderIndexingTests.ShaderIndexingCase(caseName, description, isVertexCase, varType, evalFunc, vertexShaderSource, fragmentShaderSource);
    };

    // MATRIX SUBSCRIPT.

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat2 = function(c) {
        var swizzle01 = deMath.swizzle(c.coords, [0, 1]);
        var swizzle12 = deMath.swizzle(c.coords, [1, 2]);
        c.color[0] = swizzle01[0] + 0.5 * swizzle12[0];
        c.color[1] = swizzle01[1] + 0.5 * swizzle12[1];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat2x3 = function(c) {
        var swizzle012 = deMath.swizzle(c.coords, [0, 1, 2]);
        var swizzle123 = deMath.swizzle(c.coords, [1, 2, 3]);
        c.color[0] = swizzle012[0] + 0.5 * swizzle123[0];
        c.color[1] = swizzle012[1] + 0.5 * swizzle123[1];
        c.color[2] = swizzle012[2] + 0.5 * swizzle123[2];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat2x4 = function(c) {
        c.color = deMath.add(
            deMath.swizzle(c.coords, [0,1,2,3]),
            deMath.scale(deMath.swizzle(c.coords, [1,2,3,0]), 0.5));
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat3x2 = function(c) {
        var swizzle01 = deMath.swizzle(c.coords, [0, 1]);
        var swizzle12 = deMath.swizzle(c.coords, [1, 2]);
        var swizzle23 = deMath.swizzle(c.coords, [2, 3]);
        c.color[0] = swizzle01[0] + 0.5 * swizzle12[0] + 0.25 * swizzle23[0];
        c.color[1] = swizzle01[1] + 0.5 * swizzle12[1] + 0.25 * swizzle23[1];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat3 = function(c) {
        var swizzle012 = deMath.swizzle(c.coords, [0, 1, 2]);
        var swizzle123 = deMath.swizzle(c.coords, [1, 2, 3]);
        var swizzle230 = deMath.swizzle(c.coords, [2, 3, 0]);
        c.color[0] = swizzle012[0] + 0.5 * swizzle123[0] + 0.25 * swizzle230[0];
        c.color[1] = swizzle012[1] + 0.5 * swizzle123[1] + 0.25 * swizzle230[1];
        c.color[2] = swizzle012[2] + 0.5 * swizzle123[2] + 0.25 * swizzle230[2];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat3x4 = function(c) {
        var swizzle0123 = deMath.swizzle(c.coords, [0, 1, 2, 3]);
        var swizzle1230 = deMath.swizzle(c.coords, [1, 2, 3, 0]);
        var swizzle2301 = deMath.swizzle(c.coords, [2, 3, 0, 1]);
        c.color = deMath.add(
            swizzle0123,
            deMath.add(
                deMath.scale(swizzle1230, 0.5),
                deMath.scale(swizzle2301, 0.25)));
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat4x2 = function(c) {
        var swizzle01 = deMath.swizzle(c.coords, [0, 1]);
        var swizzle12 = deMath.swizzle(c.coords, [1, 2]);
        var swizzle23 = deMath.swizzle(c.coords, [2, 3]);
        var swizzle30 = deMath.swizzle(c.coords, [3, 0]);
        c.color[0] = swizzle01[0] + 0.5 * swizzle12[0] + 0.25 * swizzle23[0] + 0.125 * swizzle30[0];
        c.color[1] = swizzle01[1] + 0.5 * swizzle12[1] + 0.25 * swizzle23[1] + 0.125 * swizzle30[1];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat4x3 = function(c) {
        var swizzle012 = deMath.swizzle(c.coords, [0, 1, 2]);
        var swizzle123 = deMath.swizzle(c.coords, [1, 2, 3]);
        var swizzle230 = deMath.swizzle(c.coords, [2, 3, 0]);
        var swizzle301 = deMath.swizzle(c.coords, [3, 0, 1]);
        c.color[0] = swizzle012[0] + 0.5 * swizzle123[0] + 0.25 * swizzle230[0] + 0.125 * swizzle301[0];
        c.color[1] = swizzle012[1] + 0.5 * swizzle123[1] + 0.25 * swizzle230[1] + 0.125 * swizzle301[1];
        c.color[2] = swizzle012[2] + 0.5 * swizzle123[2] + 0.25 * swizzle230[2] + 0.125 * swizzle301[2];
    };

    /** @param {glsShaderRenderCase.ShaderEvalContext} c */
    es3fShaderIndexingTests.evalSubscriptMat4 = function(c) {
        var swizzle1230 = deMath.swizzle(c.coords, [1, 2, 3, 0]);
        var swizzle2301 = deMath.swizzle(c.coords, [2, 3, 0, 1]);
        var swizzle3012 = deMath.swizzle(c.coords, [3, 0, 1, 2]);
        c.color = deMath.add(
            c.coords,
            deMath.add(
                deMath.scale(swizzle1230, 0.5),
                deMath.add(
                    deMath.scale(swizzle2301, 0.25),
                    deMath.scale(swizzle3012, 0.125))));
    };

    /**
     * @param  {gluShaderUtil.DataType} dataType
     * @return {function(glsShaderRenderCase.ShaderEvalContext)}
     */
    es3fShaderIndexingTests.getMatrixSubscriptEvalFunc = function(dataType) {
        switch (dataType) {
            case gluShaderUtil.DataType.FLOAT_MAT2: return es3fShaderIndexingTests.evalSubscriptMat2;
            case gluShaderUtil.DataType.FLOAT_MAT2X3: return es3fShaderIndexingTests.evalSubscriptMat2x3;
            case gluShaderUtil.DataType.FLOAT_MAT2X4: return es3fShaderIndexingTests.evalSubscriptMat2x4;
            case gluShaderUtil.DataType.FLOAT_MAT3X2: return es3fShaderIndexingTests.evalSubscriptMat3x2;
            case gluShaderUtil.DataType.FLOAT_MAT3: return es3fShaderIndexingTests.evalSubscriptMat3;
            case gluShaderUtil.DataType.FLOAT_MAT3X4: return es3fShaderIndexingTests.evalSubscriptMat3x4;
            case gluShaderUtil.DataType.FLOAT_MAT4X2: return es3fShaderIndexingTests.evalSubscriptMat4x2;
            case gluShaderUtil.DataType.FLOAT_MAT4X3: return es3fShaderIndexingTests.evalSubscriptMat4x3;
            case gluShaderUtil.DataType.FLOAT_MAT4: return es3fShaderIndexingTests.evalSubscriptMat4;
            default:
                throw new Error('Invalid data type.');
        }
    };

    /**
     * @param {string} caseName
     * @param {string} description
     * @param {boolean} isVertexCase
     * @param {gluShaderUtil.DataType} varType
     * @param {es3fShaderIndexingTests.IndexAccessType} writeAccess
     * @param {es3fShaderIndexingTests.IndexAccessType} readAccess
     * @return {es3fShaderIndexingTests.ShaderIndexingCase}
     */
    es3fShaderIndexingTests.createMatrixSubscriptCase = function(caseName, description, isVertexCase, varType, writeAccess, readAccess) {
        /** @type {string} */ var vtx = '';
        /** @type {string} */ var frag = '';
        /** @type {string} */ var op = '';

        /** @type {number} */ var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(varType);
        /** @type {number} */ var numRows = gluShaderUtil.getDataTypeMatrixNumRows(varType);
        /** @type {string} */ var matSizeName = glsShaderRenderCase.getIntUniformName(numCols);
        /** @type {gluShaderUtil.DataType} */ var vecType = gluShaderUtil.getDataTypeFloatVec(numRows);

        vtx += '#version 300 es\n';
        frag += '#version 300 es\n';

        vtx += 'in highp vec4 a_position;\n' +
               'in highp vec4 a_coords;\n';
        frag += 'layout(location = 0) out mediump vec4 o_color;\n';

        if (isVertexCase) {
            vtx += 'out mediump vec4 v_color;\n';
            frag += 'in mediump vec4 v_color;\n';
        }
        else {
            vtx += 'out mediump vec4 v_coords;\n';
            frag += 'in mediump vec4 v_coords;\n';
        }

        if (writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC || readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            op += 'uniform mediump int ui_zero';
            if (numCols >= 2) op += ', ui_one';
            if (numCols >= 3) op += ', ui_two';
            if (numCols >= 4) op += ', ui_three';
            op += ';\n';
        }

        if (writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP || readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP)
            op += 'uniform mediump int ' + matSizeName + ';\n';

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        vtx += '\n' +
               'void main()\n' +
               '{\n' +
               '    gl_Position = a_position;\n';

        frag += '\n' +
                'void main()\n' +
                '{\n';

        // Write matrix.
        if (isVertexCase)
            op += '	${PRECISION} vec4 coords = a_coords;\n';
        else
            op += '	${PRECISION} vec4 coords = v_coords;\n';

        op += '	${PRECISION} ${MAT_TYPE} tmp;\n';
        if (writeAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            op += '	tmp[0] = ${VEC_TYPE}(coords);\n';
            if (numCols >= 2) op += '    tmp[1] = ${VEC_TYPE}(coords.yzwx) * 0.5;\n';
            if (numCols >= 3) op += '    tmp[2] = ${VEC_TYPE}(coords.zwxy) * 0.25;\n';
            if (numCols >= 4) op += '    tmp[3] = ${VEC_TYPE}(coords.wxyz) * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            op += '	tmp[ui_zero]  = ${VEC_TYPE}(coords);\n';
            if (numCols >= 2) op += '    tmp[ui_one]   = ${VEC_TYPE}(coords.yzwx) * 0.5;\n';
            if (numCols >= 3) op += '    tmp[ui_two]   = ${VEC_TYPE}(coords.zwxy) * 0.25;\n';
            if (numCols >= 4) op += '    tmp[ui_three] = ${VEC_TYPE}(coords.wxyz) * 0.125;\n';
        }
        else if (writeAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            op += '    for (int i = 0; i < ' + numCols + '; i++)\n';
            op += '    {\n';
            op += '		tmp[i] = ${VEC_TYPE}(coords);\n';
            op += '        coords = coords.yzwx * 0.5;\n';
            op += '    }\n';
        }
        else {
            assertMsgOptions(writeAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'writeAccess not supported', false, true);
            op += '    for (int i = 0; i < ' + matSizeName + '; i++)\n';
            op += '    {\n';
            op += '        tmp[i] = ${VEC_TYPE}(coords);\n';
            op += '        coords = coords.yzwx * 0.5;\n';
            op += '    }\n';
        }

        // Read matrix.
        op += '	${PRECISION} ${VEC_TYPE} res = ${VEC_TYPE}(0.0);\n';
        if (readAccess === es3fShaderIndexingTests.IndexAccessType.STATIC) {
            op += '    res += tmp[0];\n';
            if (numCols >= 2) op += '    res += tmp[1];\n';
            if (numCols >= 3) op += '    res += tmp[2];\n';
            if (numCols >= 4) op += '    res += tmp[3];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC) {
            op += '    res += tmp[ui_zero];\n';
            if (numCols >= 2) op += '    res += tmp[ui_one];\n';
            if (numCols >= 3) op += '    res += tmp[ui_two];\n';
            if (numCols >= 4) op += '    res += tmp[ui_three];\n';
        }
        else if (readAccess === es3fShaderIndexingTests.IndexAccessType.STATIC_LOOP) {
            op += '    for (int i = 0; i < ' + numCols + '; i++)\n';
            op += '        res += tmp[i];\n';
        }
        else {
            assertMsgOptions(readAccess === es3fShaderIndexingTests.IndexAccessType.DYNAMIC_LOOP, 'readAccess not supported', false, true);
            op += '    for (int i = 0; i < ' + matSizeName + '; i++)\n';
            op += '        res += tmp[i];\n';
        }

        vtx += isVertexCase ? op : '';
        frag += isVertexCase ? '' : op;
        op = '';

        if (isVertexCase) {
            vtx += '    v_color = vec4(res${PADDING});\n';
            frag += '    o_color = v_color;\n';
        }
        else {
            vtx += '    v_coords = a_coords;\n';
            frag += '    o_color = vec4(res${PADDING});\n';
        }

        vtx += '}\n';
        frag += '}\n';

        // Fill in shader templates.

        /** @type {Object} */ var params = {};
        params['MAT_TYPE'] = gluShaderUtil.getDataTypeName(varType);
        params['VEC_TYPE'] = gluShaderUtil.getDataTypeName(vecType);
        params['PRECISION'] = "mediump";


        if (numRows === 2)
              params['PADDING'] = ', 0.0, 1.0';
        else if (numRows === 3)
              params['PADDING'] = ', 1.0';
        else
              params['PADDING'] = '';

      	/** @type {string} */ var vertexShaderSource = tcuStringTemplate.specialize(vtx, params);
      	/** @type {string} */ var fragmentShaderSource = tcuStringTemplate.specialize(frag, params);

        /** @type {function(glsShaderRenderCase.ShaderEvalContext)} */
        var evalFunc = es3fShaderIndexingTests.getMatrixSubscriptEvalFunc(varType);
        return new es3fShaderIndexingTests.ShaderIndexingCase(caseName, description, isVertexCase, varType, evalFunc, vertexShaderSource, fragmentShaderSource);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     */
    es3fShaderIndexingTests.ShaderIndexingTests = function() {
        tcuTestCase.DeqpTest.call(this, 'indexing', 'Indexing Tests');
    };

    es3fShaderIndexingTests.ShaderIndexingTests.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    es3fShaderIndexingTests.ShaderIndexingTests.prototype.constructor = es3fShaderIndexingTests.ShaderIndexingTests;

    es3fShaderIndexingTests.ShaderIndexingTests.prototype.init = function() {
        var testGroup = tcuTestCase.runner.testCases;
        /** @type {Array<gluShaderProgram.shaderType>} */ var s_shaderTypes = [
            gluShaderProgram.shaderType.VERTEX,
            gluShaderProgram.shaderType.FRAGMENT
        ];
        /** @type {Array<gluShaderUtil.DataType>} */ var s_floatAndVecTypes = [
            gluShaderUtil.DataType.FLOAT,
            gluShaderUtil.DataType.FLOAT_VEC2,
            gluShaderUtil.DataType.FLOAT_VEC3,
            gluShaderUtil.DataType.FLOAT_VEC4
        ];
        /** @type {string} */ var name;
        /** @type {string} */ var desc;
        /** @type {string} */ var shaderTypeName;
        /** @type {boolean} */ var isVertexCase;
        /** @type {gluShaderProgram.shaderType} */ var shaderType;
        /** @type {string} */ var writeAccessName;
        /** @type {string} */ var readAccessName;
        // Varying array access cases.
        /** @type {tcuTestCase.DeqpTest} */ var varyingGroup = tcuTestCase.newTest('varying_array', 'Varying array access tests.');
        testGroup.addChild(varyingGroup);
        /** @type {gluShaderUtil.DataType} */ var varType;
        for (var typeNdx = 0; typeNdx < s_floatAndVecTypes.length; typeNdx++) {
            varType = s_floatAndVecTypes[typeNdx];
            for (var vertAccessStr in es3fShaderIndexingTests.IndexAccessType) {
                for (var fragAccessStr in es3fShaderIndexingTests.IndexAccessType) {
                    var vertAccess = es3fShaderIndexingTests.IndexAccessType[vertAccessStr];
                    var fragAccess = es3fShaderIndexingTests.IndexAccessType[fragAccessStr];
                    /** @type {string} */ var vertAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(vertAccess);
                    /** @type {string} */ var fragAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(fragAccess);
                    name = gluShaderUtil.getDataTypeName(varType) + '_' + vertAccessName + '_write_' + fragAccessName + '_read';
                    desc = 'Varying array with ' + vertAccessName + ' write in vertex shader and ' + fragAccessName + ' read in fragment shader.';
                    varyingGroup.addChild(es3fShaderIndexingTests.createVaryingArrayCase(name, desc, varType, vertAccess, fragAccess));
                }
            }
        }

        // Uniform array access cases.
        /** @type {tcuTestCase.DeqpTest} */ var uniformGroup = tcuTestCase.newTest("uniform_array", "Uniform array access tests.");
        testGroup.addChild(uniformGroup);

        for (var typeNdx = 0; typeNdx < s_floatAndVecTypes.length; typeNdx++) {
            varType = s_floatAndVecTypes[typeNdx];
            for (var readAccessStr in es3fShaderIndexingTests.IndexAccessType) {
                var readAccess = es3fShaderIndexingTests.IndexAccessType[readAccessStr];
                readAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(readAccess);
                for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                    shaderType = s_shaderTypes[shaderTypeNdx];
                    shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                    name = gluShaderUtil.getDataTypeName(varType) + "_" + readAccessName + "_read_" + shaderTypeName;
                    desc = "Uniform array with " + readAccessName + " read in " + shaderTypeName + " shader.";
                    isVertexCase = shaderType === gluShaderProgram.shaderType.VERTEX;
                    uniformGroup.addChild(es3fShaderIndexingTests.createUniformArrayCase(name, desc, isVertexCase, varType, readAccess));
                }
            }
        }

        // Temporary array access cases.
        /** @type {tcuTestCase.DeqpTest} */ var tmpGroup = tcuTestCase.newTest("tmp_array", "Temporary array access tests.");
        testGroup.addChild(tmpGroup);

        for (var typeNdx = 0; typeNdx < s_floatAndVecTypes.length; typeNdx++) {
            varType = s_floatAndVecTypes[typeNdx];
            for (var writeAccess in es3fShaderIndexingTests.IndexAccessType) {
                for (var readAccess in es3fShaderIndexingTests.IndexAccessType) {
                    writeAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(es3fShaderIndexingTests.IndexAccessType[writeAccess]);
                    readAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(es3fShaderIndexingTests.IndexAccessType[readAccess]);

                    for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                        shaderType = s_shaderTypes[shaderTypeNdx];
                        shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                        name = gluShaderUtil.getDataTypeName(varType) + "_" + writeAccessName + "_write_" + readAccessName + "_read_" + shaderTypeName;
                        desc = "Temporary array with " + writeAccessName + " write and " + readAccessName + " read in " + shaderTypeName + " shader.";
                        isVertexCase = (shaderType === gluShaderProgram.shaderType.VERTEX);
                        tmpGroup.addChild(es3fShaderIndexingTests.createTmpArrayCase(name, desc, isVertexCase, varType, es3fShaderIndexingTests.IndexAccessType[writeAccess], es3fShaderIndexingTests.IndexAccessType[readAccess]));
                    }
                }
            }
        }

        // Vector indexing with subscripts.

        /** @type {Array<gluShaderUtil.DataType>} */ var s_vectorTypes = [
            gluShaderUtil.DataType.FLOAT_VEC2,
            gluShaderUtil.DataType.FLOAT_VEC3,
            gluShaderUtil.DataType.FLOAT_VEC4
        ];

        for (var typeNdx = 0; typeNdx < s_vectorTypes.length; typeNdx++) {
            /** @type {tcuTestCase.DeqpTest} */ var vecGroup = tcuTestCase.newTest("vector_subscript", "Vector subscript indexing.");
            testGroup.addChild(vecGroup);

            varType = s_vectorTypes[typeNdx];
            for (var writeAccess in es3fShaderIndexingTests.VectorAccessType) {
                for (var readAccess in es3fShaderIndexingTests.VectorAccessType) {
                    writeAccessName = es3fShaderIndexingTests.getVectorAccessTypeName(es3fShaderIndexingTests.VectorAccessType[writeAccess]);
                    readAccessName = es3fShaderIndexingTests.getVectorAccessTypeName(es3fShaderIndexingTests.VectorAccessType[readAccess]);

                    for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                        shaderType = s_shaderTypes[shaderTypeNdx];
                        shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                        name = gluShaderUtil.getDataTypeName(varType) + "_" + writeAccessName + "_write_" + readAccessName + "_read_" + shaderTypeName;
                        desc = "Vector subscript access with " + writeAccessName + " write and " + readAccessName + " read in " + shaderTypeName + " shader.";
                        isVertexCase = shaderType === gluShaderProgram.shaderType.VERTEX;
                        vecGroup.addChild(es3fShaderIndexingTests.createVectorSubscriptCase(name, desc, isVertexCase, varType, es3fShaderIndexingTests.VectorAccessType[writeAccess], es3fShaderIndexingTests.VectorAccessType[readAccess]));
                    }
                }
            }
        }

        // Matrix indexing with subscripts.
        /** @type {Array<tcuTestCase.DeqpTest>} */ var matGroup = [
            tcuTestCase.newTest("matrix_subscript", "Matrix subscript indexing."),
            tcuTestCase.newTest("matrix_subscript", "Matrix subscript indexing."),
            tcuTestCase.newTest("matrix_subscript", "Matrix subscript indexing."),
        ];
        for (var ii = 0; ii < matGroup.length; ++ii) {
            testGroup.addChild(matGroup[ii]);
        }

        /** @type {Array<gluShaderUtil.DataType>} */ var s_matrixTypes = [
            gluShaderUtil.DataType.FLOAT_MAT2,
            gluShaderUtil.DataType.FLOAT_MAT2X3,
            gluShaderUtil.DataType.FLOAT_MAT2X4,
            gluShaderUtil.DataType.FLOAT_MAT3X2,
            gluShaderUtil.DataType.FLOAT_MAT3,
            gluShaderUtil.DataType.FLOAT_MAT3X4,
            gluShaderUtil.DataType.FLOAT_MAT4X2,
            gluShaderUtil.DataType.FLOAT_MAT4X3,
            gluShaderUtil.DataType.FLOAT_MAT4
        ];

        for (var typeNdx = 0; typeNdx < s_matrixTypes.length; typeNdx++) {
            varType = s_matrixTypes[typeNdx];
            for (var writeAccess in es3fShaderIndexingTests.IndexAccessType) {
                for (var readAccess in es3fShaderIndexingTests.IndexAccessType) {
                    writeAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(es3fShaderIndexingTests.IndexAccessType[writeAccess]);
                    readAccessName = es3fShaderIndexingTests.getIndexAccessTypeName(es3fShaderIndexingTests.IndexAccessType[readAccess]);

                    for (var shaderTypeNdx = 0; shaderTypeNdx < s_shaderTypes.length; shaderTypeNdx++) {
                        shaderType = s_shaderTypes[shaderTypeNdx];
                        shaderTypeName = gluShaderProgram.getShaderTypeName(shaderType);
                        name = gluShaderUtil.getDataTypeName(varType) + "_" + writeAccessName + "_write_" + readAccessName + "_read_" + shaderTypeName;
                        desc = "Vector subscript access with " + writeAccessName + " write and " + readAccessName + " read in " + shaderTypeName + " shader.";
                        isVertexCase = shaderType === gluShaderProgram.shaderType.VERTEX;
                        matGroup[typeNdx % matGroup.length].addChild(es3fShaderIndexingTests.createMatrixSubscriptCase(
                            name, desc, isVertexCase, varType, es3fShaderIndexingTests.IndexAccessType[writeAccess], es3fShaderIndexingTests.IndexAccessType[readAccess]));
                    }
                }
            }
        }
    };

    /**
     * Run test
     * @param {WebGL2RenderingContext} context
     */
    es3fShaderIndexingTests.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var state = tcuTestCase.runner;
        state.setRoot(new es3fShaderIndexingTests.ShaderIndexingTests());

        //Set up name and description of this test series.
        setCurrentTestName(state.testCases.fullName());
        description(state.testCases.getDescription());

        try {
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            testFailedOptions('Failed to es3fShaderIndexingTests.run tests', false);
            tcuTestCase.runner.terminate();
        }
    };

});
