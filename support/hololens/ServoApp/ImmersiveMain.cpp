/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#include "pch.h"
#include "ImmersiveMain.h"
#include "Common/DirectXHelper.h"

#include <windows.graphics.directx.direct3d11.interop.h>

using namespace Immersive;
using namespace concurrency;
using namespace Microsoft::WRL;
using namespace std::placeholders;
using namespace winrt::Windows::Foundation::Numerics;
using namespace winrt::Windows::Gaming::Input;
using namespace winrt::Windows::Graphics::Holographic;
using namespace winrt::Windows::Graphics::DirectX::Direct3D11;
using namespace winrt::Windows::Perception::Spatial;
using namespace winrt::Windows::UI::Input::Spatial;

// Loads and initializes application assets when the application is loaded.
ImmersiveMain::ImmersiveMain(
    std::shared_ptr<DX::DeviceResources> const &deviceResources)
    : m_deviceResources(deviceResources) {
  // Register to be notified if the device is lost or recreated.
  m_deviceResources->RegisterDeviceNotify(this);

  // If connected, a game controller can also be used for input.
  m_gamepadAddedEventToken =
      Gamepad::GamepadAdded(bind(&ImmersiveMain::OnGamepadAdded, this, _1, _2));
  m_gamepadRemovedEventToken = Gamepad::GamepadRemoved(
      bind(&ImmersiveMain::OnGamepadRemoved, this, _1, _2));

  for (Gamepad const &gamepad : Gamepad::Gamepads()) {
    OnGamepadAdded(nullptr, gamepad);
  }

  m_canGetHolographicDisplayForCamera =
      winrt::Windows::Foundation::Metadata::ApiInformation::IsPropertyPresent(
          L"Windows.Graphics.Holographic.HolographicCamera", L"Display");
  m_canGetDefaultHolographicDisplay =
      winrt::Windows::Foundation::Metadata::ApiInformation::IsMethodPresent(
          L"Windows.Graphics.Holographic.HolographicDisplay", L"GetDefault");
  m_canCommitDirect3D11DepthBuffer =
      winrt::Windows::Foundation::Metadata::ApiInformation::IsMethodPresent(
          L"Windows.Graphics.Holographic.HolographicCameraRenderingParameters",
          L"CommitDirect3D11DepthBuffer");

  if (m_canGetDefaultHolographicDisplay) {
    // Subscribe for notifications about changes to the state of the default
    // HolographicDisplay and its SpatialLocator.
    m_holographicDisplayIsAvailableChangedEventToken =
        HolographicSpace::IsAvailableChanged(
            bind(&ImmersiveMain::OnHolographicDisplayIsAvailableChanged, this,
                 _1, _2));
  }

  // Acquire the current state of the default HolographicDisplay and its
  // SpatialLocator.
  OnHolographicDisplayIsAvailableChanged(nullptr, nullptr);
}

void ImmersiveMain::SetHolographicSpace(
    HolographicSpace const &holographicSpace) {
  UnregisterHolographicEventHandlers();

  m_holographicSpace = holographicSpace;

  //
  // TODO: Add code here to initialize your holographic content.
  //

#ifdef DRAW_SAMPLE_CONTENT
  // Initialize the sample hologram.
  m_spinningCubeRenderer =
      std::make_unique<SpinningCubeRenderer>(m_deviceResources);
  m_spatialInputHandler = std::make_unique<SpatialInputHandler>();
#endif

  // Respond to camera added events by creating any resources that are specific
  // to that camera, such as the back buffer render target view.
  // When we add an event handler for CameraAdded, the API layer will avoid
  // putting the new camera in new HolographicFrames until we complete the
  // deferral we created for that handler, or return from the handler without
  // creating a deferral. This allows the app to take more than one frame to
  // finish creating resources and loading assets for the new holographic
  // camera. This function should be registered before the app creates any
  // HolographicFrames.
  m_cameraAddedToken = m_holographicSpace.CameraAdded(
      std::bind(&ImmersiveMain::OnCameraAdded, this, _1, _2));

  // Respond to camera removed events by releasing resources that were created
  // for that camera. When the app receives a CameraRemoved event, it releases
  // all references to the back buffer right away. This includes render target
  // views, Direct2D target bitmaps, and so on. The app must also ensure that
  // the back buffer is not attached as a render target, as shown in
  // DeviceResources::ReleaseResourcesForBackBuffer.
  m_cameraRemovedToken = m_holographicSpace.CameraRemoved(
      std::bind(&ImmersiveMain::OnCameraRemoved, this, _1, _2));

  // Notes on spatial tracking APIs:
  // * Stationary reference frames are designed to provide a best-fit position
  // relative to the
  //   overall space. Individual positions within that reference frame are
  //   allowed to drift slightly as the device learns more about the
  //   environment.
  // * When precise placement of individual holograms is required, a
  // SpatialAnchor should be used to
  //   anchor the individual hologram to a position in the real world - for
  //   example, a point the user indicates to be of special interest. Anchor
  //   positions do not drift, but can be corrected; the anchor will use the
  //   corrected position starting in the next frame after the correction has
  //   occurred.
}

