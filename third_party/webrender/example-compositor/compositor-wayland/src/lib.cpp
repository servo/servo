/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define UNICODE

#include <algorithm>
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <map>
#include <math.h>
#include <stdio.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>
#include <unordered_map>
#include <vector>

#include <wayland-client.h>
#include <wayland-egl.h>

#include <EGL/egl.h>
#include <EGL/eglext.h>
#include <GL/gl.h>
#include <GLES2/gl2.h>

#include "viewporter-client-protocol.h"
#include "xdg-shell-client-protocol.h"

#define UNUSED(x) (void)(x)

#define MIN(x, y) (((x) < (y)) ? (x) : (y))
#define MAX(x, y) (((x) > (y)) ? (x) : (y))

#define NUM_QUERIES 2

#define VIRTUAL_OFFSET 512 * 1024

enum SyncMode {
  None_ = 0,
  Swap = 1,
  Commit = 2,
  Flush = 3,
  Query = 4,
};

// The OS compositor representation of a picture cache tile.
struct Tile {
  uint64_t surface_id;
  int x;
  int y;

  struct wl_surface* surface;
  struct wl_subsurface* subsurface;
  struct wp_viewport* viewport;
  struct wl_egl_window* egl_window;
  EGLSurface egl_surface;
  bool is_visible;

  std::vector<EGLint> damage_rects;
};

struct TileKey {
  int x;
  int y;

  TileKey(int ax, int ay) : x(ax), y(ay) {}
};

bool operator==(const TileKey& k0, const TileKey& k1) {
  return k0.x == k1.x && k0.y == k1.y;
}

struct TileKeyHasher {
  size_t operator()(const TileKey& key) const { return key.x ^ key.y; }
};

struct Surface {
  uint64_t id;
  int tile_width;
  int tile_height;
  bool is_opaque;
  std::unordered_map<TileKey, Tile*, TileKeyHasher> tiles;
};

struct WLDisplay {
  struct wl_display* display;
  struct wl_registry* registry;
  struct wl_compositor* compositor;
  struct wl_subcompositor* subcompositor;
  struct xdg_wm_base* wm_base;
  struct wl_seat* seat;
  struct wl_pointer* pointer;
  struct wl_touch* touch;
  struct wl_keyboard* keyboard;
  struct wl_shm* shm;
  struct wl_cursor_theme* cursor_theme;
  struct wl_cursor* default_cursor;
  struct wl_surface* cursor_surface;
  struct wp_viewporter* viewporter;

  PFNEGLSWAPBUFFERSWITHDAMAGEEXTPROC swap_buffers_with_damage;
};

struct WLGeometry {
  int width, height;
};

struct WLWindow {
  WLGeometry geometry;
  bool enable_compositor;
  SyncMode sync_mode;
  bool closed;

  WLDisplay* display;
  struct wl_surface* surface;
  struct xdg_surface* xdg_surface;
  struct xdg_toplevel* xdg_toplevel;
  struct wl_callback* callback;
  struct wp_viewport* viewport;
  bool wait_for_configure;

  struct wl_egl_window* egl_window;
  EGLSurface egl_surface;

  EGLDeviceEXT eglDevice;
  EGLDisplay eglDisplay;
  EGLContext eglContext;
  EGLConfig config;

  // Maintain list of layer state between frames to avoid visual tree rebuild.
  std::vector<uint64_t> currentLayers;
  std::vector<uint64_t> prevLayers;

  // Maps WR surface IDs to each OS surface
  std::unordered_map<uint64_t, Surface> surfaces;
  std::vector<Tile*> destroyedTiles;
  std::vector<Tile*> hiddenTiles;
};

