cmake_minimum_required(VERSION 3.0)

project(glsl_optimizer VERSION 0.1
                       DESCRIPTION "GLSL Optimizer"
                       LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 11)

include_directories(include)
include_directories(src/mesa)
include_directories(src/mapi)
include_directories(src/compiler)
include_directories(src/compiler/glsl)
include_directories(src/gallium/auxiliary)
include_directories(src/gallium/include)
include_directories(src)
include_directories(src/util)

add_definitions(-D__STDC_FORMAT_MACROS)
if(${CMAKE_SYSTEM_NAME} MATCHES "Linux")
  add_definitions(-D_GNU_SOURCE)
  add_definitions(-DHAVE_ENDIAN_H)
endif()
if(${CMAKE_SYSTEM_NAME} MATCHES "Windows")
  add_definitions(-D_USE_MATH_DEFINES)
else()
  add_definitions(-DHAVE_PTHREAD)
  add_definitions(-DHAVE_TIMESPEC_GET)
endif()

add_library(glcpp STATIC "src/compiler/glsl/glcpp/glcpp-lex.c"
                         "src/compiler/glsl/glcpp/glcpp-parse.c"
                         "src/compiler/glsl/glcpp/glcpp.h"
                         "src/compiler/glsl/glcpp/pp_standalone_scaffolding.c"
                         "src/compiler/glsl/glcpp/pp.c"
                         "src/util/blob.c"
                         "src/util/half_float.c"
                         "src/util/hash_table.c"
                         "src/util/mesa-sha1.c"
                         "src/util/os_misc.c"
                         "src/util/ralloc.c"
                         "src/util/set.c"
                         "src/util/sha1/sha1.c"
                         "src/util/softfloat.c"
                         "src/util/string_buffer.c"
                         "src/util/strtod.c"
                         "src/util/u_debug.c")

add_library(mesa STATIC "src/mesa/program/dummy_errors.c"
                        "src/mesa/program/symbol_table.c"
                        "src/mesa/main/extensions_table.c")

