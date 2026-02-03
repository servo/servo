/**************************************************************************
 * 
 * Copyright 2007-2008 VMware, Inc.
 * All Rights Reserved.
 * 
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sub license, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 * 
 * The above copyright notice and this permission notice (including the
 * next paragraph) shall be included in all copies or substantial portions
 * of the Software.
 * 
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT.
 * IN NO EVENT SHALL VMWARE AND/OR ITS SUPPLIERS BE LIABLE FOR
 * ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 * 
 **************************************************************************/

#ifndef P_COMPILER_H
#define P_COMPILER_H


#include "c99_compat.h" /* inline, __func__, etc. */

#include "p_config.h"

#include "util/macros.h"

#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <stdarg.h>
#include <limits.h>


#if defined(_WIN32) && !defined(__WIN32__)
#define __WIN32__
#endif

#if defined(_MSC_VER)

#include <intrin.h>

/* Avoid 'expression is always true' warning */
#pragma warning(disable: 4296)

#endif /* _MSC_VER */


/*
 * Alternative stdint.h and stdbool.h headers are supplied in include/c99 for
 * systems that lack it.
 */
#include <stdint.h>
#include <stdbool.h>


#ifdef __cplusplus
extern "C" {
#endif


#if !defined(__HAIKU__) && !defined(__USE_MISC)
#if !defined(PIPE_OS_ANDROID)
typedef unsigned int       uint;
#endif
typedef unsigned short     ushort;
#endif
typedef unsigned char      ubyte;

typedef unsigned char boolean;
#ifndef TRUE
#define TRUE  true
#endif
#ifndef FALSE
#define FALSE false
#endif

#ifndef va_copy
#ifdef __va_copy
#define va_copy(dest, src) __va_copy((dest), (src))
#else
#define va_copy(dest, src) (dest) = (src)
#endif
#endif


/* XXX: Use standard `__func__` instead */
#ifndef __FUNCTION__
#  define __FUNCTION__ __func__
#endif


/* This should match linux gcc cdecl semantics everywhere, so that we
 * just codegen one calling convention on all platforms.
 */
#ifdef _MSC_VER
#define PIPE_CDECL __cdecl
#else
#define PIPE_CDECL
#endif



#if defined(__GNUC__)
#define PIPE_DEPRECATED  __attribute__((__deprecated__))
#else
#define PIPE_DEPRECATED
#endif



/* Macros for data alignment. */
#if defined(__GNUC__)

/* See http://gcc.gnu.org/onlinedocs/gcc-4.4.2/gcc/Type-Attributes.html */
#define PIPE_ALIGN_TYPE(_alignment, _type) _type __attribute__((aligned(_alignment)))

/* See http://gcc.gnu.org/onlinedocs/gcc-4.4.2/gcc/Variable-Attributes.html */
#define PIPE_ALIGN_VAR(_alignment) __attribute__((aligned(_alignment)))

#if defined(__GNUC__) && defined(PIPE_ARCH_X86)
#define PIPE_ALIGN_STACK __attribute__((force_align_arg_pointer))
#else
#define PIPE_ALIGN_STACK
#endif

#elif defined(_MSC_VER)

/* See http://msdn.microsoft.com/en-us/library/83ythb65.aspx */
#define PIPE_ALIGN_TYPE(_alignment, _type) __declspec(align(_alignment)) _type
#define PIPE_ALIGN_VAR(_alignment) __declspec(align(_alignment))

#define PIPE_ALIGN_STACK

#elif defined(SWIG)

#define PIPE_ALIGN_TYPE(_alignment, _type) _type
#define PIPE_ALIGN_VAR(_alignment)

#define PIPE_ALIGN_STACK

#else

#error "Unsupported compiler"

#endif


#if defined(__GNUC__)

#define PIPE_READ_WRITE_BARRIER() __asm__("":::"memory")

#elif defined(_MSC_VER)

#define PIPE_READ_WRITE_BARRIER() _ReadWriteBarrier()

#else

#warning "Unsupported compiler"
#define PIPE_READ_WRITE_BARRIER() /* */

#endif

#if defined(__cplusplus)
}
#endif


#endif /* P_COMPILER_H */
