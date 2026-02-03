/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 1999-2016  Brian Paul, et al   All Rights Reserved.
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


#ifndef DEBUG_OUTPUT_H
#define DEBUG_OUTPUT_H


#include <stdio.h>
#include <stdarg.h>
#include "glheader.h"
#include "menums.h"


#ifdef __cplusplus
extern "C" {
#endif

struct gl_context;

void
_mesa_init_debug_output(struct gl_context *ctx);

void
_mesa_free_errors_data(struct gl_context *ctx);

void
_mesa_debug_get_id(GLuint *id);

bool
_mesa_set_debug_state_int(struct gl_context *ctx, GLenum pname, GLint val);

GLint
_mesa_get_debug_state_int(struct gl_context *ctx, GLenum pname);

void *
_mesa_get_debug_state_ptr(struct gl_context *ctx, GLenum pname);

void
_mesa_log_msg(struct gl_context *ctx, enum mesa_debug_source source,
              enum mesa_debug_type type, GLuint id,
              enum mesa_debug_severity severity, GLint len, const char *buf);

bool
_mesa_debug_is_message_enabled(const struct gl_debug_state *debug,
                               enum mesa_debug_source source,
                               enum mesa_debug_type type,
                               GLuint id,
                               enum mesa_debug_severity severity);

void GLAPIENTRY
_mesa_DebugMessageInsert(GLenum source, GLenum type, GLuint id,
                         GLenum severity, GLint length,
                         const GLchar* buf);

GLuint GLAPIENTRY
_mesa_GetDebugMessageLog(GLuint count, GLsizei logSize, GLenum* sources,
                         GLenum* types, GLenum* ids, GLenum* severities,
                         GLsizei* lengths, GLchar* messageLog);

void GLAPIENTRY
_mesa_DebugMessageControl(GLenum source, GLenum type, GLenum severity,
                          GLsizei count, const GLuint *ids,
                          GLboolean enabled);

void GLAPIENTRY
_mesa_DebugMessageCallback(GLDEBUGPROC callback,
                           const void *userParam);

void GLAPIENTRY
_mesa_PushDebugGroup(GLenum source, GLuint id, GLsizei length,
                     const GLchar *message);

void GLAPIENTRY
_mesa_PopDebugGroup(void);

void GLAPIENTRY
_mesa_StringMarkerGREMEDY(GLsizei len, const GLvoid *string);


#ifdef __cplusplus
}
#endif


#endif /* DEBUG_OUTPUT_H */
