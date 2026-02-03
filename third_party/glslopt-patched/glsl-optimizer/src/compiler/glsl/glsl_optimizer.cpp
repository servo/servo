#include "glsl_optimizer.h"
#include "ast.h"
#include "glsl_parser_extras.h"
#include "glsl_parser.h"
#include "ir_optimization.h"
// FIXME: metal
// #include "ir_print_metal_visitor.h"
#include "ir_print_glsl_visitor.h"
#include "ir_print_visitor.h"
// FIXME: stats
// #include "ir_stats.h"
#include "loop_analysis.h"
#include "program.h"
#include "linker.h"
#include "main/mtypes.h"
#include "standalone_scaffolding.h"
#include "builtin_functions.h"
#include "program/program.h"

static void
init_gl_program(struct gl_program *prog, bool is_arb_asm, gl_shader_stage stage)
{
   prog->RefCount = 1;
   prog->Format = GL_PROGRAM_FORMAT_ASCII_ARB;
   prog->is_arb_asm = is_arb_asm;
   prog->info.stage = stage;
}

static struct gl_program *
new_program(UNUSED struct gl_context *ctx, gl_shader_stage stage,
            UNUSED GLuint id, bool is_arb_asm)
{
   struct gl_program *prog = rzalloc(NULL, struct gl_program);
   init_gl_program(prog, is_arb_asm, stage);
   return prog;
}

static void
initialize_mesa_context(struct gl_context *ctx, glslopt_target api)
{
	gl_api mesaAPI;
	switch(api)
	{
		default:
		case kGlslTargetOpenGL:
			mesaAPI = API_OPENGL_COMPAT;
			break;
		case kGlslTargetOpenGLES20:
			mesaAPI = API_OPENGLES2;
			break;
		case kGlslTargetOpenGLES30:
			mesaAPI = API_OPENGL_CORE;
			break;
		case kGlslTargetMetal:
			mesaAPI = API_OPENGL_CORE;
			break;
	}
	initialize_context_to_defaults (ctx, mesaAPI);
	_mesa_glsl_builtin_functions_init_or_ref();

	switch(api)
	{
	default:
	case kGlslTargetOpenGL:
		ctx->Const.GLSLVersion = 150;
		break;
	case kGlslTargetOpenGLES20:
		ctx->Extensions.OES_standard_derivatives = true;
		// FIXME: extensions
		// ctx->Extensions.EXT_shadow_samplers = true;
		// ctx->Extensions.EXT_frag_depth = true;
		ctx->Extensions.EXT_shader_framebuffer_fetch = true;
		break;
	case kGlslTargetOpenGLES30:
		ctx->Extensions.ARB_ES3_1_compatibility = true;
		ctx->Extensions.EXT_shader_framebuffer_fetch = true;
		break;
	case kGlslTargetMetal:
		ctx->Extensions.ARB_ES3_compatibility = true;
		ctx->Extensions.EXT_shader_framebuffer_fetch = true;
		break;
	}


   // allow high amount of texcoords
   ctx->Const.MaxTextureCoordUnits = 16;

   ctx->Const.Program[MESA_SHADER_VERTEX].MaxTextureImageUnits = 16;
   ctx->Const.Program[MESA_SHADER_FRAGMENT].MaxTextureImageUnits = 16;
   ctx->Const.Program[MESA_SHADER_GEOMETRY].MaxTextureImageUnits = 16;

   // For GLES2.0 this would be 1, but we do support GL_EXT_draw_buffers
   ctx->Const.MaxDrawBuffers = 4;

   ctx->Driver.NewProgram = new_program;
}


struct glslopt_ctx {
	glslopt_ctx (glslopt_target target) {
		this->target = target;
		mem_ctx = ralloc_context (NULL);
		initialize_mesa_context (&mesa_ctx, target);
	}
	~glslopt_ctx() {
		ralloc_free (mem_ctx);
	}
	struct gl_context mesa_ctx;
	void* mem_ctx;
	glslopt_target target;
};

