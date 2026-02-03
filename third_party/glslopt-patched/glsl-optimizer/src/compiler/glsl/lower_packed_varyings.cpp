/*
 * Copyright © 2011 Intel Corporation
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
 * \file lower_varyings_to_packed.cpp
 *
 * This lowering pass generates GLSL code that manually packs varyings into
 * vec4 slots, for the benefit of back-ends that don't support packed varyings
 * natively.
 *
 * For example, the following shader:
 *
 *   out mat3x2 foo;  // location=4, location_frac=0
 *   out vec3 bar[2]; // location=5, location_frac=2
 *
 *   main()
 *   {
 *     ...
 *   }
 *
 * Is rewritten to:
 *
 *   mat3x2 foo;
 *   vec3 bar[2];
 *   out vec4 packed4; // location=4, location_frac=0
 *   out vec4 packed5; // location=5, location_frac=0
 *   out vec4 packed6; // location=6, location_frac=0
 *
 *   main()
 *   {
 *     ...
 *     packed4.xy = foo[0];
 *     packed4.zw = foo[1];
 *     packed5.xy = foo[2];
 *     packed5.zw = bar[0].xy;
 *     packed6.x = bar[0].z;
 *     packed6.yzw = bar[1];
 *   }
 *
 * This lowering pass properly handles "double parking" of a varying vector
 * across two varying slots.  For example, in the code above, two of the
 * components of bar[0] are stored in packed5, and the remaining component is
 * stored in packed6.
 *
 * Note that in theory, the extra instructions may cause some loss of
 * performance.  However, hopefully in most cases the performance loss will
 * either be absorbed by a later optimization pass, or it will be offset by
 * memory bandwidth savings (because fewer varyings are used).
 *
 * This lowering pass also packs flat floats, ints, and uints together, by
 * using ivec4 as the base type of flat "varyings", and using appropriate
 * casts to convert floats and uints into ints.
 *
 * This lowering pass also handles varyings whose type is a struct or an array
 * of struct.  Structs are packed in order and with no gaps, so there may be a
 * performance penalty due to structure elements being double-parked.
 *
 * Lowering of geometry shader inputs is slightly more complex, since geometry
 * inputs are always arrays, so we need to lower arrays to arrays.  For
 * example, the following input:
 *
 *   in struct Foo {
 *     float f;
 *     vec3 v;
 *     vec2 a[2];
 *   } arr[3];         // location=4, location_frac=0
 *
 * Would get lowered like this if it occurred in a fragment shader:
 *
 *   struct Foo {
 *     float f;
 *     vec3 v;
 *     vec2 a[2];
 *   } arr[3];
 *   in vec4 packed4;  // location=4, location_frac=0
 *   in vec4 packed5;  // location=5, location_frac=0
 *   in vec4 packed6;  // location=6, location_frac=0
 *   in vec4 packed7;  // location=7, location_frac=0
 *   in vec4 packed8;  // location=8, location_frac=0
 *   in vec4 packed9;  // location=9, location_frac=0
 *
 *   main()
 *   {
 *     arr[0].f = packed4.x;
 *     arr[0].v = packed4.yzw;
 *     arr[0].a[0] = packed5.xy;
 *     arr[0].a[1] = packed5.zw;
 *     arr[1].f = packed6.x;
 *     arr[1].v = packed6.yzw;
 *     arr[1].a[0] = packed7.xy;
 *     arr[1].a[1] = packed7.zw;
 *     arr[2].f = packed8.x;
 *     arr[2].v = packed8.yzw;
 *     arr[2].a[0] = packed9.xy;
 *     arr[2].a[1] = packed9.zw;
 *     ...
 *   }
 *
 * But it would get lowered like this if it occurred in a geometry shader:
 *
 *   struct Foo {
 *     float f;
 *     vec3 v;
 *     vec2 a[2];
 *   } arr[3];
 *   in vec4 packed4[3];  // location=4, location_frac=0
 *   in vec4 packed5[3];  // location=5, location_frac=0
 *
 *   main()
 *   {
 *     arr[0].f = packed4[0].x;
 *     arr[0].v = packed4[0].yzw;
 *     arr[0].a[0] = packed5[0].xy;
 *     arr[0].a[1] = packed5[0].zw;
 *     arr[1].f = packed4[1].x;
 *     arr[1].v = packed4[1].yzw;
 *     arr[1].a[0] = packed5[1].xy;
 *     arr[1].a[1] = packed5[1].zw;
 *     arr[2].f = packed4[2].x;
 *     arr[2].v = packed4[2].yzw;
 *     arr[2].a[0] = packed5[2].xy;
 *     arr[2].a[1] = packed5[2].zw;
 *     ...
 *   }
 */

