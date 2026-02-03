/* A Bison parser, made by GNU Bison 3.5.  */

/* Bison implementation for Yacc-like parsers in C

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

/* C LALR(1) parser skeleton written by Richard Stallman, by
   simplifying the original so-called "semantic" parser.  */

/* All symbols defined below should begin with yy or YY, to avoid
   infringing on user name space.  This should be done even for local
   variables, as they might otherwise be expanded by user macros.
   There are some unavoidable exceptions within include files to
   define necessary library symbols; they are noted "INFRINGES ON
   USER NAME SPACE" below.  */

/* Undocumented macros, especially those whose name start with YY_,
   are private implementation details.  Do not rely on them.  */

/* Identify Bison output.  */
#define YYBISON 1

/* Bison version.  */
#define YYBISON_VERSION "3.5"

/* Skeleton name.  */
#define YYSKELETON_NAME "yacc.c"

/* Pure parsers.  */
#define YYPURE 1

/* Push parsers.  */
#define YYPUSH 0

/* Pull parsers.  */
#define YYPULL 1


/* Substitute the variable and function names.  */
#define yyparse         glcpp_parser_parse
#define yylex           glcpp_parser_lex
#define yyerror         glcpp_parser_error
#define yydebug         glcpp_parser_debug
#define yynerrs         glcpp_parser_nerrs

/* First part of user prologue.  */
#line 1 "src/compiler/glsl/glcpp/glcpp-parse.y"

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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <inttypes.h>

#include "glcpp.h"
#include "main/mtypes.h"
#include "util/strndup.h"

const char *
_mesa_lookup_shader_include(struct gl_context *ctx, char *path,
                            bool error_check);

size_t
_mesa_get_shader_include_cursor(struct gl_shared_state *shared);

void
_mesa_set_shader_include_cursor(struct gl_shared_state *shared, size_t cursor);

static void
yyerror(YYLTYPE *locp, glcpp_parser_t *parser, const char *error);

static void
_define_object_macro(glcpp_parser_t *parser,
                     YYLTYPE *loc,
                     const char *macro,
                     token_list_t *replacements);

static void
_define_function_macro(glcpp_parser_t *parser,
                       YYLTYPE *loc,
                       const char *macro,
                       string_list_t *parameters,
                       token_list_t *replacements);

static string_list_t *
_string_list_create(glcpp_parser_t *parser);

static void
_string_list_append_item(glcpp_parser_t *parser, string_list_t *list,
                         const char *str);

static int
_string_list_contains(string_list_t *list, const char *member, int *index);

static const char *
_string_list_has_duplicate(string_list_t *list);

static int
_string_list_length(string_list_t *list);

static int
_string_list_equal(string_list_t *a, string_list_t *b);

static argument_list_t *
_argument_list_create(glcpp_parser_t *parser);

static void
_argument_list_append(glcpp_parser_t *parser, argument_list_t *list,
                      token_list_t *argument);

static int
_argument_list_length(argument_list_t *list);

static token_list_t *
_argument_list_member_at(argument_list_t *list, int index);

static token_t *
_token_create_str(glcpp_parser_t *parser, int type, char *str);

static token_t *
_token_create_ival(glcpp_parser_t *parser, int type, int ival);

static token_list_t *
_token_list_create(glcpp_parser_t *parser);

static void
_token_list_append(glcpp_parser_t *parser, token_list_t *list, token_t *token);

static void
_token_list_append_list(token_list_t *list, token_list_t *tail);

static int
_token_list_equal_ignoring_space(token_list_t *a, token_list_t *b);

static void
_parser_active_list_push(glcpp_parser_t *parser, const char *identifier,
                         token_node_t *marker);

static void
_parser_active_list_pop(glcpp_parser_t *parser);

static int
_parser_active_list_contains(glcpp_parser_t *parser, const char *identifier);

typedef enum {
   EXPANSION_MODE_IGNORE_DEFINED,
   EXPANSION_MODE_EVALUATE_DEFINED
} expansion_mode_t;

/* Expand list, and begin lexing from the result (after first
 * prefixing a token of type 'head_token_type').
 */
static void
_glcpp_parser_expand_and_lex_from(glcpp_parser_t *parser, int head_token_type,
                                  token_list_t *list, expansion_mode_t mode);

/* Perform macro expansion in-place on the given list. */
static void
_glcpp_parser_expand_token_list(glcpp_parser_t *parser, token_list_t *list,
                                expansion_mode_t mode);

static void
_glcpp_parser_print_expanded_token_list(glcpp_parser_t *parser,
                                        token_list_t *list);

static void
_glcpp_parser_skip_stack_push_if(glcpp_parser_t *parser, YYLTYPE *loc,
                                 int condition);

static void
_glcpp_parser_skip_stack_change_if(glcpp_parser_t *parser, YYLTYPE *loc,
                                   const char *type, int condition);

static void
_glcpp_parser_skip_stack_pop(glcpp_parser_t *parser, YYLTYPE *loc);

static void
_glcpp_parser_handle_version_declaration(glcpp_parser_t *parser, intmax_t version,
                                         const char *ident, bool explicitly_set);

static int
glcpp_parser_lex(YYSTYPE *yylval, YYLTYPE *yylloc, glcpp_parser_t *parser);

static void
glcpp_parser_lex_from(glcpp_parser_t *parser, token_list_t *list);

struct define_include {
   glcpp_parser_t *parser;
   YYLTYPE *loc;
};

static void
glcpp_parser_copy_defines(const void *key, void *data, void *closure);

static void
add_builtin_define(glcpp_parser_t *parser, const char *name, int value);


#line 250 "src/compiler/glsl/glcpp/glcpp-parse.c"

# ifndef YY_CAST
#  ifdef __cplusplus
#   define YY_CAST(Type, Val) static_cast<Type> (Val)
#   define YY_REINTERPRET_CAST(Type, Val) reinterpret_cast<Type> (Val)
#  else
#   define YY_CAST(Type, Val) ((Type) (Val))
#   define YY_REINTERPRET_CAST(Type, Val) ((Type) (Val))
#  endif
# endif
# ifndef YY_NULLPTR
#  if defined __cplusplus
#   if 201103L <= __cplusplus
#    define YY_NULLPTR nullptr
#   else
#    define YY_NULLPTR 0
#   endif
#  else
#   define YY_NULLPTR ((void*)0)
#  endif
# endif

/* Enabling verbose error messages.  */
#ifdef YYERROR_VERBOSE
# undef YYERROR_VERBOSE
# define YYERROR_VERBOSE 1
#else
# define YYERROR_VERBOSE 1
#endif

/* Use api.header.include to #include this header
   instead of duplicating it here.  */
#ifndef YY_GLCPP_PARSER_SRC_COMPILER_GLSL_GLCPP_GLCPP_PARSE_H_INCLUDED
# define YY_GLCPP_PARSER_SRC_COMPILER_GLSL_GLCPP_GLCPP_PARSE_H_INCLUDED
/* Debug traces.  */
#ifndef YYDEBUG
# define YYDEBUG 1
#endif
#if YYDEBUG
extern int glcpp_parser_debug;
#endif

/* Token type.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
  enum yytokentype
  {
    DEFINED = 258,
    ELIF_EXPANDED = 259,
    HASH_TOKEN = 260,
    DEFINE_TOKEN = 261,
    FUNC_IDENTIFIER = 262,
    OBJ_IDENTIFIER = 263,
    ELIF = 264,
    ELSE = 265,
    ENDIF = 266,
    ERROR_TOKEN = 267,
    IF = 268,
    IFDEF = 269,
    IFNDEF = 270,
    LINE = 271,
    PRAGMA = 272,
    UNDEF = 273,
    VERSION_TOKEN = 274,
    GARBAGE = 275,
    IDENTIFIER = 276,
    IF_EXPANDED = 277,
    INTEGER = 278,
    INTEGER_STRING = 279,
    LINE_EXPANDED = 280,
    NEWLINE = 281,
    OTHER = 282,
    PLACEHOLDER = 283,
    SPACE = 284,
    PLUS_PLUS = 285,
    MINUS_MINUS = 286,
    PATH = 287,
    INCLUDE = 288,
    PASTE = 289,
    OR = 290,
    AND = 291,
    EQUAL = 292,
    NOT_EQUAL = 293,
    LESS_OR_EQUAL = 294,
    GREATER_OR_EQUAL = 295,
    LEFT_SHIFT = 296,
    RIGHT_SHIFT = 297,
    UNARY = 298
  };
#endif

/* Value type.  */

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



int glcpp_parser_parse (glcpp_parser_t *parser);

#endif /* !YY_GLCPP_PARSER_SRC_COMPILER_GLSL_GLCPP_GLCPP_PARSE_H_INCLUDED  */



#ifdef short
# undef short
#endif

/* On compilers that do not define __PTRDIFF_MAX__ etc., make sure
   <limits.h> and (if available) <stdint.h> are included
   so that the code can choose integer types of a good width.  */

#ifndef __PTRDIFF_MAX__
# include <limits.h> /* INFRINGES ON USER NAME SPACE */
# if defined __STDC_VERSION__ && 199901 <= __STDC_VERSION__
#  include <stdint.h> /* INFRINGES ON USER NAME SPACE */
#  define YY_STDINT_H
# endif
#endif

/* Narrow types that promote to a signed type and that can represent a
   signed or unsigned integer of at least N bits.  In tables they can
   save space and decrease cache pressure.  Promoting to a signed type
   helps avoid bugs in integer arithmetic.  */

#ifdef __INT_LEAST8_MAX__
typedef __INT_LEAST8_TYPE__ yytype_int8;
#elif defined YY_STDINT_H
typedef int_least8_t yytype_int8;
#else
typedef signed char yytype_int8;
#endif

#ifdef __INT_LEAST16_MAX__
typedef __INT_LEAST16_TYPE__ yytype_int16;
#elif defined YY_STDINT_H
typedef int_least16_t yytype_int16;
#else
typedef short yytype_int16;
#endif

#if defined __UINT_LEAST8_MAX__ && __UINT_LEAST8_MAX__ <= __INT_MAX__
typedef __UINT_LEAST8_TYPE__ yytype_uint8;
#elif (!defined __UINT_LEAST8_MAX__ && defined YY_STDINT_H \
       && UINT_LEAST8_MAX <= INT_MAX)
typedef uint_least8_t yytype_uint8;
#elif !defined __UINT_LEAST8_MAX__ && UCHAR_MAX <= INT_MAX
typedef unsigned char yytype_uint8;
#else
typedef short yytype_uint8;
#endif

#if defined __UINT_LEAST16_MAX__ && __UINT_LEAST16_MAX__ <= __INT_MAX__
typedef __UINT_LEAST16_TYPE__ yytype_uint16;
#elif (!defined __UINT_LEAST16_MAX__ && defined YY_STDINT_H \
       && UINT_LEAST16_MAX <= INT_MAX)
typedef uint_least16_t yytype_uint16;
#elif !defined __UINT_LEAST16_MAX__ && USHRT_MAX <= INT_MAX
typedef unsigned short yytype_uint16;
#else
typedef int yytype_uint16;
#endif

#ifndef YYPTRDIFF_T
# if defined __PTRDIFF_TYPE__ && defined __PTRDIFF_MAX__
#  define YYPTRDIFF_T __PTRDIFF_TYPE__
#  define YYPTRDIFF_MAXIMUM __PTRDIFF_MAX__
# elif defined PTRDIFF_MAX
#  ifndef ptrdiff_t
#   include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  endif
#  define YYPTRDIFF_T ptrdiff_t
#  define YYPTRDIFF_MAXIMUM PTRDIFF_MAX
# else
#  define YYPTRDIFF_T long
#  define YYPTRDIFF_MAXIMUM LONG_MAX
# endif
#endif

#ifndef YYSIZE_T
# ifdef __SIZE_TYPE__
#  define YYSIZE_T __SIZE_TYPE__
# elif defined size_t
#  define YYSIZE_T size_t
# elif defined __STDC_VERSION__ && 199901 <= __STDC_VERSION__
#  include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  define YYSIZE_T size_t
# else
#  define YYSIZE_T unsigned
# endif
#endif

#define YYSIZE_MAXIMUM                                  \
  YY_CAST (YYPTRDIFF_T,                                 \
           (YYPTRDIFF_MAXIMUM < YY_CAST (YYSIZE_T, -1)  \
            ? YYPTRDIFF_MAXIMUM                         \
            : YY_CAST (YYSIZE_T, -1)))

#define YYSIZEOF(X) YY_CAST (YYPTRDIFF_T, sizeof (X))

/* Stored state numbers (used for stacks). */
typedef yytype_uint8 yy_state_t;

/* State numbers in computations.  */
typedef int yy_state_fast_t;

#ifndef YY_
# if defined YYENABLE_NLS && YYENABLE_NLS
#  if ENABLE_NLS
#   include <libintl.h> /* INFRINGES ON USER NAME SPACE */
#   define YY_(Msgid) dgettext ("bison-runtime", Msgid)
#  endif
# endif
# ifndef YY_
#  define YY_(Msgid) Msgid
# endif
#endif

#ifndef YY_ATTRIBUTE_PURE
# if defined __GNUC__ && 2 < __GNUC__ + (96 <= __GNUC_MINOR__)
#  define YY_ATTRIBUTE_PURE __attribute__ ((__pure__))
# else
#  define YY_ATTRIBUTE_PURE
# endif
#endif

#ifndef YY_ATTRIBUTE_UNUSED
# if defined __GNUC__ && 2 < __GNUC__ + (7 <= __GNUC_MINOR__)
#  define YY_ATTRIBUTE_UNUSED __attribute__ ((__unused__))
# else
#  define YY_ATTRIBUTE_UNUSED
# endif
#endif

/* Suppress unused-variable warnings by "using" E.  */
#if ! defined lint || defined __GNUC__
# define YYUSE(E) ((void) (E))
#else
# define YYUSE(E) /* empty */
#endif

#if defined __GNUC__ && ! defined __ICC && 407 <= __GNUC__ * 100 + __GNUC_MINOR__
/* Suppress an incorrect diagnostic about yylval being uninitialized.  */
# define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN                            \
    _Pragma ("GCC diagnostic push")                                     \
    _Pragma ("GCC diagnostic ignored \"-Wuninitialized\"")              \
    _Pragma ("GCC diagnostic ignored \"-Wmaybe-uninitialized\"")
# define YY_IGNORE_MAYBE_UNINITIALIZED_END      \
    _Pragma ("GCC diagnostic pop")
#else
# define YY_INITIAL_VALUE(Value) Value
#endif
#ifndef YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_END
#endif
#ifndef YY_INITIAL_VALUE
# define YY_INITIAL_VALUE(Value) /* Nothing. */
#endif

#if defined __cplusplus && defined __GNUC__ && ! defined __ICC && 6 <= __GNUC__
# define YY_IGNORE_USELESS_CAST_BEGIN                          \
    _Pragma ("GCC diagnostic push")                            \
    _Pragma ("GCC diagnostic ignored \"-Wuseless-cast\"")
# define YY_IGNORE_USELESS_CAST_END            \
    _Pragma ("GCC diagnostic pop")
#endif
#ifndef YY_IGNORE_USELESS_CAST_BEGIN
# define YY_IGNORE_USELESS_CAST_BEGIN
# define YY_IGNORE_USELESS_CAST_END
#endif


#define YY_ASSERT(E) ((void) (0 && (E)))

#if ! defined yyoverflow || YYERROR_VERBOSE

/* The parser invokes alloca or malloc; define the necessary symbols.  */

# ifdef YYSTACK_USE_ALLOCA
#  if YYSTACK_USE_ALLOCA
#   ifdef __GNUC__
#    define YYSTACK_ALLOC __builtin_alloca
#   elif defined __BUILTIN_VA_ARG_INCR
#    include <alloca.h> /* INFRINGES ON USER NAME SPACE */
#   elif defined _AIX
#    define YYSTACK_ALLOC __alloca
#   elif defined _MSC_VER
#    include <malloc.h> /* INFRINGES ON USER NAME SPACE */
#    define alloca _alloca
#   else
#    define YYSTACK_ALLOC alloca
#    if ! defined _ALLOCA_H && ! defined EXIT_SUCCESS
#     include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
      /* Use EXIT_SUCCESS as a witness for stdlib.h.  */
#     ifndef EXIT_SUCCESS
#      define EXIT_SUCCESS 0
#     endif
#    endif
#   endif
#  endif
# endif

# ifdef YYSTACK_ALLOC
   /* Pacify GCC's 'empty if-body' warning.  */
#  define YYSTACK_FREE(Ptr) do { /* empty */; } while (0)
#  ifndef YYSTACK_ALLOC_MAXIMUM
    /* The OS might guarantee only one guard page at the bottom of the stack,
       and a page size can be as small as 4096 bytes.  So we cannot safely
       invoke alloca (N) if N exceeds 4096.  Use a slightly smaller number
       to allow for a few compiler-allocated temporary stack slots.  */
#   define YYSTACK_ALLOC_MAXIMUM 4032 /* reasonable circa 2006 */
#  endif
# else
#  define YYSTACK_ALLOC YYMALLOC
#  define YYSTACK_FREE YYFREE
#  ifndef YYSTACK_ALLOC_MAXIMUM
#   define YYSTACK_ALLOC_MAXIMUM YYSIZE_MAXIMUM
#  endif
#  if (defined __cplusplus && ! defined EXIT_SUCCESS \
       && ! ((defined YYMALLOC || defined malloc) \
             && (defined YYFREE || defined free)))
#   include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
#   ifndef EXIT_SUCCESS
#    define EXIT_SUCCESS 0
#   endif
#  endif
#  ifndef YYMALLOC
#   define YYMALLOC malloc
#   if ! defined malloc && ! defined EXIT_SUCCESS
void *malloc (YYSIZE_T); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
#  ifndef YYFREE
#   define YYFREE free
#   if ! defined free && ! defined EXIT_SUCCESS
void free (void *); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
# endif
#endif /* ! defined yyoverflow || YYERROR_VERBOSE */


