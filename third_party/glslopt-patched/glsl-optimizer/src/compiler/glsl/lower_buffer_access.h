/*
 * Copyright (c) 2015 Intel Corporation
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

/**
 * \file lower_buffer_access.h
 *
 * Helper for IR lowering pass to replace dereferences of buffer object based
 * shader variables with intrinsic function calls.
 *
 * This helper is used by lowering passes for UBOs, SSBOs and compute shader
 * shared variables.
 */

#ifndef LOWER_BUFFER_ACCESS_H
#define LOWER_BUFFER_ACCESS_H

#include "ir.h"
#include "ir_rvalue_visitor.h"

namespace lower_buffer_access {

class lower_buffer_access : public ir_rvalue_enter_visitor {
public:
   virtual void
   insert_buffer_access(void *mem_ctx, ir_dereference *deref,
                        const glsl_type *type, ir_rvalue *offset,
                        unsigned mask, int channel) = 0;

   void emit_access(void *mem_ctx, bool is_write, ir_dereference *deref,
                    ir_variable *base_offset, unsigned int deref_offset,
                    bool row_major, const glsl_type *matrix_type,
                    enum glsl_interface_packing packing,
                    unsigned int write_mask);

   bool is_dereferenced_thing_row_major(const ir_rvalue *deref);

   void setup_buffer_access(void *mem_ctx, ir_rvalue *deref,
                            ir_rvalue **offset, unsigned *const_offset,
                            bool *row_major,
                            const glsl_type **matrix_type,
                            const glsl_struct_field **struct_field,
                            enum glsl_interface_packing packing);

protected:
   bool use_std430_as_default;
};

} /* namespace lower_buffer_access */

#endif /* LOWER_BUFFER_ACCESS_H */
