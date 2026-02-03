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


/**
 * Building this file with MinGW g++ 7.3 or 7.4 with:
 *   scons platform=windows toolchain=crossmingw machine=x86 build=profile
 * triggers an internal compiler error.
 * Overriding the optimization level to -O1 works around the issue.
 * MinGW 5.3.1 does not seem to have the bug, neither does 8.3.  So for now
 * we're simply testing for version 7.x here.
 */
#if defined(__MINGW32__) && __GNUC__ == 7
#warning "disabling optimizations for this file to work around compiler bug in MinGW gcc 7.x"
#pragma GCC optimize("O1")
#endif


#include "ir.h"
#include "ir_builder.h"
#include "linker.h"
#include "glsl_parser_extras.h"
#include "glsl_symbol_table.h"
#include "main/mtypes.h"
#include "main/uniforms.h"
#include "program/prog_statevars.h"
#include "program/prog_instruction.h"
#include "builtin_functions.h"

using namespace ir_builder;

static const struct gl_builtin_uniform_element gl_NumSamples_elements[] = {
   {NULL, {STATE_NUM_SAMPLES, 0, 0}, SWIZZLE_XXXX}
};

static const struct gl_builtin_uniform_element gl_DepthRange_elements[] = {
   {"near", {STATE_DEPTH_RANGE, 0, 0}, SWIZZLE_XXXX},
   {"far", {STATE_DEPTH_RANGE, 0, 0}, SWIZZLE_YYYY},
   {"diff", {STATE_DEPTH_RANGE, 0, 0}, SWIZZLE_ZZZZ},
};

static const struct gl_builtin_uniform_element gl_ClipPlane_elements[] = {
   {NULL, {STATE_CLIPPLANE, 0, 0}, SWIZZLE_XYZW}
};

static const struct gl_builtin_uniform_element gl_Point_elements[] = {
   {"size", {STATE_POINT_SIZE}, SWIZZLE_XXXX},
   {"sizeMin", {STATE_POINT_SIZE}, SWIZZLE_YYYY},
   {"sizeMax", {STATE_POINT_SIZE}, SWIZZLE_ZZZZ},
   {"fadeThresholdSize", {STATE_POINT_SIZE}, SWIZZLE_WWWW},
   {"distanceConstantAttenuation", {STATE_POINT_ATTENUATION}, SWIZZLE_XXXX},
   {"distanceLinearAttenuation", {STATE_POINT_ATTENUATION}, SWIZZLE_YYYY},
   {"distanceQuadraticAttenuation", {STATE_POINT_ATTENUATION}, SWIZZLE_ZZZZ},
};

static const struct gl_builtin_uniform_element gl_FrontMaterial_elements[] = {
   {"emission", {STATE_MATERIAL, 0, STATE_EMISSION}, SWIZZLE_XYZW},
   {"ambient", {STATE_MATERIAL, 0, STATE_AMBIENT}, SWIZZLE_XYZW},
   {"diffuse", {STATE_MATERIAL, 0, STATE_DIFFUSE}, SWIZZLE_XYZW},
   {"specular", {STATE_MATERIAL, 0, STATE_SPECULAR}, SWIZZLE_XYZW},
   {"shininess", {STATE_MATERIAL, 0, STATE_SHININESS}, SWIZZLE_XXXX},
};

static const struct gl_builtin_uniform_element gl_BackMaterial_elements[] = {
   {"emission", {STATE_MATERIAL, 1, STATE_EMISSION}, SWIZZLE_XYZW},
   {"ambient", {STATE_MATERIAL, 1, STATE_AMBIENT}, SWIZZLE_XYZW},
   {"diffuse", {STATE_MATERIAL, 1, STATE_DIFFUSE}, SWIZZLE_XYZW},
   {"specular", {STATE_MATERIAL, 1, STATE_SPECULAR}, SWIZZLE_XYZW},
   {"shininess", {STATE_MATERIAL, 1, STATE_SHININESS}, SWIZZLE_XXXX},
};

static const struct gl_builtin_uniform_element gl_LightSource_elements[] = {
   {"ambient", {STATE_LIGHT, 0, STATE_AMBIENT}, SWIZZLE_XYZW},
   {"diffuse", {STATE_LIGHT, 0, STATE_DIFFUSE}, SWIZZLE_XYZW},
   {"specular", {STATE_LIGHT, 0, STATE_SPECULAR}, SWIZZLE_XYZW},
   {"position", {STATE_LIGHT, 0, STATE_POSITION}, SWIZZLE_XYZW},
   {"halfVector", {STATE_LIGHT, 0, STATE_HALF_VECTOR}, SWIZZLE_XYZW},
   {"spotDirection", {STATE_LIGHT, 0, STATE_SPOT_DIRECTION},
    MAKE_SWIZZLE4(SWIZZLE_X,
		  SWIZZLE_Y,
		  SWIZZLE_Z,
		  SWIZZLE_Z)},
   {"spotExponent", {STATE_LIGHT, 0, STATE_ATTENUATION}, SWIZZLE_WWWW},
   {"spotCutoff", {STATE_LIGHT, 0, STATE_SPOT_CUTOFF}, SWIZZLE_XXXX},
   {"spotCosCutoff", {STATE_LIGHT, 0, STATE_SPOT_DIRECTION}, SWIZZLE_WWWW},
   {"constantAttenuation", {STATE_LIGHT, 0, STATE_ATTENUATION}, SWIZZLE_XXXX},
   {"linearAttenuation", {STATE_LIGHT, 0, STATE_ATTENUATION}, SWIZZLE_YYYY},
   {"quadraticAttenuation", {STATE_LIGHT, 0, STATE_ATTENUATION}, SWIZZLE_ZZZZ},
};

