/* -*- c++ -*- */
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

#include "glsl_symbol_table.h"
#include "ast.h"

class symbol_table_entry {
public:
   DECLARE_LINEAR_ALLOC_CXX_OPERATORS(symbol_table_entry);

   bool add_interface(const glsl_type *i, enum ir_variable_mode mode)
   {
      const glsl_type **dest;

      switch (mode) {
      case ir_var_uniform:
         dest = &ibu;
         break;
      case ir_var_shader_storage:
         dest = &iss;
         break;
      case ir_var_shader_in:
         dest = &ibi;
         break;
      case ir_var_shader_out:
         dest = &ibo;
         break;
      default:
         assert(!"Unsupported interface variable mode!");
         return false;
      }

      if (*dest != NULL) {
         return false;
      } else {
         *dest = i;
         return true;
      }
   }

   const glsl_type *get_interface(enum ir_variable_mode mode)
   {
      switch (mode) {
      case ir_var_uniform:
         return ibu;
      case ir_var_shader_storage:
         return iss;
      case ir_var_shader_in:
         return ibi;
      case ir_var_shader_out:
         return ibo;
      default:
         assert(!"Unsupported interface variable mode!");
         return NULL;
      }
   }

   symbol_table_entry(ir_variable *v)               :
      v(v), f(0), t(0), ibu(0), iss(0), ibi(0), ibo(0), a(0) {}
   symbol_table_entry(ir_function *f)               :
      v(0), f(f), t(0), ibu(0), iss(0), ibi(0), ibo(0), a(0) {}
   symbol_table_entry(const glsl_type *t)           :
      v(0), f(0), t(t), ibu(0), iss(0), ibi(0), ibo(0), a(0) {}
   symbol_table_entry(const glsl_type *t, enum ir_variable_mode mode) :
      v(0), f(0), t(0), ibu(0), iss(0), ibi(0), ibo(0), a(0)
   {
      assert(t->is_interface());
      add_interface(t, mode);
   }
   symbol_table_entry(const class ast_type_specifier *a):
      v(0), f(0), t(0), ibu(0), iss(0), ibi(0), ibo(0), a(a) {}

   ir_variable *v;
   ir_function *f;
   const glsl_type *t;
   const glsl_type *ibu;
   const glsl_type *iss;
   const glsl_type *ibi;
   const glsl_type *ibo;
   const class ast_type_specifier *a;
};

glsl_symbol_table::glsl_symbol_table()
{
   this->separate_function_namespace = false;
   this->table = _mesa_symbol_table_ctor();
   this->mem_ctx = ralloc_context(NULL);
   this->linalloc = linear_alloc_parent(this->mem_ctx, 0);
}

glsl_symbol_table::~glsl_symbol_table()
{
   _mesa_symbol_table_dtor(table);
   ralloc_free(mem_ctx);
}

void glsl_symbol_table::push_scope()
{
   _mesa_symbol_table_push_scope(table);
}

void glsl_symbol_table::pop_scope()
{
   _mesa_symbol_table_pop_scope(table);
}

bool glsl_symbol_table::name_declared_this_scope(const char *name)
{
   return _mesa_symbol_table_symbol_scope(table, name) == 0;
}

bool glsl_symbol_table::add_variable(ir_variable *v)
{
   assert(v->data.mode != ir_var_temporary);

   if (this->separate_function_namespace) {
      /* In 1.10, functions and variables have separate namespaces. */
      symbol_table_entry *existing = get_entry(v->name);
      if (name_declared_this_scope(v->name)) {
	 /* If there's already an existing function (not a constructor!) in
	  * the current scope, just update the existing entry to include 'v'.
	  */
	 if (existing->v == NULL && existing->t == NULL) {
	    existing->v = v;
	    return true;
	 }
      } else {
	 /* If not declared at this scope, add a new entry.  But if an existing
	  * entry includes a function, propagate that to this block - otherwise
	  * the new variable declaration would shadow the function.
	  */
	 symbol_table_entry *entry = new(linalloc) symbol_table_entry(v);
	 if (existing != NULL)
	    entry->f = existing->f;
	 int added = _mesa_symbol_table_add_symbol(table, v->name, entry);
	 assert(added == 0);
	 (void)added;
	 return true;
      }
      return false;
   }

   /* 1.20+ rules: */
   symbol_table_entry *entry = new(linalloc) symbol_table_entry(v);
   return _mesa_symbol_table_add_symbol(table, v->name, entry) == 0;
}

