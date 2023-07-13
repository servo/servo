/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#define UNICODE

#include <windows.h>
#include <math.h>
#include <dcomp.h>
#include <d3d11.h>
#include <assert.h>
#include <map>
#include <vector>
#include <dwmapi.h>
#include <unordered_map>

#define EGL_EGL_PROTOTYPES 1
#define EGL_EGLEXT_PROTOTYPES 1
#define GL_GLEXT_PROTOTYPES 1
#include "EGL/egl.h"
#include "EGL/eglext.h"
#include "EGL/eglext_angle.h"
#include "GL/gl.h"
#include "GLES/gl.h"
#include "GLES/glext.h"
#include "GLES3/gl3.h"

#define NUM_QUERIES 2

#define USE_VIRTUAL_SURFACES
#define VIRTUAL_OFFSET 512 * 1024

enum SyncMode {
  None = 0,
  Swap = 1,
  Commit = 2,
  Flush = 3,
  Query = 4,
};

// The OS compositor representation of a picture cache tile.
struct Tile {
#ifndef USE_VIRTUAL_SURFACES
  // Represents the underlying DirectComposition surface texture that gets drawn
  // into.
  IDCompositionSurface* pSurface;
  // Represents the node in the visual tree that defines the properties of this
  // tile (clip, position etc).
  IDCompositionVisual2* pVisual;
#endif
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
  int tile_width;
  int tile_height;
  bool is_opaque;
  std::unordered_map<TileKey, Tile, TileKeyHasher> tiles;
  IDCompositionVisual2* pVisual;
#ifdef USE_VIRTUAL_SURFACES
  IDCompositionVirtualSurface* pVirtualSurface;
#endif
};

struct CachedFrameBuffer {
  int width;
  int height;
  GLuint fboId;
  GLuint depthRboId;
};

struct Window {
  // Win32 window details
  HWND hWnd;
  HINSTANCE hInstance;
  bool enable_compositor;
  RECT client_rect;
  SyncMode sync_mode;

  // Main interfaces to D3D11 and DirectComposition
  ID3D11Device* pD3D11Device;
  IDCompositionDesktopDevice* pDCompDevice;
  IDCompositionTarget* pDCompTarget;
  IDXGIDevice* pDXGIDevice;
  ID3D11Query* pQueries[NUM_QUERIES];
  int current_query;

  // ANGLE interfaces that wrap the D3D device
  EGLDeviceEXT EGLDevice;
  EGLDisplay EGLDisplay;
  EGLContext EGLContext;
  EGLConfig config;
  // Framebuffer surface for debug mode when we are not using DC
  EGLSurface fb_surface;

  // The currently bound surface, valid during bind() and unbind()
  IDCompositionSurface* pCurrentSurface;
  EGLImage mEGLImage;
  GLuint mColorRBO;

  // The root of the DC visual tree. Nothing is drawn on this, but
  // all child tiles are parented to here.
  IDCompositionVisual2* pRoot;
  IDCompositionVisualDebug* pVisualDebug;
  std::vector<CachedFrameBuffer> mFrameBuffers;

  // Maintain list of layer state between frames to avoid visual tree rebuild.
  std::vector<uint64_t> mCurrentLayers;
  std::vector<uint64_t> mPrevLayers;

  // Maps WR surface IDs to each OS surface
  std::unordered_map<uint64_t, Surface> surfaces;
};

static const wchar_t* CLASS_NAME = L"WR DirectComposite";