glslopt_ctx* glslopt_initialize (glslopt_target target)
{
	return new glslopt_ctx(target);
}

void glslopt_cleanup (glslopt_ctx* ctx)
{
	delete ctx;
}

void glslopt_set_max_unroll_iterations (glslopt_ctx* ctx, unsigned iterations)
{
	for (int i = 0; i < MESA_SHADER_STAGES; ++i)
		ctx->mesa_ctx.Const.ShaderCompilerOptions[i].MaxUnrollIterations = iterations;
}

struct glslopt_shader_var
{
	const char* name;
	glslopt_basic_type type;
	glslopt_precision prec;
	int vectorSize;
	int matrixSize;
	int arraySize;
	int location;
};

struct glslopt_shader
{
	static void* operator new(size_t size, void *ctx)
	{
		void *node;
		node = ralloc_size(ctx, size);
		assert(node != NULL);
		return node;
	}
	static void operator delete(void *node)
	{
		ralloc_free(node);
	}

	glslopt_shader ()
		: rawOutput(0)
		, optimizedOutput(0)
		, status(false)
		, uniformCount(0)
		, uniformsSize(0)
		, inputCount(0)
		, textureCount(0)
		, statsMath(0)
		, statsTex(0)
		, statsFlow(0)
	{
		infoLog = "Shader not compiled yet";
		
		whole_program = rzalloc (NULL, struct gl_shader_program);
		assert(whole_program != NULL);
		whole_program->data = rzalloc(whole_program, struct gl_shader_program_data);
		assert(whole_program->data != NULL);
		whole_program->data->InfoLog = ralloc_strdup(whole_program->data, "");

		whole_program->Shaders = reralloc(whole_program, whole_program->Shaders, struct gl_shader *, whole_program->NumShaders + 1);
		assert(whole_program->Shaders != NULL);
		
		shader = rzalloc(whole_program, gl_shader);
		whole_program->Shaders[whole_program->NumShaders] = shader;
		whole_program->NumShaders++;

		whole_program->data->LinkStatus = LINKING_SUCCESS;
	}
	
	~glslopt_shader()
	{
		for (unsigned i = 0; i < MESA_SHADER_STAGES; i++)
			ralloc_free(whole_program->_LinkedShaders[i]);
		ralloc_free(whole_program);
		ralloc_free(rawOutput);
		ralloc_free(optimizedOutput);
	}
	
	struct gl_shader_program* whole_program;
	struct gl_shader* shader;

	static const int kMaxShaderUniforms = 1024;
	static const int kMaxShaderInputs = 128;
	static const int kMaxShaderTextures = 128;
	glslopt_shader_var uniforms[kMaxShaderUniforms];
	glslopt_shader_var inputs[kMaxShaderInputs];
	glslopt_shader_var textures[kMaxShaderInputs];
	int uniformCount, uniformsSize;
	int inputCount;
	int textureCount;
	int statsMath, statsTex, statsFlow;

	char*	rawOutput;
	char*	optimizedOutput;
	const char*	infoLog;
	bool	status;
};

static inline void debug_print_ir (const char* name, exec_list* ir, _mesa_glsl_parse_state* state, void* memctx)
{
	#if 0
	printf("**** %s:\n", name);
//	_mesa_print_ir (ir, state);
	char* foobar = _mesa_print_ir_glsl(ir, state, ralloc_strdup(memctx, ""), kPrintGlslFragment);
	printf("%s\n", foobar);
	validate_ir_tree(ir);
	#endif
}


// FIXME: precision
// struct precision_ctx
// {
// 	exec_list* root_ir;
// 	bool res;
// };


// static void propagate_precision_deref(ir_instruction *ir, void *data)
// {
// 	// variable deref with undefined precision: take from variable itself
// 	ir_dereference_variable* der = ir->as_dereference_variable();
// 	if (der && der->get_precision() == glsl_precision_undefined && der->var->data.precision != glsl_precision_undefined)
// 	{
// 		der->set_precision ((glsl_precision)der->var->data.precision);
// 		((precision_ctx*)data)->res = true;
// 	}