bool glsl_symbol_table::add_type(const char *name, const glsl_type *t)
{
   symbol_table_entry *entry = new(linalloc) symbol_table_entry(t);
   return _mesa_symbol_table_add_symbol(table, name, entry) == 0;
}

bool glsl_symbol_table::add_interface(const char *name, const glsl_type *i,
                                      enum ir_variable_mode mode)
{
   assert(i->is_interface());
   symbol_table_entry *entry = get_entry(name);
   if (entry == NULL) {
      symbol_table_entry *entry =
         new(linalloc) symbol_table_entry(i, mode);
      bool add_interface_symbol_result =
         _mesa_symbol_table_add_symbol(table, name, entry) == 0;
      assert(add_interface_symbol_result);
      return add_interface_symbol_result;
   } else {
      return entry->add_interface(i, mode);
   }
}

bool glsl_symbol_table::add_function(ir_function *f)
{
   if (this->separate_function_namespace && name_declared_this_scope(f->name)) {
      /* In 1.10, functions and variables have separate namespaces. */
      symbol_table_entry *existing = get_entry(f->name);
      if ((existing->f == NULL) && (existing->t == NULL)) {
	 existing->f = f;
	 return true;
      }
   }
   symbol_table_entry *entry = new(linalloc) symbol_table_entry(f);
   return _mesa_symbol_table_add_symbol(table, f->name, entry) == 0;
}

bool glsl_symbol_table::add_default_precision_qualifier(const char *type_name,
                                                        int precision)
{
   char *name = ralloc_asprintf(mem_ctx, "#default_precision_%s", type_name);

   ast_type_specifier *default_specifier = new(linalloc) ast_type_specifier(name);
   default_specifier->default_precision = precision;

   symbol_table_entry *entry =
      new(linalloc) symbol_table_entry(default_specifier);

   if (!get_entry(name))
      return _mesa_symbol_table_add_symbol(table, name, entry) == 0;

   return _mesa_symbol_table_replace_symbol(table, name, entry) == 0;
}

void glsl_symbol_table::add_global_function(ir_function *f)
{
   symbol_table_entry *entry = new(linalloc) symbol_table_entry(f);
   int added = _mesa_symbol_table_add_global_symbol(table, f->name, entry);
   assert(added == 0);
   (void)added;
}

ir_variable *glsl_symbol_table::get_variable(const char *name)
{
   symbol_table_entry *entry = get_entry(name);
   return entry != NULL ? entry->v : NULL;
}

const glsl_type *glsl_symbol_table::get_type(const char *name)
{
   symbol_table_entry *entry = get_entry(name);
   return entry != NULL ? entry->t : NULL;
}

const glsl_type *glsl_symbol_table::get_interface(const char *name,
                                                  enum ir_variable_mode mode)
{
   symbol_table_entry *entry = get_entry(name);
   return entry != NULL ? entry->get_interface(mode) : NULL;
}

ir_function *glsl_symbol_table::get_function(const char *name)
{
   symbol_table_entry *entry = get_entry(name);
   return entry != NULL ? entry->f : NULL;
}

int glsl_symbol_table::get_default_precision_qualifier(const char *type_name)
{
   char *name = ralloc_asprintf(mem_ctx, "#default_precision_%s", type_name);
   symbol_table_entry *entry = get_entry(name);
   if (!entry)
      return ast_precision_none;
   return entry->a->default_precision;
}

symbol_table_entry *glsl_symbol_table::get_entry(const char *name)
{
   return (symbol_table_entry *)
      _mesa_symbol_table_find_symbol(table, name);
}

void
glsl_symbol_table::disable_variable(const char *name)
{
   /* Ideally we would remove the variable's entry from the symbol table, but
    * that would be difficult.  Fortunately, since this is only used for
    * built-in variables, it won't be possible for the shader to re-introduce
    * the variable later, so all we really need to do is to make sure that
    * further attempts to access it using get_variable() will return NULL.
    */
   symbol_table_entry *entry = get_entry(name);
   if (entry != NULL) {
      entry->v = NULL;
   }
}

void
glsl_symbol_table::replace_variable(const char *name,
                                    ir_variable *v)
{
   symbol_table_entry *entry = get_entry(name);
   if (entry != NULL) {
      entry->v = v;
   }
}