#include "glsl_symbol_table.h"
#include "ir.h"
#include "ir_builder.h"
#include "ir_optimization.h"
#include "program/prog_instruction.h"
#include "main/mtypes.h"

using namespace ir_builder;

namespace {

/**
 * Visitor that performs varying packing.  For each varying declared in the
 * shader, this visitor determines whether it needs to be packed.  If so, it
 * demotes it to an ordinary global, creates new packed varyings, and
 * generates assignments to convert between the original varying and the
 * packed varying.
 */
class lower_packed_varyings_visitor
{
public:
   lower_packed_varyings_visitor(void *mem_ctx,
                                 unsigned locations_used,
                                 const uint8_t *components,
                                 ir_variable_mode mode,
                                 unsigned gs_input_vertices,
                                 exec_list *out_instructions,
                                 exec_list *out_variables,
                                 bool disable_varying_packing,
                                 bool disable_xfb_packing,
                                 bool xfb_enabled);

   void run(struct gl_linked_shader *shader);

private:
   void bitwise_assign_pack(ir_rvalue *lhs, ir_rvalue *rhs);
   void bitwise_assign_unpack(ir_rvalue *lhs, ir_rvalue *rhs);
   unsigned lower_rvalue(ir_rvalue *rvalue, unsigned fine_location,
                         ir_variable *unpacked_var, const char *name,
                         bool gs_input_toplevel, unsigned vertex_index);
   unsigned lower_arraylike(ir_rvalue *rvalue, unsigned array_size,
                            unsigned fine_location,
                            ir_variable *unpacked_var, const char *name,
                            bool gs_input_toplevel, unsigned vertex_index);
   ir_dereference *get_packed_varying_deref(unsigned location,
                                            ir_variable *unpacked_var,
                                            const char *name,
                                            unsigned vertex_index);
   bool needs_lowering(ir_variable *var);

   /**
    * Memory context used to allocate new instructions for the shader.
    */
   void * const mem_ctx;

   /**
    * Number of generic varying slots which are used by this shader.  This is
    * used to allocate temporary intermediate data structures.  If any varying
    * used by this shader has a location greater than or equal to
    * VARYING_SLOT_VAR0 + locations_used, an assertion will fire.
    */
   const unsigned locations_used;

   const uint8_t* components;

   /**
    * Array of pointers to the packed varyings that have been created for each
    * generic varying slot.  NULL entries in this array indicate varying slots
    * for which a packed varying has not been created yet.
    */
   ir_variable **packed_varyings;

   /**
    * Type of varying which is being lowered in this pass (either
    * ir_var_shader_in or ir_var_shader_out).
    */
   const ir_variable_mode mode;

   /**
    * If we are currently lowering geometry shader inputs, the number of input
    * vertices the geometry shader accepts.  Otherwise zero.
    */
   const unsigned gs_input_vertices;

   /**
    * Exec list into which the visitor should insert the packing instructions.
    * Caller provides this list; it should insert the instructions into the
    * appropriate place in the shader once the visitor has finished running.
    */
   exec_list *out_instructions;

   /**
    * Exec list into which the visitor should insert any new variables.
    */
   exec_list *out_variables;

   bool disable_varying_packing;
   bool disable_xfb_packing;
   bool xfb_enabled;
};

} /* anonymous namespace */

lower_packed_varyings_visitor::lower_packed_varyings_visitor(
      void *mem_ctx, unsigned locations_used, const uint8_t *components,
      ir_variable_mode mode,
      unsigned gs_input_vertices, exec_list *out_instructions,
      exec_list *out_variables, bool disable_varying_packing,
      bool disable_xfb_packing, bool xfb_enabled)
   : mem_ctx(mem_ctx),
     locations_used(locations_used),
     components(components),
     packed_varyings((ir_variable **)
                     rzalloc_array_size(mem_ctx, sizeof(*packed_varyings),
                                        locations_used)),
     mode(mode),
     gs_input_vertices(gs_input_vertices),
     out_instructions(out_instructions),
     out_variables(out_variables),
     disable_varying_packing(disable_varying_packing),
     disable_xfb_packing(disable_xfb_packing),
     xfb_enabled(xfb_enabled)
{
}

