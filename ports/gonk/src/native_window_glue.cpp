/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <stdarg.h>
#include <stdio.h>
#include "system/window.h"

struct GonkNativeWindow {
	ANativeWindow window;
	int (*set_usage)(struct ANativeWindow* window, int usage);
	int (*set_format)(struct ANativeWindow* window, int format);
	int (*set_transform)(struct ANativeWindow* window, int transform);
	int (*set_dimensions)(struct ANativeWindow* window, int w, int h);
	int (*api_connect)(struct ANativeWindow* window, int api);
	int (*api_disconnect)(struct ANativeWindow* window, int api);
};

// Rust doesn't support implementing variadic functions, so handle that here

extern "C" int
gnw_perform(struct ANativeWindow* window, int op, ...) {
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
