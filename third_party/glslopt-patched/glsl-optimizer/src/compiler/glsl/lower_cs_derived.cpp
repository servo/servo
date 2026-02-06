/*
 * Copyright © 2017 Ilia Mirkin
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
 * \file lower_cs_derived.cpp
 *
 * For hardware that does not support the gl_GlobalInvocationID and
 * gl_LocalInvocationIndex system values, replace them with fresh
 * globals. Note that we can't rely on gl_WorkGroupSize or
 * gl_LocalGroupSizeARB being available, since they may only have been defined
 * in a non-main shader.
 *
 * [ This can happen if only a secondary shader has the layout(local_size_*)
 *   declaration. ]
 *
 * This is meant to be run post-linking.
 */

#include "glsl_symbol_table.h"
#include "ir_hierarchical_visitor.h"
#include "ir.h"
#include "ir_builder.h"
#include "linker.h"
#include "program/prog_statevars.h"
#include "builtin_functions.h"
#include "main/mtypes.h"

using namespace ir_builder;

namespace {

class lower_cs_derived_visitor : public ir_hierarchical_visitor {
public:
   explicit lower_cs_derived_visitor(gl_linked_shader *shader)
      : progress(false),
        shader(shader),
        local_size_variable(shader->Program->info.cs.local_size_variable),
        gl_WorkGroupSize(NULL),
        gl_WorkGroupID(NULL),
        gl_LocalInvocationID(NULL),
        gl_GlobalInvocationID(NULL),
        gl_LocalInvocationIndex(NULL)
   {
      main_sig = _mesa_get_main_function_signature(shader->symbols);
      assert(main_sig);
   }

   virtual ir_visitor_status visit(ir_dereference_variable *);

   ir_variable *add_system_value(
         int slot, const glsl_type *type, const char *name);
   void find_sysvals();
   void make_gl_GlobalInvocationID();
   void make_gl_LocalInvocationIndex();

   bool progress;

private:
   gl_linked_shader *shader;
   bool local_size_variable;
   ir_function_signature *main_sig;

   ir_rvalue *gl_WorkGroupSize;
   ir_variable *gl_WorkGroupID;
   ir_variable *gl_LocalInvocationID;

   ir_variable *gl_GlobalInvocationID;
   ir_variable *gl_LocalInvocationIndex;
};

} /* anonymous namespace */

ir_variable *
lower_cs_derived_visitor::add_system_value(
      int slot, const glsl_type *type, const char *name)
{
   ir_variable *var = new(shader) ir_variable(type, name, ir_var_system_value);
   var->data.how_declared = ir_var_declared_implicitly;
   var->data.read_only = true;
   var->data.location = slot;
   var->data.explicit_location = true;
   var->data.explicit_index = 0;
   shader->ir->push_head(var);

   return var;
}

void
lower_cs_derived_visitor::find_sysvals()
{
   if (gl_WorkGroupSize != NULL)
      return;

   ir_variable *WorkGroupSize;
   if (local_size_variable)
      WorkGroupSize = shader->symbols->get_variable("gl_LocalGroupSizeARB");
   else
      WorkGroupSize = shader->symbols->get_variable("gl_WorkGroupSize");
   if (WorkGroupSize)
      gl_WorkGroupSize = new(shader) ir_dereference_variable(WorkGroupSize);
   gl_WorkGroupID = shader->symbols->get_variable("gl_WorkGroupID");
   gl_LocalInvocationID = shader->symbols->get_variable("gl_LocalInvocationID");

   /*
    * These may be missing due to either dead code elimination, or, in the
    * case of the group size, due to the layout being declared in a non-main
    * shader. Re-create them.
    */

   if (!gl_WorkGroupID)
      gl_WorkGroupID = add_system_value(
            SYSTEM_VALUE_WORK_GROUP_ID, glsl_type::uvec3_type, "gl_WorkGroupID");
   if (!gl_LocalInvocationID)
      gl_LocalInvocationID = add_system_value(
            SYSTEM_VALUE_LOCAL_INVOCATION_ID, glsl_type::uvec3_type,
            "gl_LocalInvocationID");
   if (!WorkGroupSize) {
      if (local_size_variable) {
         gl_WorkGroupSize = new(shader) ir_dereference_variable(
               add_system_value(
                     SYSTEM_VALUE_LOCAL_GROUP_SIZE, glsl_type::uvec3_type,
                     "gl_LocalGroupSizeARB"));
      } else {
         ir_constant_data data;
         memset(&data, 0, sizeof(data));
         for (int i = 0; i < 3; i++)
            data.u[i] = shader->Program->info.cs.local_size[i];
         gl_WorkGroupSize = new(shader) ir_constant(glsl_type::uvec3_type, &data);
      }
   }
}

