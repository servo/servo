/*
 * Copyright © 2010 Intel Corporation
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

#include "glsl_symbol_table.h"
#include "glsl_parser_extras.h"
#include "ir.h"
#include "program.h"
#include "util/set.h"
#include "util/hash_table.h"
#include "linker.h"
#include "main/mtypes.h"

static ir_function_signature *
find_matching_signature(const char *name, const exec_list *actual_parameters,
                        glsl_symbol_table *symbols);

namespace {

class call_link_visitor : public ir_hierarchical_visitor {
public:
   call_link_visitor(gl_shader_program *prog, gl_linked_shader *linked,
		     gl_shader **shader_list, unsigned num_shaders)
   {
      this->prog = prog;
      this->shader_list = shader_list;
      this->num_shaders = num_shaders;
      this->success = true;
      this->linked = linked;

      this->locals = _mesa_pointer_set_create(NULL);
   }

   ~call_link_visitor()
   {
      _mesa_set_destroy(this->locals, NULL);
   }

   virtual ir_visitor_status visit(ir_variable *ir)
   {
      _mesa_set_add(locals, ir);
      return visit_continue;
   }

   virtual ir_visitor_status visit_enter(ir_call *ir)
   {
      /* If ir is an ir_call from a function that was imported from another
       * shader callee will point to an ir_function_signature in the original
       * shader.  In this case the function signature MUST NOT BE MODIFIED.
       * Doing so will modify the original shader.  This may prevent that
       * shader from being linkable in other programs.
       */
      const ir_function_signature *const callee = ir->callee;
      assert(callee != NULL);
      const char *const name = callee->function_name();

      /* We don't actually need to find intrinsics; they're not real */
      if (callee->is_intrinsic())
         return visit_continue;

      /* Determine if the requested function signature already exists in the
       * final linked shader.  If it does, use it as the target of the call.
       */
      ir_function_signature *sig =
         find_matching_signature(name, &callee->parameters, linked->symbols);
      if (sig != NULL) {
	 ir->callee = sig;
	 return visit_continue;
      }

      /* Try to find the signature in one of the other shaders that is being
       * linked.  If it's not found there, return an error.
       */
      for (unsigned i = 0; i < num_shaders; i++) {
         sig = find_matching_signature(name, &ir->actual_parameters,
                                       shader_list[i]->symbols);
         if (sig)
            break;
      }

      if (sig == NULL) {
	 /* FINISHME: Log the full signature of unresolved function.
	  */
	 linker_error(this->prog, "unresolved reference to function `%s'\n",
		      name);
	 this->success = false;
	 return visit_stop;
      }

      /* Find the prototype information in the linked shader.  Generate any
       * details that may be missing.
       */
      ir_function *f = linked->symbols->get_function(name);
      if (f == NULL) {
	 f = new(linked) ir_function(name);

	 /* Add the new function to the linked IR.  Put it at the end
          * so that it comes after any global variable declarations
          * that it refers to.
	  */
	 linked->symbols->add_function(f);
	 linked->ir->push_tail(f);
      }

      ir_function_signature *linked_sig =
	 f->exact_matching_signature(NULL, &callee->parameters);
      if (linked_sig == NULL) {
	 linked_sig = new(linked) ir_function_signature(callee->return_type);
	 f->add_signature(linked_sig);
      }

      /* At this point linked_sig and called may be the same.  If ir is an
       * ir_call from linked then linked_sig and callee will be
       * ir_function_signatures that have no definitions (is_defined is false).
       */
      assert(!linked_sig->is_defined);
      assert(linked_sig->body.is_empty());

      /* Create an in-place clone of the function definition.  This multistep
       * process introduces some complexity here, but it has some advantages.
       * The parameter list and the and function body are cloned separately.
       * The clone of the parameter list is used to prime the hashtable used
       * to replace variable references in the cloned body.
       *
       * The big advantage is that the ir_function_signature does not change.
       * This means that we don't have to process the rest of the IR tree to
       * patch ir_call nodes.  In addition, there is no way to remove or
       * replace signature stored in a function.  One could easily be added,
       * but this avoids the need.
       */
      struct hash_table *ht = _mesa_pointer_hash_table_create(NULL);

      exec_list formal_parameters;
      foreach_in_list(const ir_instruction, original, &sig->parameters) {
         assert(const_cast<ir_instruction *>(original)->as_variable());

         ir_instruction *copy = original->clone(linked, ht);
         formal_parameters.push_tail(copy);
      }

      linked_sig->replace_parameters(&formal_parameters);

      linked_sig->intrinsic_id = sig->intrinsic_id;

      if (sig->is_defined) {
         foreach_in_list(const ir_instruction, original, &sig->body) {
            ir_instruction *copy = original->clone(linked, ht);
            linked_sig->body.push_tail(copy);
         }

         linked_sig->is_defined = true;
      }

      _mesa_hash_table_destroy(ht, NULL);

      /* Patch references inside the function to things outside the function
       * (i.e., function calls and global variables).
       */
      linked_sig->accept(this);

      ir->callee = linked_sig;

      return visit_continue;
   }

   virtual ir_visitor_status visit_leave(ir_call *ir)
   {
      /* Traverse list of function parameters, and for array parameters
       * propagate max_array_access. Otherwise arrays that are only referenced
       * from inside functions via function parameters will be incorrectly
       * optimized. This will lead to incorrect code being generated (or worse).
       * Do it when leaving the node so the children would propagate their
       * array accesses first.
       */

      const exec_node *formal_param_node = ir->callee->parameters.get_head();
      if (formal_param_node) {
         const exec_node *actual_param_node = ir->actual_parameters.get_head();
         while (!actual_param_node->is_tail_sentinel()) {
            ir_variable *formal_param = (ir_variable *) formal_param_node;
            ir_rvalue *actual_param = (ir_rvalue *) actual_param_node;

            formal_param_node = formal_param_node->get_next();
            actual_param_node = actual_param_node->get_next();

            if (formal_param->type->is_array()) {
               ir_dereference_variable *deref = actual_param->as_dereference_variable();
               if (deref && deref->var && deref->var->type->is_array()) {
                  deref->var->data.max_array_access =
                     MAX2(formal_param->data.max_array_access,
                         deref->var->data.max_array_access);
               }
            }
         }
      }
      return visit_continue;
   }

   virtual ir_visitor_status visit(ir_dereference_variable *ir)
   {
      if (_mesa_set_search(locals, ir->var) == NULL) {
	 /* The non-function variable must be a global, so try to find the
	  * variable in the shader's symbol table.  If the variable is not
	  * found, then it's a global that *MUST* be defined in the original
	  * shader.
	  */
	 ir_variable *var = linked->symbols->get_variable(ir->var->name);
	 if (var == NULL) {
	    /* Clone the ir_variable that the dereference already has and add
	     * it to the linked shader.
	     */
	    var = ir->var->clone(linked, NULL);
	    linked->symbols->add_variable(var);
	    linked->ir->push_head(var);
	 } else {
            if (var->type->is_array()) {
               /* It is possible to have a global array declared in multiple
                * shaders without a size.  The array is implicitly sized by
                * the maximal access to it in *any* shader.  Because of this,
                * we need to track the maximal access to the array as linking
                * pulls more functions in that access the array.
                */
               var->data.max_array_access =
                  MAX2(var->data.max_array_access,
                       ir->var->data.max_array_access);

               if (var->type->length == 0 && ir->var->type->length != 0)
                  var->type = ir->var->type;
            }
            if (var->is_interface_instance()) {
               /* Similarly, we need implicit sizes of arrays within interface
                * blocks to be sized by the maximal access in *any* shader.
                */
               int *const linked_max_ifc_array_access =
                  var->get_max_ifc_array_access();
               int *const ir_max_ifc_array_access =
                  ir->var->get_max_ifc_array_access();

               assert(linked_max_ifc_array_access != NULL);
               assert(ir_max_ifc_array_access != NULL);

               for (unsigned i = 0; i < var->get_interface_type()->length;
                    i++) {
                  linked_max_ifc_array_access[i] =
                     MAX2(linked_max_ifc_array_access[i],
                          ir_max_ifc_array_access[i]);
               }
            }
	 }

	 ir->var = var;
      }

      return visit_continue;
   }

   /** Was function linking successful? */
   bool success;

