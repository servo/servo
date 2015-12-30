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
goog.provide('framework.referencerenderer.rrUtil');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('framework.opengl.simplereference.sglrReferenceContext');

goog.scope(function() {

    var rrUtil = framework.referencerenderer.rrUtil;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var sglrReferenceContext = framework.opengl.simplereference.sglrReferenceContext;

    /**
     * @param {sglrGLContext.GLContext | WebGL2RenderingContext | sglrReferenceContext.ReferenceContext} ctx
     * @param {number|Object} program
     * @param {Array<number>} p0
     * @param {Array<number>} p1
     */
    rrUtil.drawQuad = function(ctx, program, p0, p1) {
        // Vertex data.
        var hz = (p0[2] + p1[2]) * 0.5;
        /** @type {Array<number>} */ var position = [
        p0[0], p0[1], p0[2], 1.0,
        p0[0], p1[1], hz, 1.0,
        p1[0], p0[1], hz, 1.0,
        p1[0], p0[1], hz, 1.0,
        p0[0], p1[1], hz, 1.0,
        p1[0], p1[1], p1[2], 1.0
        ];
        /** @type {Array<number>} */ var coord = [
        0.0, 0.0,
        0.0, 1.0,
        1.0, 0.0,
        1.0, 0.0,
        0.0, 1.0,
        1.0, 1.0
        ];

        var posLoc = ctx.getAttribLocation(program, 'a_position');
        if (posLoc == -1)
            throw new Error('a_position attribute is not defined.');

        var coordLoc = ctx.getAttribLocation(program, 'a_coord');
        var vaoID;
        var bufIDs = [];

        vaoID = ctx.createVertexArray();
        ctx.bindVertexArray(vaoID);

        bufIDs[0] = ctx.createBuffer();
        bufIDs[1] = ctx.createBuffer();

        ctx.useProgram(program);
        ctx.bindBuffer(gl.ARRAY_BUFFER, bufIDs[0]);
        ctx.bufferData(gl.ARRAY_BUFFER, new Float32Array(position), gl.STATIC_DRAW);

        ctx.enableVertexAttribArray(posLoc);
        ctx.vertexAttribPointer(posLoc, 4, gl.FLOAT, false, 0, 0);

        ctx.bindBuffer(gl.ARRAY_BUFFER, null);

        if (coordLoc >= 0) {
            ctx.bindBuffer(gl.ARRAY_BUFFER, bufIDs[1]);
            ctx.bufferData(gl.ARRAY_BUFFER, new Float32Array(coord), gl.STATIC_DRAW);

            ctx.enableVertexAttribArray(coordLoc);
            ctx.vertexAttribPointer(coordLoc, 2, gl.FLOAT, false, 0, 0);

            ctx.bindBuffer(gl.ARRAY_BUFFER, null);
        }

        ctx.drawQuads(gl.TRIANGLES, 0, 6);

        ctx.bindVertexArray(null);
        ctx.deleteBuffer(bufIDs[0]);
        ctx.deleteBuffer(bufIDs[1]);
        ctx.deleteVertexArray(vaoID);
    };

});
