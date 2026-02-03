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
 * \file lower_buffer_access.cpp
 *
 * Helper for IR lowering pass to replace dereferences of buffer object based
 * shader variables with intrinsic function calls.
 *
 * This helper is used by lowering passes for UBOs, SSBOs and compute shader
 * shared variables.
 */

#include "lower_buffer_access.h"
#include "ir_builder.h"
#include "main/macros.h"
#include "util/list.h"
#include "glsl_parser_extras.h"
#include "linker.h"

using namespace ir_builder;

namespace lower_buffer_access {

static inline int
writemask_for_size(unsigned n)
{
   return ((1 << n) - 1);
}

/**
 * Takes a deref and recursively calls itself to break the deref down to the
 * point that the reads or writes generated are contiguous scalars or vectors.
 */
void
lower_buffer_access::emit_access(void *mem_ctx,
                                 bool is_write,
                                 ir_dereference *deref,
                                 ir_variable *base_offset,
                                 unsigned int deref_offset,
                                 bool row_major,
                                 const glsl_type *matrix_type,
                                 enum glsl_interface_packing packing,
                                 unsigned int write_mask)
{
   if (deref->type->is_struct()) {
      unsigned int field_offset = 0;

      for (unsigned i = 0; i < deref->type->length; i++) {
         const struct glsl_struct_field *field =
            &deref->type->fields.structure[i];
         ir_dereference *field_deref =
            new(mem_ctx) ir_dereference_record(deref->clone(mem_ctx, NULL),
                                               field->name);

         unsigned field_align;
         if (packing == GLSL_INTERFACE_PACKING_STD430)
            field_align = field->type->std430_base_alignment(row_major);
         else
            field_align = field->type->std140_base_alignment(row_major);
         field_offset = glsl_align(field_offset, field_align);

         emit_access(mem_ctx, is_write, field_deref, base_offset,
                     deref_offset + field_offset,
                     row_major, NULL, packing,
                     writemask_for_size(field_deref->type->vector_elements));

         if (packing == GLSL_INTERFACE_PACKING_STD430)
            field_offset += field->type->std430_size(row_major);
         else
            field_offset += field->type->std140_size(row_major);
      }
      return;
   }

   if (deref->type->is_array()) {
      unsigned array_stride = packing == GLSL_INTERFACE_PACKING_STD430 ?
         deref->type->fields.array->std430_array_stride(row_major) :
         glsl_align(deref->type->fields.array->std140_size(row_major), 16);

      for (unsigned i = 0; i < deref->type->length; i++) {
         ir_constant *element = new(mem_ctx) ir_constant(i);
         ir_dereference *element_deref =
            new(mem_ctx) ir_dereference_array(deref->clone(mem_ctx, NULL),
                                              element);
         emit_access(mem_ctx, is_write, element_deref, base_offset,
                     deref_offset + i * array_stride,
                     row_major, NULL, packing,
                     writemask_for_size(element_deref->type->vector_elements));
      }
      return;
   }

   if (deref->type->is_matrix()) {
      for (unsigned i = 0; i < deref->type->matrix_columns; i++) {
         ir_constant *col = new(mem_ctx) ir_constant(i);
         ir_dereference *col_deref =
            new(mem_ctx) ir_dereference_array(deref->clone(mem_ctx, NULL), col);

         /* For a row-major matrix, the next column starts at the next
          * element.  Otherwise it is offset by the matrix stride.
          */
         const unsigned size_mul = row_major
            ? (deref->type->is_double() ? 8 : 4)
            : link_calculate_matrix_stride(deref->type, row_major, packing);

         emit_access(mem_ctx, is_write, col_deref, base_offset,
                     deref_offset + i * size_mul,
                     row_major, deref->type, packing,
                     writemask_for_size(col_deref->type->vector_elements));
      }
      return;
   }

   assert(deref->type->is_scalar() || deref->type->is_vector());

   if (!row_major) {
      ir_rvalue *offset =
         add(base_offset, new(mem_ctx) ir_constant(deref_offset));
      unsigned mask =
         is_write ? write_mask : (1 << deref->type->vector_elements) - 1;
      insert_buffer_access(mem_ctx, deref, deref->type, offset, mask, -1);
   } else {
      /* We're dereffing a column out of a row-major matrix, so we
       * gather the vector from each stored row.
       */
      assert(deref->type->is_float() || deref->type->is_double());
      assert(matrix_type != NULL);

      const unsigned matrix_stride =
         link_calculate_matrix_stride(matrix_type, row_major, packing);

      const glsl_type *deref_type = deref->type->get_scalar_type();

      for (unsigned i = 0; i < deref->type->vector_elements; i++) {
         ir_rvalue *chan_offset =
            add(base_offset,
                new(mem_ctx) ir_constant(deref_offset + i * matrix_stride));
         if (!is_write || ((1U << i) & write_mask))
            insert_buffer_access(mem_ctx, deref, deref_type, chan_offset,
                                 (1U << i), i);
      }
   }
}

/**
 * Determine if a thing being dereferenced is row-major
 *
 * There is some trickery here.
 *
 * If the thing being dereferenced is a member of uniform block \b without an
 * instance name, then the name of the \c ir_variable is the field name of an
 * interface type.  If this field is row-major, then the thing referenced is
 * row-major.
 *
 * If the thing being dereferenced is a member of uniform block \b with an
 * instance name, then the last dereference in the tree will be an
 * \c ir_dereference_record.  If that record field is row-major, then the
 * thing referenced is row-major.
 */
bool
lower_buffer_access::is_dereferenced_thing_row_major(const ir_rvalue *deref)
{
   bool matrix = false;
   const ir_rvalue *ir = deref;

   while (true) {
      matrix = matrix || ir->type->without_array()->is_matrix();

      switch (ir->ir_type) {
      case ir_type_dereference_array: {
         const ir_dereference_array *const array_deref =
            (const ir_dereference_array *) ir;

         ir = array_deref->array;
         break;
      }

      case ir_type_dereference_record: {
         const ir_dereference_record *const record_deref =
            (const ir_dereference_record *) ir;

         ir = record_deref->record;

         const int idx = record_deref->field_idx;
         assert(idx >= 0);

         const enum glsl_matrix_layout matrix_layout =
            glsl_matrix_layout(ir->type->fields.structure[idx].matrix_layout);

         switch (matrix_layout) {
         case GLSL_MATRIX_LAYOUT_INHERITED:
            break;
         case GLSL_MATRIX_LAYOUT_COLUMN_MAJOR:
            return false;
         case GLSL_MATRIX_LAYOUT_ROW_MAJOR:
            return matrix || deref->type->without_array()->is_struct();
         }

         break;
      }

      case ir_type_dereference_variable: {
         const ir_dereference_variable *const var_deref =
            (const ir_dereference_variable *) ir;

         const enum glsl_matrix_layout matrix_layout =
            glsl_matrix_layout(var_deref->var->data.matrix_layout);

         switch (matrix_layout) {
         case GLSL_MATRIX_LAYOUT_INHERITED: {
            /* For interface block matrix variables we handle inherited
             * layouts at HIR generation time, but we don't do that for shared
             * variables, which are always column-major
             */
            ASSERTED ir_variable *var = deref->variable_referenced();
            assert((var->is_in_buffer_block() && !matrix) ||
                   var->data.mode == ir_var_shader_shared);
            return false;
         }
         case GLSL_MATRIX_LAYOUT_COLUMN_MAJOR:
            return false;
         case GLSL_MATRIX_LAYOUT_ROW_MAJOR:
            return matrix || deref->type->without_array()->is_struct();
         }

         unreachable("invalid matrix layout");
         break;
      }

      default:
         return false;
      }
   }

   /* The tree must have ended with a dereference that wasn't an
    * ir_dereference_variable.  That is invalid, and it should be impossible.
    */
   unreachable("invalid dereference tree");
   return false;
}

/**
 * This function initializes various values that will be used later by
 * emit_access when actually emitting loads or stores.
 *
 * Note: const_offset is an input as well as an output, clients must
 * initialize it to the offset of the variable in the underlying block, and
 * this function will adjust it by adding the constant offset of the member
 * being accessed into that variable.
 */
void
lower_buffer_access::setup_buffer_access(void *mem_ctx,
                                         ir_rvalue *deref,
                                         ir_rvalue **offset,
                                         unsigned *const_offset,
                                         bool *row_major,
                                         const glsl_type **matrix_type,
                                         const glsl_struct_field **struct_field,
                                         enum glsl_interface_packing packing)
{
   *offset = new(mem_ctx) ir_constant(0u);
   *row_major = is_dereferenced_thing_row_major(deref);
   *matrix_type = NULL;

