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

goog.provide('framework.opengl.gluObjectWrapper');

goog.scope(function() {
    var gluObjectWrapper = framework.opengl.gluObjectWrapper;

    /**
    * @typedef {function(this:WebGLRenderingContextBase): WebGLObject}
    */
    gluObjectWrapper.funcGenT;

    /**
    * @typedef {function(this:WebGLRenderingContextBase, WebGLObject)}
    */
    gluObjectWrapper.funcDelT;

    /**
    * @typedef {{name: string, funcGen: !gluObjectWrapper.funcGenT, funcDel: !gluObjectWrapper.funcDelT}}
    */
    gluObjectWrapper.traitsT;

    /**
    * Returns an object containing a configuration for an ObjectWrapper
    * @param {string} name
    * @param {gluObjectWrapper.funcGenT} funcGen
    * @param {gluObjectWrapper.funcDelT} funcDel
    * @return {gluObjectWrapper.traitsT}
    */
    gluObjectWrapper.traits = function(name, funcGen, funcDel) {
        return {
            name: name,
            funcGen: funcGen,
            funcDel: funcDel
        };
    };

    /**
    * @constructor
    * @param {WebGLRenderingContextBase} gl
    * @param {gluObjectWrapper.traitsT} traits
    */
    gluObjectWrapper.ObjectWrapper = function(gl, traits) {
        /**
        * @protected
        * @type {WebGLRenderingContextBase}
        */
        this.m_gl = gl;

        /**
        * @protected
        * @type {gluObjectWrapper.traitsT}
        */
        this.m_traits = traits;

        /**
        * @protected
        * @type {WebGLObject}
        */
        this.m_object = this.m_traits.funcGen.call(gl);

    };

    /**
    * Destorys the WebGLObject associated with this object.
    */
    gluObjectWrapper.ObjectWrapper.prototype.clear = function() {
        this.m_traits.funcDel.call(this.m_gl, this.m_object);
    };

    /**
    * Returns the WebGLObject associated with this object.
    * @return {WebGLObject}
    */
    gluObjectWrapper.ObjectWrapper.prototype.get = function() {
        return this.m_object;
    };

    /**
    * @constructor
    * @extends {gluObjectWrapper.ObjectWrapper}
    * @param {WebGLRenderingContextBase} gl
    */
    gluObjectWrapper.Framebuffer = function(gl) {
        gluObjectWrapper.ObjectWrapper.call(this, gl, gluObjectWrapper.traits(
            'framebuffer',
            /** @type {gluObjectWrapper.funcGenT} */(gl.createFramebuffer),
            /** @type {gluObjectWrapper.funcDelT} */(gl.deleteFramebuffer)
        ));
    };
    gluObjectWrapper.Framebuffer.prototype = Object.create(gluObjectWrapper.ObjectWrapper.prototype);
    gluObjectWrapper.Framebuffer.prototype.constructor = gluObjectWrapper.Framebuffer;

    /**
    * @return {WebGLFramebuffer}
    */
    gluObjectWrapper.Framebuffer.prototype.get;
});
