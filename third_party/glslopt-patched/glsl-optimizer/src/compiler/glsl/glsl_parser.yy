%{
/*
 * Copyright © 2008, 2009 Intel Corporation
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
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifndef _MSC_VER
#include <strings.h>
#endif
#include <assert.h>

#include "ast.h"
#include "glsl_parser_extras.h"
#include "compiler/glsl_types.h"
#include "main/context.h"
#include "util/u_string.h"
#include "util/format/u_format.h"

#ifdef _MSC_VER
#pragma warning( disable : 4065 ) // switch statement contains 'default' but no 'case' labels
#endif

#undef yyerror

static void yyerror(YYLTYPE *loc, _mesa_glsl_parse_state *st, const char *msg)
{
   _mesa_glsl_error(loc, st, "%s", msg);
}

static int
_mesa_glsl_lex(YYSTYPE *val, YYLTYPE *loc, _mesa_glsl_parse_state *state)
{
   return _mesa_glsl_lexer_lex(val, loc, state->scanner);
}

static bool match_layout_qualifier(const char *s1, const char *s2,
                                   _mesa_glsl_parse_state *state)
{
   /* From the GLSL 1.50 spec, section 4.3.8 (Layout Qualifiers):
    *
    *     "The tokens in any layout-qualifier-id-list ... are not case
    *     sensitive, unless explicitly noted otherwise."
    *
    * The text "unless explicitly noted otherwise" appears to be
    * vacuous--no desktop GLSL spec (up through GLSL 4.40) notes
    * otherwise.
    *
    * However, the GLSL ES 3.00 spec says, in section 4.3.8 (Layout
    * Qualifiers):
    *
    *     "As for other identifiers, they are case sensitive."
    *
    * So we need to do a case-sensitive or a case-insensitive match,
    * depending on whether we are compiling for GLSL ES.
    */
   if (state->es_shader)
      return strcmp(s1, s2);
   else
      return strcasecmp(s1, s2);
}
%}

%expect 0

%define api.pure
%define parse.error verbose

%locations
%initial-action {
   @$.first_line = 1;
   @$.first_column = 1;
   @$.last_line = 1;
   @$.last_column = 1;
   @$.source = 0;
   @$.path = NULL;
}

%lex-param   {struct _mesa_glsl_parse_state *state}
%parse-param {struct _mesa_glsl_parse_state *state}

%union {
   int n;
   int64_t n64;
   float real;
   double dreal;
   const char *identifier;

   struct ast_type_qualifier type_qualifier;

   ast_node *node;
   ast_type_specifier *type_specifier;
   ast_array_specifier *array_specifier;
   ast_fully_specified_type *fully_specified_type;
   ast_function *function;
   ast_parameter_declarator *parameter_declarator;
   ast_function_definition *function_definition;
   ast_compound_statement *compound_statement;
   ast_expression *expression;
   ast_declarator_list *declarator_list;
   ast_struct_specifier *struct_specifier;
   ast_declaration *declaration;
   ast_switch_body *switch_body;
   ast_case_label *case_label;
   ast_case_label_list *case_label_list;
   ast_case_statement *case_statement;
   ast_case_statement_list *case_statement_list;
   ast_interface_block *interface_block;
   ast_subroutine_list *subroutine_list;
   struct {
      ast_node *cond;
      ast_expression *rest;
   } for_rest_statement;

   struct {
      ast_node *then_statement;
      ast_node *else_statement;
   } selection_rest_statement;

   const glsl_type *type;
}

%token ATTRIBUTE CONST_TOK
%token <type> BASIC_TYPE_TOK
%token BREAK BUFFER CONTINUE DO ELSE FOR IF DEMOTE DISCARD RETURN SWITCH CASE DEFAULT
%token CENTROID IN_TOK OUT_TOK INOUT_TOK UNIFORM VARYING SAMPLE
%token NOPERSPECTIVE FLAT SMOOTH
%token IMAGE1DSHADOW IMAGE2DSHADOW IMAGE1DARRAYSHADOW IMAGE2DARRAYSHADOW
%token COHERENT VOLATILE RESTRICT READONLY WRITEONLY
%token SHARED
%token STRUCT VOID_TOK WHILE
%token <identifier> IDENTIFIER TYPE_IDENTIFIER NEW_IDENTIFIER
%type <identifier> any_identifier
%type <interface_block> instance_name_opt
%token <real> FLOATCONSTANT
%token <dreal> DOUBLECONSTANT
%token <n> INTCONSTANT UINTCONSTANT BOOLCONSTANT
%token <n64> INT64CONSTANT UINT64CONSTANT
%token <identifier> FIELD_SELECTION
%token LEFT_OP RIGHT_OP
%token INC_OP DEC_OP LE_OP GE_OP EQ_OP NE_OP
%token AND_OP OR_OP XOR_OP MUL_ASSIGN DIV_ASSIGN ADD_ASSIGN
%token MOD_ASSIGN LEFT_ASSIGN RIGHT_ASSIGN AND_ASSIGN XOR_ASSIGN OR_ASSIGN
%token SUB_ASSIGN
%token INVARIANT PRECISE
%token LOWP MEDIUMP HIGHP SUPERP PRECISION

%token VERSION_TOK EXTENSION LINE COLON EOL INTERFACE OUTPUT
%token PRAGMA_DEBUG_ON PRAGMA_DEBUG_OFF
%token PRAGMA_OPTIMIZE_ON PRAGMA_OPTIMIZE_OFF
%token PRAGMA_WARNING_ON PRAGMA_WARNING_OFF
%token PRAGMA_INVARIANT_ALL
%token LAYOUT_TOK
%token DOT_TOK
   /* Reserved words that are not actually used in the grammar.
    */
%token ASM CLASS UNION ENUM TYPEDEF TEMPLATE THIS PACKED_TOK GOTO
%token INLINE_TOK NOINLINE PUBLIC_TOK STATIC EXTERN EXTERNAL
%token LONG_TOK SHORT_TOK HALF FIXED_TOK UNSIGNED INPUT_TOK
%token HVEC2 HVEC3 HVEC4 FVEC2 FVEC3 FVEC4
%token SAMPLER3DRECT
%token SIZEOF CAST NAMESPACE USING
%token RESOURCE PATCH
%token SUBROUTINE

%token ERROR_TOK

%token COMMON PARTITION ACTIVE FILTER ROW_MAJOR

%type <identifier> variable_identifier
%type <node> statement
%type <node> statement_list
%type <node> simple_statement
%type <n> precision_qualifier
%type <type_qualifier> type_qualifier
%type <type_qualifier> auxiliary_storage_qualifier
%type <type_qualifier> storage_qualifier
%type <type_qualifier> interpolation_qualifier
%type <type_qualifier> layout_qualifier
%type <type_qualifier> layout_qualifier_id_list layout_qualifier_id
%type <type_qualifier> interface_block_layout_qualifier
%type <type_qualifier> memory_qualifier
%type <type_qualifier> subroutine_qualifier
%type <subroutine_list> subroutine_type_list
%type <type_qualifier> interface_qualifier
%type <type_specifier> type_specifier
%type <type_specifier> type_specifier_nonarray
%type <array_specifier> array_specifier
%type <type> basic_type_specifier_nonarray
%type <fully_specified_type> fully_specified_type
%type <function> function_prototype
%type <function> function_header
%type <function> function_header_with_parameters
%type <function> function_declarator
%type <parameter_declarator> parameter_declarator
%type <parameter_declarator> parameter_declaration
%type <type_qualifier> parameter_qualifier
%type <type_qualifier> parameter_direction_qualifier
%type <type_specifier> parameter_type_specifier
%type <function_definition> function_definition
%type <compound_statement> compound_statement_no_new_scope
%type <compound_statement> compound_statement
%type <node> statement_no_new_scope
%type <node> expression_statement
%type <expression> expression
%type <expression> primary_expression
%type <expression> assignment_expression
%type <expression> conditional_expression
%type <expression> logical_or_expression
%type <expression> logical_xor_expression
%type <expression> logical_and_expression
%type <expression> inclusive_or_expression
%type <expression> exclusive_or_expression
%type <expression> and_expression
%type <expression> equality_expression
%type <expression> relational_expression
%type <expression> shift_expression
%type <expression> additive_expression
%type <expression> multiplicative_expression
%type <expression> unary_expression
%type <expression> constant_expression
%type <expression> integer_expression
%type <expression> postfix_expression
%type <expression> function_call_header_with_parameters
%type <expression> function_call_header_no_parameters
%type <expression> function_call_header
%type <expression> function_call_generic
%type <expression> function_call_or_method
%type <expression> function_call
%type <n> assignment_operator
%type <n> unary_operator
%type <expression> function_identifier
%type <node> external_declaration
%type <node> pragma_statement
%type <declarator_list> init_declarator_list
%type <declarator_list> single_declaration
%type <expression> initializer
%type <expression> initializer_list
%type <node> declaration
%type <node> declaration_statement
%type <node> jump_statement
%type <node> demote_statement
%type <node> interface_block
%type <interface_block> basic_interface_block
%type <struct_specifier> struct_specifier
%type <declarator_list> struct_declaration_list
%type <declarator_list> struct_declaration
%type <declaration> struct_declarator
%type <declaration> struct_declarator_list
%type <declarator_list> member_list
%type <declarator_list> member_declaration
%type <node> selection_statement
%type <selection_rest_statement> selection_rest_statement
%type <node> switch_statement
%type <switch_body> switch_body
%type <case_label_list> case_label_list
%type <case_label> case_label
%type <case_statement> case_statement
%type <case_statement_list> case_statement_list
%type <node> iteration_statement
%type <node> condition
%type <node> conditionopt
%type <node> for_init_statement
%type <for_rest_statement> for_rest_statement
%type <node> layout_defaults
%type <type_qualifier> layout_uniform_defaults
%type <type_qualifier> layout_buffer_defaults
%type <type_qualifier> layout_in_defaults
%type <type_qualifier> layout_out_defaults

%right THEN ELSE
%%

translation_unit:
   version_statement extension_statement_list
   {
      _mesa_glsl_initialize_types(state);
   }
   external_declaration_list
   {
      delete state->symbols;
      state->symbols = new(ralloc_parent(state)) glsl_symbol_table;
      if (state->es_shader) {
         if (state->stage == MESA_SHADER_FRAGMENT) {
            state->symbols->add_default_precision_qualifier("int", ast_precision_medium);
         } else {
            state->symbols->add_default_precision_qualifier("float", ast_precision_high);
            state->symbols->add_default_precision_qualifier("int", ast_precision_high);
         }
         state->symbols->add_default_precision_qualifier("sampler2D", ast_precision_low);
         state->symbols->add_default_precision_qualifier("samplerExternalOES", ast_precision_low);
         state->symbols->add_default_precision_qualifier("samplerCube", ast_precision_low);
         state->symbols->add_default_precision_qualifier("atomic_uint", ast_precision_high);
      }
      _mesa_glsl_initialize_types(state);
   }
   ;

version_statement:
   /* blank - no #version specified: defaults are already set */
   | VERSION_TOK INTCONSTANT EOL
   {
      state->process_version_directive(&@2, $2, NULL);
      if (state->error) {
         YYERROR;
      }
   }
   | VERSION_TOK INTCONSTANT any_identifier EOL
   {
      state->process_version_directive(&@2, $2, $3);
      if (state->error) {
         YYERROR;
      }
   }
   ;

