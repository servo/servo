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
goog.provide('framework.opengl.simplereference.sglrReferenceContextTest');
goog.require('framework.common.tcuLogImage');
goog.require('framework.common.tcuPixelFormat');
goog.require('framework.common.tcuRGBA');
goog.require('framework.common.tcuSurface');
goog.require('framework.common.tcuTestCase');
goog.require('framework.opengl.gluDrawUtil');
goog.require('framework.opengl.simplereference.sglrReferenceContext');
goog.require('framework.opengl.simplereference.sglrShaderProgram');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');
goog.require('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrVertexAttrib');
goog.require('framework.referencerenderer.rrVertexPacket');

goog.scope(function() {
    var sglrReferenceContextTest = framework.opengl.simplereference.sglrReferenceContextTest;
    var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;
    var tcuTestCase = framework.common.tcuTestCase;
    var tcuPixelFormat = framework.common.tcuPixelFormat;
    var gluDrawUtil = framework.opengl.gluDrawUtil;
    var tcuSurface = framework.common.tcuSurface;
    var tcuLogImage = framework.common.tcuLogImage;
    var sglrShaderProgram = framework.opengl.simplereference.sglrShaderProgram;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;
    var rrVertexAttrib = framework.referencerenderer.rrVertexAttrib;
    var rrShadingContext = framework.referencerenderer.rrShadingContext;
    var rrVertexPacket = framework.referencerenderer.rrVertexPacket;
    var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
    var tcuRGBA = framework.common.tcuRGBA;

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    sglrReferenceContextTest.ClearContext = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
    };

    sglrReferenceContextTest.ClearContext.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    sglrReferenceContextTest.ClearContext.prototype.constructor = sglrReferenceContextTest.ClearContext;

    sglrReferenceContextTest.ClearContext.prototype.init = function() {};

    sglrReferenceContextTest.ClearContext.prototype.iterate = function() {

        var width = 200;
        var height = 188;
        var samples = 1;
        var limits = new sglrReferenceContext.ReferenceContextLimits(gl);
        var format = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(format, 24, 8, width, height, samples);
        var ctx = new sglrReferenceContext.ReferenceContext(limits, buffers.getColorbuffer(), buffers.getDepthbuffer(), buffers.getStencilbuffer());
        ctx.clearColor(1, 0, 0, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        var pixels = new tcuSurface.Surface(width, height);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        var numFailedPixels = 0;
        var redPixel = new gluDrawUtil.Pixel([255, 0, 0, 255]);
        for (var x = 0; x < width; x++)
            for (var y = 0; y < height; y++) {
                var pixel = new gluDrawUtil.Pixel(pixels.getPixel(x, y));
                if (!pixel.equals(redPixel))
                    numFailedPixels += 1;
            }

        var access = pixels.getAccess();

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        ctx.scissor(width / 4, height / 4, width / 2, height / 2);
        ctx.enable(gl.SCISSOR_TEST);
        ctx.clearColor(0, 1, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        numFailedPixels = 0;
        var greenBluePixel = new gluDrawUtil.Pixel([0, 255, 255, 255]);
        for (var x = 0; x < width; x++)
            for (var y = 0; y < height; y++) {
                var pixel = new gluDrawUtil.Pixel(pixels.getPixel(x, y));
                if ((x >= width / 4 && x < width - width / 4) && (y >= height / 4 && y < height - height / 4)) {
                    if (!pixel.equals(greenBluePixel))
                        numFailedPixels += 1;
                } else
                    if (!pixel.equals(redPixel))
                        numFailedPixels += 1;
            }

        access = pixels.getAccess();

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    sglrReferenceContextTest.Framebuffer = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
    };

    sglrReferenceContextTest.Framebuffer.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    sglrReferenceContextTest.Framebuffer.prototype.constructor = sglrReferenceContextTest.Framebuffer;

    sglrReferenceContextTest.Framebuffer.prototype.init = function() {};

    sglrReferenceContextTest.Framebuffer.prototype.iterate = function() {
        var limits = new sglrReferenceContext.ReferenceContextLimits(gl);
        var format = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var width = 200;
        var height = 188;
        var samples = 1;
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(format, 24, 8, width, height, samples);
        var ctx = new sglrReferenceContext.ReferenceContext(limits, buffers.getColorbuffer(), buffers.getDepthbuffer(), buffers.getStencilbuffer());
        ctx.clearColor(0, 0, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        var fbo = ctx.createFramebuffer();
        var rbo = ctx.createRenderbuffer();
        ctx.bindFramebuffer(gl.FRAMEBUFFER, fbo);
        ctx.bindRenderbuffer(gl.RENDERBUFFER, rbo);
        ctx.renderbufferStorage(gl.RENDERBUFFER, gl.RGBA8, width, height);
        ctx.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.RENDERBUFFER, rbo);
        bufferedLogToConsole('Framebuffer status: ' + (ctx.checkFramebufferStatus(gl.FRAMEBUFFER) == gl.FRAMEBUFFER_COMPLETE));
        ctx.clearColor(1, 0, 0, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        var pixels = new tcuSurface.Surface(width, height);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());
        var numFailedPixels = 0;
        var redPixel = new gluDrawUtil.Pixel([255, 0, 0, 255]);
        for (var x = 0; x < width; x++)
            for (var y = 0; y < height; y++) {
                var pixel = new gluDrawUtil.Pixel(pixels.getPixel(x, y));
                if (!pixel.equals(redPixel))
                    numFailedPixels += 1;
            }
        var access = pixels.getAccess();
        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        ctx.scissor(width / 4, height / 4, width / 2, height / 2);
        ctx.enable(gl.SCISSOR_TEST);
        ctx.clearColor(0, 1, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        numFailedPixels = 0;
        var greenBluePixel = new gluDrawUtil.Pixel([0, 255, 255, 255]);
        for (var x = 0; x < width; x++)
            for (var y = 0; y < height; y++) {
                var pixel = new gluDrawUtil.Pixel(pixels.getPixel(x, y));
                if ((x >= width / 4 && x < width - width / 4) && (y >= height / 4 && y < height - height / 4)) {
                    if (!pixel.equals(greenBluePixel))
                        numFailedPixels += 1;
                } else
                    if (!pixel.equals(redPixel))
                        numFailedPixels += 1;
            }

        access = pixels.getAccess();

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        ctx.bindFramebuffer(gl.FRAMEBUFFER, null);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        var bluePixel = new gluDrawUtil.Pixel([0, 0, 255, 255]);
        for (var x = 0; x < width; x++)
            for (var y = 0; y < height; y++) {
                var pixel = new gluDrawUtil.Pixel(pixels.getPixel(x, y));
                if (!pixel.equals(bluePixel))
                    numFailedPixels += 1;
            }
        access = pixels.getAccess();
        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    sglrReferenceContextTest.Shader = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
    };

    sglrReferenceContextTest.Shader.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    sglrReferenceContextTest.Shader.prototype.constructor = sglrReferenceContextTest.Shader;

    sglrReferenceContextTest.Shader.prototype.init = function() {};

    sglrReferenceContextTest.Shader.prototype.iterate = function() {
        var limits = new sglrReferenceContext.ReferenceContextLimits(gl);
        var format = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var width = 200;
        var height = 188;
        var samples = 1;
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(format, 24, 8, width, height, samples);
        var ctx = new sglrReferenceContext.ReferenceContext(limits, buffers.getColorbuffer(), buffers.getDepthbuffer(), buffers.getStencilbuffer());
        ctx.clearColor(0, 0, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        var vertices = [
            -0.5, 0.5,
            0.5, 0.5,
            -0.5, -0.5,
            0.5, 0.5,
            0.5, -0.5,
            -0.5, -0.5
        ];

        var vertices32 = new Float32Array(vertices);

        var squareVerticesBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, vertices32, gl.STATIC_DRAW);

        var colors = [
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1
        ];

        var colors32 = new Float32Array(colors);

        var squareColorsBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, colors32, gl.STATIC_DRAW);

        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */ var progDecl = new sglrShaderProgram.ShaderProgramDeclaration();

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexPosition', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexColor', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexSource(new sglrShaderProgram.VertexSource(''));

        progDecl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushFragmentSource(new sglrShaderProgram.FragmentSource(''));

        /** @type {sglrReferenceContextTest.ContextShaderProgram} */ var program = new sglrReferenceContextTest.ContextShaderProgram(progDecl);

        //Create program
        ctx.createProgram(program);

        //Use program
        ctx.useProgram(program);

        var vertexPositionAttribute = ctx.getAttribLocation(program, 'aVertexPosition');
        var vertexColorAttribute = ctx.getAttribLocation(program, 'aVertexColor');
        ctx.enableVertexAttribArray(vertexPositionAttribute);
        ctx.enableVertexAttribArray(vertexColorAttribute);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.vertexAttribPointer(vertexPositionAttribute, 2, gl.FLOAT, false, 0, 0);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.vertexAttribPointer(vertexColorAttribute, 4, gl.FLOAT, false, 0, 0);

        ctx.drawQuads(gl.TRIANGLES, 0, 6);

        var pixels = new tcuSurface.Surface(width, height);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        var numFailedPixels = 0;

        var redPixel = new gluDrawUtil.Pixel([255, 0, 0, 255]);
        var bluePixel = new gluDrawUtil.Pixel([0, 0, 255, 255]);

        var pixel = new gluDrawUtil.Pixel(pixels.getPixel(0, 0));
        if (!pixel.equals(bluePixel))
            numFailedPixels += 1;

        pixel = new gluDrawUtil.Pixel(pixels.getPixel(100, 94));
        if (!pixel.equals(redPixel))
            numFailedPixels += 1;

        var access = pixels.getAccess();

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    sglrReferenceContextTest.TriangleStrip = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
    };

    sglrReferenceContextTest.TriangleStrip.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    sglrReferenceContextTest.TriangleStrip.prototype.constructor = sglrReferenceContextTest.TriangleStrip;

    sglrReferenceContextTest.TriangleStrip.prototype.init = function() {};

    sglrReferenceContextTest.TriangleStrip.prototype.iterate = function() {
        var limits = new sglrReferenceContext.ReferenceContextLimits(gl);
        var format = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var width = 200;
        var height = 188;
        var samples = 1;
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(format, 24, 8, width, height, samples);
        var ctx = new sglrReferenceContext.ReferenceContext(limits, buffers.getColorbuffer(), buffers.getDepthbuffer(), buffers.getStencilbuffer());
        ctx.clearColor(0, 0, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        var vertices = [
            -0.5, 0.5,
            0.5, 0.5,
            -0.5, 0,
            0.5, 0,
            -0.5, -0.5,
            0.5, -0.5
        ];

        var vertices32 = new Float32Array(vertices);

        var squareVerticesBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, vertices32, gl.STATIC_DRAW);

        var colors = [
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1
        ];

        var colors32 = new Float32Array(colors);

        var squareColorsBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, colors32, gl.STATIC_DRAW);

        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */ var progDecl = new sglrShaderProgram.ShaderProgramDeclaration();

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexPosition', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexColor', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexSource(new sglrShaderProgram.VertexSource(''));

        progDecl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushFragmentSource(new sglrShaderProgram.FragmentSource(''));

        /** @type {sglrReferenceContextTest.ContextShaderProgram} */ var program = new sglrReferenceContextTest.ContextShaderProgram(progDecl);

        //Create program
        ctx.createProgram(program);

        //Use program
        ctx.useProgram(program);

        var vertexPositionAttribute = ctx.getAttribLocation(program, 'aVertexPosition');
        var vertexColorAttribute = ctx.getAttribLocation(program, 'aVertexColor');
        ctx.enableVertexAttribArray(vertexPositionAttribute);
        ctx.enableVertexAttribArray(vertexColorAttribute);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.vertexAttribPointer(vertexPositionAttribute, 2, gl.FLOAT, false, 0, 0);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.vertexAttribPointer(vertexColorAttribute, 4, gl.FLOAT, false, 0, 0);

        ctx.drawQuads(gl.TRIANGLE_STRIP, 0, 6);

        var pixels = new tcuSurface.Surface(width, height);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        var numFailedPixels = 0;

        var redPixel = new gluDrawUtil.Pixel([255, 0, 0, 255]);
        var bluePixel = new gluDrawUtil.Pixel([0, 0, 255, 255]);

        var pixel = new gluDrawUtil.Pixel(pixels.getPixel(0, 0));
        if (!pixel.equals(bluePixel))
            numFailedPixels += 1;

        pixel = new gluDrawUtil.Pixel(pixels.getPixel(100, 94));
        if (!pixel.equals(redPixel))
            numFailedPixels += 1;

        var access = pixels.getAccess();

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    sglrReferenceContextTest.TriangleFan = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
    };

    sglrReferenceContextTest.TriangleFan.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    sglrReferenceContextTest.TriangleFan.prototype.constructor = sglrReferenceContextTest.TriangleFan;

    sglrReferenceContextTest.TriangleFan.prototype.init = function() {};

    sglrReferenceContextTest.TriangleFan.prototype.iterate = function() {
        var limits = new sglrReferenceContext.ReferenceContextLimits(gl);
        var format = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var width = 200;
        var height = 188;
        var samples = 1;
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(format, 24, 8, width, height, samples);
        var ctx = new sglrReferenceContext.ReferenceContext(limits, buffers.getColorbuffer(), buffers.getDepthbuffer(), buffers.getStencilbuffer());
        ctx.clearColor(0, 0, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        var vertices = [
            -0.5, 0,
            -0.5, 0.5,
            0.5, 0.5,
            0.5, 0,
            0.5, -0.5,
            -0.5, -0.5
        ];

        var vertices32 = new Float32Array(vertices);

        var squareVerticesBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, vertices32, gl.STATIC_DRAW);

        var colors = [
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1,
            1, 0, 0, 1
        ];

        var colors32 = new Float32Array(colors);

        var squareColorsBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, colors32, gl.STATIC_DRAW);

        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */ var progDecl = new sglrShaderProgram.ShaderProgramDeclaration();

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexPosition', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexColor', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexSource(new sglrShaderProgram.VertexSource(''));

        progDecl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushFragmentSource(new sglrShaderProgram.FragmentSource(''));

        /** @type {sglrReferenceContextTest.ContextShaderProgram} */ var program = new sglrReferenceContextTest.ContextShaderProgram(progDecl);

        //Create program
        ctx.createProgram(program);

        //Use program
        ctx.useProgram(program);

        var vertexPositionAttribute = ctx.getAttribLocation(program, 'aVertexPosition');
        var vertexColorAttribute = ctx.getAttribLocation(program, 'aVertexColor');
        ctx.enableVertexAttribArray(vertexPositionAttribute);
        ctx.enableVertexAttribArray(vertexColorAttribute);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.vertexAttribPointer(vertexPositionAttribute, 2, gl.FLOAT, false, 0, 0);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.vertexAttribPointer(vertexColorAttribute, 4, gl.FLOAT, false, 0, 0);

        ctx.drawQuads(gl.TRIANGLE_FAN, 0, 6);

        var pixels = new tcuSurface.Surface(width, height);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        var numFailedPixels = 0;

        var redPixel = new gluDrawUtil.Pixel([255, 0, 0, 255]);
        var bluePixel = new gluDrawUtil.Pixel([0, 0, 255, 255]);

        var pixel = new gluDrawUtil.Pixel(pixels.getPixel(0, 0));
        if (!pixel.equals(bluePixel))
            numFailedPixels += 1;

        pixel = new gluDrawUtil.Pixel(pixels.getPixel(100, 94));
        if (!pixel.equals(redPixel))
            numFailedPixels += 1;

        var access = pixels.getAccess();

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {string} name
     * @param {string} description
     */
    sglrReferenceContextTest.DrawElements = function(name, description) {
        tcuTestCase.DeqpTest.call(this, name, description);
    };

    sglrReferenceContextTest.DrawElements.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    sglrReferenceContextTest.DrawElements.prototype.constructor = sglrReferenceContextTest.DrawElements;

    sglrReferenceContextTest.DrawElements.prototype.init = function() {};

    sglrReferenceContextTest.DrawElements.prototype.iterate = function() {
        var limits = new sglrReferenceContext.ReferenceContextLimits(gl);
        var format = new tcuPixelFormat.PixelFormat(8, 8, 8, 8);
        var width = 200;
        var height = 188;
        var samples = 1;
        var buffers = new sglrReferenceContext.ReferenceContextBuffers(format, 24, 8, width, height, samples);
        var ctx = new sglrReferenceContext.ReferenceContext(limits, buffers.getColorbuffer(), buffers.getDepthbuffer(), buffers.getStencilbuffer());
        ctx.clearColor(0, 0, 1, 1);
        ctx.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT | gl.STENCIL_BUFFER_BIT);

        var vertices = [
            -0.5, 0.5,
            0, 0.5,
            0.4, 0.5,

            -0.5, 0.1,
            0, 0.1,
            0.4, 0.1,

            -0.5, -0.7,
            0, -0.7,
            0.4, -0.7
        ];

        var vertices32 = new Float32Array(vertices);

        var squareVerticesBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, vertices32, gl.STATIC_DRAW);

        var indices = [
            0, 1, 3, 1, 3, 4,
            1, 2, 4, 2, 4, 5,
            3, 4, 6, 4, 6, 7,
            4, 5, 7, 5, 7, 8
        ];
        var indicesBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indicesBuffer);
        ctx.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);

        var colors = [
            1, 0, 0, 1,
            0, 1, 0, 1,
            0, 0, 1, 1,
            1, 1, 1, 1,
            1, 1, 0, 1,
            0, 1, 1, 1,
            1, 0, 1, 1,
            0.5, 0.5, 0.5, 1,
            0, 0, 0, 0
        ];

        var colors32 = new Float32Array(colors);

        var squareColorsBuffer = ctx.createBuffer();
        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.bufferData(gl.ARRAY_BUFFER, colors32, gl.STATIC_DRAW);

        /** @type {sglrShaderProgram.ShaderProgramDeclaration} */ var progDecl = new sglrShaderProgram.ShaderProgramDeclaration();

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexPosition', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexAttribute(new sglrShaderProgram.VertexAttribute('aVertexColor', rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushVertexSource(new sglrShaderProgram.VertexSource(''));

        progDecl.pushFragmentOutput(new sglrShaderProgram.FragmentOutput(rrGenericVector.GenericVecType.FLOAT));

        progDecl.pushFragmentSource(new sglrShaderProgram.FragmentSource(''));

        /** @type {sglrReferenceContextTest.ContextShaderProgram} */ var program = new sglrReferenceContextTest.ContextShaderProgram(progDecl);

        //Create program
        ctx.createProgram(program);

        //Use program
        ctx.useProgram(program);

        var vertexPositionAttribute = ctx.getAttribLocation(program, 'aVertexPosition');
        var vertexColorAttribute = ctx.getAttribLocation(program, 'aVertexColor');
        ctx.enableVertexAttribArray(vertexPositionAttribute);
        ctx.enableVertexAttribArray(vertexColorAttribute);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareVerticesBuffer);
        ctx.vertexAttribPointer(vertexPositionAttribute, 2, gl.FLOAT, false, 0, 0);

        ctx.bindBuffer(gl.ARRAY_BUFFER, squareColorsBuffer);
        ctx.vertexAttribPointer(vertexColorAttribute, 4, gl.FLOAT, false, 0, 0);

        ctx.drawElements(gl.TRIANGLES, indices.length, gl.UNSIGNED_SHORT, 0);

        var pixels = new tcuSurface.Surface(width, height);
        ctx.readPixels(0, 0, width, height, gl.RGBA, gl.UNSIGNED_BYTE, pixels.getAccess().getDataPtr());

        var numFailedPixels = 0;

        var access = pixels.getAccess();

        var pixelsTotest = [
            // location, color
            [2, 1], [0, 0, 255, 255],
            // The red vertex is between 140 and 141 so account for some blending with the white vertex
            [50, 140], [255, 5, 5, 255],
            [50, 28], [255, 0, 255, 255],
            [139, 28], [0, 0, 0, 255],
            [50, 102], [255, 255, 255, 255],
            [139, 102], [0, 255, 255, 255]
        ];

        var threshold = new tcuRGBA.RGBA([5, 5, 5, 5]);

        for (var i = 0; i < pixelsTotest.length; i += 2) {
            var location = pixelsTotest[i];
            var reference = new tcuRGBA.RGBA(pixelsTotest[i + 1]);
            var color = access.getPixelInt(location[0], location[1]);
            var pixel = new tcuRGBA.RGBA(color);
            if (!tcuRGBA.compareThreshold(pixel, reference, threshold))
                numFailedPixels++;
        }

        tcuLogImage.logImage('Result', '', access);

        if (numFailedPixels > 0)
            testFailedOptions('Image comparison failed, got ' + numFailedPixels + ' non-equal pixels.', false);
        else
            testPassedOptions('Image comparison succeed', true);

        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {sglrShaderProgram.ShaderProgram}
     * @param {sglrShaderProgram.ShaderProgramDeclaration} progDecl
     */
    sglrReferenceContextTest.ContextShaderProgram = function(progDecl) {
        sglrShaderProgram.ShaderProgram.call(this, progDecl);
    };

    sglrReferenceContextTest.ContextShaderProgram.prototype = Object.create(sglrShaderProgram.ShaderProgram.prototype);
    sglrReferenceContextTest.ContextShaderProgram.prototype.constructor = sglrReferenceContextTest.ContextShaderProgram;

    /**
     * @param {Array<rrVertexAttrib.VertexAttrib>} inputs
     * @param {Array<rrVertexPacket.VertexPacket>} packets
     * @param {number} numPackets
     */
    sglrReferenceContextTest.ContextShaderProgram.prototype.shadeVertices = function(inputs, packets, numPackets) {
        for (var packetNdx = 0; packetNdx < numPackets; ++packetNdx) {
            /** @type {number} */ var varyingLocColor = 0;

            /** @type {rrVertexPacket.VertexPacket} */ var packet = packets[packetNdx];

            // Calc output color
            /** @type {Array<number>} */ var coord = [1.0, 1.0];
            /** @type {Array<number>} */ var color = [1.0, 1.0, 1.0];

            for (var attribNdx = 0; attribNdx < this.getVertexShader().getInputs().length; attribNdx++) {
                /** @type {number} */ var numComponents = inputs[attribNdx].componentCount;

                var attribValue = rrVertexAttrib.readVertexAttrib(inputs[attribNdx], packet.instanceNdx, packet.vertexNdx, this.getVertexShader().getInputs()[attribNdx].type);

                if (attribNdx == 0) {
                    coord[0] = attribValue[0];
                    coord[1] = attribValue[1];
                } else {
                    color[0] = attribValue[0] * attribValue[3];
                    color[1] = attribValue[1] * attribValue[3];
                    color[2] = attribValue[2] * attribValue[3];
                }
            }

            // Transform position
            packet.position = [coord[0], coord[1], 1.0, 1.0];

            // Pass color to FS
            packet.outputs[varyingLocColor] = [color[0], color[1], color[2], 1.0];
        }
    };

    /**
     * @param {Array<rrFragmentOperations.Fragment>} packets
     * @param {rrShadingContext.FragmentShadingContext} context
     */
    sglrReferenceContextTest.ContextShaderProgram.prototype.shadeFragments = function(packets, context) {
        var varyingLocColor = 0;

        // Normal shading
        for (var packetNdx = 0; packetNdx < packets.length; ++packetNdx)
            packets[packetNdx].value = rrShadingContext.readTriangleVarying(packets[packetNdx], context, varyingLocColor);
    };

    sglrReferenceContextTest.init = function() {
        var state = tcuTestCase.runner;
        /** @type {tcuTestCase.DeqpTest} */ var testGroup = state.testCases;

        /** @type {tcuTestCase.DeqpTest} */ var referenceContextGroup = tcuTestCase.newTest('reference_context', 'Test reference context');

        referenceContextGroup.addChild(new sglrReferenceContextTest.ClearContext('clear_context', 'Clear Context Test'));
        referenceContextGroup.addChild(new sglrReferenceContextTest.Framebuffer('Framebuffer', 'Framebuffer Test'));
        referenceContextGroup.addChild(new sglrReferenceContextTest.Shader('Shaders', 'Drawing using TRIANGLES'));
        referenceContextGroup.addChild(new sglrReferenceContextTest.TriangleStrip('TriangleStrip', 'Drawing using TRIANGLE_STRIP'));
        referenceContextGroup.addChild(new sglrReferenceContextTest.TriangleFan('TriangleFan', 'Drawing using TRIANGLE_FAN'));
        referenceContextGroup.addChild(new sglrReferenceContextTest.DrawElements('DrawElements', 'Drawing using DrawElements and TRIANGLES'));

        testGroup.addChild(referenceContextGroup);

    };

    sglrReferenceContextTest.run = function(context) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'single_reference_context';
        var testDescription = 'Single Reference Context Tests';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            sglrReferenceContextTest.init();
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }

    };

});
