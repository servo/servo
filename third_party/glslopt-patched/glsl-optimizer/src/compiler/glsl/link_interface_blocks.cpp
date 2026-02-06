/*
 * Copyright © 2013 Intel Corporation
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
 * \file link_interface_blocks.cpp
 * Linker support for GLSL's interface blocks.
 */

#include "ir.h"
#include "glsl_symbol_table.h"
#include "linker.h"
#include "main/macros.h"
#include "main/mtypes.h"
#include "util/hash_table.h"
#include "util/u_string.h"


namespace {

/**
 * Return true if interface members mismatch and its not allowed by GLSL.
 */
static bool
interstage_member_mismatch(struct gl_shader_program *prog,
                           const glsl_type *c, const glsl_type *p) {

   if (c->length != p->length)
      return true;

   for (unsigned i = 0; i < c->length; i++) {
      if (c->fields.structure[i].type != p->fields.structure[i].type)
         return true;
      if (strcmp(c->fields.structure[i].name,
                 p->fields.structure[i].name) != 0)
         return true;
      if (c->fields.structure[i].location !=
          p->fields.structure[i].location)
         return true;
      if (c->fields.structure[i].patch !=
          p->fields.structure[i].patch)
         return true;

      /* From Section 4.5 (Interpolation Qualifiers) of the GLSL 4.40 spec:
       *
       *    "It is a link-time error if, within the same stage, the
       *    interpolation qualifiers of variables of the same name do not
       *    match."
       */
      if (prog->IsES || prog->data->Version < 440)
         if (c->fields.structure[i].interpolation !=
             p->fields.structure[i].interpolation)
            return true;

      /* From Section 4.3.4 (Input Variables) of the GLSL ES 3.0 spec:
       *
       *    "The output of the vertex shader and the input of the fragment
       *    shader form an interface.  For this interface, vertex shader
       *    output variables and fragment shader input variables of the same
       *    name must match in type and qualification (other than precision
       *    and out matching to in).
       *
       * The table in Section 9.2.1 Linked Shaders of the GLSL ES 3.1 spec
       * says that centroid no longer needs to match for varyings.
       *
       * The table in Section 9.2.1 Linked Shaders of the GLSL ES 3.2 spec
       * says that sample need not match for varyings.
       */
      if (!prog->IsES || prog->data->Version < 310)
         if (c->fields.structure[i].centroid !=
             p->fields.structure[i].centroid)
            return true;
      if (!prog->IsES)
         if (c->fields.structure[i].sample !=
             p->fields.structure[i].sample)
            return true;
   }

   return false;
}

/**
 * Check if two interfaces match, according to intrastage interface matching
 * rules.  If they do, and the first interface uses an unsized array, it will
 * be updated to reflect the array size declared in the second interface.
 */
bool
intrastage_match(ir_variable *a,
                 ir_variable *b,
                 struct gl_shader_program *prog,
                 bool match_precision)
{
   /* Types must match. */
   if (a->get_interface_type() != b->get_interface_type()) {
      /* Exception: if both the interface blocks are implicitly declared,
       * don't force their types to match.  They might mismatch due to the two
       * shaders using different GLSL versions, and that's ok.
       */
      if ((a->data.how_declared != ir_var_declared_implicitly ||
           b->data.how_declared != ir_var_declared_implicitly) &&
          (!prog->IsES ||
           interstage_member_mismatch(prog, a->get_interface_type(),
                                      b->get_interface_type())))
         return false;
   }

   /* Presence/absence of interface names must match. */
   if (a->is_interface_instance() != b->is_interface_instance())
      return false;

   /* For uniforms, instance names need not match.  For shader ins/outs,
    * it's not clear from the spec whether they need to match, but
    * Mesa's implementation relies on them matching.
    */
   if (a->is_interface_instance() && b->data.mode != ir_var_uniform &&
       b->data.mode != ir_var_shader_storage &&
       strcmp(a->name, b->name) != 0) {
      return false;
   }

   bool type_match = (match_precision ?
                      a->type == b->type :
                      a->type->compare_no_precision(b->type));

   /* If a block is an array then it must match across the shader.
    * Unsized arrays are also processed and matched agaist sized arrays.
    */
   if (!type_match && (b->type->is_array() || a->type->is_array()) &&
       (b->is_interface_instance() || a->is_interface_instance()) &&
       !validate_intrastage_arrays(prog, b, a, match_precision))
      return false;

   return true;
}

/**
 * Check if two interfaces match, according to interstage (in/out) interface
 * matching rules.
 *
 * If \c extra_array_level is true, the consumer interface is required to be
 * an array and the producer interface is required to be a non-array.
 * This is used for tessellation control and geometry shader consumers.
 */
static bool
interstage_match(struct gl_shader_program *prog, ir_variable *producer,
                 ir_variable *consumer, bool extra_array_level)
{
   /* Types must match. */
   if (consumer->get_interface_type() != producer->get_interface_type()) {
      /* Exception: if both the interface blocks are implicitly declared,
       * don't force their types to match.  They might mismatch due to the two
       * shaders using different GLSL versions, and that's ok.
       *
       * Also we store some member information such as interpolation in
       * glsl_type that doesn't always have to match across shader stages.
       * Therefore we make a pass over the members glsl_struct_field to make
       * sure we don't reject shaders where fields don't need to match.
       */
      if ((consumer->data.how_declared != ir_var_declared_implicitly ||
           producer->data.how_declared != ir_var_declared_implicitly) &&
          interstage_member_mismatch(prog, consumer->get_interface_type(),
                                     producer->get_interface_type()))
         return false;
   }

   /* Ignore outermost array if geom shader */
   const glsl_type *consumer_instance_type;
   if (extra_array_level) {
      consumer_instance_type = consumer->type->fields.array;
   } else {
      consumer_instance_type = consumer->type;
   }

   /* If a block is an array then it must match across shaders.
    * Since unsized arrays have been ruled out, we can check this by just
    * making sure the types are equal.
    */
   if ((consumer->is_interface_instance() &&
        consumer_instance_type->is_array()) ||
       (producer->is_interface_instance() &&
        producer->type->is_array())) {
      if (consumer_instance_type != producer->type)
         return false;
   }

   return true;
}


/**
 * This class keeps track of a mapping from an interface block name to the
 * necessary information about that interface block to determine whether to
 * generate a link error.
 *
 * Note: this class is expected to be short lived, so it doesn't make copies
 * of the strings it references; it simply borrows the pointers from the
 * ir_variable class.
 */
class interface_block_definitions
{
public:
   interface_block_definitions()
      : mem_ctx(ralloc_context(NULL)),
        ht(_mesa_hash_table_create(NULL, _mesa_hash_string,
                                   _mesa_key_string_equal))
   {
   }