pragma_statement:
   PRAGMA_DEBUG_ON EOL { $$ = NULL; }
   | PRAGMA_DEBUG_OFF EOL { $$ = NULL; }
   | PRAGMA_OPTIMIZE_ON EOL { $$ = NULL; }
   | PRAGMA_OPTIMIZE_OFF EOL { $$ = NULL; }
   | PRAGMA_INVARIANT_ALL EOL
   {
      /* Pragma invariant(all) cannot be used in a fragment shader.
       *
       * Page 27 of the GLSL 1.20 spec, Page 53 of the GLSL ES 3.00 spec:
       *
       *     "It is an error to use this pragma in a fragment shader."
       */
      if (state->is_version(120, 300) &&
          state->stage == MESA_SHADER_FRAGMENT) {
         _mesa_glsl_error(& @1, state,
                          "pragma `invariant(all)' cannot be used "
                          "in a fragment shader.");
      } else if (!state->is_version(120, 100)) {
         _mesa_glsl_warning(& @1, state,
                            "pragma `invariant(all)' not supported in %s "
                            "(GLSL ES 1.00 or GLSL 1.20 required)",
                            state->get_version_string());
      } else {
         state->all_invariant = true;
      }

      $$ = NULL;
   }
   | PRAGMA_WARNING_ON EOL
   {
      void *mem_ctx = state->linalloc;
      $$ = new(mem_ctx) ast_warnings_toggle(true);
   }
   | PRAGMA_WARNING_OFF EOL
   {
      void *mem_ctx = state->linalloc;
      $$ = new(mem_ctx) ast_warnings_toggle(false);
   }
   ;

extension_statement_list:

   | extension_statement_list extension_statement
   ;

any_identifier:
   IDENTIFIER
   | TYPE_IDENTIFIER
   | NEW_IDENTIFIER
   ;

extension_statement:
   EXTENSION any_identifier COLON any_identifier EOL
   {
      if (!_mesa_glsl_process_extension($2, & @2, $4, & @4, state)) {
         YYERROR;
      }
   }
   ;

external_declaration_list:
   external_declaration
   {
      /* FINISHME: The NULL test is required because pragmas are set to
       * FINISHME: NULL. (See production rule for external_declaration.)
       */
      if ($1 != NULL)
         state->translation_unit.push_tail(& $1->link);
   }
   | external_declaration_list external_declaration
   {
      /* FINISHME: The NULL test is required because pragmas are set to
       * FINISHME: NULL. (See production rule for external_declaration.)
       */
      if ($2 != NULL)
         state->translation_unit.push_tail(& $2->link);
   }
   | external_declaration_list extension_statement {
      if (!state->allow_extension_directive_midshader) {
         _mesa_glsl_error(& @2, state,
                          "#extension directive is not allowed "
                          "in the middle of a shader");
         YYERROR;
      }
   }
   ;

variable_identifier:
   IDENTIFIER
   | NEW_IDENTIFIER
   ;

primary_expression:
   variable_identifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_identifier, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.identifier = $1;
   }
   | INTCONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_int_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.int_constant = $1;
   }
   | UINTCONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_uint_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.uint_constant = $1;
   }
   | INT64CONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_int64_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.int64_constant = $1;
   }
   | UINT64CONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_uint64_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.uint64_constant = $1;
   }
   | FLOATCONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_float_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.float_constant = $1;
   }
   | DOUBLECONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_double_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.double_constant = $1;
   }
   | BOOLCONSTANT
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_bool_constant, NULL, NULL, NULL);
      $$->set_location(@1);
      $$->primary_expression.bool_constant = $1;
   }
   | '(' expression ')'
   {
      $$ = $2;
   }
   ;

postfix_expression:
   primary_expression
   | postfix_expression '[' integer_expression ']'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_array_index, $1, $3, NULL);
      $$->set_location_range(@1, @4);
   }
   | function_call
   {
      $$ = $1;
   }
   | postfix_expression DOT_TOK FIELD_SELECTION
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_field_selection, $1, NULL, NULL);
      $$->set_location_range(@1, @3);
      $$->primary_expression.identifier = $3;
   }
   | postfix_expression INC_OP
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_post_inc, $1, NULL, NULL);
      $$->set_location_range(@1, @2);
   }
   | postfix_expression DEC_OP
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_post_dec, $1, NULL, NULL);
      $$->set_location_range(@1, @2);
   }
   ;

integer_expression:
   expression
   ;

function_call:
   function_call_or_method
   ;

function_call_or_method:
   function_call_generic
   ;

function_call_generic:
   function_call_header_with_parameters ')'
   | function_call_header_no_parameters ')'
   ;

function_call_header_no_parameters:
   function_call_header VOID_TOK
   | function_call_header
   ;

function_call_header_with_parameters:
   function_call_header assignment_expression
   {
      $$ = $1;
      $$->set_location(@1);
      $$->expressions.push_tail(& $2->link);
   }
   | function_call_header_with_parameters ',' assignment_expression
   {
      $$ = $1;
      $$->set_location(@1);
      $$->expressions.push_tail(& $3->link);
   }
   ;

   // Grammar Note: Constructors look like functions, but lexical
   // analysis recognized most of them as keywords. They are now
   // recognized through "type_specifier".
function_call_header:
   function_identifier '('
   ;

function_identifier:
   type_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_function_expression($1);
      $$->set_location(@1);
      }
   | postfix_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_function_expression($1);
      $$->set_location(@1);
      }
   ;

   // Grammar Note: Constructors look like methods, but lexical
   // analysis recognized most of them as keywords. They are now
   // recognized through "type_specifier".

   // Grammar Note: No traditional style type casts.
unary_expression:
   postfix_expression
   | INC_OP unary_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_pre_inc, $2, NULL, NULL);
      $$->set_location(@1);
   }
   | DEC_OP unary_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_pre_dec, $2, NULL, NULL);
      $$->set_location(@1);
   }
   | unary_operator unary_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression($1, $2, NULL, NULL);
      $$->set_location_range(@1, @2);
   }
   ;

   // Grammar Note: No '*' or '&' unary ops. Pointers are not supported.
unary_operator:
   '+'   { $$ = ast_plus; }
   | '-' { $$ = ast_neg; }
   | '!' { $$ = ast_logic_not; }
   | '~' { $$ = ast_bit_not; }
   ;

multiplicative_expression:
   unary_expression
   | multiplicative_expression '*' unary_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_mul, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | multiplicative_expression '/' unary_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_div, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | multiplicative_expression '%' unary_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_mod, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

additive_expression:
   multiplicative_expression
   | additive_expression '+' multiplicative_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_add, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | additive_expression '-' multiplicative_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_sub, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

shift_expression:
   additive_expression
   | shift_expression LEFT_OP additive_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_lshift, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | shift_expression RIGHT_OP additive_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_rshift, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

relational_expression:
   shift_expression
   | relational_expression '<' shift_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_less, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | relational_expression '>' shift_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_greater, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | relational_expression LE_OP shift_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_lequal, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | relational_expression GE_OP shift_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_gequal, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

equality_expression:
   relational_expression
   | equality_expression EQ_OP relational_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_equal, $1, $3);
      $$->set_location_range(@1, @3);
   }
   | equality_expression NE_OP relational_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_nequal, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

and_expression:
   equality_expression
   | and_expression '&' equality_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_bit_and, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

exclusive_or_expression:
   and_expression
   | exclusive_or_expression '^' and_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_bit_xor, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

inclusive_or_expression:
   exclusive_or_expression
   | inclusive_or_expression '|' exclusive_or_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_bit_or, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

logical_and_expression:
   inclusive_or_expression
   | logical_and_expression AND_OP inclusive_or_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_logic_and, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

logical_xor_expression:
   logical_and_expression
   | logical_xor_expression XOR_OP logical_and_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_logic_xor, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

logical_or_expression:
   logical_xor_expression
   | logical_or_expression OR_OP logical_xor_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_bin(ast_logic_or, $1, $3);
      $$->set_location_range(@1, @3);
   }
   ;

conditional_expression:
   logical_or_expression
   | logical_or_expression '?' expression ':' assignment_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression(ast_conditional, $1, $3, $5);
      $$->set_location_range(@1, @5);
   }
   ;

assignment_expression:
   conditional_expression
   | unary_expression assignment_operator assignment_expression
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression($2, $1, $3, NULL);
      $$->set_location_range(@1, @3);
   }
   ;

assignment_operator:
   '='                { $$ = ast_assign; }
   | MUL_ASSIGN       { $$ = ast_mul_assign; }
   | DIV_ASSIGN       { $$ = ast_div_assign; }
   | MOD_ASSIGN       { $$ = ast_mod_assign; }
   | ADD_ASSIGN       { $$ = ast_add_assign; }
   | SUB_ASSIGN       { $$ = ast_sub_assign; }
   | LEFT_ASSIGN      { $$ = ast_ls_assign; }
   | RIGHT_ASSIGN     { $$ = ast_rs_assign; }
   | AND_ASSIGN       { $$ = ast_and_assign; }
   | XOR_ASSIGN       { $$ = ast_xor_assign; }
   | OR_ASSIGN        { $$ = ast_or_assign; }
   ;

expression:
   assignment_expression
   {
      $$ = $1;
   }
   | expression ',' assignment_expression
   {
      void *ctx = state->linalloc;
      if ($1->oper != ast_sequence) {
         $$ = new(ctx) ast_expression(ast_sequence, NULL, NULL, NULL);
         $$->set_location_range(@1, @3);
         $$->expressions.push_tail(& $1->link);
      } else {
         $$ = $1;
      }

      $$->expressions.push_tail(& $3->link);
   }
   ;

constant_expression:
   conditional_expression
   ;

declaration:
   function_prototype ';'
   {
      state->symbols->pop_scope();
      $$ = $1;
   }
   | init_declarator_list ';'
   {
      $$ = $1;
   }
   | PRECISION precision_qualifier type_specifier ';'
   {
      $3->default_precision = $2;
      $$ = $3;
   }
   | interface_block
   {
      ast_interface_block *block = (ast_interface_block *) $1;
      if (block->layout.has_layout() || block->layout.has_memory()) {
         if (!block->default_layout.merge_qualifier(& @1, state, block->layout, false)) {
            YYERROR;
         }
      }
      block->layout = block->default_layout;
      if (!block->layout.push_to_global(& @1, state)) {
         YYERROR;
      }
      $$ = $1;
   }
   ;

function_prototype:
   function_declarator ')'
   ;

function_declarator:
   function_header
   | function_header_with_parameters
   ;

function_header_with_parameters:
   function_header parameter_declaration
   {
      $$ = $1;
      $$->parameters.push_tail(& $2->link);
   }
   | function_header_with_parameters ',' parameter_declaration
   {
      $$ = $1;
      $$->parameters.push_tail(& $3->link);
   }
   ;

function_header:
   fully_specified_type variable_identifier '('
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_function();
      $$->set_location(@2);
      $$->return_type = $1;
      $$->identifier = $2;

      if ($1->qualifier.is_subroutine_decl()) {
         /* add type for IDENTIFIER search */
         state->symbols->add_type($2, glsl_type::get_subroutine_instance($2));
      } else
         state->symbols->add_function(new(state) ir_function($2));
      state->symbols->push_scope();
   }
   ;

