/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * WebGL IDL definitions from the Khronos specification:
 * https://www.khronos.org/registry/webgl/extensions/ANGLE_instanced_arrays/
 */

[NoInterfaceObject]
interface ANGLEInstancedArrays {
    const GLenum VERTEX_ATTRIB_ARRAY_DIVISOR_ANGLE = 0x88FE;
    void drawArraysInstancedANGLE(GLenum mode, GLint first, GLsizei count, GLsizei primcount);
    void drawElementsInstancedANGLE(GLenum mode, GLsizei count, GLenum type, GLintptr offset, GLsizei primcount);
    void vertexAttribDivisorANGLE(GLuint index, GLuint divisor);
};
