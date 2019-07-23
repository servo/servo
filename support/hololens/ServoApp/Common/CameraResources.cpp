#include "pch.h"

#include "CameraResources.h"
#include "Common/DirectXHelper.h"
#include "DeviceResources.h"

using namespace DirectX;
using namespace Microsoft::WRL;
using namespace winrt::Windows::Foundation::Numerics;
using namespace winrt::Windows::Graphics::DirectX::Direct3D11;
using namespace winrt::Windows::Graphics::Holographic;
using namespace winrt::Windows::Perception::Spatial;

DX::CameraResources::CameraResources(HolographicCamera const &camera)
    : m_holographicCamera(camera), m_isStereo(camera.IsStereo()),
      m_d3dRenderTargetSize(camera.RenderTargetSize()) {
  m_d3dViewport = CD3D11_VIEWPORT(0.f, 0.f, m_d3dRenderTargetSize.Width,
                                  m_d3dRenderTargetSize.Height);
};

// Updates resources associated with a holographic camera's swap chain.
// The app does not access the swap chain directly, but it does create
// resource views for the back buffer.
void DX::CameraResources::CreateResourcesForBackBuffer(
    DX::DeviceResources *pDeviceResources,
    HolographicCameraRenderingParameters const &cameraParameters) {
  ID3D11Device *device = pDeviceResources->GetD3DDevice();

  // Get the WinRT object representing the holographic camera's back buffer.
  IDirect3DSurface surface = cameraParameters.Direct3D11BackBuffer();

  // Get the holographic camera's back buffer.
  // Holographic apps do not create a swap chain themselves; instead, buffers
  // are owned by the system. The Direct3D back buffer resources are provided to
  // the app using WinRT interop APIs.
  ComPtr<ID3D11Texture2D> cameraBackBuffer;
  winrt::check_hresult(surface
                           .as<::Windows::Graphics::DirectX::Direct3D11::
                                   IDirect3DDxgiInterfaceAccess>()
                           ->GetInterface(IID_PPV_ARGS(&cameraBackBuffer)));

  // Determine if the back buffer has changed. If so, ensure that the render
  // target view is for the current back buffer.
  if (m_d3dBackBuffer.Get() != cameraBackBuffer.Get()) {
    // This can change every frame as the system moves to the next buffer in the
    // swap chain. This mode of operation will occur when certain rendering
    // modes are activated.
    m_d3dBackBuffer = cameraBackBuffer;

    // Create a render target view of the back buffer.
    // Creating this resource is inexpensive, and is better than keeping track
    // of the back buffers in order to pre-allocate render target views for each
    // one.
    winrt::check_hresult(device->CreateRenderTargetView(
        m_d3dBackBuffer.Get(), nullptr, &m_d3dRenderTargetView));

    // Get the DXGI format for the back buffer.
    // This information can be accessed by the app using
    // CameraResources::GetBackBufferDXGIFormat().
    D3D11_TEXTURE2D_DESC backBufferDesc;
    m_d3dBackBuffer->GetDesc(&backBufferDesc);
    m_dxgiFormat = backBufferDesc.Format;

    // Check for render target size changes.
    winrt::Windows::Foundation::Size currentSize =
        m_holographicCamera.RenderTargetSize();
    if (m_d3dRenderTargetSize != currentSize) {
      // Set render target size.
      m_d3dRenderTargetSize = currentSize;

      // A new depth stencil view is also needed.
      m_d3dDepthStencilView.Reset();
    }
  }

  // Refresh depth stencil resources, if needed.
  if (m_d3dDepthStencilView == nullptr) {
    // Create a depth stencil view for use with 3D rendering if needed.
    CD3D11_TEXTURE2D_DESC depthStencilDesc(
        DXGI_FORMAT_R16_TYPELESS,
        static_cast<UINT>(m_d3dRenderTargetSize.Width),
        static_cast<UINT>(m_d3dRenderTargetSize.Height),
        m_isStereo ? 2 : 1, // Create two textures when rendering in stereo.
        1,                  // Use a single mipmap level.
        D3D11_BIND_DEPTH_STENCIL | D3D11_BIND_SHADER_RESOURCE);

    winrt::check_hresult(device->CreateTexture2D(&depthStencilDesc, nullptr,
                                                 &m_d3dDepthStencil));

    CD3D11_DEPTH_STENCIL_VIEW_DESC depthStencilViewDesc(
        m_isStereo ? D3D11_DSV_DIMENSION_TEXTURE2DARRAY
                   : D3D11_DSV_DIMENSION_TEXTURE2D,
        DXGI_FORMAT_D16_UNORM);
    winrt::check_hresult(device->CreateDepthStencilView(
        m_d3dDepthStencil.Get(), &depthStencilViewDesc,
        &m_d3dDepthStencilView));
  }

  // Create the constant buffer, if needed.
  if (m_viewProjectionConstantBuffer == nullptr) {
    // Create a constant buffer to store view and projection matrices for the
    // camera.
    CD3D11_BUFFER_DESC constantBufferDesc(sizeof(ViewProjectionConstantBuffer),
                                          D3D11_BIND_CONSTANT_BUFFER);
    winrt::check_hresult(device->CreateBuffer(&constantBufferDesc, nullptr,
                                              &m_viewProjectionConstantBuffer));
  }
}