parameter_declarator:
   type_specifier any_identifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_parameter_declarator();
      $$->set_location_range(@1, @2);
      $$->type = new(ctx) ast_fully_specified_type();
      $$->type->set_location(@1);
      $$->type->specifier = $1;
      $$->identifier = $2;
      state->symbols->add_variable(new(state) ir_variable(NULL, $2, ir_var_auto));
   }
   | layout_qualifier type_specifier any_identifier
   {
      if (state->allow_layout_qualifier_on_function_parameter) {
         void *ctx = state->linalloc;
         $$ = new(ctx) ast_parameter_declarator();
         $$->set_location_range(@2, @3);
         $$->type = new(ctx) ast_fully_specified_type();
         $$->type->set_location(@2);
         $$->type->specifier = $2;
         $$->identifier = $3;
         state->symbols->add_variable(new(state) ir_variable(NULL, $3, ir_var_auto));
      } else {
         _mesa_glsl_error(&@1, state,
                          "is is not allowed on function parameter");
         YYERROR;
      }
   }
   | type_specifier any_identifier array_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_parameter_declarator();
      $$->set_location_range(@1, @3);
      $$->type = new(ctx) ast_fully_specified_type();
      $$->type->set_location(@1);
      $$->type->specifier = $1;
      $$->identifier = $2;
      $$->array_specifier = $3;
      state->symbols->add_variable(new(state) ir_variable(NULL, $2, ir_var_auto));
   }
   ;

parameter_declaration:
   parameter_qualifier parameter_declarator
   {
      $$ = $2;
      $$->type->qualifier = $1;
      if (!$$->type->qualifier.push_to_global(& @1, state)) {
         YYERROR;
      }
   }
   | parameter_qualifier parameter_type_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_parameter_declarator();
      $$->set_location(@2);
      $$->type = new(ctx) ast_fully_specified_type();
      $$->type->set_location_range(@1, @2);
      $$->type->qualifier = $1;
      if (!$$->type->qualifier.push_to_global(& @1, state)) {
         YYERROR;
      }
      $$->type->specifier = $2;
   }
   ;

parameter_qualifier:
   /* empty */
   {
      memset(& $$, 0, sizeof($$));
   }
   | CONST_TOK parameter_qualifier
   {
      if ($2.flags.q.constant)
         _mesa_glsl_error(&@1, state, "duplicate const qualifier");

      $$ = $2;
      $$.flags.q.constant = 1;
   }
   | PRECISE parameter_qualifier
   {
      if ($2.flags.q.precise)
         _mesa_glsl_error(&@1, state, "duplicate precise qualifier");

      $$ = $2;
      $$.flags.q.precise = 1;
   }
   | parameter_direction_qualifier parameter_qualifier
   {
      if (($1.flags.q.in || $1.flags.q.out) && ($2.flags.q.in || $2.flags.q.out))
         _mesa_glsl_error(&@1, state, "duplicate in/out/inout qualifier");

      if (!state->has_420pack_or_es31() && $2.flags.q.constant)
         _mesa_glsl_error(&@1, state, "in/out/inout must come after const "
                                      "or precise");

      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }
   | precision_qualifier parameter_qualifier
   {
      if ($2.precision != ast_precision_none)
         _mesa_glsl_error(&@1, state, "duplicate precision qualifier");

      if (!state->has_420pack_or_es31() &&
          $2.flags.i != 0)
         _mesa_glsl_error(&@1, state, "precision qualifiers must come last");

      $$ = $2;
      $$.precision = $1;
   }
   | memory_qualifier parameter_qualifier
   {
      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }

parameter_direction_qualifier:
   IN_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.in = 1;
   }
   | OUT_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.out = 1;
   }
   | INOUT_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.in = 1;
      $$.flags.q.out = 1;
   }
   ;

parameter_type_specifier:
   type_specifier
   ;

init_declarator_list:
   single_declaration
   | init_declarator_list ',' any_identifier
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($3, NULL, NULL);
      decl->set_location(@3);

      $$ = $1;
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $3, ir_var_auto));
   }
   | init_declarator_list ',' any_identifier array_specifier
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($3, $4, NULL);
      decl->set_location_range(@3, @4);

      $$ = $1;
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $3, ir_var_auto));
   }
   | init_declarator_list ',' any_identifier array_specifier '=' initializer
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($3, $4, $6);
      decl->set_location_range(@3, @4);

      $$ = $1;
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $3, ir_var_auto));
   }
   | init_declarator_list ',' any_identifier '=' initializer
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($3, NULL, $5);
      decl->set_location(@3);

      $$ = $1;
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $3, ir_var_auto));
   }
   ;

   // Grammar Note: No 'enum', or 'typedef'.
single_declaration:
   fully_specified_type
   {
      void *ctx = state->linalloc;
      /* Empty declaration list is valid. */
      $$ = new(ctx) ast_declarator_list($1);
      $$->set_location(@1);
   }
   | fully_specified_type any_identifier
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, NULL, NULL);
      decl->set_location(@2);

      $$ = new(ctx) ast_declarator_list($1);
      $$->set_location_range(@1, @2);
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $2, ir_var_auto));
   }
   | fully_specified_type any_identifier array_specifier
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, $3, NULL);
      decl->set_location_range(@2, @3);

      $$ = new(ctx) ast_declarator_list($1);
      $$->set_location_range(@1, @3);
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $2, ir_var_auto));
   }
   | fully_specified_type any_identifier array_specifier '=' initializer
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, $3, $5);
      decl->set_location_range(@2, @3);

      $$ = new(ctx) ast_declarator_list($1);
      $$->set_location_range(@1, @3);
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $2, ir_var_auto));
   }
   | fully_specified_type any_identifier '=' initializer
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, NULL, $4);
      decl->set_location(@2);

      $$ = new(ctx) ast_declarator_list($1);
      $$->set_location_range(@1, @2);
      $$->declarations.push_tail(&decl->link);
      state->symbols->add_variable(new(state) ir_variable(NULL, $2, ir_var_auto));
   }
   | INVARIANT variable_identifier
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, NULL, NULL);
      decl->set_location(@2);

      $$ = new(ctx) ast_declarator_list(NULL);
      $$->set_location_range(@1, @2);
      $$->invariant = true;

      $$->declarations.push_tail(&decl->link);
   }
   | PRECISE variable_identifier
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, NULL, NULL);
      decl->set_location(@2);

      $$ = new(ctx) ast_declarator_list(NULL);
      $$->set_location_range(@1, @2);
      $$->precise = true;

      $$->declarations.push_tail(&decl->link);
   }
   ;

fully_specified_type:
   type_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_fully_specified_type();
      $$->set_location(@1);
      $$->specifier = $1;
   }
   | type_qualifier type_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_fully_specified_type();
      $$->set_location_range(@1, @2);
      $$->qualifier = $1;
      if (!$$->qualifier.push_to_global(& @1, state)) {
         YYERROR;
      }
      $$->specifier = $2;
      if ($$->specifier->structure != NULL &&
          $$->specifier->structure->is_declaration) {
            $$->specifier->structure->layout = &$$->qualifier;
      }
   }
   ;

layout_qualifier:
   LAYOUT_TOK '(' layout_qualifier_id_list ')'
   {
      $$ = $3;
   }
   ;

layout_qualifier_id_list:
   layout_qualifier_id
   | layout_qualifier_id_list ',' layout_qualifier_id
   {
      $$ = $1;
      if (!$$.merge_qualifier(& @3, state, $3, true)) {
         YYERROR;
      }
   }
   ;

