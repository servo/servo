/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// The immersive mode Servo magicleap demo

#include <stdio.h>
#include <stdlib.h>

#include <unistd.h>
#include <sys/syscall.h>

#ifndef EGL_EGLEXT_PROTOTYPES
#define EGL_EGLEXT_PROTOTYPES
#endif

#include <EGL/egl.h>
#include <EGL/eglext.h>

#ifndef GL_GLEXT_PROTOTYPES
#define GL_GLEXT_PROTOTYPES
#endif

#include <GLES3/gl3.h>
#include <GLES3/gl3ext.h>

#include <ml_graphics.h>
#include <ml_head_tracking.h>
#include <ml_perception.h>
#include <ml_fileinfo.h>
#include <ml_lifecycle.h>
#include <ml_logging.h>
#include <ml_privileges.h>

// Constants
const char application_name[] = "com.mozilla.servo3d";

// A function which calls the ML logger, suitable for passing into Servo
typedef void (*MLLogger)(MLLogLevel lvl, char* msg);
void logger(MLLogLevel lvl, char* msg) {
  if (MLLoggingLogLevelIsEnabled(lvl)) {
    MLLoggingLog(lvl, "Servo3D", msg);
  }
}

// Entry points to servo
typedef struct Opaque ServoInstance;
extern "C" ServoInstance* init_servo(EGLContext, EGLSurface, EGLDisplay, bool landscape,
                                     void*, MLLogger, void*, void*, void*,
                                     const char* url, const char* args,
                                     int width, int height, float hidpi);
extern "C" void heartbeat_servo(ServoInstance*);
extern "C" void discard_servo(ServoInstance*);

// The Servo3D app
struct Servo3D {
  ServoInstance* servo;
  bool running;
};

// Callbacks
static void onStop(void* app)
{
  ML_LOG(Info, "%s: On stop called.", application_name);
  Servo3D* servo3d = (Servo3D*)app;
  servo3d->running = false;
}

static void onPause(void* app)
{
  ML_LOG(Info, "%s: On pause called.", application_name);
  // Treat a pause the same as a stop
  Servo3D* servo3d = (Servo3D*)app;
  servo3d->running = false;
}

static void onResume(void* app)
{
  ML_LOG(Info, "%s: On resume called.", application_name);
}

static void onNewInitArg(void* app)
{
  // TODO: call servo_navigate when a new URL arrives
  ML_LOG(Info, "%s: On new init arg called.", application_name);
}