static GLuint GetOrCreateFbo(Window* window, int aWidth, int aHeight) {
  GLuint fboId = 0;

  // Check if we have a cached FBO with matching dimensions
  for (auto it = window->mFrameBuffers.begin();
       it != window->mFrameBuffers.end(); ++it) {
    if (it->width == aWidth && it->height == aHeight) {
      fboId = it->fboId;
      break;
    }
  }

  // If not, create a new FBO with attached depth buffer
  if (fboId == 0) {
    // Create the depth buffer
    GLuint depthRboId;
    glGenRenderbuffers(1, &depthRboId);
    glBindRenderbuffer(GL_RENDERBUFFER, depthRboId);
    glRenderbufferStorage(GL_RENDERBUFFER, GL_DEPTH_COMPONENT24, aWidth,
                          aHeight);

    // Create the framebuffer and attach the depth buffer to it
    glGenFramebuffers(1, &fboId);
    glBindFramebuffer(GL_DRAW_FRAMEBUFFER, fboId);
    glFramebufferRenderbuffer(GL_DRAW_FRAMEBUFFER, GL_DEPTH_ATTACHMENT,
                              GL_RENDERBUFFER, depthRboId);

    // Store this in the cache for future calls.
    CachedFrameBuffer frame_buffer_info;
    frame_buffer_info.width = aWidth;
    frame_buffer_info.height = aHeight;
    frame_buffer_info.fboId = fboId;
    frame_buffer_info.depthRboId = depthRboId;
    window->mFrameBuffers.push_back(frame_buffer_info);
  }

  return fboId;
}

static LRESULT CALLBACK WndProc(HWND hwnd, UINT message, WPARAM wParam,
                                LPARAM lParam) {
  switch (message) {
    case WM_DESTROY:
      PostQuitMessage(0);
      return 1;
  }

  return DefWindowProc(hwnd, message, wParam, lParam);
}

