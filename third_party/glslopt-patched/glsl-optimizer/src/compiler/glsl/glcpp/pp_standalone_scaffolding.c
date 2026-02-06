/*
 * Copyright © 2019 Timothy Arceri
 *
 * Permission is hereby granted, free of charge, to any person obtaining a
 * copy of this software and associated documentation files (the "Software"),
 * to deal in the Software without restriction, including without limitation
 * the rights to use, copy, modify, merge, publish, distribute, sublicense,
 * and/or sell copies of the Software, and to permit persons to whom the
 * Software is furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice (including the next
 * paragraph) shall be included in all copies or substantial portions of the
 * Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 * FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 */

/* This file declares stripped-down versions of functions that
 * normally exist outside of the glsl folder, so that they can be used
 * when running the GLSL compiler standalone (for unit testing or
 * compiling builtins).
 */

#include "pp_standalone_scaffolding.h"

const char *
_mesa_lookup_shader_include(struct gl_context *ctx, char *path,
                            bool error_check)
{
   (void) ctx;
   (void) path;
   (void) error_check;

   return NULL;
}

size_t
_mesa_get_shader_include_cursor(struct gl_shared_state *shared)
{
   (void) shared;

   return 0;
}

void
_mesa_set_shader_include_cursor(struct gl_shared_state *shared,
                                 size_t cursor)
{
   (void) shared;
   (void) cursor;
}