layout_qualifier_id:
   any_identifier
   {
      memset(& $$, 0, sizeof($$));

      /* Layout qualifiers for ARB_fragment_coord_conventions. */
      if (!$$.flags.i && (state->ARB_fragment_coord_conventions_enable ||
                          state->is_version(150, 0))) {
         if (match_layout_qualifier($1, "origin_upper_left", state) == 0) {
            $$.flags.q.origin_upper_left = 1;
         } else if (match_layout_qualifier($1, "pixel_center_integer",
                                           state) == 0) {
            $$.flags.q.pixel_center_integer = 1;
         }

         if ($$.flags.i && state->ARB_fragment_coord_conventions_warn) {
            _mesa_glsl_warning(& @1, state,
                               "GL_ARB_fragment_coord_conventions layout "
                               "identifier `%s' used", $1);
         }
      }

      /* Layout qualifiers for AMD/ARB_conservative_depth. */
      if (!$$.flags.i &&
          (state->AMD_conservative_depth_enable ||
           state->ARB_conservative_depth_enable ||
           state->is_version(420, 0))) {
         if (match_layout_qualifier($1, "depth_any", state) == 0) {
            $$.flags.q.depth_type = 1;
            $$.depth_type = ast_depth_any;
         } else if (match_layout_qualifier($1, "depth_greater", state) == 0) {
            $$.flags.q.depth_type = 1;
            $$.depth_type = ast_depth_greater;
         } else if (match_layout_qualifier($1, "depth_less", state) == 0) {
            $$.flags.q.depth_type = 1;
            $$.depth_type = ast_depth_less;
         } else if (match_layout_qualifier($1, "depth_unchanged",
                                           state) == 0) {
            $$.flags.q.depth_type = 1;
            $$.depth_type = ast_depth_unchanged;
         }

         if ($$.flags.i && state->AMD_conservative_depth_warn) {
            _mesa_glsl_warning(& @1, state,
                               "GL_AMD_conservative_depth "
                               "layout qualifier `%s' is used", $1);
         }
         if ($$.flags.i && state->ARB_conservative_depth_warn) {
            _mesa_glsl_warning(& @1, state,
                               "GL_ARB_conservative_depth "
                               "layout qualifier `%s' is used", $1);
         }
      }

      /* See also interface_block_layout_qualifier. */
      if (!$$.flags.i && state->has_uniform_buffer_objects()) {
         if (match_layout_qualifier($1, "std140", state) == 0) {
            $$.flags.q.std140 = 1;
         } else if (match_layout_qualifier($1, "shared", state) == 0) {
            $$.flags.q.shared = 1;
         } else if (match_layout_qualifier($1, "std430", state) == 0) {
            $$.flags.q.std430 = 1;
         } else if (match_layout_qualifier($1, "column_major", state) == 0) {
            $$.flags.q.column_major = 1;
         /* "row_major" is a reserved word in GLSL 1.30+. Its token is parsed
          * below in the interface_block_layout_qualifier rule.
          *
          * It is not a reserved word in GLSL ES 3.00, so it's handled here as
          * an identifier.
          *
          * Also, this takes care of alternate capitalizations of
          * "row_major" (which is necessary because layout qualifiers
          * are case-insensitive in desktop GLSL).
          */
         } else if (match_layout_qualifier($1, "row_major", state) == 0) {
            $$.flags.q.row_major = 1;
         /* "packed" is a reserved word in GLSL, and its token is
          * parsed below in the interface_block_layout_qualifier rule.
          * However, we must take care of alternate capitalizations of
          * "packed", because layout qualifiers are case-insensitive
          * in desktop GLSL.
          */
         } else if (match_layout_qualifier($1, "packed", state) == 0) {
           $$.flags.q.packed = 1;
         }

         if ($$.flags.i && state->ARB_uniform_buffer_object_warn) {
            _mesa_glsl_warning(& @1, state,
                               "#version 140 / GL_ARB_uniform_buffer_object "
                               "layout qualifier `%s' is used", $1);
         }
      }

      /* Layout qualifiers for GLSL 1.50 geometry shaders. */
      if (!$$.flags.i) {
         static const struct {
            const char *s;
            GLenum e;
         } map[] = {
                 { "points", GL_POINTS },
                 { "lines", GL_LINES },
                 { "lines_adjacency", GL_LINES_ADJACENCY },
                 { "line_strip", GL_LINE_STRIP },
                 { "triangles", GL_TRIANGLES },
                 { "triangles_adjacency", GL_TRIANGLES_ADJACENCY },
                 { "triangle_strip", GL_TRIANGLE_STRIP },
         };
         for (unsigned i = 0; i < ARRAY_SIZE(map); i++) {
            if (match_layout_qualifier($1, map[i].s, state) == 0) {
               $$.flags.q.prim_type = 1;
               $$.prim_type = map[i].e;
               break;
            }
         }

         if ($$.flags.i && !state->has_geometry_shader() &&
             !state->has_tessellation_shader()) {
            _mesa_glsl_error(& @1, state, "#version 150 layout "
                             "qualifier `%s' used", $1);
         }
      }

      /* Layout qualifiers for ARB_shader_image_load_store. */
      if (state->has_shader_image_load_store()) {
         if (!$$.flags.i) {
            static const struct {
               const char *name;
               enum pipe_format format;
               glsl_base_type base_type;
               /** Minimum desktop GLSL version required for the image
                * format.  Use 130 if already present in the original
                * ARB extension.
                */
               unsigned required_glsl;
               /** Minimum GLSL ES version required for the image format. */
               unsigned required_essl;
               /* NV_image_formats */
               bool nv_image_formats;
               bool ext_qualifiers;
            } map[] = {
               { "rgba32f", PIPE_FORMAT_R32G32B32A32_FLOAT, GLSL_TYPE_FLOAT, 130, 310, false, false },
               { "rgba16f", PIPE_FORMAT_R16G16B16A16_FLOAT, GLSL_TYPE_FLOAT, 130, 310, false, false },
               { "rg32f", PIPE_FORMAT_R32G32_FLOAT, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rg16f", PIPE_FORMAT_R16G16_FLOAT, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "r11f_g11f_b10f", PIPE_FORMAT_R11G11B10_FLOAT, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "r32f", PIPE_FORMAT_R32_FLOAT, GLSL_TYPE_FLOAT, 130, 310, false, false },
               { "r16f", PIPE_FORMAT_R16_FLOAT, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rgba32ui", PIPE_FORMAT_R32G32B32A32_UINT, GLSL_TYPE_UINT, 130, 310, false, false },
               { "rgba16ui", PIPE_FORMAT_R16G16B16A16_UINT, GLSL_TYPE_UINT, 130, 310, false, false },
               { "rgb10_a2ui", PIPE_FORMAT_R10G10B10A2_UINT, GLSL_TYPE_UINT, 130, 0, true, false },
               { "rgba8ui", PIPE_FORMAT_R8G8B8A8_UINT, GLSL_TYPE_UINT, 130, 310, false, false },
               { "rg32ui", PIPE_FORMAT_R32G32_UINT, GLSL_TYPE_UINT, 130, 0, true, false },
               { "rg16ui", PIPE_FORMAT_R16G16_UINT, GLSL_TYPE_UINT, 130, 0, true, false },
               { "rg8ui", PIPE_FORMAT_R8G8_UINT, GLSL_TYPE_UINT, 130, 0, true, false },
               { "r32ui", PIPE_FORMAT_R32_UINT, GLSL_TYPE_UINT, 130, 310, false, false },
               { "r16ui", PIPE_FORMAT_R16_UINT, GLSL_TYPE_UINT, 130, 0, true, false },
               { "r8ui", PIPE_FORMAT_R8_UINT, GLSL_TYPE_UINT, 130, 0, true, false },
               { "rgba32i", PIPE_FORMAT_R32G32B32A32_SINT, GLSL_TYPE_INT, 130, 310, false, false },
               { "rgba16i", PIPE_FORMAT_R16G16B16A16_SINT, GLSL_TYPE_INT, 130, 310, false, false },
               { "rgba8i", PIPE_FORMAT_R8G8B8A8_SINT, GLSL_TYPE_INT, 130, 310, false, false },
               { "rg32i", PIPE_FORMAT_R32G32_SINT, GLSL_TYPE_INT, 130, 0, true, false },
               { "rg16i", PIPE_FORMAT_R16G16_SINT, GLSL_TYPE_INT, 130, 0, true, false },
               { "rg8i", PIPE_FORMAT_R8G8_SINT, GLSL_TYPE_INT, 130, 0, true, false },
               { "r32i", PIPE_FORMAT_R32_SINT, GLSL_TYPE_INT, 130, 310, false, false },
               { "r16i", PIPE_FORMAT_R16_SINT, GLSL_TYPE_INT, 130, 0, true, false },
               { "r8i", PIPE_FORMAT_R8_SINT, GLSL_TYPE_INT, 130, 0, true, false },
               { "rgba16", PIPE_FORMAT_R16G16B16A16_UNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rgb10_a2", PIPE_FORMAT_R10G10B10A2_UNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rgba8", PIPE_FORMAT_R8G8B8A8_UNORM, GLSL_TYPE_FLOAT, 130, 310, false, false },
               { "rg16", PIPE_FORMAT_R16G16_UNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rg8", PIPE_FORMAT_R8G8_UNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "r16", PIPE_FORMAT_R16_UNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "r8", PIPE_FORMAT_R8_UNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rgba16_snorm", PIPE_FORMAT_R16G16B16A16_SNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rgba8_snorm", PIPE_FORMAT_R8G8B8A8_SNORM, GLSL_TYPE_FLOAT, 130, 310, false, false },
               { "rg16_snorm", PIPE_FORMAT_R16G16_SNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "rg8_snorm", PIPE_FORMAT_R8G8_SNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "r16_snorm", PIPE_FORMAT_R16_SNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },
               { "r8_snorm", PIPE_FORMAT_R8_SNORM, GLSL_TYPE_FLOAT, 130, 0, true, false },

               /* From GL_EXT_shader_image_load_store: */
               /* base_type is incorrect but it'll be patched later when we know
                * the variable type. See ast_to_hir.cpp */
               { "size1x8", PIPE_FORMAT_R8_SINT, GLSL_TYPE_VOID, 130, 0, false, true },
               { "size1x16", PIPE_FORMAT_R16_SINT, GLSL_TYPE_VOID, 130, 0, false, true },
               { "size1x32", PIPE_FORMAT_R32_SINT, GLSL_TYPE_VOID, 130, 0, false, true },
               { "size2x32", PIPE_FORMAT_R32G32_SINT, GLSL_TYPE_VOID, 130, 0, false, true },
               { "size4x32", PIPE_FORMAT_R32G32B32A32_SINT, GLSL_TYPE_VOID, 130, 0, false, true },
            };

            for (unsigned i = 0; i < ARRAY_SIZE(map); i++) {
               if ((state->is_version(map[i].required_glsl,
                                      map[i].required_essl) ||
                    (state->NV_image_formats_enable &&
                     map[i].nv_image_formats)) &&
                   match_layout_qualifier($1, map[i].name, state) == 0) {
                  /* Skip ARB_shader_image_load_store qualifiers if not enabled */
                  if (!map[i].ext_qualifiers && !(state->ARB_shader_image_load_store_enable ||
                                                  state->is_version(420, 310))) {
                     continue;
                  }
                  /* Skip EXT_shader_image_load_store qualifiers if not enabled */
                  if (map[i].ext_qualifiers && !state->EXT_shader_image_load_store_enable) {
                     continue;
                  }
                  $$.flags.q.explicit_image_format = 1;
                  $$.image_format = map[i].format;
                  $$.image_base_type = map[i].base_type;
                  break;
               }
            }
         }
      }

      if (!$$.flags.i) {
         if (match_layout_qualifier($1, "early_fragment_tests", state) == 0) {
            /* From section 4.4.1.3 of the GLSL 4.50 specification
             * (Fragment Shader Inputs):
             *
             *  "Fragment shaders also allow the following layout
             *   qualifier on in only (not with variable declarations)
             *     layout-qualifier-id
             *        early_fragment_tests
             *   [...]"
             */
            if (state->stage != MESA_SHADER_FRAGMENT) {
               _mesa_glsl_error(& @1, state,
                                "early_fragment_tests layout qualifier only "
                                "valid in fragment shaders");
            }

            $$.flags.q.early_fragment_tests = 1;
         }

         if (match_layout_qualifier($1, "inner_coverage", state) == 0) {
            if (state->stage != MESA_SHADER_FRAGMENT) {
               _mesa_glsl_error(& @1, state,
                                "inner_coverage layout qualifier only "
                                "valid in fragment shaders");
            }

	    if (state->INTEL_conservative_rasterization_enable) {
	       $$.flags.q.inner_coverage = 1;
	    } else {
	       _mesa_glsl_error(& @1, state,
                                "inner_coverage layout qualifier present, "
                                "but the INTEL_conservative_rasterization extension "
                                "is not enabled.");
            }
         }

         if (match_layout_qualifier($1, "post_depth_coverage", state) == 0) {
            if (state->stage != MESA_SHADER_FRAGMENT) {
               _mesa_glsl_error(& @1, state,
                                "post_depth_coverage layout qualifier only "
                                "valid in fragment shaders");
            }

            if (state->ARB_post_depth_coverage_enable ||
		state->INTEL_conservative_rasterization_enable) {
               $$.flags.q.post_depth_coverage = 1;
            } else {
               _mesa_glsl_error(& @1, state,
                                "post_depth_coverage layout qualifier present, "
                                "but the GL_ARB_post_depth_coverage extension "
                                "is not enabled.");
            }
         }

         if ($$.flags.q.post_depth_coverage && $$.flags.q.inner_coverage) {
            _mesa_glsl_error(& @1, state,
                             "post_depth_coverage & inner_coverage layout qualifiers "
                             "are mutually exclusive");
         }
      }

      const bool pixel_interlock_ordered = match_layout_qualifier($1,
         "pixel_interlock_ordered", state) == 0;
      const bool pixel_interlock_unordered = match_layout_qualifier($1,
         "pixel_interlock_unordered", state) == 0;
      const bool sample_interlock_ordered = match_layout_qualifier($1,
         "sample_interlock_ordered", state) == 0;
      const bool sample_interlock_unordered = match_layout_qualifier($1,
         "sample_interlock_unordered", state) == 0;

      if (pixel_interlock_ordered + pixel_interlock_unordered +
          sample_interlock_ordered + sample_interlock_unordered > 0 &&
          state->stage != MESA_SHADER_FRAGMENT) {
         _mesa_glsl_error(& @1, state, "interlock layout qualifiers: "
                          "pixel_interlock_ordered, pixel_interlock_unordered, "
                          "sample_interlock_ordered and sample_interlock_unordered, "
                          "only valid in fragment shader input layout declaration.");
      } else if (pixel_interlock_ordered + pixel_interlock_unordered +
                 sample_interlock_ordered + sample_interlock_unordered > 0 &&
                 !state->ARB_fragment_shader_interlock_enable &&
                 !state->NV_fragment_shader_interlock_enable) {
         _mesa_glsl_error(& @1, state,
                          "interlock layout qualifier present, but the "
                          "GL_ARB_fragment_shader_interlock or "
                          "GL_NV_fragment_shader_interlock extension is not "
                          "enabled.");
      } else {
         $$.flags.q.pixel_interlock_ordered = pixel_interlock_ordered;
         $$.flags.q.pixel_interlock_unordered = pixel_interlock_unordered;
         $$.flags.q.sample_interlock_ordered = sample_interlock_ordered;
         $$.flags.q.sample_interlock_unordered = sample_interlock_unordered;
      }

      /* Layout qualifiers for tessellation evaluation shaders. */
      if (!$$.flags.i) {
         static const struct {
            const char *s;
            GLenum e;
         } map[] = {
                 /* triangles already parsed by gs-specific code */
                 { "quads", GL_QUADS },
                 { "isolines", GL_ISOLINES },
         };
         for (unsigned i = 0; i < ARRAY_SIZE(map); i++) {
            if (match_layout_qualifier($1, map[i].s, state) == 0) {
               $$.flags.q.prim_type = 1;
               $$.prim_type = map[i].e;
               break;
            }
         }

         if ($$.flags.i && !state->has_tessellation_shader()) {
            _mesa_glsl_error(& @1, state,
                             "primitive mode qualifier `%s' requires "
                             "GLSL 4.00 or ARB_tessellation_shader", $1);
         }
      }
      if (!$$.flags.i) {
         static const struct {
            const char *s;
            enum gl_tess_spacing e;
         } map[] = {
                 { "equal_spacing", TESS_SPACING_EQUAL },
                 { "fractional_odd_spacing", TESS_SPACING_FRACTIONAL_ODD },
                 { "fractional_even_spacing", TESS_SPACING_FRACTIONAL_EVEN },
         };
         for (unsigned i = 0; i < ARRAY_SIZE(map); i++) {
            if (match_layout_qualifier($1, map[i].s, state) == 0) {
               $$.flags.q.vertex_spacing = 1;
               $$.vertex_spacing = map[i].e;
               break;
            }
         }

         if ($$.flags.i && !state->has_tessellation_shader()) {
            _mesa_glsl_error(& @1, state,
                             "vertex spacing qualifier `%s' requires "
                             "GLSL 4.00 or ARB_tessellation_shader", $1);
         }
      }
      if (!$$.flags.i) {
         if (match_layout_qualifier($1, "cw", state) == 0) {
            $$.flags.q.ordering = 1;
            $$.ordering = GL_CW;
         } else if (match_layout_qualifier($1, "ccw", state) == 0) {
            $$.flags.q.ordering = 1;
            $$.ordering = GL_CCW;
         }

         if ($$.flags.i && !state->has_tessellation_shader()) {
            _mesa_glsl_error(& @1, state,
                             "ordering qualifier `%s' requires "
                             "GLSL 4.00 or ARB_tessellation_shader", $1);
         }
      }
      if (!$$.flags.i) {
         if (match_layout_qualifier($1, "point_mode", state) == 0) {
            $$.flags.q.point_mode = 1;
            $$.point_mode = true;
         }

         if ($$.flags.i && !state->has_tessellation_shader()) {
            _mesa_glsl_error(& @1, state,
                             "qualifier `point_mode' requires "
                             "GLSL 4.00 or ARB_tessellation_shader");
         }
      }

      if (!$$.flags.i) {
         static const struct {
            const char *s;
            uint32_t mask;
         } map[] = {
                 { "blend_support_multiply",       BLEND_MULTIPLY },
                 { "blend_support_screen",         BLEND_SCREEN },
                 { "blend_support_overlay",        BLEND_OVERLAY },
                 { "blend_support_darken",         BLEND_DARKEN },
                 { "blend_support_lighten",        BLEND_LIGHTEN },
                 { "blend_support_colordodge",     BLEND_COLORDODGE },
                 { "blend_support_colorburn",      BLEND_COLORBURN },
                 { "blend_support_hardlight",      BLEND_HARDLIGHT },
                 { "blend_support_softlight",      BLEND_SOFTLIGHT },
                 { "blend_support_difference",     BLEND_DIFFERENCE },
                 { "blend_support_exclusion",      BLEND_EXCLUSION },
                 { "blend_support_hsl_hue",        BLEND_HSL_HUE },
                 { "blend_support_hsl_saturation", BLEND_HSL_SATURATION },
                 { "blend_support_hsl_color",      BLEND_HSL_COLOR },
                 { "blend_support_hsl_luminosity", BLEND_HSL_LUMINOSITY },
                 { "blend_support_all_equations",  BLEND_ALL },
         };
         for (unsigned i = 0; i < ARRAY_SIZE(map); i++) {
            if (match_layout_qualifier($1, map[i].s, state) == 0) {
               $$.flags.q.blend_support = 1;
               state->fs_blend_support |= map[i].mask;
               break;
            }
         }

         if ($$.flags.i &&
             !state->KHR_blend_equation_advanced_enable &&
             !state->is_version(0, 320)) {
            _mesa_glsl_error(& @1, state,
                             "advanced blending layout qualifiers require "
                             "ESSL 3.20 or KHR_blend_equation_advanced");
         }

         if ($$.flags.i && state->stage != MESA_SHADER_FRAGMENT) {
            _mesa_glsl_error(& @1, state,
                             "advanced blending layout qualifiers only "
                             "valid in fragment shaders");
         }
      }

      /* Layout qualifiers for ARB_compute_variable_group_size. */
      if (!$$.flags.i) {
         if (match_layout_qualifier($1, "local_size_variable", state) == 0) {
            $$.flags.q.local_size_variable = 1;
         }

         if ($$.flags.i && !state->ARB_compute_variable_group_size_enable) {
            _mesa_glsl_error(& @1, state,
                             "qualifier `local_size_variable` requires "
                             "ARB_compute_variable_group_size");
         }
      }

      /* Layout qualifiers for ARB_bindless_texture. */
      if (!$$.flags.i) {
         if (match_layout_qualifier($1, "bindless_sampler", state) == 0)
            $$.flags.q.bindless_sampler = 1;
         if (match_layout_qualifier($1, "bound_sampler", state) == 0)
            $$.flags.q.bound_sampler = 1;

         if (state->has_shader_image_load_store()) {
            if (match_layout_qualifier($1, "bindless_image", state) == 0)
               $$.flags.q.bindless_image = 1;
            if (match_layout_qualifier($1, "bound_image", state) == 0)
               $$.flags.q.bound_image = 1;
         }

         if ($$.flags.i && !state->has_bindless()) {
            _mesa_glsl_error(& @1, state,
                             "qualifier `%s` requires "
                             "ARB_bindless_texture", $1);
         }
      }

      if (!$$.flags.i &&
          state->EXT_shader_framebuffer_fetch_non_coherent_enable) {
         if (match_layout_qualifier($1, "noncoherent", state) == 0)
            $$.flags.q.non_coherent = 1;
      }

      // Layout qualifiers for NV_compute_shader_derivatives.
      if (!$$.flags.i) {
         if (match_layout_qualifier($1, "derivative_group_quadsNV", state) == 0) {
            $$.flags.q.derivative_group = 1;
            $$.derivative_group = DERIVATIVE_GROUP_QUADS;
         } else if (match_layout_qualifier($1, "derivative_group_linearNV", state) == 0) {
            $$.flags.q.derivative_group = 1;
            $$.derivative_group = DERIVATIVE_GROUP_LINEAR;
         }

         if ($$.flags.i) {
            if (!state->has_compute_shader()) {
               _mesa_glsl_error(& @1, state,
                                "qualifier `%s' requires "
                                "a compute shader", $1);
            }

            if (!state->NV_compute_shader_derivatives_enable) {
               _mesa_glsl_error(& @1, state,
                                "qualifier `%s' requires "
                                "NV_compute_shader_derivatives", $1);
            }

            if (state->NV_compute_shader_derivatives_warn) {
               _mesa_glsl_warning(& @1, state,
                                  "NV_compute_shader_derivatives layout "
                                  "qualifier `%s' used", $1);
            }
         }
      }

      /* Layout qualifier for NV_viewport_array2. */
      if (!$$.flags.i && state->stage != MESA_SHADER_FRAGMENT) {
         if (match_layout_qualifier($1, "viewport_relative", state) == 0) {
            $$.flags.q.viewport_relative = 1;
         }

         if ($$.flags.i && !state->NV_viewport_array2_enable) {
            _mesa_glsl_error(& @1, state,
                             "qualifier `%s' requires "
                             "GL_NV_viewport_array2", $1);
         }

         if ($$.flags.i && state->NV_viewport_array2_warn) {
            _mesa_glsl_warning(& @1, state,
                               "GL_NV_viewport_array2 layout "
                               "identifier `%s' used", $1);
         }
      }

      if (!$$.flags.i) {
         _mesa_glsl_error(& @1, state, "unrecognized layout identifier "
                          "`%s'", $1);
         YYERROR;
      }
   }
   | any_identifier '=' constant_expression
   {
      memset(& $$, 0, sizeof($$));
      void *ctx = state->linalloc;

      if ($3->oper != ast_int_constant &&
          $3->oper != ast_uint_constant &&
          !state->has_enhanced_layouts()) {
         _mesa_glsl_error(& @1, state,
                          "compile-time constant expressions require "
                          "GLSL 4.40 or ARB_enhanced_layouts");
      }

      if (match_layout_qualifier("align", $1, state) == 0) {
         if (!state->has_enhanced_layouts()) {
            _mesa_glsl_error(& @1, state,
                             "align qualifier requires "
                             "GLSL 4.40 or ARB_enhanced_layouts");
         } else {
            $$.flags.q.explicit_align = 1;
            $$.align = $3;
         }
      }

      if (match_layout_qualifier("location", $1, state) == 0) {
         $$.flags.q.explicit_location = 1;

         if ($$.flags.q.attribute == 1 &&
             state->ARB_explicit_attrib_location_warn) {
            _mesa_glsl_warning(& @1, state,
                               "GL_ARB_explicit_attrib_location layout "
                               "identifier `%s' used", $1);
         }
         $$.location = $3;
      }

      if (match_layout_qualifier("component", $1, state) == 0) {
         if (!state->has_enhanced_layouts()) {
            _mesa_glsl_error(& @1, state,
                             "component qualifier requires "
                             "GLSL 4.40 or ARB_enhanced_layouts");
         } else {
            $$.flags.q.explicit_component = 1;
            $$.component = $3;
         }
      }

      if (match_layout_qualifier("index", $1, state) == 0) {
         if (state->es_shader && !state->EXT_blend_func_extended_enable) {
            _mesa_glsl_error(& @3, state, "index layout qualifier requires EXT_blend_func_extended");
            YYERROR;
         }

         $$.flags.q.explicit_index = 1;
         $$.index = $3;
      }

      if ((state->has_420pack_or_es31() ||
           state->has_atomic_counters() ||
           state->has_shader_storage_buffer_objects()) &&
          match_layout_qualifier("binding", $1, state) == 0) {
         $$.flags.q.explicit_binding = 1;
         $$.binding = $3;
      }

      if ((state->has_atomic_counters() ||
           state->has_enhanced_layouts()) &&
          match_layout_qualifier("offset", $1, state) == 0) {
         $$.flags.q.explicit_offset = 1;
         $$.offset = $3;
      }

      if (match_layout_qualifier("max_vertices", $1, state) == 0) {
         $$.flags.q.max_vertices = 1;
         $$.max_vertices = new(ctx) ast_layout_expression(@1, $3);
         if (!state->has_geometry_shader()) {
            _mesa_glsl_error(& @3, state,
                             "#version 150 max_vertices qualifier "
                             "specified", $3);
         }
      }

      if (state->stage == MESA_SHADER_GEOMETRY) {
         if (match_layout_qualifier("stream", $1, state) == 0 &&
             state->check_explicit_attrib_stream_allowed(& @3)) {
            $$.flags.q.stream = 1;
            $$.flags.q.explicit_stream = 1;
            $$.stream = $3;
         }
      }

      if (state->has_enhanced_layouts()) {
         if (match_layout_qualifier("xfb_buffer", $1, state) == 0) {
            $$.flags.q.xfb_buffer = 1;
            $$.flags.q.explicit_xfb_buffer = 1;
            $$.xfb_buffer = $3;
         }

         if (match_layout_qualifier("xfb_offset", $1, state) == 0) {
            $$.flags.q.explicit_xfb_offset = 1;
            $$.offset = $3;
         }

         if (match_layout_qualifier("xfb_stride", $1, state) == 0) {
            $$.flags.q.xfb_stride = 1;
            $$.flags.q.explicit_xfb_stride = 1;
            $$.xfb_stride = $3;
         }
      }

      static const char * const local_size_qualifiers[3] = {
         "local_size_x",
         "local_size_y",
         "local_size_z",
      };
      for (int i = 0; i < 3; i++) {
         if (match_layout_qualifier(local_size_qualifiers[i], $1,
                                    state) == 0) {
            if (!state->has_compute_shader()) {
               _mesa_glsl_error(& @3, state,
                                "%s qualifier requires GLSL 4.30 or "
                                "GLSL ES 3.10 or ARB_compute_shader",
                                local_size_qualifiers[i]);
               YYERROR;
            } else {
               $$.flags.q.local_size |= (1 << i);
               $$.local_size[i] = new(ctx) ast_layout_expression(@1, $3);
            }
            break;
         }
      }

      if (match_layout_qualifier("invocations", $1, state) == 0) {
         $$.flags.q.invocations = 1;
         $$.invocations = new(ctx) ast_layout_expression(@1, $3);
         if (!state->is_version(400, 320) &&
             !state->ARB_gpu_shader5_enable &&
             !state->OES_geometry_shader_enable &&
             !state->EXT_geometry_shader_enable) {
            _mesa_glsl_error(& @3, state,
                             "GL_ARB_gpu_shader5 invocations "
                             "qualifier specified", $3);
         }
      }

      /* Layout qualifiers for tessellation control shaders. */
      if (match_layout_qualifier("vertices", $1, state) == 0) {
         $$.flags.q.vertices = 1;
         $$.vertices = new(ctx) ast_layout_expression(@1, $3);
         if (!state->has_tessellation_shader()) {
            _mesa_glsl_error(& @1, state,
                             "vertices qualifier requires GLSL 4.00 or "
                             "ARB_tessellation_shader");
         }
      }

      /* If the identifier didn't match any known layout identifiers,
       * emit an error.
       */
      if (!$$.flags.i) {
         _mesa_glsl_error(& @1, state, "unrecognized layout identifier "
                          "`%s'", $1);
         YYERROR;
      }
   }
   | interface_block_layout_qualifier
   {
      $$ = $1;
      /* Layout qualifiers for ARB_uniform_buffer_object. */
      if ($$.flags.q.uniform && !state->has_uniform_buffer_objects()) {
         _mesa_glsl_error(& @1, state,
                          "#version 140 / GL_ARB_uniform_buffer_object "
                          "layout qualifier `%s' is used", $1);
      } else if ($$.flags.q.uniform && state->ARB_uniform_buffer_object_warn) {
         _mesa_glsl_warning(& @1, state,
                            "#version 140 / GL_ARB_uniform_buffer_object "
                            "layout qualifier `%s' is used", $1);
      }
   }
   ;

/* This is a separate language rule because we parse these as tokens
 * (due to them being reserved keywords) instead of identifiers like
 * most qualifiers.  See the any_identifier path of
 * layout_qualifier_id for the others.
 *
 * Note that since layout qualifiers are case-insensitive in desktop
 * GLSL, all of these qualifiers need to be handled as identifiers as
 * well (by the any_identifier path of layout_qualifier_id).
 */
interface_block_layout_qualifier:
   ROW_MAJOR
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.row_major = 1;
   }
   | PACKED_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.packed = 1;
   }
   | SHARED
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.shared = 1;
   }
   ;