extern "C" {
Window* com_dc_create_window(int width, int height, bool enable_compositor,
                             SyncMode sync_mode) {
  // Create a simple Win32 window
  Window* window = new Window;
  window->hInstance = GetModuleHandle(NULL);
  window->enable_compositor = enable_compositor;
  window->mEGLImage = EGL_NO_IMAGE;
  window->sync_mode = sync_mode;

  WNDCLASSEX wcex = {sizeof(WNDCLASSEX)};
  wcex.style = CS_HREDRAW | CS_VREDRAW;
  wcex.lpfnWndProc = WndProc;
  wcex.cbClsExtra = 0;
  wcex.cbWndExtra = 0;
  wcex.hInstance = window->hInstance;
  wcex.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
  ;
  wcex.lpszMenuName = nullptr;
  wcex.hCursor = LoadCursor(NULL, IDC_ARROW);
  wcex.lpszClassName = CLASS_NAME;
  RegisterClassEx(&wcex);

  int dpiX = 0;
  int dpiY = 0;
  HDC hdc = GetDC(NULL);
  if (hdc) {
    dpiX = GetDeviceCaps(hdc, LOGPIXELSX);
    dpiY = GetDeviceCaps(hdc, LOGPIXELSY);
    ReleaseDC(NULL, hdc);
  }

  RECT window_rect = {0, 0, width, height};
  AdjustWindowRect(&window_rect, WS_OVERLAPPEDWINDOW, FALSE);
  UINT window_width = static_cast<UINT>(
      ceil(float(window_rect.right - window_rect.left) * dpiX / 96.f));
  UINT window_height = static_cast<UINT>(
      ceil(float(window_rect.bottom - window_rect.top) * dpiY / 96.f));

  LPCWSTR name;
  DWORD style;
  if (enable_compositor) {
    name = L"example-compositor (DirectComposition)";
    style = WS_EX_NOREDIRECTIONBITMAP;
  } else {
    name = L"example-compositor (Simple)";
    style = 0;
  }

  window->hWnd =
      CreateWindowEx(style, CLASS_NAME, name, WS_OVERLAPPEDWINDOW,
                     CW_USEDEFAULT, CW_USEDEFAULT, window_width, window_height,
                     NULL, NULL, window->hInstance, NULL);

  ShowWindow(window->hWnd, SW_SHOWNORMAL);
  UpdateWindow(window->hWnd);
  GetClientRect(window->hWnd, &window->client_rect);

  // Create a D3D11 device
  D3D_FEATURE_LEVEL featureLevelSupported;
  HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, NULL,
                                 D3D11_CREATE_DEVICE_BGRA_SUPPORT, NULL, 0,
                                 D3D11_SDK_VERSION, &window->pD3D11Device,
                                 &featureLevelSupported, nullptr);
  assert(SUCCEEDED(hr));

  D3D11_QUERY_DESC query_desc;
  memset(&query_desc, 0, sizeof(query_desc));
  query_desc.Query = D3D11_QUERY_EVENT;
  for (int i = 0; i < NUM_QUERIES; ++i) {
    hr = window->pD3D11Device->CreateQuery(&query_desc, &window->pQueries[i]);
    assert(SUCCEEDED(hr));
  }
  window->current_query = 0;

  hr = window->pD3D11Device->QueryInterface(&window->pDXGIDevice);
  assert(SUCCEEDED(hr));

  // Create a DirectComposition device
  hr = DCompositionCreateDevice2(window->pDXGIDevice,
                                 __uuidof(IDCompositionDesktopDevice),
                                 (void**)&window->pDCompDevice);
  assert(SUCCEEDED(hr));

  // Create a DirectComposition target for a Win32 window handle
  hr = window->pDCompDevice->CreateTargetForHwnd(window->hWnd, TRUE,
                                                 &window->pDCompTarget);
  assert(SUCCEEDED(hr));

  // Create an ANGLE EGL device that wraps D3D11
  window->EGLDevice = eglCreateDeviceANGLE(EGL_D3D11_DEVICE_ANGLE,
                                           window->pD3D11Device, nullptr);

  EGLint display_attribs[] = {EGL_NONE};

  window->EGLDisplay = eglGetPlatformDisplayEXT(
      EGL_PLATFORM_DEVICE_EXT, window->EGLDevice, display_attribs);

  eglInitialize(window->EGLDisplay, nullptr, nullptr);

  EGLint num_configs = 0;
  EGLint cfg_attribs[] = {EGL_SURFACE_TYPE,
                          EGL_WINDOW_BIT,
                          EGL_RENDERABLE_TYPE,
                          EGL_OPENGL_ES2_BIT,
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

  eglChooseConfig(window->EGLDisplay, cfg_attribs, configs,
                  sizeof(configs) / sizeof(EGLConfig), &num_configs);
  assert(num_configs > 0);
  window->config = configs[0];

  if (window->enable_compositor) {
    window->fb_surface = EGL_NO_SURFACE;
  } else {
    window->fb_surface = eglCreateWindowSurface(
        window->EGLDisplay, window->config, window->hWnd, NULL);
    assert(window->fb_surface != EGL_NO_SURFACE);
  }

  EGLint ctx_attribs[] = {EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE};

  // Create an EGL context that can be used for drawing
  window->EGLContext = eglCreateContext(window->EGLDisplay, window->config,
                                        EGL_NO_CONTEXT, ctx_attribs);

  // Create the root of the DirectComposition visual tree
  hr = window->pDCompDevice->CreateVisual(&window->pRoot);
  assert(SUCCEEDED(hr));
  hr = window->pDCompTarget->SetRoot(window->pRoot);
  assert(SUCCEEDED(hr));

  hr = window->pRoot->QueryInterface(__uuidof(IDCompositionVisualDebug),
                                     (void**)&window->pVisualDebug);
  assert(SUCCEEDED(hr));

  // Uncomment this to see redraw regions during composite
  // window->pVisualDebug->EnableRedrawRegions();

  EGLBoolean ok = eglMakeCurrent(window->EGLDisplay, window->fb_surface,
                                 window->fb_surface, window->EGLContext);
  assert(ok);

  return window;
}

void com_dc_destroy_window(Window* window) {
  for (auto surface_it = window->surfaces.begin();
       surface_it != window->surfaces.end(); ++surface_it) {
    Surface& surface = surface_it->second;

#ifndef USE_VIRTUAL_SURFACES
    for (auto tile_it = surface.tiles.begin(); tile_it != surface.tiles.end();
         ++tile_it) {
      tile_it->second.pSurface->Release();
      tile_it->second.pVisual->Release();
    }
#endif

    surface.pVisual->Release();
  }

  if (window->fb_surface != EGL_NO_SURFACE) {
    eglDestroySurface(window->EGLDisplay, window->fb_surface);
  }
  eglDestroyContext(window->EGLDisplay, window->EGLContext);
  eglTerminate(window->EGLDisplay);
  eglReleaseDeviceANGLE(window->EGLDevice);

  for (int i = 0; i < NUM_QUERIES; ++i) {
    window->pQueries[i]->Release();
  }
  window->pRoot->Release();
  window->pVisualDebug->Release();
  window->pD3D11Device->Release();
  window->pDXGIDevice->Release();
  window->pDCompDevice->Release();
  window->pDCompTarget->Release();

  CloseWindow(window->hWnd);
  UnregisterClass(CLASS_NAME, window->hInstance);

  delete window;
}

bool com_dc_tick(Window*) {
  // Check and dispatch the windows event loop
  MSG msg;
  while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
    if (msg.message == WM_QUIT) {
      return false;
    }

    TranslateMessage(&msg);
    DispatchMessage(&msg);
  }

  return true;
}