void
lower_packed_varyings_visitor::run(struct gl_linked_shader *shader)
{
   foreach_in_list(ir_instruction, node, shader->ir) {
      ir_variable *var = node->as_variable();
      if (var == NULL)
         continue;

      if (var->data.mode != this->mode ||
          var->data.location < VARYING_SLOT_VAR0 ||
          !this->needs_lowering(var))
         continue;

      /* This lowering pass is only capable of packing floats and ints
       * together when their interpolation mode is "flat".  Treat integers as
       * being flat when the interpolation mode is none.
       */
      assert(var->data.interpolation == INTERP_MODE_FLAT ||
             var->data.interpolation == INTERP_MODE_NONE ||
             !var->type->contains_integer());

      /* Clone the variable for program resource list before
       * it gets modified and lost.
       */
      if (!shader->packed_varyings)
         shader->packed_varyings = new (shader) exec_list;

      shader->packed_varyings->push_tail(var->clone(shader, NULL));

      /* Change the old varying into an ordinary global. */
      assert(var->data.mode != ir_var_temporary);
      var->data.mode = ir_var_auto;

      /* Create a reference to the old varying. */
      ir_dereference_variable *deref
         = new(this->mem_ctx) ir_dereference_variable(var);

      /* Recursively pack or unpack it. */
      this->lower_rvalue(deref, var->data.location * 4 + var->data.location_frac, var,
                         var->name, this->gs_input_vertices != 0, 0);
   }
}

#define SWIZZLE_ZWZW MAKE_SWIZZLE4(SWIZZLE_Z, SWIZZLE_W, SWIZZLE_Z, SWIZZLE_W)

/**
 * Make an ir_assignment from \c rhs to \c lhs, performing appropriate
 * bitcasts if necessary to match up types.
 *
 * This function is called when packing varyings.
 */
void
lower_packed_varyings_visitor::bitwise_assign_pack(ir_rvalue *lhs,
                                                   ir_rvalue *rhs)
{
   if (lhs->type->base_type != rhs->type->base_type) {
      /* Since we only mix types in flat varyings, and we always store flat
       * varyings as type ivec4, we need only produce conversions from (uint
       * or float) to int.
       */
      assert(lhs->type->base_type == GLSL_TYPE_INT);
      switch (rhs->type->base_type) {
      case GLSL_TYPE_UINT:
         rhs = new(this->mem_ctx)
            ir_expression(ir_unop_u2i, lhs->type, rhs);
         break;
      case GLSL_TYPE_FLOAT:
         rhs = new(this->mem_ctx)
            ir_expression(ir_unop_bitcast_f2i, lhs->type, rhs);
         break;
      case GLSL_TYPE_DOUBLE:
         assert(rhs->type->vector_elements <= 2);
         if (rhs->type->vector_elements == 2) {
            ir_variable *t = new(mem_ctx) ir_variable(lhs->type, "pack", ir_var_temporary);

            assert(lhs->type->vector_elements == 4);
            this->out_variables->push_tail(t);
            this->out_instructions->push_tail(
                  assign(t, u2i(expr(ir_unop_unpack_double_2x32, swizzle_x(rhs->clone(mem_ctx, NULL)))), 0x3));
            this->out_instructions->push_tail(
                  assign(t,  u2i(expr(ir_unop_unpack_double_2x32, swizzle_y(rhs))), 0xc));
            rhs = deref(t).val;
         } else {
            rhs = u2i(expr(ir_unop_unpack_double_2x32, rhs));
         }
         break;
      case GLSL_TYPE_INT64:
         assert(rhs->type->vector_elements <= 2);
         if (rhs->type->vector_elements == 2) {
            ir_variable *t = new(mem_ctx) ir_variable(lhs->type, "pack", ir_var_temporary);

            assert(lhs->type->vector_elements == 4);
            this->out_variables->push_tail(t);
            this->out_instructions->push_tail(
               assign(t, expr(ir_unop_unpack_int_2x32, swizzle_x(rhs->clone(mem_ctx, NULL))), 0x3));
            this->out_instructions->push_tail(
               assign(t,  expr(ir_unop_unpack_int_2x32, swizzle_y(rhs)), 0xc));
            rhs = deref(t).val;
         } else {
            rhs = expr(ir_unop_unpack_int_2x32, rhs);
         }
         break;
      case GLSL_TYPE_UINT64:
         assert(rhs->type->vector_elements <= 2);
         if (rhs->type->vector_elements == 2) {
            ir_variable *t = new(mem_ctx) ir_variable(lhs->type, "pack", ir_var_temporary);

            assert(lhs->type->vector_elements == 4);
            this->out_variables->push_tail(t);
            this->out_instructions->push_tail(
                  assign(t, u2i(expr(ir_unop_unpack_uint_2x32, swizzle_x(rhs->clone(mem_ctx, NULL)))), 0x3));
            this->out_instructions->push_tail(
                  assign(t,  u2i(expr(ir_unop_unpack_uint_2x32, swizzle_y(rhs))), 0xc));
            rhs = deref(t).val;
         } else {
            rhs = u2i(expr(ir_unop_unpack_uint_2x32, rhs));
         }
         break;
      case GLSL_TYPE_SAMPLER:
         rhs = u2i(expr(ir_unop_unpack_sampler_2x32, rhs));
         break;
      case GLSL_TYPE_IMAGE:
         rhs = u2i(expr(ir_unop_unpack_image_2x32, rhs));
         break;
      default:
         assert(!"Unexpected type conversion while lowering varyings");
         break;
      }
   }
   this->out_instructions->push_tail(new (this->mem_ctx) ir_assignment(lhs, rhs));
}


