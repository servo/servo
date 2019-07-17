#pragma once

namespace DX {
class DeviceResources;

// Constant buffer used to send the view-projection matrices to the shader
// pipeline.
struct ViewProjectionConstantBuffer {
  DirectX::XMFLOAT4X4 viewProjection[2];
};

// Assert that the constant buffer remains 16-byte aligned (best practice).
static_assert((sizeof(ViewProjectionConstantBuffer) % (sizeof(float) * 4)) == 0,
              "ViewProjection constant buffer size must be 16-byte aligned (16 "
              "bytes is the length of four floats).");

// Manages DirectX device resources that are specific to a holographic camera,
// such as the back buffer, ViewProjection constant buffer, and viewport.
class CameraResources {
public:
  CameraResources(winrt::Windows::Graphics::Holographic::HolographicCamera const
                      &holographicCamera);

  void CreateResourcesForBackBuffer(
      DX::DeviceResources *pDeviceResources,
      winrt::Windows::Graphics::Holographic::
          HolographicCameraRenderingParameters const &cameraParameters);
  void ReleaseResourcesForBackBuffer(DX::DeviceResources *pDeviceResources);

  void UpdateViewProjectionBuffer(
      std::shared_ptr<DX::DeviceResources> deviceResources,
      winrt::Windows::Graphics::Holographic::HolographicCameraPose const
          &cameraPose,
      winrt::Windows::Perception::Spatial::SpatialCoordinateSystem const
          &coordinateSystem);

  bool AttachViewProjectionBuffer(
      std::shared_ptr<DX::DeviceResources> &deviceResources);

  // Direct3D device resources.
  ID3D11RenderTargetView *GetBackBufferRenderTargetView() const {
    return m_d3dRenderTargetView.Get();
  }
  ID3D11DepthStencilView *GetDepthStencilView() const {
    return m_d3dDepthStencilView.Get();
  }
  ID3D11Texture2D *GetBackBufferTexture2D() const {
    return m_d3dBackBuffer.Get();
  }
  ID3D11Texture2D *GetDepthStencilTexture2D() const {
    return m_d3dDepthStencil.Get();
  }
  D3D11_VIEWPORT GetViewport() const { return m_d3dViewport; }
  DXGI_FORMAT GetBackBufferDXGIFormat() const { return m_dxgiFormat; }

  // Render target properties.
  winrt::Windows::Foundation::Size GetRenderTargetSize() const & {
    return m_d3dRenderTargetSize;
  }
  bool IsRenderingStereoscopic() const { return m_isStereo; }

  // The holographic camera these resources are for.
  winrt::Windows::Graphics::Holographic::HolographicCamera const &
  GetHolographicCamera() const {
    return m_holographicCamera;
  }

private:
  // Direct3D rendering objects. Required for 3D.
  Microsoft::WRL::ComPtr<ID3D11RenderTargetView> m_d3dRenderTargetView;
  Microsoft::WRL::ComPtr<ID3D11DepthStencilView> m_d3dDepthStencilView;
  Microsoft::WRL::ComPtr<ID3D11Texture2D> m_d3dBackBuffer;
  Microsoft::WRL::ComPtr<ID3D11Texture2D> m_d3dDepthStencil;

  // Device resource to store view and projection matrices.
  Microsoft::WRL::ComPtr<ID3D11Buffer> m_viewProjectionConstantBuffer;

  // Direct3D rendering properties.
  DXGI_FORMAT m_dxgiFormat;
  winrt::Windows::Foundation::Size m_d3dRenderTargetSize;
  D3D11_VIEWPORT m_d3dViewport;

  // Indicates whether the camera supports stereoscopic rendering.
  bool m_isStereo = false;

  // Indicates whether this camera has a pending frame.
  bool m_framePending = false;

  // Pointer to the holographic camera these resources are for.
  winrt::Windows::Graphics::Holographic::HolographicCamera m_holographicCamera =
      nullptr;
};
} // namespace DX