subroutine_qualifier:
   SUBROUTINE
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.subroutine = 1;
   }
   | SUBROUTINE '(' subroutine_type_list ')'
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.subroutine = 1;
      $$.subroutine_list = $3;
   }
   ;

subroutine_type_list:
   any_identifier
   {
        void *ctx = state->linalloc;
        ast_declaration *decl = new(ctx)  ast_declaration($1, NULL, NULL);
        decl->set_location(@1);

        $$ = new(ctx) ast_subroutine_list();
        $$->declarations.push_tail(&decl->link);
   }
   | subroutine_type_list ',' any_identifier
   {
        void *ctx = state->linalloc;
        ast_declaration *decl = new(ctx)  ast_declaration($3, NULL, NULL);
        decl->set_location(@3);

        $$ = $1;
        $$->declarations.push_tail(&decl->link);
   }
   ;

interpolation_qualifier:
   SMOOTH
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.smooth = 1;
   }
   | FLAT
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.flat = 1;
   }
   | NOPERSPECTIVE
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.noperspective = 1;
   }
   ;

type_qualifier:
   /* Single qualifiers */
   INVARIANT
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.invariant = 1;
   }
   | PRECISE
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.precise = 1;
   }
   | auxiliary_storage_qualifier
   | storage_qualifier
   | interpolation_qualifier
   | layout_qualifier
   | memory_qualifier
   | subroutine_qualifier
   | precision_qualifier
   {
      memset(&$$, 0, sizeof($$));
      $$.precision = $1;
   }

   /* Multiple qualifiers:
    * In GLSL 4.20, these can be specified in any order.  In earlier versions,
    * they appear in this order (see GLSL 1.50 section 4.7 & comments below):
    *
    *    invariant interpolation auxiliary storage precision  ...or...
    *    layout storage precision
    *
    * Each qualifier's rule ensures that the accumulated qualifiers on the right
    * side don't contain any that must appear on the left hand side.
    * For example, when processing a storage qualifier, we check that there are
    * no auxiliary, interpolation, layout, invariant, or precise qualifiers to the right.
    */
   | PRECISE type_qualifier
   {
      if ($2.flags.q.precise)
         _mesa_glsl_error(&@1, state, "duplicate \"precise\" qualifier");

      $$ = $2;
      $$.flags.q.precise = 1;
   }
   | INVARIANT type_qualifier
   {
      if ($2.flags.q.invariant)
         _mesa_glsl_error(&@1, state, "duplicate \"invariant\" qualifier");

      if (!state->has_420pack_or_es31() && $2.flags.q.precise)
         _mesa_glsl_error(&@1, state,
                          "\"invariant\" must come after \"precise\"");

      $$ = $2;
      $$.flags.q.invariant = 1;

      /* GLSL ES 3.00 spec, section 4.6.1 "The Invariant Qualifier":
       *
       * "Only variables output from a shader can be candidates for invariance.
       * This includes user-defined output variables and the built-in output
       * variables. As only outputs can be declared as invariant, an invariant
       * output from one shader stage will still match an input of a subsequent
       * stage without the input being declared as invariant."
       *
       * On the desktop side, this text first appears in GLSL 4.30.
       */
      if (state->is_version(430, 300) && $$.flags.q.in)
         _mesa_glsl_error(&@1, state, "invariant qualifiers cannot be used with shader inputs");
   }
   | interpolation_qualifier type_qualifier
   {
      /* Section 4.3 of the GLSL 1.40 specification states:
       * "...qualified with one of these interpolation qualifiers"
       *
       * GLSL 1.30 claims to allow "one or more", but insists that:
       * "These interpolation qualifiers may only precede the qualifiers in,
       *  centroid in, out, or centroid out in a declaration."
       *
       * ...which means that e.g. smooth can't precede smooth, so there can be
       * only one after all, and the 1.40 text is a clarification, not a change.
       */
      if ($2.has_interpolation())
         _mesa_glsl_error(&@1, state, "duplicate interpolation qualifier");

      if (!state->has_420pack_or_es31() &&
          ($2.flags.q.precise || $2.flags.q.invariant)) {
         _mesa_glsl_error(&@1, state, "interpolation qualifiers must come "
                          "after \"precise\" or \"invariant\"");
      }

      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }
   | layout_qualifier type_qualifier
   {
      /* In the absence of ARB_shading_language_420pack, layout qualifiers may
       * appear no later than auxiliary storage qualifiers. There is no
       * particularly clear spec language mandating this, but in all examples
       * the layout qualifier precedes the storage qualifier.
       *
       * We allow combinations of layout with interpolation, invariant or
       * precise qualifiers since these are useful in ARB_separate_shader_objects.
       * There is no clear spec guidance on this either.
       */
      $$ = $1;
      $$.merge_qualifier(& @1, state, $2, false, $2.has_layout());
   }
   | subroutine_qualifier type_qualifier
   {
      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }
   | auxiliary_storage_qualifier type_qualifier
   {
      if ($2.has_auxiliary_storage()) {
         _mesa_glsl_error(&@1, state,
                          "duplicate auxiliary storage qualifier (centroid or sample)");
      }

      if ((!state->has_420pack_or_es31() && !state->EXT_gpu_shader4_enable) &&
          ($2.flags.q.precise || $2.flags.q.invariant ||
           $2.has_interpolation() || $2.has_layout())) {
         _mesa_glsl_error(&@1, state, "auxiliary storage qualifiers must come "
                          "just before storage qualifiers");
      }
      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }
   | storage_qualifier type_qualifier
   {
      /* Section 4.3 of the GLSL 1.20 specification states:
       * "Variable declarations may have a storage qualifier specified..."
       *  1.30 clarifies this to "may have one storage qualifier".
       *
       * GL_EXT_gpu_shader4 allows "varying out" in fragment shaders.
       */
      if ($2.has_storage() &&
          (!state->EXT_gpu_shader4_enable ||
           state->stage != MESA_SHADER_FRAGMENT ||
           !$1.flags.q.varying || !$2.flags.q.out))
         _mesa_glsl_error(&@1, state, "duplicate storage qualifier");

      if (!state->has_420pack_or_es31() &&
          ($2.flags.q.precise || $2.flags.q.invariant || $2.has_interpolation() ||
           $2.has_layout() || $2.has_auxiliary_storage())) {
         _mesa_glsl_error(&@1, state, "storage qualifiers must come after "
                          "precise, invariant, interpolation, layout and auxiliary "
                          "storage qualifiers");
      }

      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }
   | precision_qualifier type_qualifier
   {
      if ($2.precision != ast_precision_none)
         _mesa_glsl_error(&@1, state, "duplicate precision qualifier");

      if (!(state->has_420pack_or_es31()) &&
          $2.flags.i != 0)
         _mesa_glsl_error(&@1, state, "precision qualifiers must come last");

      $$ = $2;
      $$.precision = $1;
   }
   | memory_qualifier type_qualifier
   {
      $$ = $1;
      $$.merge_qualifier(&@1, state, $2, false);
   }
   ;