   ~interface_block_definitions()
   {
      ralloc_free(mem_ctx);
      _mesa_hash_table_destroy(ht, NULL);
   }

   /**
    * Lookup the interface definition. Return NULL if none is found.
    */
   ir_variable *lookup(ir_variable *var)
   {
      if (var->data.explicit_location &&
          var->data.location >= VARYING_SLOT_VAR0) {
         char location_str[11];
         snprintf(location_str, 11, "%d", var->data.location);

         const struct hash_entry *entry =
            _mesa_hash_table_search(ht, location_str);
         return entry ? (ir_variable *) entry->data : NULL;
      } else {
         const struct hash_entry *entry =
            _mesa_hash_table_search(ht,
               var->get_interface_type()->without_array()->name);
         return entry ? (ir_variable *) entry->data : NULL;
      }
   }

   /**
    * Add a new interface definition.
    */
   void store(ir_variable *var)
   {
      if (var->data.explicit_location &&
          var->data.location >= VARYING_SLOT_VAR0) {
         /* If explicit location is given then lookup the variable by location.
          * We turn the location into a string and use this as the hash key
          * rather than the name. Note: We allocate enough space for a 32-bit
          * unsigned location value which is overkill but future proof.
          */
         char location_str[11];
         snprintf(location_str, 11, "%d", var->data.location);
         _mesa_hash_table_insert(ht, ralloc_strdup(mem_ctx, location_str), var);
      } else {
         _mesa_hash_table_insert(ht,
            var->get_interface_type()->without_array()->name, var);
      }
   }

private:
   /**
    * Ralloc context for data structures allocated by this class.
    */
   void *mem_ctx;

   /**
    * Hash table mapping interface block name to an \c
    * ir_variable.
    */
   hash_table *ht;
};


}; /* anonymous namespace */