/**
 * Make an ir_assignment from \c rhs to \c lhs, performing appropriate
 * bitcasts if necessary to match up types.
 *
 * This function is called when unpacking varyings.
 */
void
lower_packed_varyings_visitor::bitwise_assign_unpack(ir_rvalue *lhs,
                                                     ir_rvalue *rhs)
{
   if (lhs->type->base_type != rhs->type->base_type) {
      /* Since we only mix types in flat varyings, and we always store flat
       * varyings as type ivec4, we need only produce conversions from int to
       * (uint or float).
       */
      assert(rhs->type->base_type == GLSL_TYPE_INT);
      switch (lhs->type->base_type) {
      case GLSL_TYPE_UINT:
         rhs = new(this->mem_ctx)
            ir_expression(ir_unop_i2u, lhs->type, rhs);
         break;
      case GLSL_TYPE_FLOAT:
         rhs = new(this->mem_ctx)
            ir_expression(ir_unop_bitcast_i2f, lhs->type, rhs);
         break;
      case GLSL_TYPE_DOUBLE:
         assert(lhs->type->vector_elements <= 2);
         if (lhs->type->vector_elements == 2) {
            ir_variable *t = new(mem_ctx) ir_variable(lhs->type, "unpack", ir_var_temporary);
            assert(rhs->type->vector_elements == 4);
            this->out_variables->push_tail(t);
            this->out_instructions->push_tail(
                  assign(t, expr(ir_unop_pack_double_2x32, i2u(swizzle_xy(rhs->clone(mem_ctx, NULL)))), 0x1));
            this->out_instructions->push_tail(
                  assign(t, expr(ir_unop_pack_double_2x32, i2u(swizzle(rhs->clone(mem_ctx, NULL), SWIZZLE_ZWZW, 2))), 0x2));
            rhs = deref(t).val;
         } else {
            rhs = expr(ir_unop_pack_double_2x32, i2u(rhs));
         }
         break;
      case GLSL_TYPE_INT64:
         assert(lhs->type->vector_elements <= 2);
         if (lhs->type->vector_elements == 2) {
            ir_variable *t = new(mem_ctx) ir_variable(lhs->type, "unpack", ir_var_temporary);
            assert(rhs->type->vector_elements == 4);
            this->out_variables->push_tail(t);
            this->out_instructions->push_tail(
                  assign(t, expr(ir_unop_pack_int_2x32, swizzle_xy(rhs->clone(mem_ctx, NULL))), 0x1));
            this->out_instructions->push_tail(
                  assign(t, expr(ir_unop_pack_int_2x32, swizzle(rhs->clone(mem_ctx, NULL), SWIZZLE_ZWZW, 2)), 0x2));
            rhs = deref(t).val;
         } else {
            rhs = expr(ir_unop_pack_int_2x32, rhs);
         }
         break;
      case GLSL_TYPE_UINT64:
         assert(lhs->type->vector_elements <= 2);
         if (lhs->type->vector_elements == 2) {
            ir_variable *t = new(mem_ctx) ir_variable(lhs->type, "unpack", ir_var_temporary);
            assert(rhs->type->vector_elements == 4);
            this->out_variables->push_tail(t);
            this->out_instructions->push_tail(
                  assign(t, expr(ir_unop_pack_uint_2x32, i2u(swizzle_xy(rhs->clone(mem_ctx, NULL)))), 0x1));
            this->out_instructions->push_tail(
                  assign(t, expr(ir_unop_pack_uint_2x32, i2u(swizzle(rhs->clone(mem_ctx, NULL), SWIZZLE_ZWZW, 2))), 0x2));
            rhs = deref(t).val;
         } else {
            rhs = expr(ir_unop_pack_uint_2x32, i2u(rhs));
         }
         break;
      case GLSL_TYPE_SAMPLER:
         rhs = new(mem_ctx)
            ir_expression(ir_unop_pack_sampler_2x32, lhs->type, i2u(rhs));
         break;
      case GLSL_TYPE_IMAGE:
         rhs = new(mem_ctx)
            ir_expression(ir_unop_pack_image_2x32, lhs->type, i2u(rhs));
         break;
      default:
         assert(!"Unexpected type conversion while lowering varyings");
         break;
      }
   }
   this->out_instructions->push_tail(new(this->mem_ctx) ir_assignment(lhs, rhs));
}