#if (! defined yyoverflow \
     && (! defined __cplusplus \
         || (defined YYLTYPE_IS_TRIVIAL && YYLTYPE_IS_TRIVIAL \
             && defined YYSTYPE_IS_TRIVIAL && YYSTYPE_IS_TRIVIAL)))

/* A type that is properly aligned for any stack member.  */
union yyalloc
{
  yy_state_t yyss_alloc;
  YYSTYPE yyvs_alloc;
  YYLTYPE yyls_alloc;
};

/* The size of the maximum gap between one aligned stack and the next.  */
# define YYSTACK_GAP_MAXIMUM (YYSIZEOF (union yyalloc) - 1)

/* The size of an array large to enough to hold all stacks, each with
   N elements.  */
# define YYSTACK_BYTES(N) \
     ((N) * (YYSIZEOF (yy_state_t) + YYSIZEOF (YYSTYPE) \
             + YYSIZEOF (YYLTYPE)) \
      + 2 * YYSTACK_GAP_MAXIMUM)

# define YYCOPY_NEEDED 1

/* Relocate STACK from its old location to the new one.  The
   local variables YYSIZE and YYSTACKSIZE give the old and new number of
   elements in the stack, and YYPTR gives the new location of the
   stack.  Advance YYPTR to a properly aligned location for the next
   stack.  */
# define YYSTACK_RELOCATE(Stack_alloc, Stack)                           \
    do                                                                  \
      {                                                                 \
        YYPTRDIFF_T yynewbytes;                                         \
        YYCOPY (&yyptr->Stack_alloc, Stack, yysize);                    \
        Stack = &yyptr->Stack_alloc;                                    \
        yynewbytes = yystacksize * YYSIZEOF (*Stack) + YYSTACK_GAP_MAXIMUM; \
        yyptr += yynewbytes / YYSIZEOF (*yyptr);                        \
      }                                                                 \
    while (0)

#endif

#if defined YYCOPY_NEEDED && YYCOPY_NEEDED
/* Copy COUNT objects from SRC to DST.  The source and destination do
   not overlap.  */
# ifndef YYCOPY
#  if defined __GNUC__ && 1 < __GNUC__
#   define YYCOPY(Dst, Src, Count) \
      __builtin_memcpy (Dst, Src, YY_CAST (YYSIZE_T, (Count)) * sizeof (*(Src)))
#  else
#   define YYCOPY(Dst, Src, Count)              \
      do                                        \
        {                                       \
          YYPTRDIFF_T yyi;                      \
          for (yyi = 0; yyi < (Count); yyi++)   \
            (Dst)[yyi] = (Src)[yyi];            \
        }                                       \
      while (0)
#  endif
# endif
#endif /* !YYCOPY_NEEDED */

/* YYFINAL -- State number of the termination state.  */
#define YYFINAL  2
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   731

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  66
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  20
/* YYNRULES -- Number of rules.  */
#define YYNRULES  116
/* YYNSTATES -- Number of states.  */
#define YYNSTATES  185

#define YYUNDEFTOK  2
#define YYMAXUTOK   298


/* YYTRANSLATE(TOKEN-NUM) -- Symbol number corresponding to TOKEN-NUM
   as returned by yylex, with out-of-bounds checking.  */
#define YYTRANSLATE(YYX)                                                \
  (0 <= (YYX) && (YYX) <= YYMAXUTOK ? yytranslate[YYX] : YYUNDEFTOK)

/* YYTRANSLATE[TOKEN-NUM] -- Symbol number corresponding to TOKEN-NUM
   as returned by yylex.  */
static const yytype_int8 yytranslate[] =
{
       0,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,    56,     2,     2,     2,    52,    39,     2,
      54,    55,    50,    48,    58,    49,    63,    51,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,    64,
      42,    65,    43,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,    59,     2,    60,    38,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,    61,    37,    62,    57,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     1,     2,     3,     4,
       5,     6,     7,     8,     9,    10,    11,    12,    13,    14,
      15,    16,    17,    18,    19,    20,    21,    22,    23,    24,
      25,    26,    27,    28,    29,    30,    31,    32,    33,    34,
      35,    36,    40,    41,    44,    45,    46,    47,    53
};

#if YYDEBUG
  /* YYRLINE[YYN] -- Source line where rule number YYN was defined.  */
static const yytype_int16 yyrline[] =
{
       0,   220,   220,   222,   226,   227,   228,   232,   236,   241,
     246,   251,   260,   270,   273,   276,   282,   285,   286,   299,
     300,   352,   426,   447,   457,   463,   469,   495,   515,   515,
     528,   528,   531,   537,   543,   546,   552,   555,   558,   564,
     568,   573,   584,   588,   595,   606,   617,   624,   631,   638,
     645,   652,   659,   666,   673,   680,   687,   694,   701,   708,
     720,   732,   739,   743,   747,   751,   755,   761,   765,   772,
     773,   777,   778,   781,   783,   789,   794,   801,   805,   809,
     813,   817,   821,   825,   832,   833,   834,   835,   836,   837,
     838,   839,   840,   841,   842,   843,   844,   845,   846,   847,
     848,   849,   850,   851,   852,   853,   854,   855,   856,   857,
     858,   859,   860,   861,   862,   863,   864
};
#endif

#if YYDEBUG || YYERROR_VERBOSE || 1
/* YYTNAME[SYMBOL-NUM] -- String name of the symbol SYMBOL-NUM.
   First, the terminals, then, starting at YYNTOKENS, nonterminals.  */
static const char *const yytname[] =
{
  "$end", "error", "$undefined", "DEFINED", "ELIF_EXPANDED", "HASH_TOKEN",
  "DEFINE_TOKEN", "FUNC_IDENTIFIER", "OBJ_IDENTIFIER", "ELIF", "ELSE",
  "ENDIF", "ERROR_TOKEN", "IF", "IFDEF", "IFNDEF", "LINE", "PRAGMA",
  "UNDEF", "VERSION_TOKEN", "GARBAGE", "IDENTIFIER", "IF_EXPANDED",
  "INTEGER", "INTEGER_STRING", "LINE_EXPANDED", "NEWLINE", "OTHER",
  "PLACEHOLDER", "SPACE", "PLUS_PLUS", "MINUS_MINUS", "PATH", "INCLUDE",
  "PASTE", "OR", "AND", "'|'", "'^'", "'&'", "EQUAL", "NOT_EQUAL", "'<'",
  "'>'", "LESS_OR_EQUAL", "GREATER_OR_EQUAL", "LEFT_SHIFT", "RIGHT_SHIFT",
  "'+'", "'-'", "'*'", "'/'", "'%'", "UNARY", "'('", "')'", "'!'", "'~'",
  "','", "'['", "']'", "'{'", "'}'", "'.'", "';'", "'='", "$accept",
  "input", "line", "expanded_line", "define", "control_line",
  "control_line_success", "$@1", "$@2", "control_line_error",
  "integer_constant", "version_constant", "expression", "identifier_list",
  "text_line", "replacement_list", "junk", "pp_tokens",
  "preprocessing_token", "operator", YY_NULLPTR
};
#endif

# ifdef YYPRINT
/* YYTOKNUM[NUM] -- (External) token number corresponding to the
   (internal) symbol number NUM (which must be that of a token).  */
static const yytype_int16 yytoknum[] =
{
       0,   256,   257,   258,   259,   260,   261,   262,   263,   264,
     265,   266,   267,   268,   269,   270,   271,   272,   273,   274,
     275,   276,   277,   278,   279,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   124,    94,    38,
     292,   293,    60,    62,   294,   295,   296,   297,    43,    45,
      42,    47,    37,   298,    40,    41,    33,   126,    44,    91,
      93,   123,   125,    46,    59,    61
};
# endif

#define YYPACT_NINF (-145)

#define yypact_value_is_default(Yyn) \
  ((Yyn) == YYPACT_NINF)

#define YYTABLE_NINF (-1)

#define yytable_value_is_error(Yyn) \
  0

  /* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
     STATE-NUM.  */
static const yytype_int16 yypact[] =
{
    -145,   105,  -145,  -145,   -15,     4,  -145,   -15,  -145,    45,
    -145,  -145,    -2,  -145,  -145,  -145,  -145,  -145,  -145,  -145,
    -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,
    -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,
    -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,  -145,
    -145,  -145,  -145,   155,  -145,  -145,  -145,  -145,  -145,   -15,
     -15,   -15,   -15,   -15,  -145,   545,    18,   205,  -145,  -145,
       1,   255,   -10,    10,   505,    12,    19,    24,   505,  -145,
      17,   572,    26,  -145,  -145,  -145,  -145,  -145,  -145,   590,
    -145,  -145,  -145,   -15,   -15,   -15,   -15,   -15,   -15,   -15,
     -15,   -15,   -15,   -15,   -15,   -15,   -15,   -15,   -15,   -15,
     -15,    -3,   505,  -145,  -145,  -145,   305,    48,    50,  -145,
    -145,   355,   505,   505,   405,  -145,    53,  -145,   -14,   455,
    -145,  -145,  -145,    60,    80,  -145,   610,   626,   641,   655,
     668,   679,   679,    13,    13,    13,    13,    33,    33,    -5,
      -5,  -145,  -145,  -145,   -19,    85,   505,  -145,  -145,  -145,
    -145,    86,   505,    88,  -145,  -145,    89,  -145,  -145,  -145,
    -145,  -145,   505,   -26,  -145,  -145,  -145,  -145,    90,   505,
      96,  -145,    92,  -145,  -145
};

  /* YYDEFACT[STATE-NUM] -- Default reduction number in state STATE-NUM.
     Performed when YYTABLE does not specify something else to do.  Zero
     means the default is an error.  */
static const yytype_int8 yydefact[] =
{
       2,     0,     1,    81,     0,     0,    77,     0,    78,     0,
      69,    82,    83,   115,   116,    79,   114,   110,   109,   108,
     107,    91,   105,   106,   101,   102,   103,   104,    99,   100,
      93,    94,    92,    97,    98,    86,    87,    96,    95,   112,
      84,    85,    88,    89,    90,   111,   113,     3,     7,     4,
      16,    17,     6,     0,    75,    80,    43,    40,    39,     0,
       0,     0,     0,     0,    42,     0,     0,     0,    28,    30,
       0,     0,     0,     0,     0,     0,     0,     0,     0,    34,
       0,     0,     0,     5,    70,    83,    76,    65,    64,     0,
      62,    63,     9,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,    71,    37,    19,    27,     0,     0,     0,    36,
      23,     0,    73,    73,     0,    35,     0,    41,     0,     0,
      21,     8,    10,     0,     0,    66,    44,    45,    46,    47,
      48,    50,    49,    54,    53,    52,    51,    56,    55,    58,
      57,    61,    60,    59,     0,     0,    72,    26,    29,    31,
      22,     0,    74,     0,    18,    20,     0,    32,    38,    12,
      11,    67,    71,     0,    13,    24,    25,    33,     0,    71,
       0,    14,     0,    68,    15
};

  /* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -145,  -145,  -145,  -145,  -145,    59,  -145,  -145,  -145,  -145,
      -4,  -145,    -6,  -145,  -145,  -144,     0,    -1,   -49,  -145
};

  /* YYDEFGOTO[NTERM-NUM].  */
static const yytype_int16 yydefgoto[] =
{
      -1,     1,    47,    48,   114,    49,    50,   117,   118,    51,
      64,   128,    65,   173,    52,   155,   161,   156,    54,    55
};

  /* YYTABLE[YYPACT[STATE-NUM]] -- What to do in state STATE-NUM.  If
     positive, shift that token.  If negative, reduce the rule whose
     number is the opposite.  If YYTABLE_NINF, syntax error.  */
static const yytype_uint8 yytable[] =
{
      53,    81,   171,     5,    86,    82,    56,   166,    57,    58,
      66,   122,   167,    67,    68,    69,    70,    71,    72,    73,
      74,    75,    76,    77,    78,   111,   112,   119,   178,   179,
      79,   123,   180,    59,    60,   182,   172,    80,   125,    61,
     126,    62,    63,   130,   113,   108,   109,   110,   127,    57,
      58,   154,   132,    87,    88,    89,    90,    91,   133,   104,
     105,   106,   107,   108,   109,   110,   116,    86,    57,    58,
     121,    83,    86,   124,   158,    86,   159,   129,   134,   165,
      86,   106,   107,   108,   109,   110,   169,   136,   137,   138,
     139,   140,   141,   142,   143,   144,   145,   146,   147,   148,
     149,   150,   151,   152,   153,     2,   170,    86,     3,     4,
       5,   174,   175,    86,   176,   177,   181,   183,   184,     0,
       0,   162,   162,   163,     0,     0,     6,     7,     0,     8,
       9,    10,    11,     0,    12,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,    84,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,   115,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,   120,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,   157,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,   160,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,   164,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,   168,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     3,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,     0,     0,     0,     0,     0,     6,     0,     0,     8,
       0,     0,    11,     0,    85,    13,    14,    15,     0,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    28,    29,    30,    31,    32,    33,    34,     0,    35,
      36,    37,    38,    39,    40,    41,    42,    43,    44,    45,
      46,    92,     0,     0,     0,     0,     0,     0,     0,     0,
      93,    94,    95,    96,    97,    98,    99,   100,   101,   102,
     103,   104,   105,   106,   107,   108,   109,   110,   131,     0,
       0,     0,     0,     0,     0,     0,     0,    93,    94,    95,
      96,    97,    98,    99,   100,   101,   102,   103,   104,   105,
     106,   107,   108,   109,   110,    93,    94,    95,    96,    97,
      98,    99,   100,   101,   102,   103,   104,   105,   106,   107,
     108,   109,   110,     0,     0,   135,    94,    95,    96,    97,
      98,    99,   100,   101,   102,   103,   104,   105,   106,   107,
     108,   109,   110,    95,    96,    97,    98,    99,   100,   101,
     102,   103,   104,   105,   106,   107,   108,   109,   110,    96,
      97,    98,    99,   100,   101,   102,   103,   104,   105,   106,
     107,   108,   109,   110,    97,    98,    99,   100,   101,   102,
     103,   104,   105,   106,   107,   108,   109,   110,    98,    99,
     100,   101,   102,   103,   104,   105,   106,   107,   108,   109,
     110,   100,   101,   102,   103,   104,   105,   106,   107,   108,
     109,   110
};

static const yytype_int16 yycheck[] =
{
       1,     7,    21,     5,    53,     9,    21,    21,    23,    24,
       6,    21,    26,     9,    10,    11,    12,    13,    14,    15,
      16,    17,    18,    19,    20,     7,     8,    26,   172,    55,
      26,    21,    58,    48,    49,   179,    55,    33,    26,    54,
      21,    56,    57,    26,    26,    50,    51,    52,    24,    23,
      24,    54,    26,    59,    60,    61,    62,    63,    32,    46,
      47,    48,    49,    50,    51,    52,    67,   116,    23,    24,
      71,    12,   121,    74,    26,   124,    26,    78,    82,    26,
     129,    48,    49,    50,    51,    52,    26,    93,    94,    95,
      96,    97,    98,    99,   100,   101,   102,   103,   104,   105,
     106,   107,   108,   109,   110,     0,    26,   156,     3,     4,
       5,    26,    26,   162,    26,    26,    26,    21,    26,    -1,
      -1,   122,   123,   123,    -1,    -1,    21,    22,    -1,    24,
      25,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    26,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,     3,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    -1,    -1,    -1,    -1,    -1,    21,    -1,    -1,    24,
      -1,    -1,    27,    -1,    29,    30,    31,    32,    -1,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,    -1,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    26,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,    26,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    35,    36,    37,
      38,    39,    40,    41,    42,    43,    44,    45,    46,    47,
      48,    49,    50,    51,    52,    35,    36,    37,    38,    39,
      40,    41,    42,    43,    44,    45,    46,    47,    48,    49,
      50,    51,    52,    -1,    -1,    55,    36,    37,    38,    39,
      40,    41,    42,    43,    44,    45,    46,    47,    48,    49,
      50,    51,    52,    37,    38,    39,    40,    41,    42,    43,
      44,    45,    46,    47,    48,    49,    50,    51,    52,    38,
      39,    40,    41,    42,    43,    44,    45,    46,    47,    48,
      49,    50,    51,    52,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,    40,    41,
      42,    43,    44,    45,    46,    47,    48,    49,    50,    51,
      52,    42,    43,    44,    45,    46,    47,    48,    49,    50,
      51,    52
};

  /* YYSTOS[STATE-NUM] -- The (internal number of the) accessing
     symbol of state STATE-NUM.  */
static const yytype_int8 yystos[] =
{
       0,    67,     0,     3,     4,     5,    21,    22,    24,    25,
      26,    27,    29,    30,    31,    32,    34,    35,    36,    37,
      38,    39,    40,    41,    42,    43,    44,    45,    46,    47,
      48,    49,    50,    51,    52,    54,    55,    56,    57,    58,
      59,    60,    61,    62,    63,    64,    65,    68,    69,    71,
      72,    75,    80,    83,    84,    85,    21,    23,    24,    48,
      49,    54,    56,    57,    76,    78,     6,     9,    10,    11,
      12,    13,    14,    15,    16,    17,    18,    19,    20,    26,
      33,    78,    76,    71,    26,    29,    84,    78,    78,    78,
      78,    78,    26,    35,    36,    37,    38,    39,    40,    41,
      42,    43,    44,    45,    46,    47,    48,    49,    50,    51,
      52,     7,     8,    26,    70,    26,    83,    73,    74,    26,
      26,    83,    21,    21,    83,    26,    21,    24,    77,    83,
      26,    26,    26,    32,    76,    55,    78,    78,    78,    78,
      78,    78,    78,    78,    78,    78,    78,    78,    78,    78,
      78,    78,    78,    78,    54,    81,    83,    26,    26,    26,
      26,    82,    83,    82,    26,    26,    21,    26,    26,    26,
      26,    21,    55,    79,    26,    26,    26,    26,    81,    55,
      58,    26,    81,    21,    26
};

  /* YYR1[YYN] -- Symbol number of symbol that rule YYN derives.  */
