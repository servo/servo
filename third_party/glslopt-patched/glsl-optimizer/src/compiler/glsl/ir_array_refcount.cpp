/*
 * Copyright © 2016 Intel Corporation
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
 * \file ir_array_refcount.cpp
 *
 * Provides a visitor which produces a list of variables referenced.
 */

#include "ir.h"
#include "ir_visitor.h"
#include "ir_array_refcount.h"
#include "compiler/glsl_types.h"
#include "util/hash_table.h"

ir_array_refcount_visitor::ir_array_refcount_visitor()
   : last_array_deref(0), derefs(0), num_derefs(0), derefs_size(0)
{
   this->mem_ctx = ralloc_context(NULL);
   this->ht = _mesa_pointer_hash_table_create(NULL);
}

static void
free_entry(struct hash_entry *entry)
{
   ir_array_refcount_entry *ivre = (ir_array_refcount_entry *) entry->data;
   delete ivre;
}

ir_array_refcount_visitor::~ir_array_refcount_visitor()
{
   ralloc_free(this->mem_ctx);
   _mesa_hash_table_destroy(this->ht, free_entry);
}

ir_array_refcount_entry::ir_array_refcount_entry(ir_variable *var)
   : var(var), is_referenced(false)
{
   num_bits = MAX2(1, var->type->arrays_of_arrays_size());
   bits = new BITSET_WORD[BITSET_WORDS(num_bits)];
   memset(bits, 0, BITSET_WORDS(num_bits) * sizeof(bits[0]));

   /* Count the "depth" of the arrays-of-arrays. */
   array_depth = 0;
   for (const glsl_type *type = var->type;
        type->is_array();
        type = type->fields.array) {
      array_depth++;
   }
}


ir_array_refcount_entry::~ir_array_refcount_entry()
{
   delete [] bits;
}

ir_array_refcount_entry *
ir_array_refcount_visitor::get_variable_entry(ir_variable *var)
{
   assert(var);

   struct hash_entry *e = _mesa_hash_table_search(this->ht, var);
   if (e)
      return (ir_array_refcount_entry *)e->data;

   ir_array_refcount_entry *entry = new ir_array_refcount_entry(var);
   _mesa_hash_table_insert(this->ht, var, entry);

   return entry;
}


array_deref_range *
ir_array_refcount_visitor::get_array_deref()
{
   if ((num_derefs + 1) * sizeof(array_deref_range) > derefs_size) {
      void *ptr = reralloc_size(mem_ctx, derefs, derefs_size + 4096);

      if (ptr == NULL)
         return NULL;

      derefs_size += 4096;
      derefs = (array_deref_range *)ptr;
   }

   array_deref_range *d = &derefs[num_derefs];
   num_derefs++;

   return d;
}

ir_visitor_status
ir_array_refcount_visitor::visit_enter(ir_dereference_array *ir)
{
   /* It could also be a vector or a matrix.  Individual elements of vectors
    * are natrices are not tracked, so bail.
    */
   if (!ir->array->type->is_array())
      return visit_continue;

   /* If this array dereference is a child of an array dereference that was
    * already visited, just continue on.  Otherwise, for an arrays-of-arrays
    * dereference like x[1][2][3][4], we'd process the [1][2][3][4] sequence,
    * the [1][2][3] sequence, the [1][2] sequence, and the [1] sequence.  This
    * ensures that we only process the full sequence.
    */
   if (last_array_deref && last_array_deref->array == ir) {
      last_array_deref = ir;
      return visit_continue;
   }

   last_array_deref = ir;

   num_derefs = 0;

   ir_rvalue *rv = ir;
   while (rv->ir_type == ir_type_dereference_array) {
      ir_dereference_array *const deref = rv->as_dereference_array();

      assert(deref != NULL);
      assert(deref->array->type->is_array());

      ir_rvalue *const array = deref->array;
      const ir_constant *const idx = deref->array_index->as_constant();
      array_deref_range *const dr = get_array_deref();

      dr->size = array->type->array_size();

      if (idx != NULL) {
         dr->index = idx->get_int_component(0);
      } else {
         /* An unsized array can occur at the end of an SSBO.  We can't track
          * accesses to such an array, so bail.
          */
         if (array->type->array_size() == 0)
            return visit_continue;

         dr->index = dr->size;
      }

      rv = array;
   }

   ir_dereference_variable *const var_deref = rv->as_dereference_variable();

   /* If the array being dereferenced is not a variable, bail.  At the very
    * least, ir_constant and ir_dereference_record are possible.
    */
   if (var_deref == NULL)
      return visit_continue;

   ir_array_refcount_entry *const entry =
      this->get_variable_entry(var_deref->var);

   if (entry == NULL)
      return visit_stop;

   link_util_mark_array_elements_referenced(derefs, num_derefs,
                                            entry->array_depth,
                                            entry->bits);

   return visit_continue;
}


ir_visitor_status
ir_array_refcount_visitor::visit(ir_dereference_variable *ir)
{
   ir_variable *const var = ir->variable_referenced();
   ir_array_refcount_entry *entry = this->get_variable_entry(var);

   entry->is_referenced = true;

   return visit_continue;
}


ir_visitor_status
ir_array_refcount_visitor::visit_enter(ir_function_signature *ir)
{
   /* We don't want to descend into the function parameters and
    * dead-code eliminate them, so just accept the body here.
    */
   visit_list_elements(this, &ir->body);
   return visit_continue_with_parent;
}