/**
 * Recursively pack or unpack the given varying (or portion of a varying) by
 * traversing all of its constituent vectors.
 *
 * \param fine_location is the location where the first constituent vector
 * should be packed--the word "fine" indicates that this location is expressed
 * in multiples of a float, rather than multiples of a vec4 as is used
 * elsewhere in Mesa.
 *
 * \param gs_input_toplevel should be set to true if we are lowering geometry
 * shader inputs, and we are currently lowering the whole input variable
 * (i.e. we are lowering the array whose index selects the vertex).
 *
 * \param vertex_index: if we are lowering geometry shader inputs, and the
 * level of the array that we are currently lowering is *not* the top level,
 * then this indicates which vertex we are currently lowering.  Otherwise it
 * is ignored.
 *
 * \return the location where the next constituent vector (after this one)
 * should be packed.
 */
unsigned
lower_packed_varyings_visitor::lower_rvalue(ir_rvalue *rvalue,
                                            unsigned fine_location,
                                            ir_variable *unpacked_var,
                                            const char *name,
                                            bool gs_input_toplevel,
                                            unsigned vertex_index)
{
   unsigned dmul = rvalue->type->is_64bit() ? 2 : 1;
   /* When gs_input_toplevel is set, we should be looking at a geometry shader
    * input array.
    */
   assert(!gs_input_toplevel || rvalue->type->is_array());

   if (rvalue->type->is_struct()) {
      for (unsigned i = 0; i < rvalue->type->length; i++) {
         if (i != 0)
            rvalue = rvalue->clone(this->mem_ctx, NULL);
         const char *field_name = rvalue->type->fields.structure[i].name;
         ir_dereference_record *dereference_record = new(this->mem_ctx)
            ir_dereference_record(rvalue, field_name);
         char *deref_name
            = ralloc_asprintf(this->mem_ctx, "%s.%s", name, field_name);
         fine_location = this->lower_rvalue(dereference_record, fine_location,
                                            unpacked_var, deref_name, false,
                                            vertex_index);
      }
      return fine_location;
   } else if (rvalue->type->is_array()) {
      /* Arrays are packed/unpacked by considering each array element in
       * sequence.
       */
      return this->lower_arraylike(rvalue, rvalue->type->array_size(),
                                   fine_location, unpacked_var, name,
                                   gs_input_toplevel, vertex_index);
   } else if (rvalue->type->is_matrix()) {
      /* Matrices are packed/unpacked by considering each column vector in
       * sequence.
       */
      return this->lower_arraylike(rvalue, rvalue->type->matrix_columns,
                                   fine_location, unpacked_var, name,
                                   false, vertex_index);
   } else if (rvalue->type->vector_elements * dmul +
              fine_location % 4 > 4) {
      /* This vector is going to be "double parked" across two varying slots,
       * so handle it as two separate assignments. For doubles, a dvec3/dvec4
       * can end up being spread over 3 slots. However the second splitting
       * will happen later, here we just always want to split into 2.
       */
      unsigned left_components, right_components;
      unsigned left_swizzle_values[4] = { 0, 0, 0, 0 };
      unsigned right_swizzle_values[4] = { 0, 0, 0, 0 };
      char left_swizzle_name[4] = { 0, 0, 0, 0 };
      char right_swizzle_name[4] = { 0, 0, 0, 0 };

      left_components = 4 - fine_location % 4;
      if (rvalue->type->is_64bit()) {
         /* We might actually end up with 0 left components! */
         left_components /= 2;
      }
      right_components = rvalue->type->vector_elements - left_components;

      for (unsigned i = 0; i < left_components; i++) {
         left_swizzle_values[i] = i;
         left_swizzle_name[i] = "xyzw"[i];
      }
      for (unsigned i = 0; i < right_components; i++) {
         right_swizzle_values[i] = i + left_components;
         right_swizzle_name[i] = "xyzw"[i + left_components];
      }
      ir_swizzle *left_swizzle = new(this->mem_ctx)
         ir_swizzle(rvalue, left_swizzle_values, left_components);
      ir_swizzle *right_swizzle = new(this->mem_ctx)
         ir_swizzle(rvalue->clone(this->mem_ctx, NULL), right_swizzle_values,
                    right_components);
      char *left_name
         = ralloc_asprintf(this->mem_ctx, "%s.%s", name, left_swizzle_name);
      char *right_name
         = ralloc_asprintf(this->mem_ctx, "%s.%s", name, right_swizzle_name);
      if (left_components)
         fine_location = this->lower_rvalue(left_swizzle, fine_location,
                                            unpacked_var, left_name, false,
                                            vertex_index);
      else
         /* Top up the fine location to the next slot */
         fine_location++;
      return this->lower_rvalue(right_swizzle, fine_location, unpacked_var,
                                right_name, false, vertex_index);
   } else {
      /* No special handling is necessary; pack the rvalue into the
       * varying.
       */
      unsigned swizzle_values[4] = { 0, 0, 0, 0 };
      unsigned components = rvalue->type->vector_elements * dmul;
      unsigned location = fine_location / 4;
      unsigned location_frac = fine_location % 4;
      for (unsigned i = 0; i < components; ++i)
         swizzle_values[i] = i + location_frac;
      ir_dereference *packed_deref =
         this->get_packed_varying_deref(location, unpacked_var, name,
                                        vertex_index);
      if (unpacked_var->data.stream != 0) {
         assert(unpacked_var->data.stream < 4);
         ir_variable *packed_var = packed_deref->variable_referenced();
         for (unsigned i = 0; i < components; ++i) {
            packed_var->data.stream |=
               unpacked_var->data.stream << (2 * (location_frac + i));
         }
      }
      ir_swizzle *swizzle = new(this->mem_ctx)
         ir_swizzle(packed_deref, swizzle_values, components);
      if (this->mode == ir_var_shader_out) {
         this->bitwise_assign_pack(swizzle, rvalue);
      } else {
         this->bitwise_assign_unpack(rvalue, swizzle);
      }
      return fine_location + components;
   }
}