static const yytype_int8 yyr1[] =
{
       0,    66,    67,    67,    68,    68,    68,    68,    69,    69,
      69,    69,    69,    70,    70,    70,    71,    71,    71,    72,
      72,    72,    72,    72,    72,    72,    72,    72,    73,    72,
      74,    72,    72,    72,    72,    72,    75,    75,    75,    76,
      76,    77,    78,    78,    78,    78,    78,    78,    78,    78,
      78,    78,    78,    78,    78,    78,    78,    78,    78,    78,
      78,    78,    78,    78,    78,    78,    78,    79,    79,    80,
      80,    81,    81,    82,    82,    83,    83,    84,    84,    84,
      84,    84,    84,    84,    85,    85,    85,    85,    85,    85,
      85,    85,    85,    85,    85,    85,    85,    85,    85,    85,
      85,    85,    85,    85,    85,    85,    85,    85,    85,    85,
      85,    85,    85,    85,    85,    85,    85
};

  /* YYR2[YYN] -- Number of symbols on the right hand side of rule YYN.  */
static const yytype_int8 yyr2[] =
{
       0,     2,     0,     2,     1,     2,     1,     1,     3,     3,
       3,     4,     4,     3,     5,     6,     1,     1,     4,     3,
       4,     3,     4,     3,     5,     5,     4,     3,     0,     4,
       0,     4,     4,     5,     2,     3,     3,     3,     4,     1,
       1,     1,     1,     1,     3,     3,     3,     3,     3,     3,
       3,     3,     3,     3,     3,     3,     3,     3,     3,     3,
       3,     3,     2,     2,     2,     2,     3,     1,     3,     1,
       2,     0,     1,     0,     1,     1,     2,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1
};


#define yyerrok         (yyerrstatus = 0)
#define yyclearin       (yychar = YYEMPTY)
#define YYEMPTY         (-2)
#define YYEOF           0

#define YYACCEPT        goto yyacceptlab
#define YYABORT         goto yyabortlab
#define YYERROR         goto yyerrorlab


#define YYRECOVERING()  (!!yyerrstatus)

#define YYBACKUP(Token, Value)                                    \
  do                                                              \
    if (yychar == YYEMPTY)                                        \
      {                                                           \
        yychar = (Token);                                         \
        yylval = (Value);                                         \
        YYPOPSTACK (yylen);                                       \
        yystate = *yyssp;                                         \
        goto yybackup;                                            \
      }                                                           \
    else                                                          \
      {                                                           \
        yyerror (&yylloc, parser, YY_("syntax error: cannot back up")); \
        YYERROR;                                                  \
      }                                                           \
  while (0)

/* Error token number */
#define YYTERROR        1
#define YYERRCODE       256


/* YYLLOC_DEFAULT -- Set CURRENT to span from RHS[1] to RHS[N].
   If N is 0, then set CURRENT to the empty location which ends
   the previous symbol: RHS[0] (always defined).  */

#ifndef YYLLOC_DEFAULT
# define YYLLOC_DEFAULT(Current, Rhs, N)                                \
    do                                                                  \
      if (N)                                                            \
        {                                                               \
          (Current).first_line   = YYRHSLOC (Rhs, 1).first_line;        \
          (Current).first_column = YYRHSLOC (Rhs, 1).first_column;      \
          (Current).last_line    = YYRHSLOC (Rhs, N).last_line;         \
          (Current).last_column  = YYRHSLOC (Rhs, N).last_column;       \
        }                                                               \
      else                                                              \
        {                                                               \
          (Current).first_line   = (Current).last_line   =              \
            YYRHSLOC (Rhs, 0).last_line;                                \
          (Current).first_column = (Current).last_column =              \
            YYRHSLOC (Rhs, 0).last_column;                              \
        }                                                               \
    while (0)
#endif

#define YYRHSLOC(Rhs, K) ((Rhs)[K])


/* Enable debugging if requested.  */
#if YYDEBUG

# ifndef YYFPRINTF
#  include <stdio.h> /* INFRINGES ON USER NAME SPACE */
#  define YYFPRINTF fprintf
# endif

# define YYDPRINTF(Args)                        \
do {                                            \
  if (yydebug)                                  \
    YYFPRINTF Args;                             \
} while (0)


/* YY_LOCATION_PRINT -- Print the location on the stream.
   This macro was not mandated originally: define only if we know
   we won't break user code: when these are the locations we know.  */

#ifndef YY_LOCATION_PRINT
# if defined YYLTYPE_IS_TRIVIAL && YYLTYPE_IS_TRIVIAL

/* Print *YYLOCP on YYO.  Private, do not rely on its existence. */

YY_ATTRIBUTE_UNUSED
static int
yy_location_print_ (FILE *yyo, YYLTYPE const * const yylocp)
{
  int res = 0;
  int end_col = 0 != yylocp->last_column ? yylocp->last_column - 1 : 0;
  if (0 <= yylocp->first_line)
    {
      res += YYFPRINTF (yyo, "%d", yylocp->first_line);
      if (0 <= yylocp->first_column)
        res += YYFPRINTF (yyo, ".%d", yylocp->first_column);
    }
  if (0 <= yylocp->last_line)
    {
      if (yylocp->first_line < yylocp->last_line)
        {
          res += YYFPRINTF (yyo, "-%d", yylocp->last_line);
          if (0 <= end_col)
            res += YYFPRINTF (yyo, ".%d", end_col);
        }
      else if (0 <= end_col && yylocp->first_column < end_col)
        res += YYFPRINTF (yyo, "-%d", end_col);
    }
  return res;
 }

#  define YY_LOCATION_PRINT(File, Loc)          \
  yy_location_print_ (File, &(Loc))

# else
#  define YY_LOCATION_PRINT(File, Loc) ((void) 0)
# endif
#endif


# define YY_SYMBOL_PRINT(Title, Type, Value, Location)                    \
do {                                                                      \
  if (yydebug)                                                            \
    {                                                                     \
      YYFPRINTF (stderr, "%s ", Title);                                   \
      yy_symbol_print (stderr,                                            \
                  Type, Value, Location, parser); \
      YYFPRINTF (stderr, "\n");                                           \
    }                                                                     \
} while (0)


/*-----------------------------------.
| Print this symbol's value on YYO.  |
`-----------------------------------*/

static void
yy_symbol_value_print (FILE *yyo, int yytype, YYSTYPE const * const yyvaluep, YYLTYPE const * const yylocationp, glcpp_parser_t *parser)
{
  FILE *yyoutput = yyo;
  YYUSE (yyoutput);
  YYUSE (yylocationp);
  YYUSE (parser);
  if (!yyvaluep)
    return;
# ifdef YYPRINT
  if (yytype < YYNTOKENS)
    YYPRINT (yyo, yytoknum[yytype], *yyvaluep);
# endif
  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  YYUSE (yytype);
  YY_IGNORE_MAYBE_UNINITIALIZED_END
}


/*---------------------------.
| Print this symbol on YYO.  |
`---------------------------*/

static void
yy_symbol_print (FILE *yyo, int yytype, YYSTYPE const * const yyvaluep, YYLTYPE const * const yylocationp, glcpp_parser_t *parser)
{
  YYFPRINTF (yyo, "%s %s (",
             yytype < YYNTOKENS ? "token" : "nterm", yytname[yytype]);

  YY_LOCATION_PRINT (yyo, *yylocationp);
  YYFPRINTF (yyo, ": ");
  yy_symbol_value_print (yyo, yytype, yyvaluep, yylocationp, parser);
  YYFPRINTF (yyo, ")");
}

/*------------------------------------------------------------------.
| yy_stack_print -- Print the state stack from its BOTTOM up to its |
| TOP (included).                                                   |
`------------------------------------------------------------------*/

static void
yy_stack_print (yy_state_t *yybottom, yy_state_t *yytop)
{
  YYFPRINTF (stderr, "Stack now");
  for (; yybottom <= yytop; yybottom++)
    {
      int yybot = *yybottom;
      YYFPRINTF (stderr, " %d", yybot);
    }
  YYFPRINTF (stderr, "\n");
}

# define YY_STACK_PRINT(Bottom, Top)                            \
do {                                                            \
  if (yydebug)                                                  \
    yy_stack_print ((Bottom), (Top));                           \
} while (0)


/*------------------------------------------------.
| Report that the YYRULE is going to be reduced.  |
`------------------------------------------------*/

static void
yy_reduce_print (yy_state_t *yyssp, YYSTYPE *yyvsp, YYLTYPE *yylsp, int yyrule, glcpp_parser_t *parser)
{
  int yylno = yyrline[yyrule];
  int yynrhs = yyr2[yyrule];
  int yyi;
  YYFPRINTF (stderr, "Reducing stack by rule %d (line %d):\n",
             yyrule - 1, yylno);
  /* The symbols being reduced.  */
  for (yyi = 0; yyi < yynrhs; yyi++)
    {
      YYFPRINTF (stderr, "   $%d = ", yyi + 1);
      yy_symbol_print (stderr,
                       yystos[yyssp[yyi + 1 - yynrhs]],
                       &yyvsp[(yyi + 1) - (yynrhs)]
                       , &(yylsp[(yyi + 1) - (yynrhs)])                       , parser);
      YYFPRINTF (stderr, "\n");
    }
}

# define YY_REDUCE_PRINT(Rule)          \
do {                                    \
  if (yydebug)                          \
    yy_reduce_print (yyssp, yyvsp, yylsp, Rule, parser); \
} while (0)

/* Nonzero means print parse trace.  It is left uninitialized so that
   multiple parsers can coexist.  */
int yydebug;
#else /* !YYDEBUG */
# define YYDPRINTF(Args)
# define YY_SYMBOL_PRINT(Title, Type, Value, Location)
# define YY_STACK_PRINT(Bottom, Top)
# define YY_REDUCE_PRINT(Rule)
#endif /* !YYDEBUG */


/* YYINITDEPTH -- initial size of the parser's stacks.  */
#ifndef YYINITDEPTH
# define YYINITDEPTH 200
#endif

/* YYMAXDEPTH -- maximum size the stacks can grow to (effective only
   if the built-in stack extension method is used).

   Do not make this value too large; the results are undefined if
   YYSTACK_ALLOC_MAXIMUM < YYSTACK_BYTES (YYMAXDEPTH)
   evaluated with infinite-precision integer arithmetic.  */

#ifndef YYMAXDEPTH
# define YYMAXDEPTH 10000
#endif


#if YYERROR_VERBOSE

# ifndef yystrlen
#  if defined __GLIBC__ && defined _STRING_H
#   define yystrlen(S) (YY_CAST (YYPTRDIFF_T, strlen (S)))
#  else
/* Return the length of YYSTR.  */
static YYPTRDIFF_T
yystrlen (const char *yystr)
{
  YYPTRDIFF_T yylen;
  for (yylen = 0; yystr[yylen]; yylen++)
    continue;
  return yylen;
}
#  endif
# endif

# ifndef yystpcpy
#  if defined __GLIBC__ && defined _STRING_H && defined _GNU_SOURCE
#   define yystpcpy stpcpy
#  else
/* Copy YYSRC to YYDEST, returning the address of the terminating '\0' in
   YYDEST.  */
static char *
yystpcpy (char *yydest, const char *yysrc)
{
  char *yyd = yydest;
  const char *yys = yysrc;

  while ((*yyd++ = *yys++) != '\0')
    continue;

  return yyd - 1;
}
#  endif
# endif

# ifndef yytnamerr
/* Copy to YYRES the contents of YYSTR after stripping away unnecessary
   quotes and backslashes, so that it's suitable for yyerror.  The
   heuristic is that double-quoting is unnecessary unless the string
   contains an apostrophe, a comma, or backslash (other than
   backslash-backslash).  YYSTR is taken from yytname.  If YYRES is
   null, do not copy; instead, return the length of what the result
   would have been.  */
static YYPTRDIFF_T
yytnamerr (char *yyres, const char *yystr)
{
  if (*yystr == '"')
    {
      YYPTRDIFF_T yyn = 0;
      char const *yyp = yystr;

      for (;;)
        switch (*++yyp)
          {
          case '\'':
          case ',':
            goto do_not_strip_quotes;

          case '\\':
            if (*++yyp != '\\')
              goto do_not_strip_quotes;
            else
              goto append;

          append:
          default:
            if (yyres)
              yyres[yyn] = *yyp;
            yyn++;
            break;

          case '"':
            if (yyres)
              yyres[yyn] = '\0';
            return yyn;
          }
    do_not_strip_quotes: ;
    }

  if (yyres)
    return yystpcpy (yyres, yystr) - yyres;
  else
    return yystrlen (yystr);
}
# endif

/* Copy into *YYMSG, which is of size *YYMSG_ALLOC, an error message
   about the unexpected token YYTOKEN for the state stack whose top is
   YYSSP.

   Return 0 if *YYMSG was successfully written.  Return 1 if *YYMSG is
   not large enough to hold the message.  In that case, also set
   *YYMSG_ALLOC to the required number of bytes.  Return 2 if the
   required number of bytes is too large to store.  */
static int
yysyntax_error (YYPTRDIFF_T *yymsg_alloc, char **yymsg,
                yy_state_t *yyssp, int yytoken)
{
  enum { YYERROR_VERBOSE_ARGS_MAXIMUM = 5 };
  /* Internationalized format string. */
  const char *yyformat = YY_NULLPTR;
  /* Arguments of yyformat: reported tokens (one for the "unexpected",
     one per "expected"). */
  char const *yyarg[YYERROR_VERBOSE_ARGS_MAXIMUM];
  /* Actual size of YYARG. */
  int yycount = 0;
  /* Cumulated lengths of YYARG.  */
  YYPTRDIFF_T yysize = 0;

  /* There are many possibilities here to consider:
     - If this state is a consistent state with a default action, then
       the only way this function was invoked is if the default action
       is an error action.  In that case, don't check for expected
       tokens because there are none.
     - The only way there can be no lookahead present (in yychar) is if
       this state is a consistent state with a default action.  Thus,
       detecting the absence of a lookahead is sufficient to determine
       that there is no unexpected or expected token to report.  In that
       case, just report a simple "syntax error".
     - Don't assume there isn't a lookahead just because this state is a
       consistent state with a default action.  There might have been a
       previous inconsistent state, consistent state with a non-default
       action, or user semantic action that manipulated yychar.
     - Of course, the expected token list depends on states to have
       correct lookahead information, and it depends on the parser not
       to perform extra reductions after fetching a lookahead from the
       scanner and before detecting a syntax error.  Thus, state merging
       (from LALR or IELR) and default reductions corrupt the expected
       token list.  However, the list is correct for canonical LR with
       one exception: it will still contain any token that will not be
       accepted due to an error action in a later state.
  */
  if (yytoken != YYEMPTY)
    {
      int yyn = yypact[*yyssp];
      YYPTRDIFF_T yysize0 = yytnamerr (YY_NULLPTR, yytname[yytoken]);
      yysize = yysize0;
      yyarg[yycount++] = yytname[yytoken];
      if (!yypact_value_is_default (yyn))
        {
          /* Start YYX at -YYN if negative to avoid negative indexes in
             YYCHECK.  In other words, skip the first -YYN actions for
             this state because they are default actions.  */
          int yyxbegin = yyn < 0 ? -yyn : 0;
          /* Stay within bounds of both yycheck and yytname.  */
          int yychecklim = YYLAST - yyn + 1;
          int yyxend = yychecklim < YYNTOKENS ? yychecklim : YYNTOKENS;
          int yyx;

          for (yyx = yyxbegin; yyx < yyxend; ++yyx)
            if (yycheck[yyx + yyn] == yyx && yyx != YYTERROR
                && !yytable_value_is_error (yytable[yyx + yyn]))
              {
                if (yycount == YYERROR_VERBOSE_ARGS_MAXIMUM)
                  {
                    yycount = 1;
                    yysize = yysize0;
                    break;
                  }
                yyarg[yycount++] = yytname[yyx];
                {
                  YYPTRDIFF_T yysize1
                    = yysize + yytnamerr (YY_NULLPTR, yytname[yyx]);
                  if (yysize <= yysize1 && yysize1 <= YYSTACK_ALLOC_MAXIMUM)
                    yysize = yysize1;
                  else
                    return 2;
                }
              }
        }
    }

  switch (yycount)
    {
# define YYCASE_(N, S)                      \
      case N:                               \
        yyformat = S;                       \
      break
    default: /* Avoid compiler warnings. */
      YYCASE_(0, YY_("syntax error"));
      YYCASE_(1, YY_("syntax error, unexpected %s"));
      YYCASE_(2, YY_("syntax error, unexpected %s, expecting %s"));
      YYCASE_(3, YY_("syntax error, unexpected %s, expecting %s or %s"));
      YYCASE_(4, YY_("syntax error, unexpected %s, expecting %s or %s or %s"));
      YYCASE_(5, YY_("syntax error, unexpected %s, expecting %s or %s or %s or %s"));
# undef YYCASE_
    }

  {
    /* Don't count the "%s"s in the final size, but reserve room for
       the terminator.  */
    YYPTRDIFF_T yysize1 = yysize + (yystrlen (yyformat) - 2 * yycount) + 1;
    if (yysize <= yysize1 && yysize1 <= YYSTACK_ALLOC_MAXIMUM)
      yysize = yysize1;
    else
      return 2;
  }

  if (*yymsg_alloc < yysize)
    {
      *yymsg_alloc = 2 * yysize;
      if (! (yysize <= *yymsg_alloc
             && *yymsg_alloc <= YYSTACK_ALLOC_MAXIMUM))
        *yymsg_alloc = YYSTACK_ALLOC_MAXIMUM;
      return 1;
    }

  /* Avoid sprintf, as that infringes on the user's name space.
     Don't have undefined behavior even if the translation
     produced a string with the wrong number of "%s"s.  */
  {
    char *yyp = *yymsg;
    int yyi = 0;
    while ((*yyp = *yyformat) != '\0')
      if (*yyp == '%' && yyformat[1] == 's' && yyi < yycount)
        {
          yyp += yytnamerr (yyp, yyarg[yyi++]);
          yyformat += 2;
        }
      else
        {
          ++yyp;
          ++yyformat;
        }
  }
  return 0;
}
#endif /* YYERROR_VERBOSE */

