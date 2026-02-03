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

#include "ir.h"
#include "ir_visitor.h"
#include "ir_unused_structs.h"
#include "glsl_types.h"


class ir_struct_usage_visitor : public ir_hierarchical_visitor {
public:
	ir_struct_usage_visitor();
	~ir_struct_usage_visitor(void);

	virtual ir_visitor_status visit(ir_dereference_variable *);

	bool has_struct_entry(const glsl_type *t) const;

	exec_list struct_list;
	void *mem_ctx;
};

class ir_decl_removal_visitor : public ir_hierarchical_visitor {
public:
	ir_decl_removal_visitor(ir_struct_usage_visitor* used_structs)
	: used_structs(used_structs)
	{
	}

	virtual ir_visitor_status visit(ir_typedecl_statement* ir)
	{
		if (ir->type_decl->is_struct() && !used_structs->has_struct_entry(ir->type_decl))
		{
			ir->remove();
		}
		return visit_continue;
	}

	ir_struct_usage_visitor* used_structs;
};


struct struct_entry : public exec_node
{
	struct_entry(const glsl_type *type_) : type(type_) { }
	const glsl_type *type;
};


bool
ir_struct_usage_visitor::has_struct_entry(const glsl_type *t) const
{
	assert(t);
	foreach_in_list(struct_entry, entry, &this->struct_list) {
		if (entry->type == t)
			return true;
	}
	return false;
}


ir_visitor_status
ir_struct_usage_visitor::visit(ir_dereference_variable *ir)
{
	const glsl_type* t = ir->type;
	if (t->base_type == GLSL_TYPE_STRUCT)
	{
		if (!has_struct_entry (t))
		{
			struct_entry *entry = new(mem_ctx) struct_entry(t);
			this->struct_list.push_tail (entry);
		}
	}
	return visit_continue;
}

static void visit_variable (ir_instruction* ir, void* data)
{
	ir_variable* var = ir->as_variable();
	if (!var)
		return;
	ir_struct_usage_visitor* self = reinterpret_cast<ir_struct_usage_visitor*>(data);
	const glsl_type* t = var->type;
	if (t->base_type == GLSL_TYPE_ARRAY)
		t = t->fields.array; // handle array of structs case
	if (t->base_type == GLSL_TYPE_STRUCT)
	{
		if (!self->has_struct_entry (t))
		{
			struct_entry *entry = new(self->mem_ctx) struct_entry(t);
			self->struct_list.push_tail (entry);
		}
	}

}

ir_struct_usage_visitor::ir_struct_usage_visitor()
{
	this->mem_ctx = ralloc_context(NULL);
	this->struct_list.make_empty();
	this->callback_enter = visit_variable;
	this->data_enter = this;
}

ir_struct_usage_visitor::~ir_struct_usage_visitor(void)
{
	ralloc_free(mem_ctx);
}



void do_remove_unused_typedecls(exec_list* instructions)
{
	ir_struct_usage_visitor v;
	v.run (instructions);

	ir_decl_removal_visitor v2(&v);
	v2.run (instructions);
}