void
validate_intrastage_interface_blocks(struct gl_shader_program *prog,
                                     const gl_shader **shader_list,
                                     unsigned num_shaders)
{
   interface_block_definitions in_interfaces;
   interface_block_definitions out_interfaces;
   interface_block_definitions uniform_interfaces;
   interface_block_definitions buffer_interfaces;

   for (unsigned int i = 0; i < num_shaders; i++) {
      if (shader_list[i] == NULL)
         continue;

      foreach_in_list(ir_instruction, node, shader_list[i]->ir) {
         ir_variable *var = node->as_variable();
         if (!var)
            continue;

         const glsl_type *iface_type = var->get_interface_type();

         if (iface_type == NULL)
            continue;

         interface_block_definitions *definitions;
         switch (var->data.mode) {
         case ir_var_shader_in:
            definitions = &in_interfaces;
            break;
         case ir_var_shader_out:
            definitions = &out_interfaces;
            break;
         case ir_var_uniform:
            definitions = &uniform_interfaces;
            break;
         case ir_var_shader_storage:
            definitions = &buffer_interfaces;
            break;
         default:
            /* Only in, out, and uniform interfaces are legal, so we should
             * never get here.
             */
            assert(!"illegal interface type");
            continue;
         }

         ir_variable *prev_def = definitions->lookup(var);
         if (prev_def == NULL) {
            /* This is the first time we've seen the interface, so save
             * it into the appropriate data structure.
             */
            definitions->store(var);
         } else if (!intrastage_match(prev_def, var, prog,
                                      true /* match_precision */)) {
            linker_error(prog, "definitions of interface block `%s' do not"
                         " match\n", iface_type->name);
            return;
         }
      }
   }
}

static bool
is_builtin_gl_in_block(ir_variable *var, int consumer_stage)
{
   return !strcmp(var->name, "gl_in") &&
          (consumer_stage == MESA_SHADER_TESS_CTRL ||
           consumer_stage == MESA_SHADER_TESS_EVAL ||
           consumer_stage == MESA_SHADER_GEOMETRY);
}

