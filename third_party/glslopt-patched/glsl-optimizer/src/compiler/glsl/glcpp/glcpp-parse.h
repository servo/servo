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