/*-----------------------------------------------.
| Release the memory associated to this symbol.  |
`-----------------------------------------------*/

static void
yydestruct (const char *yymsg, int yytype, YYSTYPE *yyvaluep, YYLTYPE *yylocationp, glcpp_parser_t *parser)
{
  YYUSE (yyvaluep);
  YYUSE (yylocationp);
  YYUSE (parser);
  if (!yymsg)
    yymsg = "Deleting";
  YY_SYMBOL_PRINT (yymsg, yytype, yyvaluep, yylocationp);

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  YYUSE (yytype);
  YY_IGNORE_MAYBE_UNINITIALIZED_END
}




/*----------.
| yyparse.  |
`----------*/

int
yyparse (glcpp_parser_t *parser)
{
/* The lookahead symbol.  */
int yychar;


/* The semantic value of the lookahead symbol.  */
/* Default value used for initialization, for pacifying older GCCs
   or non-GCC compilers.  */
YY_INITIAL_VALUE (static YYSTYPE yyval_default;)
YYSTYPE yylval YY_INITIAL_VALUE (= yyval_default);

/* Location data for the lookahead symbol.  */
static YYLTYPE yyloc_default
# if defined YYLTYPE_IS_TRIVIAL && YYLTYPE_IS_TRIVIAL
  = { 1, 1, 1, 1 }
# endif
;
YYLTYPE yylloc = yyloc_default;

    /* Number of syntax errors so far.  */
    int yynerrs;

    yy_state_fast_t yystate;
    /* Number of tokens to shift before error messages enabled.  */
    int yyerrstatus;

    /* The stacks and their tools:
       'yyss': related to states.
       'yyvs': related to semantic values.
       'yyls': related to locations.

       Refer to the stacks through separate pointers, to allow yyoverflow
       to reallocate them elsewhere.  */

    /* The state stack.  */
    yy_state_t yyssa[YYINITDEPTH];
    yy_state_t *yyss;
    yy_state_t *yyssp;

    /* The semantic value stack.  */
    YYSTYPE yyvsa[YYINITDEPTH];
    YYSTYPE *yyvs;
    YYSTYPE *yyvsp;

    /* The location stack.  */
    YYLTYPE yylsa[YYINITDEPTH];
    YYLTYPE *yyls;
    YYLTYPE *yylsp;

    /* The locations where the error started and ended.  */
    YYLTYPE yyerror_range[3];

    YYPTRDIFF_T yystacksize;

  int yyn;
  int yyresult;
  /* Lookahead token as an internal (translated) token number.  */
  int yytoken = 0;
  /* The variables used to return semantic value and location from the
     action routines.  */
  YYSTYPE yyval;
  YYLTYPE yyloc;

#if YYERROR_VERBOSE
  /* Buffer for error messages, and its allocated size.  */
  char yymsgbuf[128];
  char *yymsg = yymsgbuf;
  YYPTRDIFF_T yymsg_alloc = sizeof yymsgbuf;
#endif

#define YYPOPSTACK(N)   (yyvsp -= (N), yyssp -= (N), yylsp -= (N))

  /* The number of symbols on the RHS of the reduced rule.
     Keep to zero when no symbol should be popped.  */
  int yylen = 0;

  yyssp = yyss = yyssa;
  yyvsp = yyvs = yyvsa;
  yylsp = yyls = yylsa;
  yystacksize = YYINITDEPTH;

  YYDPRINTF ((stderr, "Starting parse\n"));

  yystate = 0;
  yyerrstatus = 0;
  yynerrs = 0;
  yychar = YYEMPTY; /* Cause a token to be read.  */

/* User initialization code.  */
#line 180 "src/compiler/glsl/glcpp/glcpp-parse.y"
{
   yylloc.first_line = 1;
   yylloc.first_column = 1;
   yylloc.last_line = 1;
   yylloc.last_column = 1;
   yylloc.source = 0;
}

#line 1689 "src/compiler/glsl/glcpp/glcpp-parse.c"

  yylsp[0] = yylloc;
  goto yysetstate;


/*------------------------------------------------------------.
| yynewstate -- push a new state, which is found in yystate.  |
`------------------------------------------------------------*/
yynewstate:
  /* In all cases, when you get here, the value and location stacks
     have just been pushed.  So pushing a state here evens the stacks.  */
  yyssp++;


/*--------------------------------------------------------------------.
| yysetstate -- set current state (the top of the stack) to yystate.  |
`--------------------------------------------------------------------*/
yysetstate:
  YYDPRINTF ((stderr, "Entering state %d\n", yystate));
  YY_ASSERT (0 <= yystate && yystate < YYNSTATES);
  YY_IGNORE_USELESS_CAST_BEGIN
  *yyssp = YY_CAST (yy_state_t, yystate);
  YY_IGNORE_USELESS_CAST_END

  if (yyss + yystacksize - 1 <= yyssp)
#if !defined yyoverflow && !defined YYSTACK_RELOCATE
    goto yyexhaustedlab;
#else
    {
      /* Get the current used size of the three stacks, in elements.  */
      YYPTRDIFF_T yysize = yyssp - yyss + 1;

# if defined yyoverflow
      {
        /* Give user a chance to reallocate the stack.  Use copies of
           these so that the &'s don't force the real ones into
           memory.  */
        yy_state_t *yyss1 = yyss;
        YYSTYPE *yyvs1 = yyvs;
        YYLTYPE *yyls1 = yyls;

        /* Each stack pointer address is followed by the size of the
           data in use in that stack, in bytes.  This used to be a
           conditional around just the two extra args, but that might
           be undefined if yyoverflow is a macro.  */
        yyoverflow (YY_("memory exhausted"),
                    &yyss1, yysize * YYSIZEOF (*yyssp),
                    &yyvs1, yysize * YYSIZEOF (*yyvsp),
                    &yyls1, yysize * YYSIZEOF (*yylsp),
                    &yystacksize);
        yyss = yyss1;
        yyvs = yyvs1;
        yyls = yyls1;
      }
# else /* defined YYSTACK_RELOCATE */
      /* Extend the stack our own way.  */
      if (YYMAXDEPTH <= yystacksize)
        goto yyexhaustedlab;
      yystacksize *= 2;
      if (YYMAXDEPTH < yystacksize)
        yystacksize = YYMAXDEPTH;

      {
        yy_state_t *yyss1 = yyss;
        union yyalloc *yyptr =
          YY_CAST (union yyalloc *,
                   YYSTACK_ALLOC (YY_CAST (YYSIZE_T, YYSTACK_BYTES (yystacksize))));
        if (! yyptr)
          goto yyexhaustedlab;
        YYSTACK_RELOCATE (yyss_alloc, yyss);
        YYSTACK_RELOCATE (yyvs_alloc, yyvs);
        YYSTACK_RELOCATE (yyls_alloc, yyls);
# undef YYSTACK_RELOCATE
        if (yyss1 != yyssa)
          YYSTACK_FREE (yyss1);
      }
# endif

      yyssp = yyss + yysize - 1;
      yyvsp = yyvs + yysize - 1;
      yylsp = yyls + yysize - 1;

      YY_IGNORE_USELESS_CAST_BEGIN
      YYDPRINTF ((stderr, "Stack size increased to %ld\n",
                  YY_CAST (long, yystacksize)));
      YY_IGNORE_USELESS_CAST_END

      if (yyss + yystacksize - 1 <= yyssp)
        YYABORT;
    }
#endif /* !defined yyoverflow && !defined YYSTACK_RELOCATE */

  if (yystate == YYFINAL)
    YYACCEPT;

  goto yybackup;


/*-----------.
| yybackup.  |
`-----------*/
yybackup:
  /* Do appropriate processing given the current state.  Read a
     lookahead token if we need one and don't already have one.  */

  /* First try to decide what to do without reference to lookahead token.  */
  yyn = yypact[yystate];
  if (yypact_value_is_default (yyn))
    goto yydefault;

  /* Not known => get a lookahead token if don't already have one.  */

  /* YYCHAR is either YYEMPTY or YYEOF or a valid lookahead symbol.  */
  if (yychar == YYEMPTY)
    {
      YYDPRINTF ((stderr, "Reading a token: "));
      yychar = yylex (&yylval, &yylloc, parser);
    }

  if (yychar <= YYEOF)
    {
      yychar = yytoken = YYEOF;
      YYDPRINTF ((stderr, "Now at end of input.\n"));
    }
  else
    {
      yytoken = YYTRANSLATE (yychar);
      YY_SYMBOL_PRINT ("Next token is", yytoken, &yylval, &yylloc);
    }

  /* If the proper action on seeing token YYTOKEN is to reduce or to
     detect an error, take that action.  */
  yyn += yytoken;
  if (yyn < 0 || YYLAST < yyn || yycheck[yyn] != yytoken)
    goto yydefault;
  yyn = yytable[yyn];
  if (yyn <= 0)
    {
      if (yytable_value_is_error (yyn))
        goto yyerrlab;
      yyn = -yyn;
      goto yyreduce;
    }

  /* Count tokens shifted since error; after three, turn off error
     status.  */
  if (yyerrstatus)
    yyerrstatus--;

  /* Shift the lookahead token.  */
  YY_SYMBOL_PRINT ("Shifting", yytoken, &yylval, &yylloc);
  yystate = yyn;
  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END
  *++yylsp = yylloc;

  /* Discard the shifted token.  */
  yychar = YYEMPTY;
  goto yynewstate;


/*-----------------------------------------------------------.
| yydefault -- do the default action for the current state.  |
`-----------------------------------------------------------*/
yydefault:
  yyn = yydefact[yystate];
  if (yyn == 0)
    goto yyerrlab;
  goto yyreduce;


/*-----------------------------.
| yyreduce -- do a reduction.  |
`-----------------------------*/
yyreduce:
  /* yyn is the number of a rule to reduce with.  */
  yylen = yyr2[yyn];

  /* If YYLEN is nonzero, implement the default value of the action:
     '$$ = $1'.

     Otherwise, the following line sets YYVAL to garbage.
     This behavior is undocumented and Bison
     users should not rely upon it.  Assigning to YYVAL
     unconditionally makes the parser a bit smaller, and it avoids a
     GCC warning that YYVAL may be used uninitialized.  */
  yyval = yyvsp[1-yylen];

  /* Default location. */
  YYLLOC_DEFAULT (yyloc, (yylsp - yylen), yylen);
  yyerror_range[1] = yyloc;
  YY_REDUCE_PRINT (yyn);
  switch (yyn)
    {
  case 6:
#line 228 "src/compiler/glsl/glcpp/glcpp-parse.y"
                  {
		_glcpp_parser_print_expanded_token_list (parser, (yyvsp[0].token_list));
		_mesa_string_buffer_append_char(parser->output, '\n');
	}
#line 1891 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 8:
#line 236 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                       {
		if (parser->is_gles && (yyvsp[-1].expression_value).undefined_macro)
			glcpp_error(& (yylsp[-2]), parser, "undefined macro %s in expression (illegal in GLES)", (yyvsp[-1].expression_value).undefined_macro);
		_glcpp_parser_skip_stack_push_if (parser, & (yylsp[-2]), (yyvsp[-1].expression_value).value);
	}
#line 1901 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 9:
#line 241 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                         {
		if (parser->is_gles && (yyvsp[-1].expression_value).undefined_macro)
			glcpp_error(& (yylsp[-2]), parser, "undefined macro %s in expression (illegal in GLES)", (yyvsp[-1].expression_value).undefined_macro);
		_glcpp_parser_skip_stack_change_if (parser, & (yylsp[-2]), "elif", (yyvsp[-1].expression_value).value);
	}
#line 1911 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 10:
#line 246 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                               {
		parser->has_new_line_number = 1;
		parser->new_line_number = (yyvsp[-1].ival);
		_mesa_string_buffer_printf(parser->output, "#line %" PRIiMAX "\n", (yyvsp[-1].ival));
	}
#line 1921 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 11:
#line 251 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                                {
		parser->has_new_line_number = 1;
		parser->new_line_number = (yyvsp[-2].ival);
		parser->has_new_source_number = 1;
		parser->new_source_number = (yyvsp[-1].ival);
		_mesa_string_buffer_printf(parser->output,
					   "#line %" PRIiMAX " %" PRIiMAX "\n",
					    (yyvsp[-2].ival), (yyvsp[-1].ival));
	}
#line 1935 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 12:
#line 260 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                    {
		parser->has_new_line_number = 1;
		parser->new_line_number = (yyvsp[-2].ival);
		_mesa_string_buffer_printf(parser->output,
					   "#line %" PRIiMAX " %s\n",
					    (yyvsp[-2].ival), (yyvsp[-1].str));
	}
#line 1947 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 13:
#line 270 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                {
		_define_object_macro (parser, & (yylsp[-2]), (yyvsp[-2].str), (yyvsp[-1].token_list));
	}
#line 1955 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 14:
#line 273 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                         {
		_define_function_macro (parser, & (yylsp[-4]), (yyvsp[-4].str), NULL, (yyvsp[-1].token_list));
	}
#line 1963 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 15:
#line 276 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                                         {
		_define_function_macro (parser, & (yylsp[-5]), (yyvsp[-5].str), (yyvsp[-3].string_list), (yyvsp[-1].token_list));
	}
#line 1971 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 16:
#line 282 "src/compiler/glsl/glcpp/glcpp-parse.y"
                             {
		_mesa_string_buffer_append_char(parser->output, '\n');
	}
#line 1979 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 18:
#line 286 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                          {

		if (parser->skip_stack == NULL ||
		    parser->skip_stack->type == SKIP_NO_SKIP)
		{
			_glcpp_parser_expand_and_lex_from (parser,
							   LINE_EXPANDED, (yyvsp[-1].token_list),
							   EXPANSION_MODE_IGNORE_DEFINED);
		}
	}
#line 1994 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 20:
#line 300 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                            {
		struct hash_entry *entry;

                /* Section 3.4 (Preprocessor) of the GLSL ES 3.00 spec says:
                 *
                 *    It is an error to undefine or to redefine a built-in
                 *    (pre-defined) macro name.
                 *
                 * The GLSL ES 1.00 spec does not contain this text, but
                 * dEQP's preprocess test in GLES2 checks for it.
                 *
                 * Section 3.3 (Preprocessor) revision 7, of the GLSL 4.50
                 * spec says:
                 *
                 *    By convention, all macro names containing two consecutive
                 *    underscores ( __ ) are reserved for use by underlying
                 *    software layers. Defining or undefining such a name
                 *    in a shader does not itself result in an error, but may
                 *    result in unintended behaviors that stem from having
                 *    multiple definitions of the same name. All macro names
                 *    prefixed with "GL_" (...) are also reseved, and defining
                 *    such a name results in a compile-time error.
                 *
                 * The code below implements the same checks as GLSLang.
                 */
		if (strncmp("GL_", (yyvsp[-1].str), 3) == 0)
			glcpp_error(& (yylsp[-3]), parser, "Built-in (pre-defined)"
				    " names beginning with GL_ cannot be undefined.");
		else if (strstr((yyvsp[-1].str), "__") != NULL) {
			if (parser->is_gles
			    && parser->version >= 300
			    && (strcmp("__LINE__", (yyvsp[-1].str)) == 0
				|| strcmp("__FILE__", (yyvsp[-1].str)) == 0
				|| strcmp("__VERSION__", (yyvsp[-1].str)) == 0)) {
				glcpp_error(& (yylsp[-3]), parser, "Built-in (pre-defined)"
					    " names cannot be undefined.");
			} else if (parser->is_gles && parser->version <= 300) {
				glcpp_error(& (yylsp[-3]), parser,
					    " names containing consecutive underscores"
					    " are reserved.");
			} else {
				glcpp_warning(& (yylsp[-3]), parser,
					      " names containing consecutive underscores"
					      " are reserved.");
			}
		}

		entry = _mesa_hash_table_search (parser->defines, (yyvsp[-1].str));
		if (entry) {
			_mesa_hash_table_remove (parser->defines, entry);
		}
	}