void
validate_interstage_inout_blocks(struct gl_shader_program *prog,
                                 const gl_linked_shader *producer,
                                 const gl_linked_shader *consumer)
{
   interface_block_definitions definitions;
   /* VS -> GS, VS -> TCS, VS -> TES, TES -> GS */
   const bool extra_array_level = (producer->Stage == MESA_SHADER_VERTEX &&
                                   consumer->Stage != MESA_SHADER_FRAGMENT) ||
                                  consumer->Stage == MESA_SHADER_GEOMETRY;

   /* Check that block re-declarations of gl_PerVertex are compatible
    * across shaders: From OpenGL Shading Language 4.5, section
    * "7.1 Built-In Language Variables", page 130 of the PDF:
    *
    *    "If multiple shaders using members of a built-in block belonging
    *     to the same interface are linked together in the same program,
    *     they must all redeclare the built-in block in the same way, as
    *     described in section 4.3.9 “Interface Blocks” for interface-block
    *     matching, or a link-time error will result."
    *
    * This is done explicitly outside of iterating the member variable
    * declarations because it is possible that the variables are not used and
    * so they would have been optimised out.
    */
   const glsl_type *consumer_iface =
      consumer->symbols->get_interface("gl_PerVertex",
                                       ir_var_shader_in);

   const glsl_type *producer_iface =
      producer->symbols->get_interface("gl_PerVertex",
                                       ir_var_shader_out);

   if (producer_iface && consumer_iface &&
       interstage_member_mismatch(prog, consumer_iface, producer_iface)) {
      linker_error(prog, "Incompatible or missing gl_PerVertex re-declaration "
                   "in consecutive shaders");
      return;
   }

   /* Desktop OpenGL requires redeclaration of the built-in interfaces for
    * SSO programs. Passes above implement following rules:
    *
    * From Section 7.4 (Program Pipeline Objects) of the OpenGL 4.6 Core
    * spec:
    *
    *    "To use any built-in input or output in the gl_PerVertex and
    *     gl_PerFragment blocks in separable program objects, shader code
    *     must redeclare those blocks prior to use.  A separable program
    *     will fail to link if:
    *
    *     it contains multiple shaders of a single type with different
    *     redeclarations of these built-in input and output blocks; or
    *
    *     any shader uses a built-in block member not found in the
    *     redeclaration of that block."
    *
    * ARB_separate_shader_objects issues section (issue #28) states that
    * redeclaration is not required for GLSL shaders using #version 140 or
    * earlier (since interface blocks are not possible with older versions).
    *
    * From Section 7.4.1 (Shader Interface Matching) of the OpenGL ES 3.1
    * spec:
    *
    *    "Built-in inputs or outputs do not affect interface matching."
    *
    * GL_OES_shader_io_blocks adds following:
    *
    *    "When using any built-in input or output in the gl_PerVertex block
    *     in separable program objects, shader code may redeclare that block
    *     prior to use. If the shader does not redeclare the block, the
    *     intrinsically declared definition of that block will be used."
    */

   /* Add output interfaces from the producer to the symbol table. */
   foreach_in_list(ir_instruction, node, producer->ir) {
      ir_variable *var = node->as_variable();
      if (!var || !var->get_interface_type() || var->data.mode != ir_var_shader_out)
         continue;

      /* Built-in interface redeclaration check. */
      if (prog->SeparateShader && !prog->IsES && prog->data->Version >= 150 &&
          var->data.how_declared == ir_var_declared_implicitly &&
          var->data.used && !producer_iface) {
         linker_error(prog, "missing output builtin block %s redeclaration "
                      "in separable shader program",
                      var->get_interface_type()->name);
         return;
      }

      definitions.store(var);
   }

   /* Verify that the consumer's input interfaces match. */
   foreach_in_list(ir_instruction, node, consumer->ir) {
      ir_variable *var = node->as_variable();
      if (!var || !var->get_interface_type() || var->data.mode != ir_var_shader_in)
         continue;

      ir_variable *producer_def = definitions.lookup(var);

      /* Built-in interface redeclaration check. */
      if (prog->SeparateShader && !prog->IsES && prog->data->Version >= 150 &&
          var->data.how_declared == ir_var_declared_implicitly &&
          var->data.used && !producer_iface) {
         linker_error(prog, "missing input builtin block %s redeclaration "
                      "in separable shader program",
                      var->get_interface_type()->name);
         return;
      }

      /* The producer doesn't generate this input: fail to link. Skip built-in
       * 'gl_in[]' since that may not be present if the producer does not
       * write to any of the pre-defined outputs (e.g. if the vertex shader
       * does not write to gl_Position, etc), which is allowed and results in
       * undefined behavior.
       *
       * From Section 4.3.4 (Inputs) of the GLSL 1.50 spec:
       *
       *    "Only the input variables that are actually read need to be written
       *     by the previous stage; it is allowed to have superfluous
       *     declarations of input variables."
       */
      if (producer_def == NULL &&
          !is_builtin_gl_in_block(var, consumer->Stage) && var->data.used) {
         linker_error(prog, "Input block `%s' is not an output of "
                      "the previous stage\n", var->get_interface_type()->name);
         return;
      }

      if (producer_def &&
          !interstage_match(prog, producer_def, var, extra_array_level)) {
         linker_error(prog, "definitions of interface block `%s' do not "
                      "match\n", var->get_interface_type()->name);
         return;
      }
   }
}


void
validate_interstage_uniform_blocks(struct gl_shader_program *prog,
                                   gl_linked_shader **stages)
{
   interface_block_definitions definitions;

   for (int i = 0; i < MESA_SHADER_STAGES; i++) {
      if (stages[i] == NULL)
         continue;

      const gl_linked_shader *stage = stages[i];
      foreach_in_list(ir_instruction, node, stage->ir) {
         ir_variable *var = node->as_variable();
         if (!var || !var->get_interface_type() ||
             (var->data.mode != ir_var_uniform &&
              var->data.mode != ir_var_shader_storage))
            continue;

         ir_variable *old_def = definitions.lookup(var);
         if (old_def == NULL) {
            definitions.store(var);
         } else {
            /* Interstage uniform matching rules are the same as intrastage
             * uniform matchin rules (for uniforms, it is as though all
             * shaders are in the same shader stage).
             */
            if (!intrastage_match(old_def, var, prog, false /* precision */)) {
               linker_error(prog, "definitions of uniform block `%s' do not "
                            "match\n", var->get_interface_type()->name);
               return;
            }
         }
      }
   }
}