void
lower_cs_derived_visitor::make_gl_GlobalInvocationID()
{
   if (gl_GlobalInvocationID != NULL)
      return;

   find_sysvals();

   /* gl_GlobalInvocationID =
    *    gl_WorkGroupID * gl_WorkGroupSize + gl_LocalInvocationID
    */
   gl_GlobalInvocationID = new(shader) ir_variable(
         glsl_type::uvec3_type, "__GlobalInvocationID", ir_var_temporary);
   shader->ir->push_head(gl_GlobalInvocationID);

   ir_instruction *inst =
      assign(gl_GlobalInvocationID,
             add(mul(gl_WorkGroupID, gl_WorkGroupSize->clone(shader, NULL)),
                 gl_LocalInvocationID));
   main_sig->body.push_head(inst);
}

void
lower_cs_derived_visitor::make_gl_LocalInvocationIndex()
{
   if (gl_LocalInvocationIndex != NULL)
      return;

   find_sysvals();

   /* gl_LocalInvocationIndex =
    *    gl_LocalInvocationID.z * gl_WorkGroupSize.x * gl_WorkGroupSize.y +
    *    gl_LocalInvocationID.y * gl_WorkGroupSize.x +
    *    gl_LocalInvocationID.x;
    */
   gl_LocalInvocationIndex = new(shader)
      ir_variable(glsl_type::uint_type, "__LocalInvocationIndex", ir_var_temporary);
   shader->ir->push_head(gl_LocalInvocationIndex);

   ir_expression *index_z =
      mul(mul(swizzle_z(gl_LocalInvocationID), swizzle_x(gl_WorkGroupSize->clone(shader, NULL))),
          swizzle_y(gl_WorkGroupSize->clone(shader, NULL)));
   ir_expression *index_y =
      mul(swizzle_y(gl_LocalInvocationID), swizzle_x(gl_WorkGroupSize->clone(shader, NULL)));
   ir_expression *index_y_plus_z = add(index_y, index_z);
   operand index_x(swizzle_x(gl_LocalInvocationID));
   ir_expression *index_x_plus_y_plus_z = add(index_y_plus_z, index_x);
   ir_instruction *inst =
      assign(gl_LocalInvocationIndex, index_x_plus_y_plus_z);
   main_sig->body.push_head(inst);
}

ir_visitor_status
lower_cs_derived_visitor::visit(ir_dereference_variable *ir)
{
   if (ir->var->data.mode == ir_var_system_value &&
       ir->var->data.location == SYSTEM_VALUE_GLOBAL_INVOCATION_ID) {
      make_gl_GlobalInvocationID();
      ir->var = gl_GlobalInvocationID;
      progress = true;
   }

   if (ir->var->data.mode == ir_var_system_value &&
       ir->var->data.location == SYSTEM_VALUE_LOCAL_INVOCATION_INDEX) {
      make_gl_LocalInvocationIndex();
      ir->var = gl_LocalInvocationIndex;
      progress = true;
   }

   return visit_continue;
}

bool
lower_cs_derived(gl_linked_shader *shader)
{
   if (shader->Stage != MESA_SHADER_COMPUTE)
      return false;

   lower_cs_derived_visitor v(shader);
   v.run(shader->ir);

   return v.progress;
}