#line 2051 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 21:
#line 352 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                   {
		size_t include_cursor = _mesa_get_shader_include_cursor(parser->gl_ctx->Shared);

		/* Remove leading and trailing "" or <> */
		char *start = strchr((yyvsp[-1].str), '"');
		if (!start) {
			_mesa_set_shader_include_cursor(parser->gl_ctx->Shared, 0);
			start = strchr((yyvsp[-1].str), '<');
		}
		char *path = strndup(start + 1, strlen(start + 1) - 1);

		const char *shader =
			_mesa_lookup_shader_include(parser->gl_ctx, path, false);
		free(path);

		if (!shader)
			glcpp_error(&(yylsp[-2]), parser, "%s not found", (yyvsp[-1].str));
		else {
			/* Create a temporary parser with the same settings */
			glcpp_parser_t *tmp_parser =
				glcpp_parser_create(parser->gl_ctx, parser->extensions, parser->state);
			tmp_parser->version_set = true;
			tmp_parser->version = parser->version;

			/* Set the shader source and run the lexer */
			glcpp_lex_set_source_string(tmp_parser, shader);

			/* Copy any existing define macros to the temporary
			 * shade include parser.
			 */
			struct define_include di;
			di.parser = tmp_parser;
			di.loc = &(yylsp[-2]);

			hash_table_call_foreach(parser->defines,
						glcpp_parser_copy_defines,
						&di);

			/* Print out '#include' to the glsl parser. We do this
			 * so that it can do the error checking require to
			 * make sure the ARB_shading_language_include
			 * extension is enabled.
			 */
			_mesa_string_buffer_printf(parser->output, "#include\n");

			/* Parse the include string before adding to the
			 * preprocessor output.
			 */
			glcpp_parser_parse(tmp_parser);
			_mesa_string_buffer_printf(parser->info_log, "%s",
						   tmp_parser->info_log->buf);
			_mesa_string_buffer_printf(parser->output, "%s",
						   tmp_parser->output->buf);

			/* Copy any new define macros to the parent parser
			 * and steal the memory of our temp parser so we don't
			 * free these new defines before they are no longer
			 * needed.
			 */
			di.parser = parser;
			di.loc = &(yylsp[-2]);
			ralloc_steal(parser, tmp_parser);

			hash_table_call_foreach(tmp_parser->defines,
						glcpp_parser_copy_defines,
						&di);

			/* Destroy tmp parser memory we no longer need */
			glcpp_lex_destroy(tmp_parser->scanner);
			_mesa_hash_table_destroy(tmp_parser->defines, NULL);
		}

		_mesa_set_shader_include_cursor(parser->gl_ctx->Shared, include_cursor);
	}
#line 2130 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 22:
#line 426 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                        {
		/* Be careful to only evaluate the 'if' expression if
		 * we are not skipping. When we are skipping, we
		 * simply push a new 0-valued 'if' onto the skip
		 * stack.
		 *
		 * This avoids generating diagnostics for invalid
		 * expressions that are being skipped. */
		if (parser->skip_stack == NULL ||
		    parser->skip_stack->type == SKIP_NO_SKIP)
		{
			_glcpp_parser_expand_and_lex_from (parser,
							   IF_EXPANDED, (yyvsp[-1].token_list),
							   EXPANSION_MODE_EVALUATE_DEFINED);
		}	
		else
		{
			_glcpp_parser_skip_stack_push_if (parser, & (yylsp[-3]), 0);
			parser->skip_stack->type = SKIP_TO_ENDIF;
		}
	}
#line 2156 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 23:
#line 447 "src/compiler/glsl/glcpp/glcpp-parse.y"
                              {
		/* #if without an expression is only an error if we
		 *  are not skipping */
		if (parser->skip_stack == NULL ||
		    parser->skip_stack->type == SKIP_NO_SKIP)
		{
			glcpp_error(& (yylsp[-2]), parser, "#if with no expression");
		}	
		_glcpp_parser_skip_stack_push_if (parser, & (yylsp[-2]), 0);
	}
#line 2171 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 24:
#line 457 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                 {
		struct hash_entry *entry =
				_mesa_hash_table_search(parser->defines, (yyvsp[-2].str));
		macro_t *macro = entry ? entry->data : NULL;
		_glcpp_parser_skip_stack_push_if (parser, & (yylsp[-4]), macro != NULL);
	}
#line 2182 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 25:
#line 463 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                  {
		struct hash_entry *entry =
				_mesa_hash_table_search(parser->defines, (yyvsp[-2].str));
		macro_t *macro = entry ? entry->data : NULL;
		_glcpp_parser_skip_stack_push_if (parser, & (yylsp[-2]), macro == NULL);
	}
#line 2193 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 26:
#line 469 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                          {
		/* Be careful to only evaluate the 'elif' expression
		 * if we are not skipping. When we are skipping, we
		 * simply change to a 0-valued 'elif' on the skip
		 * stack.
		 *
		 * This avoids generating diagnostics for invalid
		 * expressions that are being skipped. */
		if (parser->skip_stack &&
		    parser->skip_stack->type == SKIP_TO_ELSE)
		{
			_glcpp_parser_expand_and_lex_from (parser,
							   ELIF_EXPANDED, (yyvsp[-1].token_list),
							   EXPANSION_MODE_EVALUATE_DEFINED);
		}
		else if (parser->skip_stack &&
		    parser->skip_stack->has_else)
		{
			glcpp_error(& (yylsp[-3]), parser, "#elif after #else");
		}
		else
		{
			_glcpp_parser_skip_stack_change_if (parser, & (yylsp[-3]),
							    "elif", 0);
		}
	}
#line 2224 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 27:
#line 495 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                {
		/* #elif without an expression is an error unless we
		 * are skipping. */
		if (parser->skip_stack &&
		    parser->skip_stack->type == SKIP_TO_ELSE)
		{
			glcpp_error(& (yylsp[-2]), parser, "#elif with no expression");
		}
		else if (parser->skip_stack &&
		    parser->skip_stack->has_else)
		{
			glcpp_error(& (yylsp[-2]), parser, "#elif after #else");
		}
		else
		{
			_glcpp_parser_skip_stack_change_if (parser, & (yylsp[-2]),
							    "elif", 0);
			glcpp_warning(& (yylsp[-2]), parser, "ignoring illegal #elif without expression");
		}
	}
#line 2249 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 28:
#line 515 "src/compiler/glsl/glcpp/glcpp-parse.y"
                        { parser->lexing_directive = 1; }
#line 2255 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 29:
#line 515 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                                  {
		if (parser->skip_stack &&
		    parser->skip_stack->has_else)
		{
			glcpp_error(& (yylsp[-3]), parser, "multiple #else");
		}
		else
		{
			_glcpp_parser_skip_stack_change_if (parser, & (yylsp[-3]), "else", 1);
			if (parser->skip_stack)
				parser->skip_stack->has_else = true;
		}
	}
#line 2273 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 30:
#line 528 "src/compiler/glsl/glcpp/glcpp-parse.y"
                         {
		_glcpp_parser_skip_stack_pop (parser, & (yylsp[-1]));
	}
#line 2281 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 32:
#line 531 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                          {
		if (parser->version_set) {
			glcpp_error(& (yylsp[-3]), parser, "#version must appear on the first line");
		}
		_glcpp_parser_handle_version_declaration(parser, (yyvsp[-1].ival), NULL, true);
	}
#line 2292 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 33:
#line 537 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                                                     {
		if (parser->version_set) {
			glcpp_error(& (yylsp[-4]), parser, "#version must appear on the first line");
		}
		_glcpp_parser_handle_version_declaration(parser, (yyvsp[-2].ival), (yyvsp[-1].str), true);
	}
#line 2303 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 34:
#line 543 "src/compiler/glsl/glcpp/glcpp-parse.y"
                           {
		glcpp_parser_resolve_implicit_version(parser);
	}
#line 2311 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 35:
#line 546 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		_mesa_string_buffer_printf(parser->output, "#%s", (yyvsp[-1].str));
	}
#line 2319 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 36:
#line 552 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                       {
		glcpp_error(& (yylsp[-2]), parser, "#%s", (yyvsp[-1].str));
	}
#line 2327 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 37:
#line 555 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                        {
		glcpp_error (& (yylsp[-2]), parser, "#define without macro name");
	}
#line 2335 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 38:
#line 558 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                              {
		glcpp_error (& (yylsp[-3]), parser, "Illegal non-directive after #");
	}
#line 2343 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 39:
#line 564 "src/compiler/glsl/glcpp/glcpp-parse.y"
                       {
		/* let strtoll detect the base */
		(yyval.ival) = strtoll ((yyvsp[0].str), NULL, 0);
	}
#line 2352 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 40:
#line 568 "src/compiler/glsl/glcpp/glcpp-parse.y"
                {
		(yyval.ival) = (yyvsp[0].ival);
	}
#line 2360 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 41:
#line 573 "src/compiler/glsl/glcpp/glcpp-parse.y"
                       {
	   /* Both octal and hexadecimal constants begin with 0. */
	   if ((yyvsp[0].str)[0] == '0' && (yyvsp[0].str)[1] != '\0') {
		glcpp_error(&(yylsp[0]), parser, "invalid #version \"%s\" (not a decimal constant)", (yyvsp[0].str));
		(yyval.ival) = 0;
	   } else {
		(yyval.ival) = strtoll((yyvsp[0].str), NULL, 10);
	   }
	}
#line 2374 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 42:
#line 584 "src/compiler/glsl/glcpp/glcpp-parse.y"
                         {
		(yyval.expression_value).value = (yyvsp[0].ival);
		(yyval.expression_value).undefined_macro = NULL;
	}
#line 2383 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 43:
#line 588 "src/compiler/glsl/glcpp/glcpp-parse.y"
                   {
		(yyval.expression_value).value = 0;
		if (parser->is_gles)
			(yyval.expression_value).undefined_macro = linear_strdup(parser->linalloc, (yyvsp[0].str));
		else
			(yyval.expression_value).undefined_macro = NULL;
	}
#line 2395 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 44:
#line 595 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                 {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value || (yyvsp[0].expression_value).value;

		/* Short-circuit: Only flag undefined from right side
		 * if left side evaluates to false.
		 */
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else if (! (yyvsp[-2].expression_value).value)
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2411 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 45:
#line 606 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value && (yyvsp[0].expression_value).value;

		/* Short-circuit: Only flag undefined from right-side
		 * if left side evaluates to true.
		 */
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else if ((yyvsp[-2].expression_value).value)
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2427 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 46:
#line 617 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value | (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2439 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 47:
#line 624 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value ^ (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2451 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 48:
#line 631 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value & (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2463 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 49:
#line 638 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                        {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value != (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2475 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 50:
#line 645 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                    {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value == (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2487 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 51:
#line 652 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                               {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value >= (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2499 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 52:
#line 659 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                            {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value <= (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2511 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 53:
#line 666 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value > (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2523 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 54:
#line 673 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value < (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2535 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 55:
#line 680 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                          {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value >> (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2547 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 56:
#line 687 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                         {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value << (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2559 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 57:
#line 694 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value - (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2571 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 58:
#line 701 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value + (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2583 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 59:
#line 708 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		if ((yyvsp[0].expression_value).value == 0) {
			yyerror (& (yylsp[-2]), parser,
				 "zero modulus in preprocessor directive");
		} else {
			(yyval.expression_value).value = (yyvsp[-2].expression_value).value % (yyvsp[0].expression_value).value;
		}
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2600 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 60:
#line 720 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		if ((yyvsp[0].expression_value).value == 0) {
			yyerror (& (yylsp[-2]), parser,
				 "division by 0 in preprocessor directive");
		} else {
			(yyval.expression_value).value = (yyvsp[-2].expression_value).value / (yyvsp[0].expression_value).value;
		}
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2617 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 61:
#line 732 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                  {
		(yyval.expression_value).value = (yyvsp[-2].expression_value).value * (yyvsp[0].expression_value).value;
		if ((yyvsp[-2].expression_value).undefined_macro)
			(yyval.expression_value).undefined_macro = (yyvsp[-2].expression_value).undefined_macro;
                else
			(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2629 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 62:
#line 739 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                   {
		(yyval.expression_value).value = ! (yyvsp[0].expression_value).value;
		(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2638 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 63:
#line 743 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                   {
		(yyval.expression_value).value = ~ (yyvsp[0].expression_value).value;
		(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2647 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 64:
#line 747 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                   {
		(yyval.expression_value).value = - (yyvsp[0].expression_value).value;
		(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2656 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 65:
#line 751 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                   {
		(yyval.expression_value).value = + (yyvsp[0].expression_value).value;
		(yyval.expression_value).undefined_macro = (yyvsp[0].expression_value).undefined_macro;
	}
#line 2665 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 66:
#line 755 "src/compiler/glsl/glcpp/glcpp-parse.y"
                           {
		(yyval.expression_value) = (yyvsp[-1].expression_value);
	}
#line 2673 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 67:
#line 761 "src/compiler/glsl/glcpp/glcpp-parse.y"
                   {
		(yyval.string_list) = _string_list_create (parser);
		_string_list_append_item (parser, (yyval.string_list), (yyvsp[0].str));
	}
#line 2682 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 68:
#line 765 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                       {
		(yyval.string_list) = (yyvsp[-2].string_list);	
		_string_list_append_item (parser, (yyval.string_list), (yyvsp[0].str));
	}
#line 2691 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 69:
#line 772 "src/compiler/glsl/glcpp/glcpp-parse.y"
                { (yyval.token_list) = NULL; }
#line 2697 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 71:
#line 777 "src/compiler/glsl/glcpp/glcpp-parse.y"
                    { (yyval.token_list) = NULL; }
#line 2703 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 74:
#line 783 "src/compiler/glsl/glcpp/glcpp-parse.y"
                  {
		glcpp_error(&(yylsp[0]), parser, "extra tokens at end of directive");
	}
#line 2711 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 75:
#line 789 "src/compiler/glsl/glcpp/glcpp-parse.y"
                            {
		parser->space_tokens = 1;
		(yyval.token_list) = _token_list_create (parser);
		_token_list_append (parser, (yyval.token_list), (yyvsp[0].token));
	}
#line 2721 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 76:
#line 794 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                      {
		(yyval.token_list) = (yyvsp[-1].token_list);
		_token_list_append (parser, (yyval.token_list), (yyvsp[0].token));
	}
#line 2730 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 77:
#line 801 "src/compiler/glsl/glcpp/glcpp-parse.y"
                   {
		(yyval.token) = _token_create_str (parser, IDENTIFIER, (yyvsp[0].str));
		(yyval.token)->location = yylloc;
	}
#line 2739 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 78:
#line 805 "src/compiler/glsl/glcpp/glcpp-parse.y"
                       {
		(yyval.token) = _token_create_str (parser, INTEGER_STRING, (yyvsp[0].str));
		(yyval.token)->location = yylloc;
	}
#line 2748 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 79:
#line 809 "src/compiler/glsl/glcpp/glcpp-parse.y"
             {
		(yyval.token) = _token_create_str (parser, PATH, (yyvsp[0].str));
		(yyval.token)->location = yylloc;
	}
#line 2757 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 80:
#line 813 "src/compiler/glsl/glcpp/glcpp-parse.y"
                 {
		(yyval.token) = _token_create_ival (parser, (yyvsp[0].ival), (yyvsp[0].ival));
		(yyval.token)->location = yylloc;
	}
#line 2766 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 81:
#line 817 "src/compiler/glsl/glcpp/glcpp-parse.y"
                {
		(yyval.token) = _token_create_ival (parser, DEFINED, DEFINED);
		(yyval.token)->location = yylloc;
	}
#line 2775 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 82:
#line 821 "src/compiler/glsl/glcpp/glcpp-parse.y"
              {
		(yyval.token) = _token_create_str (parser, OTHER, (yyvsp[0].str));
		(yyval.token)->location = yylloc;
	}
#line 2784 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 83:
#line 825 "src/compiler/glsl/glcpp/glcpp-parse.y"
              {
		(yyval.token) = _token_create_ival (parser, SPACE, SPACE);
		(yyval.token)->location = yylloc;
	}
#line 2793 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 84:
#line 832 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '['; }
#line 2799 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 85:
#line 833 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = ']'; }
#line 2805 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 86:
#line 834 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '('; }
#line 2811 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 87:
#line 835 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = ')'; }
#line 2817 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 88:
#line 836 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '{'; }
#line 2823 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 89:
#line 837 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '}'; }
#line 2829 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 90:
#line 838 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '.'; }
#line 2835 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 91:
#line 839 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '&'; }
#line 2841 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 92:
#line 840 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '*'; }
#line 2847 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 93:
#line 841 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '+'; }
#line 2853 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 94:
#line 842 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '-'; }
#line 2859 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 95:
#line 843 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '~'; }
#line 2865 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 96:
#line 844 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '!'; }
#line 2871 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 97:
#line 845 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '/'; }
#line 2877 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 98:
#line 846 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '%'; }
#line 2883 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 99:
#line 847 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = LEFT_SHIFT; }
#line 2889 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 100:
#line 848 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = RIGHT_SHIFT; }
#line 2895 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 101:
#line 849 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '<'; }
#line 2901 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 102:
#line 850 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '>'; }
#line 2907 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 103:
#line 851 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = LESS_OR_EQUAL; }
#line 2913 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 104:
#line 852 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = GREATER_OR_EQUAL; }
#line 2919 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 105:
#line 853 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = EQUAL; }
#line 2925 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 106:
#line 854 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = NOT_EQUAL; }
#line 2931 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 107:
#line 855 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '^'; }
#line 2937 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 108:
#line 856 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '|'; }
#line 2943 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 109:
#line 857 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = AND; }
#line 2949 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 110:
#line 858 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = OR; }
#line 2955 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 111:
#line 859 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = ';'; }
#line 2961 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 112:
#line 860 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = ','; }
#line 2967 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 113:
#line 861 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = '='; }
#line 2973 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 114:
#line 862 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = PASTE; }
#line 2979 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 115:
#line 863 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = PLUS_PLUS; }
#line 2985 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;

  case 116:
#line 864 "src/compiler/glsl/glcpp/glcpp-parse.y"
                                { (yyval.ival) = MINUS_MINUS; }
#line 2991 "src/compiler/glsl/glcpp/glcpp-parse.c"
    break;


#line 2995 "src/compiler/glsl/glcpp/glcpp-parse.c"

      default: break;
    }
  /* User semantic actions sometimes alter yychar, and that requires
     that yytoken be updated with the new translation.  We take the
     approach of translating immediately before every use of yytoken.
     One alternative is translating here after every semantic action,
     but that translation would be missed if the semantic action invokes
     YYABORT, YYACCEPT, or YYERROR immediately after altering yychar or
     if it invokes YYBACKUP.  In the case of YYABORT or YYACCEPT, an
     incorrect destructor might then be invoked immediately.  In the
     case of YYERROR or YYBACKUP, subsequent parser actions might lead
     to an incorrect destructor call or verbose syntax error message
     before the lookahead is translated.  */
  YY_SYMBOL_PRINT ("-> $$ =", yyr1[yyn], &yyval, &yyloc);

  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);

  *++yyvsp = yyval;
  *++yylsp = yyloc;

  /* Now 'shift' the result of the reduction.  Determine what state
     that goes to, based on the state we popped back to and the rule
     number reduced by.  */
  {
    const int yylhs = yyr1[yyn] - YYNTOKENS;
    const int yyi = yypgoto[yylhs] + *yyssp;
    yystate = (0 <= yyi && yyi <= YYLAST && yycheck[yyi] == *yyssp
               ? yytable[yyi]
               : yydefgoto[yylhs]);
  }

  goto yynewstate;


