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
goog.provide('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuTexture');
goog.require('framework.common.tcuTextureUtil');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluTextureUtil');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrMultisamplePixelBufferAccess');
goog.require('framework.referencerenderer.rrRenderState');
goog.require('framework.referencerenderer.rrRenderer');
goog.require('framework.referencerenderer.rrVertexAttrib');

goog.scope(function() {

    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var tcuTexture = framework.common.tcuTexture;
    var deUtil = framework.delibs.debase.deUtil;
    var deMath = framework.delibs.debase.deMath;
    var tcuTextureUtil = framework.common.tcuTextureUtil;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var gluTextureUtil = framework.opengl.gluTextureUtil;
    var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
    var rrDefs = framework.referencerenderer.rrDefs;
    var rrMultisamplePixelBufferAccess = framework.referencerenderer.rrMultisamplePixelBufferAccess;
    var rrRenderer = framework.referencerenderer.rrRenderer;
    var rrRenderState = framework.referencerenderer.rrRenderState;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * sglrGLContext.GLContext wraps the standard WebGL context to be able to be used interchangeably with the ReferenceContext
     * @constructor
     * @extends {WebGL2RenderingContext}
     * @param {?WebGL2RenderingContext} context
     * @param {Array<number>=} viewport
     */
    sglrGLContext.GLContext = function(context, viewport) {
        DE_ASSERT(context);

        var functionwrapper = function(context, fname) {
            return function() {
                return context[fname].apply(context, arguments);
            };
        };

        var wrap = {};
        for (var i in context) {
            try {
              if (typeof context[i] == 'function') {
                wrap[i] = functionwrapper(context, i);
              } else {
                wrap[i] = context[i];
              }
            } catch (e) {
              throw new Error('GLContext: Error accessing ' + i);
            }
        }
        if (viewport)
            context.viewport(viewport[0], viewport[1], viewport[2], viewport[3]);

        /**
         * createProgram
         * @override
         * @param {sglrShaderProgram.ShaderProgram=} shader
         * @return {!WebGLProgram}
         */
        this.createProgram = function(shader) {
            var program = new gluShaderProgram.ShaderProgram(
                    context,
                    gluShaderProgram.makeVtxFragSources(
                        shader.m_vertSrc,
                        shader.m_fragSrc
                    )
                );

            if (!program.isOk()) {
                bufferedLogToConsole(program.toString());
                testFailedOptions('Compile failed', true);
            }
            return program.getProgram();
        };
        wrap['createProgram'] = this.createProgram;

        /**
         * Draws quads from vertex arrays
         * @param {number} primitive Primitive type
         * @param {number} first First vertex to begin drawing with
         * @param {number} count Number of vertices
         */
        var drawQuads = function(primitive, first, count) {
            context.drawArrays(primitive, first, count);
        };
        wrap['drawQuads'] = drawQuads;

        /**
         * @return {number}
         */
        var getWidth = function() {
            if(viewport)
                return viewport[2];
            else
                return context.drawingBufferWidth;
        };
        wrap['getWidth'] = getWidth;

        /**
         * @return {number}
         */
        var getHeight = function() {
            if(viewport)
                return viewport[3];
            else
                return context.drawingBufferHeight;
        };
        wrap['getHeight'] = getHeight;

        /**
         * @param {number} x
         * @param {number} y
         * @param {number} width
         * @param {number} height
         * @param {number} format
         * @param {number} dataType
         * @param {ArrayBuffer|ArrayBufferView} data
         */
        var readPixels = function(x, y, width, height, format, dataType, data) {
            /** @type {?ArrayBufferView} */ var dataArr;
            if (!ArrayBuffer.isView(data)) {
                var type = gluTextureUtil.mapGLChannelType(dataType, true);
                var dataArrType = tcuTexture.getTypedArray(type);
                dataArr = new dataArrType(data);
            } else {
                dataArr = /** @type {?ArrayBufferView} */ (data);
            }

            context.readPixels(x, y, width, height, format, dataType, dataArr);
        };
        wrap['readPixels'] = readPixels;

        /**
         * @param {number} target
         * @param {number} level
         * @param {number} internalFormat
         * @param {number} width
         * @param {number} height
         */
        var texImage2DDelegate = function(target, level, internalFormat, width, height) {
            var format;
            var dataType;

            switch(internalFormat)
            {
                case gl.ALPHA:
                case gl.LUMINANCE:
                case gl.LUMINANCE_ALPHA:
                case gl.RGB:
                case gl.RGBA:
                    format = internalFormat;
                    dataType = gl.UNSIGNED_BYTE;
                    break;
                default:
                {
                    var transferFmt = gluTextureUtil.getTransferFormat(gluTextureUtil.mapGLInternalFormat(internalFormat));
                    format = transferFmt.format;
                    dataType = transferFmt.dataType;
                    break;
                }
             }
             context.texImage2D(target, level, internalFormat, width, height, 0, format, dataType, null);
         };
         wrap['texImage2DDelegate'] = texImage2DDelegate;

         return wrap;
    };

    /**
     * createProgram - This had to be added here as dummy to remove a warning when the only context used is GLContext (no reference context)
     * @override
     * @param {sglrShaderProgram.ShaderProgram=} shader
     * @return {!WebGLProgram}
     */
    sglrGLContext.GLContext.prototype.createProgram = function(shader) {return this.createProgram();};

    /**
    * @param ctx GL-like context
    * @param {string} name
    * @return {boolean}
    */
    sglrGLContext.isExtensionSupported = function(ctx, name) {
        var extns = ctx.getSupportedExtensions();
        var found = false;
        if (extns) {
            var index = extns.indexOf(name);
            if (index != -1)
                found = true;
        }
        return found;
    };

});