private:
   /**
    * Shader program being linked
    *
    * This is only used for logging error messages.
    */
   gl_shader_program *prog;

   /** List of shaders available for linking. */
   gl_shader **shader_list;

   /** Number of shaders available for linking. */
   unsigned num_shaders;

   /**
    * Final linked shader
    *
    * This is used two ways.  It is used to find global variables in the
    * linked shader that are accessed by the function.  It is also used to add
    * global variables from the shader where the function originated.
    */
   gl_linked_shader *linked;

   /**
    * Table of variables local to the function.
    */
   set *locals;
};

} /* anonymous namespace */

/**
 * Searches a list of shaders for a particular function definition
 */
ir_function_signature *
find_matching_signature(const char *name, const exec_list *actual_parameters,
                        glsl_symbol_table *symbols)
{
   ir_function *const f = symbols->get_function(name);

   if (f) {
      ir_function_signature *sig =
         f->matching_signature(NULL, actual_parameters, false);

      if (sig && (sig->is_defined || sig->is_intrinsic()))
         return sig;
   }

   return NULL;
}


bool
link_function_calls(gl_shader_program *prog, gl_linked_shader *main,
		    gl_shader **shader_list, unsigned num_shaders)
{
   call_link_visitor v(prog, main, shader_list, num_shaders);

   v.run(main->ir);
   return v.success;
}