/*--------------------------------------.
| yyerrlab -- here on detecting error.  |
`--------------------------------------*/
yyerrlab:
  /* Make sure we have latest lookahead translation.  See comments at
     user semantic actions for why this is necessary.  */
  yytoken = yychar == YYEMPTY ? YYEMPTY : YYTRANSLATE (yychar);

  /* If not already recovering from an error, report this error.  */
  if (!yyerrstatus)
    {
      ++yynerrs;
#if ! YYERROR_VERBOSE
      yyerror (&yylloc, parser, YY_("syntax error"));
#else
# define YYSYNTAX_ERROR yysyntax_error (&yymsg_alloc, &yymsg, \
                                        yyssp, yytoken)
      {
        char const *yymsgp = YY_("syntax error");
        int yysyntax_error_status;
        yysyntax_error_status = YYSYNTAX_ERROR;
        if (yysyntax_error_status == 0)
          yymsgp = yymsg;
        else if (yysyntax_error_status == 1)
          {
            if (yymsg != yymsgbuf)
              YYSTACK_FREE (yymsg);
            yymsg = YY_CAST (char *, YYSTACK_ALLOC (YY_CAST (YYSIZE_T, yymsg_alloc)));
            if (!yymsg)
              {
                yymsg = yymsgbuf;
                yymsg_alloc = sizeof yymsgbuf;
                yysyntax_error_status = 2;
              }
            else
              {
                yysyntax_error_status = YYSYNTAX_ERROR;
                yymsgp = yymsg;
              }
          }
        yyerror (&yylloc, parser, yymsgp);
        if (yysyntax_error_status == 2)
          goto yyexhaustedlab;
      }
# undef YYSYNTAX_ERROR
#endif
    }

  yyerror_range[1] = yylloc;

  if (yyerrstatus == 3)
    {
      /* If just tried and failed to reuse lookahead token after an
         error, discard it.  */

      if (yychar <= YYEOF)
        {
          /* Return failure if at end of input.  */
          if (yychar == YYEOF)
            YYABORT;
        }
      else
        {
          yydestruct ("Error: discarding",
                      yytoken, &yylval, &yylloc, parser);
          yychar = YYEMPTY;
        }
    }

  /* Else will try to reuse lookahead token after shifting the error
     token.  */
  goto yyerrlab1;


/*---------------------------------------------------.
| yyerrorlab -- error raised explicitly by YYERROR.  |
`---------------------------------------------------*/
yyerrorlab:
  /* Pacify compilers when the user code never invokes YYERROR and the
     label yyerrorlab therefore never appears in user code.  */
  if (0)
    YYERROR;

  /* Do not reclaim the symbols of the rule whose action triggered
     this YYERROR.  */
  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);
  yystate = *yyssp;
  goto yyerrlab1;


/*-------------------------------------------------------------.
| yyerrlab1 -- common code for both syntax error and YYERROR.  |
`-------------------------------------------------------------*/
yyerrlab1:
  yyerrstatus = 3;      /* Each real token shifted decrements this.  */

  for (;;)
    {
      yyn = yypact[yystate];
      if (!yypact_value_is_default (yyn))
        {
          yyn += YYTERROR;
          if (0 <= yyn && yyn <= YYLAST && yycheck[yyn] == YYTERROR)
            {
              yyn = yytable[yyn];
              if (0 < yyn)
                break;
            }
        }

      /* Pop the current state because it cannot handle the error token.  */
      if (yyssp == yyss)
        YYABORT;

      yyerror_range[1] = *yylsp;
      yydestruct ("Error: popping",
                  yystos[yystate], yyvsp, yylsp, parser);
      YYPOPSTACK (1);
      yystate = *yyssp;
      YY_STACK_PRINT (yyss, yyssp);
    }

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END

  yyerror_range[2] = yylloc;
  /* Using YYLLOC is tempting, but would change the location of
     the lookahead.  YYLOC is available though.  */
  YYLLOC_DEFAULT (yyloc, yyerror_range, 2);
  *++yylsp = yyloc;

  /* Shift the error token.  */
  YY_SYMBOL_PRINT ("Shifting", yystos[yyn], yyvsp, yylsp);

  yystate = yyn;
  goto yynewstate;


/*-------------------------------------.
| yyacceptlab -- YYACCEPT comes here.  |
`-------------------------------------*/
yyacceptlab:
  yyresult = 0;
  goto yyreturn;


/*-----------------------------------.
| yyabortlab -- YYABORT comes here.  |
`-----------------------------------*/
yyabortlab:
  yyresult = 1;
  goto yyreturn;


#if !defined yyoverflow || YYERROR_VERBOSE
/*-------------------------------------------------.
| yyexhaustedlab -- memory exhaustion comes here.  |
`-------------------------------------------------*/
yyexhaustedlab:
  yyerror (&yylloc, parser, YY_("memory exhausted"));
  yyresult = 2;
  /* Fall through.  */
#endif


/*-----------------------------------------------------.
| yyreturn -- parsing is finished, return the result.  |
`-----------------------------------------------------*/
yyreturn:
  if (yychar != YYEMPTY)
    {
      /* Make sure we have latest lookahead translation.  See comments at
         user semantic actions for why this is necessary.  */
      yytoken = YYTRANSLATE (yychar);
      yydestruct ("Cleanup: discarding lookahead",
                  yytoken, &yylval, &yylloc, parser);
    }
  /* Do not reclaim the symbols of the rule whose action triggered
     this YYABORT or YYACCEPT.  */
  YYPOPSTACK (yylen);
  YY_STACK_PRINT (yyss, yyssp);
  while (yyssp != yyss)
    {
      yydestruct ("Cleanup: popping",
                  yystos[*yyssp], yyvsp, yylsp, parser);
      YYPOPSTACK (1);
    }
#ifndef yyoverflow
  if (yyss != yyssa)
    YYSTACK_FREE (yyss);
#endif
#if YYERROR_VERBOSE
  if (yymsg != yymsgbuf)
    YYSTACK_FREE (yymsg);
#endif
  return yyresult;
}
#line 867 "src/compiler/glsl/glcpp/glcpp-parse.y"


string_list_t *
_string_list_create(glcpp_parser_t *parser)
{
   string_list_t *list;

   list = linear_alloc_child(parser->linalloc, sizeof(string_list_t));
   list->head = NULL;
   list->tail = NULL;

   return list;
}

void
_string_list_append_item(glcpp_parser_t *parser, string_list_t *list,
                         const char *str)
{
   string_node_t *node;

   node = linear_alloc_child(parser->linalloc, sizeof(string_node_t));
   node->str = linear_strdup(parser->linalloc, str);

   node->next = NULL;

   if (list->head == NULL) {
      list->head = node;
   } else {
      list->tail->next = node;
   }

   list->tail = node;
}

int
_string_list_contains(string_list_t *list, const char *member, int *index)
{
   string_node_t *node;
   int i;

   if (list == NULL)
      return 0;

   for (i = 0, node = list->head; node; i++, node = node->next) {
      if (strcmp (node->str, member) == 0) {
         if (index)
            *index = i;
         return 1;
      }
   }

   return 0;
}

/* Return duplicate string in list (if any), NULL otherwise. */
const char *
_string_list_has_duplicate(string_list_t *list)
{
   string_node_t *node, *dup;

   if (list == NULL)
      return NULL;

   for (node = list->head; node; node = node->next) {
      for (dup = node->next; dup; dup = dup->next) {
         if (strcmp (node->str, dup->str) == 0)
            return node->str;
      }
   }

   return NULL;
}

int
_string_list_length(string_list_t *list)
{
   int length = 0;
   string_node_t *node;

   if (list == NULL)
      return 0;

   for (node = list->head; node; node = node->next)
      length++;

   return length;
}

int
_string_list_equal(string_list_t *a, string_list_t *b)
{
   string_node_t *node_a, *node_b;

   if (a == NULL && b == NULL)
      return 1;

   if (a == NULL || b == NULL)
      return 0;

   for (node_a = a->head, node_b = b->head;
        node_a && node_b;
        node_a = node_a->next, node_b = node_b->next)
   {
      if (strcmp (node_a->str, node_b->str))
         return 0;
   }

   /* Catch the case of lists being different lengths, (which
    * would cause the loop above to terminate after the shorter
    * list). */
   return node_a == node_b;
}

argument_list_t *
_argument_list_create(glcpp_parser_t *parser)
{
   argument_list_t *list;

   list = linear_alloc_child(parser->linalloc, sizeof(argument_list_t));
   list->head = NULL;
   list->tail = NULL;

   return list;
}

void
_argument_list_append(glcpp_parser_t *parser,
                      argument_list_t *list, token_list_t *argument)
{
   argument_node_t *node;

   node = linear_alloc_child(parser->linalloc, sizeof(argument_node_t));
   node->argument = argument;

   node->next = NULL;

   if (list->head == NULL) {
      list->head = node;
   } else {
      list->tail->next = node;
   }

   list->tail = node;
}

int
_argument_list_length(argument_list_t *list)
{
   int length = 0;
   argument_node_t *node;

   if (list == NULL)
      return 0;

   for (node = list->head; node; node = node->next)
      length++;

   return length;
}

token_list_t *
_argument_list_member_at(argument_list_t *list, int index)
{
   argument_node_t *node;
   int i;

   if (list == NULL)
      return NULL;

   node = list->head;
   for (i = 0; i < index; i++) {
      node = node->next;
      if (node == NULL)
         break;
   }

   if (node)
      return node->argument;

   return NULL;
}

token_t *
_token_create_str(glcpp_parser_t *parser, int type, char *str)
{
   token_t *token;

   token = linear_alloc_child(parser->linalloc, sizeof(token_t));
   token->type = type;
   token->value.str = str;

   return token;
}

token_t *
_token_create_ival(glcpp_parser_t *parser, int type, int ival)
{
   token_t *token;

   token = linear_alloc_child(parser->linalloc, sizeof(token_t));
   token->type = type;
   token->value.ival = ival;

   return token;
}

token_list_t *
_token_list_create(glcpp_parser_t *parser)
{
   token_list_t *list;

   list = linear_alloc_child(parser->linalloc, sizeof(token_list_t));
   list->head = NULL;
   list->tail = NULL;
   list->non_space_tail = NULL;

   return list;
}

void
_token_list_append(glcpp_parser_t *parser, token_list_t *list, token_t *token)
{
   token_node_t *node;

   node = linear_alloc_child(parser->linalloc, sizeof(token_node_t));
   node->token = token;
   node->next = NULL;

   if (list->head == NULL) {
      list->head = node;
   } else {
      list->tail->next = node;
   }

   list->tail = node;
   if (token->type != SPACE)
      list->non_space_tail = node;
}

void
_token_list_append_list(token_list_t *list, token_list_t *tail)
{
   if (tail == NULL || tail->head == NULL)
      return;

   if (list->head == NULL) {
      list->head = tail->head;
   } else {
      list->tail->next = tail->head;
   }

   list->tail = tail->tail;
   list->non_space_tail = tail->non_space_tail;
}

static token_list_t *
_token_list_copy(glcpp_parser_t *parser, token_list_t *other)
{
   token_list_t *copy;
   token_node_t *node;

   if (other == NULL)
      return NULL;

   copy = _token_list_create (parser);
   for (node = other->head; node; node = node->next) {
      token_t *new_token = linear_alloc_child(parser->linalloc, sizeof(token_t));
      *new_token = *node->token;
      _token_list_append (parser, copy, new_token);
   }

   return copy;
}

static void
_token_list_trim_trailing_space(token_list_t *list)
{
   if (list->non_space_tail) {
      list->non_space_tail->next = NULL;
      list->tail = list->non_space_tail;
   }
}

static int
_token_list_is_empty_ignoring_space(token_list_t *l)
{
   token_node_t *n;

   if (l == NULL)
      return 1;

   n = l->head;
   while (n != NULL && n->token->type == SPACE)
      n = n->next;

   return n == NULL;
}

int
_token_list_equal_ignoring_space(token_list_t *a, token_list_t *b)
{
   token_node_t *node_a, *node_b;

   if (a == NULL || b == NULL) {
      int a_empty = _token_list_is_empty_ignoring_space(a);
      int b_empty = _token_list_is_empty_ignoring_space(b);
      return a_empty == b_empty;
   }

   node_a = a->head;
   node_b = b->head;

   while (1)
   {
      if (node_a == NULL && node_b == NULL)
         break;

      /* Ignore trailing whitespace */
      if (node_a == NULL && node_b->token->type == SPACE) {
         while (node_b && node_b->token->type == SPACE)
            node_b = node_b->next;
      }

      if (node_a == NULL && node_b == NULL)
         break;

      if (node_b == NULL && node_a->token->type == SPACE) {
         while (node_a && node_a->token->type == SPACE)
            node_a = node_a->next;
      }

      if (node_a == NULL && node_b == NULL)
         break;

      if (node_a == NULL || node_b == NULL)
         return 0;
      /* Make sure whitespace appears in the same places in both.
       * It need not be exactly the same amount of whitespace,
       * though.
       */
      if (node_a->token->type == SPACE && node_b->token->type == SPACE) {
         while (node_a && node_a->token->type == SPACE)
            node_a = node_a->next;
         while (node_b && node_b->token->type == SPACE)
            node_b = node_b->next;
         continue;
      }

      if (node_a->token->type != node_b->token->type)
         return 0;

      switch (node_a->token->type) {
      case INTEGER:
         if (node_a->token->value.ival !=  node_b->token->value.ival) {
            return 0;
         }
         break;
      case IDENTIFIER:
      case INTEGER_STRING:
      case OTHER:
         if (strcmp(node_a->token->value.str, node_b->token->value.str)) {
            return 0;
         }
         break;
      }

      node_a = node_a->next;
      node_b = node_b->next;
   }

   return 1;
}

static void
_token_print(struct _mesa_string_buffer *out, token_t *token)
{
   if (token->type < 256) {
      _mesa_string_buffer_append_char(out, token->type);
      return;
   }

   switch (token->type) {
   case INTEGER:
      _mesa_string_buffer_printf(out, "%" PRIiMAX, token->value.ival);
      break;
   case IDENTIFIER:
   case INTEGER_STRING:
   case PATH:
   case OTHER:
      _mesa_string_buffer_append(out, token->value.str);
      break;
   case SPACE:
      _mesa_string_buffer_append_char(out, ' ');
      break;
   case LEFT_SHIFT:
      _mesa_string_buffer_append(out, "<<");
      break;
   case RIGHT_SHIFT:
      _mesa_string_buffer_append(out, ">>");
      break;
   case LESS_OR_EQUAL:
      _mesa_string_buffer_append(out, "<=");
      break;
   case GREATER_OR_EQUAL:
      _mesa_string_buffer_append(out, ">=");
      break;
   case EQUAL:
      _mesa_string_buffer_append(out, "==");
      break;
   case NOT_EQUAL:
      _mesa_string_buffer_append(out, "!=");
      break;
   case AND:
      _mesa_string_buffer_append(out, "&&");
      break;
   case OR:
      _mesa_string_buffer_append(out, "||");
      break;
   case PASTE:
      _mesa_string_buffer_append(out, "##");
      break;
   case PLUS_PLUS:
      _mesa_string_buffer_append(out, "++");
      break;
   case MINUS_MINUS:
      _mesa_string_buffer_append(out, "--");
      break;
   case DEFINED:
      _mesa_string_buffer_append(out, "defined");
      break;
   case PLACEHOLDER:
      /* Nothing to print. */
      break;
   default:
      assert(!"Error: Don't know how to print token.");

      break;
   }
}

/* Return a new token formed by pasting 'token' and 'other'. Note that this
 * function may return 'token' or 'other' directly rather than allocating
 * anything new.
 *
 * Caution: Only very cursory error-checking is performed to see if
 * the final result is a valid single token. */