void ImmersiveMain::UnregisterHolographicEventHandlers() {
  if (m_holographicSpace != nullptr) {
    // Clear previous event registrations.
    m_holographicSpace.CameraAdded(m_cameraAddedToken);
    m_cameraAddedToken = {};
    m_holographicSpace.CameraRemoved(m_cameraRemovedToken);
    m_cameraRemovedToken = {};
  }

  if (m_spatialLocator != nullptr) {
    m_spatialLocator.LocatabilityChanged(m_locatabilityChangedToken);
  }
}

ImmersiveMain::~ImmersiveMain() {
  // Deregister device notification.
  m_deviceResources->RegisterDeviceNotify(nullptr);

  UnregisterHolographicEventHandlers();

  Gamepad::GamepadAdded(m_gamepadAddedEventToken);
  Gamepad::GamepadRemoved(m_gamepadRemovedEventToken);
  HolographicSpace::IsAvailableChanged(
      m_holographicDisplayIsAvailableChangedEventToken);
}

// Updates the application state once per frame.
HolographicFrame ImmersiveMain::Update() {
  // Before doing the timer update, there is some work to do per-frame
  // to maintain holographic rendering. First, we will get information
  // about the current frame.

  // The HolographicFrame has information that the app needs in order
  // to update and render the current frame. The app begins each new
  // frame by calling CreateNextFrame.
  HolographicFrame holographicFrame = m_holographicSpace.CreateNextFrame();

  // Get a prediction of where holographic cameras will be when this frame
  // is presented.
  HolographicFramePrediction prediction = holographicFrame.CurrentPrediction();

  // Back buffers can change from frame to frame. Validate each buffer, and
  // recreate resource views and depth buffers as needed.
  m_deviceResources->EnsureCameraResources(holographicFrame, prediction);

#ifdef DRAW_SAMPLE_CONTENT
  if (m_stationaryReferenceFrame != nullptr) {
    // Check for new input state since the last frame.
    for (GamepadWithButtonState &gamepadWithButtonState : m_gamepads) {
      bool buttonDownThisUpdate =
          ((gamepadWithButtonState.gamepad.GetCurrentReading().Buttons &
            GamepadButtons::A) == GamepadButtons::A);
      if (buttonDownThisUpdate &&
          !gamepadWithButtonState.buttonAWasPressedLastFrame) {
        m_pointerPressed = true;
      }
      gamepadWithButtonState.buttonAWasPressedLastFrame = buttonDownThisUpdate;
    }

    SpatialInteractionSourceState pointerState =
        m_spatialInputHandler->CheckForInput();
    SpatialPointerPose pose = nullptr;
    if (pointerState != nullptr) {
      pose = pointerState.TryGetPointerPose(
          m_stationaryReferenceFrame.CoordinateSystem());
    } else if (m_pointerPressed) {
      pose = SpatialPointerPose::TryGetAtTimestamp(
          m_stationaryReferenceFrame.CoordinateSystem(),
          prediction.Timestamp());
    }
    m_pointerPressed = false;

    // When a Pressed gesture is detected, the sample hologram will be
    // repositioned two meters in front of the user.
    m_spinningCubeRenderer->PositionHologram(pose);
  }
#endif

  m_timer.Tick([this]() {
  //
  // TODO: Update scene objects.
  //
  // Put time-based updates here. By default this code will run once per frame,
  // but if you change the StepTimer to use a fixed time step this code will
  // run as many times as needed to get to the current step.
  //

#ifdef DRAW_SAMPLE_CONTENT
    m_spinningCubeRenderer->Update(m_timer);
#endif
  });

  if (!m_canCommitDirect3D11DepthBuffer) {
    // On versions of the platform that do not support the
    // CommitDirect3D11DepthBuffer API, we can control image stabilization by
    // setting a focus point with optional plane normal and velocity.
    for (HolographicCameraPose const &cameraPose : prediction.CameraPoses()) {
#ifdef DRAW_SAMPLE_CONTENT
      // The HolographicCameraRenderingParameters class provides access to set
      // the image stabilization parameters.
      HolographicCameraRenderingParameters renderingParameters =
          holographicFrame.GetRenderingParameters(cameraPose);

      // SetFocusPoint informs the system about a specific point in your scene
      // to prioritize for image stabilization. The focus point is set
      // independently for each holographic camera. When setting the focus
      // point, put it on or near content that the user is looking at. In this
      // example, we put the focus point at the center of the sample hologram.
      // You can also set the relative velocity and facing of the stabilization
      // plane using overloads of this method.
      if (m_stationaryReferenceFrame != nullptr) {
        renderingParameters.SetFocusPoint(
            m_stationaryReferenceFrame.CoordinateSystem(),
            m_spinningCubeRenderer->GetPosition());
      }
#endif
    }
  }

  // The holographic frame will be used to get up-to-date view and projection
  // matrices and to present the swap chain.
  return holographicFrame;
}