static const struct gl_builtin_uniform_element gl_LightModel_elements[] = {
   {"ambient", {STATE_LIGHTMODEL_AMBIENT, 0}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_FrontLightModelProduct_elements[] = {
   {"sceneColor", {STATE_LIGHTMODEL_SCENECOLOR, 0}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_BackLightModelProduct_elements[] = {
   {"sceneColor", {STATE_LIGHTMODEL_SCENECOLOR, 1}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_FrontLightProduct_elements[] = {
   {"ambient", {STATE_LIGHTPROD, 0, 0, STATE_AMBIENT}, SWIZZLE_XYZW},
   {"diffuse", {STATE_LIGHTPROD, 0, 0, STATE_DIFFUSE}, SWIZZLE_XYZW},
   {"specular", {STATE_LIGHTPROD, 0, 0, STATE_SPECULAR}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_BackLightProduct_elements[] = {
   {"ambient", {STATE_LIGHTPROD, 0, 1, STATE_AMBIENT}, SWIZZLE_XYZW},
   {"diffuse", {STATE_LIGHTPROD, 0, 1, STATE_DIFFUSE}, SWIZZLE_XYZW},
   {"specular", {STATE_LIGHTPROD, 0, 1, STATE_SPECULAR}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_TextureEnvColor_elements[] = {
   {NULL, {STATE_TEXENV_COLOR, 0}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_EyePlaneS_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_EYE_S}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_EyePlaneT_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_EYE_T}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_EyePlaneR_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_EYE_R}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_EyePlaneQ_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_EYE_Q}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_ObjectPlaneS_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_OBJECT_S}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_ObjectPlaneT_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_OBJECT_T}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_ObjectPlaneR_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_OBJECT_R}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_ObjectPlaneQ_elements[] = {
   {NULL, {STATE_TEXGEN, 0, STATE_TEXGEN_OBJECT_Q}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_Fog_elements[] = {
   {"color", {STATE_FOG_COLOR}, SWIZZLE_XYZW},
   {"density", {STATE_FOG_PARAMS}, SWIZZLE_XXXX},
   {"start", {STATE_FOG_PARAMS}, SWIZZLE_YYYY},
   {"end", {STATE_FOG_PARAMS}, SWIZZLE_ZZZZ},
   {"scale", {STATE_FOG_PARAMS}, SWIZZLE_WWWW},
};

static const struct gl_builtin_uniform_element gl_NormalScale_elements[] = {
   {NULL, {STATE_NORMAL_SCALE}, SWIZZLE_XXXX},
};

static const struct gl_builtin_uniform_element gl_FogParamsOptimizedMESA_elements[] = {
   {NULL, {STATE_INTERNAL, STATE_FOG_PARAMS_OPTIMIZED}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_CurrentAttribVertMESA_elements[] = {
   {NULL, {STATE_INTERNAL, STATE_CURRENT_ATTRIB, 0}, SWIZZLE_XYZW},
};

static const struct gl_builtin_uniform_element gl_CurrentAttribFragMESA_elements[] = {
   {NULL, {STATE_INTERNAL, STATE_CURRENT_ATTRIB_MAYBE_VP_CLAMPED, 0}, SWIZZLE_XYZW},
};

#define MATRIX(name, statevar, modifier)				\
   static const struct gl_builtin_uniform_element name ## _elements[] = { \
      { NULL, { statevar, 0, 0, 0, modifier}, SWIZZLE_XYZW },		\
      { NULL, { statevar, 0, 1, 1, modifier}, SWIZZLE_XYZW },		\
      { NULL, { statevar, 0, 2, 2, modifier}, SWIZZLE_XYZW },		\
      { NULL, { statevar, 0, 3, 3, modifier}, SWIZZLE_XYZW },		\
   }

MATRIX(gl_ModelViewMatrix,
       STATE_MODELVIEW_MATRIX, STATE_MATRIX_TRANSPOSE);
MATRIX(gl_ModelViewMatrixInverse,
       STATE_MODELVIEW_MATRIX, STATE_MATRIX_INVTRANS);
MATRIX(gl_ModelViewMatrixTranspose,
       STATE_MODELVIEW_MATRIX, 0);
MATRIX(gl_ModelViewMatrixInverseTranspose,
       STATE_MODELVIEW_MATRIX, STATE_MATRIX_INVERSE);

MATRIX(gl_ProjectionMatrix,
       STATE_PROJECTION_MATRIX, STATE_MATRIX_TRANSPOSE);
MATRIX(gl_ProjectionMatrixInverse,
       STATE_PROJECTION_MATRIX, STATE_MATRIX_INVTRANS);
MATRIX(gl_ProjectionMatrixTranspose,
       STATE_PROJECTION_MATRIX, 0);
MATRIX(gl_ProjectionMatrixInverseTranspose,
       STATE_PROJECTION_MATRIX, STATE_MATRIX_INVERSE);

MATRIX(gl_ModelViewProjectionMatrix,
       STATE_MVP_MATRIX, STATE_MATRIX_TRANSPOSE);
MATRIX(gl_ModelViewProjectionMatrixInverse,
       STATE_MVP_MATRIX, STATE_MATRIX_INVTRANS);
MATRIX(gl_ModelViewProjectionMatrixTranspose,
       STATE_MVP_MATRIX, 0);
MATRIX(gl_ModelViewProjectionMatrixInverseTranspose,
       STATE_MVP_MATRIX, STATE_MATRIX_INVERSE);

MATRIX(gl_TextureMatrix,
       STATE_TEXTURE_MATRIX, STATE_MATRIX_TRANSPOSE);
MATRIX(gl_TextureMatrixInverse,
       STATE_TEXTURE_MATRIX, STATE_MATRIX_INVTRANS);
MATRIX(gl_TextureMatrixTranspose,
       STATE_TEXTURE_MATRIX, 0);
MATRIX(gl_TextureMatrixInverseTranspose,
       STATE_TEXTURE_MATRIX, STATE_MATRIX_INVERSE);

static const struct gl_builtin_uniform_element gl_NormalMatrix_elements[] = {
   { NULL, { STATE_MODELVIEW_MATRIX, 0, 0, 0, STATE_MATRIX_INVERSE},
     MAKE_SWIZZLE4(SWIZZLE_X, SWIZZLE_Y, SWIZZLE_Z, SWIZZLE_Z) },
   { NULL, { STATE_MODELVIEW_MATRIX, 0, 1, 1, STATE_MATRIX_INVERSE},
     MAKE_SWIZZLE4(SWIZZLE_X, SWIZZLE_Y, SWIZZLE_Z, SWIZZLE_Z) },
   { NULL, { STATE_MODELVIEW_MATRIX, 0, 2, 2, STATE_MATRIX_INVERSE},
     MAKE_SWIZZLE4(SWIZZLE_X, SWIZZLE_Y, SWIZZLE_Z, SWIZZLE_Z) },
};

#undef MATRIX

#define STATEVAR(name) {#name, name ## _elements, ARRAY_SIZE(name ## _elements)}

static const struct gl_builtin_uniform_desc _mesa_builtin_uniform_desc[] = {
   STATEVAR(gl_NumSamples),
   STATEVAR(gl_DepthRange),
   STATEVAR(gl_ClipPlane),
   STATEVAR(gl_Point),
   STATEVAR(gl_FrontMaterial),
   STATEVAR(gl_BackMaterial),
   STATEVAR(gl_LightSource),
   STATEVAR(gl_LightModel),
   STATEVAR(gl_FrontLightModelProduct),
   STATEVAR(gl_BackLightModelProduct),
   STATEVAR(gl_FrontLightProduct),
   STATEVAR(gl_BackLightProduct),
   STATEVAR(gl_TextureEnvColor),
   STATEVAR(gl_EyePlaneS),
   STATEVAR(gl_EyePlaneT),
   STATEVAR(gl_EyePlaneR),
   STATEVAR(gl_EyePlaneQ),
   STATEVAR(gl_ObjectPlaneS),
   STATEVAR(gl_ObjectPlaneT),
   STATEVAR(gl_ObjectPlaneR),
   STATEVAR(gl_ObjectPlaneQ),
   STATEVAR(gl_Fog),

   STATEVAR(gl_ModelViewMatrix),
   STATEVAR(gl_ModelViewMatrixInverse),
   STATEVAR(gl_ModelViewMatrixTranspose),
   STATEVAR(gl_ModelViewMatrixInverseTranspose),

   STATEVAR(gl_ProjectionMatrix),
   STATEVAR(gl_ProjectionMatrixInverse),
   STATEVAR(gl_ProjectionMatrixTranspose),
   STATEVAR(gl_ProjectionMatrixInverseTranspose),

   STATEVAR(gl_ModelViewProjectionMatrix),
   STATEVAR(gl_ModelViewProjectionMatrixInverse),
   STATEVAR(gl_ModelViewProjectionMatrixTranspose),
   STATEVAR(gl_ModelViewProjectionMatrixInverseTranspose),

   STATEVAR(gl_TextureMatrix),
   STATEVAR(gl_TextureMatrixInverse),
   STATEVAR(gl_TextureMatrixTranspose),
   STATEVAR(gl_TextureMatrixInverseTranspose),

   STATEVAR(gl_NormalMatrix),
   STATEVAR(gl_NormalScale),

   STATEVAR(gl_FogParamsOptimizedMESA),
   STATEVAR(gl_CurrentAttribVertMESA),
   STATEVAR(gl_CurrentAttribFragMESA),

   {NULL, NULL, 0}
};


namespace {

/**
 * Data structure that accumulates fields for the gl_PerVertex interface
 * block.
 */
class per_vertex_accumulator
{
public:
   per_vertex_accumulator();
   void add_field(int slot, const glsl_type *type, int precision,
                  const char *name);
   const glsl_type *construct_interface_instance() const;

private:
   glsl_struct_field fields[11];
   unsigned num_fields;
};


per_vertex_accumulator::per_vertex_accumulator()
   : fields(),
     num_fields(0)
{
}


void
per_vertex_accumulator::add_field(int slot, const glsl_type *type,
                                  int precision, const char *name)
{
   assert(this->num_fields < ARRAY_SIZE(this->fields));
   this->fields[this->num_fields].type = type;
   this->fields[this->num_fields].name = name;
   this->fields[this->num_fields].matrix_layout = GLSL_MATRIX_LAYOUT_INHERITED;
   this->fields[this->num_fields].location = slot;
   this->fields[this->num_fields].offset = -1;
   this->fields[this->num_fields].interpolation = INTERP_MODE_NONE;
   this->fields[this->num_fields].centroid = 0;
   this->fields[this->num_fields].sample = 0;
   this->fields[this->num_fields].patch = 0;
   this->fields[this->num_fields].precision = precision;
   this->fields[this->num_fields].memory_read_only = 0;
   this->fields[this->num_fields].memory_write_only = 0;
   this->fields[this->num_fields].memory_coherent = 0;
   this->fields[this->num_fields].memory_volatile = 0;
   this->fields[this->num_fields].memory_restrict = 0;
   this->fields[this->num_fields].image_format = PIPE_FORMAT_NONE;
   this->fields[this->num_fields].explicit_xfb_buffer = 0;
   this->fields[this->num_fields].xfb_buffer = -1;
   this->fields[this->num_fields].xfb_stride = -1;
   this->num_fields++;
}


const glsl_type *
per_vertex_accumulator::construct_interface_instance() const
{
   return glsl_type::get_interface_instance(this->fields, this->num_fields,
                                            GLSL_INTERFACE_PACKING_STD140,
                                            false,
                                            "gl_PerVertex");
}


class builtin_variable_generator
{
public:
   builtin_variable_generator(exec_list *instructions,
                              struct _mesa_glsl_parse_state *state);
   void generate_constants();
   void generate_uniforms();
   void generate_special_vars();
   void generate_vs_special_vars();
   void generate_tcs_special_vars();
   void generate_tes_special_vars();
   void generate_gs_special_vars();
   void generate_fs_special_vars();
   void generate_cs_special_vars();
   void generate_varyings();

private:
   const glsl_type *array(const glsl_type *base, unsigned elements)
   {
      return glsl_type::get_array_instance(base, elements);
   }

   const glsl_type *type(const char *name)
   {
      return symtab->get_type(name);
   }

   ir_variable *add_input(int slot, const glsl_type *type, int precision,
                          const char *name)
   {
      return add_variable(name, type, precision, ir_var_shader_in, slot);
   }

   ir_variable *add_input(int slot, const glsl_type *type, const char *name)
   {
      return add_input(slot, type, GLSL_PRECISION_NONE, name);
   }

   ir_variable *add_output(int slot, const glsl_type *type, int precision,
                           const char *name)
   {
      return add_variable(name, type, precision, ir_var_shader_out, slot);
   }

   ir_variable *add_output(int slot, const glsl_type *type, const char *name)
   {
      return add_output(slot, type, GLSL_PRECISION_NONE, name);
   }

   ir_variable *add_index_output(int slot, int index, const glsl_type *type,
                                 int precision, const char *name)
   {
      return add_index_variable(name, type, precision, ir_var_shader_out, slot,
                                index);
   }

   ir_variable *add_system_value(int slot, const glsl_type *type, int precision,
                                 const char *name)
   {
      return add_variable(name, type, precision, ir_var_system_value, slot);
   }
   ir_variable *add_system_value(int slot, const glsl_type *type,
                                 const char *name)
   {
      return add_system_value(slot, type, GLSL_PRECISION_NONE, name);
   }

   ir_variable *add_variable(const char *name, const glsl_type *type,
                             int precision, enum ir_variable_mode mode,
                             int slot);
   ir_variable *add_index_variable(const char *name, const glsl_type *type,
                                   int precision, enum ir_variable_mode mode,
                                   int slot, int index);
   ir_variable *add_uniform(const glsl_type *type, int precision,
                            const char *name);
   ir_variable *add_uniform(const glsl_type *type, const char *name)
   {
      return add_uniform(type, GLSL_PRECISION_NONE, name);
   }
   ir_variable *add_const(const char *name, int precision, int value);
   ir_variable *add_const(const char *name, int value)
   {
      return add_const(name, GLSL_PRECISION_MEDIUM, value);
   }
   ir_variable *add_const_ivec3(const char *name, int x, int y, int z);
   void add_varying(int slot, const glsl_type *type, int precision,
                    const char *name);
   void add_varying(int slot, const glsl_type *type, const char *name)
   {
      add_varying(slot, type, GLSL_PRECISION_NONE, name);
   }

   exec_list * const instructions;
   struct _mesa_glsl_parse_state * const state;
   glsl_symbol_table * const symtab;

   /**
    * True if compatibility-profile-only variables should be included.  (In
    * desktop GL, these are always included when the GLSL version is 1.30 and
    * or below).
    */
   const bool compatibility;

   const glsl_type * const bool_t;
   const glsl_type * const int_t;
   const glsl_type * const uint_t;
   const glsl_type * const uint64_t;
   const glsl_type * const float_t;
   const glsl_type * const vec2_t;
   const glsl_type * const vec3_t;
   const glsl_type * const vec4_t;
   const glsl_type * const uvec3_t;
   const glsl_type * const mat3_t;
   const glsl_type * const mat4_t;

   per_vertex_accumulator per_vertex_in;
   per_vertex_accumulator per_vertex_out;
};


builtin_variable_generator::builtin_variable_generator(
   exec_list *instructions, struct _mesa_glsl_parse_state *state)
   : instructions(instructions), state(state), symtab(state->symbols),
     compatibility(state->compat_shader || state->ARB_compatibility_enable),
     bool_t(glsl_type::bool_type), int_t(glsl_type::int_type),
     uint_t(glsl_type::uint_type),
     uint64_t(glsl_type::uint64_t_type),
     float_t(glsl_type::float_type), vec2_t(glsl_type::vec2_type),
     vec3_t(glsl_type::vec3_type), vec4_t(glsl_type::vec4_type),
     uvec3_t(glsl_type::uvec3_type),
     mat3_t(glsl_type::mat3_type), mat4_t(glsl_type::mat4_type)
{
}

ir_variable *
builtin_variable_generator::add_index_variable(const char *name,
                                               const glsl_type *type,
                                               int precision,
                                               enum ir_variable_mode mode,
                                               int slot, int index)
{
   ir_variable *var = new(symtab) ir_variable(type, name, mode);
   var->data.how_declared = ir_var_declared_implicitly;

   switch (var->data.mode) {
   case ir_var_auto:
   case ir_var_shader_in:
   case ir_var_uniform:
   case ir_var_system_value:
      var->data.read_only = true;
      break;
   case ir_var_shader_out:
   case ir_var_shader_storage:
      break;
   default:
      /* The only variables that are added using this function should be
       * uniforms, shader storage, shader inputs, and shader outputs, constants
       * (which use ir_var_auto), and system values.
       */
      assert(0);
      break;
   }

   var->data.location = slot;
   var->data.explicit_location = (slot >= 0);
   var->data.explicit_index = 1;
   var->data.index = index;

   if (state->es_shader)
      var->data.precision = precision;

   /* Once the variable is created an initialized, add it to the symbol table
    * and add the declaration to the IR stream.
    */
   instructions->push_tail(var);

   symtab->add_variable(var);
   return var;
}

ir_variable *
builtin_variable_generator::add_variable(const char *name,
                                         const glsl_type *type,
                                         int precision,
                                         enum ir_variable_mode mode, int slot)
{
   ir_variable *var = new(symtab) ir_variable(type, name, mode);
   var->data.how_declared = ir_var_declared_implicitly;

   switch (var->data.mode) {
   case ir_var_auto:
   case ir_var_shader_in:
   case ir_var_uniform:
   case ir_var_system_value:
      var->data.read_only = true;
      break;
   case ir_var_shader_out:
   case ir_var_shader_storage:
      break;
   default:
      /* The only variables that are added using this function should be
       * uniforms, shader storage, shader inputs, and shader outputs, constants
       * (which use ir_var_auto), and system values.
       */
      assert(0);
      break;
   }

   var->data.location = slot;
   var->data.explicit_location = (slot >= 0);
   var->data.explicit_index = 0;

   if (state->es_shader)
      var->data.precision = precision;

   /* Once the variable is created an initialized, add it to the symbol table
    * and add the declaration to the IR stream.
    */
   instructions->push_tail(var);

   symtab->add_variable(var);
   return var;
}

extern "C" const struct gl_builtin_uniform_desc *
_mesa_glsl_get_builtin_uniform_desc(const char *name)
{
   for (unsigned i = 0; _mesa_builtin_uniform_desc[i].name != NULL; i++) {
      if (strcmp(_mesa_builtin_uniform_desc[i].name, name) == 0) {
         return &_mesa_builtin_uniform_desc[i];
      }
   }
   return NULL;
}

ir_variable *
builtin_variable_generator::add_uniform(const glsl_type *type,
                                        int precision,
                                        const char *name)
{
   ir_variable *const uni =
      add_variable(name, type, precision, ir_var_uniform, -1);

   const struct gl_builtin_uniform_desc* const statevar =
      _mesa_glsl_get_builtin_uniform_desc(name);
   assert(statevar != NULL);

   const unsigned array_count = type->is_array() ? type->length : 1;

   ir_state_slot *slots =
      uni->allocate_state_slots(array_count * statevar->num_elements);

   for (unsigned a = 0; a < array_count; a++) {
      for (unsigned j = 0; j < statevar->num_elements; j++) {
	 const struct gl_builtin_uniform_element *element =
	    &statevar->elements[j];

	 memcpy(slots->tokens, element->tokens, sizeof(element->tokens));
	 if (type->is_array()) {
	    if (strcmp(name, "gl_CurrentAttribVertMESA") == 0 ||
		strcmp(name, "gl_CurrentAttribFragMESA") == 0) {
	       slots->tokens[2] = a;
	    } else {
	       slots->tokens[1] = a;
	    }
	 }

	 slots->swizzle = element->swizzle;
	 slots++;
      }
   }

   return uni;
}


ir_variable *
builtin_variable_generator::add_const(const char *name, int precision,
                                      int value)
{
   ir_variable *const var = add_variable(name, glsl_type::int_type,
                                         precision, ir_var_auto, -1);
   var->constant_value = new(var) ir_constant(value);
   var->constant_initializer = new(var) ir_constant(value);
   var->data.has_initializer = true;
   return var;
}


ir_variable *
builtin_variable_generator::add_const_ivec3(const char *name, int x, int y,
                                            int z)
{
   ir_variable *const var = add_variable(name, glsl_type::ivec3_type,
                                         GLSL_PRECISION_HIGH,
                                         ir_var_auto, -1);
   ir_constant_data data;
   memset(&data, 0, sizeof(data));
   data.i[0] = x;
   data.i[1] = y;
   data.i[2] = z;
   var->constant_value = new(var) ir_constant(glsl_type::ivec3_type, &data);
   var->constant_initializer =
      new(var) ir_constant(glsl_type::ivec3_type, &data);
   var->data.has_initializer = true;
   return var;
}


void
builtin_variable_generator::generate_constants()
{
   add_const("gl_MaxVertexAttribs", state->Const.MaxVertexAttribs);
   add_const("gl_MaxVertexTextureImageUnits",
             state->Const.MaxVertexTextureImageUnits);
   add_const("gl_MaxCombinedTextureImageUnits",
             state->Const.MaxCombinedTextureImageUnits);
   add_const("gl_MaxTextureImageUnits", state->Const.MaxTextureImageUnits);
   add_const("gl_MaxDrawBuffers", state->Const.MaxDrawBuffers);

   /* Max uniforms/varyings: GLSL ES counts these in units of vectors; desktop
    * GL counts them in units of "components" or "floats" and also in units
    * of vectors since GL 4.1
    */
   if (!state->es_shader) {
      add_const("gl_MaxFragmentUniformComponents",
                state->Const.MaxFragmentUniformComponents);
      add_const("gl_MaxVertexUniformComponents",
                state->Const.MaxVertexUniformComponents);
   }

   if (state->is_version(410, 100)) {
      add_const("gl_MaxVertexUniformVectors",
                state->Const.MaxVertexUniformComponents / 4);
      add_const("gl_MaxFragmentUniformVectors",
                state->Const.MaxFragmentUniformComponents / 4);

      /* In GLSL ES 3.00, gl_MaxVaryingVectors was split out to separate
       * vertex and fragment shader constants.
       */
      if (state->is_version(0, 300)) {
         add_const("gl_MaxVertexOutputVectors",
                   state->ctx->Const.Program[MESA_SHADER_VERTEX].MaxOutputComponents / 4);
         add_const("gl_MaxFragmentInputVectors",
                   state->ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxInputComponents / 4);
      } else {
         add_const("gl_MaxVaryingVectors",
                   state->ctx->Const.MaxVarying);
      }

      /* EXT_blend_func_extended brings a built in constant
       * for determining number of dual source draw buffers
       */
      if (state->EXT_blend_func_extended_enable) {
         add_const("gl_MaxDualSourceDrawBuffersEXT",
                   state->Const.MaxDualSourceDrawBuffers);
      }
   } else {
      /* Note: gl_MaxVaryingFloats was deprecated in GLSL 1.30+, but not
       * removed
       */
      add_const("gl_MaxVaryingFloats", state->ctx->Const.MaxVarying * 4);
   }

   /* Texel offsets were introduced in ARB_shading_language_420pack (which
    * requires desktop GLSL version 130), and adopted into desktop GLSL
    * version 4.20 and GLSL ES version 3.00.
    */
   if ((state->is_version(130, 0) &&
        state->ARB_shading_language_420pack_enable) ||
      state->is_version(420, 300)) {
      add_const("gl_MinProgramTexelOffset",
                state->Const.MinProgramTexelOffset);
      add_const("gl_MaxProgramTexelOffset",
                state->Const.MaxProgramTexelOffset);
   }

   if (state->has_clip_distance()) {
      add_const("gl_MaxClipDistances", state->Const.MaxClipPlanes);
   }
   if (state->is_version(130, 0)) {
      add_const("gl_MaxVaryingComponents", state->ctx->Const.MaxVarying * 4);
   }
   if (state->has_cull_distance()) {
      add_const("gl_MaxCullDistances", state->Const.MaxClipPlanes);
      add_const("gl_MaxCombinedClipAndCullDistances",
                state->Const.MaxClipPlanes);
   }

   if (state->has_geometry_shader()) {
      add_const("gl_MaxVertexOutputComponents",
                state->Const.MaxVertexOutputComponents);
      add_const("gl_MaxGeometryInputComponents",
                state->Const.MaxGeometryInputComponents);
      add_const("gl_MaxGeometryOutputComponents",
                state->Const.MaxGeometryOutputComponents);
      add_const("gl_MaxFragmentInputComponents",
                state->Const.MaxFragmentInputComponents);
      add_const("gl_MaxGeometryTextureImageUnits",
                state->Const.MaxGeometryTextureImageUnits);
      add_const("gl_MaxGeometryOutputVertices",
                state->Const.MaxGeometryOutputVertices);
      add_const("gl_MaxGeometryTotalOutputComponents",
                state->Const.MaxGeometryTotalOutputComponents);
      add_const("gl_MaxGeometryUniformComponents",
                state->Const.MaxGeometryUniformComponents);

      /* Note: the GLSL 1.50-4.40 specs require
       * gl_MaxGeometryVaryingComponents to be present, and to be at least 64.
       * But they do not define what it means (and there does not appear to be
       * any corresponding constant in the GL specs).  However,
       * ARB_geometry_shader4 defines MAX_GEOMETRY_VARYING_COMPONENTS_ARB to
       * be the maximum number of components available for use as geometry
       * outputs.  So we assume this is a synonym for
       * gl_MaxGeometryOutputComponents.
       */
      add_const("gl_MaxGeometryVaryingComponents",
                state->Const.MaxGeometryOutputComponents);
   }

   if (compatibility) {
      /* Note: gl_MaxLights stopped being listed as an explicit constant in
       * GLSL 1.30, however it continues to be referred to (as a minimum size
       * for compatibility-mode uniforms) all the way up through GLSL 4.30, so
       * this seems like it was probably an oversight.
       */
      add_const("gl_MaxLights", state->Const.MaxLights);

      add_const("gl_MaxClipPlanes", state->Const.MaxClipPlanes);

      /* Note: gl_MaxTextureUnits wasn't made compatibility-only until GLSL
       * 1.50, however this seems like it was probably an oversight.
       */
      add_const("gl_MaxTextureUnits", state->Const.MaxTextureUnits);

      /* Note: gl_MaxTextureCoords was left out of GLSL 1.40, but it was
       * re-introduced in GLSL 1.50, so this seems like it was probably an
       * oversight.
       */
      add_const("gl_MaxTextureCoords", state->Const.MaxTextureCoords);
   }

   if (state->has_atomic_counters()) {
      add_const("gl_MaxVertexAtomicCounters",
                state->Const.MaxVertexAtomicCounters);
      add_const("gl_MaxFragmentAtomicCounters",
                state->Const.MaxFragmentAtomicCounters);
      add_const("gl_MaxCombinedAtomicCounters",
                state->Const.MaxCombinedAtomicCounters);
      add_const("gl_MaxAtomicCounterBindings",
                state->Const.MaxAtomicBufferBindings);

      if (state->has_geometry_shader()) {
         add_const("gl_MaxGeometryAtomicCounters",
                   state->Const.MaxGeometryAtomicCounters);
      }
      if (state->is_version(110, 320)) {
         add_const("gl_MaxTessControlAtomicCounters",
                   state->Const.MaxTessControlAtomicCounters);
         add_const("gl_MaxTessEvaluationAtomicCounters",
                   state->Const.MaxTessEvaluationAtomicCounters);
      }
   }

   if (state->is_version(420, 310)) {
      add_const("gl_MaxVertexAtomicCounterBuffers",
                state->Const.MaxVertexAtomicCounterBuffers);
      add_const("gl_MaxFragmentAtomicCounterBuffers",
                state->Const.MaxFragmentAtomicCounterBuffers);
      add_const("gl_MaxCombinedAtomicCounterBuffers",
                state->Const.MaxCombinedAtomicCounterBuffers);
      add_const("gl_MaxAtomicCounterBufferSize",
                state->Const.MaxAtomicCounterBufferSize);

      if (state->has_geometry_shader()) {
         add_const("gl_MaxGeometryAtomicCounterBuffers",
                   state->Const.MaxGeometryAtomicCounterBuffers);
      }
      if (state->is_version(110, 320)) {
         add_const("gl_MaxTessControlAtomicCounterBuffers",
                   state->Const.MaxTessControlAtomicCounterBuffers);
         add_const("gl_MaxTessEvaluationAtomicCounterBuffers",
                   state->Const.MaxTessEvaluationAtomicCounterBuffers);
      }
   }

   if (state->is_version(430, 310) || state->ARB_compute_shader_enable) {
      add_const("gl_MaxComputeAtomicCounterBuffers",
                state->Const.MaxComputeAtomicCounterBuffers);
      add_const("gl_MaxComputeAtomicCounters",
                state->Const.MaxComputeAtomicCounters);
      add_const("gl_MaxComputeImageUniforms",
                state->Const.MaxComputeImageUniforms);
      add_const("gl_MaxComputeTextureImageUnits",
                state->Const.MaxComputeTextureImageUnits);
      add_const("gl_MaxComputeUniformComponents",
                state->Const.MaxComputeUniformComponents);

      add_const_ivec3("gl_MaxComputeWorkGroupCount",
                      state->Const.MaxComputeWorkGroupCount[0],
                      state->Const.MaxComputeWorkGroupCount[1],
                      state->Const.MaxComputeWorkGroupCount[2]);
      add_const_ivec3("gl_MaxComputeWorkGroupSize",
                      state->Const.MaxComputeWorkGroupSize[0],
                      state->Const.MaxComputeWorkGroupSize[1],
                      state->Const.MaxComputeWorkGroupSize[2]);

      /* From the GLSL 4.40 spec, section 7.1 (Built-In Language Variables):
       *
       *     The built-in constant gl_WorkGroupSize is a compute-shader
       *     constant containing the local work-group size of the shader.  The
       *     size of the work group in the X, Y, and Z dimensions is stored in
       *     the x, y, and z components.  The constants values in
       *     gl_WorkGroupSize will match those specified in the required
       *     local_size_x, local_size_y, and local_size_z layout qualifiers
       *     for the current shader.  This is a constant so that it can be
       *     used to size arrays of memory that can be shared within the local
       *     work group.  It is a compile-time error to use gl_WorkGroupSize
       *     in a shader that does not declare a fixed local group size, or
       *     before that shader has declared a fixed local group size, using
       *     local_size_x, local_size_y, and local_size_z.
       *
       * To prevent the shader from trying to refer to gl_WorkGroupSize before
       * the layout declaration, we don't define it here.  Intead we define it
       * in ast_cs_input_layout::hir().
       */
   }

   if (state->has_enhanced_layouts()) {
      add_const("gl_MaxTransformFeedbackBuffers",
                state->Const.MaxTransformFeedbackBuffers);
      add_const("gl_MaxTransformFeedbackInterleavedComponents",
                state->Const.MaxTransformFeedbackInterleavedComponents);
   }

   if (state->has_shader_image_load_store()) {
      add_const("gl_MaxImageUnits",
                state->Const.MaxImageUnits);
      add_const("gl_MaxVertexImageUniforms",
                state->Const.MaxVertexImageUniforms);
      add_const("gl_MaxFragmentImageUniforms",
                state->Const.MaxFragmentImageUniforms);
      add_const("gl_MaxCombinedImageUniforms",
                state->Const.MaxCombinedImageUniforms);

      if (state->has_geometry_shader()) {
         add_const("gl_MaxGeometryImageUniforms",
                   state->Const.MaxGeometryImageUniforms);
      }

      if (!state->es_shader) {
         add_const("gl_MaxCombinedImageUnitsAndFragmentOutputs",
                   state->Const.MaxCombinedShaderOutputResources);
         add_const("gl_MaxImageSamples",
                   state->Const.MaxImageSamples);
      }

      if (state->has_tessellation_shader()) {
         add_const("gl_MaxTessControlImageUniforms",
                   state->Const.MaxTessControlImageUniforms);
         add_const("gl_MaxTessEvaluationImageUniforms",
                   state->Const.MaxTessEvaluationImageUniforms);
      }
   }

   if (state->is_version(440, 310) ||
       state->ARB_ES3_1_compatibility_enable) {
      add_const("gl_MaxCombinedShaderOutputResources",
                state->Const.MaxCombinedShaderOutputResources);
   }

   if (state->is_version(410, 0) ||
       state->ARB_viewport_array_enable ||
       state->OES_viewport_array_enable) {
      add_const("gl_MaxViewports", GLSL_PRECISION_HIGH,
                state->Const.MaxViewports);
   }

   if (state->has_tessellation_shader()) {
      add_const("gl_MaxPatchVertices", state->Const.MaxPatchVertices);
      add_const("gl_MaxTessGenLevel", state->Const.MaxTessGenLevel);
      add_const("gl_MaxTessControlInputComponents", state->Const.MaxTessControlInputComponents);
      add_const("gl_MaxTessControlOutputComponents", state->Const.MaxTessControlOutputComponents);
      add_const("gl_MaxTessControlTextureImageUnits", state->Const.MaxTessControlTextureImageUnits);
      add_const("gl_MaxTessEvaluationInputComponents", state->Const.MaxTessEvaluationInputComponents);
      add_const("gl_MaxTessEvaluationOutputComponents", state->Const.MaxTessEvaluationOutputComponents);
      add_const("gl_MaxTessEvaluationTextureImageUnits", state->Const.MaxTessEvaluationTextureImageUnits);
      add_const("gl_MaxTessPatchComponents", state->Const.MaxTessPatchComponents);
      add_const("gl_MaxTessControlTotalOutputComponents", state->Const.MaxTessControlTotalOutputComponents);
      add_const("gl_MaxTessControlUniformComponents", state->Const.MaxTessControlUniformComponents);
      add_const("gl_MaxTessEvaluationUniformComponents", state->Const.MaxTessEvaluationUniformComponents);
   }

   if (state->is_version(450, 320) ||
       state->OES_sample_variables_enable ||
       state->ARB_ES3_1_compatibility_enable)
      add_const("gl_MaxSamples", state->Const.MaxSamples);
}


/**
 * Generate uniform variables (which exist in all types of shaders).
 */
void
builtin_variable_generator::generate_uniforms()
{
   if (state->is_version(400, 320) ||
       state->ARB_sample_shading_enable ||
       state->OES_sample_variables_enable)
      add_uniform(int_t, GLSL_PRECISION_LOW, "gl_NumSamples");
   add_uniform(type("gl_DepthRangeParameters"), "gl_DepthRange");
   add_uniform(array(vec4_t, VERT_ATTRIB_MAX), "gl_CurrentAttribVertMESA");
   add_uniform(array(vec4_t, VARYING_SLOT_MAX), "gl_CurrentAttribFragMESA");

   if (compatibility) {
      add_uniform(mat4_t, "gl_ModelViewMatrix");
      add_uniform(mat4_t, "gl_ProjectionMatrix");
      add_uniform(mat4_t, "gl_ModelViewProjectionMatrix");
      add_uniform(mat3_t, "gl_NormalMatrix");
      add_uniform(mat4_t, "gl_ModelViewMatrixInverse");
      add_uniform(mat4_t, "gl_ProjectionMatrixInverse");
      add_uniform(mat4_t, "gl_ModelViewProjectionMatrixInverse");
      add_uniform(mat4_t, "gl_ModelViewMatrixTranspose");
      add_uniform(mat4_t, "gl_ProjectionMatrixTranspose");
      add_uniform(mat4_t, "gl_ModelViewProjectionMatrixTranspose");
      add_uniform(mat4_t, "gl_ModelViewMatrixInverseTranspose");
      add_uniform(mat4_t, "gl_ProjectionMatrixInverseTranspose");
      add_uniform(mat4_t, "gl_ModelViewProjectionMatrixInverseTranspose");
      add_uniform(float_t, "gl_NormalScale");
      add_uniform(type("gl_LightModelParameters"), "gl_LightModel");
      add_uniform(vec4_t, "gl_FogParamsOptimizedMESA");

      const glsl_type *const mat4_array_type =
	 array(mat4_t, state->Const.MaxTextureCoords);
      add_uniform(mat4_array_type, "gl_TextureMatrix");
      add_uniform(mat4_array_type, "gl_TextureMatrixInverse");
      add_uniform(mat4_array_type, "gl_TextureMatrixTranspose");
      add_uniform(mat4_array_type, "gl_TextureMatrixInverseTranspose");

      add_uniform(array(vec4_t, state->Const.MaxClipPlanes), "gl_ClipPlane");
      add_uniform(type("gl_PointParameters"), "gl_Point");

      const glsl_type *const material_parameters_type =
	 type("gl_MaterialParameters");
      add_uniform(material_parameters_type, "gl_FrontMaterial");
      add_uniform(material_parameters_type, "gl_BackMaterial");

      add_uniform(array(type("gl_LightSourceParameters"),
                        state->Const.MaxLights),
                  "gl_LightSource");

      const glsl_type *const light_model_products_type =
         type("gl_LightModelProducts");
      add_uniform(light_model_products_type, "gl_FrontLightModelProduct");
      add_uniform(light_model_products_type, "gl_BackLightModelProduct");

      const glsl_type *const light_products_type =
         array(type("gl_LightProducts"), state->Const.MaxLights);
      add_uniform(light_products_type, "gl_FrontLightProduct");
      add_uniform(light_products_type, "gl_BackLightProduct");

      add_uniform(array(vec4_t, state->Const.MaxTextureUnits),
                  "gl_TextureEnvColor");

      const glsl_type *const texcoords_vec4 =
	 array(vec4_t, state->Const.MaxTextureCoords);
      add_uniform(texcoords_vec4, "gl_EyePlaneS");
      add_uniform(texcoords_vec4, "gl_EyePlaneT");
      add_uniform(texcoords_vec4, "gl_EyePlaneR");
      add_uniform(texcoords_vec4, "gl_EyePlaneQ");
      add_uniform(texcoords_vec4, "gl_ObjectPlaneS");
      add_uniform(texcoords_vec4, "gl_ObjectPlaneT");
      add_uniform(texcoords_vec4, "gl_ObjectPlaneR");
      add_uniform(texcoords_vec4, "gl_ObjectPlaneQ");

      add_uniform(type("gl_FogParameters"), "gl_Fog");
   }
}


/**
 * Generate special variables which exist in all shaders.
 */
void
builtin_variable_generator::generate_special_vars()
{
   if (state->ARB_shader_ballot_enable) {
      add_system_value(SYSTEM_VALUE_SUBGROUP_SIZE, uint_t, "gl_SubGroupSizeARB");
      add_system_value(SYSTEM_VALUE_SUBGROUP_INVOCATION, uint_t, "gl_SubGroupInvocationARB");
      add_system_value(SYSTEM_VALUE_SUBGROUP_EQ_MASK, uint64_t, "gl_SubGroupEqMaskARB");
      add_system_value(SYSTEM_VALUE_SUBGROUP_GE_MASK, uint64_t, "gl_SubGroupGeMaskARB");
      add_system_value(SYSTEM_VALUE_SUBGROUP_GT_MASK, uint64_t, "gl_SubGroupGtMaskARB");
      add_system_value(SYSTEM_VALUE_SUBGROUP_LE_MASK, uint64_t, "gl_SubGroupLeMaskARB");
      add_system_value(SYSTEM_VALUE_SUBGROUP_LT_MASK, uint64_t, "gl_SubGroupLtMaskARB");
   }
}


/**
 * Generate variables which only exist in vertex shaders.
 */
void
builtin_variable_generator::generate_vs_special_vars()
{
   ir_variable *var;

   if (state->is_version(130, 300) || state->EXT_gpu_shader4_enable) {
      add_system_value(SYSTEM_VALUE_VERTEX_ID, int_t, GLSL_PRECISION_HIGH,
                       "gl_VertexID");
   }
   if (state->is_version(460, 0)) {
      add_system_value(SYSTEM_VALUE_BASE_VERTEX, int_t, "gl_BaseVertex");
      add_system_value(SYSTEM_VALUE_BASE_INSTANCE, int_t, "gl_BaseInstance");
      add_system_value(SYSTEM_VALUE_DRAW_ID, int_t, "gl_DrawID");
   }
   if (state->EXT_draw_instanced_enable && state->is_version(0, 100))
      add_system_value(SYSTEM_VALUE_INSTANCE_ID, int_t, GLSL_PRECISION_HIGH,
                       "gl_InstanceIDEXT");

   if (state->ARB_draw_instanced_enable)
      add_system_value(SYSTEM_VALUE_INSTANCE_ID, int_t, "gl_InstanceIDARB");

   if (state->ARB_draw_instanced_enable || state->is_version(140, 300) ||
       state->EXT_gpu_shader4_enable) {
      add_system_value(SYSTEM_VALUE_INSTANCE_ID, int_t, GLSL_PRECISION_HIGH,
                       "gl_InstanceID");
   }
   if (state->ARB_shader_draw_parameters_enable) {
      add_system_value(SYSTEM_VALUE_BASE_VERTEX, int_t, "gl_BaseVertexARB");
      add_system_value(SYSTEM_VALUE_BASE_INSTANCE, int_t, "gl_BaseInstanceARB");
      add_system_value(SYSTEM_VALUE_DRAW_ID, int_t, "gl_DrawIDARB");
   }
   if (state->AMD_vertex_shader_layer_enable ||
       state->ARB_shader_viewport_layer_array_enable ||
       state->NV_viewport_array2_enable) {
      var = add_output(VARYING_SLOT_LAYER, int_t, "gl_Layer");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
   if (state->AMD_vertex_shader_viewport_index_enable ||
       state->ARB_shader_viewport_layer_array_enable ||
       state->NV_viewport_array2_enable) {
      var = add_output(VARYING_SLOT_VIEWPORT, int_t, "gl_ViewportIndex");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
   if (state->NV_viewport_array2_enable) {
      /* From the NV_viewport_array2 specification:
       *
       *    "The variable gl_ViewportMask[] is available as an output variable
       *    in the VTG languages. The array has ceil(v/32) elements where v is
       *    the maximum number of viewports supported by the implementation."
       *
       * Since no drivers expose more than 16 viewports, we can simply set the
       * array size to 1 rather than computing it and dealing with varying
       * slot complication.
       */
      var = add_output(VARYING_SLOT_VIEWPORT_MASK, array(int_t, 1),
                       "gl_ViewportMask");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
   if (compatibility) {
      add_input(VERT_ATTRIB_POS, vec4_t, "gl_Vertex");
      add_input(VERT_ATTRIB_NORMAL, vec3_t, "gl_Normal");
      add_input(VERT_ATTRIB_COLOR0, vec4_t, "gl_Color");
      add_input(VERT_ATTRIB_COLOR1, vec4_t, "gl_SecondaryColor");
      add_input(VERT_ATTRIB_TEX0, vec4_t, "gl_MultiTexCoord0");
      add_input(VERT_ATTRIB_TEX1, vec4_t, "gl_MultiTexCoord1");
      add_input(VERT_ATTRIB_TEX2, vec4_t, "gl_MultiTexCoord2");
      add_input(VERT_ATTRIB_TEX3, vec4_t, "gl_MultiTexCoord3");
      add_input(VERT_ATTRIB_TEX4, vec4_t, "gl_MultiTexCoord4");
      add_input(VERT_ATTRIB_TEX5, vec4_t, "gl_MultiTexCoord5");
      add_input(VERT_ATTRIB_TEX6, vec4_t, "gl_MultiTexCoord6");
      add_input(VERT_ATTRIB_TEX7, vec4_t, "gl_MultiTexCoord7");
      add_input(VERT_ATTRIB_FOG, float_t, "gl_FogCoord");
   }
}


/**
 * Generate variables which only exist in tessellation control shaders.
 */
void
builtin_variable_generator::generate_tcs_special_vars()
{
   add_system_value(SYSTEM_VALUE_PRIMITIVE_ID, int_t, GLSL_PRECISION_HIGH,
                    "gl_PrimitiveID");
   add_system_value(SYSTEM_VALUE_INVOCATION_ID, int_t, GLSL_PRECISION_HIGH,
                    "gl_InvocationID");
   add_system_value(SYSTEM_VALUE_VERTICES_IN, int_t, GLSL_PRECISION_HIGH,
                    "gl_PatchVerticesIn");

   add_output(VARYING_SLOT_TESS_LEVEL_OUTER, array(float_t, 4),
              GLSL_PRECISION_HIGH, "gl_TessLevelOuter")->data.patch = 1;
   add_output(VARYING_SLOT_TESS_LEVEL_INNER, array(float_t, 2),
              GLSL_PRECISION_HIGH, "gl_TessLevelInner")->data.patch = 1;
   /* XXX What to do if multiple are flipped on? */
   int bbox_slot = state->ctx->Const.NoPrimitiveBoundingBoxOutput ? -1 :
      VARYING_SLOT_BOUNDING_BOX0;
   if (state->EXT_primitive_bounding_box_enable)
      add_output(bbox_slot, array(vec4_t, 2), "gl_BoundingBoxEXT")
         ->data.patch = 1;
   if (state->OES_primitive_bounding_box_enable) {
      add_output(bbox_slot, array(vec4_t, 2), GLSL_PRECISION_HIGH,
                 "gl_BoundingBoxOES")->data.patch = 1;
   }
   if (state->is_version(0, 320) || state->ARB_ES3_2_compatibility_enable) {
      add_output(bbox_slot, array(vec4_t, 2), GLSL_PRECISION_HIGH,
                 "gl_BoundingBox")->data.patch = 1;
   }

   /* NOTE: These are completely pointless. Writing these will never go
    * anywhere. But the specs demands it. So we add them with a slot of -1,
    * which makes the data go nowhere.
    */
   if (state->NV_viewport_array2_enable) {
      add_output(-1, int_t, "gl_Layer");
      add_output(-1, int_t, "gl_ViewportIndex");
      add_output(-1, array(int_t, 1), "gl_ViewportMask");
   }

}


/**
 * Generate variables which only exist in tessellation evaluation shaders.
 */
void
builtin_variable_generator::generate_tes_special_vars()
{
   ir_variable *var;

   add_system_value(SYSTEM_VALUE_PRIMITIVE_ID, int_t, GLSL_PRECISION_HIGH,
                    "gl_PrimitiveID");
   add_system_value(SYSTEM_VALUE_VERTICES_IN, int_t, GLSL_PRECISION_HIGH,
                    "gl_PatchVerticesIn");
   add_system_value(SYSTEM_VALUE_TESS_COORD, vec3_t, GLSL_PRECISION_HIGH,
                    "gl_TessCoord");
   if (this->state->ctx->Const.GLSLTessLevelsAsInputs) {
      add_input(VARYING_SLOT_TESS_LEVEL_OUTER, array(float_t, 4),
                GLSL_PRECISION_HIGH, "gl_TessLevelOuter")->data.patch = 1;
      add_input(VARYING_SLOT_TESS_LEVEL_INNER, array(float_t, 2),
                GLSL_PRECISION_HIGH, "gl_TessLevelInner")->data.patch = 1;
   } else {
      add_system_value(SYSTEM_VALUE_TESS_LEVEL_OUTER, array(float_t, 4),
                       GLSL_PRECISION_HIGH, "gl_TessLevelOuter");
      add_system_value(SYSTEM_VALUE_TESS_LEVEL_INNER, array(float_t, 2),
                       GLSL_PRECISION_HIGH, "gl_TessLevelInner");
   }
   if (state->ARB_shader_viewport_layer_array_enable ||
       state->NV_viewport_array2_enable) {
      var = add_output(VARYING_SLOT_LAYER, int_t, "gl_Layer");
      var->data.interpolation = INTERP_MODE_FLAT;
      var = add_output(VARYING_SLOT_VIEWPORT, int_t, "gl_ViewportIndex");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
   if (state->NV_viewport_array2_enable) {
      var = add_output(VARYING_SLOT_VIEWPORT_MASK, array(int_t, 1),
                       "gl_ViewportMask");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
}


/**
 * Generate variables which only exist in geometry shaders.
 */
void
builtin_variable_generator::generate_gs_special_vars()
{
   ir_variable *var;

   var = add_output(VARYING_SLOT_LAYER, int_t, GLSL_PRECISION_HIGH, "gl_Layer");
   var->data.interpolation = INTERP_MODE_FLAT;
   if (state->is_version(410, 0) || state->ARB_viewport_array_enable ||
       state->OES_viewport_array_enable) {
      var = add_output(VARYING_SLOT_VIEWPORT, int_t, GLSL_PRECISION_HIGH,
                       "gl_ViewportIndex");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
   if (state->NV_viewport_array2_enable) {
      var = add_output(VARYING_SLOT_VIEWPORT_MASK, array(int_t, 1),
                       "gl_ViewportMask");
      var->data.interpolation = INTERP_MODE_FLAT;
   }
   if (state->is_version(400, 320) || state->ARB_gpu_shader5_enable ||
       state->OES_geometry_shader_enable || state->EXT_geometry_shader_enable) {
      add_system_value(SYSTEM_VALUE_INVOCATION_ID, int_t, GLSL_PRECISION_HIGH,
                       "gl_InvocationID");
   }

   /* Although gl_PrimitiveID appears in tessellation control and tessellation
    * evaluation shaders, it has a different function there than it has in
    * geometry shaders, so we treat it (and its counterpart gl_PrimitiveIDIn)
    * as special geometry shader variables.
    *
    * Note that although the general convention of suffixing geometry shader
    * input varyings with "In" was not adopted into GLSL 1.50, it is used in
    * the specific case of gl_PrimitiveIDIn.  So we don't need to treat
    * gl_PrimitiveIDIn as an {ARB,EXT}_geometry_shader4-only variable.
    */
   var = add_input(VARYING_SLOT_PRIMITIVE_ID, int_t, GLSL_PRECISION_HIGH,
                   "gl_PrimitiveIDIn");
   var->data.interpolation = INTERP_MODE_FLAT;
   var = add_output(VARYING_SLOT_PRIMITIVE_ID, int_t, GLSL_PRECISION_HIGH,
                    "gl_PrimitiveID");
   var->data.interpolation = INTERP_MODE_FLAT;
}


/**
 * Generate variables which only exist in fragment shaders.
 */
void
builtin_variable_generator::generate_fs_special_vars()
{
   ir_variable *var;

   int frag_coord_precision = (state->is_version(0, 300) ?
                               GLSL_PRECISION_HIGH :
                               GLSL_PRECISION_MEDIUM);

   if (this->state->ctx->Const.GLSLFragCoordIsSysVal) {
      add_system_value(SYSTEM_VALUE_FRAG_COORD, vec4_t, frag_coord_precision,
                       "gl_FragCoord");
   } else {
      add_input(VARYING_SLOT_POS, vec4_t, frag_coord_precision, "gl_FragCoord");
   }

   if (this->state->ctx->Const.GLSLFrontFacingIsSysVal) {
      var = add_system_value(SYSTEM_VALUE_FRONT_FACE, bool_t, "gl_FrontFacing");
      var->data.interpolation = INTERP_MODE_FLAT;
   } else {
      var = add_input(VARYING_SLOT_FACE, bool_t, "gl_FrontFacing");
      var->data.interpolation = INTERP_MODE_FLAT;
   }

   if (state->is_version(120, 100)) {
      if (this->state->ctx->Const.GLSLPointCoordIsSysVal)
         add_system_value(SYSTEM_VALUE_POINT_COORD, vec2_t,
                          GLSL_PRECISION_MEDIUM, "gl_PointCoord");
      else
         add_input(VARYING_SLOT_PNTC, vec2_t, GLSL_PRECISION_MEDIUM,
                   "gl_PointCoord");
   }

   if (state->has_geometry_shader() || state->EXT_gpu_shader4_enable) {
      var = add_input(VARYING_SLOT_PRIMITIVE_ID, int_t, GLSL_PRECISION_HIGH,
                      "gl_PrimitiveID");
      var->data.interpolation = INTERP_MODE_FLAT;
   }

   /* gl_FragColor and gl_FragData were deprecated starting in desktop GLSL
    * 1.30, and were relegated to the compatibility profile in GLSL 4.20.
    * They were removed from GLSL ES 3.00.
    */
   if (compatibility || !state->is_version(420, 300)) {
      add_output(FRAG_RESULT_COLOR, vec4_t, GLSL_PRECISION_MEDIUM,
                 "gl_FragColor");
      add_output(FRAG_RESULT_DATA0,
                 array(vec4_t, state->Const.MaxDrawBuffers),
                 GLSL_PRECISION_MEDIUM,
                 "gl_FragData");
   }

   if (state->has_framebuffer_fetch() && !state->is_version(130, 300)) {
      ir_variable *const var =
         add_output(FRAG_RESULT_DATA0,
                    array(vec4_t, state->Const.MaxDrawBuffers),
                    "gl_LastFragData");
      var->data.precision = GLSL_PRECISION_MEDIUM;
      var->data.read_only = 1;
      var->data.fb_fetch_output = 1;
      var->data.memory_coherent = 1;
   }

   if (state->es_shader && state->language_version == 100 && state->EXT_blend_func_extended_enable) {
      add_index_output(FRAG_RESULT_COLOR, 1, vec4_t,
                       GLSL_PRECISION_MEDIUM, "gl_SecondaryFragColorEXT");
      add_index_output(FRAG_RESULT_DATA0, 1,
                       array(vec4_t, state->Const.MaxDualSourceDrawBuffers),
                       GLSL_PRECISION_MEDIUM, "gl_SecondaryFragDataEXT");
   }

   /* gl_FragDepth has always been in desktop GLSL, but did not appear in GLSL
    * ES 1.00.
    */
   if (state->is_version(110, 300)) {
      add_output(FRAG_RESULT_DEPTH, float_t, GLSL_PRECISION_HIGH,
                 "gl_FragDepth");
   }

   if (state->EXT_frag_depth_enable)
      add_output(FRAG_RESULT_DEPTH, float_t, "gl_FragDepthEXT");

   if (state->ARB_shader_stencil_export_enable) {
      ir_variable *const var =
         add_output(FRAG_RESULT_STENCIL, int_t, "gl_FragStencilRefARB");
      if (state->ARB_shader_stencil_export_warn)
         var->enable_extension_warning("GL_ARB_shader_stencil_export");
   }

   if (state->AMD_shader_stencil_export_enable) {
      ir_variable *const var =
         add_output(FRAG_RESULT_STENCIL, int_t, "gl_FragStencilRefAMD");
      if (state->AMD_shader_stencil_export_warn)
         var->enable_extension_warning("GL_AMD_shader_stencil_export");
   }

   if (state->is_version(400, 320) ||
       state->ARB_sample_shading_enable ||
       state->OES_sample_variables_enable) {
      add_system_value(SYSTEM_VALUE_SAMPLE_ID, int_t, GLSL_PRECISION_LOW,
                       "gl_SampleID");
      add_system_value(SYSTEM_VALUE_SAMPLE_POS, vec2_t, GLSL_PRECISION_MEDIUM,
                       "gl_SamplePosition");
      /* From the ARB_sample_shading specification:
       *    "The number of elements in the array is ceil(<s>/32), where
       *    <s> is the maximum number of color samples supported by the
       *    implementation."
       * Since no drivers expose more than 32x MSAA, we can simply set
       * the array size to 1 rather than computing it.
       */
      add_output(FRAG_RESULT_SAMPLE_MASK, array(int_t, 1),
                 GLSL_PRECISION_HIGH, "gl_SampleMask");
   }

   if (state->is_version(400, 320) ||
       state->ARB_gpu_shader5_enable ||
       state->OES_sample_variables_enable) {
      add_system_value(SYSTEM_VALUE_SAMPLE_MASK_IN, array(int_t, 1),
                       GLSL_PRECISION_HIGH, "gl_SampleMaskIn");
   }

   if (state->is_version(430, 320) ||
       state->ARB_fragment_layer_viewport_enable ||
       state->OES_geometry_shader_enable ||
       state->EXT_geometry_shader_enable) {
      var = add_input(VARYING_SLOT_LAYER, int_t, GLSL_PRECISION_HIGH,
                      "gl_Layer");
      var->data.interpolation = INTERP_MODE_FLAT;
   }

   if (state->is_version(430, 0) ||
       state->ARB_fragment_layer_viewport_enable ||
       state->OES_viewport_array_enable) {
      var = add_input(VARYING_SLOT_VIEWPORT, int_t, "gl_ViewportIndex");
      var->data.interpolation = INTERP_MODE_FLAT;
   }

   if (state->is_version(450, 310) || state->ARB_ES3_1_compatibility_enable)
      add_system_value(SYSTEM_VALUE_HELPER_INVOCATION, bool_t, "gl_HelperInvocation");
}


/**
 * Generate variables which only exist in compute shaders.
 */
void
builtin_variable_generator::generate_cs_special_vars()
{
   add_system_value(SYSTEM_VALUE_LOCAL_INVOCATION_ID, uvec3_t,
                    "gl_LocalInvocationID");
   add_system_value(SYSTEM_VALUE_WORK_GROUP_ID, uvec3_t, "gl_WorkGroupID");
   add_system_value(SYSTEM_VALUE_NUM_WORK_GROUPS, uvec3_t, "gl_NumWorkGroups");

   if (state->ARB_compute_variable_group_size_enable) {
      add_system_value(SYSTEM_VALUE_LOCAL_GROUP_SIZE,
                       uvec3_t, "gl_LocalGroupSizeARB");
   }

   add_system_value(SYSTEM_VALUE_GLOBAL_INVOCATION_ID,
                    uvec3_t, "gl_GlobalInvocationID");
   add_system_value(SYSTEM_VALUE_LOCAL_INVOCATION_INDEX,
                    uint_t, "gl_LocalInvocationIndex");
}


/**
 * Add a single "varying" variable.  The variable's type and direction (input
 * or output) are adjusted as appropriate for the type of shader being
 * compiled.
 */
void
builtin_variable_generator::add_varying(int slot, const glsl_type *type,
                                        int precision, const char *name)
{
   switch (state->stage) {
   case MESA_SHADER_TESS_CTRL:
   case MESA_SHADER_TESS_EVAL:
   case MESA_SHADER_GEOMETRY:
      this->per_vertex_in.add_field(slot, type, precision, name);
      /* FALLTHROUGH */
   case MESA_SHADER_VERTEX:
      this->per_vertex_out.add_field(slot, type, precision, name);
      break;
   case MESA_SHADER_FRAGMENT:
      add_input(slot, type, precision, name);
      break;
   case MESA_SHADER_COMPUTE:
      /* Compute shaders don't have varyings. */
      break;
   default:
      break;
   }
}


/**
 * Generate variables that are used to communicate data from one shader stage
 * to the next ("varyings").
 */
void
builtin_variable_generator::generate_varyings()
{
   struct gl_shader_compiler_options *options =
      &state->ctx->Const.ShaderCompilerOptions[state->stage];

   /* gl_Position and gl_PointSize are not visible from fragment shaders. */
   if (state->stage != MESA_SHADER_FRAGMENT) {
      add_varying(VARYING_SLOT_POS, vec4_t, GLSL_PRECISION_HIGH, "gl_Position");
      if (!state->es_shader ||
          state->stage == MESA_SHADER_VERTEX ||
          (state->stage == MESA_SHADER_GEOMETRY &&
           (state->OES_geometry_point_size_enable ||
            state->EXT_geometry_point_size_enable)) ||
          ((state->stage == MESA_SHADER_TESS_CTRL ||
            state->stage == MESA_SHADER_TESS_EVAL) &&
           (state->OES_tessellation_point_size_enable ||
            state->EXT_tessellation_point_size_enable))) {
         add_varying(VARYING_SLOT_PSIZ,
                     float_t,
                     state->is_version(0, 300) ?
                     GLSL_PRECISION_HIGH :
                     GLSL_PRECISION_MEDIUM,
                     "gl_PointSize");
      }
   }

   if (state->has_clip_distance()) {
       add_varying(VARYING_SLOT_CLIP_DIST0, array(float_t, 0),
                   GLSL_PRECISION_HIGH, "gl_ClipDistance");
   }
   if (state->has_cull_distance()) {
      add_varying(VARYING_SLOT_CULL_DIST0, array(float_t, 0),
                  GLSL_PRECISION_HIGH, "gl_CullDistance");
   }

   if (compatibility) {
      add_varying(VARYING_SLOT_TEX0, array(vec4_t, 0), "gl_TexCoord");
      add_varying(VARYING_SLOT_FOGC, float_t, "gl_FogFragCoord");
      if (state->stage == MESA_SHADER_FRAGMENT) {
         add_varying(VARYING_SLOT_COL0, vec4_t, "gl_Color");
         add_varying(VARYING_SLOT_COL1, vec4_t, "gl_SecondaryColor");
      } else {
         add_varying(VARYING_SLOT_CLIP_VERTEX, vec4_t, "gl_ClipVertex");
         add_varying(VARYING_SLOT_COL0, vec4_t, "gl_FrontColor");
         add_varying(VARYING_SLOT_BFC0, vec4_t, "gl_BackColor");
         add_varying(VARYING_SLOT_COL1, vec4_t, "gl_FrontSecondaryColor");
         add_varying(VARYING_SLOT_BFC1, vec4_t, "gl_BackSecondaryColor");
      }
   }

   /* Section 7.1 (Built-In Language Variables) of the GLSL 4.00 spec
    * says:
    *
    *    "In the tessellation control language, built-in variables are
    *    intrinsically declared as:
    *
    *        in gl_PerVertex {
    *            vec4 gl_Position;
    *            float gl_PointSize;
    *            float gl_ClipDistance[];
    *        } gl_in[gl_MaxPatchVertices];"
    */
   if (state->stage == MESA_SHADER_TESS_CTRL ||
       state->stage == MESA_SHADER_TESS_EVAL) {
      const glsl_type *per_vertex_in_type =
         this->per_vertex_in.construct_interface_instance();
      add_variable("gl_in", array(per_vertex_in_type, state->Const.MaxPatchVertices),
                   GLSL_PRECISION_NONE, ir_var_shader_in, -1);
   }
   if (state->stage == MESA_SHADER_GEOMETRY) {
      const glsl_type *per_vertex_in_type =
         this->per_vertex_in.construct_interface_instance();
      add_variable("gl_in", array(per_vertex_in_type, 0),
                   GLSL_PRECISION_NONE, ir_var_shader_in, -1);
   }
   if (state->stage == MESA_SHADER_TESS_CTRL) {
      const glsl_type *per_vertex_out_type =
         this->per_vertex_out.construct_interface_instance();
      add_variable("gl_out", array(per_vertex_out_type, 0),
                   GLSL_PRECISION_NONE, ir_var_shader_out, -1);
   }
   if (state->stage == MESA_SHADER_VERTEX ||
       state->stage == MESA_SHADER_TESS_EVAL ||
       state->stage == MESA_SHADER_GEOMETRY) {
      const glsl_type *per_vertex_out_type =
         this->per_vertex_out.construct_interface_instance();
      const glsl_struct_field *fields = per_vertex_out_type->fields.structure;
      for (unsigned i = 0; i < per_vertex_out_type->length; i++) {
         ir_variable *var =
            add_variable(fields[i].name, fields[i].type, fields[i].precision,
                         ir_var_shader_out, fields[i].location);
         var->data.interpolation = fields[i].interpolation;
         var->data.centroid = fields[i].centroid;
         var->data.sample = fields[i].sample;
         var->data.patch = fields[i].patch;
         var->init_interface_type(per_vertex_out_type);

         var->data.invariant = fields[i].location == VARYING_SLOT_POS &&
                               options->PositionAlwaysInvariant;
      }
   }
}


}; /* Anonymous namespace */


void
_mesa_glsl_initialize_variables(exec_list *instructions,
				struct _mesa_glsl_parse_state *state)
{
   builtin_variable_generator gen(instructions, state);

   gen.generate_constants();
   gen.generate_uniforms();
   gen.generate_special_vars();

   gen.generate_varyings();

   switch (state->stage) {
   case MESA_SHADER_VERTEX:
      gen.generate_vs_special_vars();
      break;
   case MESA_SHADER_TESS_CTRL:
      gen.generate_tcs_special_vars();
      break;
   case MESA_SHADER_TESS_EVAL:
      gen.generate_tes_special_vars();
      break;
   case MESA_SHADER_GEOMETRY:
      gen.generate_gs_special_vars();
      break;
   case MESA_SHADER_FRAGMENT:
      gen.generate_fs_special_vars();
      break;
   case MESA_SHADER_COMPUTE:
      gen.generate_cs_special_vars();
      break;
   default:
      break;
   }
}
