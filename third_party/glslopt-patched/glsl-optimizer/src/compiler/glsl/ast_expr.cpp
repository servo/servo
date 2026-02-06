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
#include <assert.h>
#include "ast.h"

const char *
ast_expression::operator_string(enum ast_operators op)
{
   static const char *const operators[] = {
      "=",
      "+",
      "-",
      "+",
      "-",
      "*",
      "/",
      "%",
      "<<",
      ">>",
      "<",
      ">",
      "<=",
      ">=",
      "==",
      "!=",
      "&",
      "^",
      "|",
      "~",
      "&&",
      "^^",
      "||",
      "!",

      "*=",
      "/=",
      "%=",
      "+=",
      "-=",
      "<<=",
      ">>=",
      "&=",
      "^=",
      "|=",

      "?:",

      "++",
      "--",
      "++",
      "--",
      ".",
   };

   assert((unsigned int)op < sizeof(operators) / sizeof(operators[0]));

   return operators[op];
}


ast_expression_bin::ast_expression_bin(int oper, ast_expression *ex0,
                                       ast_expression *ex1) :
   ast_expression(oper, ex0, ex1, NULL)
{
   assert((oper >= ast_plus) && (oper <= ast_logic_not));
}


void
ast_expression_bin::print(void) const
{
   subexpressions[0]->print();
   printf("%s ", operator_string(oper));
   subexpressions[1]->print();
}