// 	// array deref with undefined precision: take from array itself
// 	ir_dereference_array* der_arr = ir->as_dereference_array();
// 	if (der_arr && der_arr->get_precision() == glsl_precision_undefined && der_arr->array->get_precision() != glsl_precision_undefined)
// 	{
// 		der_arr->set_precision (der_arr->array->get_precision());
// 		((precision_ctx*)data)->res = true;
// 	}

// 	// swizzle with undefined precision: take from swizzle argument
// 	ir_swizzle* swz = ir->as_swizzle();
// 	if (swz && swz->get_precision() == glsl_precision_undefined && swz->val->get_precision() != glsl_precision_undefined)
// 	{
// 		swz->set_precision (swz->val->get_precision());
// 		((precision_ctx*)data)->res = true;
// 	}

// }

// static void propagate_precision_expr(ir_instruction *ir, void *data)
// {
// 	ir_expression* expr = ir->as_expression();
// 	if (!expr)
// 		return;
// 	if (expr->get_precision() != glsl_precision_undefined)
// 		return;

// 	glsl_precision prec_params_max = glsl_precision_undefined;
// 	for (int i = 0; i < (int)expr->get_num_operands(); ++i)
// 	{
// 		ir_rvalue* op = expr->operands[i];
// 		if (op && op->get_precision() != glsl_precision_undefined)
// 			prec_params_max = higher_precision (prec_params_max, op->get_precision());
// 	}
// 	if (expr->get_precision() != prec_params_max)
// 	{
// 		expr->set_precision (prec_params_max);
// 		((precision_ctx*)data)->res = true;
// 	}
	
// }

// static void propagate_precision_texture(ir_instruction *ir, void *data)
// {
// 	ir_texture* tex = ir->as_texture();
// 	if (!tex)
// 		return;

// 	glsl_precision sampler_prec = tex->sampler->get_precision();
// 	if (tex->get_precision() == sampler_prec || sampler_prec == glsl_precision_undefined)
// 		return;

// 	// set precision of ir_texture node to that of the sampler itself
// 	tex->set_precision(sampler_prec);
// 	((precision_ctx*)data)->res = true;
// }

// struct undefined_ass_ctx
// {
// 	ir_variable* var;
// 	bool res;
// };

// static void has_only_undefined_precision_assignments(ir_instruction *ir, void *data)
// {
// 	ir_assignment* ass = ir->as_assignment();
// 	if (!ass)
// 		return;
// 	undefined_ass_ctx* ctx = (undefined_ass_ctx*)data;
// 	if (ass->whole_variable_written() != ctx->var)
// 		return;
// 	glsl_precision prec = ass->rhs->get_precision();
// 	if (prec == glsl_precision_undefined)
// 		return;
// 	ctx->res = false;
// }


// static void propagate_precision_assign(ir_instruction *ir, void *data)
// {
// 	ir_assignment* ass = ir->as_assignment();
// 	if (!ass || !ass->lhs || !ass->rhs)
// 		return;

// 	glsl_precision lp = ass->lhs->get_precision();
// 	glsl_precision rp = ass->rhs->get_precision();

// 	// for assignments with LHS having undefined precision, take it from RHS
// 	if (rp != glsl_precision_undefined)
// 	{
// 		ir_variable* lhs_var = ass->lhs->variable_referenced();
// 		if (lp == glsl_precision_undefined)
// 		{		
// 			if (lhs_var)
// 				lhs_var->data.precision = rp;
// 			ass->lhs->set_precision (rp);
// 			((precision_ctx*)data)->res = true;
// 		}
// 		return;
// 	}
	