static token_t *
_token_paste(glcpp_parser_t *parser, token_t *token, token_t *other)
{
   token_t *combined = NULL;

   /* Pasting a placeholder onto anything makes no change. */
   if (other->type == PLACEHOLDER)
      return token;

   /* When 'token' is a placeholder, just return 'other'. */
   if (token->type == PLACEHOLDER)
      return other;

   /* A very few single-character punctuators can be combined
    * with another to form a multi-character punctuator. */
   switch (token->type) {
   case '<':
      if (other->type == '<')
         combined = _token_create_ival (parser, LEFT_SHIFT, LEFT_SHIFT);
      else if (other->type == '=')
         combined = _token_create_ival (parser, LESS_OR_EQUAL, LESS_OR_EQUAL);
      break;
   case '>':
      if (other->type == '>')
         combined = _token_create_ival (parser, RIGHT_SHIFT, RIGHT_SHIFT);
      else if (other->type == '=')
         combined = _token_create_ival (parser, GREATER_OR_EQUAL, GREATER_OR_EQUAL);
      break;
   case '=':
      if (other->type == '=')
         combined = _token_create_ival (parser, EQUAL, EQUAL);
      break;
   case '!':
      if (other->type == '=')
         combined = _token_create_ival (parser, NOT_EQUAL, NOT_EQUAL);
      break;
   case '&':
      if (other->type == '&')
         combined = _token_create_ival (parser, AND, AND);
      break;
   case '|':
      if (other->type == '|')
         combined = _token_create_ival (parser, OR, OR);
      break;
   }

   if (combined != NULL) {
      /* Inherit the location from the first token */
      combined->location = token->location;
      return combined;
   }

   /* Two string-valued (or integer) tokens can usually just be
    * mashed together. (We also handle a string followed by an
    * integer here as well.)
    *
    * There are some exceptions here. Notably, if the first token
    * is an integer (or a string representing an integer), then
    * the second token must also be an integer or must be a
    * string representing an integer that begins with a digit.
    */
   if ((token->type == IDENTIFIER || token->type == OTHER || token->type == INTEGER_STRING || token->type == INTEGER) &&
       (other->type == IDENTIFIER || other->type == OTHER || other->type == INTEGER_STRING || other->type == INTEGER))
   {
      char *str;
      int combined_type;

      /* Check that pasting onto an integer doesn't create a
       * non-integer, (that is, only digits can be
       * pasted. */
      if (token->type == INTEGER_STRING || token->type == INTEGER) {
         switch (other->type) {
         case INTEGER_STRING:
            if (other->value.str[0] < '0' || other->value.str[0] > '9')
               goto FAIL;
            break;
         case INTEGER:
            if (other->value.ival < 0)
               goto FAIL;
            break;
         default:
            goto FAIL;
         }
      }

      if (token->type == INTEGER)
         str = linear_asprintf(parser->linalloc, "%" PRIiMAX, token->value.ival);
      else
         str = linear_strdup(parser->linalloc, token->value.str);

      if (other->type == INTEGER)
         linear_asprintf_append(parser->linalloc, &str, "%" PRIiMAX, other->value.ival);
      else
         linear_strcat(parser->linalloc, &str, other->value.str);

      /* New token is same type as original token, unless we
       * started with an integer, in which case we will be
       * creating an integer-string. */
      combined_type = token->type;
      if (combined_type == INTEGER)
         combined_type = INTEGER_STRING;

      combined = _token_create_str (parser, combined_type, str);
      combined->location = token->location;
      return combined;
   }

    FAIL:
   glcpp_error (&token->location, parser, "");
   _mesa_string_buffer_append(parser->info_log, "Pasting \"");
   _token_print(parser->info_log, token);
   _mesa_string_buffer_append(parser->info_log, "\" and \"");
   _token_print(parser->info_log, other);
   _mesa_string_buffer_append(parser->info_log, "\" does not give a valid preprocessing token.\n");

   return token;
}

static void
_token_list_print(glcpp_parser_t *parser, token_list_t *list)
{
   token_node_t *node;

   if (list == NULL)
      return;

   for (node = list->head; node; node = node->next)
      _token_print(parser->output, node->token);
}

void
yyerror(YYLTYPE *locp, glcpp_parser_t *parser, const char *error)
{
   glcpp_error(locp, parser, "%s", error);
}

static void
add_builtin_define(glcpp_parser_t *parser, const char *name, int value)
{
   token_t *tok;
   token_list_t *list;

   tok = _token_create_ival (parser, INTEGER, value);

   list = _token_list_create(parser);
   _token_list_append(parser, list, tok);
   _define_object_macro(parser, NULL, name, list);
}

/* Initial output buffer size, 4096 minus ralloc() overhead. It was selected
 * to minimize total amount of allocated memory during shader-db run.
 */
#define INITIAL_PP_OUTPUT_BUF_SIZE 4048

glcpp_parser_t *
glcpp_parser_create(struct gl_context *gl_ctx,
                    glcpp_extension_iterator extensions, void *state)
{
   glcpp_parser_t *parser;

   parser = ralloc (NULL, glcpp_parser_t);

   glcpp_lex_init_extra (parser, &parser->scanner);
   parser->defines = _mesa_hash_table_create(NULL, _mesa_hash_string,
                                             _mesa_key_string_equal);
   parser->linalloc = linear_alloc_parent(parser, 0);
   parser->active = NULL;
   parser->lexing_directive = 0;
   parser->lexing_version_directive = 0;
   parser->space_tokens = 1;
   parser->last_token_was_newline = 0;
   parser->last_token_was_space = 0;
   parser->first_non_space_token_this_line = 1;
   parser->newline_as_space = 0;
   parser->in_control_line = 0;
   parser->paren_count = 0;
   parser->commented_newlines = 0;

   parser->skip_stack = NULL;
   parser->skipping = 0;

   parser->lex_from_list = NULL;
   parser->lex_from_node = NULL;

   parser->output = _mesa_string_buffer_create(parser,
                                               INITIAL_PP_OUTPUT_BUF_SIZE);
   parser->info_log = _mesa_string_buffer_create(parser,
                                                 INITIAL_PP_OUTPUT_BUF_SIZE);
   parser->error = 0;

   parser->gl_ctx = gl_ctx;
   parser->extensions = extensions;
   parser->extension_list = &gl_ctx->Extensions;
   parser->state = state;
   parser->api = gl_ctx->API;
   parser->version = 0;
   parser->version_set = false;

   parser->has_new_line_number = 0;
   parser->new_line_number = 1;
   parser->has_new_source_number = 0;
   parser->new_source_number = 0;

   parser->is_gles = false;

   return parser;
}

void
glcpp_parser_destroy(glcpp_parser_t *parser)
{
   glcpp_lex_destroy (parser->scanner);
   _mesa_hash_table_destroy(parser->defines, NULL);
   ralloc_free (parser);
}

typedef enum function_status
{
   FUNCTION_STATUS_SUCCESS,
   FUNCTION_NOT_A_FUNCTION,
   FUNCTION_UNBALANCED_PARENTHESES
} function_status_t;

/* Find a set of function-like macro arguments by looking for a
 * balanced set of parentheses.
 *
 * When called, 'node' should be the opening-parenthesis token, (or
 * perhaps preceeding SPACE tokens). Upon successful return *last will
 * be the last consumed node, (corresponding to the closing right
 * parenthesis).
 *
 * Return values:
 *
 *   FUNCTION_STATUS_SUCCESS:
 *
 *      Successfully parsed a set of function arguments.
 *
 *   FUNCTION_NOT_A_FUNCTION:
 *
 *      Macro name not followed by a '('. This is not an error, but
 *      simply that the macro name should be treated as a non-macro.
 *
 *   FUNCTION_UNBALANCED_PARENTHESES
 *
 *      Macro name is not followed by a balanced set of parentheses.
 */
static function_status_t
_arguments_parse(glcpp_parser_t *parser,
                 argument_list_t *arguments, token_node_t *node,
                 token_node_t **last)
{
   token_list_t *argument;
   int paren_count;

   node = node->next;

   /* Ignore whitespace before first parenthesis. */
   while (node && node->token->type == SPACE)
      node = node->next;

   if (node == NULL || node->token->type != '(')
      return FUNCTION_NOT_A_FUNCTION;

   node = node->next;

   argument = _token_list_create (parser);
   _argument_list_append (parser, arguments, argument);

   for (paren_count = 1; node; node = node->next) {
      if (node->token->type == '(') {
         paren_count++;
      } else if (node->token->type == ')') {
         paren_count--;
         if (paren_count == 0)
            break;
      }

      if (node->token->type == ',' && paren_count == 1) {
         _token_list_trim_trailing_space (argument);
         argument = _token_list_create (parser);
         _argument_list_append (parser, arguments, argument);
      } else {
         if (argument->head == NULL) {
            /* Don't treat initial whitespace as part of the argument. */
            if (node->token->type == SPACE)
               continue;
         }
         _token_list_append(parser, argument, node->token);
      }
   }

   if (paren_count)
      return FUNCTION_UNBALANCED_PARENTHESES;

   *last = node;

   return FUNCTION_STATUS_SUCCESS;
}

static token_list_t *
_token_list_create_with_one_ival(glcpp_parser_t *parser, int type, int ival)
{
   token_list_t *list;
   token_t *node;

   list = _token_list_create(parser);
   node = _token_create_ival(parser, type, ival);
   _token_list_append(parser, list, node);

   return list;
}

static token_list_t *
_token_list_create_with_one_space(glcpp_parser_t *parser)
{
   return _token_list_create_with_one_ival(parser, SPACE, SPACE);
}

static token_list_t *
_token_list_create_with_one_integer(glcpp_parser_t *parser, int ival)
{
   return _token_list_create_with_one_ival(parser, INTEGER, ival);
}

/* Evaluate a DEFINED token node (based on subsequent tokens in the list).
 *
 * Note: This function must only be called when "node" is a DEFINED token,
 * (and will abort with an assertion failure otherwise).
 *
 * If "node" is followed, (ignoring any SPACE tokens), by an IDENTIFIER token
 * (optionally preceded and followed by '(' and ')' tokens) then the following
 * occurs:
 *
 *   If the identifier is a defined macro, this function returns 1.
 *
 *   If the identifier is not a defined macro, this function returns 0.
 *
 *   In either case, *last will be updated to the last node in the list
 *   consumed by the evaluation, (either the token of the identifier or the
 *   token of the closing parenthesis).
 *
 * In all other cases, (such as "node is the final node of the list", or
 * "missing closing parenthesis", etc.), this function generates a
 * preprocessor error, returns -1 and *last will not be set.
 */
static int
_glcpp_parser_evaluate_defined(glcpp_parser_t *parser, token_node_t *node,
                               token_node_t **last)
{
   token_node_t *argument, *defined = node;

   assert(node->token->type == DEFINED);

   node = node->next;

   /* Ignore whitespace after DEFINED token. */
   while (node && node->token->type == SPACE)
      node = node->next;

   if (node == NULL)
      goto FAIL;

   if (node->token->type == IDENTIFIER || node->token->type == OTHER) {
      argument = node;
   } else if (node->token->type == '(') {
      node = node->next;

      /* Ignore whitespace after '(' token. */
      while (node && node->token->type == SPACE)
         node = node->next;

      if (node == NULL || (node->token->type != IDENTIFIER &&
                           node->token->type != OTHER)) {
         goto FAIL;
      }

      argument = node;

      node = node->next;

      /* Ignore whitespace after identifier, before ')' token. */
      while (node && node->token->type == SPACE)
         node = node->next;

      if (node == NULL || node->token->type != ')')
         goto FAIL;
   } else {
      goto FAIL;
   }

   *last = node;

   return _mesa_hash_table_search(parser->defines,
                                  argument->token->value.str) ? 1 : 0;

FAIL:
   glcpp_error (&defined->token->location, parser,
                "\"defined\" not followed by an identifier");
   return -1;
}

/* Evaluate all DEFINED nodes in a given list, modifying the list in place.
 */
static void
_glcpp_parser_evaluate_defined_in_list(glcpp_parser_t *parser,
                                       token_list_t *list)
{
   token_node_t *node, *node_prev, *replacement, *last = NULL;
   int value;

   if (list == NULL)
      return;

   node_prev = NULL;
   node = list->head;

   while (node) {

      if (node->token->type != DEFINED)
         goto NEXT;

      value = _glcpp_parser_evaluate_defined (parser, node, &last);
      if (value == -1)
         goto NEXT;

      replacement = linear_alloc_child(parser->linalloc, sizeof(token_node_t));
      replacement->token = _token_create_ival (parser, INTEGER, value);

      /* Splice replacement node into list, replacing from "node"
       * through "last". */
      if (node_prev)
         node_prev->next = replacement;
      else
         list->head = replacement;
      replacement->next = last->next;
      if (last == list->tail)
         list->tail = replacement;

      node = replacement;

   NEXT:
      node_prev = node;
      node = node->next;
   }
}

/* Perform macro expansion on 'list', placing the resulting tokens
 * into a new list which is initialized with a first token of type
 * 'head_token_type'. Then begin lexing from the resulting list,
 * (return to the current lexing source when this list is exhausted).
 *
 * See the documentation of _glcpp_parser_expand_token_list for a description
 * of the "mode" parameter.
 */
static void
_glcpp_parser_expand_and_lex_from(glcpp_parser_t *parser, int head_token_type,
                                  token_list_t *list, expansion_mode_t mode)
{
   token_list_t *expanded;
   token_t *token;

   expanded = _token_list_create (parser);
   token = _token_create_ival (parser, head_token_type, head_token_type);
   _token_list_append (parser, expanded, token);
   _glcpp_parser_expand_token_list (parser, list, mode);
   _token_list_append_list (expanded, list);
   glcpp_parser_lex_from (parser, expanded);
}

static void
_glcpp_parser_apply_pastes(glcpp_parser_t *parser, token_list_t *list)
{
   token_node_t *node;

   node = list->head;
   while (node) {
      token_node_t *next_non_space;

      /* Look ahead for a PASTE token, skipping space. */
      next_non_space = node->next;
      while (next_non_space && next_non_space->token->type == SPACE)
         next_non_space = next_non_space->next;

      if (next_non_space == NULL)
         break;

      if (next_non_space->token->type != PASTE) {
         node = next_non_space;
         continue;
      }

      /* Now find the next non-space token after the PASTE. */
      next_non_space = next_non_space->next;
      while (next_non_space && next_non_space->token->type == SPACE)
         next_non_space = next_non_space->next;

      if (next_non_space == NULL) {
         yyerror(&node->token->location, parser, "'##' cannot appear at either end of a macro expansion\n");
         return;
      }

      node->token = _token_paste(parser, node->token, next_non_space->token);
      node->next = next_non_space->next;
      if (next_non_space == list->tail)
         list->tail = node;
   }

   list->non_space_tail = list->tail;
}

/* This is a helper function that's essentially part of the
 * implementation of _glcpp_parser_expand_node. It shouldn't be called
 * except for by that function.
 *
 * Returns NULL if node is a simple token with no expansion, (that is,
 * although 'node' corresponds to an identifier defined as a
 * function-like macro, it is not followed with a parenthesized
 * argument list).
 *
 * Compute the complete expansion of node (which is a function-like
 * macro) and subsequent nodes which are arguments.
 *
 * Returns the token list that results from the expansion and sets
 * *last to the last node in the list that was consumed by the
 * expansion. Specifically, *last will be set as follows: as the
 * token of the closing right parenthesis.
 *
 * See the documentation of _glcpp_parser_expand_token_list for a description
 * of the "mode" parameter.
 */
static token_list_t *
_glcpp_parser_expand_function(glcpp_parser_t *parser, token_node_t *node,
                              token_node_t **last, expansion_mode_t mode)
{
   struct hash_entry *entry;
   macro_t *macro;
   const char *identifier;
   argument_list_t *arguments;
   function_status_t status;
   token_list_t *substituted;
   int parameter_index;

   identifier = node->token->value.str;

   entry = _mesa_hash_table_search(parser->defines, identifier);
   macro = entry ? entry->data : NULL;

   assert(macro->is_function);

   arguments = _argument_list_create(parser);
   status = _arguments_parse(parser, arguments, node, last);

   switch (status) {
   case FUNCTION_STATUS_SUCCESS:
      break;
   case FUNCTION_NOT_A_FUNCTION:
      return NULL;
   case FUNCTION_UNBALANCED_PARENTHESES:
      glcpp_error(&node->token->location, parser, "Macro %s call has unbalanced parentheses\n", identifier);
      return NULL;
   }

   /* Replace a macro defined as empty with a SPACE token. */
   if (macro->replacements == NULL) {
      return _token_list_create_with_one_space(parser);
   }

   if (!((_argument_list_length (arguments) ==
          _string_list_length (macro->parameters)) ||
         (_string_list_length (macro->parameters) == 0 &&
          _argument_list_length (arguments) == 1 &&
          arguments->head->argument->head == NULL))) {
      glcpp_error(&node->token->location, parser,
                  "Error: macro %s invoked with %d arguments (expected %d)\n",
                  identifier, _argument_list_length (arguments),
                  _string_list_length(macro->parameters));
      return NULL;
   }

   /* Perform argument substitution on the replacement list. */
   substituted = _token_list_create(parser);

   for (node = macro->replacements->head; node; node = node->next) {
      if (node->token->type == IDENTIFIER &&
          _string_list_contains(macro->parameters, node->token->value.str,
                                &parameter_index)) {
         token_list_t *argument;
         argument = _argument_list_member_at(arguments, parameter_index);
         /* Before substituting, we expand the argument tokens, or append a
          * placeholder token for an empty argument. */
         if (argument->head) {
            token_list_t *expanded_argument;
            expanded_argument = _token_list_copy(parser, argument);
            _glcpp_parser_expand_token_list(parser, expanded_argument, mode);
            _token_list_append_list(substituted, expanded_argument);
         } else {
            token_t *new_token;

            new_token = _token_create_ival(parser, PLACEHOLDER,
                                           PLACEHOLDER);
            _token_list_append(parser, substituted, new_token);
         }
      } else {
         _token_list_append(parser, substituted, node->token);
      }
   }

   /* After argument substitution, and before further expansion
    * below, implement token pasting. */

   _token_list_trim_trailing_space(substituted);

   _glcpp_parser_apply_pastes(parser, substituted);

   return substituted;
}

/* Compute the complete expansion of node, (and subsequent nodes after
 * 'node' in the case that 'node' is a function-like macro and
 * subsequent nodes are arguments).
 *
 * Returns NULL if node is a simple token with no expansion.
 *
 * Otherwise, returns the token list that results from the expansion
 * and sets *last to the last node in the list that was consumed by
 * the expansion. Specifically, *last will be set as follows:
 *
 *   As 'node' in the case of object-like macro expansion.
 *
 *   As the token of the closing right parenthesis in the case of
 *   function-like macro expansion.
 *
 * See the documentation of _glcpp_parser_expand_token_list for a description
 * of the "mode" parameter.
 */