add_library(glsl_optimizer STATIC "src/compiler/glsl_types.cpp"
                                  "src/compiler/glsl/ast_array_index.cpp"
                                  "src/compiler/glsl/ast_expr.cpp"
                                  "src/compiler/glsl/ast_function.cpp"
                                  "src/compiler/glsl/ast_to_hir.cpp"
                                  "src/compiler/glsl/ast_type.cpp"
                                  "src/compiler/glsl/builtin_functions.cpp"
                                  "src/compiler/glsl/builtin_int64.h"
                                  "src/compiler/glsl/builtin_types.cpp"
                                  "src/compiler/glsl/builtin_variables.cpp"
                                  "src/compiler/glsl/generate_ir.cpp"
                                  "src/compiler/glsl/glsl_lexer.cpp"
                                  "src/compiler/glsl/glsl_optimizer.cpp"
                                  "src/compiler/glsl/glsl_parser_extras.cpp"
                                  "src/compiler/glsl/glsl_parser.cpp"
                                  "src/compiler/glsl/glsl_symbol_table.cpp"
                                  "src/compiler/glsl/hir_field_selection.cpp"
                                  "src/compiler/glsl/ir_array_refcount.cpp"
                                  "src/compiler/glsl/ir_basic_block.cpp"
                                  "src/compiler/glsl/ir_builder.cpp"
                                  "src/compiler/glsl/ir_clone.cpp"
                                  "src/compiler/glsl/ir_constant_expression.cpp"
                                  "src/compiler/glsl/ir_equals.cpp"
                                  "src/compiler/glsl/ir_expression_flattening.cpp"
                                  "src/compiler/glsl/ir_function_can_inline.cpp"
                                  "src/compiler/glsl/ir_function_detect_recursion.cpp"
                                  "src/compiler/glsl/ir_function.cpp"
                                  "src/compiler/glsl/ir_hierarchical_visitor.cpp"
                                  "src/compiler/glsl/ir_hv_accept.cpp"
                                  "src/compiler/glsl/ir_print_glsl_visitor.cpp"
                                  "src/compiler/glsl/ir_print_visitor.cpp"
                                  "src/compiler/glsl/ir_reader.cpp"
                                  "src/compiler/glsl/ir_rvalue_visitor.cpp"
                                  "src/compiler/glsl/ir_set_program_inouts.cpp"
                                  "src/compiler/glsl/ir_unused_structs.cpp"
                                  "src/compiler/glsl/ir_validate.cpp"
                                  "src/compiler/glsl/ir_variable_refcount.cpp"
                                  "src/compiler/glsl/ir_visitor.h"
                                  "src/compiler/glsl/ir.cpp"
                                  "src/compiler/glsl/link_atomics.cpp"
                                  "src/compiler/glsl/link_functions.cpp"
                                  "src/compiler/glsl/link_interface_blocks.cpp"
                                  "src/compiler/glsl/link_uniform_block_active_visitor.cpp"
                                  "src/compiler/glsl/link_uniform_blocks.cpp"
                                  "src/compiler/glsl/link_uniform_initializers.cpp"
                                  "src/compiler/glsl/link_uniforms.cpp"
                                  "src/compiler/glsl/link_varyings.cpp"
                                  "src/compiler/glsl/linker_util.cpp"
                                  "src/compiler/glsl/linker.cpp"
                                  "src/compiler/glsl/list.h"
                                  "src/compiler/glsl/loop_analysis.cpp"
                                  "src/compiler/glsl/loop_unroll.cpp"
                                  "src/compiler/glsl/lower_blend_equation_advanced.cpp"
                                  "src/compiler/glsl/lower_buffer_access.cpp"
                                  "src/compiler/glsl/lower_builtins.cpp"
                                  "src/compiler/glsl/lower_const_arrays_to_uniforms.cpp"
                                  "src/compiler/glsl/lower_cs_derived.cpp"
                                  "src/compiler/glsl/lower_discard_flow.cpp"
                                  "src/compiler/glsl/lower_discard.cpp"
                                  "src/compiler/glsl/lower_distance.cpp"
                                  "src/compiler/glsl/lower_if_to_cond_assign.cpp"
                                  "src/compiler/glsl/lower_instructions.cpp"
                                  "src/compiler/glsl/lower_int64.cpp"
                                  "src/compiler/glsl/lower_jumps.cpp"
                                  "src/compiler/glsl/lower_mat_op_to_vec.cpp"
                                  "src/compiler/glsl/lower_named_interface_blocks.cpp"
                                  "src/compiler/glsl/lower_offset_array.cpp"
                                  "src/compiler/glsl/lower_output_reads.cpp"
                                  "src/compiler/glsl/lower_packed_varyings.cpp"
                                  "src/compiler/glsl/lower_packing_builtins.cpp"
                                  "src/compiler/glsl/lower_precision.cpp"
                                  "src/compiler/glsl/lower_shared_reference.cpp"
                                  "src/compiler/glsl/lower_subroutine.cpp"
                                  "src/compiler/glsl/lower_tess_level.cpp"
                                  "src/compiler/glsl/lower_texture_projection.cpp"
                                  "src/compiler/glsl/lower_ubo_reference.cpp"
                                  "src/compiler/glsl/lower_variable_index_to_cond_assign.cpp"
                                  "src/compiler/glsl/lower_vec_index_to_cond_assign.cpp"
                                  "src/compiler/glsl/lower_vec_index_to_swizzle.cpp"
                                  "src/compiler/glsl/lower_vector_derefs.cpp"
                                  "src/compiler/glsl/lower_vector_insert.cpp"
                                  "src/compiler/glsl/lower_vector.cpp"
                                  "src/compiler/glsl/lower_vertex_id.cpp"
                                  "src/compiler/glsl/lower_xfb_varying.cpp"
                                  "src/compiler/glsl/opt_algebraic.cpp"
                                  "src/compiler/glsl/opt_array_splitting.cpp"
                                  "src/compiler/glsl/opt_conditional_discard.cpp"
                                  "src/compiler/glsl/opt_constant_folding.cpp"
                                  "src/compiler/glsl/opt_constant_propagation.cpp"
                                  "src/compiler/glsl/opt_constant_variable.cpp"
                                  "src/compiler/glsl/opt_copy_propagation_elements.cpp"
                                  "src/compiler/glsl/opt_dead_builtin_variables.cpp"
                                  "src/compiler/glsl/opt_dead_builtin_varyings.cpp"
                                  "src/compiler/glsl/opt_dead_code_local.cpp"
                                  "src/compiler/glsl/opt_dead_code.cpp"
                                  "src/compiler/glsl/opt_dead_functions.cpp"
                                  "src/compiler/glsl/opt_flatten_nested_if_blocks.cpp"
                                  "src/compiler/glsl/opt_flip_matrices.cpp"
                                  "src/compiler/glsl/opt_function_inlining.cpp"
                                  "src/compiler/glsl/opt_if_simplification.cpp"
                                  "src/compiler/glsl/opt_minmax.cpp"
                                  "src/compiler/glsl/opt_rebalance_tree.cpp"
                                  "src/compiler/glsl/opt_redundant_jumps.cpp"
                                  "src/compiler/glsl/opt_structure_splitting.cpp"
                                  "src/compiler/glsl/opt_swizzle.cpp"
                                  "src/compiler/glsl/opt_tree_grafting.cpp"
                                  "src/compiler/glsl/opt_vectorize.cpp"
                                  "src/compiler/glsl/propagate_invariance.cpp"
                                  "src/compiler/glsl/s_expression.cpp"
                                  "src/compiler/glsl/serialize.cpp"
                                  "src/compiler/glsl/shader_cache.cpp"
                                  "src/compiler/glsl/standalone_scaffolding.cpp"
                                  "src/compiler/glsl/string_to_uint_map.cpp"
                                  "src/compiler/shader_enums.c")

target_link_libraries(glsl_optimizer glcpp mesa)

add_executable(glslopt "contrib/glslopt/Main.cpp")
target_link_libraries(glslopt glsl_optimizer)
