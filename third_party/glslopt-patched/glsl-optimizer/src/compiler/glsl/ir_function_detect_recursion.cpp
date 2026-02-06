/*
 * Copyright © 2011 Intel Corporation
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
 * \file ir_function_detect_recursion.cpp
 * Determine whether a shader contains static recursion.
 *
 * Consider the (possibly disjoint) graph of function calls in a shader.  If a
 * program contains recursion, this graph will contain a cycle.  If a function
 * is part of a cycle, it will have a caller and it will have a callee (it
 * calls another function).
 *
 * To detect recursion, the function call graph is constructed.  The graph is
 * repeatedly reduced by removing any function that either has no callees
 * (leaf functions) or has no caller.  Eventually the only functions that
 * remain will be the functions in the cycles.
 *
 * The GLSL spec is a bit wishy-washy about recursion.
 *
 * From page 39 (page 45 of the PDF) of the GLSL 1.10 spec:
 *
 *     "Behavior is undefined if recursion is used. Recursion means having any
 *     function appearing more than once at any one time in the run-time stack
 *     of function calls. That is, a function may not call itself either
 *     directly or indirectly. Compilers may give diagnostic messages when
 *     this is detectable at compile time, but not all such cases can be
 *     detected at compile time."
 *
 * From page 79 (page 85 of the PDF):
 *
 *     "22) Should recursion be supported?
 *
 *      DISCUSSION: Probably not necessary, but another example of limiting
 *      the language based on how it would directly map to hardware. One
 *      thought is that recursion would benefit ray tracing shaders. On the
 *      other hand, many recursion operations can also be implemented with the
 *      user managing the recursion through arrays. RenderMan doesn't support
 *      recursion. This could be added at a later date, if it proved to be
 *      necessary.
 *
 *      RESOLVED on September 10, 2002: Implementations are not required to
 *      support recursion.
 *
 *      CLOSED on September 10, 2002."
 *
 * From page 79 (page 85 of the PDF):
 *
 *     "56) Is it an error for an implementation to support recursion if the
 *     specification says recursion is not supported?
 *
 *     ADDED on September 10, 2002.
 *
 *     DISCUSSION: This issues is related to Issue (22). If we say that
 *     recursion (or some other piece of functionality) is not supported, is
 *     it an error for an implementation to support it? Perhaps the
 *     specification should remain silent on these kind of things so that they
 *     could be gracefully added later as an extension or as part of the
 *     standard.
 *
 *     RESOLUTION: Languages, in general, have programs that are not
 *     well-formed in ways a compiler cannot detect. Portability is only
 *     ensured for well-formed programs. Detecting recursion is an example of
 *     this. The language will say a well-formed program may not recurse, but
 *     compilers are not forced to detect that recursion may happen.
 *
 *     CLOSED: November 29, 2002."
 *
 * In GLSL 1.10 the behavior of recursion is undefined.  Compilers don't have
 * to reject shaders (at compile-time or link-time) that contain recursion.
 * Instead they could work, or crash, or kill a kitten.
 *
 * From page 44 (page 50 of the PDF) of the GLSL 1.20 spec:
 *
 *     "Recursion is not allowed, not even statically. Static recursion is
 *     present if the static function call graph of the program contains
 *     cycles."
 *
 * This langauge clears things up a bit, but it still leaves a lot of
 * questions unanswered.
 *
 *     - Is the error generated at compile-time or link-time?
 *
 *     - Is it an error to have a recursive function that is never statically
 *       called by main or any function called directly or indirectly by main?
 *       Technically speaking, such a function is not in the "static function
 *       call graph of the program" at all.
 *
 * \bug
 * If a shader has multiple cycles, this algorithm may erroneously complain
 * about functions that aren't in any cycle, but are in the part of the call
 * tree that connects them.  For example, if the call graph consists of a
 * cycle between A and B, and a cycle between D and E, and B also calls C
 * which calls D, then this algorithm will report C as a function which "has
 * static recursion" even though it is not part of any cycle.
 *
 * A better algorithm for cycle detection that doesn't have this drawback can
 * be found here:
 *
 * http://en.wikipedia.org/wiki/Tarjan%E2%80%99s_strongly_connected_components_algorithm
 *
 * \author Ian Romanick <ian.d.romanick@intel.com>
 */
#include "ir.h"
#include "glsl_parser_extras.h"
#include "linker.h"
#include "util/hash_table.h"
#include "program.h"

namespace {

struct call_node : public exec_node {
   class function *func;
};

class function {
public:
   function(ir_function_signature *sig)
      : sig(sig)
   {
      /* empty */
   }

   DECLARE_RALLOC_CXX_OPERATORS(function)

   ir_function_signature *sig;

   /** List of functions called by this function. */
   exec_list callees;

   /** List of functions that call this function. */
   exec_list callers;
};

class has_recursion_visitor : public ir_hierarchical_visitor {
public:
   has_recursion_visitor()
      : current(NULL)
   {
      progress = false;
      this->mem_ctx = ralloc_context(NULL);
      this->function_hash = _mesa_pointer_hash_table_create(NULL);
   }

