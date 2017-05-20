/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * WebGL IDL definitions from the Khronos specification:
 * https://www.khronos.org/registry/webgl/extensions/OES_vertex_array_object/
 */

[NoInterfaceObject]
interface OESVertexArrayObject {
    const unsigned long VERTEX_ARRAY_BINDING_OES = 0x85B5;

    WebGLVertexArrayObjectOES? createVertexArrayOES();
    void deleteVertexArrayOES(WebGLVertexArrayObjectOES? arrayObject);
    boolean isVertexArrayOES(WebGLVertexArrayObjectOES? arrayObject);
    void bindVertexArrayOES(WebGLVertexArrayObjectOES? arrayObject);
};