/**
 * Recursively pack or unpack a varying for which we need to iterate over its
 * constituent elements, accessing each one using an ir_dereference_array.
 * This takes care of both arrays and matrices, since ir_dereference_array
 * treats a matrix like an array of its column vectors.
 *
 * \param gs_input_toplevel should be set to true if we are lowering geometry
 * shader inputs, and we are currently lowering the whole input variable
 * (i.e. we are lowering the array whose index selects the vertex).
 *
 * \param vertex_index: if we are lowering geometry shader inputs, and the
 * level of the array that we are currently lowering is *not* the top level,
 * then this indicates which vertex we are currently lowering.  Otherwise it
 * is ignored.
 */
unsigned
lower_packed_varyings_visitor::lower_arraylike(ir_rvalue *rvalue,
                                               unsigned array_size,
                                               unsigned fine_location,
                                               ir_variable *unpacked_var,
                                               const char *name,
                                               bool gs_input_toplevel,
                                               unsigned vertex_index)
{
   for (unsigned i = 0; i < array_size; i++) {
      if (i != 0)
         rvalue = rvalue->clone(this->mem_ctx, NULL);
      ir_constant *constant = new(this->mem_ctx) ir_constant(i);
      ir_dereference_array *dereference_array = new(this->mem_ctx)
         ir_dereference_array(rvalue, constant);
      if (gs_input_toplevel) {
         /* Geometry shader inputs are a special case.  Instead of storing
          * each element of the array at a different location, all elements
          * are at the same location, but with a different vertex index.
          */
         (void) this->lower_rvalue(dereference_array, fine_location,
                                   unpacked_var, name, false, i);
      } else {
         char *subscripted_name
            = ralloc_asprintf(this->mem_ctx, "%s[%d]", name, i);
         fine_location =
            this->lower_rvalue(dereference_array, fine_location,
                               unpacked_var, subscripted_name,
                               false, vertex_index);
      }
   }
   return fine_location;
}

