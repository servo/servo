/*
 * Mesa 3-D graphics library
 *
 * Copyright (C) 2010  VMware, Inc.  All Rights Reserved.
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


#ifndef UNIFORMS_H
#define UNIFORMS_H

#include "main/glheader.h"
#include "compiler/glsl_types.h"
#include "compiler/glsl/ir_uniform.h"
#include "program/prog_parameter.h"

#ifdef __cplusplus
extern "C" {
#endif


struct gl_program;
struct _glapi_table;

void GLAPIENTRY
_mesa_Uniform1f(GLint, GLfloat);
void GLAPIENTRY
_mesa_Uniform2f(GLint, GLfloat, GLfloat);
void GLAPIENTRY
_mesa_Uniform3f(GLint, GLfloat, GLfloat, GLfloat);
void GLAPIENTRY
_mesa_Uniform4f(GLint, GLfloat, GLfloat, GLfloat, GLfloat);
void GLAPIENTRY
_mesa_Uniform1i(GLint, GLint);
void GLAPIENTRY
_mesa_Uniform2i(GLint, GLint, GLint);
void GLAPIENTRY
_mesa_Uniform3i(GLint, GLint, GLint, GLint);
void GLAPIENTRY
_mesa_Uniform4i(GLint, GLint, GLint, GLint, GLint);
void GLAPIENTRY
_mesa_Uniform1fv(GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_Uniform2fv(GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_Uniform3fv(GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_Uniform4fv(GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_Uniform1iv(GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_Uniform2iv(GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_Uniform3iv(GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_Uniform4iv(GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_Uniform1ui(GLint location, GLuint v0);
void GLAPIENTRY
_mesa_Uniform2ui(GLint location, GLuint v0, GLuint v1);
void GLAPIENTRY
_mesa_Uniform3ui(GLint location, GLuint v0, GLuint v1, GLuint v2);
void GLAPIENTRY
_mesa_Uniform4ui(GLint location, GLuint v0, GLuint v1, GLuint v2, GLuint v3);
void GLAPIENTRY
_mesa_Uniform1uiv(GLint location, GLsizei count, const GLuint *value);
void GLAPIENTRY
_mesa_Uniform2uiv(GLint location, GLsizei count, const GLuint *value);
void GLAPIENTRY
_mesa_Uniform3uiv(GLint location, GLsizei count, const GLuint *value);
void GLAPIENTRY
_mesa_Uniform4uiv(GLint location, GLsizei count, const GLuint *value);
void GLAPIENTRY
_mesa_UniformMatrix2fv(GLint, GLsizei, GLboolean, const GLfloat *);
void GLAPIENTRY
_mesa_UniformMatrix3fv(GLint, GLsizei, GLboolean, const GLfloat *);
void GLAPIENTRY
_mesa_UniformMatrix4fv(GLint, GLsizei, GLboolean, const GLfloat *);
void GLAPIENTRY
_mesa_UniformMatrix2x3fv(GLint location, GLsizei count, GLboolean transpose,
                         const GLfloat *value);
void GLAPIENTRY
_mesa_UniformMatrix3x2fv(GLint location, GLsizei count, GLboolean transpose,
                         const GLfloat *value);
void GLAPIENTRY
_mesa_UniformMatrix2x4fv(GLint location, GLsizei count, GLboolean transpose,
                         const GLfloat *value);
void GLAPIENTRY
_mesa_UniformMatrix4x2fv(GLint location, GLsizei count, GLboolean transpose,
                         const GLfloat *value);
void GLAPIENTRY
_mesa_UniformMatrix3x4fv(GLint location, GLsizei count, GLboolean transpose,
                         const GLfloat *value);
void GLAPIENTRY
_mesa_UniformMatrix4x3fv(GLint location, GLsizei count, GLboolean transpose,
                         const GLfloat *value);

void GLAPIENTRY
_mesa_UniformHandleui64ARB(GLint location, GLuint64 value);
void GLAPIENTRY
_mesa_UniformHandleui64vARB(GLint location, GLsizei count,
                            const GLuint64 *value);
void GLAPIENTRY
_mesa_ProgramUniformHandleui64ARB(GLuint program, GLint location,
                                  GLuint64 value);
void GLAPIENTRY
_mesa_ProgramUniformHandleui64vARB(GLuint program, GLint location,
                                   GLsizei count, const GLuint64 *values);

void GLAPIENTRY
_mesa_ProgramUniform1f(GLuint program, GLint, GLfloat);
void GLAPIENTRY
_mesa_ProgramUniform2f(GLuint program, GLint, GLfloat, GLfloat);
void GLAPIENTRY
_mesa_ProgramUniform3f(GLuint program, GLint, GLfloat, GLfloat, GLfloat);
void GLAPIENTRY
_mesa_ProgramUniform4f(GLuint program, GLint, GLfloat, GLfloat, GLfloat, GLfloat);
void GLAPIENTRY
_mesa_ProgramUniform1i(GLuint program, GLint, GLint);
void GLAPIENTRY
_mesa_ProgramUniform2i(GLuint program, GLint, GLint, GLint);
void GLAPIENTRY
_mesa_ProgramUniform3i(GLuint program, GLint, GLint, GLint, GLint);
void GLAPIENTRY
_mesa_ProgramUniform4i(GLuint program, GLint, GLint, GLint, GLint, GLint);
void GLAPIENTRY
_mesa_ProgramUniform1fv(GLuint program, GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniform2fv(GLuint program, GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniform3fv(GLuint program, GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniform4fv(GLuint program, GLint, GLsizei, const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniform1iv(GLuint program, GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_ProgramUniform2iv(GLuint program, GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_ProgramUniform3iv(GLuint program, GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_ProgramUniform4iv(GLuint program, GLint, GLsizei, const GLint *);
void GLAPIENTRY
_mesa_ProgramUniform1ui(GLuint program, GLint location, GLuint v0);
void GLAPIENTRY
_mesa_ProgramUniform2ui(GLuint program, GLint location, GLuint v0, GLuint v1);
void GLAPIENTRY
_mesa_ProgramUniform3ui(GLuint program, GLint location, GLuint v0, GLuint v1,
                        GLuint v2);
void GLAPIENTRY
_mesa_ProgramUniform4ui(GLuint program, GLint location, GLuint v0, GLuint v1,
                        GLuint v2, GLuint v3);
void GLAPIENTRY
_mesa_ProgramUniform1uiv(GLuint program, GLint location, GLsizei count,
                         const GLuint *value);
void GLAPIENTRY
_mesa_ProgramUniform2uiv(GLuint program, GLint location, GLsizei count,
                         const GLuint *value);
void GLAPIENTRY
_mesa_ProgramUniform3uiv(GLuint program, GLint location, GLsizei count,
                         const GLuint *value);
void GLAPIENTRY
_mesa_ProgramUniform4uiv(GLuint program, GLint location, GLsizei count,
                         const GLuint *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix2fv(GLuint program, GLint, GLsizei, GLboolean,
                              const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniformMatrix3fv(GLuint program, GLint, GLsizei, GLboolean,
                              const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniformMatrix4fv(GLuint program, GLint, GLsizei, GLboolean,
                              const GLfloat *);
void GLAPIENTRY
_mesa_ProgramUniformMatrix2x3fv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLfloat *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix3x2fv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLfloat *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix2x4fv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLfloat *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix4x2fv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLfloat *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix3x4fv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLfloat *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix4x3fv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLfloat *value);

void GLAPIENTRY
_mesa_GetnUniformfvARB(GLuint, GLint, GLsizei, GLfloat *);
void GLAPIENTRY
_mesa_GetUniformfv(GLuint, GLint, GLfloat *);
void GLAPIENTRY
_mesa_GetnUniformivARB(GLuint, GLint, GLsizei, GLint *);
void GLAPIENTRY
_mesa_GetUniformuiv(GLuint, GLint, GLuint *);
void GLAPIENTRY
_mesa_GetnUniformuivARB(GLuint, GLint, GLsizei, GLuint *);
void GLAPIENTRY
_mesa_GetUniformuiv(GLuint program, GLint location, GLuint *params);
void GLAPIENTRY
_mesa_GetnUniformdvARB(GLuint, GLint, GLsizei, GLdouble *);
void GLAPIENTRY
_mesa_GetUniformdv(GLuint, GLint, GLdouble *);
GLint GLAPIENTRY
_mesa_GetUniformLocation(GLuint, const GLcharARB *);
GLint GLAPIENTRY
_mesa_GetUniformLocation_no_error(GLuint, const GLcharARB *);
GLuint GLAPIENTRY
_mesa_GetUniformBlockIndex(GLuint program,
			   const GLchar *uniformBlockName);
void GLAPIENTRY
_mesa_GetUniformIndices(GLuint program,
			GLsizei uniformCount,
			const GLchar * const *uniformNames,
			GLuint *uniformIndices);

void GLAPIENTRY
_mesa_UniformBlockBinding_no_error(GLuint program, GLuint uniformBlockIndex,
                                   GLuint uniformBlockBinding);

void GLAPIENTRY
_mesa_UniformBlockBinding(GLuint program,
			  GLuint uniformBlockIndex,
			  GLuint uniformBlockBinding);

void GLAPIENTRY
_mesa_ShaderStorageBlockBinding_no_error(GLuint program,
                                         GLuint shaderStorageBlockIndex,
                                         GLuint shaderStorageBlockBinding);

void GLAPIENTRY
_mesa_ShaderStorageBlockBinding(GLuint program,
                                GLuint shaderStorageBlockIndex,
                                GLuint shaderStorageBlockBinding);
void GLAPIENTRY
_mesa_GetActiveAtomicCounterBufferiv(GLuint program, GLuint bufferIndex,
                                     GLenum pname, GLint *params);
void GLAPIENTRY
_mesa_GetActiveUniformBlockiv(GLuint program,
			      GLuint uniformBlockIndex,
			      GLenum pname,
			      GLint *params);
void GLAPIENTRY
_mesa_GetActiveUniformBlockName(GLuint program,
				GLuint uniformBlockIndex,
				GLsizei bufSize,
				GLsizei *length,
				GLchar *uniformBlockName);
void GLAPIENTRY
_mesa_GetActiveUniformName(GLuint program, GLuint uniformIndex,
			   GLsizei bufSize, GLsizei *length,
			   GLchar *uniformName);
void GLAPIENTRY
_mesa_GetActiveUniform(GLuint, GLuint, GLsizei, GLsizei *,
                       GLint *, GLenum *, GLcharARB *);
void GLAPIENTRY
_mesa_GetActiveUniformsiv(GLuint program,
			  GLsizei uniformCount,
			  const GLuint *uniformIndices,
			  GLenum pname,
			  GLint *params);
void GLAPIENTRY
_mesa_GetUniformiv(GLuint, GLint, GLint *);

void GLAPIENTRY
_mesa_Uniform1d(GLint, GLdouble);
void GLAPIENTRY
_mesa_Uniform2d(GLint, GLdouble, GLdouble);
void GLAPIENTRY
_mesa_Uniform3d(GLint, GLdouble, GLdouble, GLdouble);
void GLAPIENTRY
_mesa_Uniform4d(GLint, GLdouble, GLdouble, GLdouble, GLdouble);

void GLAPIENTRY
_mesa_Uniform1dv(GLint, GLsizei, const GLdouble *);
void GLAPIENTRY
_mesa_Uniform2dv(GLint, GLsizei, const GLdouble *);
void GLAPIENTRY
_mesa_Uniform3dv(GLint, GLsizei, const GLdouble *);
void GLAPIENTRY
_mesa_Uniform4dv(GLint, GLsizei, const GLdouble *);

void GLAPIENTRY
_mesa_GetUniformi64vARB(GLuint, GLint, GLint64 *);
void GLAPIENTRY
_mesa_GetUniformui64vARB(GLuint, GLint, GLuint64 *);

void GLAPIENTRY
_mesa_GetnUniformi64vARB(GLuint, GLint, GLsizei, GLint64 *);
void GLAPIENTRY
_mesa_GetnUniformui64vARB(GLuint, GLint, GLsizei, GLuint64 *);

void GLAPIENTRY
_mesa_UniformMatrix2dv(GLint, GLsizei, GLboolean, const GLdouble *);
void GLAPIENTRY
_mesa_UniformMatrix3dv(GLint, GLsizei, GLboolean, const GLdouble *);
void GLAPIENTRY
_mesa_UniformMatrix4dv(GLint, GLsizei, GLboolean, const GLdouble *);
void GLAPIENTRY
_mesa_UniformMatrix2x3dv(GLint location, GLsizei count, GLboolean transpose,
                         const GLdouble *value);
void GLAPIENTRY
_mesa_UniformMatrix3x2dv(GLint location, GLsizei count, GLboolean transpose,
                         const GLdouble *value);
void GLAPIENTRY
_mesa_UniformMatrix2x4dv(GLint location, GLsizei count, GLboolean transpose,
                         const GLdouble *value);
void GLAPIENTRY
_mesa_UniformMatrix4x2dv(GLint location, GLsizei count, GLboolean transpose,
                         const GLdouble *value);
void GLAPIENTRY
_mesa_UniformMatrix3x4dv(GLint location, GLsizei count, GLboolean transpose,
                         const GLdouble *value);
void GLAPIENTRY
_mesa_UniformMatrix4x3dv(GLint location, GLsizei count, GLboolean transpose,
                         const GLdouble *value);

void GLAPIENTRY
_mesa_ProgramUniform1d(GLuint program, GLint, GLdouble);
void GLAPIENTRY
_mesa_ProgramUniform2d(GLuint program, GLint, GLdouble, GLdouble);
void GLAPIENTRY
_mesa_ProgramUniform3d(GLuint program, GLint, GLdouble, GLdouble, GLdouble);
void GLAPIENTRY
_mesa_ProgramUniform4d(GLuint program, GLint, GLdouble, GLdouble, GLdouble, GLdouble);

void GLAPIENTRY
_mesa_ProgramUniform1dv(GLuint program, GLint, GLsizei, const GLdouble *);
void GLAPIENTRY
_mesa_ProgramUniform2dv(GLuint program, GLint, GLsizei, const GLdouble *);
void GLAPIENTRY
_mesa_ProgramUniform3dv(GLuint program, GLint, GLsizei, const GLdouble *);
void GLAPIENTRY
_mesa_ProgramUniform4dv(GLuint program, GLint, GLsizei, const GLdouble *);

void GLAPIENTRY
_mesa_ProgramUniformMatrix2dv(GLuint program, GLint, GLsizei, GLboolean,
                              const GLdouble *);
void GLAPIENTRY
_mesa_ProgramUniformMatrix3dv(GLuint program, GLint, GLsizei, GLboolean,
                              const GLdouble *);
void GLAPIENTRY
_mesa_ProgramUniformMatrix4dv(GLuint program, GLint, GLsizei, GLboolean,
                              const GLdouble *);
void GLAPIENTRY
_mesa_ProgramUniformMatrix2x3dv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLdouble *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix3x2dv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLdouble *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix2x4dv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLdouble *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix4x2dv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLdouble *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix3x4dv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLdouble *value);
void GLAPIENTRY
_mesa_ProgramUniformMatrix4x3dv(GLuint program, GLint location, GLsizei count,
                                GLboolean transpose, const GLdouble *value);

void GLAPIENTRY
_mesa_Uniform1i64ARB(GLint, GLint64);
void GLAPIENTRY
_mesa_Uniform2i64ARB(GLint, GLint64, GLint64);
void GLAPIENTRY
_mesa_Uniform3i64ARB(GLint, GLint64, GLint64, GLint64);
void GLAPIENTRY
_mesa_Uniform4i64ARB(GLint, GLint64, GLint64, GLint64, GLint64);

void GLAPIENTRY
_mesa_Uniform1i64vARB(GLint, GLsizei, const GLint64 *);
void GLAPIENTRY
_mesa_Uniform2i64vARB(GLint, GLsizei, const GLint64 *);
void GLAPIENTRY
_mesa_Uniform3i64vARB(GLint, GLsizei, const GLint64 *);
void GLAPIENTRY
_mesa_Uniform4i64vARB(GLint, GLsizei, const GLint64 *);

void GLAPIENTRY
_mesa_Uniform1ui64ARB(GLint, GLuint64);
void GLAPIENTRY
_mesa_Uniform2ui64ARB(GLint, GLuint64, GLuint64);
void GLAPIENTRY
_mesa_Uniform3ui64ARB(GLint, GLuint64, GLuint64, GLuint64);
void GLAPIENTRY
_mesa_Uniform4ui64ARB(GLint, GLuint64, GLuint64, GLuint64, GLuint64);

void GLAPIENTRY
_mesa_Uniform1ui64vARB(GLint, GLsizei, const GLuint64 *);
void GLAPIENTRY
_mesa_Uniform2ui64vARB(GLint, GLsizei, const GLuint64 *);
void GLAPIENTRY
_mesa_Uniform3ui64vARB(GLint, GLsizei, const GLuint64 *);
void GLAPIENTRY
_mesa_Uniform4ui64vARB(GLint, GLsizei, const GLuint64 *);

void GLAPIENTRY
_mesa_ProgramUniform1i64ARB(GLuint, GLint, GLint64);
void GLAPIENTRY
_mesa_ProgramUniform2i64ARB(GLuint, GLint, GLint64, GLint64);
void GLAPIENTRY
_mesa_ProgramUniform3i64ARB(GLuint, GLint, GLint64, GLint64, GLint64);
void GLAPIENTRY
_mesa_ProgramUniform4i64ARB(GLuint, GLint, GLint64, GLint64, GLint64, GLint64);

void GLAPIENTRY
_mesa_ProgramUniform1i64vARB(GLuint, GLint, GLsizei, const GLint64 *);
void GLAPIENTRY
_mesa_ProgramUniform2i64vARB(GLuint, GLint, GLsizei, const GLint64 *);
void GLAPIENTRY
_mesa_ProgramUniform3i64vARB(GLuint, GLint, GLsizei, const GLint64 *);
void GLAPIENTRY
_mesa_ProgramUniform4i64vARB(GLuint, GLint, GLsizei, const GLint64 *);

void GLAPIENTRY
_mesa_ProgramUniform1ui64ARB(GLuint, GLint, GLuint64);
void GLAPIENTRY
_mesa_ProgramUniform2ui64ARB(GLuint, GLint, GLuint64, GLuint64);
void GLAPIENTRY
_mesa_ProgramUniform3ui64ARB(GLuint, GLint, GLuint64, GLuint64, GLuint64);
void GLAPIENTRY
_mesa_ProgramUniform4ui64ARB(GLuint, GLint, GLuint64, GLuint64, GLuint64, GLuint64);

void GLAPIENTRY
_mesa_ProgramUniform1ui64vARB(GLuint, GLint, GLsizei, const GLuint64 *);
void GLAPIENTRY
_mesa_ProgramUniform2ui64vARB(GLuint, GLint, GLsizei, const GLuint64 *);
void GLAPIENTRY
_mesa_ProgramUniform3ui64vARB(GLuint, GLint, GLsizei, const GLuint64 *);
void GLAPIENTRY
_mesa_ProgramUniform4ui64vARB(GLuint, GLint, GLsizei, const GLuint64 *);

void
_mesa_uniform(GLint location, GLsizei count, const GLvoid *values,
              struct gl_context *, struct gl_shader_program *,
              enum glsl_base_type basicType, unsigned src_components);

void
_mesa_uniform_matrix(GLint location, GLsizei count,
                     GLboolean transpose, const void *values,
                     struct gl_context *, struct gl_shader_program *,
                     GLuint cols, GLuint rows, enum glsl_base_type basicType);

void
_mesa_uniform_handle(GLint location, GLsizei count, const GLvoid *values,
                     struct gl_context *, struct gl_shader_program *);

void
_mesa_get_uniform(struct gl_context *ctx, GLuint program, GLint location,
		  GLsizei bufSize, enum glsl_base_type returnType,
		  GLvoid *paramsOut);

extern void
_mesa_uniform_attach_driver_storage(struct gl_uniform_storage *,
				    unsigned element_stride,
				    unsigned vector_stride,
				    enum gl_uniform_driver_format format,
				    void *data);

extern void
_mesa_uniform_detach_all_driver_storage(struct gl_uniform_storage *uni);

extern void
_mesa_propagate_uniforms_to_driver_storage(struct gl_uniform_storage *uni,
					   unsigned array_index,
					   unsigned count);

extern void
_mesa_update_shader_textures_used(struct gl_shader_program *shProg,
				  struct gl_program *prog);

extern bool
_mesa_sampler_uniforms_are_valid(const struct gl_shader_program *shProg,
				 char *errMsg, size_t errMsgLength);
extern bool
_mesa_sampler_uniforms_pipeline_are_valid(struct gl_pipeline_object *);

extern void
_mesa_flush_vertices_for_uniforms(struct gl_context *ctx,
                                  const struct gl_uniform_storage *uni);

struct gl_builtin_uniform_element {
   const char *field;
   gl_state_index16 tokens[STATE_LENGTH];
   int swizzle;
};

struct gl_builtin_uniform_desc {
   const char *name;
   const struct gl_builtin_uniform_element *elements;
   unsigned int num_elements;
};

#ifdef __cplusplus
}
#endif


#endif /* UNIFORMS_H */