auxiliary_storage_qualifier:
   CENTROID
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.centroid = 1;
   }
   | SAMPLE
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.sample = 1;
   }
   | PATCH
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.patch = 1;
   }

storage_qualifier:
   CONST_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.constant = 1;
   }
   | ATTRIBUTE
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.attribute = 1;
   }
   | VARYING
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.varying = 1;
   }
   | IN_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.in = 1;
   }
   | OUT_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.out = 1;

      if (state->stage == MESA_SHADER_GEOMETRY &&
          state->has_explicit_attrib_stream()) {
         /* Section 4.3.8.2 (Output Layout Qualifiers) of the GLSL 4.00
          * spec says:
          *
          *     "If the block or variable is declared with the stream
          *     identifier, it is associated with the specified stream;
          *     otherwise, it is associated with the current default stream."
          */
          $$.flags.q.stream = 1;
          $$.flags.q.explicit_stream = 0;
          $$.stream = state->out_qualifier->stream;
      }

      if (state->has_enhanced_layouts()) {
          $$.flags.q.xfb_buffer = 1;
          $$.flags.q.explicit_xfb_buffer = 0;
          $$.xfb_buffer = state->out_qualifier->xfb_buffer;
      }
   }
   | INOUT_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.in = 1;
      $$.flags.q.out = 1;

      if (!state->has_framebuffer_fetch() ||
          !state->is_version(130, 300) ||
          state->stage != MESA_SHADER_FRAGMENT)
         _mesa_glsl_error(&@1, state, "A single interface variable cannot be "
                          "declared as both input and output");
   }
   | UNIFORM
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.uniform = 1;
   }
   | BUFFER
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.buffer = 1;
   }
   | SHARED
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.shared_storage = 1;
   }
   ;