// Releases resources associated with a back buffer.
void DX::CameraResources::ReleaseResourcesForBackBuffer(
    DX::DeviceResources *pDeviceResources) {
  ID3D11DeviceContext *context = pDeviceResources->GetD3DDeviceContext();

  // Release camera-specific resources.
  m_d3dBackBuffer.Reset();
  m_d3dDepthStencil.Reset();
  m_d3dRenderTargetView.Reset();
  m_d3dDepthStencilView.Reset();
  m_viewProjectionConstantBuffer.Reset();

  // Ensure system references to the back buffer are released by clearing the
  // render target from the graphics pipeline state, and then flushing the
  // Direct3D context.
  ID3D11RenderTargetView *nullViews[D3D11_SIMULTANEOUS_RENDER_TARGET_COUNT] = {
      nullptr};
  context->OMSetRenderTargets(ARRAYSIZE(nullViews), nullViews, nullptr);
  context->Flush();
}

// Updates the view/projection constant buffer for a holographic camera.
void DX::CameraResources::UpdateViewProjectionBuffer(
    std::shared_ptr<DX::DeviceResources> deviceResources,
    HolographicCameraPose const &cameraPose,
    SpatialCoordinateSystem const &coordinateSystem) {
  // The system changes the viewport on a per-frame basis for system
  // optimizations.
  auto viewport = cameraPose.Viewport();
  m_d3dViewport =
      CD3D11_VIEWPORT(viewport.X, viewport.Y, viewport.Width, viewport.Height);

  // The projection transform for each frame is provided by the
  // HolographicCameraPose.
  HolographicStereoTransform cameraProjectionTransform =
      cameraPose.ProjectionTransform();

  // Get a container object with the view and projection matrices for the given
  // pose in the given coordinate system.
  auto viewTransformContainer =
      cameraPose.TryGetViewTransform(coordinateSystem);

  // If TryGetViewTransform returns a null pointer, that means the pose and
  // coordinate system cannot be understood relative to one another; content
  // cannot be rendered in this coordinate system for the duration of the
  // current frame. This usually means that positional tracking is not active
  // for the current frame, in which case it is possible to use a
  // SpatialLocatorAttachedFrameOfReference to render content that is not
  // world-locked instead.
  DX::ViewProjectionConstantBuffer viewProjectionConstantBufferData;
  bool viewTransformAcquired = viewTransformContainer != nullptr;
  if (viewTransformAcquired) {
    // Otherwise, the set of view transforms can be retrieved.
    HolographicStereoTransform viewCoordinateSystemTransform =
        viewTransformContainer.Value();

    // Update the view matrices. Holographic cameras (such as Microsoft
    // HoloLens) are constantly moving relative to the world. The view matrices
    // need to be updated every frame.
    XMStoreFloat4x4(
        &viewProjectionConstantBufferData.viewProjection[0],
        XMMatrixTranspose(XMLoadFloat4x4(&viewCoordinateSystemTransform.Left) *
                          XMLoadFloat4x4(&cameraProjectionTransform.Left)));
    XMStoreFloat4x4(
        &viewProjectionConstantBufferData.viewProjection[1],
        XMMatrixTranspose(XMLoadFloat4x4(&viewCoordinateSystemTransform.Right) *
                          XMLoadFloat4x4(&cameraProjectionTransform.Right)));
  }

  // Use the D3D device context to update Direct3D device-based resources.
  ID3D11DeviceContext *context = deviceResources->GetD3DDeviceContext();

  // Loading is asynchronous. Resources must be created before they can be
  // updated.
  if (context == nullptr || m_viewProjectionConstantBuffer == nullptr ||
      !viewTransformAcquired) {
    m_framePending = false;
  } else {
    // Update the view and projection matrices.
    context->UpdateSubresource(m_viewProjectionConstantBuffer.Get(), 0, nullptr,
                               &viewProjectionConstantBufferData, 0, 0);

    m_framePending = true;
  }
}

// Gets the view-projection constant buffer for the HolographicCamera and
// attaches it to the shader pipeline.
bool DX::CameraResources::AttachViewProjectionBuffer(
    std::shared_ptr<DX::DeviceResources> &deviceResources) {
  // This method uses Direct3D device-based resources.
  ID3D11DeviceContext *context = deviceResources->GetD3DDeviceContext();

  // Loading is asynchronous. Resources must be created before they can be
  // updated. Cameras can also be added asynchronously, in which case they must
  // be initialized before they can be used.
  if (context == nullptr || m_viewProjectionConstantBuffer == nullptr ||
      m_framePending == false) {
    return false;
  }

  // Set the viewport for this camera.
  context->RSSetViewports(1, &m_d3dViewport);

  // Send the constant buffer to the vertex shader.
  context->VSSetConstantBuffers(1, 1,
                                m_viewProjectionConstantBuffer.GetAddressOf());

  // The template includes a pass-through geometry shader that is used by
  // default on systems that don't support the D3D11_FEATURE_D3D11_OPTIONS3::
  // VPAndRTArrayIndexFromAnyShaderFeedingRasterizer extension. The shader
  // will be enabled at run-time on systems that require it.
  // If your app will also use the geometry shader for other tasks and those
  // tasks require the view/projection matrix, uncomment the following line
  // of code to send the constant buffer to the geometry shader as well.
  /*context->GSSetConstantBuffers(
  1,
  1,
  m_viewProjectionConstantBuffer.GetAddressOf()
  );*/

  m_framePending = false;

  return true;
}
