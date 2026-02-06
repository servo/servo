/*
 * Copyright (c) 2013 Intel Corporation
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
 * \file lower_named_interface_blocks.cpp
 *
 * This lowering pass converts all interface blocks with instance names
 * into interface blocks without an instance name.
 *
 * For example, the following shader:
 *
 *   out block {
 *     float block_var;
 *   } inst_name;
 *
 *   main()
 *   {
 *     inst_name.block_var = 0.0;
 *   }
 *
 * Is rewritten to:
 *
 *   out block {
 *     float block_var;
 *   };
 *
 *   main()
 *   {
 *     block_var = 0.0;
 *   }
 *
 * This takes place after the shader code has already been verified with
 * the interface name in place.
 *
 * The linking phase will use the interface block name rather than the
 * interface's instance name when linking interfaces.
 *
 * This modification to the ir allows our currently existing dead code
 * elimination to work with interface blocks without changes.
 */

#include "glsl_symbol_table.h"
#include "ir.h"
#include "ir_optimization.h"
#include "ir_rvalue_visitor.h"
#include "util/hash_table.h"
#include "main/mtypes.h"

static const glsl_type *
process_array_type(const glsl_type *type, unsigned idx)
{
   const glsl_type *element_type = type->fields.array;
   if (element_type->is_array()) {
      const glsl_type *new_array_type = process_array_type(element_type, idx);
      return glsl_type::get_array_instance(new_array_type, type->length);
   } else {
      return glsl_type::get_array_instance(
         element_type->fields.structure[idx].type, type->length);
   }
}

static ir_rvalue *
process_array_ir(void * const mem_ctx,
                 ir_dereference_array *deref_array_prev,
                 ir_rvalue *deref_var)
{
   ir_dereference_array *deref_array =
      deref_array_prev->array->as_dereference_array();

   if (deref_array == NULL) {
      return new(mem_ctx) ir_dereference_array(deref_var,
                                               deref_array_prev->array_index);
   } else {
      deref_array = (ir_dereference_array *) process_array_ir(mem_ctx,
                                                              deref_array,
                                                              deref_var);
      return new(mem_ctx) ir_dereference_array(deref_array,
                                               deref_array_prev->array_index);
   }
}

namespace {

class flatten_named_interface_blocks_declarations : public ir_rvalue_visitor
{
public:
   void * const mem_ctx;
   hash_table *interface_namespace;

   flatten_named_interface_blocks_declarations(void *mem_ctx)
      : mem_ctx(mem_ctx),
        interface_namespace(NULL)
   {
   }

   void run(exec_list *instructions);

   virtual ir_visitor_status visit_leave(ir_assignment *);
   virtual ir_visitor_status visit_leave(ir_expression *);
   virtual void handle_rvalue(ir_rvalue **rvalue);
};

} /* anonymous namespace */

void
flatten_named_interface_blocks_declarations::run(exec_list *instructions)
{
   interface_namespace = _mesa_hash_table_create(NULL, _mesa_hash_string,
                                                 _mesa_key_string_equal);

   /* First pass: adjust instance block variables with an instance name
    * to not have an instance name.
    *
    * The interface block variables are stored in the interface_namespace
    * hash table so they can be used in the second pass.
    */
   foreach_in_list_safe(ir_instruction, node, instructions) {
      ir_variable *var = node->as_variable();
      if (!var || !var->is_interface_instance())
         continue;

      /* It should be possible to handle uniforms during this pass,
       * but, this will require changes to the other uniform block
       * support code.
       */
      if (var->data.mode == ir_var_uniform ||
          var->data.mode == ir_var_shader_storage)
         continue;

      const glsl_type * iface_t = var->type->without_array();
      exec_node *insert_pos = var;

      assert (iface_t->is_interface());

      for (unsigned i = 0; i < iface_t->length; i++) {
         const char * field_name = iface_t->fields.structure[i].name;
         char *iface_field_name =
            ralloc_asprintf(mem_ctx, "%s %s.%s.%s",
                            var->data.mode == ir_var_shader_in ? "in" : "out",
                            iface_t->name, var->name, field_name);

         hash_entry *entry = _mesa_hash_table_search(interface_namespace,
                                                     iface_field_name);
         ir_variable *found_var = entry ? (ir_variable *) entry->data : NULL;
         if (!found_var) {
            ir_variable *new_var;
            char *var_name =
               ralloc_strdup(mem_ctx, iface_t->fields.structure[i].name);
            if (!var->type->is_array()) {
               new_var =
                  new(mem_ctx) ir_variable(iface_t->fields.structure[i].type,
                                           var_name,
                                           (ir_variable_mode) var->data.mode);
            } else {
               const glsl_type *new_array_type =
                  process_array_type(var->type, i);
               new_var =
                  new(mem_ctx) ir_variable(new_array_type,
                                           var_name,
                                           (ir_variable_mode) var->data.mode);
            }
            new_var->data.location = iface_t->fields.structure[i].location;
            new_var->data.explicit_location = (new_var->data.location >= 0);
            new_var->data.offset = iface_t->fields.structure[i].offset;
            new_var->data.explicit_xfb_offset =
               (iface_t->fields.structure[i].offset >= 0);
            new_var->data.xfb_buffer =
               iface_t->fields.structure[i].xfb_buffer;
            new_var->data.explicit_xfb_buffer =
               iface_t->fields.structure[i].explicit_xfb_buffer;
            new_var->data.interpolation =
               iface_t->fields.structure[i].interpolation;
            new_var->data.centroid = iface_t->fields.structure[i].centroid;
            new_var->data.sample = iface_t->fields.structure[i].sample;
            new_var->data.patch = iface_t->fields.structure[i].patch;
            new_var->data.stream = var->data.stream;
            new_var->data.how_declared = var->data.how_declared;
            new_var->data.from_named_ifc_block = 1;

            new_var->init_interface_type(var->type);
            _mesa_hash_table_insert(interface_namespace, iface_field_name,
                                    new_var);
            insert_pos->insert_after(new_var);
            insert_pos = new_var;
         }
      }
      var->remove();
   }

   /* Second pass: visit all ir_dereference_record instances, and if they
    * reference an interface block, then flatten the refererence out.
    */
   visit_list_elements(this, instructions);
   _mesa_hash_table_destroy(interface_namespace, NULL);
   interface_namespace = NULL;
}