/**
 * Retrieve the packed varying corresponding to the given varying location.
 * If no packed varying has been created for the given varying location yet,
 * create it and add it to the shader before returning it.
 *
 * The newly created varying inherits its interpolation parameters from \c
 * unpacked_var.  Its base type is ivec4 if we are lowering a flat varying,
 * vec4 otherwise.
 *
 * \param vertex_index: if we are lowering geometry shader inputs, then this
 * indicates which vertex we are currently lowering.  Otherwise it is ignored.
 */
ir_dereference *
lower_packed_varyings_visitor::get_packed_varying_deref(
      unsigned location, ir_variable *unpacked_var, const char *name,
      unsigned vertex_index)
{
   unsigned slot = location - VARYING_SLOT_VAR0;
   assert(slot < locations_used);
   if (this->packed_varyings[slot] == NULL) {
      char *packed_name = ralloc_asprintf(this->mem_ctx, "packed:%s", name);
      const glsl_type *packed_type;
      assert(components[slot] != 0);
      if (unpacked_var->is_interpolation_flat())
         packed_type = glsl_type::get_instance(GLSL_TYPE_INT, components[slot], 1);
      else
         packed_type = glsl_type::get_instance(GLSL_TYPE_FLOAT, components[slot], 1);
      if (this->gs_input_vertices != 0) {
         packed_type =
            glsl_type::get_array_instance(packed_type,
                                          this->gs_input_vertices);
      }
      ir_variable *packed_var = new(this->mem_ctx)
         ir_variable(packed_type, packed_name, this->mode);
      if (this->gs_input_vertices != 0) {
         /* Prevent update_array_sizes() from messing with the size of the
          * array.
          */
         packed_var->data.max_array_access = this->gs_input_vertices - 1;
      }
      packed_var->data.centroid = unpacked_var->data.centroid;
      packed_var->data.sample = unpacked_var->data.sample;
      packed_var->data.patch = unpacked_var->data.patch;
      packed_var->data.interpolation =
         packed_type->without_array() == glsl_type::ivec4_type
         ? unsigned(INTERP_MODE_FLAT) : unpacked_var->data.interpolation;
      packed_var->data.location = location;
      packed_var->data.precision = unpacked_var->data.precision;
      packed_var->data.always_active_io = unpacked_var->data.always_active_io;
      packed_var->data.stream = 1u << 31;
      unpacked_var->insert_before(packed_var);
      this->packed_varyings[slot] = packed_var;
   } else {
      ir_variable *var = this->packed_varyings[slot];

      /* The slot needs to be marked as always active if any variable that got
       * packed there was.
       */
      var->data.always_active_io |= unpacked_var->data.always_active_io;

      /* For geometry shader inputs, only update the packed variable name the
       * first time we visit each component.
       */
      if (this->gs_input_vertices == 0 || vertex_index == 0) {
         if (var->is_name_ralloced())
            ralloc_asprintf_append((char **) &var->name, ",%s", name);
         else
            var->name = ralloc_asprintf(var, "%s,%s", var->name, name);
      }
   }

   ir_dereference *deref = new(this->mem_ctx)
      ir_dereference_variable(this->packed_varyings[slot]);
   if (this->gs_input_vertices != 0) {
      /* When lowering GS inputs, the packed variable is an array, so we need
       * to dereference it using vertex_index.
       */
      ir_constant *constant = new(this->mem_ctx) ir_constant(vertex_index);
      deref = new(this->mem_ctx) ir_dereference_array(deref, constant);
   }
   return deref;
}

bool
lower_packed_varyings_visitor::needs_lowering(ir_variable *var)
{
   /* Things composed of vec4's, varyings with explicitly assigned
    * locations or varyings marked as must_be_shader_input (which might be used
    * by interpolateAt* functions) shouldn't be lowered. Everything else can be.
    */
   if (var->data.explicit_location || var->data.must_be_shader_input)
      return false;

   const glsl_type *type = var->type;

   /* Some drivers (e.g. panfrost) don't support packing of transform
    * feedback varyings.
    */
   if (disable_xfb_packing && var->data.is_xfb &&
       !(type->is_array() || type->is_struct() || type->is_matrix()) &&
       xfb_enabled)
      return false;

   /* Override disable_varying_packing if the var is only used by transform
    * feedback. Also override it if transform feedback is enabled and the
    * variable is an array, struct or matrix as the elements of these types
    * will always have the same interpolation and therefore are safe to pack.
    */
   if (disable_varying_packing && !var->data.is_xfb_only &&
       !((type->is_array() || type->is_struct() || type->is_matrix()) &&
         xfb_enabled))
      return false;

   type = type->without_array();
   if (type->vector_elements == 4 && !type->is_64bit())
      return false;
   return true;
}