// 	// for assignments where LHS has precision, but RHS is a temporary variable
// 	// with undefined precision that's only assigned from other undefined precision
// 	// sources -> make the RHS variable take LHS precision
// 	if (lp != glsl_precision_undefined && rp == glsl_precision_undefined)
// 	{
// 		ir_dereference* deref = ass->rhs->as_dereference();
// 		if (deref)
// 		{
// 			ir_variable* rhs_var = deref->variable_referenced();
// 			if (rhs_var && rhs_var->data.mode == ir_var_temporary && rhs_var->data.precision == glsl_precision_undefined)
// 			{
// 				undefined_ass_ctx ctx;
// 				ctx.var = rhs_var;
// 				// find if we only assign to it from undefined precision sources
// 				ctx.res = true;
// 				exec_list* root_ir = ((precision_ctx*)data)->root_ir;
// 				foreach_in_list(ir_instruction, inst, root_ir)
// 				{
// 					visit_tree (ir, has_only_undefined_precision_assignments, &ctx);
// 				}
// 				if (ctx.res)
// 				{
// 					rhs_var->data.precision = lp;
// 					ass->rhs->set_precision(lp);
// 					((precision_ctx*)data)->res = true;
// 				}
// 			}
// 		}
// 		return;
// 	}
// }


// static void propagate_precision_call(ir_instruction *ir, void *data)
// {
// 	ir_call* call = ir->as_call();
// 	if (!call)
// 		return;
// 	if (!call->return_deref)
// 		return;
// 	if (call->return_deref->get_precision() == glsl_precision_undefined /*&& call->callee->precision == glsl_precision_undefined*/)
// 	{
// 		glsl_precision prec_params_max = glsl_precision_undefined;
// 		foreach_two_lists(formal_node, &call->callee->parameters,
// 						  actual_node, &call->actual_parameters) {
// 			ir_variable* sig_param = (ir_variable*)formal_node;
// 			ir_rvalue* param = (ir_rvalue*)actual_node;
			
// 			glsl_precision p = (glsl_precision)sig_param->data.precision;
// 			if (p == glsl_precision_undefined)
// 				p = param->get_precision();
			
// 			prec_params_max = higher_precision (prec_params_max, p);
// 		}
// 		if (call->return_deref->get_precision() != prec_params_max)
// 		{
// 			call->return_deref->set_precision (prec_params_max);
// 			((precision_ctx*)data)->res = true;
// 		}
// 	}
// }

// static bool propagate_precision(exec_list* list, bool assign_high_to_undefined)
// {
// 	bool anyProgress = false;
// 	precision_ctx ctx;

// 	do {
// 		ctx.res = false;
// 		ctx.root_ir = list;
// 		foreach_in_list(ir_instruction, ir, list)
// 		{
// 			visit_tree (ir, propagate_precision_texture, &ctx);
// 			visit_tree (ir, propagate_precision_deref, &ctx);
// 			bool hadProgress = ctx.res;
// 			ctx.res = false;
// 			visit_tree (ir, propagate_precision_assign, &ctx);
// 			if (ctx.res)
// 			{
// 				// assignment precision propagation might have added precision
// 				// to some variables; need to propagate dereference precision right
// 				// after that too.
// 				visit_tree (ir, propagate_precision_deref, &ctx);
// 			}
// 			ctx.res |= hadProgress;
// 			visit_tree (ir, propagate_precision_call, &ctx);
// 			visit_tree (ir, propagate_precision_expr, &ctx);
// 		}
// 		anyProgress |= ctx.res;
// 	} while (ctx.res);
// 	anyProgress |= ctx.res;

// 	// for globals that have undefined precision, set it to highp
// 	if (assign_high_to_undefined)
// 	{
// 		foreach_in_list(ir_instruction, ir, list)
// 		{
// 			ir_variable* var = ir->as_variable();
// 			if (var)
// 			{
// 				if (var->data.precision == glsl_precision_undefined)
// 				{
// 					var->data.precision = glsl_precision_high;
// 					anyProgress = true;
// 				}
// 			}
// 		}
// 	}

