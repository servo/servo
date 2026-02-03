/*
 * Copyright © 2014 Intel Corporation
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
 * \file lower_const_arrays_to_uniforms.cpp
 *
 * Lower constant arrays to uniform arrays.
 *
 * Some driver backends (such as i965 and nouveau) don't handle constant arrays
 * gracefully, instead treating them as ordinary writable temporary arrays.
 * Since arrays can be large, this often means spilling them to scratch memory,
 * which usually involves a large number of instructions.
 *
 * This must be called prior to link_set_uniform_initializers(); we need the
 * linker to process our new uniform's constant initializer.
 *
 * This should be called after optimizations, since those can result in
 * splitting and removing arrays that are indexed by constant expressions.
 */
#include "ir.h"
#include "ir_visitor.h"
#include "ir_rvalue_visitor.h"
#include "compiler/glsl_types.h"

namespace {
class lower_const_array_visitor : public ir_rvalue_visitor {
public:
   lower_const_array_visitor(exec_list *insts, unsigned s,
                             unsigned available_uni_components)
   {
      instructions = insts;
      stage = s;
      const_count = 0;
      free_uni_components = available_uni_components;
      progress = false;
   }

   bool run()
   {
      visit_list_elements(this, instructions);
      return progress;
   }

   ir_visitor_status visit_enter(ir_texture *);
   void handle_rvalue(ir_rvalue **rvalue);

private:
   exec_list *instructions;
   unsigned stage;
   unsigned const_count;
   unsigned free_uni_components;
   bool progress;
};

ir_visitor_status
lower_const_array_visitor::visit_enter(ir_texture *)
{
   return visit_continue_with_parent;
}

void
lower_const_array_visitor::handle_rvalue(ir_rvalue **rvalue)
{
   if (!*rvalue)
      return;

   ir_constant *con = (*rvalue)->as_constant();
   if (!con || !con->type->is_array())
      return;

   /* How many uniform component slots are required? */
   unsigned component_slots = con->type->component_slots();

   /* We would utilize more than is available, bail out. */
   if (component_slots > free_uni_components)
      return;

   free_uni_components -= component_slots;

   void *mem_ctx = ralloc_parent(con);

   /* In the very unlikely event of 4294967295 constant arrays in a single
    * shader, don't promote this to a uniform.
    */
   unsigned limit = ~0;
   if (const_count == limit)
      return;

   char *uniform_name = ralloc_asprintf(mem_ctx, "constarray_%x_%u",
                                        const_count, stage);
   const_count++;

   ir_variable *uni =
      new(mem_ctx) ir_variable(con->type, uniform_name, ir_var_uniform);
   uni->constant_initializer = con;
   uni->constant_value = con;
   uni->data.has_initializer = true;
   uni->data.how_declared = ir_var_hidden;
   uni->data.read_only = true;
   /* Assume the whole thing is accessed. */
   uni->data.max_array_access = uni->type->length - 1;
   instructions->push_head(uni);

   *rvalue = new(mem_ctx) ir_dereference_variable(uni);

   progress = true;
}

} /* anonymous namespace */


static unsigned
count_uniforms(exec_list *instructions)
{
   unsigned total = 0;

   foreach_in_list(ir_instruction, node, instructions) {
      ir_variable *const var = node->as_variable();

      if (!var || var->data.mode != ir_var_uniform)
         continue;

      total += var->type->component_slots();
   }
   return total;
}

bool
lower_const_arrays_to_uniforms(exec_list *instructions, unsigned stage,
                               unsigned max_uniform_components)
{
   unsigned uniform_components = count_uniforms(instructions);
   unsigned free_uniform_slots = max_uniform_components - uniform_components;

   lower_const_array_visitor v(instructions, stage, free_uniform_slots);
   return v.run();
}