// Renders the current frame to each holographic camera, according to the
// current application and spatial positioning state. Returns true if the
// frame was rendered to at least one camera.
bool ImmersiveMain::Render(HolographicFrame const &holographicFrame) {
  // Don't try to render anything before the first Update.
  if (m_timer.GetFrameCount() == 0) {
    return false;
  }

  //
  // TODO: Add code for pre-pass rendering here.
  //
  // Take care of any tasks that are not specific to an individual holographic
  // camera. This includes anything that doesn't need the final view or
  // projection matrix, such as lighting maps.
  //

  // Lock the set of holographic camera resources, then draw to each camera
  // in this frame.
  return m_deviceResources->UseHolographicCameraResources<bool>(
      [this,
       holographicFrame](std::map<UINT32, std::unique_ptr<DX::CameraResources>>
                             &cameraResourceMap) {
        // Up-to-date frame predictions enhance the effectiveness of image
        // stablization and allow more accurate positioning of holograms.
        holographicFrame.UpdateCurrentPrediction();
        HolographicFramePrediction prediction =
            holographicFrame.CurrentPrediction();

        bool atLeastOneCameraRendered = false;
        for (HolographicCameraPose const &cameraPose :
             prediction.CameraPoses()) {
          // This represents the device-based resources for a HolographicCamera.
          DX::CameraResources *pCameraResources =
              cameraResourceMap[cameraPose.HolographicCamera().Id()].get();

          // Get the device context.
          const auto context = m_deviceResources->GetD3DDeviceContext();
          const auto depthStencilView = pCameraResources->GetDepthStencilView();

          // Set render targets to the current holographic camera.
          ID3D11RenderTargetView *const targets[1] = {
              pCameraResources->GetBackBufferRenderTargetView()};
          context->OMSetRenderTargets(1, targets, depthStencilView);

          // Clear the back buffer and depth stencil view.
          if (m_canGetHolographicDisplayForCamera &&
              cameraPose.HolographicCamera().Display().IsOpaque()) {
            context->ClearRenderTargetView(targets[0],
                                           DirectX::Colors::CornflowerBlue);
          } else {
            context->ClearRenderTargetView(targets[0],
                                           DirectX::Colors::Transparent);
          }
          context->ClearDepthStencilView(
              depthStencilView, D3D11_CLEAR_DEPTH | D3D11_CLEAR_STENCIL, 1.0f,
              0);

          //
          // TODO: Replace the sample content with your own content.
          //
          // Notes regarding holographic content:
          //    * For drawing, remember that you have the potential to fill
          //    twice as many pixels
          //      in a stereoscopic render target as compared to a
          //      non-stereoscopic render target of the same resolution. Avoid
          //      unnecessary or repeated writes to the same pixel, and only
          //      draw holograms that the user can see.
          //    * To help occlude hologram geometry, you can create a depth map
          //    using geometry
          //      data obtained via the surface mapping APIs. You can use this
          //      depth map to avoid rendering holograms that are intended to be
          //      hidden behind tables, walls, monitors, and so on.
          //    * On HolographicDisplays that are transparent, black pixels will
          //    appear transparent
          //      to the user. On such devices, you should clear the screen to
          //      Transparent as shown above. You should still use alpha
          //      blending to draw semitransparent holograms.
          //

          // The view and projection matrices for each holographic camera will
          // change every frame. This function refreshes the data in the
          // constant buffer for the holographic camera indicated by cameraPose.
          if (m_stationaryReferenceFrame) {
            pCameraResources->UpdateViewProjectionBuffer(
                m_deviceResources, cameraPose,
                m_stationaryReferenceFrame.CoordinateSystem());
          }

          // Attach the view/projection constant buffer for this camera to the
          // graphics pipeline.
          bool cameraActive =
              pCameraResources->AttachViewProjectionBuffer(m_deviceResources);

#ifdef DRAW_SAMPLE_CONTENT
          // Only render world-locked content when positional tracking is
          // active.
          if (cameraActive) {
            // Draw the sample hologram.
            m_spinningCubeRenderer->Render();
            if (m_canCommitDirect3D11DepthBuffer) {
              // On versions of the platform that support the
              // CommitDirect3D11DepthBuffer API, we can provide the depth
              // buffer to the system, and it will use depth information to
              // stabilize the image at a per-pixel level.
              HolographicCameraRenderingParameters renderingParameters =
                  holographicFrame.GetRenderingParameters(cameraPose);

              IDirect3DSurface interopSurface =
                  DX::CreateDepthTextureInteropObject(
                      pCameraResources->GetDepthStencilTexture2D());

              // Calling CommitDirect3D11DepthBuffer causes the system to queue
              // Direct3D commands to read the depth buffer. It will then use
              // that information to stabilize the image as the HolographicFrame
              // is presented.
              renderingParameters.CommitDirect3D11DepthBuffer(interopSurface);
            }
          }
#endif
          atLeastOneCameraRendered = true;
        }

        return atLeastOneCameraRendered;
      });
}