void com_dc_swap_buffers(Window* window) {
  // If not using DC mode, then do a normal EGL swap buffers.
  if (window->fb_surface != EGL_NO_SURFACE) {
    switch (window->sync_mode) {
      case SyncMode::None:
        eglSwapInterval(window->EGLDisplay, 0);
        break;
      case SyncMode::Swap:
        eglSwapInterval(window->EGLDisplay, 1);
        break;
      default:
        assert(false);  // unexpected vsync mode for simple compositor.
        break;
    }

    eglSwapBuffers(window->EGLDisplay, window->fb_surface);
  } else {
    switch (window->sync_mode) {
      case SyncMode::None:
        break;
      case SyncMode::Commit:
        window->pDCompDevice->WaitForCommitCompletion();
        break;
      case SyncMode::Flush:
        DwmFlush();
        break;
      case SyncMode::Query:
        // todo!!!!
        break;
      default:
        assert(false);  // unexpected vsync mode for native compositor
        break;
    }
  }
}

// Create a new DC surface
void com_dc_create_surface(Window* window, uint64_t id, int tile_width,
                           int tile_height, bool is_opaque) {
  assert(window->surfaces.count(id) == 0);

  Surface surface;
  surface.tile_width = tile_width;
  surface.tile_height = tile_height;
  surface.is_opaque = is_opaque;

  // Create the visual node in the DC tree that stores properties
  HRESULT hr = window->pDCompDevice->CreateVisual(&surface.pVisual);
  assert(SUCCEEDED(hr));

#ifdef USE_VIRTUAL_SURFACES
  DXGI_ALPHA_MODE alpha_mode = surface.is_opaque
                                   ? DXGI_ALPHA_MODE_IGNORE
                                   : DXGI_ALPHA_MODE_PREMULTIPLIED;

  hr = window->pDCompDevice->CreateVirtualSurface(
      VIRTUAL_OFFSET * 2, VIRTUAL_OFFSET * 2, DXGI_FORMAT_B8G8R8A8_UNORM,
      alpha_mode, &surface.pVirtualSurface);
  assert(SUCCEEDED(hr));

  // Bind the surface memory to this visual
  hr = surface.pVisual->SetContent(surface.pVirtualSurface);
  assert(SUCCEEDED(hr));
#endif

  window->surfaces[id] = surface;
}

void com_dc_create_tile(Window* window, uint64_t id, int x, int y) {
  assert(window->surfaces.count(id) == 1);
  Surface& surface = window->surfaces[id];

  TileKey key(x, y);
  assert(surface.tiles.count(key) == 0);

  Tile tile;

#ifndef USE_VIRTUAL_SURFACES
  // Create the video memory surface.
  DXGI_ALPHA_MODE alpha_mode = surface.is_opaque
                                   ? DXGI_ALPHA_MODE_IGNORE
                                   : DXGI_ALPHA_MODE_PREMULTIPLIED;
  HRESULT hr = window->pDCompDevice->CreateSurface(
      surface.tile_width, surface.tile_height, DXGI_FORMAT_B8G8R8A8_UNORM,
      alpha_mode, &tile.pSurface);
  assert(SUCCEEDED(hr));

  // Create the visual node in the DC tree that stores properties
  hr = window->pDCompDevice->CreateVisual(&tile.pVisual);
  assert(SUCCEEDED(hr));

  // Bind the surface memory to this visual
  hr = tile.pVisual->SetContent(tile.pSurface);
  assert(SUCCEEDED(hr));

  // Place the visual in local-space of this surface
  float offset_x = (float)(x * surface.tile_width);
  float offset_y = (float)(y * surface.tile_height);
  tile.pVisual->SetOffsetX(offset_x);
  tile.pVisual->SetOffsetY(offset_y);

  surface.pVisual->AddVisual(tile.pVisual, FALSE, NULL);
#endif

  surface.tiles[key] = tile;
}

