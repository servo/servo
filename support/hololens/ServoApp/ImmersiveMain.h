/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#pragma once

//
// Comment out this preprocessor definition to disable all of the
// sample content.
//
// To remove the content after disabling it:
//     * Remove the unused code from your app's Main class.
//     * Delete the Content folder provided with this template.
//
#define DRAW_SAMPLE_CONTENT

#include "Common/DeviceResources.h"
#include "Common/StepTimer.h"

#ifdef DRAW_SAMPLE_CONTENT
#include "Content/SpinningCubeRenderer.h"
#include "Content/SpatialInputHandler.h"
#endif

// Updates, renders, and presents holographic content using Direct3D.
namespace Immersive {
class ImmersiveMain : public DX::IDeviceNotify {
public:
  ImmersiveMain(std::shared_ptr<DX::DeviceResources> const &deviceResources);
  ~ImmersiveMain();

  // Sets the holographic space. This is our closest analogue to setting a new
  // window for the app.
  void SetHolographicSpace(
      winrt::Windows::Graphics::Holographic::HolographicSpace const
          &holographicSpace);

  // Starts the holographic frame and updates the content.
  winrt::Windows::Graphics::Holographic::HolographicFrame Update();

  // Renders holograms, including world-locked content.
  bool Render(winrt::Windows::Graphics::Holographic::HolographicFrame const
                  &holographicFrame);

  // Handle saving and loading of app state owned by AppMain.
  void SaveAppState();
  void LoadAppState();

  // Handle mouse input.
  void OnPointerPressed();

  // IDeviceNotify
  void OnDeviceLost() override;
  void OnDeviceRestored() override;

private:
  // Asynchronously creates resources for new holographic cameras.
  void OnCameraAdded(
      winrt::Windows::Graphics::Holographic::HolographicSpace const &sender,
      winrt::Windows::Graphics::Holographic::
          HolographicSpaceCameraAddedEventArgs const &args);

  // Synchronously releases resources for holographic cameras that are no longer
  // attached to the system.
  void OnCameraRemoved(
      winrt::Windows::Graphics::Holographic::HolographicSpace const &sender,
      winrt::Windows::Graphics::Holographic::
          HolographicSpaceCameraRemovedEventArgs const &args);

  // Used to notify the app when the positional tracking state changes.
  void OnLocatabilityChanged(
      winrt::Windows::Perception::Spatial::SpatialLocator const &sender,
      winrt::Windows::Foundation::IInspectable const &args);

  // Used to be aware of gamepads that are plugged in after the app starts.
  void OnGamepadAdded(winrt::Windows::Foundation::IInspectable,
                      winrt::Windows::Gaming::Input::Gamepad const &args);

  // Used to stop looking for gamepads that are removed while the app is
  // running.
  void OnGamepadRemoved(winrt::Windows::Foundation::IInspectable,
                        winrt::Windows::Gaming::Input::Gamepad const &args);

  // Used to respond to changes to the default spatial locator.
  void OnHolographicDisplayIsAvailableChanged(
      winrt::Windows::Foundation::IInspectable,
      winrt::Windows::Foundation::IInspectable);

  // Clears event registration state. Used when changing to a new
  // HolographicSpace and when tearing down AppMain.
  void UnregisterHolographicEventHandlers();

#ifdef DRAW_SAMPLE_CONTENT
  // Renders a colorful holographic cube that's 20 centimeters wide. This sample
  // content is used to demonstrate world-locked rendering.
  std::unique_ptr<SpinningCubeRenderer> m_spinningCubeRenderer;

  // Listens for the Pressed spatial input event.
  std::shared_ptr<SpatialInputHandler> m_spatialInputHandler;
#endif

  // Cached pointer to device resources.
  std::shared_ptr<DX::DeviceResources> m_deviceResources;

  // Render loop timer.
  DX::StepTimer m_timer;

  // Represents the holographic space around the user.
  winrt::Windows::Graphics::Holographic::HolographicSpace m_holographicSpace =
      nullptr;

  // SpatialLocator that is attached to the default HolographicDisplay.
  winrt::Windows::Perception::Spatial::SpatialLocator m_spatialLocator =
      nullptr;

  // A stationary reference frame based on m_spatialLocator.
  winrt::Windows::Perception::Spatial::SpatialStationaryFrameOfReference
      m_stationaryReferenceFrame = nullptr;

  // Event registration tokens.
  winrt::event_token m_cameraAddedToken;
  winrt::event_token m_cameraRemovedToken;
  winrt::event_token m_locatabilityChangedToken;
  winrt::event_token m_gamepadAddedEventToken;
  winrt::event_token m_gamepadRemovedEventToken;
  winrt::event_token m_holographicDisplayIsAvailableChangedEventToken;

  // Keep track of gamepads.
  struct GamepadWithButtonState {
    winrt::Windows::Gaming::Input::Gamepad gamepad;
    bool buttonAWasPressedLastFrame = false;
  };
  std::vector<GamepadWithButtonState> m_gamepads;

  // Keep track of mouse input.
  bool m_pointerPressed = false;

  // Cache whether or not the HolographicCamera.Display property can be
  // accessed.
  bool m_canGetHolographicDisplayForCamera = false;

  // Cache whether or not the HolographicDisplay.GetDefault() method can be
  // called.
  bool m_canGetDefaultHolographicDisplay = false;

  // Cache whether or not the
  // HolographicCameraRenderingParameters.CommitDirect3D11DepthBuffer() method
  // can be called.
  bool m_canCommitDirect3D11DepthBuffer = false;
};
} // namespace Immersive