memory_qualifier:
   COHERENT
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.coherent = 1;
   }
   | VOLATILE
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q._volatile = 1;
   }
   | RESTRICT
   {
      STATIC_ASSERT(sizeof($$.flags.q) <= sizeof($$.flags.i));
      memset(& $$, 0, sizeof($$));
      $$.flags.q.restrict_flag = 1;
   }
   | READONLY
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.read_only = 1;
   }
   | WRITEONLY
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.write_only = 1;
   }
   ;

array_specifier:
   '[' ']'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_array_specifier(@1, new(ctx) ast_expression(
                                                  ast_unsized_array_dim, NULL,
                                                  NULL, NULL));
      $$->set_location_range(@1, @2);
   }
   | '[' constant_expression ']'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_array_specifier(@1, $2);
      $$->set_location_range(@1, @3);
   }
   | array_specifier '[' ']'
   {
      void *ctx = state->linalloc;
      $$ = $1;

      if (state->check_arrays_of_arrays_allowed(& @1)) {
         $$->add_dimension(new(ctx) ast_expression(ast_unsized_array_dim, NULL,
                                                   NULL, NULL));
      }
   }
   | array_specifier '[' constant_expression ']'
   {
      $$ = $1;

      if (state->check_arrays_of_arrays_allowed(& @1)) {
         $$->add_dimension($3);
      }
   }
   ;

type_specifier:
   type_specifier_nonarray
   | type_specifier_nonarray array_specifier
   {
      $$ = $1;
      $$->array_specifier = $2;
   }
   ;

type_specifier_nonarray:
   basic_type_specifier_nonarray
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_type_specifier($1);
      $$->set_location(@1);
   }
   | struct_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_type_specifier($1);
      $$->set_location(@1);
   }
   | TYPE_IDENTIFIER
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_type_specifier($1);
      $$->set_location(@1);
   }
   ;

basic_type_specifier_nonarray:
   VOID_TOK                 { $$ = glsl_type::void_type; }
   | BASIC_TYPE_TOK         { $$ = $1; }
   | UNSIGNED BASIC_TYPE_TOK
   {
      if ($2 == glsl_type::int_type) {
         $$ = glsl_type::uint_type;
      } else {
         _mesa_glsl_error(&@1, state,
                          "\"unsigned\" is only allowed before \"int\"");
      }
   }
   ;

precision_qualifier:
   HIGHP
   {
      state->check_precision_qualifiers_allowed(&@1);
      $$ = ast_precision_high;
   }
   | MEDIUMP
   {
      state->check_precision_qualifiers_allowed(&@1);
      $$ = ast_precision_medium;
   }
   | LOWP
   {
      state->check_precision_qualifiers_allowed(&@1);
      $$ = ast_precision_low;
   }
   ;

struct_specifier:
   STRUCT any_identifier '{' struct_declaration_list '}'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_struct_specifier($2, $4);
      $$->set_location_range(@2, @5);
      state->symbols->add_type($2, glsl_type::void_type);
   }
   | STRUCT '{' struct_declaration_list '}'
   {
      void *ctx = state->linalloc;

      /* All anonymous structs have the same name. This simplifies matching of
       * globals whose type is an unnamed struct.
       *
       * It also avoids a memory leak when the same shader is compiled over and
       * over again.
       */
      $$ = new(ctx) ast_struct_specifier("#anon_struct", $3);

      $$->set_location_range(@2, @4);
   }
   ;

struct_declaration_list:
   struct_declaration
   {
      $$ = $1;
      $1->link.self_link();
   }
   | struct_declaration_list struct_declaration
   {
      $$ = $1;
      $$->link.insert_before(& $2->link);
   }
   ;

struct_declaration:
   fully_specified_type struct_declarator_list ';'
   {
      void *ctx = state->linalloc;
      ast_fully_specified_type *const type = $1;
      type->set_location(@1);

      if (state->has_bindless()) {
         ast_type_qualifier input_layout_mask;

         /* Allow to declare qualifiers for images. */
         input_layout_mask.flags.i = 0;
         input_layout_mask.flags.q.coherent = 1;
         input_layout_mask.flags.q._volatile = 1;
         input_layout_mask.flags.q.restrict_flag = 1;
         input_layout_mask.flags.q.read_only = 1;
         input_layout_mask.flags.q.write_only = 1;
         input_layout_mask.flags.q.explicit_image_format = 1;

         if ((type->qualifier.flags.i & ~input_layout_mask.flags.i) != 0) {
            _mesa_glsl_error(&@1, state,
                             "only precision and image qualifiers may be "
                             "applied to structure members");
         }
      } else {
         if (type->qualifier.flags.i != 0)
            _mesa_glsl_error(&@1, state,
                             "only precision qualifiers may be applied to "
                             "structure members");
      }

      $$ = new(ctx) ast_declarator_list(type);
      $$->set_location(@2);

      $$->declarations.push_degenerate_list_at_head(& $2->link);
   }
   ;

struct_declarator_list:
   struct_declarator
   {
      $$ = $1;
      $1->link.self_link();
   }
   | struct_declarator_list ',' struct_declarator
   {
      $$ = $1;
      $$->link.insert_before(& $3->link);
   }
   ;

struct_declarator:
   any_identifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_declaration($1, NULL, NULL);
      $$->set_location(@1);
   }
   | any_identifier array_specifier
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_declaration($1, $2, NULL);
      $$->set_location_range(@1, @2);
   }
   ;

initializer:
   assignment_expression
   | '{' initializer_list '}'
   {
      $$ = $2;
   }
   | '{' initializer_list ',' '}'
   {
      $$ = $2;
   }
   ;

initializer_list:
   initializer
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_aggregate_initializer();
      $$->set_location(@1);
      $$->expressions.push_tail(& $1->link);
   }
   | initializer_list ',' initializer
   {
      $1->expressions.push_tail(& $3->link);
   }
   ;

declaration_statement:
   declaration
   ;

   // Grammar Note: labeled statements for SWITCH only; 'goto' is not
   // supported.
statement:
   compound_statement        { $$ = (ast_node *) $1; }
   | simple_statement
   ;

simple_statement:
   declaration_statement
   | expression_statement
   | selection_statement
   | switch_statement
   | iteration_statement
   | jump_statement
   | demote_statement
   ;

compound_statement:
   '{' '}'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_compound_statement(true, NULL);
      $$->set_location_range(@1, @2);
   }
   | '{'
   {
      state->symbols->push_scope();
   }
   statement_list '}'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_compound_statement(true, $3);
      $$->set_location_range(@1, @4);
      state->symbols->pop_scope();
   }
   ;

statement_no_new_scope:
   compound_statement_no_new_scope { $$ = (ast_node *) $1; }
   | simple_statement
   ;

compound_statement_no_new_scope:
   '{' '}'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_compound_statement(false, NULL);
      $$->set_location_range(@1, @2);
   }
   | '{' statement_list '}'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_compound_statement(false, $2);
      $$->set_location_range(@1, @3);
   }
   ;

statement_list:
   statement
   {
      if ($1 == NULL) {
         _mesa_glsl_error(& @1, state, "<nil> statement");
         assert($1 != NULL);
      }

      $$ = $1;
      $$->link.self_link();
   }
   | statement_list statement
   {
      if ($2 == NULL) {
         _mesa_glsl_error(& @2, state, "<nil> statement");
         assert($2 != NULL);
      }
      $$ = $1;
      $$->link.insert_before(& $2->link);
   }
   | statement_list extension_statement
   {
      if (!state->allow_extension_directive_midshader) {
         _mesa_glsl_error(& @1, state,
                          "#extension directive is not allowed "
                          "in the middle of a shader");
         YYERROR;
      }
   }
   ;

expression_statement:
   ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_statement(NULL);
      $$->set_location(@1);
   }
   | expression ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_expression_statement($1);
      $$->set_location(@1);
   }
   ;

selection_statement:
   IF '(' expression ')' selection_rest_statement
   {
      $$ = new(state->linalloc) ast_selection_statement($3, $5.then_statement,
                                                        $5.else_statement);
      $$->set_location_range(@1, @5);
   }
   ;

selection_rest_statement:
   statement ELSE statement
   {
      $$.then_statement = $1;
      $$.else_statement = $3;
   }
   | statement %prec THEN
   {
      $$.then_statement = $1;
      $$.else_statement = NULL;
   }
   ;

condition:
   expression
   {
      $$ = (ast_node *) $1;
   }
   | fully_specified_type any_identifier '=' initializer
   {
      void *ctx = state->linalloc;
      ast_declaration *decl = new(ctx) ast_declaration($2, NULL, $4);
      ast_declarator_list *declarator = new(ctx) ast_declarator_list($1);
      decl->set_location_range(@2, @4);
      declarator->set_location(@1);

      declarator->declarations.push_tail(&decl->link);
      $$ = declarator;
   }
   ;