static token_list_t *
_glcpp_parser_expand_node(glcpp_parser_t *parser, token_node_t *node,
                          token_node_t **last, expansion_mode_t mode,
                          int line)
{
   token_t *token = node->token;
   const char *identifier;
   struct hash_entry *entry;
   macro_t *macro;

   /* We only expand identifiers */
   if (token->type != IDENTIFIER) {
      return NULL;
   }

   *last = node;
   identifier = token->value.str;

   /* Special handling for __LINE__ and __FILE__, (not through
    * the hash table). */
   if (*identifier == '_') {
      if (strcmp(identifier, "__LINE__") == 0)
         return _token_list_create_with_one_integer(parser, line);

      if (strcmp(identifier, "__FILE__") == 0)
         return _token_list_create_with_one_integer(parser,
                                                    node->token->location.source);
   }

   /* Look up this identifier in the hash table. */
   entry = _mesa_hash_table_search(parser->defines, identifier);
   macro = entry ? entry->data : NULL;

   /* Not a macro, so no expansion needed. */
   if (macro == NULL)
      return NULL;

   /* Finally, don't expand this macro if we're already actively
    * expanding it, (to avoid infinite recursion). */
   if (_parser_active_list_contains (parser, identifier)) {
      /* We change the token type here from IDENTIFIER to OTHER to prevent any
       * future expansion of this unexpanded token. */
      char *str;
      token_list_t *expansion;
      token_t *final;

      str = linear_strdup(parser->linalloc, token->value.str);
      final = _token_create_str(parser, OTHER, str);
      expansion = _token_list_create(parser);
      _token_list_append(parser, expansion, final);
      return expansion;
   }

   if (! macro->is_function) {
      token_list_t *replacement;

      /* Replace a macro defined as empty with a SPACE token. */
      if (macro->replacements == NULL)
         return _token_list_create_with_one_space(parser);

      replacement = _token_list_copy(parser, macro->replacements);
      _glcpp_parser_apply_pastes(parser, replacement);
      return replacement;
   }

   return _glcpp_parser_expand_function(parser, node, last, mode);
}

/* Push a new identifier onto the parser's active list.
 *
 * Here, 'marker' is the token node that appears in the list after the
 * expansion of 'identifier'. That is, when the list iterator begins
 * examining 'marker', then it is time to pop this node from the
 * active stack.
 */
static void
_parser_active_list_push(glcpp_parser_t *parser, const char *identifier,
                         token_node_t *marker)
{
   active_list_t *node;

   node = linear_alloc_child(parser->linalloc, sizeof(active_list_t));
   node->identifier = linear_strdup(parser->linalloc, identifier);
   node->marker = marker;
   node->next = parser->active;

   parser->active = node;
}

static void
_parser_active_list_pop(glcpp_parser_t *parser)
{
   active_list_t *node = parser->active;

   if (node == NULL) {
      parser->active = NULL;
      return;
   }

   node = parser->active->next;
   parser->active = node;
}

static int
_parser_active_list_contains(glcpp_parser_t *parser, const char *identifier)
{
   active_list_t *node;

   if (parser->active == NULL)
      return 0;

   for (node = parser->active; node; node = node->next)
      if (strcmp(node->identifier, identifier) == 0)
         return 1;

   return 0;
}

/* Walk over the token list replacing nodes with their expansion.
 * Whenever nodes are expanded the walking will walk over the new
 * nodes, continuing to expand as necessary. The results are placed in
 * 'list' itself.
 *
 * The "mode" argument controls the handling of any DEFINED tokens that
 * result from expansion as follows:
 *
 *   EXPANSION_MODE_IGNORE_DEFINED: Any resulting DEFINED tokens will be
 *      left in the final list, unevaluated. This is the correct mode
 *      for expanding any list in any context other than a
 *      preprocessor conditional, (#if or #elif).
 *
 *   EXPANSION_MODE_EVALUATE_DEFINED: Any resulting DEFINED tokens will be
 *      evaluated to 0 or 1 tokens depending on whether the following
 *      token is the name of a defined macro. If the DEFINED token is
 *      not followed by an (optionally parenthesized) identifier, then
 *      an error will be generated. This the correct mode for
 *      expanding any list in the context of a preprocessor
 *      conditional, (#if or #elif).
 */
static void
_glcpp_parser_expand_token_list(glcpp_parser_t *parser, token_list_t *list,
                                expansion_mode_t mode)
{
   token_node_t *node_prev;
   token_node_t *node, *last = NULL;
   token_list_t *expansion;
   active_list_t *active_initial = parser->active;
   int line;

   if (list == NULL)
      return;

   _token_list_trim_trailing_space (list);

   line = list->tail->token->location.last_line;

   node_prev = NULL;
   node = list->head;

   if (mode == EXPANSION_MODE_EVALUATE_DEFINED)
      _glcpp_parser_evaluate_defined_in_list (parser, list);

   while (node) {

      while (parser->active && parser->active->marker == node)
         _parser_active_list_pop (parser);

      expansion = _glcpp_parser_expand_node (parser, node, &last, mode, line);
      if (expansion) {
         token_node_t *n;

         if (mode == EXPANSION_MODE_EVALUATE_DEFINED) {
            _glcpp_parser_evaluate_defined_in_list (parser, expansion);
         }

         for (n = node; n != last->next; n = n->next)
            while (parser->active && parser->active->marker == n) {
               _parser_active_list_pop (parser);
            }

         _parser_active_list_push(parser, node->token->value.str, last->next);

         /* Splice expansion into list, supporting a simple deletion if the
          * expansion is empty.
          */
         if (expansion->head) {
            if (node_prev)
               node_prev->next = expansion->head;
            else
               list->head = expansion->head;
            expansion->tail->next = last->next;
            if (last == list->tail)
               list->tail = expansion->tail;
         } else {
            if (node_prev)
               node_prev->next = last->next;
            else
               list->head = last->next;
            if (last == list->tail)
               list->tail = NULL;
         }
      } else {
         node_prev = node;
      }
      node = node_prev ? node_prev->next : list->head;
   }

   /* Remove any lingering effects of this invocation on the
    * active list. That is, pop until the list looks like it did
    * at the beginning of this function. */
   while (parser->active && parser->active != active_initial)
      _parser_active_list_pop (parser);

   list->non_space_tail = list->tail;
}

void
_glcpp_parser_print_expanded_token_list(glcpp_parser_t *parser,
                                        token_list_t *list)
{
   if (list == NULL)
      return;

   _glcpp_parser_expand_token_list (parser, list, EXPANSION_MODE_IGNORE_DEFINED);

   _token_list_trim_trailing_space (list);

   _token_list_print (parser, list);
}

static void
_check_for_reserved_macro_name(glcpp_parser_t *parser, YYLTYPE *loc,
                               const char *identifier)
{
   /* Section 3.3 (Preprocessor) of the GLSL 1.30 spec (and later) and
    * the GLSL ES spec (all versions) say:
    *
    *     "All macro names containing two consecutive underscores ( __ )
    *     are reserved for future use as predefined macro names. All
    *     macro names prefixed with "GL_" ("GL" followed by a single
    *     underscore) are also reserved."
    *
    * The intention is that names containing __ are reserved for internal
    * use by the implementation, and names prefixed with GL_ are reserved
    * for use by Khronos.  Since every extension adds a name prefixed
    * with GL_ (i.e., the name of the extension), that should be an
    * error.  Names simply containing __ are dangerous to use, but should
    * be allowed.
    *
    * A future version of the GLSL specification will clarify this.
    */
   if (strstr(identifier, "__")) {
      glcpp_warning(loc, parser, "Macro names containing \"__\" are reserved "
                    "for use by the implementation.\n");
   }
   if (strncmp(identifier, "GL_", 3) == 0) {
      glcpp_error (loc, parser, "Macro names starting with \"GL_\" are reserved.\n");
   }
   if (strcmp(identifier, "defined") == 0) {
      glcpp_error (loc, parser, "\"defined\" cannot be used as a macro name");
   }
}

static int
_macro_equal(macro_t *a, macro_t *b)
{
   if (a->is_function != b->is_function)
      return 0;

   if (a->is_function) {
      if (! _string_list_equal (a->parameters, b->parameters))
         return 0;
   }

   return _token_list_equal_ignoring_space(a->replacements, b->replacements);
}

void
_define_object_macro(glcpp_parser_t *parser, YYLTYPE *loc,
                     const char *identifier, token_list_t *replacements)
{
   macro_t *macro, *previous;
   struct hash_entry *entry;

   /* We define pre-defined macros before we've started parsing the actual
    * file. So if there's no location defined yet, that's what were doing and
    * we don't want to generate an error for using the reserved names. */
   if (loc != NULL)
      _check_for_reserved_macro_name(parser, loc, identifier);

   macro = linear_alloc_child(parser->linalloc, sizeof(macro_t));

   macro->is_function = 0;
   macro->parameters = NULL;
   macro->identifier = linear_strdup(parser->linalloc, identifier);
   macro->replacements = replacements;

   entry = _mesa_hash_table_search(parser->defines, identifier);
   previous = entry ? entry->data : NULL;
   if (previous) {
      if (_macro_equal (macro, previous)) {
         return;
      }
      glcpp_error (loc, parser, "Redefinition of macro %s\n",  identifier);
   }

   _mesa_hash_table_insert (parser->defines, identifier, macro);
}

void
_define_function_macro(glcpp_parser_t *parser, YYLTYPE *loc,
                       const char *identifier, string_list_t *parameters,
                       token_list_t *replacements)
{
   macro_t *macro, *previous;
   struct hash_entry *entry;
   const char *dup;

   _check_for_reserved_macro_name(parser, loc, identifier);

        /* Check for any duplicate parameter names. */
   if ((dup = _string_list_has_duplicate (parameters)) != NULL) {
      glcpp_error (loc, parser, "Duplicate macro parameter \"%s\"", dup);
   }

   macro = linear_alloc_child(parser->linalloc, sizeof(macro_t));

   macro->is_function = 1;
   macro->parameters = parameters;
   macro->identifier = linear_strdup(parser->linalloc, identifier);
   macro->replacements = replacements;

   entry = _mesa_hash_table_search(parser->defines, identifier);
   previous = entry ? entry->data : NULL;
   if (previous) {
      if (_macro_equal (macro, previous)) {
         return;
      }
      glcpp_error (loc, parser, "Redefinition of macro %s\n", identifier);
   }

   _mesa_hash_table_insert(parser->defines, identifier, macro);
}

static int
glcpp_parser_lex(YYSTYPE *yylval, YYLTYPE *yylloc, glcpp_parser_t *parser)
{
   token_node_t *node;
   int ret;

   if (parser->lex_from_list == NULL) {
      ret = glcpp_lex(yylval, yylloc, parser->scanner);

      /* XXX: This ugly block of code exists for the sole
       * purpose of converting a NEWLINE token into a SPACE
       * token, but only in the case where we have seen a
       * function-like macro name, but have not yet seen its
       * closing parenthesis.
       *
       * There's perhaps a more compact way to do this with
       * mid-rule actions in the grammar.
       *
       * I'm definitely not pleased with the complexity of
       * this code here.
       */
      if (parser->newline_as_space) {
         if (ret == '(') {
            parser->paren_count++;
         } else if (ret == ')') {
            parser->paren_count--;
            if (parser->paren_count == 0)
               parser->newline_as_space = 0;
         } else if (ret == NEWLINE) {
            ret = SPACE;
         } else if (ret != SPACE) {
            if (parser->paren_count == 0)
               parser->newline_as_space = 0;
         }
      } else if (parser->in_control_line) {
         if (ret == NEWLINE)
            parser->in_control_line = 0;
      }
      else if (ret == DEFINE_TOKEN || ret == UNDEF || ret == IF ||
               ret == IFDEF || ret == IFNDEF || ret == ELIF || ret == ELSE ||
               ret == ENDIF || ret == HASH_TOKEN) {
         parser->in_control_line = 1;
      } else if (ret == IDENTIFIER) {
         struct hash_entry *entry = _mesa_hash_table_search(parser->defines,
                                                            yylval->str);
         macro_t *macro = entry ? entry->data : NULL;
         if (macro && macro->is_function) {
            parser->newline_as_space = 1;
            parser->paren_count = 0;
         }
      }

      return ret;
   }

   node = parser->lex_from_node;

   if (node == NULL) {
      parser->lex_from_list = NULL;
      return NEWLINE;
   }

   *yylval = node->token->value;
   ret = node->token->type;

   parser->lex_from_node = node->next;

   return ret;
}

static void
glcpp_parser_lex_from(glcpp_parser_t *parser, token_list_t *list)
{
   token_node_t *node;

   assert (parser->lex_from_list == NULL);

   /* Copy list, eliminating any space tokens. */
   parser->lex_from_list = _token_list_create (parser);

   for (node = list->head; node; node = node->next) {
      if (node->token->type == SPACE)
         continue;
      _token_list_append (parser,  parser->lex_from_list, node->token);
   }

   parser->lex_from_node = parser->lex_from_list->head;

   /* It's possible the list consisted of nothing but whitespace. */
   if (parser->lex_from_node == NULL) {
      parser->lex_from_list = NULL;
   }
}

static void
_glcpp_parser_skip_stack_push_if(glcpp_parser_t *parser, YYLTYPE *loc,
                                 int condition)
{
   skip_type_t current = SKIP_NO_SKIP;
   skip_node_t *node;

   if (parser->skip_stack)
      current = parser->skip_stack->type;

   node = linear_alloc_child(parser->linalloc, sizeof(skip_node_t));
   node->loc = *loc;

   if (current == SKIP_NO_SKIP) {
      if (condition)
         node->type = SKIP_NO_SKIP;
      else
         node->type = SKIP_TO_ELSE;
   } else {
      node->type = SKIP_TO_ENDIF;
   }

   node->has_else = false;
   node->next = parser->skip_stack;
   parser->skip_stack = node;
}

static void
_glcpp_parser_skip_stack_change_if(glcpp_parser_t *parser, YYLTYPE *loc,
                                   const char *type, int condition)
{
   if (parser->skip_stack == NULL) {
      glcpp_error (loc, parser, "#%s without #if\n", type);
      return;
   }

   if (parser->skip_stack->type == SKIP_TO_ELSE) {
      if (condition)
         parser->skip_stack->type = SKIP_NO_SKIP;
   } else {
      parser->skip_stack->type = SKIP_TO_ENDIF;
   }
}

static void
_glcpp_parser_skip_stack_pop(glcpp_parser_t *parser, YYLTYPE *loc)
{
   skip_node_t *node;

   if (parser->skip_stack == NULL) {
      glcpp_error (loc, parser, "#endif without #if\n");
      return;
   }

   node = parser->skip_stack;
   parser->skip_stack = node->next;
}

static void
_glcpp_parser_handle_version_declaration(glcpp_parser_t *parser, intmax_t version,
                                         const char *identifier,
                                         bool explicitly_set)
{
   if (parser->version_set)
      return;

   parser->version = version;
   parser->version_set = true;

   add_builtin_define (parser, "__VERSION__", version);

   parser->is_gles = (version == 100) ||
                     (identifier && (strcmp(identifier, "es") == 0));
   bool is_compat = version >= 150 && identifier &&
                    strcmp(identifier, "compatibility") == 0;

   /* Add pre-defined macros. */
   if (parser->is_gles)
      add_builtin_define(parser, "GL_ES", 1);
   else if (is_compat)
      add_builtin_define(parser, "GL_compatibility_profile", 1);
   else if (version >= 150)
      add_builtin_define(parser, "GL_core_profile", 1);

   /* Currently, all ES2/ES3 implementations support highp in the
    * fragment shader, so we always define this macro in ES2/ES3.
    * If we ever get a driver that doesn't support highp, we'll
    * need to add a flag to the gl_context and check that here.
    */
   if (version >= 130 || parser->is_gles)
      add_builtin_define (parser, "GL_FRAGMENT_PRECISION_HIGH", 1);

   /* Add all the extension macros available in this context */
   if (parser->extensions)
      parser->extensions(parser->state, add_builtin_define, parser,
                         version, parser->is_gles);

   if (parser->extension_list) {
      /* If MESA_shader_integer_functions is supported, then the building
       * blocks required for the 64x64 => 64 multiply exist.  Add defines for
       * those functions so that they can be tested.
       */
      if (parser->extension_list->MESA_shader_integer_functions) {
         add_builtin_define(parser, "__have_builtin_builtin_sign64", 1);
         add_builtin_define(parser, "__have_builtin_builtin_umul64", 1);
         add_builtin_define(parser, "__have_builtin_builtin_udiv64", 1);
         add_builtin_define(parser, "__have_builtin_builtin_umod64", 1);
         add_builtin_define(parser, "__have_builtin_builtin_idiv64", 1);
         add_builtin_define(parser, "__have_builtin_builtin_imod64", 1);
      }
   }

   if (explicitly_set) {
      _mesa_string_buffer_printf(parser->output,
                                 "#version %" PRIiMAX "%s%s", version,
                                 identifier ? " " : "",
                                 identifier ? identifier : "");
   }
}

/* GLSL version if no version is explicitly specified. */
#define IMPLICIT_GLSL_VERSION 110

/* GLSL ES version if no version is explicitly specified. */
#define IMPLICIT_GLSL_ES_VERSION 100

void
glcpp_parser_resolve_implicit_version(glcpp_parser_t *parser)
{
   int language_version = parser->api == API_OPENGLES2 ?
                          IMPLICIT_GLSL_ES_VERSION : IMPLICIT_GLSL_VERSION;

   _glcpp_parser_handle_version_declaration(parser, language_version,
                                            NULL, false);
}

static void
glcpp_parser_copy_defines(const void *key, void *data, void *closure)
{
   struct define_include *di = (struct define_include *) closure;
   macro_t *macro = (macro_t *) data;

   /* If we hit an error on a previous pass, just return */
   if (di->parser->error)
      return;

   const char *identifier =  macro->identifier;
   struct hash_entry *entry = _mesa_hash_table_search(di->parser->defines,
                                                      identifier);

   macro_t *previous = entry ? entry->data : NULL;
   if (previous) {
      if (_macro_equal(macro, previous)) {
         return;
      }
      glcpp_error(di->loc, di->parser, "Redefinition of macro %s\n",
                  identifier);
   }

   _mesa_hash_table_insert(di->parser->defines, identifier, macro);
}
