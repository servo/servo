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
goog.provide('modules.shared.glsSamplerObjectTest');
goog.require('framework.common.tcuImageCompare');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deString');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluTextureUtil');

goog.scope(function() {

var glsSamplerObjectTest = modules.shared.glsSamplerObjectTest;
var tcuTestCase = framework.common.tcuTestCase;
var deRandom = framework.delibs.debase.deRandom;
var gluShaderProgram = framework.opengl.gluShaderProgram;
var tcuTexture = framework.common.tcuTexture;
var tcuSurface = framework.common.tcuSurface;
var tcuTextureUtil = framework.common.tcuTextureUtil;
var tcuImageCompare = framework.common.tcuImageCompare;
var gluDrawUtil = framework.opengl.gluDrawUtil;
var gluTextureUtil = framework.opengl.gluTextureUtil;
var deString = framework.delibs.debase.deString;

    var DE_ASSERT = function(expression) {
        if (!expression) throw new Error('Assert failed');
    };

    // glsSamplerObjectTest.TextureSamplerTest

    /** @const @type {number} */ glsSamplerObjectTest.VIEWPORT_WIDTH = 128;
    /** @const @type {number} */ glsSamplerObjectTest.VIEWPORT_HEIGHT = 128;

    /** @const @type {number} */ glsSamplerObjectTest.TEXTURE2D_WIDTH = 32;
    /** @const @type {number} */ glsSamplerObjectTest.TEXTURE2D_HEIGHT = 32;

    /** @const @type {number} */ glsSamplerObjectTest.TEXTURE3D_WIDTH = 32;
    /** @const @type {number} */ glsSamplerObjectTest.TEXTURE3D_HEIGHT = 32;
    /** @const @type {number} */ glsSamplerObjectTest.TEXTURE3D_DEPTH = 32;

    /** @const @type {number} */ glsSamplerObjectTest.CUBEMAP_SIZE = 32;

    /** @const @type {Array<number>} */ glsSamplerObjectTest.s_positions = [
        -1.0, -1.0,
         1.0, -1.0,
         1.0, 1.0,
         1.0, 1.0,
        -1.0, 1.0,
        -1.0, -1.0
    ];

    /** @const @type {Array<number>} */ glsSamplerObjectTest.s_positions3D = [
        -1.0, -1.0, -1.0,
         1.0, -1.0, 1.0,
         1.0, 1.0, -1.0,
         1.0, 1.0, -1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, -1.0
    ];

    /** @const @type {Array<number>} */ glsSamplerObjectTest.s_positionsCube = [
        -1.0, -1.0, -1.0, -0.5,
         1.0, -1.0, 1.0, -0.5,
         1.0, 1.0, 1.0, 0.5,
         1.0, 1.0, 1.0, 0.5,
        -1.0, 1.0, -1.0, 0.5,
        -1.0, -1.0, -1.0, -0.5
    ];

    /**
     * @struct
     * @constructor
     */
    glsSamplerObjectTest.SamplingState = function(minFilter, magFilter, wrapT, wrapS, wrapR, minLod, maxLod) {
        /** @type {number} */ this.minFilter = minFilter;
        /** @type {number} */ this.magFilter = magFilter;
        /** @type {number} */ this.wrapT = wrapT;
        /** @type {number} */ this.wrapS = wrapS;
        /** @type {number} */ this.wrapR = wrapR;
        /** @type {number} */ this.minLod = minLod;
        /** @type {number} */ this.maxLod = maxLod;
    };

    /**
     * @struct
     * @param {string} name
     * @param {string} desc
     * @param {number} target
     * @param {glsSamplerObjectTest.SamplingState} state1
     * @param {glsSamplerObjectTest.SamplingState} state2
     * @param {glsSamplerObjectTest.SamplingState=} state3
     * @constructor
     */
    glsSamplerObjectTest.TestSpec = function(name, desc, target, state1, state2, state3) {
        /** @type {string} */ this.name = name;
        /** @type {string} */ this.desc = desc;
        /** @type {number} */ this.target = target;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.textureState = state1;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.textureState2 = state3 !== undefined ? state2 : null; // merging TST and MTST structs
        /** @type {glsSamplerObjectTest.SamplingState} */ this.samplerState = state3 !== undefined ? state3 : state2;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @const @param {glsSamplerObjectTest.TestSpec} spec
     */
    glsSamplerObjectTest.TextureSamplerTest = function(spec) {
        tcuTestCase.DeqpTest.call(this, spec.name, spec.desc);
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {number} */ this.m_target = spec.target;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.m_textureState = spec.textureState;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.m_samplerState = spec.samplerState;
        /** @type {deRandom.Random} */ this.m_random = new deRandom.Random(deString.deStringHash(spec.name));
    };

    glsSamplerObjectTest.TextureSamplerTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsSamplerObjectTest.TextureSamplerTest.prototype.constructor = glsSamplerObjectTest.TextureSamplerTest;

    /**
     * @private
     * @param {tcuSurface.Surface} textureRef
     * @param {tcuSurface.Surface} samplerRef
     * @param {number} x
     * @param {number} y
     */
    glsSamplerObjectTest.TextureSamplerTest.prototype.renderReferences = function(textureRef, samplerRef, x, y) {
        /** @type {WebGLTexture} */ var texture = glsSamplerObjectTest.TextureSamplerTest.createTexture(this.m_target);

        gl.viewport(x, y, glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        gl.bindTexture(this.m_target, texture);

        glsSamplerObjectTest.TextureSamplerTest.setTextureState(this.m_target, this.m_textureState);
        this.render();
        var texRef = textureRef.getAccess();
        var texRefTransferFormat = gluTextureUtil.getTransferFormat(texRef.getFormat());
        gl.readPixels(x, y, texRef.m_width, texRef.m_height, texRefTransferFormat.format, texRefTransferFormat.dataType, textureRef.m_pixels);

        glsSamplerObjectTest.TextureSamplerTest.setTextureState(this.m_target, this.m_samplerState);
        this.render();
        var sampRef = samplerRef.getAccess();
        var sampRefTransferFormat = gluTextureUtil.getTransferFormat(sampRef.getFormat());
        gl.readPixels(x, y, sampRef.m_width, sampRef.m_height, sampRefTransferFormat.format, sampRefTransferFormat.dataType, samplerRef.m_pixels);

        gl.deleteTexture(texture);
    };

    /**
     * @private
     * @param {tcuSurface.Surface} textureResult
     * @param {tcuSurface.Surface} samplerResult
     * @param {number} x
     * @param {number} y
     */
    glsSamplerObjectTest.TextureSamplerTest.prototype.renderResults = function(textureResult, samplerResult, x, y) {
        /** @type {WebGLTexture} */ var texture = glsSamplerObjectTest.TextureSamplerTest.createTexture(this.m_target);

        gl.viewport(x, y, glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        var sampler = gl.createSampler();
        DE_ASSERT(sampler != -1);

        gl.bindSampler(0, sampler);

        // First set sampler state
        glsSamplerObjectTest.TextureSamplerTest.setSamplerState(this.m_samplerState, sampler);

        // Set texture state
        gl.bindTexture(this.m_target, texture);

        glsSamplerObjectTest.TextureSamplerTest.setTextureState(this.m_target, this.m_textureState);
        // Render using sampler
        this.render();
        var sampRes = samplerResult.getAccess();
        var sampResTransferFormat = gluTextureUtil.getTransferFormat(sampRes.getFormat());
        gl.readPixels(x, y, sampRes.m_width, sampRes.m_height, sampResTransferFormat.format, sampResTransferFormat.dataType, samplerResult.m_pixels);

        // Render without sampler
        gl.bindSampler(0, null);
        gl.deleteSampler(sampler);

        this.render();
        var texRes = textureResult.getAccess();
        var texResTransferFormat = gluTextureUtil.getTransferFormat(texRes.getFormat());
        gl.readPixels(x, y, texRes.m_width, texRes.m_height, texResTransferFormat.format, texResTransferFormat.dataType, textureResult.m_pixels);

        gl.deleteSampler(sampler);
        gl.deleteTexture(texture);
    };

    /**
     * @private
     */
    glsSamplerObjectTest.TextureSamplerTest.prototype.render = function() {
        /** @type {WebGLUniformLocation} */ var samplerLoc;
        /** @type {WebGLUniformLocation} */ var scaleLoc;

        gl.useProgram(this.m_program.getProgram());

        samplerLoc = gl.getUniformLocation(this.m_program.getProgram(), 'u_sampler');
        DE_ASSERT(samplerLoc != null);

        scaleLoc = gl.getUniformLocation(this.m_program.getProgram(), 'u_posScale');
        DE_ASSERT(scaleLoc != null);

        gl.clearColor(0.5, 0.5, 0.5, 1.0);

        gl.clear(gl.COLOR_BUFFER_BIT);

        gl.uniform1i(samplerLoc, 0);

        gl.uniform1f(scaleLoc, 1.0);

        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays;
        switch (this.m_target) {
            case gl.TEXTURE_2D: {
                vertexArrays = [
                    gluDrawUtil.vabFromBindingPointAndArrayPointer(
                        gluDrawUtil.bindingPointFromName('a_position'),
                        new gluDrawUtil.VertexArrayPointer(
                            gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
                            gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE,
                            2,
                            6,
                            0,
                            glsSamplerObjectTest.s_positions))
                ];

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                gl.uniform1f(scaleLoc, 0.25);

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                break;
            }

            case gl.TEXTURE_3D: {
                vertexArrays = [
                    gluDrawUtil.vabFromBindingPointAndArrayPointer(
                        gluDrawUtil.bindingPointFromName('a_position'),
                        new gluDrawUtil.VertexArrayPointer(
                            gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
                            gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE,
                            3,
                            6,
                            0,
                            glsSamplerObjectTest.s_positions3D))
                ];

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                gl.uniform1f(scaleLoc, 0.25);

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                break;
            }

            case gl.TEXTURE_CUBE_MAP: {
                vertexArrays = [
                    gluDrawUtil.vabFromBindingPointAndArrayPointer(
                        gluDrawUtil.bindingPointFromName('a_position'),
                        new gluDrawUtil.VertexArrayPointer(
                            gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
                            gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE,
                            4,
                            6,
                            0,
                            glsSamplerObjectTest.s_positionsCube))
                ];

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                gl.uniform1f(scaleLoc, 0.25);

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                break;
            }

            default:
                DE_ASSERT(false);
        }
    };

    /**
     * @private
     * @param {number} target
     * @param {glsSamplerObjectTest.SamplingState} state
     */
    glsSamplerObjectTest.TextureSamplerTest.setTextureState = function(target, state) {
        gl.texParameteri(target, gl.TEXTURE_MIN_FILTER, state.minFilter);
        gl.texParameteri(target, gl.TEXTURE_MAG_FILTER, state.magFilter);
        gl.texParameteri(target, gl.TEXTURE_WRAP_S, state.wrapS);
        gl.texParameteri(target, gl.TEXTURE_WRAP_T, state.wrapT);
        gl.texParameteri(target, gl.TEXTURE_WRAP_R, state.wrapR);
        gl.texParameterf(target, gl.TEXTURE_MAX_LOD, state.maxLod);
        gl.texParameterf(target, gl.TEXTURE_MIN_LOD, state.minLod);
    };

    /**
     * @private
     * @param {glsSamplerObjectTest.SamplingState} state
     * @param {WebGLSampler} sampler
     */
    glsSamplerObjectTest.TextureSamplerTest.setSamplerState = function(state, sampler) {
        gl.samplerParameteri(sampler, gl.TEXTURE_MIN_FILTER, state.minFilter);
        gl.samplerParameteri(sampler, gl.TEXTURE_MAG_FILTER, state.magFilter);
        gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_S, state.wrapS);
        gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_T, state.wrapT);
        gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_R, state.wrapR);
        gl.samplerParameterf(sampler, gl.TEXTURE_MAX_LOD, state.maxLod);
        gl.samplerParameterf(sampler, gl.TEXTURE_MIN_LOD, state.minLod);
    };

    /**
     * @private
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.TextureSamplerTest.createTexture2D = function() {
        /** @type {WebGLTexture} */ var texture = null;
        /** @type {tcuTexture.Texture2D} */ var refTexture = new tcuTexture.Texture2D(
                                                                new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA,
                                                                                             tcuTexture.ChannelType.UNORM_INT8),
                                                                glsSamplerObjectTest.TEXTURE2D_WIDTH,
                                                                glsSamplerObjectTest.TEXTURE2D_HEIGHT);

        refTexture.allocLevel(0);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevel(0), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);

        texture = gl.createTexture();

        gl.bindTexture(gl.TEXTURE_2D, texture);

        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, refTexture.getWidth(), refTexture.getHeight(), 0, gl.RGBA, gl.UNSIGNED_BYTE, refTexture.getLevel(0).getDataPtr());

        gl.generateMipmap(gl.TEXTURE_2D);

        gl.bindTexture(gl.TEXTURE_2D, null);

        return texture;
    };

    /**
     * @private
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.TextureSamplerTest.createTexture3D = function() {
        /** @type {WebGLTexture} */ var texture = null;
        /** @type {tcuTexture.Texture3D} */ var refTexture = new tcuTexture.Texture3D(
                                                                 new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA,
                                                                                              tcuTexture.ChannelType.UNORM_INT8),
                                                                 glsSamplerObjectTest.TEXTURE3D_WIDTH,
                                                                 glsSamplerObjectTest.TEXTURE3D_HEIGHT,
                                                                 glsSamplerObjectTest.TEXTURE3D_DEPTH);

        refTexture.allocLevel(0);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevel(0), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);

        texture = gl.createTexture();

        gl.bindTexture(gl.TEXTURE_3D, texture);

        gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, refTexture.getWidth(), refTexture.getHeight(), refTexture.getDepth(), 0, gl.RGBA, gl.UNSIGNED_BYTE, refTexture.getLevel(0).getDataPtr());

        gl.generateMipmap(gl.TEXTURE_3D);

        gl.bindTexture(gl.TEXTURE_3D, null);

        return texture;
    };

    /**
     * @private
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.TextureSamplerTest.createTextureCube = function() {
        /** @type {WebGLTexture} */ var texture = null;
        /** @type {tcuTexture.TextureCube} */ var refTexture = new tcuTexture.TextureCube(
                                                                  new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA,
                                                                                               tcuTexture.ChannelType.UNORM_INT8),
                                                                  glsSamplerObjectTest.CUBEMAP_SIZE);

        texture = gl.createTexture();

        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_POSITIVE_X, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z, 0);

        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_X), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);

        gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture);
        // TODO: check internalFormat / format parameters in texImage2D (were RGBA8 and RGBA respectively)
        for (var face in tcuTexture.CubeFace) {
            /** @const @type {number} */ var target = gluTextureUtil.getGLCubeFace(tcuTexture.CubeFace[face]);
            gl.texImage2D(target, 0, gl.RGBA, refTexture.getSize(), refTexture.getSize(), 0, gl.RGBA, gl.UNSIGNED_BYTE, refTexture.getLevelFace(0, tcuTexture.CubeFace[face]).getDataPtr());
        }

        gl.generateMipmap(gl.TEXTURE_CUBE_MAP);
        gl.bindTexture(gl.TEXTURE_CUBE_MAP, null);

        return texture;
    };

    /**
     * @private
     * @param {number} target
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.TextureSamplerTest.createTexture = function(target) {
        /** @type {WebGLTexture} */ var texture;
        switch (target) {
            case gl.TEXTURE_2D:
                texture = glsSamplerObjectTest.TextureSamplerTest.createTexture2D();
                break;

            case gl.TEXTURE_3D:
                texture = glsSamplerObjectTest.TextureSamplerTest.createTexture3D();
                break;

            case gl.TEXTURE_CUBE_MAP:
                texture = glsSamplerObjectTest.TextureSamplerTest.createTextureCube();
                break;

            default:
                throw new Error('Unsupported target: ' + WebGLTestUtils.glEnumToString(gl, target));
        }

        return texture;
    };

    /**
     * @private
     * @param {number} target
     * @return {string}
     */
     glsSamplerObjectTest.TextureSamplerTest.selectVertexShader = function(target) {
         switch (target) {
             case gl.TEXTURE_2D:
                 return '#version 300 es\n' +
                 'in highp vec2 a_position;\n' +
                 'uniform highp float u_posScale;\n' +
                 'out mediump vec2 v_texCoord;\n' +
                 'void main (void)\n' +
                 ' {\n' +
                 '\tv_texCoord = a_position;\n' +
                 '\tgl_Position = vec4(u_posScale * a_position, 0.0, 1.0);\n' +
                 '}';

             case gl.TEXTURE_3D:
                 return '#version 300 es\n' +
                 'in highp vec3 a_position;\n' +
                 'uniform highp float u_posScale;\n' +
                 'out mediump vec3 v_texCoord;\n' +
                 'void main (void)\n' +
                 ' {\n' +
                 '\tv_texCoord = a_position;\n' +
                 '\tgl_Position = vec4(u_posScale * a_position.xy, 0.0, 1.0);\n' +
                 '}';

             case gl.TEXTURE_CUBE_MAP:
                 return '#version 300 es\n' +
                 'in highp vec4 a_position;\n' +
                 'uniform highp float u_posScale;\n' +
                 'out mediump vec2 v_texCoord;\n' +
                 'void main (void)\n' +
                 ' {\n' +
                 '\tv_texCoord = a_position.zw;\n' +
                 '\tgl_Position = vec4(u_posScale * a_position.xy, 0.0, 1.0);\n' +
                 '}';

             default:
                 DE_ASSERT(false);
                 return '';
         }
     };

     /**
      * @private
      * @param {number} target
      * @return {string}
      */
     glsSamplerObjectTest.TextureSamplerTest.selectFragmentShader = function(target) {
         switch (target) {
             case gl.TEXTURE_2D:
                 return '#version 300 es\nlayout(location = 0) out mediump vec4 o_color;\n' +
                 'uniform lowp sampler2D u_sampler;\n' +
                 'in mediump vec2 v_texCoord;\n' +
                 'void main (void)\n' +
                 ' {\n' +
                 '\to_color = texture(u_sampler, v_texCoord);\n' +
                 '}';

             case gl.TEXTURE_3D:
                 return '#version 300 es\nlayout(location = 0) out mediump vec4 o_color;\n' +
                 'uniform lowp sampler3D u_sampler;\n' +
                 'in mediump vec3 v_texCoord;\n' +
                 'void main (void)\n' +
                 ' {\n' +
                 '\to_color = texture(u_sampler, v_texCoord);\n' +
                 '}';

             case gl.TEXTURE_CUBE_MAP:
                 return '#version 300 es\nlayout(location = 0) out mediump vec4 o_color;\n' +
                 'uniform lowp samplerCube u_sampler;\n' +
                 'in mediump vec2 v_texCoord;\n' +
                 'void main (void)\n' +
                 ' {\n' +
                 '\to_color = texture(u_sampler, vec3(cos(3.14 * v_texCoord.y) * sin(3.14 * v_texCoord.x), sin(3.14 * v_texCoord.y), cos(3.14 * v_texCoord.y) * cos(3.14 * v_texCoord.x)));\n' +
                 '}';

             default:
                 DE_ASSERT(false);
                 return '';
         }
     };

    glsSamplerObjectTest.TextureSamplerTest.prototype.init = function() {
        /** @const @type {string} */ var vertexShaderTemplate = glsSamplerObjectTest.TextureSamplerTest.selectVertexShader(this.m_target);
        /** @const @type {string} */ var fragmentShaderTemplate = glsSamplerObjectTest.TextureSamplerTest.selectFragmentShader(this.m_target);

        DE_ASSERT(!this.m_program);
        this.m_program = new gluShaderProgram.ShaderProgram(gl,
                                                       gluShaderProgram.makeVtxFragSources(
                                                           vertexShaderTemplate,
                                                           fragmentShaderTemplate));

        if (!this.m_program.isOk()) {
            // tcu::TestLog& log = m_testCtx.getLog();
            // log << *m_program;
            throw new Error('Failed to compile shaders');
        }
    };

    glsSamplerObjectTest.TextureSamplerTest.prototype.iterate = function() {
        //tcu::TestLog& log = m_testCtx.getLog();

        /** @type {tcuSurface.Surface} */ var textureRef = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);
        /** @type {tcuSurface.Surface} */ var samplerRef = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        /** @type {tcuSurface.Surface} */ var textureResult = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);
        /** @type {tcuSurface.Surface} */ var samplerResult = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        /** @type {number} */ var x = this.m_random.getInt(0, gl.drawingBufferWidth - glsSamplerObjectTest.VIEWPORT_WIDTH);
        /** @type {number} */ var y = this.m_random.getInt(0, gl.drawingBufferHeight - glsSamplerObjectTest.VIEWPORT_HEIGHT);

        this.renderReferences(textureRef, samplerRef, x, y);
        this.renderResults(textureResult, samplerResult, x, y);

        /** @type {boolean} */ var isOk = tcuImageCompare.pixelThresholdCompare('Sampler render result', 'Result from rendering with sampler', samplerRef, samplerResult, [0, 0, 0, 0]);

        if (!tcuImageCompare.pixelThresholdCompare('Texture render result', 'Result from rendering with texture state', textureRef, textureResult, [0, 0, 0, 0]))
            isOk = false;

        assertMsgOptions(isOk, '', true, false);

        return tcuTestCase.IterateResult.STOP;
    };

    // glsSamplerObjectTest.MultiTextureSamplerTest

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @const @param {glsSamplerObjectTest.TestSpec} spec
     */
    glsSamplerObjectTest.MultiTextureSamplerTest = function(spec) {
        tcuTestCase.DeqpTest.call(this, spec.name, spec.desc);
        /** @type {gluShaderProgram.ShaderProgram} */ this.m_program = null;
        /** @type {number} */ this.m_target = spec.target;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.m_textureState1 = spec.textureState;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.m_textureState2 = spec.textureState2;
        /** @type {glsSamplerObjectTest.SamplingState} */ this.m_samplerState = spec.samplerState;
        /** @type {deRandom.Random} */ this.m_random = new deRandom.Random(deString.deStringHash(spec.name));
    };

    glsSamplerObjectTest.MultiTextureSamplerTest.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsSamplerObjectTest.MultiTextureSamplerTest.prototype.constructor = glsSamplerObjectTest.MultiTextureSamplerTest;

    glsSamplerObjectTest.MultiTextureSamplerTest.prototype.init = function() {
        /** @type {string} */ var vertexShaderTemplate = glsSamplerObjectTest.MultiTextureSamplerTest.selectVertexShader(this.m_target);
        /** @type {string} */ var fragmentShaderTemplate = glsSamplerObjectTest.MultiTextureSamplerTest.selectFragmentShader(this.m_target);

        DE_ASSERT(!this.m_program);
        this.m_program = new gluShaderProgram.ShaderProgram(gl,
                                                            gluShaderProgram.makeVtxFragSources(
                                                                vertexShaderTemplate,
                                                                fragmentShaderTemplate));
        if (!this.m_program.isOk()) {
            // tcu::TestLog& log = m_testCtx.getLog();
            //
            // log << *m_program;
            throw new Error('Failed to compile shaders');
        }
    };

    glsSamplerObjectTest.MultiTextureSamplerTest.prototype.iterate = function() {
        //tcu::TestLog& log = m_testCtx.getLog();

        /** @type {tcuSurface.Surface} */ var textureRef = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);
        /** @type {tcuSurface.Surface} */ var samplerRef = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        /** @type {tcuSurface.Surface} */ var textureResult = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);
        /** @type {tcuSurface.Surface} */ var samplerResult = new tcuSurface.Surface(glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        /** @type {number} */ var x = this.m_random.getInt(0, gl.drawingBufferWidth - glsSamplerObjectTest.VIEWPORT_WIDTH);
        /** @type {number} */ var y = this.m_random.getInt(0, gl.drawingBufferHeight - glsSamplerObjectTest.VIEWPORT_HEIGHT);

        this.renderReferences(textureRef, samplerRef, x, y);
        this.renderResults(textureResult, samplerResult, x, y);

        /** @type {boolean} */ var isOk = tcuImageCompare.pixelThresholdCompare('Sampler render result', 'Result from rendering with sampler', samplerRef, samplerResult, [0, 0, 0, 0]);

        if (!tcuImageCompare.pixelThresholdCompare('Texture render result', 'Result from rendering with texture state', textureRef, textureResult, [0, 0, 0, 0]))
            isOk = false;

        assertMsgOptions(isOk, '', true, false);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @private
     * @param {tcuSurface.Surface} textureRef
     * @param {tcuSurface.Surface} samplerRef
     * @param {number} x
     * @param {number} y
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.prototype.renderReferences = function(textureRef, samplerRef, x, y) {
        /** @type {WebGLTexture} */ var texture1 = glsSamplerObjectTest.MultiTextureSamplerTest.createTexture(this.m_target, 0);
        /** @type {WebGLTexture} */ var texture2 = glsSamplerObjectTest.MultiTextureSamplerTest.createTexture(this.m_target, 1);

        gl.viewport(x, y, glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        // Generate texture rendering reference
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(this.m_target, texture1);
        glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState(this.m_target, this.m_textureState1);

        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(this.m_target, texture2);
        glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState(this.m_target, this.m_textureState2);

        this.render();
        var texRef = textureRef.getAccess();
        var texRefTransferFormat = gluTextureUtil.getTransferFormat(texRef.getFormat());
        gl.readPixels(x, y, texRef.m_width, texRef.m_height, texRefTransferFormat.format, texRefTransferFormat.dataType, textureRef.m_pixels);

        // Generate sampler rendering reference
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(this.m_target, texture1);
        glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState(this.m_target, this.m_samplerState);

        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(this.m_target, texture2);
        glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState(this.m_target, this.m_samplerState);

        this.render();
        var sampRef = samplerRef.getAccess();
        var sampRefTransferFormat = gluTextureUtil.getTransferFormat(sampRef.getFormat());
        gl.readPixels(x, y, sampRef.m_width, sampRef.m_height, sampRefTransferFormat.format, sampRefTransferFormat.dataType, samplerRef.m_pixels);
    };

    /**
     * @private
     * @param {tcuSurface.Surface} textureResult
     * @param {tcuSurface.Surface} samplerResult
     * @param {number} x
     * @param {number} y
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.prototype.renderResults = function(textureResult, samplerResult, x, y) {
        /** @type {WebGLTexture} */ var texture1 = glsSamplerObjectTest.MultiTextureSamplerTest.createTexture(this.m_target, 0);
        /** @type {WebGLTexture} */ var texture2 = glsSamplerObjectTest.MultiTextureSamplerTest.createTexture(this.m_target, 1);

        gl.viewport(x, y, glsSamplerObjectTest.VIEWPORT_WIDTH, glsSamplerObjectTest.VIEWPORT_HEIGHT);

        /** @type {WebGLSampler} */ var sampler = gl.createSampler();
        DE_ASSERT(sampler != -1);

        gl.bindSampler(0, sampler);
        gl.bindSampler(1, sampler);

        // First set sampler state
        glsSamplerObjectTest.MultiTextureSamplerTest.setSamplerState(this.m_samplerState, sampler);

        // Set texture state
        gl.bindTexture(this.m_target, texture1);
        glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState(this.m_target, this.m_textureState1);

        gl.bindTexture(this.m_target, texture2);
        glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState(this. m_target, this.m_textureState2);

        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(this.m_target, texture1);

        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(this.m_target, texture2);

        // Render using sampler
        this.render();
        var sampRes = samplerResult.getAccess();
        var sampResTransferFormat = gluTextureUtil.getTransferFormat(sampRes.getFormat());
        gl.readPixels(x, y, sampRes.m_width, sampRes.m_height, sampResTransferFormat.format, sampResTransferFormat.dataType, samplerResult.m_pixels);

        gl.bindSampler(0, null);
        gl.bindSampler(1, null);

        this.render();
        var texRes = textureResult.getAccess();
        var texResTransferFormat = gluTextureUtil.getTransferFormat(texRes.getFormat());
        gl.readPixels(x, y, texRes.m_width, texRes.m_height, texResTransferFormat.format, texResTransferFormat.dataType, textureResult.m_pixels);

        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(this.m_target, null);

        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(this.m_target, null);

        gl.deleteSampler(sampler);
        gl.deleteTexture(texture1);
        gl.deleteTexture(texture2);
    };

    glsSamplerObjectTest.MultiTextureSamplerTest.prototype.render = function() {

        gl.useProgram(this.m_program.getProgram());

        /** @type {WebGLUniformLocation} */ var samplerLoc1 = gl.getUniformLocation(this.m_program.getProgram(), 'u_sampler1');
        DE_ASSERT(samplerLoc1 != -1);

        /** @type {WebGLUniformLocation} */ var samplerLoc2 = gl.getUniformLocation(this.m_program.getProgram(), 'u_sampler2');
        DE_ASSERT(samplerLoc2 != -1);

        /** @type {WebGLUniformLocation} */ var scaleLoc = gl.getUniformLocation(this.m_program.getProgram(), 'u_posScale');
        DE_ASSERT(scaleLoc != -1);

        gl.clearColor(0.5, 0.5, 0.5, 1.0);

        gl.clear(gl.COLOR_BUFFER_BIT);

        gl.uniform1i(samplerLoc1, 0);

        gl.uniform1i(samplerLoc2, 1);

        gl.uniform1f(scaleLoc, 1.0);

        /** @type {Array<gluDrawUtil.VertexArrayBinding>} */ var vertexArrays;
        switch (this.m_target) {
            case gl.TEXTURE_2D: {
                vertexArrays = [
                    gluDrawUtil.vabFromBindingPointAndArrayPointer(
                        gluDrawUtil.bindingPointFromName('a_position'),
                        new gluDrawUtil.VertexArrayPointer(
                            gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
                            gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE,
                            2,
                            6,
                            0,
                            glsSamplerObjectTest.s_positions))
                ];

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                gl.uniform1f(scaleLoc, 0.25);

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                break;
            }

            case gl.TEXTURE_3D: {
                vertexArrays = [
                    gluDrawUtil.vabFromBindingPointAndArrayPointer(
                        gluDrawUtil.bindingPointFromName('a_position'),
                        new gluDrawUtil.VertexArrayPointer(
                            gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
                            gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE,
                            3,
                            6,
                            0,
                            glsSamplerObjectTest.s_positions3D))
                ];

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                gl.uniform1f(scaleLoc, 0.25);

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                break;
            }

            case gl.TEXTURE_CUBE_MAP: {
                vertexArrays = [
                    gluDrawUtil.vabFromBindingPointAndArrayPointer(
                        gluDrawUtil.bindingPointFromName('a_position'),
                        new gluDrawUtil.VertexArrayPointer(
                            gluDrawUtil.VertexComponentType.VTX_COMP_FLOAT,
                            gluDrawUtil.VertexComponentConversion.VTX_COMP_CONVERT_NONE,
                            4,
                            6,
                            0,
                            glsSamplerObjectTest.s_positionsCube))
                ];

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                gl.uniform1f(scaleLoc, 0.25);

                gluDrawUtil.draw(gl, this.m_program.getProgram(), vertexArrays, new gluDrawUtil.PrimitiveList(gluDrawUtil.primitiveType.TRIANGLES, 6));

                break;
            }

            default:
                DE_ASSERT(false);
        }

    };

    /**
     * @private
     * @param {number} target
     * @param {glsSamplerObjectTest.SamplingState} state
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.setTextureState = function(target, state) {
        gl.texParameteri(target, gl.TEXTURE_MIN_FILTER, state.minFilter);
        gl.texParameteri(target, gl.TEXTURE_MAG_FILTER, state.magFilter);
        gl.texParameteri(target, gl.TEXTURE_WRAP_S, state.wrapS);
        gl.texParameteri(target, gl.TEXTURE_WRAP_T, state.wrapT);
        gl.texParameteri(target, gl.TEXTURE_WRAP_R, state.wrapR);
        gl.texParameterf(target, gl.TEXTURE_MAX_LOD, state.maxLod);
        gl.texParameterf(target, gl.TEXTURE_MIN_LOD, state.minLod);
    };

    /**
     * @private
     * @param {glsSamplerObjectTest.SamplingState} state
     * @param {WebGLSampler} sampler
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.setSamplerState = function(state, sampler) {
        gl.samplerParameteri(sampler, gl.TEXTURE_MIN_FILTER, state.minFilter);
        gl.samplerParameteri(sampler, gl.TEXTURE_MAG_FILTER, state.magFilter);
        gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_S, state.wrapS);
        gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_T, state.wrapT);
        gl.samplerParameteri(sampler, gl.TEXTURE_WRAP_R, state.wrapR);
        gl.samplerParameterf(sampler, gl.TEXTURE_MAX_LOD, state.maxLod);
        gl.samplerParameterf(sampler, gl.TEXTURE_MIN_LOD, state.minLod);
    };

    /**
     * @private
     * @param {number} id
     * @return {WebGLTexture }
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.createTexture2D = function(id) {
        /** @type {WebGLTexture} */ var texture = null;
        /** @type {tcuTexture.Texture2D} */ var refTexture = new tcuTexture.Texture2D(
                                                                 new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA,
                                                                                              tcuTexture.ChannelType.UNORM_INT8),
                                                                 glsSamplerObjectTest.TEXTURE2D_WIDTH,
                                                                 glsSamplerObjectTest.TEXTURE2D_HEIGHT);

        refTexture.allocLevel(0);

        texture = gl.createTexture();

        switch (id) {
            case 0:
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevel(0), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 0.5, 0.5]);
                break;

            case 1:
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevel(0), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 1.0, 1.0]);
                break;

            default:
                DE_ASSERT(false);
        }

        gl.bindTexture(gl.TEXTURE_2D, texture);

        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, refTexture.getWidth(), refTexture.getHeight(), 0, gl.RGBA, gl.UNSIGNED_BYTE, refTexture.getLevel(0).getDataPtr());

        gl.generateMipmap(gl.TEXTURE_2D);

        gl.bindTexture(gl.TEXTURE_2D, null);

        return texture;
    };

    /**
     * @private
     * @param {number} id
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.createTexture3D = function(id) {
        /** @type {WebGLTexture} */ var texture = null;
        /** @type {tcuTexture.Texture3D} */ var refTexture = new tcuTexture.Texture3D(
                                                                 new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA,
                                                                                              tcuTexture.ChannelType.UNORM_INT8),
                                                                 glsSamplerObjectTest.TEXTURE3D_WIDTH,
                                                                 glsSamplerObjectTest.TEXTURE3D_HEIGHT,
                                                                 glsSamplerObjectTest.TEXTURE3D_DEPTH);

        refTexture.allocLevel(0);

        texture = gl.createTexture();

        switch (id) {
            case 0:
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevel(0), [0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 0.5, 0.5]);
                break;

            case 1:
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevel(0), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 1.0, 1.0]);
                break;

            default:
                DE_ASSERT(false);
        }

        gl.bindTexture(gl.TEXTURE_3D, texture);
        // TODO: check internalFormat and format in texImage3D
        gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA, refTexture.getWidth(), refTexture.getHeight(), refTexture.getDepth(), 0, gl.RGBA, gl.UNSIGNED_BYTE, refTexture.getLevel(0).getDataPtr());

        gl.generateMipmap(gl.TEXTURE_3D);

        gl.bindTexture(gl.TEXTURE_3D, null);

        return texture;
    };

    /**
     * @private
     * @param {number} id
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.createTextureCube = function(id) {
        /** @type {WebGLTexture} */ var texture = null;
        /** @type {tcuTexture.TextureCube} */ var refTexture = new tcuTexture.TextureCube(
                                                                     new tcuTexture.TextureFormat(tcuTexture.ChannelOrder.RGBA,
                                                                                                  tcuTexture.ChannelType.UNORM_INT8),
                                                                     glsSamplerObjectTest.CUBEMAP_SIZE);

        texture = gl.createTexture();

        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_POSITIVE_X, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y, 0);
        refTexture.allocLevel(tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z, 0);

        switch (id) {
            case 0:
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_X), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 0.5]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 0.5]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 0.5]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 0.5]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 0.5]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z), [0.0, 0.0, 0.0, 0.0], [0.5, 0.5, 0.5, 0.5]);
                break;

            case 1:
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_X), [0.5, 0.5, 0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_Y), [0.5, 0.5, 0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_POSITIVE_Z), [0.5, 0.5, 0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_X), [0.5, 0.5, 0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Y), [0.5, 0.5, 0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
                tcuTextureUtil.fillWithComponentGradients(refTexture.getLevelFace(0, tcuTexture.CubeFace.CUBEFACE_NEGATIVE_Z), [0.5, 0.5, 0.5, 0.5], [1.0, 1.0, 1.0, 1.0]);
                break;

            default:
                DE_ASSERT(false);
        }

        gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture);

        for (var face in tcuTexture.CubeFace) {
            /** @const @type {number} */ var target = gluTextureUtil.getGLCubeFace(tcuTexture.CubeFace[face]);
            gl.texImage2D(target, 0, gl.RGBA, refTexture.getSize(), refTexture.getSize(), 0, gl.RGBA, gl.UNSIGNED_BYTE, refTexture.getLevelFace(0, tcuTexture.CubeFace[face]).getDataPtr());
        }

        gl.generateMipmap(gl.TEXTURE_CUBE_MAP);
        gl.bindTexture(gl.TEXTURE_CUBE_MAP, null);

        return texture;
    };

    /**
     * @private
     * @param {number} target
     * @param {number} id
     * @return {WebGLTexture}
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.createTexture = function(target, id) {
        /** @type {WebGLTexture} */ var texture;
        switch (target) {
            case gl.TEXTURE_2D:
                texture = glsSamplerObjectTest.MultiTextureSamplerTest.createTexture2D(id);
                break;

            case gl.TEXTURE_3D:
                texture = glsSamplerObjectTest.MultiTextureSamplerTest.createTexture3D(id);
                break;

            case gl.TEXTURE_CUBE_MAP:
                texture = glsSamplerObjectTest.MultiTextureSamplerTest.createTextureCube(id);
                break;

            default:
                DE_ASSERT(false);
        }

        return texture;
    };

    /**
     * @private
     * @param {number} target
     * @return {string}
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.selectVertexShader = function(target) {
        switch (target) {
                case gl.TEXTURE_2D:
                    return '#version 300 es\n' +
                    'in highp vec2 a_position;\n' +
                    'uniform highp float u_posScale;\n' +
                    'out mediump vec2 v_texCoord;\n' +
                    'void main (void)\n' +
                    ' {\n' +
                    '\tv_texCoord = a_position;\n' +
                    '\tgl_Position = vec4(u_posScale * a_position, 0.0, 1.0);\n' +
                    '}';

                case gl.TEXTURE_3D:
                    return '#version 300 es\n' +
                    'in highp vec3 a_position;\n' +
                    'uniform highp float u_posScale;\n' +
                    'out mediump vec3 v_texCoord;\n' +
                    'void main (void)\n' +
                    ' {\n' +
                    '\tv_texCoord = a_position;\n' +
                    '\tgl_Position = vec4(u_posScale * a_position.xy, 0.0, 1.0);\n' +
                    '}';

                case gl.TEXTURE_CUBE_MAP:
                    return '#version 300 es\n' +
                    'in highp vec4 a_position;\n' +
                    'uniform highp float u_posScale;\n' +
                    'out mediump vec2 v_texCoord;\n' +
                    'void main (void)\n' +
                    ' {\n' +
                    '\tv_texCoord = a_position.zw;\n' +
                    '\tgl_Position = vec4(u_posScale * a_position.xy, 0.0, 1.0);\n' +
                    '}';

                default:
                    DE_ASSERT(false);
                    return '';
            }
    };

    /**
     * @private
     * @param {number} target
     * @return {string}
     */
    glsSamplerObjectTest.MultiTextureSamplerTest.selectFragmentShader = function(target) {
        switch (target) {
                case gl.TEXTURE_2D:
                    return '#version 300 es\nlayout(location = 0) out mediump vec4 o_color;\n' +
                    'uniform lowp sampler2D u_sampler1;\n' +
                    'uniform lowp sampler2D u_sampler2;\n' +
                    'in mediump vec2 v_texCoord;\n' +
                    'void main (void)\n' +
                    ' {\n' +
                    '\to_color = vec4(0.75, 0.75, 0.75, 1.0) * (texture(u_sampler1, v_texCoord) + texture(u_sampler2, v_texCoord));\n' +
                    '}';

                    break;

                case gl.TEXTURE_3D:
                    return '#version 300 es\nlayout(location = 0) out mediump vec4 o_color;\n' +
                    'uniform lowp sampler3D u_sampler1;\n' +
                    'uniform lowp sampler3D u_sampler2;\n' +
                    'in mediump vec3 v_texCoord;\n' +
                    'void main (void)\n' +
                    ' {\n' +
                    '\to_color = vec4(0.75, 0.75, 0.75, 1.0) * (texture(u_sampler1, v_texCoord) + texture(u_sampler2, v_texCoord));\n' +
                    '}';

                case gl.TEXTURE_CUBE_MAP:
                    return '#version 300 es\nlayout(location = 0) out mediump vec4 o_color;\n' +
                    'uniform lowp samplerCube u_sampler1;\n' +
                    'uniform lowp samplerCube u_sampler2;\n' +
                    'in mediump vec2 v_texCoord;\n' +
                    'void main (void)\n' +
                    ' {\n' +
                    '\to_color = vec4(0.5, 0.5, 0.5, 1.0) * (texture(u_sampler1, vec3(cos(3.14 * v_texCoord.y) * sin(3.14 * v_texCoord.x), sin(3.14 * v_texCoord.y), cos(3.14 * v_texCoord.y) * cos(3.14 * v_texCoord.x)))' +
                    '+ texture(u_sampler2, vec3(cos(3.14 * v_texCoord.y) * sin(3.14 * v_texCoord.x), sin(3.14 * v_texCoord.y), cos(3.14 * v_texCoord.y) * cos(3.14 * v_texCoord.x))));\n' +
                    '}';

                default:
                    DE_ASSERT(false);
                    return '';
            }
    };

});