   /* Calculate the offset to the start of the region of the UBO
    * dereferenced by *rvalue.  This may be a variable offset if an
    * array dereference has a variable index.
    */
   while (deref) {
      switch (deref->ir_type) {
      case ir_type_dereference_variable: {
         deref = NULL;
         break;
      }

      case ir_type_dereference_array: {
         ir_dereference_array *deref_array = (ir_dereference_array *) deref;
         unsigned array_stride;
         if (deref_array->array->type->is_vector()) {
            /* We get this when storing or loading a component out of a vector
             * with a non-constant index. This happens for v[i] = f where v is
             * a vector (or m[i][j] = f where m is a matrix). If we don't
             * lower that here, it gets turned into v = vector_insert(v, i,
             * f), which loads the entire vector, modifies one component and
             * then write the entire thing back.  That breaks if another
             * thread or SIMD channel is modifying the same vector.
             */
            array_stride = 4;
            if (deref_array->array->type->is_64bit())
               array_stride *= 2;
         } else if (deref_array->array->type->is_matrix() && *row_major) {
            /* When loading a vector out of a row major matrix, the
             * step between the columns (vectors) is the size of a
             * float, while the step between the rows (elements of a
             * vector) is handled below in emit_ubo_loads.
             */
            array_stride = 4;
            if (deref_array->array->type->is_64bit())
               array_stride *= 2;
            *matrix_type = deref_array->array->type;
         } else if (deref_array->type->without_array()->is_interface()) {
            /* We're processing an array dereference of an interface instance
             * array. The thing being dereferenced *must* be a variable
             * dereference because interfaces cannot be embedded in other
             * types. In terms of calculating the offsets for the lowering
             * pass, we don't care about the array index. All elements of an
             * interface instance array will have the same offsets relative to
             * the base of the block that backs them.
             */
            deref = deref_array->array->as_dereference();
            break;
         } else {
            /* Whether or not the field is row-major (because it might be a
             * bvec2 or something) does not affect the array itself. We need
             * to know whether an array element in its entirety is row-major.
             */
            const bool array_row_major =
               is_dereferenced_thing_row_major(deref_array);

            /* The array type will give the correct interface packing
             * information
             */
            if (packing == GLSL_INTERFACE_PACKING_STD430) {
               array_stride = deref_array->type->std430_array_stride(array_row_major);
            } else {
               array_stride = deref_array->type->std140_size(array_row_major);
               array_stride = glsl_align(array_stride, 16);
            }
         }

         ir_rvalue *array_index = deref_array->array_index;
         if (array_index->type->base_type == GLSL_TYPE_INT)
            array_index = i2u(array_index);

         ir_constant *const_index =
            array_index->constant_expression_value(mem_ctx, NULL);
         if (const_index) {
            *const_offset += array_stride * const_index->value.u[0];
         } else {
            *offset = add(*offset,
                          mul(array_index,
                              new(mem_ctx) ir_constant(array_stride)));
         }
         deref = deref_array->array->as_dereference();
         break;
      }

      case ir_type_dereference_record: {
         ir_dereference_record *deref_record = (ir_dereference_record *) deref;
         const glsl_type *struct_type = deref_record->record->type;
         unsigned intra_struct_offset = 0;

         for (unsigned int i = 0; i < struct_type->length; i++) {
            const glsl_type *type = struct_type->fields.structure[i].type;

            ir_dereference_record *field_deref = new(mem_ctx)
               ir_dereference_record(deref_record->record,
                                     struct_type->fields.structure[i].name);
            const bool field_row_major =
               is_dereferenced_thing_row_major(field_deref);

            ralloc_free(field_deref);

            unsigned field_align = 0;

            if (packing == GLSL_INTERFACE_PACKING_STD430)
               field_align = type->std430_base_alignment(field_row_major);
            else
               field_align = type->std140_base_alignment(field_row_major);

            if (struct_type->fields.structure[i].offset != -1) {
               intra_struct_offset = struct_type->fields.structure[i].offset;
            }

            intra_struct_offset = glsl_align(intra_struct_offset, field_align);

            assert(deref_record->field_idx >= 0);
            if (i == (unsigned) deref_record->field_idx) {
               if (struct_field)
                  *struct_field = &struct_type->fields.structure[i];
               break;
            }

            if (packing == GLSL_INTERFACE_PACKING_STD430)
               intra_struct_offset += type->std430_size(field_row_major);
            else
               intra_struct_offset += type->std140_size(field_row_major);

            /* If the field just examined was itself a structure, apply rule
             * #9:
             *
             *     "The structure may have padding at the end; the base offset
             *     of the member following the sub-structure is rounded up to
             *     the next multiple of the base alignment of the structure."
             */
            if (type->without_array()->is_struct()) {
               intra_struct_offset = glsl_align(intra_struct_offset,
                                                field_align);

            }
         }

         *const_offset += intra_struct_offset;
         deref = deref_record->record->as_dereference();
         break;
      }

      case ir_type_swizzle: {
         ir_swizzle *deref_swizzle = (ir_swizzle *) deref;

         assert(deref_swizzle->mask.num_components == 1);

         *const_offset += deref_swizzle->mask.x * sizeof(int);
         deref = deref_swizzle->val->as_dereference();
         break;
      }

      default:
         assert(!"not reached");
         deref = NULL;
         break;
      }
   }
}

} /* namespace lower_buffer_access */