/**
 * Visitor that splices varying packing code before every use of EmitVertex()
 * in a geometry shader.
 */
class lower_packed_varyings_gs_splicer : public ir_hierarchical_visitor
{
public:
   explicit lower_packed_varyings_gs_splicer(void *mem_ctx,
                                             const exec_list *instructions);

   virtual ir_visitor_status visit_leave(ir_emit_vertex *ev);

private:
   /**
    * Memory context used to allocate new instructions for the shader.
    */
   void * const mem_ctx;

   /**
    * Instructions that should be spliced into place before each EmitVertex()
    * call.
    */
   const exec_list *instructions;
};


lower_packed_varyings_gs_splicer::lower_packed_varyings_gs_splicer(
      void *mem_ctx, const exec_list *instructions)
   : mem_ctx(mem_ctx), instructions(instructions)
{
}


ir_visitor_status
lower_packed_varyings_gs_splicer::visit_leave(ir_emit_vertex *ev)
{
   foreach_in_list(ir_instruction, ir, this->instructions) {
      ev->insert_before(ir->clone(this->mem_ctx, NULL));
   }
   return visit_continue;
}

/**
 * Visitor that splices varying packing code before every return.
 */
class lower_packed_varyings_return_splicer : public ir_hierarchical_visitor
{
public:
   explicit lower_packed_varyings_return_splicer(void *mem_ctx,
                                                 const exec_list *instructions);

   virtual ir_visitor_status visit_leave(ir_return *ret);

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


lower_packed_varyings_return_splicer::lower_packed_varyings_return_splicer(
      void *mem_ctx, const exec_list *instructions)
   : mem_ctx(mem_ctx), instructions(instructions)
{
}


ir_visitor_status
lower_packed_varyings_return_splicer::visit_leave(ir_return *ret)
{
   foreach_in_list(ir_instruction, ir, this->instructions) {
      ret->insert_before(ir->clone(this->mem_ctx, NULL));
   }
   return visit_continue;
}

void
lower_packed_varyings(void *mem_ctx, unsigned locations_used,
                      const uint8_t *components,
                      ir_variable_mode mode, unsigned gs_input_vertices,
                      gl_linked_shader *shader, bool disable_varying_packing,
                      bool disable_xfb_packing, bool xfb_enabled)
{
   exec_list *instructions = shader->ir;
   ir_function *main_func = shader->symbols->get_function("main");
   exec_list void_parameters;
   ir_function_signature *main_func_sig
      = main_func->matching_signature(NULL, &void_parameters, false);
   exec_list new_instructions, new_variables;
   lower_packed_varyings_visitor visitor(mem_ctx,
                                         locations_used,
                                         components,
                                         mode,
                                         gs_input_vertices,
                                         &new_instructions,
                                         &new_variables,
                                         disable_varying_packing,
                                         disable_xfb_packing,
                                         xfb_enabled);
   visitor.run(shader);
   if (mode == ir_var_shader_out) {
      if (shader->Stage == MESA_SHADER_GEOMETRY) {
         /* For geometry shaders, outputs need to be lowered before each call
          * to EmitVertex()
          */
         lower_packed_varyings_gs_splicer splicer(mem_ctx, &new_instructions);

         /* Add all the variables in first. */
         main_func_sig->body.get_head_raw()->insert_before(&new_variables);

         /* Now update all the EmitVertex instances */
         splicer.run(instructions);
      } else {
         /* For other shader types, outputs need to be lowered before each
          * return statement and at the end of main()
          */

         lower_packed_varyings_return_splicer splicer(mem_ctx, &new_instructions);

         main_func_sig->body.get_head_raw()->insert_before(&new_variables);

         splicer.run(instructions);

         /* Lower outputs at the end of main() if the last instruction is not
          * a return statement
          */
         if (((ir_instruction*)instructions->get_tail())->ir_type != ir_type_return) {
            main_func_sig->body.append_list(&new_instructions);
         }
      }
   } else {
      /* Shader inputs need to be lowered at the beginning of main() */
      main_func_sig->body.get_head_raw()->insert_before(&new_instructions);
      main_func_sig->body.get_head_raw()->insert_before(&new_variables);
   }
}
