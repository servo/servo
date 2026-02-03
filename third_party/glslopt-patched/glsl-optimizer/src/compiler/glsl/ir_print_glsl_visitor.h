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

#pragma once
#ifndef IR_PRINT_GLSL_VISITOR_H
#define IR_PRINT_GLSL_VISITOR_H

#include "ir.h"

enum PrintGlslMode {
	kPrintGlslNone = 0,
	kPrintGlslVertex,
	kPrintGlslFragment,
};

extern char* _mesa_print_ir_glsl(exec_list *instructions,
			struct _mesa_glsl_parse_state *state,
			char* buf, PrintGlslMode mode);



class string_buffer
{
public:
	string_buffer(void* mem_ctx)
	{
		m_Capacity = 512;
		m_Ptr = (char*)ralloc_size(mem_ctx, m_Capacity);
		m_Size = 0;
		m_Ptr[0] = 0;
	}
	
	~string_buffer()
	{
		ralloc_free(m_Ptr);
	}
	
	bool empty() const { return m_Size == 0; }
	
	const char* c_str() const { return m_Ptr; }
	
	void asprintf_append(const char *fmt, ...) PRINTFLIKE(2, 3)
	{
		va_list args;
		va_start(args, fmt);
		vasprintf_append(fmt, args);
		va_end(args);
	}
	
	void vasprintf_append(const char *fmt, va_list args)
	{
		assert (m_Ptr != NULL);
		vasprintf_rewrite_tail (&m_Size, fmt, args);
	}
	
	void vasprintf_rewrite_tail (size_t *start, const char *fmt, va_list args)
	{
		assert (m_Ptr != NULL);
		
		size_t new_length = printf_length(fmt, args);
		size_t needed_length = m_Size + new_length + 1;
		
		if (m_Capacity < needed_length)
		{
			m_Capacity = MAX2 (m_Capacity + m_Capacity/2, needed_length);
			m_Ptr = (char*)reralloc_size(ralloc_parent(m_Ptr), m_Ptr, m_Capacity);
		}
		
		vsnprintf(m_Ptr + m_Size, new_length+1, fmt, args);
		m_Size += new_length;
		assert (m_Capacity >= m_Size);
	}
	
private:
	char* m_Ptr;
	size_t m_Size;
	size_t m_Capacity;
};


extern void print_float (string_buffer& buffer, float f);


#endif /* IR_PRINT_GLSL_VISITOR_H */
