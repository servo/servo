/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include "system/window.h"
#include "utils/Atomic.h"

struct GonkNativeWindow {
	ANativeWindow window;
	volatile int32_t count;
	int (*set_usage)(struct ANativeWindow* window, int usage);
	int (*set_format)(struct ANativeWindow* window, int format);
	int (*set_transform)(struct ANativeWindow* window, int transform);
	int (*set_dimensions)(struct ANativeWindow* window, int w, int h);
	int (*api_connect)(struct ANativeWindow* window, int api);
	int (*api_disconnect)(struct ANativeWindow* window, int api);
	// XXX I would do this in rust if I knew how
	static void incRef(struct android_native_base_t* base) {
		GonkNativeWindow *gnw = (GonkNativeWindow *) base;
		android_atomic_inc(&gnw->count);
	}

	static void decRef(struct android_native_base_t* base) {
		GonkNativeWindow *gnw = (GonkNativeWindow *) base;
		const int32_t c = android_atomic_dec(&gnw->count);
		if (c <= 0)
			free(base);
	}
};

struct GonkNativeWindowBuffer {
	ANativeWindowBuffer buffer;
	volatile int32_t count;

	static void incRef(struct android_native_base_t* base) {
		GonkNativeWindowBuffer *gnw = (GonkNativeWindowBuffer *) base;
		android_atomic_inc(&gnw->count);
	}

	static void decRef(struct android_native_base_t* base) {
		GonkNativeWindowBuffer *gnw = (GonkNativeWindowBuffer *) base;
		const int32_t c = android_atomic_dec(&gnw->count);
		if (c <= 0)
			free(base);
	}
};

// Rust doesn't support implementing variadic functions, so handle that here

static int perform(struct ANativeWindow* window, int op, ...) {
	GonkNativeWindow *gnw = (GonkNativeWindow *)window;
	va_list ap;

	switch (op) {
        case NATIVE_WINDOW_SET_USAGE: {
		int usage;
		va_start(ap, op);
		usage = va_arg(ap, int);
		va_end(ap);
		return gnw->set_usage(window, usage);
        }
	case NATIVE_WINDOW_SET_BUFFERS_FORMAT: {
		int format;
		va_start(ap, op);
		format = va_arg(ap, int);
		va_end(ap);
		return gnw->set_format(window, format);
	}
	case NATIVE_WINDOW_SET_BUFFERS_TRANSFORM: {
		int transform;
		va_start(ap, op);
		transform = va_arg(ap, int);
		va_end(ap);
		return gnw->set_transform(window, transform);
	}
	case NATIVE_WINDOW_SET_BUFFERS_DIMENSIONS: {
		int w, h;
		va_start(ap, op);
		w = va_arg(ap, int);
		h = va_arg(ap, int);
		va_end(ap);
		return gnw->set_dimensions(window, w, h);
	}
	case NATIVE_WINDOW_API_CONNECT: {
		int api;
		va_start(ap, op);
		api = va_arg(ap, int);
		va_end(ap);
		return gnw->api_connect(window, api);
	}
	case NATIVE_WINDOW_API_DISCONNECT: {
		int api;
		va_start(ap, op);
		api = va_arg(ap, int);
		va_end(ap);
		return gnw->api_disconnect(window, api);
	}
	default:
		printf("Unsupported GonkNativeWindow operation! %d\n", op);
		return -1;
	}
}

extern "C" void*
alloc_native_window(uint32_t size)
{
	GonkNativeWindow *gnw = (GonkNativeWindow *)calloc(size, 1);
	ANativeWindow *window = &gnw->window;
	GonkNativeWindow::incRef(&window->common);

	window->common.incRef = GonkNativeWindow::incRef;
	window->common.decRef = GonkNativeWindow::decRef;
	window->perform = perform;
	return gnw;
}

extern "C" void*
alloc_native_buffer(uint32_t size)
{
	GonkNativeWindowBuffer *buf = (GonkNativeWindowBuffer *)calloc(size, 1);
	buf->buffer.common.incRef = GonkNativeWindowBuffer::incRef;
	buf->buffer.common.decRef = GonkNativeWindowBuffer::decRef;

	GonkNativeWindowBuffer::incRef(&buf->buffer.common);
	return buf;
}