void ImmersiveMain::SaveAppState() {
  //
  // TODO: Insert code here to save your app state.
  //       This method is called when the app is about to suspend.
  //
  //       For example, store information in the SpatialAnchorStore.
  //
}

void ImmersiveMain::LoadAppState() {
  //
  // TODO: Insert code here to load your app state.
  //       This method is called when the app resumes.
  //
  //       For example, load information from the SpatialAnchorStore.
  //
}

void ImmersiveMain::OnPointerPressed() { m_pointerPressed = true; }

// Notifies classes that use Direct3D device resources that the device resources
// need to be released before this method returns.
void ImmersiveMain::OnDeviceLost() {
#ifdef DRAW_SAMPLE_CONTENT
  m_spinningCubeRenderer->ReleaseDeviceDependentResources();
#endif
}

// Notifies classes that use Direct3D device resources that the device resources
// may now be recreated.
void ImmersiveMain::OnDeviceRestored() {
#ifdef DRAW_SAMPLE_CONTENT
  m_spinningCubeRenderer->CreateDeviceDependentResources();
#endif
}

void ImmersiveMain::OnLocatabilityChanged(
    SpatialLocator const &sender,
    winrt::Windows::Foundation::IInspectable const &) {
  switch (sender.Locatability()) {
  case SpatialLocatability::Unavailable:
    // Holograms cannot be rendered.
    {
      winrt::hstring message(L"Warning! Positional tracking is " +
                             std::to_wstring(int(sender.Locatability())) +
                             L".\n");
      OutputDebugStringW(message.data());
    }
    break;

  // In the following three cases, it is still possible to place holograms using
  // a SpatialLocatorAttachedFrameOfReference.
  case SpatialLocatability::PositionalTrackingActivating:
    // The system is preparing to use positional tracking.

  case SpatialLocatability::OrientationOnly:
    // Positional tracking has not been activated.

  case SpatialLocatability::PositionalTrackingInhibited:
    // Positional tracking is temporarily inhibited. User action may be required
    // in order to restore positional tracking.
    break;

  case SpatialLocatability::PositionalTrackingActive:
    // Positional tracking is active. World-locked content can be rendered.
    break;
  }
}

