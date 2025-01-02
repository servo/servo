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
goog.provide('functional.gles3.es3fTransformFeedbackTests');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluVarType');
goog.require('framework.opengl.gluVarTypeUtil');

goog.scope(function() {

    var es3fTransformFeedbackTests = functional.gles3.es3fTransformFeedbackTests;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var gluVarType = framework.opengl.gluVarType;
    var gluVarTypeUtil = framework.opengl.gluVarTypeUtil;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var deRandom = framework.delibs.debase.deRandom;
    var deMath = framework.delibs.debase.deMath;
    var deString = framework.delibs.debase.deString;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuSurface = framework.common.tcuSurface;
    var tcuImageCompare = framework.common.tcuImageCompare;

    /** @type {WebGL2RenderingContext} */ var gl;

    var setParentClass = function(child, parent) {
        child.prototype = Object.create(parent.prototype);
        child.prototype.constructor = child;
    };

    /**
     * @enum
     */
    es3fTransformFeedbackTests.State = {
        DRAW: 0,
        VERIFY: 1,
        FINISH: 2
    };

    /* Maximum time to wait for query result (in seconds) */
    /** @const */ es3fTransformFeedbackTests.MAX_VERIFY_WAIT = 5;

    /** @const @type {number} */ es3fTransformFeedbackTests.VIEWPORT_WIDTH = 128;
    /** @const @type {number} */ es3fTransformFeedbackTests.VIEWPORT_HEIGHT = 128;
    /** @const @type {number} */ es3fTransformFeedbackTests.BUFFER_GUARD_MULTIPLIER = 2;

    /**
     * Enums for es3fTransformFeedbackTests.interpolation
     * @enum {number}
     */
    es3fTransformFeedbackTests.interpolation = {
        SMOOTH: 0,
        FLAT: 1,
        CENTROID: 2

    };

    /**
     * Returns es3fTransformFeedbackTests.interpolation name: smooth, flat or centroid
     * @param {number} interpol es3fTransformFeedbackTests.interpolation enum value
     * @return {string}
     */
    es3fTransformFeedbackTests.getInterpolationName = function(interpol) {

        switch (interpol) {
        case es3fTransformFeedbackTests.interpolation.SMOOTH: return 'smooth';
        case es3fTransformFeedbackTests.interpolation.FLAT: return 'flat';
        case es3fTransformFeedbackTests.interpolation.CENTROID: return 'centroid';
        default:
            throw new Error('Unrecognized es3fTransformFeedbackTests.interpolation name ' + interpol);
       }

    };

    /**
     * @struct
     * @param {string} name
     * @param {gluVarType.VarType} type
     * @param {number} interpolation
     * @constructor
     */
    es3fTransformFeedbackTests.Varying = function(name, type, interpolation) {
        this.name = name;
        this.type = type;
        this.interpolation = interpolation;
    };

    /** es3fTransformFeedbackTests.findAttributeNameEquals
     * Replaces original implementation of "VaryingNameEquals" and "AttributeNameEquals" in the C++ version
     * Returns an es3fTransformFeedbackTests.Attribute or es3fTransformFeedbackTests.Varying object which matches its name with the passed string value in the function
     * @param {(Array<es3fTransformFeedbackTests.Attribute> | Array<es3fTransformFeedbackTests.Varying>)} array
     * @param {string} name
     * @return { (es3fTransformFeedbackTests.Attribute | es3fTransformFeedbackTests.Varying | null)}
     */
    es3fTransformFeedbackTests.findAttributeNameEquals = function(array, name) {
        for (var pos = 0; pos < array.length; pos++) {
            if (array[pos].name === name) {
                return array[pos];
            }
        }
        return null;
    };

    /**
     * @struct
     * @param {string} name
     * @param {gluVarType.VarType} type
     * @param {number} offset
     * @constructor
     */
    es3fTransformFeedbackTests.Attribute = function(name, type, offset) {
        this.name = name;
        this.type = type;
        this.offset = offset;
    };

    /**
     * Constructs an es3fTransformFeedbackTests.Output object
     * @constructor
     */
    es3fTransformFeedbackTests.Output = function() {
        /** @type {number} */ this.bufferNdx = 0;
        /** @type {number} */ this.offset = 0;
        /** @type {string} */ this.name;
        /** @type {gluVarType.VarType} */ this.type = null;
        /** @type {Array<es3fTransformFeedbackTests.Attribute>} */ this.inputs = [];
    };

    /**
     * Constructs an object type es3fTransformFeedbackTests.DrawCall.
     * Contains the number of elements as well as whether the Transform Feedback is enabled or not.
     * @struct
     * @param {number} numElements
     * @param {boolean} tfEnabled is Transform Feedback enabled or not
     * @constructor
     */
    es3fTransformFeedbackTests.DrawCall = function(numElements, tfEnabled) {
        this.numElements = numElements;
        this.transformFeedbackEnabled = tfEnabled;
    };

    /**
     * @constructor
     */
    es3fTransformFeedbackTests.ProgramSpec = function() {

    /** @type {Array<gluVarType.StructType>} */ var m_structs = [];
    /** @type {Array<es3fTransformFeedbackTests.Varying>} */ var m_varyings = [];
    /** @type {Array<string>} */ var m_transformFeedbackVaryings = [];

        this.createStruct = function(name) {
            var struct = gluVarType.newStructType(name);
            m_structs.push(struct);
            return struct;
        };

        this.addVarying = function(name, type, interp) {
            m_varyings.push(new es3fTransformFeedbackTests.Varying(name, type, interp));
        };

        this.addTransformFeedbackVarying = function(name) {
            m_transformFeedbackVaryings.push(name);
        };

        this.getStructs = function() {
            return m_structs;
        };
        this.getVaryings = function() {
            return m_varyings;
        };
        this.getTransformFeedbackVaryings = function() {
            return m_transformFeedbackVaryings;
        };

        this.isPointSizeUsed = function() {
            for (var i = 0; i < m_transformFeedbackVaryings.length; ++i) {
                if (m_transformFeedbackVaryings[i] == 'gl_PointSize') return true;
            }
            return false;
        };

    };

    /** Returns if the program is supported or not
     * @param {es3fTransformFeedbackTests.ProgramSpec} spec
     * @param {number} tfMode
     * @return {boolean}
     */
    es3fTransformFeedbackTests.isProgramSupported = function(spec, tfMode) {
        var maxVertexAttribs = Number(gl.getParameter(gl.MAX_VERTEX_ATTRIBS));
        var maxTfInterleavedComponents = Number(gl.getParameter(gl.MAX_TRANSFORM_FEEDBACK_INTERLEAVED_COMPONENTS));
        var maxTfSeparateAttribs = Number(gl.getParameter(gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_ATTRIBS));
        var maxTfSeparateComponents = Number(gl.getParameter(gl.MAX_TRANSFORM_FEEDBACK_SEPARATE_COMPONENTS));

        // Check vertex attribs.
        /** @type {number} */ var totalVertexAttribs = (
            1 /* a_position */ + (spec.isPointSizeUsed() ? 1 : 0)
        );

        for (var i = 0; i < spec.getVaryings().length; ++i) {
            for (var v_iter = new gluVarTypeUtil.VectorTypeIterator(spec.getVaryings()[i].type); !v_iter.end(); v_iter.next()) {
                totalVertexAttribs += 1;
            }
        }

        if (totalVertexAttribs > maxVertexAttribs)
            return false; // Vertex attribute es3fTransformFeedbackTests.count exceeded.

        // check varyings
        /** @type {number} */ var totalTfComponents = 0;
        /** @type {number} */ var totalTfAttribs = 0;
        /** @type {Object.<number, number>} */ var presetNumComponents = {
            gl_Position: 4,
            gl_PointSize: 1
        };
        for (var i = 0; i < spec.getTransformFeedbackVaryings().length; ++i) {
            var name = spec.getTransformFeedbackVaryings()[i];
            var numComponents = 0;

            if (typeof(presetNumComponents[name]) != 'undefined') {
                numComponents = presetNumComponents[name];
            } else {
                var varName = gluVarTypeUtil.parseVariableName(name);
                // find the varying called varName
                /** @type {es3fTransformFeedbackTests.Varying} */ var varying = (function(varyings) {
                    for (var i = 0; i < varyings.length; ++i) {
                        if (varyings[i].name == varName) {
                            return varyings[i];
                        }
                    }
                    return null;
                }(spec.getVaryings()));

                // glu::TypeComponentVector
                var varPath = gluVarTypeUtil.parseTypePath(name, varying.type);
                numComponents = gluVarTypeUtil.getVarType(varying.type, varPath).getScalarSize();
            }

            if (tfMode == gl.SEPARATE_ATTRIBS && numComponents > maxTfSeparateComponents)
                return false; // Per-attribute component es3fTransformFeedbackTests.count exceeded.

            totalTfComponents += numComponents;
            totalTfAttribs += 1;
        }

        if (tfMode == gl.SEPARATE_ATTRIBS && totalTfAttribs > maxTfSeparateAttribs)
            return false;

        if (tfMode == gl.INTERLEAVED_ATTRIBS && totalTfComponents > maxTfInterleavedComponents)
            return false;

        return true;

    };

    /**
     * @param {string} varyingName
     * @param {Array<gluVarTypeUtil.VarTypeComponent>} path
     * @return {string}
     */
    es3fTransformFeedbackTests.getAttributeName = function(varyingName, path) {
    /** @type {string} */ var str = 'a_' + varyingName.substr(/^v_/.test(varyingName) ? 2 : 0);

        for (var i = 0; i < path.length; ++i) {
        /** @type {string} */ var prefix;

            switch (path[i].type) {
                case gluVarTypeUtil.VarTypeComponent.s_Type.STRUCT_MEMBER: prefix = '_m'; break;
                case gluVarTypeUtil.VarTypeComponent.s_Type.ARRAY_ELEMENT: prefix = '_e'; break;
                case gluVarTypeUtil.VarTypeComponent.s_Type.MATRIX_COLUMN: prefix = '_c'; break;
                case gluVarTypeUtil.VarTypeComponent.s_Type.VECTOR_COMPONENT: prefix = '_s'; break;
                default:
                    throw new Error('invalid type in the component path.');
            }
            str += prefix + path[i].index;
        }
        return str;
    };

    /**
     * original definition:
     * static void es3fTransformFeedbackTests.genShaderSources (const es3fTransformFeedbackTests.ProgramSpec& spec, std::string& vertSource, std::string& fragSource, bool pointSizeRequired)
     * in place of the std::string references, this function returns those params in an object
     *
     * @param {es3fTransformFeedbackTests.ProgramSpec} spec
     * @param {boolean} pointSizeRequired
     * @return {Object.<string, string>}
     */
    es3fTransformFeedbackTests.genShaderSources = function(spec, pointSizeRequired) {

        var vtx = { str: null };
        var frag = { str: null };
        var addPointSize = spec.isPointSizeUsed();

        vtx.str = '#version 300 es\n' +
                 'in highp vec4 a_position;\n';
        frag.str = '#version 300 es\n' +
                 'layout(location = 0) out mediump vec4 o_color;\n' +
                 'uniform highp vec4 u_scale;\n' +
                 'uniform highp vec4 u_bias;\n';
        //vtx.str = 'attribute highp vec4 a_position;\n';
        //frag.str = 'uniform highp vec4 u_scale;\n' +
        //         'uniform highp vec4 u_bias;\n';

        if (addPointSize) {
            vtx.str += 'in highp float a_pointSize;\n';
            //vtx.str += 'attribute highp float a_pointSize;\n';
        }

        // Declare attributes.
        for (var i = 0; i < spec.getVaryings().length; ++i) {

        /** @type {string} */ var name = spec.getVaryings()[i].name;
        /** @type {gluVarType.VarType} */ var type = spec.getVaryings()[i].type;

            for (var vecIter = new gluVarTypeUtil.VectorTypeIterator(type); !vecIter.end(); vecIter.next()) {

                /** @type {gluVarType.VarType} */
                var attribType = gluVarTypeUtil.getVarType(type, vecIter.getPath());

                /** @type {string} */
                var attribName = es3fTransformFeedbackTests.getAttributeName(name, vecIter.getPath());
                vtx.str += 'in ' + gluVarType.declareVariable(attribType, attribName) + ';\n';

            }
        }

        // Declare varyings.
        for (var ndx = 0; ndx < 2; ++ndx) {
            var inout = ndx ? 'in' : 'out';
            var shader = ndx ? frag : vtx;

            for (var i = 0; i < spec.getStructs().length; ++i) {
                var struct = spec.getStructs()[i];
                if (struct.hasTypeName()) {
                    shader.str += gluVarType.declareStructType(struct) + ';\n';
                }
            }

            /** @type {Array<es3fTransformFeedbackTests.Varying>} */ var varyings = spec.getVaryings();
            for (var i = 0; i < varyings.length; ++i) {
                var varying = varyings[i];
                shader.str += es3fTransformFeedbackTests.getInterpolationName(varying.interpolation) +
                           ' ' + inout + ' ' +
                           gluVarType.declareVariable(varying.type, varying.name) +
                           ';\n';
            }
        }

        vtx.str += '\nvoid main (void)\n {\n' +
                 '\tgl_Position = a_position;\n';
        frag.str += '\nvoid main (void)\n {\n' +
                 '\thighp vec4 res = vec4(0.0);\n';

        if (addPointSize) {
            vtx.str += '\tgl_PointSize = a_pointSize;\n';
        } else if (pointSizeRequired) {
            vtx.str += '\tgl_PointSize = 1.0;\n';
        }

        for (var i = 0; i < spec.getVaryings().length; ++i) {
            var name = spec.getVaryings()[i].name;
            var type = spec.getVaryings()[i].type;

            for (var vecIter = new gluVarTypeUtil.VectorTypeIterator(type); !vecIter.end(); vecIter.next()) {
                /** @type {gluVarType.VarType} */var subType = gluVarTypeUtil.getVarType(type, vecIter.getPath());
                var attribName = es3fTransformFeedbackTests.getAttributeName(name, vecIter.getPath());

                if (!(
                    subType.isBasicType() &&
                    gluShaderUtil.isDataTypeScalarOrVector(subType.getBasicType())
                )) throw new Error('Not a scalar or vector.');

                // Vertex: assign from attribute.
                vtx.str += '\t' + name + vecIter.toString() + ' = ' + attribName + ';\n';

                // Fragment: add to res variable.
                var scalarSize = gluShaderUtil.getDataTypeScalarSize(subType.getBasicType());

                frag.str += '\tres += ';
                if (scalarSize == 1) frag.str += 'vec4(' + name + vecIter.toString() + ')';
                else if (scalarSize == 2) frag.str += 'vec2(' + name + vecIter.toString() + ').xxyy';
                else if (scalarSize == 3) frag.str += 'vec3(' + name + vecIter.toString() + ').xyzx';
                else if (scalarSize == 4) frag.str += 'vec4(' + name + vecIter.toString() + ')';

                frag.str += ';\n';
            }
        }

        frag.str += '\to_color = res * u_scale + u_bias;\n}\n';
        //frag.str += '\tgl_FragColor = res * u_scale + u_bias;\n}\n';
        vtx.str += '}\n';

        return {
            vertSource: vtx.str,
            fragSource: frag.str
        };
    };

    /**
     * Returns a Shader program
     * @param {es3fTransformFeedbackTests.ProgramSpec} spec
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @return {gluShaderProgram.ShaderProgram}
     */
    es3fTransformFeedbackTests.createVertexCaptureProgram = function(spec, bufferMode, primitiveType) {

    /** @type {Object.<string, string>} */ var source = es3fTransformFeedbackTests.genShaderSources(spec, primitiveType === gluDrawUtil.primitiveType.POINTS /* Is point size required? */);

        var programSources = new gluShaderProgram.ProgramSources();
        programSources.add(new gluShaderProgram.VertexSource(source.vertSource))
                      .add(new gluShaderProgram.FragmentSource(source.fragSource))
                      .add(new gluShaderProgram.TransformFeedbackVaryings(spec.getTransformFeedbackVaryings()))
                      .add(new gluShaderProgram.TransformFeedbackMode(bufferMode));

        return new gluShaderProgram.ShaderProgram(gl, programSources);

    };

    /**
     * @param {Array<es3fTransformFeedbackTests.Attribute>} attributes
     * @param {Array<es3fTransformFeedbackTests.Varying>} varyings
     * @param {boolean} usePointSize
     * @return {number} input stride
     */
    es3fTransformFeedbackTests.computeInputLayout = function(attributes, varyings, usePointSize) {

        var inputStride = 0;

        // Add position
        var dataTypeVec4 = gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT_VEC4, gluShaderUtil.precision.PRECISION_HIGHP);
        attributes.push(new es3fTransformFeedbackTests.Attribute('a_position', dataTypeVec4, inputStride));
        inputStride += 4 * 4; /*sizeof(deUint32)*/

        if (usePointSize) {
            var dataTypeFloat = gluVarType.newTypeBasic(gluShaderUtil.DataType.FLOAT, gluShaderUtil.precision.PRECISION_HIGHP);
            attributes.push(new es3fTransformFeedbackTests.Attribute('a_pointSize', dataTypeFloat, inputStride));
            inputStride += 1 * 4; /*sizeof(deUint32)*/
        }

        for (var i = 0; i < varyings.length; i++) {
            for (var vecIter = new gluVarTypeUtil.VectorTypeIterator(varyings[i].type); !vecIter.end(); vecIter.next()) {
                var type = vecIter.getType(); // originally getType() in getVarType() within gluVARTypeUtil.hpp.
                var name = es3fTransformFeedbackTests.getAttributeName(varyings[i].name, vecIter.getPath());

                attributes.push(new es3fTransformFeedbackTests.Attribute(name, type, inputStride));
                inputStride += gluShaderUtil.getDataTypeScalarSize(type.getBasicType()) * 4; /*sizeof(deUint32)*/
            }
        }

        return inputStride;
    };

    /**
     * @param {Array<es3fTransformFeedbackTests.Output>} transformFeedbackOutputs
     * @param {Array<es3fTransformFeedbackTests.Attribute>} attributes
     * @param {Array<es3fTransformFeedbackTests.Varying>} varyings
     * @param {Array<string>} transformFeedbackVaryings
     * @param {number} bufferMode
     */
    es3fTransformFeedbackTests.computeTransformFeedbackOutputs = function(transformFeedbackOutputs, attributes, varyings, transformFeedbackVaryings, bufferMode) {

        /** @type {number} */ var accumulatedSize = 0;

        // transformFeedbackOutputs.resize(transformFeedbackVaryings.size());
        for (var varNdx = 0; varNdx < transformFeedbackVaryings.length; varNdx++) {
            /** @type {string} */ var name = transformFeedbackVaryings[varNdx];
            /** @type {number} */ var bufNdx = (bufferMode === gl.SEPARATE_ATTRIBS ? varNdx : 0);
            /** @type {number} */ var offset = (bufferMode === gl.SEPARATE_ATTRIBS ? 0 : accumulatedSize);
            /** @type {es3fTransformFeedbackTests.Output} */ var output = new es3fTransformFeedbackTests.Output();

            output.name = name;
            output.bufferNdx = bufNdx;
            output.offset = offset;

            if (name === 'gl_Position') {
                var posIn = es3fTransformFeedbackTests.findAttributeNameEquals(attributes, 'a_position');
                output.type = posIn.type;
                output.inputs.push(posIn);
            } else if (name === 'gl_PointSize') {
                var sizeIn = es3fTransformFeedbackTests.findAttributeNameEquals(attributes, 'a_pointSize');
                output.type = sizeIn.type;
                output.inputs.push(sizeIn);
            } else {
                var varName = gluVarTypeUtil.parseVariableName(name);
                var varying = es3fTransformFeedbackTests.findAttributeNameEquals(varyings, varName);

                var varPath = gluVarTypeUtil.parseTypePath(name, varying.type);
                output.type = gluVarTypeUtil.getVarType(varying.type, varPath);

                // Add all vectorized attributes as inputs.
                for (var iter = new gluVarTypeUtil.VectorTypeIterator(output.type); !iter.end(); iter.next()) {
                    var fullpath = varPath.concat(iter.getPath());
                    var attribName = es3fTransformFeedbackTests.getAttributeName(varName, fullpath);
                    var attrib = es3fTransformFeedbackTests.findAttributeNameEquals(attributes, attribName);
                    output.inputs.push(attrib);
                }
            }
            transformFeedbackOutputs.push(output);
            accumulatedSize += output.type.getScalarSize() * 4; /*sizeof(deUint32)*/
        }
    };

    /**
     * @param {es3fTransformFeedbackTests.Attribute} attrib
     * @param {ArrayBuffer} buffer
     * @param {number} stride
     * @param {number} numElements
     * @param {deRandom.Random} rnd
     */
    es3fTransformFeedbackTests.genAttributeData = function(attrib, buffer, stride, numElements, rnd) {

        /** @type {number} */ var elementSize = 4; /*sizeof(deUint32)*/
        /** @type {boolean} */ var isFloat = gluShaderUtil.isDataTypeFloatOrVec(attrib.type.getBasicType());
        /** @type {boolean} */ var isInt = gluShaderUtil.isDataTypeIntOrIVec(attrib.type.getBasicType());
        /** @type {boolean} */ var isUint = gluShaderUtil.isDataTypeUintOrUVec(attrib.type.getBasicType());

        /** @type {gluShaderUtil.precision} */ var precision = attrib.type.getPrecision();

        /** @type {number} */ var numComps = gluShaderUtil.getDataTypeScalarSize(attrib.type.getBasicType());

        for (var elemNdx = 0; elemNdx < numElements; elemNdx++) {
            for (var compNdx = 0; compNdx < numComps; compNdx++) {
                /** @type {number} */ var offset = attrib.offset + elemNdx * stride + compNdx * elementSize;
                if (isFloat) {
                    var pos = new Float32Array(buffer, offset, 1);
                    switch (precision) {
                        case gluShaderUtil.precision.PRECISION_LOWP: pos[0] = 0.25 * rnd.getInt(0, 4); break;
                        case gluShaderUtil.precision.PRECISION_MEDIUMP: pos[0] = rnd.getFloat(-1e3, 1e3); break;
                        case gluShaderUtil.precision.PRECISION_HIGHP: pos[0] = rnd.getFloat(-1e5, 1e5); break;
                        default: throw new Error('Unknown precision: ' + precision);
                    }
                } else if (isInt) {
                    var pos = new Int32Array(buffer, offset, 1);
                    switch (precision) {
                        case gluShaderUtil.precision.PRECISION_LOWP: pos[0] = rnd.getInt(-128, 127); break;
                        case gluShaderUtil.precision.PRECISION_MEDIUMP: pos[0] = rnd.getInt(-32768, 32767); break;
                        case gluShaderUtil.precision.PRECISION_HIGHP: pos[0] = rnd.getInt(); break;
                        default: throw new Error('Unknown precision: ' + precision);
                    }
                } else if (isUint) {
                    var pos = new Uint32Array(buffer, offset, 1);
                    switch (precision) {
                        case gluShaderUtil.precision.PRECISION_LOWP: pos[0] = rnd.getInt(0, 255); break;
                        case gluShaderUtil.precision.PRECISION_MEDIUMP: pos[0] = rnd.getInt(0, 65535); break;
                        case gluShaderUtil.precision.PRECISION_HIGHP: pos[0] = Math.abs(rnd.getInt()); break;
                        default: throw new Error('Unknown precision: ' + precision);
                    }
                }
            }
        }
    };

    /**
     * @param {Array<es3fTransformFeedbackTests.Attribute>} attributes
     * @param {number} numInputs
     * @param {number} inputStride
     * @param {deRandom.Random} rnd
     * @return {ArrayBuffer}
     */
    es3fTransformFeedbackTests.genInputData = function(attributes, numInputs, inputStride, rnd) {
        var buffer = new ArrayBuffer(numInputs * inputStride);

        var position = es3fTransformFeedbackTests.findAttributeNameEquals(attributes, 'a_position');
        if (!position)
            throw new Error('Position attribute not found.');

        for (var ndx = 0; ndx < numInputs; ndx++) {
            var pos = new Float32Array(buffer, position.offset + inputStride * ndx, 4);
            pos[0] = rnd.getFloat(-1.2, 1.2);
            pos[1] = rnd.getFloat(-1.2, 1.2);
            pos[2] = rnd.getFloat(-1.2, 1.2);
            pos[3] = rnd.getFloat(0.1, 2.0);
        }

        var pointSizePos = es3fTransformFeedbackTests.findAttributeNameEquals(attributes, 'a_pointSize');
        if (pointSizePos) {
            for (var ndx = 0; ndx < numInputs; ndx++) {
                var pos = new Float32Array(buffer, pointSizePos.offset + inputStride * ndx, 1);
                pos[0] = rnd.getFloat(1, 8);
            }
        }

        // Random data for rest of components.
        for (var i = 0; i < attributes.length; i++) {
            if (attributes[i].name != 'a_position' && attributes[i].name != 'a_pointSize')
                es3fTransformFeedbackTests.genAttributeData(attributes[i], buffer, inputStride, numInputs, rnd);
        }

        return buffer;
    };

    /**
     * Returns the number of outputs with the es3fTransformFeedbackTests.count for the Primitives in the Transform Feedback.
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {number} numElements
     * @return {number}
     */
    es3fTransformFeedbackTests.getTransformFeedbackOutputCount = function(primitiveType, numElements) {

    switch (primitiveType) {
        case gluDrawUtil.primitiveType.TRIANGLES: return numElements - numElements % 3;
        case gluDrawUtil.primitiveType.TRIANGLE_STRIP: return Math.max(0, numElements - 2) * 3;
        case gluDrawUtil.primitiveType.TRIANGLE_FAN: return Math.max(0, numElements - 2) * 3;
        case gluDrawUtil.primitiveType.LINES: return numElements - numElements % 2;
        case gluDrawUtil.primitiveType.LINE_STRIP: return Math.max(0, numElements - 1) * 2;
        case gluDrawUtil.primitiveType.LINE_LOOP: return numElements > 1 ? numElements * 2 : 0;
        case gluDrawUtil.primitiveType.POINTS: return numElements;
        default:
            throw new Error('Unrecognized primitiveType ' + primitiveType);
       }

    };

    /**
     * Returns a number with the es3fTransformFeedbackTests.count for the Primitives in the Transform Feedback.
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {number} numElements
     * @return {number}
     */
    es3fTransformFeedbackTests.getTransformFeedbackPrimitiveCount = function(primitiveType, numElements) {

    switch (primitiveType) {
        case gluDrawUtil.primitiveType.TRIANGLES: return Math.floor(numElements / 3);
        case gluDrawUtil.primitiveType.TRIANGLE_STRIP: return Math.max(0, numElements - 2);
        case gluDrawUtil.primitiveType.TRIANGLE_FAN: return Math.max(0, numElements - 2);
        case gluDrawUtil.primitiveType.LINES: return Math.floor(numElements / 2);
        case gluDrawUtil.primitiveType.LINE_STRIP: return Math.max(0, numElements - 1);
        case gluDrawUtil.primitiveType.LINE_LOOP: return numElements > 1 ? numElements : 0;
        case gluDrawUtil.primitiveType.POINTS: return numElements;
        default:
            throw new Error('Unrecognized primitiveType ' + primitiveType);
       }

    };

    /**
     * Returns the type of Primitive Mode: Triangles for all Triangle Primitive's type and same for Line and Points.
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @return {number} primitiveType
     */
    es3fTransformFeedbackTests.getTransformFeedbackPrimitiveMode = function(primitiveType) {

    switch (primitiveType) {
        case gluDrawUtil.primitiveType.TRIANGLES:
        case gluDrawUtil.primitiveType.TRIANGLE_STRIP:
        case gluDrawUtil.primitiveType.TRIANGLE_FAN:
            return gl.TRIANGLES;

        case gluDrawUtil.primitiveType.LINES:
        case gluDrawUtil.primitiveType.LINE_STRIP:
        case gluDrawUtil.primitiveType.LINE_LOOP:
            return gl.LINES;

        case gluDrawUtil.primitiveType.POINTS:
            return gl.POINTS;

        default:
            throw new Error('Unrecognized primitiveType ' + primitiveType);
       }

    };

    /**
     * Returns the attribute index for a certain primitive type.
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {number} numInputs
     * @param {number} outNdx
     * @return {number}
     */
    es3fTransformFeedbackTests.getAttributeIndex = function(primitiveType, numInputs, outNdx) {

    switch (primitiveType) {

        case gluDrawUtil.primitiveType.TRIANGLES: return outNdx;
        case gluDrawUtil.primitiveType.LINES: return outNdx;
        case gluDrawUtil.primitiveType.POINTS: return outNdx;

        case gluDrawUtil.primitiveType.TRIANGLE_STRIP: {
            /** @type {number} */ var triNdx = outNdx / 3;
            /** @type {number} */ var vtxNdx = outNdx % 3;
            return (triNdx % 2 != 0 && vtxNdx < 2) ? (triNdx + 1 - vtxNdx) : (triNdx + vtxNdx);
        }

        case gluDrawUtil.primitiveType.TRIANGLE_FAN:
            return (outNdx % 3 != 0) ? (outNdx / 3 + outNdx % 3) : 0;

        case gluDrawUtil.primitiveType.LINE_STRIP:
            return outNdx / 2 + outNdx % 2;

        case gluDrawUtil.primitiveType.LINE_LOOP: {
            var inNdx = outNdx / 2 + outNdx % 2;
            return inNdx < numInputs ? inNdx : 0;
        }

        default:
            throw new Error('Unrecognized primitiveType ' + primitiveType);
       }

    };

    /**
     * @param {gluDrawUtil.primitiveType} primitiveType type number in gluDrawUtil.primitiveType
     * @param {es3fTransformFeedbackTests.Output} output
     * @param {number} numInputs
     * @param {Object} buffers
     * @return {boolean} isOk
     */
    es3fTransformFeedbackTests.compareTransformFeedbackOutput = function(primitiveType, output, numInputs, buffers) {
        /** @type {boolean} */ var isOk = true;
        /** @type {number} */ var outOffset = output.offset;

        for (var attrNdx = 0; attrNdx < output.inputs.length; attrNdx++) {
        /** @type {es3fTransformFeedbackTests.Attribute} */ var attribute = output.inputs[attrNdx];
        /** @type {gluShaderUtil.DataType} */ var type = attribute.type.getBasicType();
        /** @type {number} */ var numComponents = gluShaderUtil.getDataTypeScalarSize(type);

        /** @type {gluShaderUtil.precision} */ var precision = attribute.type.getPrecision();

        /** @type {string} */ var scalarType = gluShaderUtil.getDataTypeScalarType(type);
        /** @type {number} */ var numOutputs = es3fTransformFeedbackTests.getTransformFeedbackOutputCount(primitiveType, numInputs);

            for (var outNdx = 0; outNdx < numOutputs; outNdx++) {
            /** @type {number} */ var inNdx = es3fTransformFeedbackTests.getAttributeIndex(primitiveType, numInputs, outNdx);

                for (var compNdx = 0; compNdx < numComponents; compNdx++) {
                /** @type {boolean} */ var isEqual = false;

                    if (scalarType === 'float') {
                        var outBuffer = new Float32Array(buffers.output.buffer, buffers.output.offset + buffers.output.stride * outNdx + outOffset + compNdx * 4, 1);
                        var inBuffer = new Float32Array(buffers.input.buffer, buffers.input.offset + buffers.input.stride * inNdx + attribute.offset + compNdx * 4, 1);
                        var difInOut = inBuffer[0] - outBuffer[0];
                        /* TODO: Original code used ULP comparison for highp and mediump precision. This could cause failures. */
                        switch (precision) {
                            case gluShaderUtil.precision.PRECISION_HIGHP: {
                                isEqual = Math.abs(difInOut) < 0.1;
                                break;
                            }

                            case gluShaderUtil.precision.PRECISION_MEDIUMP: {
                                isEqual = Math.abs(difInOut) < 0.1;
                                break;
                            }

                            case gluShaderUtil.precision.PRECISION_LOWP: {
                                isEqual = Math.abs(difInOut) < 0.1;
                                break;
                            }
                            default:
                                throw new Error('Unknown precision: ' + precision);
                        }
                    } else {
                        var outBuffer = new Uint32Array(buffers.output.buffer, buffers.output.offset + buffers.output.stride * outNdx + outOffset + compNdx * 4, 1);
                        var inBuffer = new Uint32Array(buffers.input.buffer, buffers.input.offset + buffers.input.stride * inNdx + attribute.offset + compNdx * 4, 1);
                        isEqual = (inBuffer[0] == outBuffer[0]); // Bit-exact match required for integer types.
                    }

                    if (!isEqual) {
                        bufferedLogToConsole('Mismatch in ' + output.name + ' (' + attribute.name + '), output = ' + outNdx + ', input = ' + inNdx + ', component = ' + compNdx);
                        isOk = false;
                        break;
                    }
                }

                if (!isOk)
                    break;
            }

            if (!isOk)
                break;

            outOffset += numComponents * 4; /*sizeof(deUint32)*/
        }

        return isOk;
    };

    /**
     * Returns (for all the draw calls) the type of Primitive Mode, as it calls "es3fTransformFeedbackTests.getTransformFeedbackPrimitiveCount".
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {Array<es3fTransformFeedbackTests.DrawCall>} array Object.<number, boolean>
     * @return {number} primCount
     */
    es3fTransformFeedbackTests.computeTransformFeedbackPrimitiveCount = function(primitiveType, array) {

    /** @type {number} */ var primCount = 0;

        for (var i = 0; i < array.length; ++ i) {

            if (array[i].transformFeedbackEnabled)
                primCount += es3fTransformFeedbackTests.getTransformFeedbackPrimitiveCount(primitiveType, array[i].numElements);
        }

        return primCount;
    };

    /**
     * @param {number} target
     * @param {number} bufferSize
     * @param {number} guardSize
     */
    es3fTransformFeedbackTests.writeBufferGuard = function(target, bufferSize, guardSize) {
        var buffer = new ArrayBuffer(guardSize);
        var view = new Uint8Array(buffer);
        for (var i = 0; i < guardSize; ++i) view[i] = 0xcd;
        gl.bufferSubData(target, bufferSize, buffer);
    };

    /**
     * Verifies guard
     * @param {ArrayBuffer} buffer
     * @param {number} start
     * @return {boolean}
     */
    es3fTransformFeedbackTests.verifyGuard = function(buffer, start) {
        start = start || 0;
        var view = new Uint8Array(buffer, start);
        for (var i = 0; i < view.length; i++) {
            if (view[i] != 0xcd)
                return false;
        }
        return true;
    };

    /**
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @constructor
     */
    es3fTransformFeedbackTests.TransformFeedbackCase = function(name, desc, bufferMode, primitiveType) {
        tcuTestCase.DeqpTest.call(this, name, desc);
        this.m_bufferMode = bufferMode;
        this.m_primitiveType = primitiveType;
        this.m_progSpec = new es3fTransformFeedbackTests.ProgramSpec();

        // Derived from es3fTransformFeedbackTests.ProgramSpec in es3fTransformFeedbackTests.init()
        this.m_inputStride = 0;
        this.m_attributes = []; // vector<es3fTransformFeedbackTests.Attribute>
        this.m_transformFeedbackOutputs = []; // vector<es3fTransformFeedbackTests.Output>
        this.m_bufferStrides = []; // vector<int>

        // GL state.
        this.m_program = null; // glu::ShaderProgram
        this.m_transformFeedback = null; // glu::TransformFeedback
        this.m_outputBuffers = []; // vector<deUint32>

        this.m_iterNdx = 0; // int
        this.m_testPassed = true;
        // State machine
        this.m_state = es3fTransformFeedbackTests.State.DRAW;
        this.m_verifyStart = null;

        this.m_frameWithTf = null;
        this.m_frameWithoutTf = null;

        this.m_viewportW = 0;
        this.m_viewportH = 0;
        this.m_viewportX = 0;
        this.m_viewportY = 0;

        this.m_primitiveQuery = null;
        this.m_outputsOk = true;

    };

    setParentClass(es3fTransformFeedbackTests.TransformFeedbackCase, tcuTestCase.DeqpTest);

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.createVerificationResult = function(retry, result) {
        return { retry: retry, result: result };
    }

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.dumpShaderText = function() {
        var dbgext = gl.getExtension('WEBGL_debug_shaders');
        for (var ii = 0; ii < this.m_program.shaders.length; ++ii) {
            debug('Shader source ' + ii + ' before translation:')
            debug(this.m_program.shaders[ii].info.source);
            debug('');
            debug('Shader source ' + ii + ' after translation:');
            debug(dbgext.getTranslatedShaderSource(this.m_program.shaders[ii].shader));
        }
    };

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.init = function() {
        this.m_program = es3fTransformFeedbackTests.createVertexCaptureProgram(
            this.m_progSpec,
            this.m_bufferMode,
            this.m_primitiveType
        );

        if (!this.m_program.isOk()) {
            // this.dumpShaderText();

            var linkFail = this.m_program.shadersOK &&
                           !this.m_program.getProgramInfo().linkOk;

            if (linkFail) {
                if (!es3fTransformFeedbackTests.isProgramSupported(this.m_progSpec, this.m_bufferMode)) {
                    var msg = 'Not Supported. Implementation limits exceeded.';
                    checkMessage(false, msg);
                    throw new TestFailedException(msg);
                } else if (es3fTransformFeedbackTests.hasArraysInTFVaryings(this.m_progSpec)) {
                    msg = 'Capturing arrays is not supported (undefined in specification)';
                    checkMessage(false, msg);
                    throw new TestFailedException(msg);
                } else {
                    throw new Error('Link failed: ' + this.m_program.getProgramInfo().infoLog);
                }
            } else {
                throw new Error('Compile failed');
            }
        } else {
            // debug('Program is ' +
            //       (gl.getProgramParameter(this.m_program.getProgram(), gl.LINK_STATUS) ? 'linked' : 'not linked'));
            // this.dumpShaderText();
        }

//          bufferedLogToConsole('Transform feedback varyings: ' + tcu.formatArray(this.m_progSpec.getTransformFeedbackVaryings()));
        bufferedLogToConsole('Transform feedback varyings: ' + this.m_progSpec.getTransformFeedbackVaryings());

        // Print out transform feedback points reported by GL.
        // bufferedLogToConsole('Transform feedback varyings reported by compiler:');
        //logTransformFeedbackVaryings(log, gl, this.m_program.getProgram());

        // Compute input specification.
        this.m_inputStride = es3fTransformFeedbackTests.computeInputLayout(this.m_attributes, this.m_progSpec.getVaryings(), this.m_progSpec.isPointSizeUsed());

        // Build list of varyings used in transform feedback.
        es3fTransformFeedbackTests.computeTransformFeedbackOutputs(
            this.m_transformFeedbackOutputs,
            this.m_attributes,
            this.m_progSpec.getVaryings(),
            this.m_progSpec.getTransformFeedbackVaryings(),
            this.m_bufferMode
        );
        if (!this.m_transformFeedbackOutputs.length) {
            throw new Error('transformFeedbackOutputs cannot be empty.');
        }

        if (this.m_bufferMode == gl.SEPARATE_ATTRIBS) {
            for (var i = 0; i < this.m_transformFeedbackOutputs.length; ++i) {
                this.m_bufferStrides.push(this.m_transformFeedbackOutputs[i].type.getScalarSize() * 4 /*sizeof(deUint32)*/);
            }
        } else {
            var totalSize = 0;
            for (var i = 0; i < this.m_transformFeedbackOutputs.length; ++i) {
                totalSize += this.m_transformFeedbackOutputs[i].type.getScalarSize() * 4 /*sizeof(deUint32)*/;
            }
            this.m_bufferStrides.push(totalSize);
        }

        this.m_outputBuffers.length = this.m_bufferStrides.length;
        for (var i = 0; i < this.m_outputBuffers.length; i++)
            this.m_outputBuffers[i] = gl.createBuffer();

        this.m_transformFeedback = gl.createTransformFeedback();

        this.m_iterNdx = 0;
//          this.m_testCtx.setTestResult(QP_TEST_RESULT_PASS, 'Pass');

    };

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.deinit = function() {
        for (var i = 0; i < this.m_outputBuffers.length; i++)
            gl.deleteBuffer(this.m_outputBuffers[i]);

    //    delete this.m_transformFeedback;
        this.m_transformFeedback = null;

    //    delete this.m_program;
        this.m_program = null;

        // Clean up state.
        this.m_attributes = [];
        this.m_transformFeedbackOutputs = [];
        this.m_bufferStrides = [];
        this.m_inputStride = 0;

    };

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.iterate = function() {
        var s = es3fTransformFeedbackTests.TransformFeedbackCase.s_iterate;
        var numIterations = s.iterations.length;
        var seed = deMath.deMathHash(this.m_iterNdx);
        switch(this.m_state) {
            case es3fTransformFeedbackTests.State.DRAW:
                bufferedLogToConsole('Testing ' +
                    s.testCases[s.iterations[this.m_iterNdx]].length +
                    ' draw calls, (element es3fTransformFeedbackTests.count, TF state): ' +
                    s.testCases[s.iterations[this.m_iterNdx]]
                );
                this.draw(s.testCases[s.iterations[this.m_iterNdx]], seed);
                this.m_state = es3fTransformFeedbackTests.State.VERIFY;
                break;
            case es3fTransformFeedbackTests.State.VERIFY:
                var verifyResult = this.verify(s.testCases[s.iterations[this.m_iterNdx]]);
                if (verifyResult.retry) {
                    break;
                }
                this.m_testPassed = verifyResult.result;
                this.m_iterNdx += 1;
                if (this.m_testPassed && this.m_iterNdx < numIterations) {
                    this.m_state = es3fTransformFeedbackTests.State.DRAW;
                    break;
                }
                // Fall through
            case es3fTransformFeedbackTests.State.FINISH:
                if (!this.m_testPassed) testFailedOptions('Result comparison failed for iteration ' + s.iterations[this.m_iterNdx - 1], false);
                else testPassedOptions('Result comparison succeeded', true);
                return tcuTestCase.IterateResult.STOP;
        }

        return tcuTestCase.IterateResult.CONTINUE;

    };

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.draw = function(calls, seed) {
        var _min = function(x, y) { return x < y ? x : y; };

        var rnd = new deRandom.Random(seed);
        var numInputs = 0;
        var numOutputs = 0;
        var width = gl.drawingBufferWidth;
        var height = gl.drawingBufferHeight;
        this.m_viewportW = _min(es3fTransformFeedbackTests.VIEWPORT_WIDTH, width);
        this.m_viewportH = _min(es3fTransformFeedbackTests.VIEWPORT_HEIGHT, height);
        this.m_viewportX = rnd.getInt(0, width - this.m_viewportW);
        this.m_viewportY = rnd.getInt(0, height - this.m_viewportH);
        this.m_frameWithTf = new tcuSurface.Surface(this.m_viewportW, this.m_viewportH); // tcu::Surface
        this.m_frameWithoutTf = new tcuSurface.Surface(this.m_viewportW, this.m_viewportH); // tcu::Surface
        this.m_primitiveQuery = gl.createQuery();
        this.m_outputsOk = true;

        // Compute totals.
        for (var i = 0; i < calls.length; ++i) {
            var call = calls[i];
            numInputs += call.numElements;
            numOutputs += call.transformFeedbackEnabled ? es3fTransformFeedbackTests.getTransformFeedbackOutputCount(this.m_primitiveType, call.numElements) : 0;
        }

        // Input data.
        var inputData = es3fTransformFeedbackTests.genInputData(this.m_attributes, numInputs, this.m_inputStride, rnd);

        gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, this.m_transformFeedback);

        // Allocate storage for transform feedback output buffers and bind to targets.
        for (var bufNdx = 0; bufNdx < this.m_outputBuffers.length; ++bufNdx) {
            var buffer = this.m_outputBuffers[bufNdx]; // deUint32
            var stride = this.m_bufferStrides[bufNdx]; // int
            var target = bufNdx; // int
            var size = stride * numOutputs; // int
            var guardSize = stride * es3fTransformFeedbackTests.BUFFER_GUARD_MULTIPLIER; // int
            var usage = gl.DYNAMIC_READ; // const deUint32

            gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, buffer);
            gl.bufferData(gl.TRANSFORM_FEEDBACK_BUFFER, size + guardSize, usage);
            es3fTransformFeedbackTests.writeBufferGuard(gl.TRANSFORM_FEEDBACK_BUFFER, size, guardSize);

            // \todo [2012-07-30 pyry] glBindBufferRange()?
            gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, target, buffer);
        }

        var attribBuffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, attribBuffer);
        gl.bufferData(gl.ARRAY_BUFFER, inputData, gl.STATIC_DRAW);

        // Setup attributes.
        for (var i = 0; i < this.m_attributes.length; ++i) {
            var attrib = this.m_attributes[i];
            var loc = gl.getAttribLocation(this.m_program.getProgram(), attrib.name);
            /** @type {string} */
            var scalarType = gluShaderUtil.getDataTypeScalarType(attrib.type.getBasicType());
            /** @type {number} */
            var numComponents = gluShaderUtil.getDataTypeScalarSize(attrib.type.getBasicType());

            if (loc >= 0) {
                gl.enableVertexAttribArray(loc);
                switch (scalarType) {
                    case 'float':
                        gl.vertexAttribPointer(loc, numComponents, gl.FLOAT, false, this.m_inputStride, attrib.offset); break;
                    case 'int':
                        gl.vertexAttribIPointer(loc, numComponents, gl.INT, this.m_inputStride, attrib.offset); break;
                    case 'uint':
                        gl.vertexAttribIPointer(loc, numComponents, gl.UNSIGNED_INT, this.m_inputStride, attrib.offset); break;
                }
            }
        }

        // Setup viewport.
        gl.viewport(this.m_viewportX, this.m_viewportY, this.m_viewportW, this.m_viewportH);

        // Setup program.
        gl.useProgram(this.m_program.getProgram());

        gl.uniform4fv(
            gl.getUniformLocation(this.m_program.getProgram(), 'u_scale'),
            [0.01, 0.01, 0.01, 0.01]
        );
        gl.uniform4fv(
            gl.getUniformLocation(this.m_program.getProgram(), 'u_bias'),
            [0.5, 0.5, 0.5, 0.5]
        );

        // Enable query.
        gl.beginQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, this.m_primitiveQuery);

        // Draw
        var offset = 0;
        var tfEnabled = true;

        gl.clear(gl.COLOR_BUFFER_BIT);

        var tfPrimitiveMode = es3fTransformFeedbackTests.getTransformFeedbackPrimitiveMode(this.m_primitiveType);
        gl.beginTransformFeedback(tfPrimitiveMode);

        for (var i = 0; i < calls.length; ++i) {
            var call = calls[i];

            // Pause or resume transform feedback if necessary.
            if (call.transformFeedbackEnabled != tfEnabled) {
                if (call.transformFeedbackEnabled)
                    gl.resumeTransformFeedback();
                else
                    gl.pauseTransformFeedback();
                tfEnabled = call.transformFeedbackEnabled;
            }

            gl.drawArrays(gluDrawUtil.getPrimitiveGLType(gl, this.m_primitiveType), offset, call.numElements);
            offset += call.numElements;
        }

        // Resume feedback before finishing it.
        if (!tfEnabled)
            gl.resumeTransformFeedback();

        gl.endTransformFeedback();

        gl.endQuery(gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN);

        // Check and log query status right after submit
        var query = this.m_primitiveQuery;

        var available = gl.getQueryParameter(query, gl.QUERY_RESULT_AVAILABLE);

        if (available) {
            this.m_testPassed = false;
            this.m_state = es3fTransformFeedbackTests.State.FINISH;
            testFailedOptions('Transform feedback query result must not be available the same frame as they are issued.', true);
        }

        // Compare result buffers.
        for (var bufferNdx = 0; bufferNdx < this.m_outputBuffers.length; ++bufferNdx) {
            var stride = this.m_bufferStrides[bufferNdx]; // int
            var size = stride * numOutputs; // int
            var guardSize = stride * es3fTransformFeedbackTests.BUFFER_GUARD_MULTIPLIER; // int
            var buffer = new ArrayBuffer(size + guardSize); // const void*

            // Bind buffer for reading.
            gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, this.m_outputBuffers[bufferNdx]);

            gl.getBufferSubData(gl.TRANSFORM_FEEDBACK_BUFFER, 0, new Uint8Array(buffer));

            // Verify all output variables that are written to this buffer.
            for (var i = 0; i < this.m_transformFeedbackOutputs.length; ++i) {
                var out = this.m_transformFeedbackOutputs[i];

                if (out.bufferNdx != bufferNdx)
                    continue;

                var inputOffset = 0;
                var outputOffset = 0;

                // Process all draw calls and check ones with transform feedback enabled
                for (var callNdx = 0; callNdx < calls.length; ++callNdx) {
                    var call = calls[callNdx];

                    if (call.transformFeedbackEnabled) {
                        var inputPtr = inputData[0] + inputOffset * this.m_inputStride; // const deUint8*
                        var outputPtr = outputOffset * stride; // const deUint8*

                        if (!es3fTransformFeedbackTests.compareTransformFeedbackOutput(this.m_primitiveType, out, call.numElements, {
                                 input: {
                                    buffer: inputData,
                                    offset: inputOffset * this.m_inputStride,
                                    stride: this.m_inputStride
                                },
                                output: {
                                    buffer: buffer,
                                    offset: outputOffset * stride,
                                    stride: stride
                                }
                            })) {
                            this.m_outputsOk = false;
                            break;
                        }
                    }

                    inputOffset += call.numElements;
                    outputOffset += call.transformFeedbackEnabled ? es3fTransformFeedbackTests.getTransformFeedbackOutputCount(this.m_primitiveType, call.numElements) : 0;
                }
            }

            // Verify guardband.
            if (!es3fTransformFeedbackTests.verifyGuard(buffer, size)) {
                bufferedLogToConsole('Error: Transform feedback buffer overrun detected');
                this.m_outputsOk = false;
            }
        }
    };

    es3fTransformFeedbackTests.TransformFeedbackCase.prototype.verify = function(calls) {
        // Check status after mapping buffers.
        var mustBeReady = this.m_outputBuffers.length > 0; // Mapping buffer forces synchronization. // const bool
        var expectedCount = es3fTransformFeedbackTests.computeTransformFeedbackPrimitiveCount(this.m_primitiveType, calls); // const int
        var available = /** @type {boolean} */ (gl.getQueryParameter(this.m_primitiveQuery, gl.QUERY_RESULT_AVAILABLE));
        var verify_offset = 0;
        var queryOk = true;
        if (!available) {
            if (!this.m_verifyStart)
                this.m_verifyStart = new Date();
            else {
                var current = new Date();
                var elapsedTime = 0.001 * (current.getTime() - this.m_verifyStart.getTime());
                if (elapsedTime > es3fTransformFeedbackTests.MAX_VERIFY_WAIT) {
                    testFailed('Query result not available after ' + elapsedTime + ' seconds.');
                    this.m_state = es3fTransformFeedbackTests.State.FINISH;
                    return this.createVerificationResult(false, false);
                }
            }
            return this.createVerificationResult(true, false);
        }

        var numPrimitives = /** @type {number} */ (gl.getQueryParameter(this.m_primitiveQuery, gl.QUERY_RESULT));

        if (!mustBeReady && available == false)
            bufferedLogToConsole('ERROR: gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN result not available after mapping buffers!');

        bufferedLogToConsole('gl.TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN = ' + numPrimitives);

        if (numPrimitives != expectedCount) {
            queryOk = false;
            bufferedLogToConsole('ERROR: Expected ' + expectedCount + ' primitives!');
        }

        // Clear transform feedback state.
        gl.bindTransformFeedback(gl.TRANSFORM_FEEDBACK, null);
        for (var bufNdx = 0; bufNdx < this.m_outputBuffers.length; ++bufNdx) {
            gl.bindBuffer(gl.TRANSFORM_FEEDBACK_BUFFER, null);
            gl.bindBufferBase(gl.TRANSFORM_FEEDBACK_BUFFER, bufNdx, null);
        }

        gl.bindBuffer(gl.ARRAY_BUFFER, null);

        // Read back rendered image.
        this.m_frameWithTf.readViewport(gl, [this.m_viewportX, this.m_viewportY, this.m_viewportW, this.m_viewportH]);

        // Render without transform feedback.

        gl.clear(gl.COLOR_BUFFER_BIT);

        for (var i = 0; i < calls.length; ++i) {
            var call = calls[i];
            gl.drawArrays(gluDrawUtil.getPrimitiveGLType(gl, this.m_primitiveType), verify_offset, call.numElements);
            verify_offset += call.numElements;
        }
        this.m_frameWithoutTf.readViewport(gl, [this.m_viewportX, this.m_viewportY, this.m_viewportW, this.m_viewportH]);

        // Compare images with and without transform feedback.
        var imagesOk = tcuImageCompare.pixelThresholdCompare('Result', 'Image comparison result', this.m_frameWithoutTf, this.m_frameWithTf, [1, 1, 1, 1], tcuImageCompare.CompareLogMode.ON_ERROR);

        if (imagesOk)
            bufferedLogToConsole('Rendering result comparison between TF enabled and TF disabled passed.');
        else
            bufferedLogToConsole('ERROR: Rendering result comparison between TF enabled and TF disabled failed!');

        return this.createVerificationResult(false, this.m_outputsOk && imagesOk && queryOk);

    };

    es3fTransformFeedbackTests.dc = function(numElements, tfEnabled) {
        return new es3fTransformFeedbackTests.DrawCall(numElements, tfEnabled);
    };

    // static data
    es3fTransformFeedbackTests.TransformFeedbackCase.s_iterate = {

        testCases: {
            elemCount1: [es3fTransformFeedbackTests.dc(1, true)],
            elemCount2: [es3fTransformFeedbackTests.dc(2, true)],
            elemCount3: [es3fTransformFeedbackTests.dc(3, true)],
            elemCount4: [es3fTransformFeedbackTests.dc(4, true)],
            elemCount123: [es3fTransformFeedbackTests.dc(123, true)],
            basicPause1: [es3fTransformFeedbackTests.dc(64, true), es3fTransformFeedbackTests.dc(64, false), es3fTransformFeedbackTests.dc(64, true)],
            basicPause2: [es3fTransformFeedbackTests.dc(13, true), es3fTransformFeedbackTests.dc(5, true), es3fTransformFeedbackTests.dc(17, false),
                           es3fTransformFeedbackTests.dc(3, true), es3fTransformFeedbackTests.dc(7, false)],
            startPaused: [es3fTransformFeedbackTests.dc(123, false), es3fTransformFeedbackTests.dc(123, true)],
            random1: [es3fTransformFeedbackTests.dc(65, true), es3fTransformFeedbackTests.dc(135, false), es3fTransformFeedbackTests.dc(74, true),
                           es3fTransformFeedbackTests.dc(16, false), es3fTransformFeedbackTests.dc(226, false), es3fTransformFeedbackTests.dc(9, true),
                           es3fTransformFeedbackTests.dc(174, false)],
            random2: [es3fTransformFeedbackTests.dc(217, true), es3fTransformFeedbackTests.dc(171, true), es3fTransformFeedbackTests.dc(147, true),
                           es3fTransformFeedbackTests.dc(152, false), es3fTransformFeedbackTests.dc(55, true)]
        },
        iterations: [
            'elemCount1', 'elemCount2', 'elemCount3', 'elemCount4', 'elemCount123',
            'basicPause1', 'basicPause2', 'startPaused',
            'random1', 'random2'
        ]
    };

    es3fTransformFeedbackTests.hasArraysInTFVaryings = function(spec) {

        for (var i = 0; i < spec.getTransformFeedbackVaryings().length; ++i) {
            var tfVar = spec.getTransformFeedbackVaryings()[i];
            var varName = gluVarTypeUtil.parseVariableName(tfVar);

        var attr = es3fTransformFeedbackTests.findAttributeNameEquals(spec.getVaryings(), varName);
        if (attr && attr.type.isArrayType())
                return true;
        }
        return false;

    };

    /** es3fTransformFeedbackTests.PositionCase
     * @extends {es3fTransformFeedbackTests.TransformFeedbackCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @constructor
     */
    es3fTransformFeedbackTests.PositionCase = function(name, desc, bufferMode, primitiveType) {
        es3fTransformFeedbackTests.TransformFeedbackCase.call(this, name, desc, bufferMode, primitiveType);
        this.m_progSpec.addTransformFeedbackVarying('gl_Position');
    };

    setParentClass(es3fTransformFeedbackTests.PositionCase, es3fTransformFeedbackTests.TransformFeedbackCase);

    /** es3fTransformFeedbackTests.PointSizeCase
     * @extends {es3fTransformFeedbackTests.TransformFeedbackCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @constructor
     */
    es3fTransformFeedbackTests.PointSizeCase = function(name, desc, bufferMode, primitiveType) {
        es3fTransformFeedbackTests.TransformFeedbackCase.call(this, name, desc, bufferMode, primitiveType);
        this.m_progSpec.addTransformFeedbackVarying('gl_PointSize');

    };

    setParentClass(es3fTransformFeedbackTests.PointSizeCase, es3fTransformFeedbackTests.TransformFeedbackCase);

    /** es3fTransformFeedbackTests.BasicTypeCase
     * @extends {es3fTransformFeedbackTests.TransformFeedbackCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {gluShaderUtil.DataType} type
     * @param {gluShaderUtil.precision} precision
     * @param {es3fTransformFeedbackTests.interpolation} interpolation enum number in this javascript
     * @constructor
     */
    es3fTransformFeedbackTests.BasicTypeCase = function(name, desc, bufferMode, primitiveType, type, precision, interpolation) {
        es3fTransformFeedbackTests.TransformFeedbackCase.call(this, name, desc, bufferMode, primitiveType);

        this.m_progSpec.addVarying('v_varA', gluVarType.newTypeBasic(type, precision), interpolation);
        this.m_progSpec.addVarying('v_varB', gluVarType.newTypeBasic(type, precision), interpolation);

        this.m_progSpec.addTransformFeedbackVarying('v_varA');
        this.m_progSpec.addTransformFeedbackVarying('v_varB');

    };

    setParentClass(es3fTransformFeedbackTests.BasicTypeCase, es3fTransformFeedbackTests.TransformFeedbackCase);

    /** es3fTransformFeedbackTests.BasicArrayCase
     * @extends {es3fTransformFeedbackTests.TransformFeedbackCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {gluShaderUtil.DataType} type
     * @param {gluShaderUtil.precision} precision
     * @param {es3fTransformFeedbackTests.interpolation} interpolation enum number in this javascript
     * @constructor
     */
    es3fTransformFeedbackTests.BasicArrayCase = function(name, desc, bufferMode, primitiveType, type, precision, interpolation) {
        es3fTransformFeedbackTests.TransformFeedbackCase.call(this, name, desc, bufferMode, primitiveType);

        if (gluShaderUtil.isDataTypeMatrix(type) || this.m_bufferMode === gl.SEPARATE_ATTRIBS) {
            // note For matrix types we need to use reduced array sizes or otherwise we will exceed maximum attribute (16)
            // or transform feedback component es3fTransformFeedbackTests.count (64).
            // On separate attribs mode maximum component es3fTransformFeedbackTests.count per varying is 4.
            this.m_progSpec.addVarying('v_varA', gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), 1), interpolation);
            this.m_progSpec.addVarying('v_varB', gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), 2), interpolation);
        } else {
            this.m_progSpec.addVarying('v_varA', gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), 3), interpolation);
            this.m_progSpec.addVarying('v_varB', gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), 4), interpolation);
        }

        this.m_progSpec.addTransformFeedbackVarying('v_varA');
        this.m_progSpec.addTransformFeedbackVarying('v_varB');

    };

    setParentClass(es3fTransformFeedbackTests.BasicArrayCase, es3fTransformFeedbackTests.TransformFeedbackCase);

    /** es3fTransformFeedbackTests.ArrayElementCase
     * @extends {es3fTransformFeedbackTests.TransformFeedbackCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {gluShaderUtil.DataType} type
     * @param {gluShaderUtil.precision} precision
     * @param {es3fTransformFeedbackTests.interpolation} interpolation enum number in this javascript
     * @constructor
     */
    es3fTransformFeedbackTests.ArrayElementCase = function(name, desc, bufferMode, primitiveType, type, precision, interpolation) {

        es3fTransformFeedbackTests.TransformFeedbackCase.call(this, name, desc, bufferMode, primitiveType);

        this.m_progSpec.addVarying('v_varA', gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), 3), interpolation);
        this.m_progSpec.addVarying('v_varB', gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), 4), interpolation);

        this.m_progSpec.addTransformFeedbackVarying('v_varA[1]');
        this.m_progSpec.addTransformFeedbackVarying('v_varB[0]');
        this.m_progSpec.addTransformFeedbackVarying('v_varB[3]');

    };

    setParentClass(es3fTransformFeedbackTests.ArrayElementCase, es3fTransformFeedbackTests.TransformFeedbackCase);

    /** es3fTransformFeedbackTests.RandomCase
     * @extends {es3fTransformFeedbackTests.TransformFeedbackCase}
     * @param {string} name
     * @param {string} desc
     * @param {number} bufferMode
     * @param {gluDrawUtil.primitiveType} primitiveType GLenum that specifies what kind of primitive is
     * @param {number} seed
     * @constructor
     */
    es3fTransformFeedbackTests.RandomCase = function(name, desc, bufferMode, primitiveType, seed) {
        es3fTransformFeedbackTests.TransformFeedbackCase.call(this, name, desc, bufferMode, primitiveType);

    };

    setParentClass(es3fTransformFeedbackTests.RandomCase, es3fTransformFeedbackTests.TransformFeedbackCase);

    es3fTransformFeedbackTests.RandomCase.prototype.init = function() {

        /** @type {number} */
        var seed = /*deString.deStringHash(getName()) ^ */ deMath.deMathHash(this.m_iterNdx);

        /** @type {Array<gluShaderUtil.DataType>} */
        var typeCandidates = [
            gluShaderUtil.DataType.FLOAT,
            gluShaderUtil.DataType.FLOAT_VEC2,
            gluShaderUtil.DataType.FLOAT_VEC3,
            gluShaderUtil.DataType.FLOAT_VEC4,
            gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT_VEC2,
            gluShaderUtil.DataType.INT_VEC3,
            gluShaderUtil.DataType.INT_VEC4,
            gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT_VEC2,
            gluShaderUtil.DataType.UINT_VEC3,
            gluShaderUtil.DataType.UINT_VEC4,

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

        /** @type {Array<gluShaderUtil.precision>} */
        var precisions = [
            gluShaderUtil.precision.PRECISION_LOWP,
            gluShaderUtil.precision.PRECISION_MEDIUMP,
            gluShaderUtil.precision.PRECISION_HIGHP
        ];

        var interpModes = [{name: 'smooth', interp: es3fTransformFeedbackTests.interpolation.SMOOTH}, {name: 'flat', interp: es3fTransformFeedbackTests.interpolation.FLAT}, {name: 'centroid', interp: es3fTransformFeedbackTests.interpolation.CENTROID}
        ];

        /** @type {number} */ var maxAttributeVectors = 16;
       //** @type {number} */  var maxTransformFeedbackComponents = 64; // note It is enough to limit attribute set size.
        /** @type {boolean} */ var isSeparateMode = (this.m_bufferMode === gl.SEPARATE_ATTRIBS);
        /** @type {number} */ var maxTransformFeedbackVars = isSeparateMode ? 4 : maxAttributeVectors;
        /** @type {number} */ var arrayWeight = 0.3;
        /** @type {number} */ var positionWeight = 0.7;
        /** @type {number} */ var pointSizeWeight = 0.1;
        /** @type {number} */ var captureFullArrayWeight = 0.5;

        /** @type {deRandom.Random} */
                               var rnd = new deRandom.Random(seed);
        /** @type {boolean} */ var usePosition = rnd.getFloat() < positionWeight;
        /** @type {boolean} */ var usePointSize = rnd.getFloat() < pointSizeWeight;
        /** @type {number} */ var numAttribVectorsToUse = rnd.getInt(
            1, maxAttributeVectors - 1/*position*/ - (usePointSize ? 1 : 0)
        );

        /** @type {number} */ var numAttributeVectors = 0;
        /** @type {number} */ var varNdx = 0;

        // Generate varyings.
        while (numAttributeVectors < numAttribVectorsToUse) {
            /** @type {number} */
            var maxVecs = isSeparateMode ? Math.min(2 /*at most 2*mat2*/, numAttribVectorsToUse - numAttributeVectors) : numAttribVectorsToUse - numAttributeVectors;
            /** @type {gluShaderUtil.DataType} */
            var begin = typeCandidates[0];
            /** @type {number} */
            var endCandidates = begin + (
                maxVecs >= 4 ? 21 : (
                    maxVecs >= 3 ? 18 : (
                        maxVecs >= 2 ? (isSeparateMode ? 13 : 15) : 12
                    )
                )
            );
            /** @type {gluShaderUtil.DataType} */
            var end = typeCandidates[endCandidates];

            /** @type {gluShaderUtil.DataType} */
            var type = rnd.choose(typeCandidates)[0];

            /** @type {gluShaderUtil.precision} */
            var precision = rnd.choose(precisions)[0];

            /** @type {es3fTransformFeedbackTests.interpolation} */
            var interp = (type === gluShaderUtil.DataType.FLOAT) ?
                       rnd.choose(interpModes)[0].interp :
                       es3fTransformFeedbackTests.interpolation.FLAT;

            /** @type {number} */
            var numVecs = gluShaderUtil.isDataTypeMatrix(type) ? gluShaderUtil.getDataTypeMatrixNumColumns(type) : 1;
            /** @type {number} */
            var numComps = gluShaderUtil.getDataTypeScalarSize(type);
            /** @type {number} */
            var maxArrayLen = Math.max(1, isSeparateMode ? (4 / numComps) : (maxVecs / numVecs));
            /** @type {boolean} */
            var useArray = rnd.getFloat() < arrayWeight;
            /** @type {number} */
            var arrayLen = useArray ? rnd.getInt(1, maxArrayLen) : 1;
            /** @type {string} */
            var name = 'v_var' + varNdx;

            if (useArray)
                this.m_progSpec.addVarying(name, gluVarType.newTypeArray(gluVarType.newTypeBasic(type, precision), arrayLen), interp);
            else
                this.m_progSpec.addVarying(name, gluVarType.newTypeBasic(type, precision), interp);

            numAttributeVectors += arrayLen * numVecs;
            varNdx += 1;
        }

        // Generate transform feedback candidate set.
        /** @type {Array<string>} */ var tfCandidates = [];

        if (usePosition) tfCandidates.push('gl_Position');
        if (usePointSize) tfCandidates.push('gl_PointSize');

        for (var ndx = 0; ndx < varNdx; ndx++) {
            /** @type {es3fTransformFeedbackTests.Varying} */
            var varying = this.m_progSpec.getVaryings()[ndx];

            if (varying.type.isArrayType()) {
                /** @type {boolean} */
                var captureFull = rnd.getFloat() < captureFullArrayWeight;

                if (captureFull) {
                    tfCandidates.push(varying.name);
                } else {
                    /** @type {number} */
                    var numElem = varying.type.getArraySize();
                    for (var elemNdx = 0; elemNdx < numElem; elemNdx++)
                        tfCandidates.push(varying.name + '[' + elemNdx + ']');
                }
            } else
                tfCandidates.push(varying.name);
        }

        // Pick random selection.
        var tfVaryings = [];
        rnd.choose(tfCandidates, tfVaryings, Math.min(tfCandidates.length, maxTransformFeedbackVars));
        rnd.shuffle(tfVaryings);
        for (var i = 0; i < tfVaryings.length; i++)
            this.m_progSpec.addTransformFeedbackVarying(tfVaryings[i]);

        es3fTransformFeedbackTests.TransformFeedbackCase.prototype.init.call(this);

    };

    /**
     * Creates the test in order to be executed
    **/
    es3fTransformFeedbackTests.init = function() {

        /** @const @type {tcuTestCase.DeqpTest} */
        var testGroup = tcuTestCase.runner.testCases;

        var bufferModes = [{name: 'separate', mode: gl.SEPARATE_ATTRIBS}, {name: 'interleaved', mode: gl.INTERLEAVED_ATTRIBS}
        ];

        var primitiveTypes = [{name: 'points', type: gluDrawUtil.primitiveType.POINTS}, {name: 'lines', type: gluDrawUtil.primitiveType.LINES}, {name: 'triangles', type: gluDrawUtil.primitiveType.TRIANGLES}
        ];

        /** @type {Array<gluShaderUtil.DataType>} */
        var basicTypes = [
            gluShaderUtil.DataType.FLOAT,
            gluShaderUtil.DataType.FLOAT_VEC2,
            gluShaderUtil.DataType.FLOAT_VEC3,
            gluShaderUtil.DataType.FLOAT_VEC4,
            gluShaderUtil.DataType.FLOAT_MAT2,
            gluShaderUtil.DataType.FLOAT_MAT2X3,
            gluShaderUtil.DataType.FLOAT_MAT2X4,
            gluShaderUtil.DataType.FLOAT_MAT3X2,
            gluShaderUtil.DataType.FLOAT_MAT3,
            gluShaderUtil.DataType.FLOAT_MAT3X4,
            gluShaderUtil.DataType.FLOAT_MAT4X2,
            gluShaderUtil.DataType.FLOAT_MAT4X3,
            gluShaderUtil.DataType.FLOAT_MAT4,
            gluShaderUtil.DataType.INT,
            gluShaderUtil.DataType.INT_VEC2,
            gluShaderUtil.DataType.INT_VEC3,
            gluShaderUtil.DataType.INT_VEC4,
            gluShaderUtil.DataType.UINT,
            gluShaderUtil.DataType.UINT_VEC2,
            gluShaderUtil.DataType.UINT_VEC3,
            gluShaderUtil.DataType.UINT_VEC4
        ];

        /** @type {Array<gluShaderUtil.precision>} */
        var precisions = [

            gluShaderUtil.precision.PRECISION_LOWP,
            gluShaderUtil.precision.PRECISION_MEDIUMP,
            gluShaderUtil.precision.PRECISION_HIGHP

            // glsUBC.UniformFlags.PRECISION_LOW,
            // glsUBC.UniformFlags.PRECISION_MEDIUM,
            // glsUBC.UniformFlags.PRECISION_HIGH
        ];

        var interpModes = [{name: 'smooth', interp: es3fTransformFeedbackTests.interpolation.SMOOTH}, {name: 'flat', interp: es3fTransformFeedbackTests.interpolation.FLAT}, {name: 'centroid', interp: es3fTransformFeedbackTests.interpolation.CENTROID}
        ];

        // .position
        /** @type {tcuTestCase.DeqpTest} */
        var positionGroup = tcuTestCase.newTest('position', 'gl_Position capture using transform feedback');
        testGroup.addChild(positionGroup);

        for (var primitiveType = 0; primitiveType < primitiveTypes.length; primitiveType++) {
            for (var bufferMode = 0; bufferMode < bufferModes.length; bufferMode++) {
                /** @type {string} */
                var name = primitiveTypes[primitiveType].name + '_' + bufferModes[bufferMode].name;

                positionGroup.addChild(new es3fTransformFeedbackTests.PositionCase(
                    name,
                    '',
                    bufferModes[bufferMode].mode,
                    primitiveTypes[primitiveType].type
                ));
            }
        }

        // .point_size
        /** @type {tcuTestCase.DeqpTest} */ var pointSizeGroup = tcuTestCase.newTest('point_size', 'gl_PointSize capture using transform feedback');
        testGroup.addChild(pointSizeGroup);

        for (var primitiveType = 0; primitiveType < primitiveTypes.length; primitiveType++) {
            for (var bufferMode = 0; bufferMode < bufferModes.length; bufferMode++) {
                var name = primitiveTypes[primitiveType].name + '_' + bufferModes[bufferMode].name;

                pointSizeGroup.addChild(new es3fTransformFeedbackTests.PointSizeCase(
                    name,
                    '',
                    bufferModes[bufferMode].mode,
                    primitiveTypes[primitiveType].type
                ));
            }
        }

        // .basic_type
        for (var bufferModeNdx = 0; bufferModeNdx < bufferModes.length; bufferModeNdx++) {
            /** @type {number} */
            var bufferMode = bufferModes[bufferModeNdx].mode;
            for (var primitiveTypeNdx = 0; primitiveTypeNdx < primitiveTypes.length; primitiveTypeNdx++) {
                /** @type {tcuTestCase.DeqpTest} */
                var primitiveGroup = tcuTestCase.newTest(
                    'basic_types.' + bufferModes[bufferModeNdx].name + '.' + primitiveTypes[primitiveTypeNdx].name,
                    'Basic types in transform feedback');
                /** @type {number} */
                var primitiveType = primitiveTypes[primitiveTypeNdx].type;
                testGroup.addChild(primitiveGroup);

                for (var typeNdx = 0; typeNdx < basicTypes.length; typeNdx++) {
                    /** @type {gluShaderUtil.DataType} */
                    var type = basicTypes[typeNdx];
                    /** @type {boolean} */
                    var isFloat = gluShaderUtil.getDataTypeScalarType(type) == gluShaderUtil.DataType.FLOAT;

                    for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                        /** @type {gluShaderUtil.precision} */
                        var precision = precisions[precNdx];
                        var name = gluShaderUtil.getPrecisionName(precision) + '_' + gluShaderUtil.getDataTypeName(type);

                        primitiveGroup.addChild(new es3fTransformFeedbackTests.BasicTypeCase(
                            name,
                            '',
                            bufferMode,
                            primitiveType,
                            type,
                            precision,
                            isFloat ? es3fTransformFeedbackTests.interpolation.SMOOTH : es3fTransformFeedbackTests.interpolation.FLAT
                        ));
                    }
                }
            }
        }

        // .array
        for (var bufferModeNdx = 0; bufferModeNdx < bufferModes.length; bufferModeNdx++) {
            var bufferMode = bufferModes[bufferModeNdx].mode;
            for (var primitiveTypeNdx = 0; primitiveTypeNdx < primitiveTypes.length; primitiveTypeNdx++) {
                var primitiveGroup = tcuTestCase.newTest(
                    'array.' + bufferModes[bufferModeNdx].name + '.' + primitiveTypes[primitiveTypeNdx].name,
                    'Capturing whole array in TF');
                /** @type {number} */
                var primitiveType = primitiveTypes[primitiveTypeNdx].type;
                testGroup.addChild(primitiveGroup);

                for (var typeNdx = 0; typeNdx < basicTypes.length; typeNdx++) {
                    var type = basicTypes[typeNdx];
                    var isFloat = gluShaderUtil.getDataTypeScalarType(type) == gluShaderUtil.DataType.FLOAT;

                    for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                        var precision = precisions[precNdx];
                        var name = gluShaderUtil.getPrecisionName(precision) + '_' + gluShaderUtil.getDataTypeName(type);

                        primitiveGroup.addChild(new es3fTransformFeedbackTests.BasicArrayCase(
                            name,
                            '',
                            bufferMode,
                            primitiveType,
                            type,
                            precision,
                            isFloat ? es3fTransformFeedbackTests.interpolation.SMOOTH : es3fTransformFeedbackTests.interpolation.FLAT
                        ));
                    }
                }
            }
        }

        // .array_element
        for (var bufferModeNdx = 0; bufferModeNdx < bufferModes.length; bufferModeNdx++) {
            var bufferMode = bufferModes[bufferModeNdx].mode;
            for (var primitiveTypeNdx = 0; primitiveTypeNdx < primitiveTypes.length; primitiveTypeNdx++) {
                var primitiveGroup = tcuTestCase.newTest(
                    'array_element.' + bufferModes[bufferModeNdx].name + '.' + primitiveTypes[primitiveTypeNdx].name,
                    'Capturing single array element in TF');
                var primitiveType = primitiveTypes[primitiveTypeNdx].type;
                testGroup.addChild(primitiveGroup);

                for (var typeNdx = 0; typeNdx < basicTypes.length; typeNdx++) {
                    var type = basicTypes[typeNdx];
                    var isFloat = gluShaderUtil.getDataTypeScalarType(type) == gluShaderUtil.DataType.FLOAT;

                    for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                        var precision = precisions[precNdx];
                        var name = gluShaderUtil.getPrecisionName(precision) + '_' + gluShaderUtil.getDataTypeName(type);

                        primitiveGroup.addChild(new es3fTransformFeedbackTests.ArrayElementCase(
                            name,
                            '',
                            bufferMode,
                            primitiveType,
                            type,
                            precision,
                            isFloat ? es3fTransformFeedbackTests.interpolation.SMOOTH : es3fTransformFeedbackTests.interpolation.FLAT
                        ));
                    }
                }
            }
        }

        // .interpolation
        for (var modeNdx = 0; modeNdx < interpModes.length; modeNdx++) {
            var interp = interpModes[modeNdx].interp;
            var modeGroup = tcuTestCase.newTest(
                'interpolation.' + interpModes[modeNdx].name,
                'Different interpolation modes in transform feedback varyings');
            testGroup.addChild(modeGroup);

            for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                var precision = precisions[precNdx];

                for (var primitiveType = 0; primitiveType < primitiveTypes.length; primitiveType++) {
                    for (var bufferMode = 0; bufferMode < bufferModes.length; bufferMode++) {
                        var name = (
                            gluShaderUtil.getPrecisionName(precision) +
                            '_vec4_' + primitiveTypes[primitiveType].name +
                            '_' + bufferModes[bufferMode].name
                        );

                        modeGroup.addChild(new es3fTransformFeedbackTests.BasicTypeCase(
                            name,
                            '',
                            bufferModes[bufferMode].mode,
                            primitiveTypes[primitiveType].type,
                            gluShaderUtil.DataType.FLOAT_VEC4,
                            precision,
                            interp
                        ));
                    }
                }
            }
        }

        // .random
        for (var bufferModeNdx = 0; bufferModeNdx < bufferModes.length; bufferModeNdx++) {
            /** @type {number} */
            var bufferMode = bufferModes[bufferModeNdx].mode;
            for (var primitiveTypeNdx = 0; primitiveTypeNdx < primitiveTypes.length; primitiveTypeNdx++) {
                var primitiveGroup = tcuTestCase.newTest(
                    'random.' + bufferModes[bufferModeNdx].name + '.' + primitiveTypes[primitiveTypeNdx].name,
                    'Randomized transform feedback cases');
                /** @type {number} */
                var primitiveType = primitiveTypes[primitiveTypeNdx].type;
                testGroup.addChild(primitiveGroup);

                for (var ndx = 0; ndx < 10; ndx++) {
                    /** @type {number} */
                    var seed = deMath.deMathHash(bufferMode) ^ deMath.deMathHash(primitiveType) ^ deMath.deMathHash(ndx);

                    primitiveGroup.addChild(new es3fTransformFeedbackTests.RandomCase(
                        (ndx + 1).toString(),
                        '',
                        bufferMode,
                        primitiveType,
                        seed
                    ));
                }
            }
        }

    };

    /**
     * Create and execute the test cases
     */
    es3fTransformFeedbackTests.run = function(context, range) {
        gl = context;
        var testName = 'transform_feedback';
        var testDescription = 'Transform Feedback Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);
        try {
            es3fTransformFeedbackTests.init();
            if (range)
                state.setRange(range);
            tcuTestCase.runTestCases();
        } catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }

    };

});
