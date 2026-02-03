/* A Bison parser, made by GNU Bison 3.5.  */

/* Bison interface for Yacc-like parsers in C

   Copyright (C) 1984, 1989-1990, 2000-2015, 2018-2019 Free Software Foundation,
   Inc.

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.

   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.  */

/* As a special exception, you may create a larger work that contains
   part or all of the Bison parser skeleton and distribute that work
   under terms of your choice, so long as that work isn't itself a
   parser generator using the skeleton or a modified version thereof
   as a parser skeleton.  Alternatively, if you modify or redistribute
   the parser skeleton itself, you may (at your option) remove this
   special exception, which will cause the skeleton and the resulting
   Bison output files to be licensed under the GNU General Public
   License without this special exception.

   This special exception was added by the Free Software Foundation in
   version 2.2 of Bison.  */

/* Undocumented macros, especially those whose name start with YY_,
   are private implementation details.  Do not rely on them.  */

#ifndef YY__MESA_GLSL_SRC_COMPILER_GLSL_GLSL_PARSER_H_INCLUDED
# define YY__MESA_GLSL_SRC_COMPILER_GLSL_GLSL_PARSER_H_INCLUDED
/* Debug traces.  */
#ifndef YYDEBUG
# define YYDEBUG 0
#endif
#if YYDEBUG
extern int _mesa_glsl_debug;
#endif

/* Token type.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
  enum yytokentype
  {
    ATTRIBUTE = 258,
    CONST_TOK = 259,
    BASIC_TYPE_TOK = 260,
    BREAK = 261,
    BUFFER = 262,
    CONTINUE = 263,
    DO = 264,
    ELSE = 265,
    FOR = 266,
    IF = 267,
    DEMOTE = 268,
    DISCARD = 269,
    RETURN = 270,
    SWITCH = 271,
    CASE = 272,
    DEFAULT = 273,
    CENTROID = 274,
    IN_TOK = 275,
    OUT_TOK = 276,
    INOUT_TOK = 277,
    UNIFORM = 278,
    VARYING = 279,
    SAMPLE = 280,
    NOPERSPECTIVE = 281,
    FLAT = 282,
    SMOOTH = 283,
    IMAGE1DSHADOW = 284,
    IMAGE2DSHADOW = 285,
    IMAGE1DARRAYSHADOW = 286,
    IMAGE2DARRAYSHADOW = 287,
    COHERENT = 288,
    VOLATILE = 289,
    RESTRICT = 290,
    READONLY = 291,
    WRITEONLY = 292,
    SHARED = 293,
    STRUCT = 294,
    VOID_TOK = 295,
    WHILE = 296,
    IDENTIFIER = 297,
    TYPE_IDENTIFIER = 298,
    NEW_IDENTIFIER = 299,
    FLOATCONSTANT = 300,
    DOUBLECONSTANT = 301,
    INTCONSTANT = 302,
    UINTCONSTANT = 303,
    BOOLCONSTANT = 304,
    INT64CONSTANT = 305,
    UINT64CONSTANT = 306,
    FIELD_SELECTION = 307,
    LEFT_OP = 308,
    RIGHT_OP = 309,
    INC_OP = 310,
    DEC_OP = 311,
    LE_OP = 312,
    GE_OP = 313,
    EQ_OP = 314,
    NE_OP = 315,
    AND_OP = 316,
    OR_OP = 317,
    XOR_OP = 318,
    MUL_ASSIGN = 319,
    DIV_ASSIGN = 320,
    ADD_ASSIGN = 321,
    MOD_ASSIGN = 322,
    LEFT_ASSIGN = 323,
    RIGHT_ASSIGN = 324,
    AND_ASSIGN = 325,
    XOR_ASSIGN = 326,
    OR_ASSIGN = 327,
    SUB_ASSIGN = 328,
    INVARIANT = 329,
    PRECISE = 330,
    LOWP = 331,
    MEDIUMP = 332,
    HIGHP = 333,
    SUPERP = 334,
    PRECISION = 335,
    VERSION_TOK = 336,
    EXTENSION = 337,
    LINE = 338,
    COLON = 339,
    EOL = 340,
    INTERFACE = 341,
    OUTPUT = 342,
    PRAGMA_DEBUG_ON = 343,
    PRAGMA_DEBUG_OFF = 344,
    PRAGMA_OPTIMIZE_ON = 345,
    PRAGMA_OPTIMIZE_OFF = 346,
    PRAGMA_WARNING_ON = 347,
    PRAGMA_WARNING_OFF = 348,
    PRAGMA_INVARIANT_ALL = 349,
    LAYOUT_TOK = 350,
    DOT_TOK = 351,
    ASM = 352,
    CLASS = 353,
    UNION = 354,
    ENUM = 355,
    TYPEDEF = 356,
    TEMPLATE = 357,
    THIS = 358,
    PACKED_TOK = 359,
    GOTO = 360,
    INLINE_TOK = 361,
    NOINLINE = 362,
    PUBLIC_TOK = 363,
    STATIC = 364,
    EXTERN = 365,
    EXTERNAL = 366,
    LONG_TOK = 367,
    SHORT_TOK = 368,
    HALF = 369,
    FIXED_TOK = 370,
    UNSIGNED = 371,
    INPUT_TOK = 372,
    HVEC2 = 373,
    HVEC3 = 374,
    HVEC4 = 375,
    FVEC2 = 376,
    FVEC3 = 377,
    FVEC4 = 378,
    SAMPLER3DRECT = 379,
    SIZEOF = 380,
    CAST = 381,
    NAMESPACE = 382,
    USING = 383,
    RESOURCE = 384,
    PATCH = 385,
    SUBROUTINE = 386,
    ERROR_TOK = 387,
    COMMON = 388,
    PARTITION = 389,
    ACTIVE = 390,
    FILTER = 391,
    ROW_MAJOR = 392,
    THEN = 393
  };
#endif

/* Value type.  */
#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
union YYSTYPE
{
#line 101 "src/compiler/glsl/glsl_parser.yy"

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

#line 237 "src/compiler/glsl/glsl_parser.h"

};
typedef union YYSTYPE YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define YYSTYPE_IS_DECLARED 1
#endif

/* Location type.  */
#if ! defined YYLTYPE && ! defined YYLTYPE_IS_DECLARED
typedef struct YYLTYPE YYLTYPE;
struct YYLTYPE
{
  int first_line;
  int first_column;
  int last_line;
  int last_column;
};
# define YYLTYPE_IS_DECLARED 1
# define YYLTYPE_IS_TRIVIAL 1
#endif



int _mesa_glsl_parse (struct _mesa_glsl_parse_state *state);

#endif /* !YY__MESA_GLSL_SRC_COMPILER_GLSL_GLSL_PARSER_H_INCLUDED  */
