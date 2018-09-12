/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <Servo2D.h>
#include <lumin/node/RootNode.h>
#include <lumin/node/QuadNode.h>
#include <lumin/ui/Cursor.h>
#include <ml_logging.h>
#include <scenesGen.h>
#include <SceneDescriptor.h>
#include <EGL/egl.h>
#include <GLES/gl.h>
#include <string.h>

// The viewport dimensions (in px).
const unsigned int VIEWPORT_W = 500;
const unsigned int VIEWPORT_H = 500;

// The hidpi factor.
const float HIDPI = 1.0;

// The prism dimensions (in m).
const float PRISM_W = 0.5;
const float PRISM_H = 0.5;
const float PRISM_D = 0.5;

// A function which calls the ML logger, suitable for passing into Servo
typedef void (*MLLogger)(MLLogLevel lvl, char* msg);
void logger(MLLogLevel lvl, char* msg) {
  if (MLLoggingLogLevelIsEnabled(lvl)) {
    MLLoggingLog(lvl, ML_DEFAULT_LOG_TAG, msg);
  }
}

// The functions Servo provides for hooking up to the ML.
// For the moment, this doesn't handle input events.
extern "C" ServoInstance init_servo(EGLContext, EGLSurface, EGLDisplay, MLLogger,
                                    const char* url, int width, int height, float hidpi);
extern "C" void heartbeat_servo(ServoInstance);
extern "C" void discard_servo(ServoInstance);

// Create a Servo2D instance
Servo2D::Servo2D() {
  ML_LOG(Debug, "Servo2D Constructor.");
}

// Destroy a Servo 2D instance
Servo2D::~Servo2D() {
  ML_LOG(Debug, "Servo2D Destructor.");
  discard_servo(servo_);
  servo_ = nullptr;
}

// The prism dimensions
const glm::vec3 Servo2D::getInitialPrismExtents() const {
  return glm::vec3(PRISM_W, PRISM_H, PRISM_D);
}

// Create the prism for Servo
int Servo2D::createInitialPrism() {
  prism_ = requestNewPrism(getInitialPrismExtents());
  if (!prism_) {
    ML_LOG(Error, "Servo2D Error creating default prism.");
    return 1;
  }
  return 0;
}

// Initialize a Servo instance
int Servo2D::init() {

  ML_LOG(Debug, "Servo2D Initializing.");

  // Set up the prism
  createInitialPrism();
  lumin::ui::Cursor::SetScale(prism_, 0.03f);
  instanceInitialScenes();

  // Get the planar resource that holds the EGL context
  lumin::RootNode* root_node = prism_->getRootNode();
  if (!root_node) {
    ML_LOG(Error, "Servo2D Failed to get root node");
    abort();
    return 1;
  }

  std::string content_node_id = Servo2D_exportedNodes::content;
  lumin::QuadNode* content_node = lumin::QuadNode::CastFrom(prism_->findNode(content_node_id, root_node));
  if (!content_node) {
    ML_LOG(Error, "Servo2D Failed to get content node");
    abort();
    return 1;
  }

  lumin::ResourceIDType plane_id = prism_->createPlanarEGLResourceId();
  if (!plane_id) {
    ML_LOG(Error, "Servo2D Failed to create EGL resource");
    abort();
    return 1;
  }

  plane_ = static_cast<lumin::PlanarResource*>(prism_->getResource(plane_id));
  if (!plane_) {
    ML_LOG(Error, "Servo2D Failed to create plane");
    abort();
    return 1;
  }

  content_node->setRenderResource(plane_id);

  // Get the EGL context, surface and display.
  EGLContext ctx = plane_->getEGLContext();
  EGLSurface surf = plane_->getEGLSurface();
  EGLDisplay dpy = eglGetDisplay(EGL_DEFAULT_DISPLAY);
  eglMakeCurrent(dpy, surf, surf, ctx);
  glViewport(0, 0, VIEWPORT_W, VIEWPORT_H);

  // Hook into servo
  servo_ = init_servo(ctx, surf, dpy, logger, "https://servo.org", VIEWPORT_H, VIEWPORT_W, HIDPI);
  if (!servo_) {
    ML_LOG(Error, "Servo2D Failed to init servo instance");
    abort();
    return 1;
  }

  // Flush GL
  glFlush();
  eglSwapBuffers(dpy, surf);
  return 0;
}

int Servo2D::deInit() {
  ML_LOG(Debug, "Servo2D Deinitializing.");
  return 0;
}

lumin::Node* Servo2D::instanceScene(const SceneDescriptor& scene) {
  // Load resources.
  if (!prism_->loadResourceModel(scene.getResourceModelPath())) {
    ML_LOG(Info, "No resource model loaded");
  }

  // Load a scene file.
  std::string editorObjectModelName;
  if (!prism_->loadObjectModel(scene.getSceneGraphPath(), editorObjectModelName)) {
    ML_LOG(Error, "Servo2D Failed to load object model");
    abort();
    return nullptr;
  }

  // Add scene to this prism.
  lumin::Node* newTree = prism_->createAll(editorObjectModelName);
  if (!prism_->getRootNode()->addChild(newTree)) {
    ML_LOG(Error, "Servo2D Failed to add newTree to the prism root node");
    abort();
    return nullptr;
  }

  return newTree;
}

void Servo2D::instanceInitialScenes() {
  // Iterate over all the exported scenes
  for (auto& exportedSceneEntry : scenes::exportedScenes ) {

    // If this scene was marked to be instanced at app initialization, do it
    const SceneDescriptor &sd = exportedSceneEntry.second;
    if (sd.getInitiallyInstanced()) {
      instanceScene(sd);
    }
  }
}

bool Servo2D::updateLoop(float fDelta) {
  // Get the EGL context, surface and display.
  EGLContext ctx = plane_->getEGLContext();
  EGLSurface surf = plane_->getEGLSurface();
  EGLDisplay dpy = eglGetDisplay(EGL_DEFAULT_DISPLAY);
  eglMakeCurrent(dpy, surf, surf, ctx);
  glViewport(0, 0, VIEWPORT_W, VIEWPORT_H);

  // Hook into servo
  heartbeat_servo(servo_);

  // Flush GL
  glFlush();
  eglSwapBuffers(dpy, surf);

  // Return true for your app to continue running, false to terminate the app.
  return true;
}

bool Servo2D::eventListener(lumin::ServerEvent* event) {

  // Place your event handling here.

  // Return true if the event is consumed.
  return false;
}