void com_dc_destroy_tile(Window* window, uint64_t id, int x, int y) {
  assert(window->surfaces.count(id) == 1);
  Surface& surface = window->surfaces[id];

  TileKey key(x, y);
  assert(surface.tiles.count(key) == 1);
  Tile& tile = surface.tiles[key];

#ifndef USE_VIRTUAL_SURFACES
  surface.pVisual->RemoveVisual(tile.pVisual);

  tile.pVisual->Release();
  tile.pSurface->Release();
#endif

  surface.tiles.erase(key);
}

void com_dc_destroy_surface(Window* window, uint64_t id) {
  assert(window->surfaces.count(id) == 1);
  Surface& surface = window->surfaces[id];

  window->pRoot->RemoveVisual(surface.pVisual);

#ifdef USE_VIRTUAL_SURFACES
  surface.pVirtualSurface->Release();
#else
  // Release the video memory and visual in the tree
  for (auto tile_it = surface.tiles.begin(); tile_it != surface.tiles.end();
       ++tile_it) {
    tile_it->second.pSurface->Release();
    tile_it->second.pVisual->Release();
  }
#endif

  surface.pVisual->Release();
  window->surfaces.erase(id);
}

// Bind a DC surface to allow issuing GL commands to it
GLuint com_dc_bind_surface(Window* window, uint64_t surface_id, int tile_x,
                           int tile_y, int* x_offset, int* y_offset,
                           int dirty_x0, int dirty_y0, int dirty_width,
                           int dirty_height) {
  assert(window->surfaces.count(surface_id) == 1);
  Surface& surface = window->surfaces[surface_id];

  TileKey key(tile_x, tile_y);
  assert(surface.tiles.count(key) == 1);
  Tile& tile = surface.tiles[key];

  // Inform DC that we want to draw on this surface. DC uses texture
  // atlases when the tiles are small. It returns an offset where the
  // client code must draw into this surface when this happens.
  RECT update_rect;
  update_rect.left = dirty_x0;
  update_rect.top = dirty_y0;
  update_rect.right = dirty_x0 + dirty_width;
  update_rect.bottom = dirty_y0 + dirty_height;
  POINT offset;
  D3D11_TEXTURE2D_DESC desc;
  ID3D11Texture2D* pTexture;
  HRESULT hr;

  // Store the current surface for unbinding later
#ifdef USE_VIRTUAL_SURFACES
  LONG tile_offset_x = VIRTUAL_OFFSET + tile_x * surface.tile_width;
  LONG tile_offset_y = VIRTUAL_OFFSET + tile_y * surface.tile_height;

  update_rect.left += tile_offset_x;
  update_rect.top += tile_offset_y;
  update_rect.right += tile_offset_x;
  update_rect.bottom += tile_offset_y;

  hr = surface.pVirtualSurface->BeginDraw(
      &update_rect, __uuidof(ID3D11Texture2D), (void**)&pTexture, &offset);
  window->pCurrentSurface = surface.pVirtualSurface;
#else
  hr = tile.pSurface->BeginDraw(&update_rect, __uuidof(ID3D11Texture2D),
                                (void**)&pTexture, &offset);
  window->pCurrentSurface = tile.pSurface;
#endif

  // DC includes the origin of the dirty / update rect in the draw offset,
  // undo that here since WR expects it to be an absolute offset.
  assert(SUCCEEDED(hr));
  offset.x -= dirty_x0;
  offset.y -= dirty_y0;
  pTexture->GetDesc(&desc);
  *x_offset = offset.x;
  *y_offset = offset.y;

  // Construct an EGLImage wrapper around the D3D texture for ANGLE.
  const EGLAttrib attribs[] = {EGL_NONE};
  window->mEGLImage = eglCreateImage(
      window->EGLDisplay, EGL_NO_CONTEXT, EGL_D3D11_TEXTURE_ANGLE,
      static_cast<EGLClientBuffer>(pTexture), attribs);

  // Get the current FBO and RBO id, so we can restore them later
  GLint currentFboId, currentRboId;
  glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING, &currentFboId);
  glGetIntegerv(GL_RENDERBUFFER_BINDING, &currentRboId);

  // Create a render buffer object that is backed by the EGL image.
  glGenRenderbuffers(1, &window->mColorRBO);
  glBindRenderbuffer(GL_RENDERBUFFER, window->mColorRBO);
  glEGLImageTargetRenderbufferStorageOES(GL_RENDERBUFFER, window->mEGLImage);

  // Get or create an FBO for the specified dimensions
  GLuint fboId = GetOrCreateFbo(window, desc.Width, desc.Height);

  // Attach the new renderbuffer to the FBO
  glBindFramebuffer(GL_DRAW_FRAMEBUFFER, fboId);
  glFramebufferRenderbuffer(GL_DRAW_FRAMEBUFFER, GL_COLOR_ATTACHMENT0,
                            GL_RENDERBUFFER, window->mColorRBO);

  // Restore previous FBO and RBO bindings
  glBindFramebuffer(GL_DRAW_FRAMEBUFFER, currentFboId);
  glBindRenderbuffer(GL_RENDERBUFFER, currentRboId);

  return fboId;
}

