/*
 * Copyright ©2019 Collabora Ltd.
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
 * \file lower_xfb_varying.cpp
 *
 */

#include "ir.h"
#include "main/mtypes.h"
#include "glsl_symbol_table.h"
#include "util/strndup.h"

namespace {

/**
 * Visitor that splices varying packing code before every return.
 */
class lower_xfb_var_splicer : public ir_hierarchical_visitor
{
public:
   explicit lower_xfb_var_splicer(void *mem_ctx,
                                  const exec_list *instructions);

   virtual ir_visitor_status visit_leave(ir_return *ret);
   virtual ir_visitor_status visit_leave(ir_function_signature *sig);

private:
   /**
    * Memory context used to allocate new instructions for the shader.
    */
   void * const mem_ctx;

   /**
    * Instructions that should be spliced into place before each return.
    */
   const exec_list *instructions;
};

} /* anonymous namespace */


lower_xfb_var_splicer::lower_xfb_var_splicer(void *mem_ctx, const exec_list *instructions)
   : mem_ctx(mem_ctx), instructions(instructions)
{
}

ir_visitor_status
lower_xfb_var_splicer::visit_leave(ir_return *ret)
{
   foreach_in_list(ir_instruction, ir, this->instructions) {
      ret->insert_before(ir->clone(this->mem_ctx, NULL));
   }
   return visit_continue;
}

/** Insert a copy-back assignment at the end of the main() function */
ir_visitor_status
lower_xfb_var_splicer::visit_leave(ir_function_signature *sig)
{
   if (strcmp(sig->function_name(), "main") != 0)
      return visit_continue;

   if (((ir_instruction*)sig->body.get_tail())->ir_type == ir_type_return)
      return visit_continue;

   foreach_in_list(ir_instruction, ir, this->instructions) {
      sig->body.push_tail(ir->clone(this->mem_ctx, NULL));
   }

   return visit_continue;
}

static char*
get_field_name(const char *name)
{
   const char *first_dot = strchr(name, '.');
   const char *first_square_bracket = strchr(name, '[');
   int name_size = 0;

   if (!first_square_bracket && !first_dot)
      name_size = strlen(name);
   else if ((!first_square_bracket ||
            (first_dot && first_dot < first_square_bracket)))
      name_size = first_dot - name;
   else
      name_size = first_square_bracket - name;

   return strndup(name, name_size);
}

/* Generate a new name given the old xfb declaration string by replacing dots
 * with '_', brackets with '@' and appending "-xfb" */
static char *
generate_new_name(void *mem_ctx, const char *name)
{
   char *new_name;
   unsigned i = 0;

   new_name = ralloc_strdup(mem_ctx, name);
   while (new_name[i]) {
      if (new_name[i] == '.') {
         new_name[i] = '_';
      } else if (new_name[i] == '[' || new_name[i] == ']') {
         new_name[i] = '@';
      }
      i++;
   }

   if (!ralloc_strcat(&new_name, "-xfb")) {
      ralloc_free(new_name);
      return NULL;
   }

   return new_name;
}

/* Get the dereference for the given variable name. The method is called
 * recursively to parse array indices and struct members. */
static bool
get_deref(void *ctx,
          const char *name,
          struct gl_linked_shader *shader,
          ir_dereference **deref,
          const glsl_type **type)
{
   if (name[0] == '\0') {
      /* End */
      return (*deref != NULL);
   } else if (name[0] == '[') {
      /* Array index */
      char *endptr = NULL;
      unsigned index;

      index = strtol(name + 1, &endptr, 10);
      assert(*type != NULL && (*type)->is_array() && endptr[0] == ']');
      *deref = new(ctx) ir_dereference_array(*deref, new(ctx) ir_constant(index));
      *type = (*type)->without_array();
      return get_deref(ctx, endptr + 1, shader, deref, type);
   } else if (name[0] == '.') {
      /* Struct member */
      char *field = get_field_name(name + 1);

      assert(*type != NULL && (*type)->is_struct() && field != NULL);
      *deref = new(ctx) ir_dereference_record(*deref, field);
      *type = (*type)->field_type(field);
      assert(*type != glsl_type::error_type);
      name += 1 + strlen(field);
      free(field);
      return get_deref(ctx, name, shader, deref, type);
   } else {
      /* Top level variable */
      char *field = get_field_name(name);
      ir_variable *toplevel_var;

      toplevel_var = shader->symbols->get_variable(field);
      name += strlen(field);
      free(field);
      if (toplevel_var == NULL) {
         return false;
      }

      *deref = new (ctx) ir_dereference_variable(toplevel_var);
      *type = toplevel_var->type;
      return get_deref(ctx, name, shader, deref, type);
   }
}

ir_variable *
lower_xfb_varying(void *mem_ctx,
                  struct gl_linked_shader *shader,
                  const char *old_var_name)
{
   exec_list new_instructions;
   char *new_var_name;
   ir_dereference *deref = NULL;
   const glsl_type *type = NULL;

   if (!get_deref(mem_ctx, old_var_name, shader, &deref, &type)) {
      if (deref) {
         delete deref;
      }
      return NULL;
   }

   new_var_name = generate_new_name(mem_ctx, old_var_name);
   ir_variable *new_variable
      = new(mem_ctx) ir_variable(type, new_var_name, ir_var_shader_out);
   new_variable->data.assigned = true;
   new_variable->data.used = true;
   shader->ir->push_head(new_variable);
   ralloc_free(new_var_name);

   ir_dereference *lhs = new(mem_ctx) ir_dereference_variable(new_variable);
   ir_assignment *new_assignment = new(mem_ctx) ir_assignment(lhs, deref);
   new_instructions.push_tail(new_assignment);

   lower_xfb_var_splicer splicer(mem_ctx, &new_instructions);
   visit_list_elements(&splicer, shader->ir);

   return new_variable;
}