   ~has_recursion_visitor()
   {
      _mesa_hash_table_destroy(this->function_hash, NULL);
      ralloc_free(this->mem_ctx);
   }

   function *get_function(ir_function_signature *sig)
   {
      function *f;
      hash_entry *entry = _mesa_hash_table_search(this->function_hash, sig);
      if (entry == NULL) {
         f = new(mem_ctx) function(sig);
         _mesa_hash_table_insert(this->function_hash, sig, f);
      } else {
         f = (function *) entry->data;
      }

      return f;
   }

   virtual ir_visitor_status visit_enter(ir_function_signature *sig)
   {
      this->current = this->get_function(sig);
      return visit_continue;
   }

   virtual ir_visitor_status visit_leave(ir_function_signature *sig)
   {
      (void) sig;
      this->current = NULL;
      return visit_continue;
   }

   virtual ir_visitor_status visit_enter(ir_call *call)
   {
      /* At global scope this->current will be NULL.  Since there is no way to
       * call global scope, it can never be part of a cycle.  Don't bother
       * adding calls from global scope to the graph.
       */
      if (this->current == NULL)
	 return visit_continue;

      function *const target = this->get_function(call->callee);

      /* Create a link from the caller to the callee.
       */
      call_node *node = new(mem_ctx) call_node;
      node->func = target;
      this->current->callees.push_tail(node);

      /* Create a link from the callee to the caller.
       */
      node = new(mem_ctx) call_node;
      node->func = this->current;
      target->callers.push_tail(node);
      return visit_continue;
   }

   function *current;
   struct hash_table *function_hash;
   void *mem_ctx;
   bool progress;
};

} /* anonymous namespace */

static void
destroy_links(exec_list *list, function *f)
{
   foreach_in_list_safe(call_node, node, list) {
      /* If this is the right function, remove it.  Note that the loop cannot
       * terminate now.  There can be multiple links to a function if it is
       * either called multiple times or calls multiple times.
       */
      if (node->func == f)
	 node->remove();
   }
}


/**
 * Remove a function if it has either no in or no out links
 */
static void
remove_unlinked_functions(const void *key, void *data, void *closure)
{
   has_recursion_visitor *visitor = (has_recursion_visitor *) closure;
   function *f = (function *) data;

   if (f->callers.is_empty() || f->callees.is_empty()) {
      while (!f->callers.is_empty()) {
         struct call_node *n = (struct call_node *) f->callers.pop_head();
         destroy_links(& n->func->callees, f);
      }

      while (!f->callees.is_empty()) {
         struct call_node *n = (struct call_node *) f->callees.pop_head();
         destroy_links(& n->func->callers, f);
      }

      hash_entry *entry = _mesa_hash_table_search(visitor->function_hash, key);
      _mesa_hash_table_remove(visitor->function_hash, entry);
      visitor->progress = true;
   }
}


static void
emit_errors_unlinked(const void *key, void *data, void *closure)
{
   struct _mesa_glsl_parse_state *state =
      (struct _mesa_glsl_parse_state *) closure;
   function *f = (function *) data;
   YYLTYPE loc;

   (void) key;

   char *proto = prototype_string(f->sig->return_type,
				  f->sig->function_name(),
				  &f->sig->parameters);

   memset(&loc, 0, sizeof(loc));
   _mesa_glsl_error(&loc, state,
		    "function `%s' has static recursion",
		    proto);
   ralloc_free(proto);
}


static void
emit_errors_linked(const void *key, void *data, void *closure)
{
   struct gl_shader_program *prog =
      (struct gl_shader_program *) closure;
   function *f = (function *) data;

   (void) key;

   char *proto = prototype_string(f->sig->return_type,
				  f->sig->function_name(),
				  &f->sig->parameters);

   linker_error(prog, "function `%s' has static recursion.\n", proto);
   ralloc_free(proto);
}


void
detect_recursion_unlinked(struct _mesa_glsl_parse_state *state,
			  exec_list *instructions)
{
   has_recursion_visitor v;

   /* Collect all of the information about which functions call which other
    * functions.
    */
   v.run(instructions);

   /* Remove from the set all of the functions that either have no caller or
    * call no other functions.  Repeat until no functions are removed.
    */
   do {
      v.progress = false;
      hash_table_call_foreach(v.function_hash, remove_unlinked_functions, & v);
   } while (v.progress);


   /* At this point any functions still in the hash must be part of a cycle.
    */
   hash_table_call_foreach(v.function_hash, emit_errors_unlinked, state);
}


void
detect_recursion_linked(struct gl_shader_program *prog,
			exec_list *instructions)
{
   has_recursion_visitor v;

   /* Collect all of the information about which functions call which other
    * functions.
    */
   v.run(instructions);

   /* Remove from the set all of the functions that either have no caller or
    * call no other functions.  Repeat until no functions are removed.
    */
   do {
      v.progress = false;
      hash_table_call_foreach(v.function_hash, remove_unlinked_functions, & v);
   } while (v.progress);


   /* At this point any functions still in the hash must be part of a cycle.
    */
   hash_table_call_foreach(v.function_hash, emit_errors_linked, prog);
}