// 	return anyProgress;
// }


static void do_optimization_passes(exec_list* ir, bool linked, _mesa_glsl_parse_state* state, void* mem_ctx)
{
	bool progress;
	// FIXME: Shouldn't need to bound the number of passes
	int passes = 0,
		kMaximumPasses = 1000;
	do {
		progress = false;
		++passes;
		bool progress2;
		debug_print_ir ("Initial", ir, state, mem_ctx);
		if (linked) {
			progress2 = do_function_inlining(ir); progress |= progress2; if (progress2) debug_print_ir ("After inlining", ir, state, mem_ctx);
			progress2 = do_dead_functions(ir); progress |= progress2; if (progress2) debug_print_ir ("After dead functions", ir, state, mem_ctx);
			progress2 = do_structure_splitting(ir); progress |= progress2; if (progress2) debug_print_ir ("After struct splitting", ir, state, mem_ctx);
		}
		progress2 = do_if_simplification(ir); progress |= progress2; if (progress2) debug_print_ir ("After if simpl", ir, state, mem_ctx);
		progress2 = opt_flatten_nested_if_blocks(ir); progress |= progress2; if (progress2) debug_print_ir ("After if flatten", ir, state, mem_ctx);
		// progress2 = propagate_precision (ir, state->metal_target); progress |= progress2; if (progress2) debug_print_ir ("After prec propagation", ir, state, mem_ctx);
		progress2 = do_copy_propagation_elements(ir); progress |= progress2; if (progress2) debug_print_ir ("After copy propagation elems", ir, state, mem_ctx);

		if (linked)
		{
			progress2 = do_vectorize(ir); progress |= progress2; if (progress2) debug_print_ir ("After vectorize", ir, state, mem_ctx);
		}
		if (linked) {
			progress2 = do_dead_code(ir,false); progress |= progress2; if (progress2) debug_print_ir ("After dead code", ir, state, mem_ctx);
		} else {
			progress2 = do_dead_code_unlinked(ir); progress |= progress2; if (progress2) debug_print_ir ("After dead code unlinked", ir, state, mem_ctx);
		}
		progress2 = do_dead_code_local(ir); progress |= progress2; if (progress2) debug_print_ir ("After dead code local", ir, state, mem_ctx);
		// progress2 = propagate_precision (ir, state->metal_target); progress |= progress2; if (progress2) debug_print_ir ("After prec propagation", ir, state, mem_ctx);
		progress2 = do_tree_grafting(ir); progress |= progress2; if (progress2) debug_print_ir ("After tree grafting", ir, state, mem_ctx);
		progress2 = do_constant_propagation(ir); progress |= progress2; if (progress2) debug_print_ir ("After const propagation", ir, state, mem_ctx);
		if (linked) {
			progress2 = do_constant_variable(ir); progress |= progress2; if (progress2) debug_print_ir ("After const variable", ir, state, mem_ctx);
		} else {
			progress2 = do_constant_variable_unlinked(ir); progress |= progress2; if (progress2) debug_print_ir ("After const variable unlinked", ir, state, mem_ctx);
		}
		progress2 = do_constant_folding(ir); progress |= progress2; if (progress2) debug_print_ir ("After const folding", ir, state, mem_ctx);
		progress2 = do_minmax_prune(ir); progress |= progress2; if (progress2) debug_print_ir ("After minmax prune", ir, state, mem_ctx);
		progress2 = do_rebalance_tree(ir); progress |= progress2; if (progress2) debug_print_ir ("After rebalance tree", ir, state, mem_ctx);
		progress2 = do_algebraic(ir, state->ctx->Const.NativeIntegers, &state->ctx->Const.ShaderCompilerOptions[state->stage]); progress |= progress2; if (progress2) debug_print_ir ("After algebraic", ir, state, mem_ctx);
		progress2 = do_lower_jumps(ir); progress |= progress2; if (progress2) debug_print_ir ("After lower jumps", ir, state, mem_ctx);
		progress2 = do_vec_index_to_swizzle(ir); progress |= progress2; if (progress2) debug_print_ir ("After vec index to swizzle", ir, state, mem_ctx);
		progress2 = lower_vector_insert(ir, false); progress |= progress2; if (progress2) debug_print_ir ("After lower vector insert", ir, state, mem_ctx);
		progress2 = optimize_swizzles(ir); progress |= progress2; if (progress2) debug_print_ir ("After optimize swizzles", ir, state, mem_ctx);
		progress2 = optimize_split_arrays(ir, linked); progress |= progress2; if (progress2) debug_print_ir ("After split arrays", ir, state, mem_ctx);
		progress2 = optimize_redundant_jumps(ir); progress |= progress2; if (progress2) debug_print_ir ("After redundant jumps", ir, state, mem_ctx);

		// do loop stuff only when linked; otherwise causes duplicate loop induction variable
		// problems (ast-in.txt test)
		if (linked)
		{
			loop_state *ls = analyze_loop_variables(ir);
			if (ls->loop_found) {
				progress2 = unroll_loops(ir, ls, &state->ctx->Const.ShaderCompilerOptions[state->stage]); progress |= progress2; if (progress2) debug_print_ir ("After unroll", ir, state, mem_ctx);
			}
			delete ls;
		}
	} while (progress && passes < kMaximumPasses);

	// GLSL/ES does not have saturate, so lower it
	lower_instructions(ir, SAT_TO_CLAMP);
}

