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
 * \file lower_vertex_id.cpp
 *
 * There exists hardware, such as i965, that does not implement the OpenGL
 * semantic for gl_VertexID.  Instead, that hardware does not include the
 * value of basevertex in the gl_VertexID value.  To implement the OpenGL
 * semantic, we'll have to convert gl_Vertex_ID to
 * gl_VertexIDMESA+gl_BaseVertexMESA.
 */

#include "glsl_symbol_table.h"
#include "ir_hierarchical_visitor.h"
#include "ir.h"
#include "ir_builder.h"
#include "linker.h"
#include "program/prog_statevars.h"
#include "builtin_functions.h"
#include "main/mtypes.h"

namespace {

class lower_vertex_id_visitor : public ir_hierarchical_visitor {
public:
   explicit lower_vertex_id_visitor(ir_function_signature *main_sig,
                                    exec_list *ir_list)
      : progress(false), VertexID(NULL), gl_VertexID(NULL),
        gl_BaseVertex(NULL), main_sig(main_sig), ir_list(ir_list)
   {
      foreach_in_list(ir_instruction, ir, ir_list) {
         ir_variable *const var = ir->as_variable();

         if (var != NULL && var->data.mode == ir_var_system_value &&
             var->data.location == SYSTEM_VALUE_BASE_VERTEX) {
            gl_BaseVertex = var;
            break;
         }
      }
   }

   virtual ir_visitor_status visit(ir_dereference_variable *);

   bool progress;

private:
   ir_variable *VertexID;
   ir_variable *gl_VertexID;
   ir_variable *gl_BaseVertex;

   ir_function_signature *main_sig;
   exec_list *ir_list;
};

} /* anonymous namespace */

ir_visitor_status
lower_vertex_id_visitor::visit(ir_dereference_variable *ir)
{
   if (ir->var->data.mode != ir_var_system_value ||
       ir->var->data.location != SYSTEM_VALUE_VERTEX_ID)
      return visit_continue;

   if (VertexID == NULL) {
      const glsl_type *const int_t = glsl_type::int_type;
      void *const mem_ctx = ralloc_parent(ir);

      VertexID = new(mem_ctx) ir_variable(int_t, "__VertexID",
                                          ir_var_temporary);
      ir_list->push_head(VertexID);

      gl_VertexID = new(mem_ctx) ir_variable(int_t, "gl_VertexIDMESA",
                                             ir_var_system_value);
      gl_VertexID->data.how_declared = ir_var_declared_implicitly;
      gl_VertexID->data.read_only = true;
      gl_VertexID->data.location = SYSTEM_VALUE_VERTEX_ID_ZERO_BASE;
      gl_VertexID->data.explicit_location = true;
      gl_VertexID->data.explicit_index = 0;
      ir_list->push_head(gl_VertexID);

      if (gl_BaseVertex == NULL) {
         gl_BaseVertex = new(mem_ctx) ir_variable(int_t, "gl_BaseVertex",
                                                  ir_var_system_value);
         gl_BaseVertex->data.how_declared = ir_var_hidden;
         gl_BaseVertex->data.read_only = true;
         gl_BaseVertex->data.location = SYSTEM_VALUE_BASE_VERTEX;
         gl_BaseVertex->data.explicit_location = true;
         gl_BaseVertex->data.explicit_index = 0;
         ir_list->push_head(gl_BaseVertex);
      }

      ir_instruction *const inst =
         ir_builder::assign(VertexID,
                            ir_builder::add(gl_VertexID, gl_BaseVertex));

      main_sig->body.push_head(inst);
   }

   ir->var = VertexID;
   progress = true;

   return visit_continue;
}

bool
lower_vertex_id(gl_linked_shader *shader)
{
   /* gl_VertexID only exists in the vertex shader.
    */
   if (shader->Stage != MESA_SHADER_VERTEX)
      return false;

   ir_function_signature *const main_sig =
      _mesa_get_main_function_signature(shader->symbols);
   if (main_sig == NULL) {
      assert(main_sig != NULL);
      return false;
   }

   lower_vertex_id_visitor v(main_sig, shader->ir);

   v.run(shader->ir);

   return v.progress;
}
