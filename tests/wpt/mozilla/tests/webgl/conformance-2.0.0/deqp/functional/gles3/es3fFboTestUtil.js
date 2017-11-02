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
goog.provide('functional.gles3.es3fFboTestUtil');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');

goog.scope(function() {

var es3fFboTestUtil = functional.gles3.es3fFboTestUtil;
var tcuTexture = framework.common.tcuTexture;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuRGBA = framework.common.tcuRGBA;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var deMath = framework.delibs.debase.deMath;
var rrShadingContext = framework.referencerenderer.rrShadingContext;
var rrVertexPacket = framework.referencerenderer.rrVertexPacket;
var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
var gluShaderUtil = framework.opengl.gluShaderUtil;
var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;
var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
var rrGenericVector = framework.referencerenderer.rrGenericVector;
var tcuMatrix = framework.common.tcuMatrix;
var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
var tcuSurface = framework.common.tcuSurface;

var DE_ASSERT = function(x) {
    if (!x)
        throw new Error('Assert failed');
};

/**
 * Defines the exception type for a test failure.
 * @constructor
 * @param {number} reason The error code.
 */
es3fFboTestUtil.FboIncompleteException = function(reason) {
   this.reason = reason;
   this.name = 'es3fFboTestUtil.FboIncompleteException';
};

/** @typedef { (WebGL2RenderingContext|sglrReferenceContext.ReferenceContext)} */
es3fFboTestUtil.Context;

es3fFboTestUtil.FboIncompleteException.prototype.getReason = function() {return this.reason; };

    /**
     * @param {gluShaderUtil.DataType} type
     * @return {rrGenericVector.GenericVecType}
     */
    es3fFboTestUtil.mapDataTypeToGenericVecType = function(type) {
        switch (type) {
            case gluShaderUtil.DataType.FLOAT_VEC4: return rrGenericVector.GenericVecType.FLOAT;
            case gluShaderUtil.DataType.INT_VEC4: return rrGenericVector.GenericVecType.INT32;
            case gluShaderUtil.DataType.UINT_VEC4: return rrGenericVector.GenericVecType.UINT32;
            default:
                throw new Error('Unrecognized type: ' + type);
        }
    };

    /**
     * @param {Array<number>} input
     * @param {{max: number, min: number}} type min, max information
     * @return {Array<number>}
     */
    es3fFboTestUtil.castVectorSaturate = function(input, type) {
        return [
            (input[0] + 0.5 >= type.max) ? (type.max) : ((input[0] - 0.5 <= type.min) ? (type.min) : Math.round(input[0])),
            (input[1] + 0.5 >= type.max) ? (type.max) : ((input[1] - 0.5 <= type.min) ? (type.min) : Math.round(input[1])),
            (input[2] + 0.5 >= type.max) ? (type.max) : ((input[2] - 0.5 <= type.min) ? (type.min) : Math.round(input[2])),
            (input[3] + 0.5 >= type.max) ? (type.max) : ((input[3] - 0.5 <= type.min) ? (type.min) : Math.round(input[3]))
        ];
    };

    /**
     * es3fFboTestUtil.FlatColorShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {gluShaderUtil.DataType} outputType
     * @param {number=} pointSize
     */
    es3fFboTestUtil.FlatColorShader = function(outputType, pointSize) {
        pointSize = pointSize || 1;
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();
        /** @type {gluShaderUtil.DataType} */ this.m_outputType = outputType;

        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_color', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            ' gl_PointSize = ' + pointSize + '.0;\n' +
            '}\n'));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
            '#version 300 es\n' +
            'uniform highp vec4 u_color;\n' +
            'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color;\n' +
            'void main (void)\n' +
            '{\n' +
            ' o_color = ' + gluShaderUtil.getDataTypeName(outputType) + '(u_color);\n' +
            '}\n'));
        sglrShaderProgram.ShaderProgram.call(this, decl);
        this.m_pointSize = pointSize;
    };

    es3fFboTestUtil.FlatColorShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.FlatColorShader.prototype.constructor = es3fFboTestUtil.FlatColorShader;

    /**
     * @param {(WebGL2RenderingContext|sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext)} context
     * @param program GL program object
     * @param {Array<number>} color
     */
    es3fFboTestUtil.FlatColorShader.prototype.setColor = function(context, program, color) {
        /** @type {number} */ var location = context.getUniformLocation(program, 'u_color');

        context.useProgram(program);
        context.uniform4fv(location, color);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.FlatColorShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];
            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.pointSize = this.m_pointSize;
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.FlatColorShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @const {Array<number>} */ var color = this.m_uniforms[0].value;
        /** @const {Array<number>} */ var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
        /** @const {Array<number>} */ var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

        if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4) {
            for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx)
                packet[packetNdx].value = color;
        } else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4) {
            for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx)
                packet[packetNdx].value = icolor;
        } else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4) {
            for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx)
                packet[packetNdx].value = uicolor;
        } else
            throw new Error('Invalid output type: ' + this.m_outputType);
    };

    /**
     * es3fFboTestUtil.GradientShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {gluShaderUtil.DataType} outputType
     */
    es3fFboTestUtil.GradientShader = function(outputType) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();
        /** @type {gluShaderUtil.DataType} */ this.m_outputType = outputType;
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_coord', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_gradientMin', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_gradientMax', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in highp vec4 a_coord;\n' +
            'out highp vec4 v_coord;\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            ' v_coord = a_coord;\n' +
            '}\n'));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
            '#version 300 es\n' +
            'in highp vec4 v_coord;\n' +
            'uniform highp vec4 u_gradientMin;\n' +
            'uniform highp vec4 u_gradientMax;\n' +
            'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color;\n' +
            'void main (void)\n' +
            '{\n' +
            ' highp float x = v_coord.x;\n' +
            ' highp float y = v_coord.y;\n' +
            ' highp float f0 = (x + y) * 0.5;\n' +
            ' highp float f1 = 0.5 + (x - y) * 0.5;\n' +
            ' highp vec4 fv = vec4(f0, f1, 1.0f-f0, 1.0f-f1);\n' +
            ' o_color = ' + gluShaderUtil.getDataTypeName(outputType) + '(u_gradientMin + (u_gradientMax-u_gradientMin)*fv);\n' +
            '}\n'));
        sglrShaderProgram.ShaderProgram.call(this, decl);
    };

    es3fFboTestUtil.GradientShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.GradientShader.prototype.constructor = es3fFboTestUtil.GradientShader;

    /**
     * @param {es3fFboTestUtil.Context} ctx GL-like context
     * @param program GL program
     * @param {Array<number>} gradientMin
     * @param {Array<number>} gradientMax
     */
    es3fFboTestUtil.GradientShader.prototype.setGradient = function(ctx, program, gradientMin, gradientMax) {
        ctx.useProgram(program);
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_gradientMin'), gradientMin);
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_gradientMax'), gradientMax);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.GradientShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];

            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.outputs[0] = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.GradientShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @const {Array<number>} */ var gradientMin = this.m_uniforms[0].value;
        /** @const {Array<number>} */ var gradientMax = this.m_uniforms[1].value;

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @const {Array<number>} */ var coord = rrShadingContext.readTriangleVarying(packet[packetNdx], context, 0);
            /** @const {number} */ var x = coord[0];
            /** @const {number} */ var y = coord[1];
            /** @const {number} */ var f0 = (x + y) * 0.5;
            /** @const {number} */ var f1 = 0.5 + (x - y) * 0.5;
            /** @const {Array<number>} */ var fv = [f0, f1, 1.0 - f0, 1.0 - f1];

            /** @const {Array<number>} */ var color = deMath.add(gradientMin, deMath.multiply(deMath.subtract(gradientMax, gradientMin), fv));
            /** @const {Array<number>} */ var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
            /** @const {Array<number>} */ var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

            if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4)
                packet[packetNdx].value = color;
            else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4)
                packet[packetNdx].value = icolor;
            else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4)
                packet[packetNdx].value = uicolor;
            else
                throw new Error('Invalid output type: ' + this.m_outputType);
        }
    };

    /**
    * @param {Array<gluShaderUtil.DataType>} samplerTypes
    * @param {gluShaderUtil.DataType} outputType
    * @return {string}
     */
    es3fFboTestUtil.genTexFragmentShader = function(samplerTypes, outputType) {
        /** @type {string} */ var precision = 'highp';
        /** @type {string} */ var src = '';

        src = '#version 300 es\n' +
              'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color0;\n' +
              'in highp vec2 v_coord;\n';

        for (var samplerNdx = 0; samplerNdx < samplerTypes.length; samplerNdx++) {
            src += 'uniform ' + precision + ' ' + gluShaderUtil.getDataTypeName(samplerTypes[samplerNdx]) + ' u_sampler' + samplerNdx + ';\n' +
                   'uniform ' + precision + ' vec4 u_texScale' + samplerNdx + ';\n' +
                   'uniform ' + precision + ' vec4 u_texBias' + samplerNdx + ';\n';
        }

        // Output scale & bias
        src += 'uniform ' + precision + ' vec4 u_outScale0;\n' +
               'uniform ' + precision + ' vec4 u_outBias0;\n';

        src += '\n' +
               'void main (void)\n' +
               '{\n' +
               ' ' + precision + ' vec4 out0 = vec4(0.0);\n';

        // Texture input fetch and combine.
        for (var inNdx = 0; inNdx < samplerTypes.length; inNdx++)
            src += '\tout0 += vec4(' +
                   'texture(u_sampler' + inNdx + ', v_coord)) * u_texScale' + inNdx + ' + u_texBias' + inNdx + ';\n';

        // Write output.
        src += ' o_color0 = ' + gluShaderUtil.getDataTypeName(outputType) + '(out0 * u_outScale0 + u_outBias0);\n' +
               '}\n';

        return src;
    };

    /**
     * @param {Array<gluShaderUtil.DataType>} samplerTypes
     * @param {gluShaderUtil.DataType} outputType
     * @return {sglrShaderProgram.ShaderProgramDeclaration}
     */
    es3fFboTestUtil.genTexture2DShaderDecl = function(samplerTypes, outputType) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();

        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_coord', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));

        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in highp vec2 a_coord;\n' +
            'out highp vec2 v_coord;\n' +
            'void main(void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            ' v_coord = a_coord;\n' +
            '}\n'));

        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(es3fFboTestUtil.genTexFragmentShader(samplerTypes, outputType)));

        decl.pushUniform(new sglrShaderProgram.Uniform('u_outScale0', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_outBias0', gluShaderUtil.DataType.FLOAT_VEC4));

        for (var ndx = 0; ndx < samplerTypes.length; ++ndx) {
            decl.pushUniform(new sglrShaderProgram.Uniform('u_sampler' + ndx, samplerTypes[ndx]));
            decl.pushUniform(new sglrShaderProgram.Uniform('u_texScale' + ndx, gluShaderUtil.DataType.FLOAT_VEC4));
            decl.pushUniform(new sglrShaderProgram.Uniform('u_texBias' + ndx, gluShaderUtil.DataType.FLOAT_VEC4));
        }

        return decl;
    };

    /**
     * For use in es3fFboTestUtil.Texture2DShader
     * @constructor
     */
    es3fFboTestUtil.Input = function() {
        /** @type {number} */ this.unitNdx;
        /** @type {Array<number>} */ this.scale;
        /** @type {Array<number>} */ this.bias;
    };

    /**
     * es3fFboTestUtil.Texture2DShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {Array<gluShaderUtil.DataType>} samplerTypes
     * @param {gluShaderUtil.DataType} outputType
     * @param {Array<number>=} outScale - default [1.0, 1.0, 1.0, 1.0]
     * @param {Array<number>=} outBias - default [0.0, 0.0, 0.0, 0.0]
     */
    es3fFboTestUtil.Texture2DShader = function(samplerTypes, outputType, outScale, outBias) {
        if (outScale === undefined) outScale = [1.0, 1.0, 1.0, 1.0];
        if (outBias === undefined) outBias = [0.0, 0.0, 0.0, 0.0];
        sglrShaderProgram.ShaderProgram.call(this, es3fFboTestUtil.genTexture2DShaderDecl(samplerTypes, outputType));
        /** @type {Array<es3fFboTestUtil.Input>} */ this.m_inputs = [];
        /** @type {Array<number>} */ this.m_outScale = outScale;
        /** @type {Array<number>} */ this.m_outBias = outBias;
        /** @const {gluShaderUtil.DataType} */ this.m_outputType = outputType;
        for (var ndx = 0; ndx < samplerTypes.length; ndx++) {
            var input = new es3fFboTestUtil.Input();
            input.unitNdx = ndx;
            input.scale = [1.0, 1.0, 1.0, 1.0];
            input.bias = [0.0, 0.0, 0.0, 0.0];
            this.m_inputs[ndx] = input;
        }
    };

    es3fFboTestUtil.Texture2DShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.Texture2DShader.prototype.constructor = es3fFboTestUtil.Texture2DShader;

    /**
     * @param {number} inputNdx
     * @param {number} unitNdx
     */
    es3fFboTestUtil.Texture2DShader.prototype.setUnit = function(inputNdx, unitNdx) {
        this.m_inputs[inputNdx].unitNdx = unitNdx;
    };

    /**
     * @param {number} inputNdx
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     */
    es3fFboTestUtil.Texture2DShader.prototype.setTexScaleBias = function(inputNdx, scale, bias) {
        this.m_inputs[inputNdx].scale = scale;
        this.m_inputs[inputNdx].bias = bias;
    };

    /**
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     */
    es3fFboTestUtil.Texture2DShader.prototype.setOutScaleBias = function(scale, bias) {
        this.m_outScale = scale;
        this.m_outBias = bias;
    };

    /**
     * @param context GL-like context
     * @param program
     */
    es3fFboTestUtil.Texture2DShader.prototype.setUniforms = function(context, program) {
        context.useProgram(program);

        for (var texNdx = 0; texNdx < this.m_inputs.length; texNdx++) {
            /** @type {string} */ var samplerName = 'u_sampler' + texNdx;
            /** @type {string} */ var scaleName = 'u_texScale' + texNdx;
            /** @type {string} */ var biasName = 'u_texBias' + texNdx;

            context.uniform1i(context.getUniformLocation(program, samplerName), this.m_inputs[texNdx].unitNdx);
            context.uniform4fv(context.getUniformLocation(program, scaleName), this.m_inputs[texNdx].scale);
            context.uniform4fv(context.getUniformLocation(program, biasName), this.m_inputs[texNdx].bias);
        }

        context.uniform4fv(context.getUniformLocation(program, 'u_outScale0'), this.m_outScale);
        context.uniform4fv(context.getUniformLocation(program, 'u_outBias0'), this.m_outBias);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.Texture2DShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
        // TODO: implement rrVertexAttrib.readVertexAttribFloat
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];
            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.outputs[0] = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.Texture2DShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @type {Array<number>} */ var outScale = this.m_uniforms[0].value;
        /** @type {Array<number>} */ var outBias = this.m_uniforms[1].value;
        var texCoords = [];
        var colors = [];

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            // setup tex coords
                /** @const {Array<number>} */ var coord = rrShadingContext.readTriangleVarying(packet[packetNdx], context, 0);
                texCoords = [coord[0], coord[1]];

            // clear result
            colors = [0.0, 0.0, 0.0, 0.0];

            // sample each texture
            for (var ndx = 0; ndx < this.m_inputs.length; ndx++) {
                var tex = this.m_uniforms[2 + ndx * 3].sampler;
                var ratioX = tex.m_view.getWidth() / context.getWidth();
                var ratioY = tex.m_view.getHeight() / context.getHeight();
                var lod = Math.floor(Math.log2(Math.max(ratioX, ratioY)));

                /** @const {Array<number>} */ var scale = this.m_uniforms[2 + ndx * 3 + 1].value;
                /** @const {Array<number>} */ var bias = this.m_uniforms[2 + ndx * 3 + 2].value;

                var tmpColors = tex.sample(texCoords, lod);

                colors = deMath.add(colors, deMath.add(deMath.multiply(tmpColors, scale), bias));
            }

            // write out
            /** @const {Array<number>} */ var color = deMath.add(deMath.multiply(colors, outScale), outBias);
            /** @const {Array<number>} */ var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
            /** @const {Array<number>} */ var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

            if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4)
                packet[packetNdx].value = color;
            else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4)
                packet[packetNdx].value = icolor;
            else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4)
                packet[packetNdx].value = uicolor;
        }
    };

    /**
     * es3fFboTestUtil.TextureCubeShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {gluShaderUtil.DataType} samplerType
     * @param {gluShaderUtil.DataType} outputType
     */
    es3fFboTestUtil.TextureCubeShader = function(samplerType, outputType) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_coord', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_coordMat', gluShaderUtil.DataType.FLOAT_MAT3));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_sampler0', samplerType));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_scale', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_bias', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in mediump vec2 a_coord;\n' +
            'uniform mat3 u_coordMat;\n' +
            'out mediump vec3 v_coord;\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            ' v_coord = u_coordMat * vec3(a_coord, 1.0);\n' +
            '}\n'));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
            '#version 300 es\n' +
            'uniform highp ' + gluShaderUtil.getDataTypeName(samplerType) + ' u_sampler0;\n' +
            'uniform highp vec4 u_scale;\n' +
            'uniform highp vec4 u_bias;\n' +
            'in mediump vec3 v_coord;\n' +
            'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color;\n' +
            'void main (void)\n' +
            '{\n' +
            ' o_color = ' + gluShaderUtil.getDataTypeName(outputType) + '(vec4(texture(u_sampler0, v_coord)) * u_scale + u_bias);\n' +
            '}\n'));
        sglrShaderProgram.ShaderProgram.call(this, decl);
        /** @type {Array<number>} */ this.m_texScale = [1.0, 1.0, 1.0, 1.0];
        /** @type {Array<number>} */ this.m_texBias = [0.0, 0.0, 0.0, 0.0];
        /** @type {tcuMatrix.Mat3} */ this.m_coordMat;
        /** @type {gluShaderUtil.DataType} */ this.m_outputType = outputType;
    };

    es3fFboTestUtil.TextureCubeShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.TextureCubeShader.prototype.constructor = es3fFboTestUtil.TextureCubeShader;

    /**
     * @param {tcuTexture.CubeFace} face
     */
    es3fFboTestUtil.TextureCubeShader.prototype.setFace = function(face) {
        /** @const {Array<Array<number>>} */ var s_cubeTransforms = [
            // Face -X: (x, y, 1) -> (-1, -(2*y-1), +(2*x-1))
            [0, 0, -1,
             0, -2, 1,
             2, 0, -1],
            // Face +X: (x, y, 1) -> (+1, -(2*y-1), -(2*x-1))
            [0, 0, 1,
             0, -2, 1,
            -2, 0, 1],
            // Face -Y: (x, y, 1) -> (+(2*x-1), -1, -(2*y-1))
            [2, 0, -1,
             0, 0, -1,
             0, -2, 1],
            // Face +Y: (x, y, 1) -> (+(2*x-1), +1, +(2*y-1))
            [2, 0, -1,
             0, 0, 1,
             0, 2, -1],
            // Face -Z: (x, y, 1) -> (-(2*x-1), -(2*y-1), -1)
            [-2, 0, 1,
             0, -2, 1,
              0, 0, -1],
            // Face +Z: (x, y, 1) -> (+(2*x-1), -(2*y-1), +1)
            [2, 0, -1,
             0, -2, 1,
             0, 0, 1]];
        this.m_coordMat = /** @type {tcuMatrix.Mat3} */ (tcuMatrix.matrixFromArray(3, 3, s_cubeTransforms[face]));
    };

    /**
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     */
    es3fFboTestUtil.TextureCubeShader.prototype.setTexScaleBias = function(scale, bias) {
        this.m_texScale = scale;
        this.m_texBias = bias;
    };

    /**
     * @param ctx GL-like context
     * @param program
     */
    es3fFboTestUtil.TextureCubeShader.prototype.setUniforms = function(ctx, program) {
        ctx.useProgram(program);

        ctx.uniform1i(ctx.getUniformLocation(program, 'u_sampler0'), 0);
        ctx.uniformMatrix3fv(ctx.getUniformLocation(program, 'u_coordMat'), false, this.m_coordMat.getColumnMajorData());
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_scale'), this.m_texScale);
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_bias'), this.m_texBias);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.TextureCubeShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
    /** @type {tcuMatrix.Matrix} */ var texCoordMat = tcuMatrix.matrixFromArray(3, 3, this.m_uniforms[0].value);

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];
            var x = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT)[0];
            var y = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT)[1];
            /** @type {Array<number>} */ var a_coord = [x, y];
            /** @type {Array<number>} */ var v_coord = tcuMatrix.multiplyMatVec(texCoordMat, [a_coord[0], a_coord[1], 1.0]);

            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.outputs[0] = [v_coord[0], v_coord[1], v_coord[2], 0.0];
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.TextureCubeShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @const {Array<number>} */ var texScale = this.m_uniforms[2].value;
        /** @const {Array<number>} */ var texBias = this.m_uniforms[3].value;

        var texCoords = [];
        var colors = [];

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            var tex = this.m_uniforms[1].sampler;
            var ratioX = tex.m_view.getSize() / context.getWidth();
            var ratioY = tex.m_view.getSize() / context.getHeight();
            var lod = Math.floor(Math.log2(Math.max(ratioX, ratioY)));

            var coord = rrShadingContext.readTriangleVarying(packet[packetNdx], context, 0);
            texCoords = [coord[0], coord[1], coord[2]];

            colors = tex.sample(texCoords, lod);

            var color = deMath.add(deMath.multiply(colors, texScale), texBias);
            var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
            var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

            if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4)
                packet[packetNdx].value = color;
            else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4)
                packet[packetNdx].value = icolor;
            else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4)
                packet[packetNdx].value = uicolor;
        }
    };

    /**
     * es3fFboTestUtil.Texture2DArrayShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {gluShaderUtil.DataType} samplerType
     * @param {gluShaderUtil.DataType} outputType
     */
    es3fFboTestUtil.Texture2DArrayShader = function(samplerType, outputType) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_coord', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_sampler0', samplerType));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_scale', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_bias', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_layer', gluShaderUtil.DataType.INT));
        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
                '#version 300 es\n' +
                'in highp vec4 a_position;\n' +
                'in highp vec2 a_coord;\n' +
                'out highp vec2 v_coord;\n' +
                'void main (void)\n' +
                '{\n' +
                ' gl_Position = a_position;\n' +
                ' v_coord = a_coord;\n' +
                '}\n'));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
                '#version 300 es\n' +
                'uniform highp ' + gluShaderUtil.getDataTypeName(samplerType) + ' u_sampler0;\n' +
                'uniform highp vec4 u_scale;\n' +
                'uniform highp vec4 u_bias;\n' +
                'uniform highp int u_layer;\n' +
                'in highp vec2 v_coord;\n' +
                'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color;\n' +
                'void main (void)\n' +
                '{\n' +
                ' o_color = ' + gluShaderUtil.getDataTypeName(outputType) + '(vec4(texture(u_sampler0, vec3(v_coord, u_layer))) * u_scale + u_bias);\n' +
                '}\n'));
        sglrShaderProgram.ShaderProgram.call(this, decl);
        /** @type {Array<number>} */ this.m_texScale = [1.0, 1.0, 1.0, 1.0];
        /** @type {Array<number>} */ this.m_texBias = [0.0, 0.0, 0.0, 0.0];
        /** @type {number} */ this.m_layer = 0;
        /** @type {gluShaderUtil.DataType} */ this.m_outputType = outputType;
    };

    es3fFboTestUtil.Texture2DArrayShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.Texture2DArrayShader.prototype.constructor = es3fFboTestUtil.Texture2DArrayShader;

    /**
     * @param {number} layer
     */
    es3fFboTestUtil.Texture2DArrayShader.prototype.setLayer = function(layer) {
        this.m_layer = layer;
    };
    /**
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     */
    es3fFboTestUtil.Texture2DArrayShader.prototype.setTexScaleBias = function(scale, bias) {
        this.m_texScale = scale;
        this.m_texBias = bias;
    };
    /**
     * @param {es3fFboTestUtil.Context} ctx GL-like context
     * @param program
     */
    es3fFboTestUtil.Texture2DArrayShader.prototype.setUniforms = function(ctx, program) {
        ctx.useProgram(program);

        ctx.uniform1i(ctx.getUniformLocation(program, 'u_sampler0'), 0);
        ctx.uniform1i(ctx.getUniformLocation(program, 'u_layer'), this.m_layer);
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_scale'), this.m_texScale);
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_bias'), this.m_texBias);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.Texture2DArrayShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];

            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.outputs[0] = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.Texture2DArrayShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @const {Array<number>} */ var texScale = this.m_uniforms[1].value;
        /** @const {Array<number>} */ var texBias = this.m_uniforms[2].value;
        /** @const {number} */ var layer = this.m_uniforms[3].value[0];

        var texCoords = [];
        var colors = [];

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            var tex = this.m_uniforms[0].sampler;
            var ratioX = tex.m_view.getWidth() / context.getWidth();
            var ratioY = tex.m_view.getHeight() / context.getHeight();
            var lod = Math.floor(Math.log2(Math.max(ratioX, ratioY)));

            /** @const {Array<number>} */ var coord = rrShadingContext.readTriangleVarying(packet[packetNdx], context, 0);
            texCoords = [coord[0], coord[1], layer];

            colors = tex.sample(texCoords, lod);

            /** @const {Array<number>} */ var color = deMath.add(deMath.multiply(colors, texScale), texBias);
            /** @const {Array<number>} */ var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
            /** @const {Array<number>} */ var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

            if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4)
                packet[packetNdx].value = color;
            else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4)
                packet[packetNdx].value = icolor;
            else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4)
                packet[packetNdx].value = uicolor;
        }
    };

    /**
     * es3fFboTestUtil.Texture3DShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {gluShaderUtil.DataType} samplerType
     * @param {gluShaderUtil.DataType} outputType
     */
    es3fFboTestUtil.Texture3DShader = function(samplerType, outputType) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_coord', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_sampler0', samplerType));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_scale', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_bias', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_depth', gluShaderUtil.DataType.FLOAT));
        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
            '#version 300 es\n' +
            'in highp vec4 a_position;\n' +
            'in highp vec2 a_coord;\n' +
            'out highp vec2 v_coord;\n' +
            'void main (void)\n' +
            '{\n' +
            ' gl_Position = a_position;\n' +
            ' v_coord = a_coord;\n' +
            '}\n'));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
            '#version 300 es\n' +
            'uniform highp ' + gluShaderUtil.getDataTypeName(samplerType) + ' u_sampler0;\n' +
            'uniform highp vec4 u_scale;\n' +
            'uniform highp vec4 u_bias;\n' +
            'uniform highp float u_depth;\n' +
            'in highp vec2 v_coord;\n' +
            'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color;\n' +
            'void main (void)\n' +
            '{\n' +
            ' o_color = ' + gluShaderUtil.getDataTypeName(outputType) + '(vec4(texture(u_sampler0, vec3(v_coord, u_depth))) * u_scale + u_bias);\n' +
            '}\n'));
        sglrShaderProgram.ShaderProgram.call(this, decl);
        /** @type {Array<number>} */ this.m_texScale = [1.0, 1.0, 1.0, 1.0];
        /** @type {Array<number>} */ this.m_texBias = [0.0, 0.0, 0.0, 0.0];
        /** @type {number} */ this.m_depth = 0.0;
        /** @type {gluShaderUtil.DataType} */ this.m_outputType = outputType;
    };

    es3fFboTestUtil.Texture3DShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.Texture3DShader.prototype.constructor = es3fFboTestUtil.Texture3DShader;

    /**
     * @param {number} depth
     */
    es3fFboTestUtil.Texture3DShader.prototype.setDepth = function(depth) {
        this.m_depth = depth;
    };

    /**
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     */
    es3fFboTestUtil.Texture3DShader.prototype.setTexScaleBias = function(scale, bias) {
        this.m_texScale = scale;
        this.m_texBias = bias;
    };

    /**
     * @param context GL-like context
     * @param program
     */
    es3fFboTestUtil.Texture3DShader.prototype.setUniforms = function(context, program) {
        context.useProgram(program);
        context.uniform1i(context.getUniformLocation(program, 'u_sampler0'), 0);
        context.uniform1f(context.getUniformLocation(program, 'u_depth'), this.m_depth);
        context.uniform4fv(context.getUniformLocation(program, 'u_scale'), this.m_texScale);
        context.uniform4fv(context.getUniformLocation(program, 'u_bias'), this.m_texBias);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.Texture3DShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];

            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.outputs[0] = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.Texture3DShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @const {Array<number>} */ var texScale = this.m_uniforms[1].value;
        /** @const {Array<number>} */ var texBias = this.m_uniforms[2].value;
        /** @const {number} */ var depth = this.m_uniforms[3].value[0];

        var texCoords = [];
        var colors = [];

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            var tex = this.m_uniforms[0].sampler;
            var ratioX = tex.m_view.getWidth() / context.getWidth();
            var ratioY = tex.m_view.getHeight() / context.getHeight();
            // TODO: what to do with Z coordinate?
            var lod = Math.floor(Math.log2(Math.max(ratioX, ratioY)));

            var coord = rrShadingContext.readTriangleVarying(packet[packetNdx], context, 0);
            texCoords = [coord[0], coord[1], depth];

            colors = tex.sample(texCoords, lod);

            /** @const {Array<number>} */ var color = deMath.add(deMath.multiply(colors, texScale), texBias);
            /** @const {Array<number>} */ var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
            /** @const {Array<number>} */ var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

            if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4)
                packet[packetNdx].value = color;
            else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4)
                packet[packetNdx].value = icolor;
            else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4)
                packet[packetNdx].value = uicolor;
        }
    };

    /**
     * es3fFboTestUtil.DepthGradientShader inherits from sglrShaderProgram
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {gluShaderUtil.DataType} outputType
     */
    es3fFboTestUtil.DepthGradientShader = function(outputType) {
        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */
        var decl = new sglrShaderProgram.ShaderProgramDeclaration();
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_position', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('a_coord', rrGenericVector.GenericVecType.FLOAT));
        decl.pushVertexToFragmentVarying(new sglrShaderProgram.VertexToFragmentVarying(rrGenericVector.GenericVecType.FLOAT));
        decl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(es3fFboTestUtil.mapDataTypeToGenericVecType(outputType)));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_maxGradient', gluShaderUtil.DataType.FLOAT));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_minGradient', gluShaderUtil.DataType.FLOAT));
        decl.pushUniform(new sglrShaderProgram.Uniform('u_color', gluShaderUtil.DataType.FLOAT_VEC4));
        decl.pushVertexSource(new sglrShaderProgram.VertexSource(
                '#version 300 es\n' +
                'in highp vec4 a_position;\n' +
                'in highp vec4 a_coord;\n' +
                'out highp vec4 v_coord;\n' +
                'void main (void)\n' +
                '{\n' +
                ' gl_Position = a_position;\n' +
                ' v_coord = a_coord;\n' +
                '}\n'));
        decl.pushFragmentSource(new sglrShaderProgram.FragmentSource(
                    '#version 300 es\n' +
                    'in highp vec4 v_coord;\n' +
                    'uniform highp float u_minGradient;\n' +
                    'uniform highp float u_maxGradient;\n' +
                    'uniform highp vec4 u_color;\n' +
                    'layout(location = 0) out highp ' + gluShaderUtil.getDataTypeName(outputType) + ' o_color;\n' +
                    'void main (void)\n' +
                    '{\n' +
                    ' highp float x = v_coord.x;\n' +
                    ' highp float y = v_coord.y;\n' +
                    ' highp float f0 = (x + y) * 0.5;\n' +
                    ' gl_FragDepth = u_minGradient + (u_maxGradient-u_minGradient)*f0;\n' +
                    ' o_color = ' + gluShaderUtil.getDataTypeName(outputType) + '(u_color);\n' +
                    '}\n'));
        this.m_outputType = outputType;
        sglrShaderProgram.ShaderProgram.call(this, decl);
        /** @const {sglrShaderProgram.Uniform} */ this.u_minGradient = this.getUniformByName('u_minGradient');
        /** @const {sglrShaderProgram.Uniform} */ this.u_maxGradient = this.getUniformByName('u_maxGradient');
        /** @const {sglrShaderProgram.Uniform} */ this.u_color = this.getUniformByName('u_color');
    };

    es3fFboTestUtil.DepthGradientShader.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    es3fFboTestUtil.DepthGradientShader.prototype.constructor = es3fFboTestUtil.DepthGradientShader;

    /**
     * @param ctx GL-like context
     * @param program
     * @param {number} gradientMin
     * @param {number} gradientMax
     * @param {Array<number>} color
     */
    es3fFboTestUtil.DepthGradientShader.prototype.setUniforms = function(ctx, program, gradientMin, gradientMax, color) {
        ctx.useProgram(program);
        ctx.uniform1fv(ctx.getUniformLocation(program, 'u_minGradient'), [gradientMin]);
        ctx.uniform1fv(ctx.getUniformLocation(program, 'u_maxGradient'), [gradientMax]);
        ctx.uniform4fv(ctx.getUniformLocation(program, 'u_color'), color);
    };

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    es3fFboTestUtil.DepthGradientShader.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];

            packet.position = rrVertexAttrib.readVertexAttrib(inputs[0], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
            packet.outputs[0] = rrVertexAttrib.readVertexAttrib(inputs[1], packet.instanceNdx, packet.vertexNdx, rrGenericVector.GenericVecType.FLOAT);
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    es3fFboTestUtil.DepthGradientShader.prototype.shadeFragments = function(packet, context) {
        var numPackets = packet.length;
        /** @const {number} */ var gradientMin = this.u_minGradient.value[0];
        /** @const {number} */ var gradientMax = this.u_maxGradient.value[0];
        /** @type {Array<number>} */ var color = this.u_color.value;
        /** @type {Array<number>} */ var icolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deInt32);
        /** @type {Array<number>} */ var uicolor = es3fFboTestUtil.castVectorSaturate(color, tcuTexture.deTypes.deUint32);

        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {Array<number>} */ var coord = rrShadingContext.readTriangleVarying(packet[packetNdx], context, 0);
            /** @const {number} */ var x = coord[0];
            /** @const {number} */ var y = coord[1];
            /** @const {number} */ var f0 = (x + y) * 0.5;

            packet[packetNdx].sampleDepths[0] = gradientMin + (gradientMax - gradientMin) * f0;

            if (this.m_outputType == gluShaderUtil.DataType.FLOAT_VEC4)
                packet[packetNdx].value = color;
            else if (this.m_outputType == gluShaderUtil.DataType.INT_VEC4)
                packet[packetNdx].value = icolor;
            else if (this.m_outputType == gluShaderUtil.DataType.UINT_VEC4)
                packet[packetNdx].value = uicolor;
        }
    };

    es3fFboTestUtil.getFormatName = function(format) {
        switch (format) {
            case gl.RGB565: return 'rgb565';
            case gl.RGB5_A1: return 'rgb5_a1';
            case gl.RGBA4: return 'rgba4';
            case gl.DEPTH_COMPONENT16: return 'depth_component16';
            case gl.STENCIL_INDEX8: return 'stencil_index8';
            case gl.RGBA32F: return 'rgba32f';
            case gl.RGBA32I: return 'rgba32i';
            case gl.RGBA32UI: return 'rgba32ui';
            case gl.RGBA16F: return 'rgba16f';
            case gl.RGBA16I: return 'rgba16i';
            case gl.RGBA16UI: return 'rgba16ui';
            case gl.RGBA8: return 'rgba8';
            case gl.RGBA8I: return 'rgba8i';
            case gl.RGBA8UI: return 'rgba8ui';
            case gl.SRGB8_ALPHA8: return 'srgb8_alpha8';
            case gl.RGB10_A2: return 'rgb10_a2';
            case gl.RGB10_A2UI: return 'rgb10_a2ui';
            case gl.RGBA8_SNORM: return 'rgba8_snorm';
            case gl.RGB8: return 'rgb8';
            case gl.R11F_G11F_B10F: return 'r11f_g11f_b10f';
            case gl.RGB32F: return 'rgb32f';
            case gl.RGB32I: return 'rgb32i';
            case gl.RGB32UI: return 'rgb32ui';
            case gl.RGB16F: return 'rgb16f';
            case gl.RGB16I: return 'rgb16i';
            case gl.RGB16UI: return 'rgb16ui';
            case gl.RGB8_SNORM: return 'rgb8_snorm';
            case gl.RGB8I: return 'rgb8i';
            case gl.RGB8UI: return 'rgb8ui';
            case gl.SRGB8: return 'srgb8';
            case gl.RGB9_E5: return 'rgb9_e5';
            case gl.RG32F: return 'rg32f';
            case gl.RG32I: return 'rg32i';
            case gl.RG32UI: return 'rg32ui';
            case gl.RG16F: return 'rg16f';
            case gl.RG16I: return 'rg16i';
            case gl.RG16UI: return 'rg16ui';
            case gl.RG8: return 'rg8';
            case gl.RG8I: return 'rg8i';
            case gl.RG8UI: return 'rg8ui';
            case gl.RG8_SNORM: return 'rg8_snorm';
            case gl.R32F: return 'r32f';
            case gl.R32I: return 'r32i';
            case gl.R32UI: return 'r32ui';
            case gl.R16F: return 'r16f';
            case gl.R16I: return 'r16i';
            case gl.R16UI: return 'r16ui';
            case gl.R8: return 'r8';
            case gl.R8I: return 'r8i';
            case gl.R8UI: return 'r8ui';
            case gl.R8_SNORM: return 'r8_snorm';
            case gl.DEPTH_COMPONENT32F: return 'depth_component32f';
            case gl.DEPTH_COMPONENT24: return 'depth_component24';
            case gl.DEPTH32F_STENCIL8: return 'depth32f_stencil8';
            case gl.DEPTH24_STENCIL8: return 'depth24_stencil8';

            default:
                throw new Error('Unknown format in getFromatName()');
        }
    };

    /**
     * @param {tcuTexture.TextureFormat} format
     * @return {gluShaderUtil.DataType}
     */
    es3fFboTestUtil.getFragmentOutputType = function(format) {
        switch (tcuTexture.getTextureChannelClass(format.type)) {
            case tcuTexture.TextureChannelClass.FLOATING_POINT:
            case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
            case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
                return gluShaderUtil.DataType.FLOAT_VEC4;

            case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
                return gluShaderUtil.DataType.UINT_VEC4;

            case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                return gluShaderUtil.DataType.INT_VEC4;

            default:
                throw new Error('Unknown format');
        }
    };

    /**
     * @param {tcuTexture.TextureFormat} format
     * @return {tcuTexture.TextureFormat}
     */
    es3fFboTestUtil.getFramebufferReadFormat = function(format) {
        switch (tcuTexture.getTextureChannelClass(format.type)) {
            case tcuTexture.TextureChannelClass.FLOATING_POINT:
                return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.FLOAT);

            case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
            case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
                return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNORM_INT8);

            case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
                return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.UNSIGNED_INT32);

            case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                return new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA, tcuTexture.ChannelType.SIGNED_INT32);

            default:
                throw new Error('Unknown format in es3fFboTestUtil.getFramebufferReadFormat()');
        }
    };

    /**
     * @param {es3fFboTestUtil.Context} ctx GL-like context
     * @param {tcuTexture.TextureFormat} format
     * @param {Array<number>} value
     */
    es3fFboTestUtil.clearColorBuffer = function(ctx, format, value) {
        /** @const @type {tcuTexture.TextureChannelClass} */
        var fmtClass = tcuTexture.getTextureChannelClass(format.type);

        switch (fmtClass) {
            case tcuTexture.TextureChannelClass.FLOATING_POINT:
            case tcuTexture.TextureChannelClass.SIGNED_FIXED_POINT:
            case tcuTexture.TextureChannelClass.UNSIGNED_FIXED_POINT:
                ctx.clearBufferfv(gl.COLOR, 0, value);
                break;

            case tcuTexture.TextureChannelClass.UNSIGNED_INTEGER:
                ctx.clearBufferuiv(gl.COLOR, 0, value);
                break;

            case tcuTexture.TextureChannelClass.SIGNED_INTEGER:
                ctx.clearBufferiv(gl.COLOR, 0, value);
                break;

            default:
                throw new Error('Invalid channel class: ' + fmtClass);
        }
    };

    /**
     * @param {tcuTexture.TextureFormat} format
     * @return {tcuRGBA.RGBA}
     */
    es3fFboTestUtil.getThresholdFromTextureFormat = function(format) {
        /** @const @type {Array<number>} */ var bits = tcuTextureUtil.getTextureFormatMantissaBitDepth(format);
        return tcuRGBA.newRGBAComponents(
            es3fFboTestUtil.calculateU8ConversionError(bits[0]),
            es3fFboTestUtil.calculateU8ConversionError(bits[1]),
            es3fFboTestUtil.calculateU8ConversionError(bits[2]),
            es3fFboTestUtil.calculateU8ConversionError(bits[3])
        );
    };

    /**
     * @param {number} glFormat
     * @return {tcuRGBA.RGBA}
     */
    es3fFboTestUtil.getFormatThreshold = function(glFormat) {
        /** @const @type {tcuTexture.TextureFormat} */ var format = gluTextureUtil.mapGLInternalFormat(glFormat);
        return es3fFboTestUtil.getThresholdFromTextureFormat(format);
    };

    /**
     * @param {number} srcBits
     * @return {number}
     */
    es3fFboTestUtil.getToSRGB8ConversionError = function(srcBits) {
        // \note These are pre-computed based on simulation results.
        /** @const @type {Array<number>} */ var errors = [
            1, // 0 bits - rounding
            255, // 1 bits
            157, // 2 bits
            106, // 3 bits
            74, // 4 bits
            51, // 5 bits
            34, // 6 bits
            22, // 7 bits
            13, // 8 bits
            7, // 9 bits
            4, // 10 bits
            3, // 11 bits
            2 // 12 bits
            // 1 from this on
        ];

        DE_ASSERT(srcBits >= 0);
        if (srcBits < errors.length)
            return errors[srcBits];
        else
            return 1;
    };

    /**
     * @param {tcuTexture.TextureFormat} src
     * @param {tcuTexture.TextureFormat} dst
     * @return {tcuRGBA.RGBA}
     */
    es3fFboTestUtil.getToSRGBConversionThreshold = function(src, dst) {
        // Only SRGB8 and SRGB8_ALPHA8 formats are supported.
        DE_ASSERT(dst.type == tcuTexture.ChannelType.UNORM_INT8);
        DE_ASSERT(dst.order == tcuTexture.ChannelOrder.sRGB || dst.order == tcuTexture.ChannelOrder.sRGBA);

        /** @const @type {Array<number>} */ var bits = tcuTextureUtil.getTextureFormatMantissaBitDepth(src);
        /** @const @type {boolean} */ var dstHasAlpha = dst.order == tcuTexture.ChannelOrder.sRGBA;

        return tcuRGBA.newRGBAComponents(
            es3fFboTestUtil.getToSRGB8ConversionError(bits[0]),
            es3fFboTestUtil.getToSRGB8ConversionError(bits[1]),
            es3fFboTestUtil.getToSRGB8ConversionError(bits[2]),
            dstHasAlpha ? es3fFboTestUtil.calculateU8ConversionError(bits[3]) : 0);
    };

    /**
     * es3fFboTestUtil.readPixels()
     * @param {(WebGL2RenderingContext|sglrGLContext.GLContext|sglrReferenceContext.ReferenceContext)} ctx
     * @param {tcuSurface.Surface} dst
     * @param {number} x
     * @param {number} y
     * @param {number} width
     * @param {number} height
     * @param {tcuTexture.TextureFormat} format
     * @param {Array<number>} scale
     * @param {Array<number>} bias
     */
    es3fFboTestUtil.readPixels = function(ctx, dst, x, y, width, height, format, scale, bias) {
        /** @type {tcuTexture.TextureFormat} */ var readFormat = es3fFboTestUtil.getFramebufferReadFormat(format);
        /** @type {gluTextureUtil.TransferFormat} */ var transferFmt = gluTextureUtil.getTransferFormat(readFormat);
        /** @type {number} */ var alignment = 4; // \note gl.PACK_ALIGNMENT = 4 is assumed.
        /** @type {number} */ var rowSize = deMath.deAlign32(readFormat.getPixelSize() * width, alignment);
        var typedArrayType = tcuTexture.getTypedArray(readFormat.type);
        var data = new typedArrayType(rowSize * height);
        ctx.readPixels(x, y, width, height, transferFmt.format, transferFmt.dataType, data);

        // Convert to surface.
        var cpbaDescriptor = {
            format: readFormat,
            width: width,
            height: height,
            depth: 1,
            rowPitch: rowSize,
            slicePitch: 0,
            data: data.buffer
        };

        /** @type {tcuTexture.ConstPixelBufferAccess} */
        var src = new tcuTexture.ConstPixelBufferAccess(cpbaDescriptor);

        dst.setSize(width, height);
        /** @type {tcuTexture.PixelBufferAccess} */ var dstAccess = dst.getAccess();

        for (var yo = 0; yo < height; yo++)
        for (var xo = 0; xo < width; xo++)
            dstAccess.setPixel(deMath.add(deMath.multiply(src.getPixel(xo, yo), scale), bias), xo, yo);
    };

    /**
     * @param {number} srcBits
     * @return {number}
     */
    es3fFboTestUtil.calculateU8ConversionError = function(srcBits) {
        if (srcBits > 0) {
            /** @const @type {number} */ var clampedBits = deMath.clamp(srcBits, 0, 8);
            /** @const @type {number} */ var srcMaxValue = Math.max((1 << clampedBits) - 1, 1);
            /** @const @type {number} */ var error = Math.floor(Math.ceil(255.0 * 2.0 / srcMaxValue));

            return deMath.clamp(error, 0, 255);
        } else
            return 1;
    };

});