// FIXME
// static void glsl_type_to_optimizer_desc(const glsl_type* type, glsl_precision prec, glslopt_shader_var* out)
// {
// 	out->arraySize = type->array_size();

// 	// type; use element type when in array
// 	if (type->is_array())
// 		type = type->element_type();

// 	if (type->is_float())
// 		out->type = kGlslTypeFloat;
// 	else if (type->is_integer())
// 		out->type = kGlslTypeInt;
// 	else if (type->is_boolean())
// 		out->type = kGlslTypeBool;
// 	else if (type->is_sampler())
// 	{
// 		if (type->sampler_dimensionality == GLSL_SAMPLER_DIM_2D)
// 		{
// 			if (type->sampler_shadow)
// 				out->type = kGlslTypeTex2DShadow;
// 			else if (type->sampler_array)
// 				out->type = kGlslTypeTex2DArray;
// 			else
// 				out->type = kGlslTypeTex2D;
// 		}
// 		else if (type->sampler_dimensionality == GLSL_SAMPLER_DIM_3D)
// 			out->type = kGlslTypeTex3D;
// 		else if (type->sampler_dimensionality == GLSL_SAMPLER_DIM_CUBE)
// 			out->type = kGlslTypeTexCube;
// 		else
// 			out->type = kGlslTypeOther;
// 	}
// 	else
// 		out->type = kGlslTypeOther;

// 	// sizes
// 	out->vectorSize = type->vector_elements;
// 	out->matrixSize = type->matrix_columns;

// 	// precision
// 	switch (prec)
// 	{
// 		case glsl_precision_high: out->prec = kGlslPrecHigh; break;
// 		case glsl_precision_medium: out->prec = kGlslPrecMedium; break;
// 		case glsl_precision_low: out->prec = kGlslPrecLow; break;
// 		default: out->prec = kGlslPrecHigh; break;
// 	}
// }

