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
 * \brief Array type draw functions, the main workhorse of any OpenGL API
 * \author Keith Whitwell
 */


#ifndef DRAW_H
#define DRAW_H

#include <stdbool.h>
#include "main/glheader.h"

#ifdef __cplusplus
extern "C" {
#endif

struct gl_context;

struct _mesa_prim
{
   GLubyte mode;    /**< GL_POINTS, GL_LINES, GL_QUAD_STRIP, etc */

   /**
    * tnl: If true, line stipple emulation will reset the pattern walker.
    * vbo: If false and the primitive is a line loop, the first vertex is
    *      the beginning of the line loop and it won't be drawn.
    *      Instead, it will be moved to the end.
    */
   bool begin;

   /**
    * tnl: If true and the primitive is a line loop, it will be closed.
    * vbo: Same as tnl.
    */
   bool end;

   GLuint start;
   GLuint count;
   GLint basevertex;
   GLuint draw_id;
};

/* Would like to call this a "vbo_index_buffer", but this would be
 * confusing as the indices are not neccessarily yet in a non-null
 * buffer object.
 */
struct _mesa_index_buffer
{
   GLuint count;
   uint8_t index_size_shift; /* logbase2(index_size) */
   struct gl_buffer_object *obj;
   const void *ptr;
};


void
_mesa_initialize_exec_dispatch(const struct gl_context *ctx,
                               struct _glapi_table *exec);

void GLAPIENTRY
_mesa_EvalMesh1(GLenum mode, GLint i1, GLint i2);

void GLAPIENTRY
_mesa_EvalMesh2(GLenum mode, GLint i1, GLint i2, GLint j1, GLint j2);

void GLAPIENTRY
_mesa_DrawElementsInstancedARB(GLenum mode, GLsizei count, GLenum type,
                               const GLvoid * indices, GLsizei numInstances);

void GLAPIENTRY
_mesa_DrawArraysInstancedBaseInstance(GLenum mode, GLint first,
                                      GLsizei count, GLsizei numInstances,
                                      GLuint baseInstance);

void GLAPIENTRY
_mesa_DrawElementsInstancedBaseVertex(GLenum mode, GLsizei count,
                                      GLenum type, const GLvoid * indices,
                                      GLsizei numInstances,
                                      GLint basevertex);

void GLAPIENTRY
_mesa_DrawElementsInstancedBaseInstance(GLenum mode, GLsizei count,
                                        GLenum type,
                                        const GLvoid *indices,
                                        GLsizei numInstances,
                                        GLuint baseInstance);

void GLAPIENTRY
_mesa_DrawTransformFeedbackStream(GLenum mode, GLuint name, GLuint stream);

void GLAPIENTRY
_mesa_DrawTransformFeedbackInstanced(GLenum mode, GLuint name,
                                     GLsizei primcount);

void GLAPIENTRY
_mesa_DrawTransformFeedbackStreamInstanced(GLenum mode, GLuint name,
                                           GLuint stream,
                                           GLsizei primcount);

void GLAPIENTRY
_mesa_DrawArraysIndirect(GLenum mode, const GLvoid *indirect);

void GLAPIENTRY
_mesa_DrawElementsIndirect(GLenum mode, GLenum type, const GLvoid *indirect);

void GLAPIENTRY
_mesa_MultiDrawArraysIndirect(GLenum mode, const GLvoid *indirect,
                              GLsizei primcount, GLsizei stride);

void GLAPIENTRY
_mesa_MultiDrawElementsIndirect(GLenum mode, GLenum type,
                                const GLvoid *indirect,
                                GLsizei primcount, GLsizei stride);

void GLAPIENTRY
_mesa_MultiDrawArraysIndirectCountARB(GLenum mode, GLintptr indirect,
                                      GLintptr drawcount_offset,
                                      GLsizei maxdrawcount, GLsizei stride);

void GLAPIENTRY
_mesa_MultiDrawElementsIndirectCountARB(GLenum mode, GLenum type,
                                        GLintptr indirect,
                                        GLintptr drawcount_offset,
                                        GLsizei maxdrawcount, GLsizei stride);

void GLAPIENTRY
_mesa_DrawArrays(GLenum mode, GLint first, GLsizei count);


void GLAPIENTRY
_mesa_DrawArraysInstancedARB(GLenum mode, GLint first, GLsizei count,
                             GLsizei primcount);

void GLAPIENTRY
_mesa_DrawElementsInstancedBaseVertexBaseInstance(GLenum mode,
                                                  GLsizei count,
                                                  GLenum type,
                                                  const GLvoid *indices,
                                                  GLsizei numInstances,
                                                  GLint basevertex,
                                                  GLuint baseInstance);

void GLAPIENTRY
_mesa_DrawElements(GLenum mode, GLsizei count, GLenum type,
                   const GLvoid *indices);


void GLAPIENTRY
_mesa_DrawRangeElements(GLenum mode, GLuint start, GLuint end, GLsizei count,
                        GLenum type, const GLvoid *indices);


void GLAPIENTRY
_mesa_DrawElementsBaseVertex(GLenum mode, GLsizei count, GLenum type,
                             const GLvoid *indices, GLint basevertex);


void GLAPIENTRY
_mesa_DrawRangeElementsBaseVertex(GLenum mode, GLuint start, GLuint end,
                                  GLsizei count, GLenum type,
                                  const GLvoid *indices,
                                  GLint basevertex);


void GLAPIENTRY
_mesa_DrawTransformFeedback(GLenum mode, GLuint name);



void GLAPIENTRY
_mesa_MultiDrawArrays(GLenum mode, const GLint *first,
                      const GLsizei *count, GLsizei primcount);


void GLAPIENTRY
_mesa_MultiDrawElements(GLenum mode, const GLsizei *count, GLenum type,
                        const GLvoid *const *indices, GLsizei primcount);


void GLAPIENTRY
_mesa_MultiDrawElementsBaseVertex(GLenum mode,
                                  const GLsizei *count, GLenum type,
                                  const GLvoid * const * indices, GLsizei primcount,
                                  const GLint *basevertex);


void GLAPIENTRY
_mesa_MultiModeDrawArraysIBM(const GLenum * mode, const GLint * first,
                             const GLsizei * count,
                             GLsizei primcount, GLint modestride);


void GLAPIENTRY
_mesa_MultiModeDrawElementsIBM(const GLenum * mode, const GLsizei * count,
                               GLenum type, const GLvoid * const * indices,
                               GLsizei primcount, GLint modestride);


#ifdef __cplusplus
} // extern "C"
#endif

#endif