void ImmersiveMain::OnCameraAdded(
    HolographicSpace const &,
    HolographicSpaceCameraAddedEventArgs const &args) {
  winrt::Windows::Foundation::Deferral deferral = args.GetDeferral();
  HolographicCamera holographicCamera = args.Camera();
  create_task([this, deferral, holographicCamera]() {
    //
    // TODO: Allocate resources for the new camera and load any content specific
    // to
    //       that camera. Note that the render target size (in pixels) is a
    //       property of the HolographicCamera object, and can be used to create
    //       off-screen render targets that match the resolution of the
    //       HolographicCamera.
    //

    // Create device-based resources for the holographic camera and add it to
    // the list of cameras used for updates and rendering. Notes:
    //   * Since this function may be called at any time, the
    //   AddHolographicCamera function
    //     waits until it can get a lock on the set of holographic camera
    //     resources before adding the new camera. At 60 frames per second this
    //     wait should not take long.
    //   * A subsequent Update will take the back buffer from the
    //   RenderingParameters of this
    //     camera's CameraPose and use it to create the ID3D11RenderTargetView
    //     for this camera. Content can then be rendered for the
    //     HolographicCamera.
    m_deviceResources->AddHolographicCamera(holographicCamera);

    // Holographic frame predictions will not include any information about this
    // camera until the deferral is completed.
    deferral.Complete();
  });
}

void ImmersiveMain::OnCameraRemoved(
    HolographicSpace const &,
    HolographicSpaceCameraRemovedEventArgs const &args) {
  create_task([this]() {
    //
    // TODO: Asynchronously unload or deactivate content resources (not back
    // buffer
    //       resources) that are specific only to the camera that was removed.
    //
  });

  // Before letting this callback return, ensure that all references to the back
  // buffer are released. Since this function may be called at any time, the
  // RemoveHolographicCamera function waits until it can get a lock on the set
  // of holographic camera resources before deallocating resources for this
  // camera. At 60 frames per second this wait should not take long.
  m_deviceResources->RemoveHolographicCamera(args.Camera());
}

void ImmersiveMain::OnGamepadAdded(winrt::Windows::Foundation::IInspectable,
                                   Gamepad const &args) {
  for (GamepadWithButtonState const &gamepadWithButtonState : m_gamepads) {
    if (args == gamepadWithButtonState.gamepad) {
      // This gamepad is already in the list.
      return;
    }
  }

  GamepadWithButtonState newGamepad = {args, false};
  m_gamepads.push_back(newGamepad);
}

void ImmersiveMain::OnGamepadRemoved(winrt::Windows::Foundation::IInspectable,
                                     Gamepad const &args) {
  m_gamepads.erase(
      std::remove_if(m_gamepads.begin(), m_gamepads.end(),
                     [&](GamepadWithButtonState &gamepadWithState) {
                       return gamepadWithState.gamepad == args;
                     }),
      m_gamepads.end());
}

void ImmersiveMain::OnHolographicDisplayIsAvailableChanged(
    winrt::Windows::Foundation::IInspectable,
    winrt::Windows::Foundation::IInspectable) {
  // Get the spatial locator for the default HolographicDisplay, if one is
  // available.
  SpatialLocator spatialLocator = nullptr;
  if (m_canGetDefaultHolographicDisplay) {
    HolographicDisplay defaultHolographicDisplay =
        HolographicDisplay::GetDefault();
    if (defaultHolographicDisplay) {
      spatialLocator = defaultHolographicDisplay.SpatialLocator();
    }
  } else {
    spatialLocator = SpatialLocator::GetDefault();
  }

  if (m_spatialLocator != spatialLocator) {
    // If the spatial locator is disconnected or replaced, we should discard all
    // state that was based on it.
    if (m_spatialLocator != nullptr) {
      m_spatialLocator.LocatabilityChanged(m_locatabilityChangedToken);
      m_spatialLocator = nullptr;
    }

    m_stationaryReferenceFrame = nullptr;

    if (spatialLocator != nullptr) {
      // Use the SpatialLocator from the default HolographicDisplay to track the
      // motion of the device.
      m_spatialLocator = spatialLocator;

      // Respond to changes in the positional tracking state.
      m_locatabilityChangedToken = m_spatialLocator.LocatabilityChanged(
          std::bind(&ImmersiveMain::OnLocatabilityChanged, this, _1, _2));

      // The simplest way to render world-locked holograms is to create a
      // stationary reference frame based on a SpatialLocator. This is roughly
      // analogous to creating a "world" coordinate system with the origin
      // placed at the device's position as the app is launched.
      m_stationaryReferenceFrame =
          m_spatialLocator.CreateStationaryFrameOfReferenceAtCurrentLocation();
    }
  }
}
