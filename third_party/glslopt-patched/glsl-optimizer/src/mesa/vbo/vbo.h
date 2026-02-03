/*
 * mesa 3-D graphics library
 *
 * Copyright (C) 1999-2006  Brian Paul   All Rights Reserved.
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included
 * in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR
 * OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
 * ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
 * OTHER DEALINGS IN THE SOFTWARE.
 */

/**
 * \brief Public interface to the VBO module
 * \author Keith Whitwell
 */


#ifndef _VBO_H
#define _VBO_H

#include <stdbool.h>
#include "main/glheader.h"
#include "main/draw.h"

#ifdef __cplusplus
extern "C" {
#endif

struct gl_context;

GLboolean
_vbo_CreateContext(struct gl_context *ctx, bool use_buffer_objects);

void
_vbo_DestroyContext(struct gl_context *ctx);

void
vbo_exec_update_eval_maps(struct gl_context *ctx);

void
_vbo_install_exec_vtxfmt(struct gl_context *ctx);

void
vbo_initialize_exec_dispatch(const struct gl_context *ctx,
                             struct _glapi_table *exec);

void
vbo_initialize_save_dispatch(const struct gl_context *ctx,
                             struct _glapi_table *exec);

void
vbo_exec_FlushVertices(struct gl_context *ctx, GLuint flags);

void
vbo_save_SaveFlushVertices(struct gl_context *ctx);

void
vbo_save_NotifyBegin(struct gl_context *ctx, GLenum mode,
                     bool no_current_update);

void
vbo_save_NewList(struct gl_context *ctx, GLuint list, GLenum mode);

void
vbo_save_EndList(struct gl_context *ctx);

void
vbo_save_BeginCallList(struct gl_context *ctx, struct gl_display_list *list);

void
vbo_save_EndCallList(struct gl_context *ctx);


void
vbo_delete_minmax_cache(struct gl_buffer_object *bufferObj);

void
vbo_get_minmax_index_mapped(unsigned count, unsigned index_size,
                            unsigned restartIndex, bool restart,
                            const void *indices,
                            unsigned *min_index, unsigned *max_index);

void
vbo_get_minmax_indices(struct gl_context *ctx, const struct _mesa_prim *prim,
                       const struct _mesa_index_buffer *ib,
                       GLuint *min_index, GLuint *max_index, GLuint nr_prims);

void
vbo_sw_primitive_restart(struct gl_context *ctx,
                         const struct _mesa_prim *prim,
                         GLuint nr_prims,
                         const struct _mesa_index_buffer *ib,
                         GLuint num_instances, GLuint base_instance,
                         struct gl_buffer_object *indirect,
                         GLsizeiptr indirect_offset);


const struct gl_array_attributes*
_vbo_current_attrib(const struct gl_context *ctx, gl_vert_attrib attr);


const struct gl_vertex_buffer_binding*
_vbo_current_binding(const struct gl_context *ctx);


void GLAPIENTRY
_es_Color4f(GLfloat r, GLfloat g, GLfloat b, GLfloat a);

void GLAPIENTRY
_es_Normal3f(GLfloat x, GLfloat y, GLfloat z);

void GLAPIENTRY
_es_MultiTexCoord4f(GLenum target, GLfloat s, GLfloat t, GLfloat r, GLfloat q);

void GLAPIENTRY
_es_Materialfv(GLenum face, GLenum pname, const GLfloat *params);

void GLAPIENTRY
_es_Materialf(GLenum face, GLenum pname, GLfloat param);

void GLAPIENTRY
_es_VertexAttrib4f(GLuint index, GLfloat x, GLfloat y, GLfloat z, GLfloat w);

void GLAPIENTRY
_es_VertexAttrib1f(GLuint indx, GLfloat x);

void GLAPIENTRY
_es_VertexAttrib1fv(GLuint indx, const GLfloat* values);

void GLAPIENTRY
_es_VertexAttrib2f(GLuint indx, GLfloat x, GLfloat y);

void GLAPIENTRY
_es_VertexAttrib2fv(GLuint indx, const GLfloat* values);

void GLAPIENTRY
_es_VertexAttrib3f(GLuint indx, GLfloat x, GLfloat y, GLfloat z);

void GLAPIENTRY
_es_VertexAttrib3fv(GLuint indx, const GLfloat* values);

void GLAPIENTRY
_es_VertexAttrib4fv(GLuint indx, const GLfloat* values);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
