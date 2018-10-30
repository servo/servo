/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <Servo2D.h>
#include <lumin/node/RootNode.h>
#include <lumin/node/QuadNode.h>
#include <lumin/ui/Cursor.h>
#include <lumin/ui/node/UiButton.h>
#include <ml_logging.h>
#include <scenesGen.h>
#include <SceneDescriptor.h>
#include <EGL/egl.h>
#include <GLES/gl.h>
#include <glm/gtc/matrix_transform.hpp>
#include <string.h>

// The viewport dimensions (in px).
const int VIEWPORT_W = 500;
const int VIEWPORT_H = 500;

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
extern "C" ServoInstance* init_servo(EGLContext, EGLSurface, EGLDisplay, MLLogger,
                                    const char* url, int width, int height, float hidpi);
extern "C" void heartbeat_servo(ServoInstance*);
extern "C" void cursor_servo(ServoInstance*, float x, float y, bool triggered);
extern "C" void traverse_servo(ServoInstance*, int delta);
extern "C" void discard_servo(ServoInstance*);

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
  content_node_ = lumin::QuadNode::CastFrom(prism_->findNode(content_node_id, root_node));
  if (!content_node_) {
    ML_LOG(Error, "Servo2D Failed to get content node");
    abort();
    return 1;
  }
  content_node_->setTriggerable(true);

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

  content_node_->setRenderResource(plane_id);

  // Get the EGL context, surface and display.
  EGLContext ctx = plane_->getEGLContext();
  EGLSurface surf = plane_->getEGLSurface();
  EGLDisplay dpy = eglGetDisplay(EGL_DEFAULT_DISPLAY);

  // Hook into servo
  servo_ = init_servo(ctx, surf, dpy, logger, "https://servo.org/", VIEWPORT_H, VIEWPORT_W, HIDPI);
  if (!servo_) {
    ML_LOG(Error, "Servo2D Failed to init servo instance");
    abort();
    return 1;
  }

  // Add a callback to the back button
  std::string back_button_id = Servo2D_exportedNodes::backButton;
  lumin::ui::UiButton* back_button = lumin::ui::UiButton::CastFrom(prism_->findNode(back_button_id, root_node));
  if (!back_button) {
    ML_LOG(Error, "Servo2D Failed to get back button");
    abort();
    return 1;
  }
  back_button->onActivateSub(std::bind(traverse_servo, servo_, -1));

  // Add a callback to the forward button
  std::string fwd_button_id = Servo2D_exportedNodes::fwdButton;
  lumin::ui::UiButton* fwd_button = lumin::ui::UiButton::CastFrom(prism_->findNode(fwd_button_id, root_node));
  if (!fwd_button) {
    ML_LOG(Error, "Servo2D Failed to get forward button");
    abort();
    return 1;
  }
  fwd_button->onActivateSub(std::bind(traverse_servo, servo_, +1));

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
  // Hook into servo
  heartbeat_servo(servo_);

  // Return true for your app to continue running, false to terminate the app.
  return true;
}

bool Servo2D::eventListener(lumin::ServerEvent* event) {
  // Dispatch based on event type
  switch (event->getServerEventType()) {
    case lumin::ServerEventType::kControlTouchPadInputEvent:
      return touchpadEventListener(static_cast<lumin::ControlTouchPadInputEventData*>(event));
    case lumin::ServerEventType::kKeyInputEvent:
      return keyEventListener(static_cast<lumin::KeyInputEventData*>(event));
    default:
      return false;
  }
}

glm::vec2 Servo2D::viewportCursorPosition() {
  // Get the cursor position relative to the origin of the content node (in m)
  glm::vec3 pos = lumin::ui::Cursor::GetPosition(prism_) - content_node_->getPrismPosition();

  // Get the size of the content node (in m)
  glm::vec2 sz = content_node_->getSize();

  // Convert to a position in viewport px
  float x = (pos.x / sz.x) * (float)VIEWPORT_W;
  float y = (1 - pos.y / sz.y) * (float)VIEWPORT_H; // Sigh, invert the y coordinate

  return glm::vec2(x, y);
}

bool Servo2D::pointInsideViewport(glm::vec2 pt) {
   return (0 <= pt.x && 0 <= pt.y && pt.x <= VIEWPORT_W && pt.y <= VIEWPORT_H);
}

bool Servo2D::touchpadEventListener(lumin::ControlTouchPadInputEventData* event) {
  // Only respond when the cursor is enabled
  if (!lumin::ui::Cursor::IsEnabled(prism_)) {
    return false;
  }

  // Only respond when the cursor is inside the viewport
  glm::vec2 pos = viewportCursorPosition();
  if (!pointInsideViewport(pos)) {
    return false;
  }

  // Inform Servo of the trigger
  cursor_servo(servo_, pos.x, pos.y, false);
  return true;
}

bool Servo2D::keyEventListener(lumin::KeyInputEventData* event) {
  // Only respond to trigger keys
  if (event->keyCode() != lumin::input::KeyCodes::AKEYCODE_EX_TRIGGER) {
    return false;
  }

  // Only respond when the cursor is enabled
  if (!lumin::ui::Cursor::IsEnabled(prism_)) {
    return false;
  }

  // Only respond when the cursor is inside the viewport
  glm::vec2 pos = viewportCursorPosition();
  if (!pointInsideViewport(pos)) {
    return false;
  }

  // Inform Servo of the trigger
  cursor_servo(servo_, pos.x, pos.y, true);
  return true;
}