extern "C" {

static void init_wl_registry(WLWindow* window);
static void init_xdg_window(WLWindow* window);

WLWindow* com_wl_create_window(int width, int height, bool enable_compositor,
                               SyncMode sync_mode) {
  WLDisplay* display = new WLDisplay;
  WLWindow* window = new WLWindow;

  window->display = display;
  window->geometry.width = width;
  window->geometry.height = height;
  window->enable_compositor = enable_compositor;
  window->sync_mode = sync_mode;
  window->closed = false;

  display->display = wl_display_connect(NULL);
  assert(display->display);

  init_wl_registry(window);
  if (enable_compositor && !display->viewporter) {
    fprintf(stderr, "Native compositor mode requires wp_viewporter support\n");
    window->closed = true;
  }

  window->eglDisplay =
      eglGetPlatformDisplay(EGL_PLATFORM_WAYLAND_KHR, display->display, NULL);

  eglInitialize(window->eglDisplay, nullptr, nullptr);
  eglBindAPI(EGL_OPENGL_API);

  EGLint num_configs = 0;
  EGLint cfg_attribs[] = {EGL_SURFACE_TYPE,
                          EGL_WINDOW_BIT,
                          EGL_RENDERABLE_TYPE,
                          EGL_OPENGL_BIT,
                          EGL_RED_SIZE,
                          8,
                          EGL_GREEN_SIZE,
                          8,
                          EGL_BLUE_SIZE,
                          8,
                          EGL_ALPHA_SIZE,
                          8,
                          EGL_DEPTH_SIZE,
                          24,
                          EGL_NONE};
  EGLConfig configs[32];

  eglChooseConfig(window->eglDisplay, cfg_attribs, configs,
                  sizeof(configs) / sizeof(EGLConfig), &num_configs);
  assert(num_configs > 0);
  window->config = configs[0];

  EGLint ctx_attribs[] = {EGL_CONTEXT_OPENGL_PROFILE_MASK,
                          EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT,
                          EGL_CONTEXT_MAJOR_VERSION,
                          3,
                          EGL_CONTEXT_MINOR_VERSION,
                          2,
                          EGL_NONE};

  // Create an EGL context that can be used for drawing
  window->eglContext = eglCreateContext(window->eglDisplay, window->config,
                                        EGL_NO_CONTEXT, ctx_attribs);

  window->surface = wl_compositor_create_surface(display->compositor);
  init_xdg_window(window);

  struct wl_region* region =
      wl_compositor_create_region(window->display->compositor);
  wl_region_add(region, 0, 0, INT32_MAX, INT32_MAX);
  wl_surface_set_opaque_region(window->surface, region);
  wl_region_destroy(region);

  if (enable_compositor) {
    xdg_toplevel_set_title(window->xdg_toplevel,
                           "example-compositor (Wayland)");
  } else {
    xdg_toplevel_set_title(window->xdg_toplevel, "example-compositor (Simple)");
  }

  window->wait_for_configure = true;
  wl_surface_commit(window->surface);

  EGLBoolean ok = eglMakeCurrent(window->eglDisplay, EGL_NO_SURFACE,
                                 EGL_NO_SURFACE, window->eglContext);
  assert(ok);

  display->swap_buffers_with_damage =
      (PFNEGLSWAPBUFFERSWITHDAMAGEEXTPROC)eglGetProcAddress(
          "eglSwapBuffersWithDamageKHR");

  return window;
}

bool com_wl_tick(WLWindow* window) {
  if (window->wait_for_configure) {
    int ret = 0;
    while (window->wait_for_configure && !window->closed && ret != -1) {
      wl_display_dispatch(window->display->display);
    }
  } else {
    wl_display_dispatch_pending(window->display->display);
  }

  return !window->closed;
}

static void unmap_hidden_tiles(WLWindow* window) {
  for (Tile* tile : window->hiddenTiles) {
    if (tile->subsurface) {
      wl_subsurface_destroy(tile->subsurface);
      tile->subsurface = nullptr;
    }
  }
  window->hiddenTiles.clear();
}

static void clean_up_tiles(WLWindow* window) {
  for (Tile* tile : window->destroyedTiles) {
    eglDestroySurface(window->eglDisplay, tile->egl_surface);
    wl_egl_window_destroy(tile->egl_window);
    wp_viewport_destroy(tile->viewport);
    wl_surface_destroy(tile->surface);
    delete tile;
  }
  window->destroyedTiles.clear();
}

static void handle_callback(void* data, struct wl_callback* callback,
                            uint32_t time) {
  WLWindow* window = (WLWindow*)data;
  UNUSED(time);

  assert(window->callback == callback);

  wl_callback_destroy(callback);
  window->callback = nullptr;
}

static const struct wl_callback_listener frame_listener = {handle_callback};

void com_wl_swap_buffers(WLWindow* window) {
  if (window->enable_compositor) {
    for (auto surface_it = window->surfaces.begin();
         surface_it != window->surfaces.end(); ++surface_it) {
      Surface* surface = &surface_it->second;

      for (auto tile_it = surface->tiles.begin();
           tile_it != surface->tiles.end(); ++tile_it) {
        Tile* tile = tile_it->second;

        if (!tile->damage_rects.empty() && tile->is_visible) {
          eglMakeCurrent(window->eglDisplay, tile->egl_surface,
                         tile->egl_surface, window->eglContext);
          eglSwapInterval(window->eglDisplay, 0);

          /* if (window->display->swap_buffers_with_damage) {
            window->display->swap_buffers_with_damage(
                window->eglDisplay, tile->egl_surface,
                tile->damage_rects.data(), tile->damage_rects.size() / 4);
          } else */
          eglSwapBuffers(window->eglDisplay, tile->egl_surface);
          tile->damage_rects.clear();

          eglMakeCurrent(window->eglDisplay, EGL_NO_SURFACE, EGL_NO_SURFACE,
                         window->eglContext);
        } else {
          wl_surface_commit(tile->surface);
        }
      }
    }
    wl_surface_commit(window->surface);
    unmap_hidden_tiles(window);
    clean_up_tiles(window);

    int ret = 0;
    switch (window->sync_mode) {
      case SyncMode::None_:
        wl_display_roundtrip(window->display->display);
        break;
      case SyncMode::Swap:
        window->callback = wl_surface_frame(window->surface);
        wl_callback_add_listener(window->callback, &frame_listener, window);
        wl_surface_commit(window->surface);

        while (window->callback && !window->closed && ret != -1) {
          ret = wl_display_dispatch(window->display->display);
        }
        break;
      default:
        assert(false);
        break;
    }
  } else {
    // If not using native mode, then do a normal EGL swap buffers.
    switch (window->sync_mode) {
      case SyncMode::None_:
        eglSwapInterval(window->eglDisplay, 0);
        break;
      case SyncMode::Swap:
        eglSwapInterval(window->eglDisplay, 1);
        break;
      default:
        assert(false);
        break;
    }
    eglSwapBuffers(window->eglDisplay, window->egl_surface);
  }
}

// Create a new native surface
void com_wl_create_surface(WLWindow* window, uint64_t surface_id,
                           int tile_width, int tile_height, bool is_opaque) {
  assert(window->surfaces.count(surface_id) == 0);

  Surface surface;
  surface.id = surface_id;
  surface.tile_width = tile_width;
  surface.tile_height = tile_height;
  surface.is_opaque = is_opaque;

  window->surfaces.emplace(surface_id, surface);
}

void com_wl_create_tile(WLWindow* window, uint64_t surface_id, int x, int y) {
  WLDisplay* display = window->display;

  assert(window->surfaces.count(surface_id) == 1);
  Surface* surface = &window->surfaces.at(surface_id);

  TileKey key(x, y);
  assert(surface->tiles.count(key) == 0);

  Tile* tile = new Tile;
  tile->surface_id = surface_id;
  tile->x = x;
  tile->y = y;
  tile->is_visible = false;

  tile->surface = wl_compositor_create_surface(display->compositor);
  tile->viewport =
      wp_viewporter_get_viewport(display->viewporter, tile->surface);

  if (surface->is_opaque) {
    struct wl_region* region =
        wl_compositor_create_region(window->display->compositor);
    wl_region_add(region, 0, 0, INT32_MAX, INT32_MAX);
    wl_surface_set_opaque_region(tile->surface, region);
    wl_region_destroy(region);
  }

  tile->egl_window = wl_egl_window_create(tile->surface, surface->tile_width,
                                          surface->tile_height);
  tile->egl_surface = eglCreateWindowSurface(window->eglDisplay, window->config,
                                             tile->egl_window, NULL);
  assert(tile->egl_surface != EGL_NO_SURFACE);

  surface->tiles.emplace(key, tile);
}

static void show_tile(WLWindow* window, Tile* tile) {
  if (tile->is_visible) {
    assert(tile->subsurface);
    return;
  }

  tile->subsurface = wl_subcompositor_get_subsurface(
      window->display->subcompositor, tile->surface, window->surface);

  /* This is not comprehensive yet, see hide_tile() */
  Surface* surface = &window->surfaces.at(tile->surface_id);
  for (auto tile_it = surface->tiles.begin(); tile_it != surface->tiles.end();
       ++tile_it) {
    Tile* other_tile = tile_it->second;

    if (other_tile->is_visible) {
      wl_subsurface_place_above(tile->subsurface, other_tile->surface);
    }
  }

  tile->is_visible = true;
}

static void hide_tile(WLWindow* window, Tile* tile) {
  if (!tile->is_visible) {
    return;
  }

  /*
   * This is a workaround for missing API on the egl-wayland platform. We
   * likely want to replace it a solution that detaches the buffer from
   * the surface, which would require us to manage buffers manually.
   */
  wl_subsurface_set_position(tile->subsurface, window->geometry.width / 2,
                             window->geometry.height / 2);
  wp_viewport_set_source(tile->viewport, wl_fixed_from_int(0),
                         wl_fixed_from_int(0), wl_fixed_from_int(1),
                         wl_fixed_from_int(1));
  wl_subsurface_place_below(tile->subsurface, window->surface);
  tile->is_visible = false;
  window->hiddenTiles.push_back(tile);
}

void com_wl_destroy_tile(WLWindow* window, uint64_t surface_id, int x, int y) {
  assert(window->surfaces.count(surface_id) == 1);

  Surface* surface = &window->surfaces.at(surface_id);
  TileKey key(x, y);
  assert(surface->tiles.count(key) == 1);
  Tile* tile = surface->tiles[key];

  hide_tile(window, tile);
  wl_surface_commit(tile->surface);

  window->destroyedTiles.push_back(tile);
  surface->tiles.erase(key);
}

void com_wl_destroy_surface(WLWindow* window, uint64_t surface_id) {
  assert(window->surfaces.count(surface_id) == 1);

  Surface* surface = &window->surfaces.at(surface_id);
  for (auto tile_it = surface->tiles.begin(); tile_it != surface->tiles.end();
       tile_it = surface->tiles.begin()) {
    Tile* tile = tile_it->second;

    com_wl_destroy_tile(window, surface_id, tile->x, tile->y);
  }

  window->surfaces.erase(surface_id);
}

void com_wl_destroy_window(WLWindow* window) {
  for (auto surface_it = window->surfaces.begin();
       surface_it != window->surfaces.end(); ++surface_it) {
    Surface& surface = surface_it->second;

    com_wl_destroy_surface(window, surface.id);
  }

  if (window->egl_surface != EGL_NO_SURFACE) {
    eglDestroySurface(window->eglDisplay, window->egl_surface);
  }
  eglDestroyContext(window->eglDisplay, window->eglContext);
  eglTerminate(window->eglDisplay);

  delete window;
}

// Bind a native surface to allow issuing GL commands to it
GLuint com_wl_bind_surface(WLWindow* window, uint64_t surface_id, int tile_x,
                           int tile_y, int* x_offset, int* y_offset,
                           int dirty_x0, int dirty_y0, int dirty_width,
                           int dirty_height) {
  *x_offset = 0;
  *y_offset = 0;

  assert(window->surfaces.count(surface_id) == 1);
  Surface* surface = &window->surfaces[surface_id];

  TileKey key(tile_x, tile_y);
  assert(surface->tiles.count(key) == 1);
  Tile* tile = surface->tiles[key];

  tile->damage_rects.push_back(dirty_x0);
  tile->damage_rects.push_back(dirty_y0);
  tile->damage_rects.push_back(dirty_width);
  tile->damage_rects.push_back(dirty_height);

  EGLBoolean ok = eglMakeCurrent(window->eglDisplay, tile->egl_surface,
                                 tile->egl_surface, window->eglContext);
  assert(ok);

  return 0;
}

// Unbind a currently bound native surface
void com_wl_unbind_surface(WLWindow* window) {
  eglMakeCurrent(window->eglDisplay, EGL_NO_SURFACE, EGL_NO_SURFACE,
                 window->eglContext);
}

void com_wl_begin_transaction(WLWindow*) {}

// Add a native surface to the visual tree. Called per-frame to build the
// composition.
void com_wl_add_surface(WLWindow* window, uint64_t surface_id, int offset_x,
                        int offset_y, int clip_x, int clip_y, int clip_w,
                        int clip_h) {
  Surface* surface = &window->surfaces[surface_id];
  window->currentLayers.push_back(surface_id);

  for (auto tile_it = surface->tiles.begin(); tile_it != surface->tiles.end();
       ++tile_it) {
    Tile* tile = tile_it->second;

    int pos_x = MAX((tile->x * surface->tile_width) + offset_x, clip_x);
    int pos_y = MAX((tile->y * surface->tile_height) + offset_y, clip_y);

    float view_x = MAX((clip_x - offset_x) - tile->x * surface->tile_width, 0);
    float view_y = MAX((clip_y - offset_y) - tile->y * surface->tile_height, 0);

    float view_w = MIN(surface->tile_width - view_x, (clip_x + clip_w) - pos_x);
    float view_h =
        MIN(surface->tile_height - view_y, (clip_y + clip_h) - pos_y);
    view_w = MIN(window->geometry.width - pos_x, view_w);
    view_h = MIN(window->geometry.height - pos_y, view_h);

    if (view_w > 0 && view_h > 0) {
      show_tile(window, tile);

      wl_surface_set_buffer_transform(tile->surface,
                                      WL_OUTPUT_TRANSFORM_FLIPPED_180);
      wl_subsurface_set_position(tile->subsurface, pos_x, pos_y);
      wp_viewport_set_source(tile->viewport, wl_fixed_from_double(view_x),
                             wl_fixed_from_double(view_y),
                             wl_fixed_from_double(view_w),
                             wl_fixed_from_double(view_h));
    } else {
      hide_tile(window, tile);
    }
  }
}

void com_wl_end_transaction(WLWindow* window) {
  bool same = window->prevLayers == window->currentLayers;
  if (!same) {
    struct wl_surface* prev_surface = window->surface;

    for (auto it = window->currentLayers.begin();
         it != window->currentLayers.end(); ++it) {
      Surface* surface = &window->surfaces[*it];

      struct wl_surface* next_surface = nullptr;
      for (auto tile_it = surface->tiles.begin();
           tile_it != surface->tiles.end(); ++tile_it) {
        Tile* tile = tile_it->second;

        if (tile->is_visible) {
          wl_subsurface_place_above(tile->subsurface, prev_surface);

          if (!next_surface) {
            next_surface = tile->surface;
          }
        }
      }
      prev_surface = next_surface;
    }
  }

  window->prevLayers.swap(window->currentLayers);
  window->currentLayers.clear();
}

void glInvalidateFramebuffer(GLenum target, GLsizei numAttachments,
                             const GLenum* attachments) {
  UNUSED(target);
  UNUSED(numAttachments);
  UNUSED(attachments);
}

// Get a pointer to an EGL symbol
void* com_wl_get_proc_address(const char* name) {
  /* Disable glInvalidateFramebuffer for now as it triggers errors.
   * This is likely due to the egl-wayland platform, which we may want to
   * replace with a custom implementation in order to have more control
   * over the low-lever bits.
   */
  if (strcmp(name, "glInvalidateFramebuffer") == 0) {
    return (void*)glInvalidateFramebuffer;
  }

  return (void*)eglGetProcAddress(name);
}

void com_wl_deinit(WLWindow* window) { UNUSED(window); }

static void handle_xdg_surface_configure(void* data,
                                         struct xdg_surface* surface,
                                         uint32_t serial) {
  WLWindow* window = (WLWindow*)data;

  xdg_surface_ack_configure(surface, serial);

  if (window->wait_for_configure) {
    if (window->enable_compositor) {
      int width = window->geometry.width;
      int height = window->geometry.height;

      window->egl_window = wl_egl_window_create(window->surface, 1, 1);
      window->egl_surface = eglCreateWindowSurface(
          window->eglDisplay, window->config, window->egl_window, NULL);
      assert(window->egl_surface != EGL_NO_SURFACE);

      EGLBoolean ok = eglMakeCurrent(window->eglDisplay, window->egl_surface,
                                     window->egl_surface, window->eglContext);
      assert(ok);

      glClearColor(1.0, 1.0, 1.0, 1.0);
      glClear(GL_COLOR_BUFFER_BIT);

      window->viewport = wp_viewporter_get_viewport(window->display->viewporter,
                                                    window->surface);
      wp_viewport_set_destination(window->viewport, width, height);

      eglSwapBuffers(window->eglDisplay, window->egl_surface);
    } else {
      window->egl_window = wl_egl_window_create(
          window->surface, window->geometry.width, window->geometry.height);
      window->egl_surface = eglCreateWindowSurface(
          window->eglDisplay, window->config, window->egl_window, NULL);
      assert(window->egl_surface != EGL_NO_SURFACE);

      EGLBoolean ok = eglMakeCurrent(window->eglDisplay, window->egl_surface,
                                     window->egl_surface, window->eglContext);
      assert(ok);
    }
  }

  window->wait_for_configure = false;
}

static const struct xdg_surface_listener xdg_surface_listener = {
    handle_xdg_surface_configure};

static void handle_xdg_toplevel_configure(void* data,
                                          struct xdg_toplevel* toplevel,
                                          int32_t width, int32_t height,
                                          struct wl_array* states) {
  WLWindow* window = (WLWindow*)data;
  UNUSED(toplevel);
  UNUSED(states);

  if (width > 0 && height > 0) {
    window->geometry.width = width;
    window->geometry.height = height;

    if (!window->wait_for_configure) {
      if (window->enable_compositor) {
        wp_viewport_set_destination(window->viewport, window->geometry.width,
                                    window->geometry.height);
      } else {
        wl_egl_window_resize(window->egl_window, window->geometry.width,
                             window->geometry.height, 0, 0);
      }
    }
  }
}

static void handle_xdg_toplevel_close(void* data,
                                      struct xdg_toplevel* toplevel) {
  UNUSED(toplevel);
  WLWindow* window = (WLWindow*)data;
  window->closed = true;
}

static const struct xdg_toplevel_listener xdg_toplevel_listener = {
    handle_xdg_toplevel_configure,
    handle_xdg_toplevel_close,
};

static void xdg_wm_base_ping(void* data, struct xdg_wm_base* shell,
                             uint32_t serial) {
  UNUSED(data);
  xdg_wm_base_pong(shell, serial);
}

static const struct xdg_wm_base_listener wm_base_listener = {
    xdg_wm_base_ping,
};

static void registry_handle_global(void* data, struct wl_registry* registry,
                                   uint32_t name, const char* interface,
                                   uint32_t version) {
  WLDisplay* d = (WLDisplay*)data;

  if (strcmp(interface, "wl_compositor") == 0) {
    d->compositor = (struct wl_compositor*)wl_registry_bind(
        registry, name, &wl_compositor_interface, MIN(version, 4));
  } else if (strcmp(interface, "wp_viewporter") == 0) {
    d->viewporter = (struct wp_viewporter*)wl_registry_bind(
        registry, name, &wp_viewporter_interface, 1);
  } else if (strcmp(interface, "xdg_wm_base") == 0) {
    d->wm_base = (struct xdg_wm_base*)wl_registry_bind(
        registry, name, &xdg_wm_base_interface, 1);
    xdg_wm_base_add_listener(d->wm_base, &wm_base_listener, NULL);
  } else if (strcmp(interface, "wl_subcompositor") == 0) {
    d->subcompositor = (struct wl_subcompositor*)wl_registry_bind(
        registry, name, &wl_subcompositor_interface, 1);
  }
}

static void registry_handle_global_remove(void* data,
                                          struct wl_registry* registry,
                                          uint32_t name) {
  UNUSED(data);
  UNUSED(registry);
  UNUSED(name);
}

static const struct wl_registry_listener registry_listener = {
    registry_handle_global, registry_handle_global_remove};

static void init_wl_registry(WLWindow* window) {
  WLDisplay* display = window->display;

  display->registry = wl_display_get_registry(display->display);
  wl_registry_add_listener(display->registry, &registry_listener, display);

  wl_display_roundtrip(display->display);

  assert(display->compositor);
  assert(display->wm_base);
  assert(display->subcompositor);
}

static void init_xdg_window(WLWindow* window) {
  window->xdg_surface =
      xdg_wm_base_get_xdg_surface(window->display->wm_base, window->surface);
  assert(window->xdg_surface);
  xdg_surface_add_listener(window->xdg_surface, &xdg_surface_listener, window);

  window->xdg_toplevel = xdg_surface_get_toplevel(window->xdg_surface);
  xdg_toplevel_add_listener(window->xdg_toplevel, &xdg_toplevel_listener,
                            window);
  assert(window->xdg_toplevel);
}
}