static void find_shader_variables(glslopt_shader* sh, exec_list* ir)
{
	foreach_in_list(ir_instruction, node, ir)
	{
		ir_variable* const var = node->as_variable();
		if (var == NULL)
			continue;
		if (var->data.mode == ir_var_shader_in)
		{
			if (sh->inputCount >= glslopt_shader::kMaxShaderInputs)
				continue;

			glslopt_shader_var& v = sh->inputs[sh->inputCount];
			v.name = ralloc_strdup(sh, var->name);
			// glsl_type_to_optimizer_desc(var->type, (glsl_precision)var->data.precision, &v);
			v.location = var->data.explicit_location ? var->data.location : -1;
			++sh->inputCount;
		}
		if (var->data.mode == ir_var_uniform && !var->type->is_sampler())
		{
			if (sh->uniformCount >= glslopt_shader::kMaxShaderUniforms)
				continue;

			glslopt_shader_var& v = sh->uniforms[sh->uniformCount];
			v.name = ralloc_strdup(sh, var->name);
			// glsl_type_to_optimizer_desc(var->type, (glsl_precision)var->data.precision, &v);
			v.location = var->data.explicit_location ? var->data.location : -1;
			++sh->uniformCount;
		}
		if (var->data.mode == ir_var_uniform && var->type->is_sampler())
		{
			if (sh->textureCount >= glslopt_shader::kMaxShaderTextures)
				continue;
			
			glslopt_shader_var& v = sh->textures[sh->textureCount];
			v.name = ralloc_strdup(sh, var->name);
			// glsl_type_to_optimizer_desc(var->type, (glsl_precision)var->data.precision, &v);
			v.location = var->data.explicit_location ? var->data.location : -1;
			++sh->textureCount;
		}
	}
}

glslopt_shader* glslopt_optimize (glslopt_ctx* ctx, glslopt_shader_type type, const char* shaderSource, unsigned options)
{
	glslopt_shader* shader = new (ctx->mem_ctx) glslopt_shader ();

	PrintGlslMode printMode = kPrintGlslVertex;
	switch (type) {
	case kGlslOptShaderVertex:
			shader->shader->Type = GL_VERTEX_SHADER;
			shader->shader->Stage = MESA_SHADER_VERTEX;
			printMode = kPrintGlslVertex;
			break;
	case kGlslOptShaderFragment:
			shader->shader->Type = GL_FRAGMENT_SHADER;
			shader->shader->Stage = MESA_SHADER_FRAGMENT;
			printMode = kPrintGlslFragment;
			break;
	}
	if (!shader->shader->Type)
	{
		shader->infoLog = ralloc_asprintf (shader, "Unknown shader type %d", (int)type);
		shader->status = false;
		return shader;
	}

	_mesa_glsl_parse_state* state = new (shader) _mesa_glsl_parse_state (&ctx->mesa_ctx, shader->shader->Stage, shader);
	state->error = 0;

	if (!(options & kGlslOptionSkipPreprocessor))
	{
		state->error = !!glcpp_preprocess (state, &shaderSource, &state->info_log, add_builtin_defines, state, &ctx->mesa_ctx);
		if (state->error)
		{
			shader->status = !state->error;
			shader->infoLog = state->info_log;
			return shader;
		}
	}

	_mesa_glsl_lexer_ctor (state, shaderSource);
	_mesa_glsl_parse (state);
	_mesa_glsl_lexer_dtor (state);

	exec_list* ir = new (shader) exec_list();
	shader->shader->ir = ir;

	if (!state->error && !state->translation_unit.is_empty())
		_mesa_ast_to_hir (ir, state);

	// Un-optimized output
	if (!state->error) {
		validate_ir_tree(ir);
		shader->rawOutput = _mesa_print_ir_glsl(ir, state, ralloc_strdup(shader, ""), printMode);
	}

	// Lower builtin functions prior to linking.
	lower_builtins(ir);

	// Link built-in functions
	shader->shader->symbols = state->symbols;
	
	struct gl_linked_shader* linked_shader = NULL;

	if (!state->error && !ir->is_empty() && !(options & kGlslOptionNotFullShader))
	{
		linked_shader = link_intrastage_shaders(shader,
												&ctx->mesa_ctx,
												shader->whole_program,
												shader->whole_program->Shaders,
												shader->whole_program->NumShaders,
												true);
		if (!linked_shader)
		{
			shader->status = false;
			shader->infoLog = shader->whole_program->data->InfoLog;
			return shader;
		}
		ir = linked_shader->ir;
		
		debug_print_ir ("==== After link ====", ir, state, shader);
	}

	// Do optimization post-link
	if (!state->error && !ir->is_empty())
	{		
		const bool linked = !(options & kGlslOptionNotFullShader);
		do_optimization_passes(ir, linked, state, shader);
		validate_ir_tree(ir);
	}	
	
	// Final optimized output
	if (!state->error)
	{
		shader->optimizedOutput = _mesa_print_ir_glsl(ir, state, ralloc_strdup(shader, ""), printMode);
	}

	shader->status = !state->error;
	shader->infoLog = state->info_log;

	find_shader_variables (shader, ir);
	// FIXME: stats
	// if (!state->error)
	// 	calculate_shader_stats (ir, &shader->statsMath, &shader->statsTex, &shader->statsFlow);

	ralloc_free (ir);
	ralloc_free (state);

	if (linked_shader)
		ralloc_free(linked_shader);

	return shader;
}