// Unbind a currently bound DC surface
void com_dc_unbind_surface(Window* window) {
  HRESULT hr = window->pCurrentSurface->EndDraw();
  assert(SUCCEEDED(hr));

  glDeleteRenderbuffers(1, &window->mColorRBO);
  window->mColorRBO = 0;

  eglDestroyImage(window->EGLDisplay, window->mEGLImage);
  window->mEGLImage = EGL_NO_IMAGE;
}

void com_dc_begin_transaction(Window*) {}

// Add a DC surface to the visual tree. Called per-frame to build the
// composition.
void com_dc_add_surface(Window* window, uint64_t id, int x, int y, int clip_x,
                        int clip_y, int clip_w, int clip_h) {
  Surface surface = window->surfaces[id];
  window->mCurrentLayers.push_back(id);

  // Place the visual - this changes frame to frame based on scroll position
  // of the slice.
  float offset_x = (float)(x + window->client_rect.left);
  float offset_y = (float)(y + window->client_rect.top);
#ifdef USE_VIRTUAL_SURFACES
  offset_x -= VIRTUAL_OFFSET;
  offset_y -= VIRTUAL_OFFSET;
#endif
  surface.pVisual->SetOffsetX(offset_x);
  surface.pVisual->SetOffsetY(offset_y);

  // Set the clip rect - converting from world space to the pre-offset space
  // that DC requires for rectangle clips.
  D2D_RECT_F clip_rect;
  clip_rect.left = clip_x - offset_x;
  clip_rect.top = clip_y - offset_y;
  clip_rect.right = clip_rect.left + clip_w;
  clip_rect.bottom = clip_rect.top + clip_h;
  surface.pVisual->SetClip(clip_rect);
}

// Finish the composition transaction, telling DC to composite
void com_dc_end_transaction(Window* window) {
  bool same = window->mPrevLayers == window->mCurrentLayers;

  if (!same) {
    HRESULT hr = window->pRoot->RemoveAllVisuals();
    assert(SUCCEEDED(hr));

    for (auto it = window->mCurrentLayers.begin();
         it != window->mCurrentLayers.end(); ++it) {
      Surface& surface = window->surfaces[*it];

      // Add this visual as the last element in the visual tree (z-order is
      // implicit, based on the order tiles are added).
      hr = window->pRoot->AddVisual(surface.pVisual, FALSE, NULL);
      assert(SUCCEEDED(hr));
    }
  }

  window->mPrevLayers.swap(window->mCurrentLayers);
  window->mCurrentLayers.clear();

  HRESULT hr = window->pDCompDevice->Commit();
  assert(SUCCEEDED(hr));
}

// Get a pointer to an EGL symbol
void* com_dc_get_proc_address(const char* name) {
  return eglGetProcAddress(name);
}
}
