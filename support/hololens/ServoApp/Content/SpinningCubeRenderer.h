#pragma once

#include "../Common/DeviceResources.h"
#include "../Common/StepTimer.h"
#include "ShaderStructures.h"

namespace Immersive {
// This sample renderer instantiates a basic rendering pipeline.
class SpinningCubeRenderer {
public:
  SpinningCubeRenderer(
      std::shared_ptr<DX::DeviceResources> const &deviceResources);
  std::future<void> CreateDeviceDependentResources();
  void ReleaseDeviceDependentResources();
  void Update(DX::StepTimer const &timer);
  void Render();

  // Repositions the sample hologram.
  void
  PositionHologram(winrt::Windows::UI::Input::Spatial::SpatialPointerPose const
                       &pointerPose);

  // Property accessors.
  void SetPosition(winrt::Windows::Foundation::Numerics::float3 const &pos) {
    m_position = pos;
  }
  winrt::Windows::Foundation::Numerics::float3 const &GetPosition() {
    return m_position;
  }

private:
  // Cached pointer to device resources.
  std::shared_ptr<DX::DeviceResources> m_deviceResources;

  // Direct3D resources for cube geometry.
  Microsoft::WRL::ComPtr<ID3D11InputLayout> m_inputLayout;
  Microsoft::WRL::ComPtr<ID3D11Buffer> m_vertexBuffer;
  Microsoft::WRL::ComPtr<ID3D11Buffer> m_indexBuffer;
  Microsoft::WRL::ComPtr<ID3D11VertexShader> m_vertexShader;
  Microsoft::WRL::ComPtr<ID3D11GeometryShader> m_geometryShader;
  Microsoft::WRL::ComPtr<ID3D11PixelShader> m_pixelShader;
  Microsoft::WRL::ComPtr<ID3D11Buffer> m_modelConstantBuffer;

  // System resources for cube geometry.
  ModelConstantBuffer m_modelConstantBufferData;
  uint32_t m_indexCount = 0;

  // Variables used with the rendering loop.
  bool m_loadingComplete = false;
  float m_degreesPerSecond = 45.f;
  winrt::Windows::Foundation::Numerics::float3 m_position = {0.f, 0.f, -2.f};

  // If the current D3D Device supports VPRT, we can avoid using a geometry
  // shader just to set the render target array index.
  bool m_usingVprtShaders = false;
};
} // namespace Immersive