int main() {
  // set up graphics surface
  ML_LOG(Info, "%s: Initializing EGL.", application_name);
  EGLDisplay egl_display = eglGetDisplay(EGL_DEFAULT_DISPLAY);

  EGLint major = 4;
  EGLint minor = 0;
  eglInitialize(egl_display, &major, &minor);

  // The GL API used should match https://github.com/servo/rust-offscreen-rendering-context/blob/fcbbb4d40dac5e969233c1519151ad5e07b7f22e/src/platform/with_egl/native_gl_context.rs#L14
  eglBindAPI(EGL_OPENGL_ES_API);

  // Should match https://github.com/servo/rust-offscreen-rendering-context/blob/fcbbb4d40dac5e969233c1519151ad5e07b7f22e/src/platform/with_egl/utils.rs#L46
  EGLint config_attribs[] = {
    EGL_RED_SIZE, 8,
    EGL_GREEN_SIZE, 8,
    EGL_BLUE_SIZE, 8,
    EGL_ALPHA_SIZE, 0,
    EGL_DEPTH_SIZE, 24,
    EGL_STENCIL_SIZE, 0,
    EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
    EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
    EGL_NONE
  };
  EGLConfig egl_config = nullptr;
  EGLint config_size = 0;
  eglChooseConfig(egl_display, config_attribs, &egl_config, 1, &config_size);
  if (config_size < 1) {
    ML_LOG(Error, "%s: Failed to choose EGL config. (%x)", application_name, eglGetError());
    return -1;
  }

  // Should match https://github.com/servo/rust-offscreen-rendering-context/blob/fcbbb4d40dac5e969233c1519151ad5e07b7f22e/src/platform/with_egl/native_gl_context.rs#L47
  EGLint context_attribs[] = {
    EGL_CONTEXT_CLIENT_VERSION, 3,
    EGL_NONE
  };
  EGLContext egl_context = eglCreateContext(egl_display, egl_config, EGL_NO_CONTEXT, context_attribs);
  if (EGL_NO_CONTEXT == egl_context) {
    ML_LOG(Error, "%s: Failed to initialize EGL context. (%x)", application_name, eglGetError());
    return -1;
  }

  EGLint surface_attribs[] = {
    EGL_WIDTH, 1280,
    EGL_HEIGHT, 960,
    EGL_NONE
  };
  EGLSurface egl_surface = eglCreatePbufferSurface(egl_display, egl_config, surface_attribs);
  if (EGL_NO_SURFACE == egl_surface) {
    ML_LOG(Error, "%s: Failed to initialize EGL surface. (%x)", application_name, eglGetError());
    return -1;
  }

  if (!eglMakeCurrent(egl_display, egl_surface, egl_surface, egl_context)) {
    ML_LOG(Error, "%s: Failed to make EGL surface current. (%x)", application_name, eglGetError());
    return -1;
  }

  GLenum read_status = glCheckFramebufferStatus(GL_READ_FRAMEBUFFER);
  GLenum draw_status = glCheckFramebufferStatus(GL_DRAW_FRAMEBUFFER);
  if ((read_status != GL_FRAMEBUFFER_COMPLETE) || (draw_status != GL_FRAMEBUFFER_COMPLETE)) {
    ML_LOG(Error, "%s: Incomplete GL framebuffer. (%x, %x)", application_name, read_status, draw_status);
    return -1;
  }

  ML_LOG(Info, "%s: Initialized EGL.", application_name);

  // The app
  Servo3D app = { nullptr, true };

  // let system know our app has started
  MLLifecycleCallbacks lifecycle_callbacks = {};
  lifecycle_callbacks.on_stop = onStop;
  lifecycle_callbacks.on_pause = onPause;
  lifecycle_callbacks.on_resume = onResume;
  lifecycle_callbacks.on_new_initarg = onNewInitArg;

  if (MLResult_Ok != MLLifecycleInit(&lifecycle_callbacks, &app)) {
    ML_LOG(Error, "%s: Failed to initialize lifecycle.", application_name);
    return -1;
  }

  // Get the file argument if there is one
  MLLifecycleInitArgList* arg_list = nullptr;
  const MLLifecycleInitArg* arg = nullptr;
  const char* url = "https://webvr.info/samples/03-vr-presentation.html";
  int64_t arg_list_len = 0;

  if (MLResult_Ok != MLLifecycleGetInitArgList(&arg_list)) {
    ML_LOG(Error, "%s: Failed to get init args.", application_name);
    return -1;
  }

  if (MLResult_Ok == MLLifecycleGetInitArgListLength(arg_list, &arg_list_len)) {
    if (arg_list_len) {
      if (MLResult_Ok != MLLifecycleGetInitArgByIndex(arg_list, 0, &arg)) {
        ML_LOG(Error, "%s: Failed to get init arg.", application_name);
        return -1;
      }

      if (MLResult_Ok != MLLifecycleGetInitArgUri(arg, &url)) {
        ML_LOG(Error, "%s: Failed to get init arg uri.", application_name);
        return -1;
      }
    }
  }

  // init_servo calls MLLifecycleSetReadyIndication()

  MLLifecycleFreeInitArgList(&arg_list);

  // Check privileges
  if (MLResult_Ok != MLPrivilegesStartup()) {
    ML_LOG(Error, "%s: Failed to initialize privileges.", application_name);
    return -1;
  }
  if (MLPrivilegesRequestPrivilege(MLPrivilegeID_LowLatencyLightwear) != MLPrivilegesResult_Granted) {
    ML_LOG(Error, "Privilege %d denied.", MLPrivilegeID_LowLatencyLightwear);
    return -1;
  }
  if (MLPrivilegesRequestPrivilege(MLPrivilegeID_Internet) != MLPrivilegesResult_Granted) {
    ML_LOG(Error, "Privilege %d denied.", MLPrivilegeID_Internet);
    return -1;
  }

  // initialize perception system
  MLPerceptionSettings perception_settings;
  if (MLResult_Ok != MLPerceptionInitSettings(&perception_settings)) {
    ML_LOG(Error, "%s: Failed to initialize perception.", application_name);
  }

  if (MLResult_Ok != MLPerceptionStartup(&perception_settings)) {
    ML_LOG(Error, "%s: Failed to startup perception.", application_name);
    return -1;
  }

  ML_LOG(Info, "%s: Initializing servo for %s.", application_name, url);

  // Initialize servo
  app.servo = init_servo(egl_context, egl_surface, egl_display, false,
                         &app, logger, nullptr, nullptr, nullptr,
                         url,
                         "--pref dom.webvr.enabled --pref dom.gamepad.enabled",
                         500, 500, 1.0);

  // Pin the main thread to the Denver core
  // https://forum.magicleap.com/hc/en-us/community/posts/360043120832-How-many-CPUs-does-an-immersive-app-have-access-to-
  uint32_t DenverCoreAffinityMask = 1 << 2; // Denver core is CPU2
  pid_t ThreadId = gettid();
  syscall(__NR_sched_setaffinity, ThreadId, sizeof(DenverCoreAffinityMask), &DenverCoreAffinityMask);

  // Run the demo!
  ML_LOG(Info, "%s: Begin demo.", application_name);
  while (app.running) {
      ML_LOG(Debug, "%s: heartbeat.", application_name);
      heartbeat_servo(app.servo);
      // TODO: check heart_racing.
  }
  ML_LOG(Info, "%s: End demo.", application_name);

  // Shut down
  discard_servo(app.servo);
  MLPerceptionShutdown();
  eglDestroyContext(egl_display, egl_context);
  eglTerminate(egl_display);

  return 0;
}