/*
 * switch_statement grammar is based on the syntax described in the body
 * of the GLSL spec, not in it's appendix!!!
 */
switch_statement:
   SWITCH '(' expression ')' switch_body
   {
      $$ = new(state->linalloc) ast_switch_statement($3, $5);
      $$->set_location_range(@1, @5);
   }
   ;

switch_body:
   '{' '}'
   {
      $$ = new(state->linalloc) ast_switch_body(NULL);
      $$->set_location_range(@1, @2);
   }
   | '{' case_statement_list '}'
   {
      $$ = new(state->linalloc) ast_switch_body($2);
      $$->set_location_range(@1, @3);
   }
   ;

case_label:
   CASE expression ':'
   {
      $$ = new(state->linalloc) ast_case_label($2);
      $$->set_location(@2);
   }
   | DEFAULT ':'
   {
      $$ = new(state->linalloc) ast_case_label(NULL);
      $$->set_location(@2);
   }
   ;

case_label_list:
   case_label
   {
      ast_case_label_list *labels = new(state->linalloc) ast_case_label_list();

      labels->labels.push_tail(& $1->link);
      $$ = labels;
      $$->set_location(@1);
   }
   | case_label_list case_label
   {
      $$ = $1;
      $$->labels.push_tail(& $2->link);
   }
   ;

case_statement:
   case_label_list statement
   {
      ast_case_statement *stmts = new(state->linalloc) ast_case_statement($1);
      stmts->set_location(@2);

      stmts->stmts.push_tail(& $2->link);
      $$ = stmts;
   }
   | case_statement statement
   {
      $$ = $1;
      $$->stmts.push_tail(& $2->link);
   }
   ;

case_statement_list:
   case_statement
   {
      ast_case_statement_list *cases= new(state->linalloc) ast_case_statement_list();
      cases->set_location(@1);

      cases->cases.push_tail(& $1->link);
      $$ = cases;
   }
   | case_statement_list case_statement
   {
      $$ = $1;
      $$->cases.push_tail(& $2->link);
   }
   ;

iteration_statement:
   WHILE '(' condition ')' statement_no_new_scope
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_iteration_statement(ast_iteration_statement::ast_while,
                                            NULL, $3, NULL, $5);
      $$->set_location_range(@1, @4);
   }
   | DO statement WHILE '(' expression ')' ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_iteration_statement(ast_iteration_statement::ast_do_while,
                                            NULL, $5, NULL, $2);
      $$->set_location_range(@1, @6);
   }
   | FOR '(' for_init_statement for_rest_statement ')' statement_no_new_scope
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_iteration_statement(ast_iteration_statement::ast_for,
                                            $3, $4.cond, $4.rest, $6);
      $$->set_location_range(@1, @6);
   }
   ;

for_init_statement:
   expression_statement
   | declaration_statement
   ;

conditionopt:
   condition
   | /* empty */
   {
      $$ = NULL;
   }
   ;

for_rest_statement:
   conditionopt ';'
   {
      $$.cond = $1;
      $$.rest = NULL;
   }
   | conditionopt ';' expression
   {
      $$.cond = $1;
      $$.rest = $3;
   }
   ;

   // Grammar Note: No 'goto'. Gotos are not supported.
jump_statement:
   CONTINUE ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_jump_statement(ast_jump_statement::ast_continue, NULL);
      $$->set_location(@1);
   }
   | BREAK ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_jump_statement(ast_jump_statement::ast_break, NULL);
      $$->set_location(@1);
   }
   | RETURN ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_jump_statement(ast_jump_statement::ast_return, NULL);
      $$->set_location(@1);
   }
   | RETURN expression ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_jump_statement(ast_jump_statement::ast_return, $2);
      $$->set_location_range(@1, @2);
   }
   | DISCARD ';' // Fragment shader only.
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_jump_statement(ast_jump_statement::ast_discard, NULL);
      $$->set_location(@1);
   }
   ;

demote_statement:
   DEMOTE ';'
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_demote_statement();
      $$->set_location(@1);
   }
   ;

external_declaration:
   function_definition      { $$ = $1; }
   | declaration            { $$ = $1; }
   | pragma_statement       { $$ = $1; }
   | layout_defaults        { $$ = $1; }
   | ';'                    { $$ = NULL; }
   ;

function_definition:
   function_prototype compound_statement_no_new_scope
   {
      void *ctx = state->linalloc;
      $$ = new(ctx) ast_function_definition();
      $$->set_location_range(@1, @2);
      $$->prototype = $1;
      $$->body = $2;

      state->symbols->pop_scope();
   }
   ;

/* layout_qualifieropt is packed into this rule */
interface_block:
   basic_interface_block
   {
      $$ = $1;
   }
   | layout_qualifier interface_block
   {
      ast_interface_block *block = (ast_interface_block *) $2;

      if (!$1.merge_qualifier(& @1, state, block->layout, false,
                              block->layout.has_layout())) {
         YYERROR;
      }

      block->layout = $1;

      $$ = block;
   }
   | memory_qualifier interface_block
   {
      ast_interface_block *block = (ast_interface_block *)$2;

      if (!block->default_layout.flags.q.buffer) {
            _mesa_glsl_error(& @1, state,
                             "memory qualifiers can only be used in the "
                             "declaration of shader storage blocks");
      }
      if (!$1.merge_qualifier(& @1, state, block->layout, false)) {
         YYERROR;
      }
      block->layout = $1;
      $$ = block;
   }
   ;

basic_interface_block:
   interface_qualifier NEW_IDENTIFIER '{' member_list '}' instance_name_opt ';'
   {
      ast_interface_block *const block = $6;

      if ($1.flags.q.uniform) {
         block->default_layout = *state->default_uniform_qualifier;
      } else if ($1.flags.q.buffer) {
         block->default_layout = *state->default_shader_storage_qualifier;
      }
      block->block_name = $2;
      block->declarations.push_degenerate_list_at_head(& $4->link);

      _mesa_ast_process_interface_block(& @1, state, block, $1);

      $$ = block;
   }
   ;

interface_qualifier:
   IN_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.in = 1;
   }
   | OUT_TOK
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.out = 1;
   }
   | UNIFORM
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.uniform = 1;
   }
   | BUFFER
   {
      memset(& $$, 0, sizeof($$));
      $$.flags.q.buffer = 1;
   }
   | auxiliary_storage_qualifier interface_qualifier
   {
      if (!$1.flags.q.patch) {
         _mesa_glsl_error(&@1, state, "invalid interface qualifier");
      }
      if ($2.has_auxiliary_storage()) {
         _mesa_glsl_error(&@1, state, "duplicate patch qualifier");
      }
      $$ = $2;
      $$.flags.q.patch = 1;
   }
   ;

instance_name_opt:
   /* empty */
   {
      $$ = new(state->linalloc) ast_interface_block(NULL, NULL);
   }
   | NEW_IDENTIFIER
   {
      $$ = new(state->linalloc) ast_interface_block($1, NULL);
      $$->set_location(@1);
   }
   | NEW_IDENTIFIER array_specifier
   {
      $$ = new(state->linalloc) ast_interface_block($1, $2);
      $$->set_location_range(@1, @2);
   }
   ;

member_list:
   member_declaration
   {
      $$ = $1;
      $1->link.self_link();
   }
   | member_declaration member_list
   {
      $$ = $1;
      $2->link.insert_before(& $$->link);
   }
   ;

member_declaration:
   fully_specified_type struct_declarator_list ';'
   {
      void *ctx = state->linalloc;
      ast_fully_specified_type *type = $1;
      type->set_location(@1);

      if (type->qualifier.flags.q.attribute) {
         _mesa_glsl_error(& @1, state,
                          "keyword 'attribute' cannot be used with "
                          "interface block member");
      } else if (type->qualifier.flags.q.varying) {
         _mesa_glsl_error(& @1, state,
                          "keyword 'varying' cannot be used with "
                          "interface block member");
      }

      $$ = new(ctx) ast_declarator_list(type);
      $$->set_location(@2);

      $$->declarations.push_degenerate_list_at_head(& $2->link);
   }
   ;

layout_uniform_defaults:
   layout_qualifier layout_uniform_defaults
   {
      $$ = $1;
      if (!$$.merge_qualifier(& @1, state, $2, false, true)) {
         YYERROR;
      }
   }
   | layout_qualifier UNIFORM ';'
   ;

layout_buffer_defaults:
   layout_qualifier layout_buffer_defaults
   {
      $$ = $1;
      if (!$$.merge_qualifier(& @1, state, $2, false, true)) {
         YYERROR;
      }
   }
   | layout_qualifier BUFFER ';'
   ;

layout_in_defaults:
   layout_qualifier layout_in_defaults
   {
      $$ = $1;
      if (!$$.merge_qualifier(& @1, state, $2, false, true)) {
         YYERROR;
      }
      if (!$$.validate_in_qualifier(& @1, state)) {
         YYERROR;
      }
   }
   | layout_qualifier IN_TOK ';'
   {
      if (!$1.validate_in_qualifier(& @1, state)) {
         YYERROR;
      }
   }
   ;

layout_out_defaults:
   layout_qualifier layout_out_defaults
   {
      $$ = $1;
      if (!$$.merge_qualifier(& @1, state, $2, false, true)) {
         YYERROR;
      }
      if (!$$.validate_out_qualifier(& @1, state)) {
         YYERROR;
      }
   }
   | layout_qualifier OUT_TOK ';'
   {
      if (!$1.validate_out_qualifier(& @1, state)) {
         YYERROR;
      }
   }
   ;

layout_defaults:
   layout_uniform_defaults
   {
      $$ = NULL;
      if (!state->default_uniform_qualifier->
             merge_qualifier(& @1, state, $1, false)) {
         YYERROR;
      }
      if (!state->default_uniform_qualifier->
             push_to_global(& @1, state)) {
         YYERROR;
      }
   }
   | layout_buffer_defaults
   {
      $$ = NULL;
      if (!state->default_shader_storage_qualifier->
             merge_qualifier(& @1, state, $1, false)) {
         YYERROR;
      }
      if (!state->default_shader_storage_qualifier->
             push_to_global(& @1, state)) {
         YYERROR;
      }

      /* From the GLSL 4.50 spec, section 4.4.5:
       *
       *     "It is a compile-time error to specify the binding identifier for
       *     the global scope or for block member declarations."
       */
      if (state->default_shader_storage_qualifier->flags.q.explicit_binding) {
         _mesa_glsl_error(& @1, state,
                          "binding qualifier cannot be set for default layout");
      }
   }
   | layout_in_defaults
   {
      $$ = NULL;
      if (!$1.merge_into_in_qualifier(& @1, state, $$)) {
         YYERROR;
      }
      if (!state->in_qualifier->push_to_global(& @1, state)) {
         YYERROR;
      }
   }
   | layout_out_defaults
   {
      $$ = NULL;
      if (!$1.merge_into_out_qualifier(& @1, state, $$)) {
         YYERROR;
      }
      if (!state->out_qualifier->push_to_global(& @1, state)) {
         YYERROR;
      }
   }
   ;