void glslopt_shader_delete (glslopt_shader* shader)
{
	delete shader;
}

bool glslopt_get_status (glslopt_shader* shader)
{
	return shader->status;
}

const char* glslopt_get_output (glslopt_shader* shader)
{
	return shader->optimizedOutput;
}

const char* glslopt_get_raw_output (glslopt_shader* shader)
{
	return shader->rawOutput;
}

const char* glslopt_get_log (glslopt_shader* shader)
{
	return shader->infoLog;
}

int glslopt_shader_get_input_count (glslopt_shader* shader)
{
	return shader->inputCount;
}

int glslopt_shader_get_uniform_count (glslopt_shader* shader)
{
	return shader->uniformCount;
}

int glslopt_shader_get_uniform_total_size (glslopt_shader* shader)
{
	return shader->uniformsSize;
}

int glslopt_shader_get_texture_count (glslopt_shader* shader)
{
	return shader->textureCount;
}

void glslopt_shader_get_input_desc (glslopt_shader* shader, int index, const char** outName, glslopt_basic_type* outType, glslopt_precision* outPrec, int* outVecSize, int* outMatSize, int* outArraySize, int* outLocation)
{
	const glslopt_shader_var& v = shader->inputs[index];
	*outName = v.name;
	*outType = v.type;
	*outPrec = v.prec;
	*outVecSize = v.vectorSize;
	*outMatSize = v.matrixSize;
	*outArraySize = v.arraySize;
	*outLocation = v.location;
}

void glslopt_shader_get_uniform_desc (glslopt_shader* shader, int index, const char** outName, glslopt_basic_type* outType, glslopt_precision* outPrec, int* outVecSize, int* outMatSize, int* outArraySize, int* outLocation)
{
	const glslopt_shader_var& v = shader->uniforms[index];
	*outName = v.name;
	*outType = v.type;
	*outPrec = v.prec;
	*outVecSize = v.vectorSize;
	*outMatSize = v.matrixSize;
	*outArraySize = v.arraySize;
	*outLocation = v.location;
}

void glslopt_shader_get_texture_desc (glslopt_shader* shader, int index, const char** outName, glslopt_basic_type* outType, glslopt_precision* outPrec, int* outVecSize, int* outMatSize, int* outArraySize, int* outLocation)
{
	const glslopt_shader_var& v = shader->textures[index];
	*outName = v.name;
	*outType = v.type;
	*outPrec = v.prec;
	*outVecSize = v.vectorSize;
	*outMatSize = v.matrixSize;
	*outArraySize = v.arraySize;
	*outLocation = v.location;
}

void glslopt_shader_get_stats (glslopt_shader* shader, int* approxMath, int* approxTex, int* approxFlow)
{
	*approxMath = shader->statsMath;
	*approxTex = shader->statsTex;
	*approxFlow = shader->statsFlow;
}