ir_visitor_status
flatten_named_interface_blocks_declarations::visit_leave(ir_assignment *ir)
{
   ir_dereference_record *lhs_rec = ir->lhs->as_dereference_record();

   ir_variable *lhs_var =  ir->lhs->variable_referenced();
   if (lhs_var && lhs_var->get_interface_type()) {
      lhs_var->data.assigned = 1;
   }

   if (lhs_rec) {
      ir_rvalue *lhs_rec_tmp = lhs_rec;
      handle_rvalue(&lhs_rec_tmp);
      if (lhs_rec_tmp != lhs_rec) {
         ir->set_lhs(lhs_rec_tmp);
      }

      ir_variable *lhs_var =  lhs_rec_tmp->variable_referenced();
      if (lhs_var) {
         lhs_var->data.assigned = 1;
      }
   }
   return rvalue_visit(ir);
}

ir_visitor_status
flatten_named_interface_blocks_declarations::visit_leave(ir_expression *ir)
{
   ir_visitor_status status = rvalue_visit(ir);

   if (ir->operation == ir_unop_interpolate_at_centroid ||
       ir->operation == ir_binop_interpolate_at_offset ||
       ir->operation == ir_binop_interpolate_at_sample) {
      const ir_rvalue *val = ir->operands[0];

      /* This disables varying packing for this input. */
      val->variable_referenced()->data.must_be_shader_input = 1;
   }

   return status;
}

void
flatten_named_interface_blocks_declarations::handle_rvalue(ir_rvalue **rvalue)
{
   if (*rvalue == NULL)
      return;

   ir_dereference_record *ir = (*rvalue)->as_dereference_record();
   if (ir == NULL)
      return;

   ir_variable *var = ir->variable_referenced();
   if (var == NULL)
      return;

   if (!var->is_interface_instance())
      return;

   /* It should be possible to handle uniforms during this pass,
    * but, this will require changes to the other uniform block
    * support code.
    */
   if (var->data.mode == ir_var_uniform || var->data.mode == ir_var_shader_storage)
      return;

   if (var->get_interface_type() != NULL) {
      char *iface_field_name =
         ralloc_asprintf(mem_ctx, "%s %s.%s.%s",
                         var->data.mode == ir_var_shader_in ? "in" : "out",
                         var->get_interface_type()->name,
                         var->name,
                         ir->record->type->fields.structure[ir->field_idx].name);

      /* Find the variable in the set of flattened interface blocks */
      hash_entry *entry = _mesa_hash_table_search(interface_namespace,
                                                  iface_field_name);
      assert(entry);
      ir_variable *found_var = (ir_variable *) entry->data;

      ir_dereference_variable *deref_var =
         new(mem_ctx) ir_dereference_variable(found_var);

      ir_dereference_array *deref_array =
         ir->record->as_dereference_array();
      if (deref_array != NULL) {
         *rvalue = process_array_ir(mem_ctx, deref_array,
                                    (ir_rvalue *)deref_var);
      } else {
         *rvalue = deref_var;
      }
   }
}

void
lower_named_interface_blocks(void *mem_ctx, gl_linked_shader *shader)
{
   flatten_named_interface_blocks_declarations v_decl(mem_ctx);
   v_decl.run(shader->ir);
}

