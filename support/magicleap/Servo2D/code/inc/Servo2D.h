/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <lumin/LandscapeApp.h>
#include <lumin/Prism.h>
#include <lumin/event/ServerEvent.h>
#include <lumin/resource/PlanarResource.h>
#include <SceneDescriptor.h>

typedef void* ServoInstance;

/**
 * Servo2D Landscape Application
 */
class Servo2D : public lumin::LandscapeApp {
public:
  /**
   * Constructs the Landscape Application.
   */
  Servo2D();

  /**
   * Destroys the Landscape Application.
   */
  virtual ~Servo2D();

  /**
   * Disallows the copy constructor.
   */
  Servo2D(const Servo2D&) = delete;

  /**
   * Disallows the move constructor.
   */
  Servo2D(Servo2D&&) = delete;

  /**
   * Disallows the copy assignment operator.
   */
  Servo2D& operator=(const Servo2D&) = delete;

  /**
   * Disallows the move assignment operator.
   */
  Servo2D& operator=(Servo2D&&) = delete;

protected:
  /**
   * Initializes the Landscape Application.
   * @return - 0 on success, error code on failure.
   */
  int init() override;

  /**
   * Deinitializes the Landscape Application.
   * @return - 0 on success, error code on failure.
   */
  int deInit() override;

  /**
   * Returns the size of the Prism, default = +/- (1.0f, 1.0f, 1.0f) meters.
   * Used in createPrism().
   */
  const glm::vec3 getInitialPrismExtents() const;

  /**
   * Creates the prism, updates the private variable prism_ with the created prism.
   */
  int createInitialPrism();

  /**
   * Initializes and creates the scene of all scenes marked as initially instanced
   */
  void instanceInitialScenes();

  /**
   * Initializes and creates the scene of the scene and instances it into the prism
   */
  lumin::Node* instanceScene(const SceneDescriptor & sceneToInit);

  /**
   * Run application login
   */
  virtual bool updateLoop(float fDelta) override;

  /**
   * Handle events from the server
   */
  virtual bool eventListener(lumin::ServerEvent* event) override;

private:
  lumin::Prism* prism_ = nullptr;  // represents the bounded space where the App renders.
  lumin::PlanarResource* plane_ = nullptr; // the plane we're rendering into
  ServoInstance servo_ = nullptr; // the servo instance we're embedding
};
