#pragma once

namespace DX {
// Function that reads from a binary file asynchronously.
inline std::future<std::vector<byte>>
ReadDataAsync(const std::wstring_view &filename) {
  using namespace winrt::Windows::Storage;
  using namespace winrt::Windows::Storage::Streams;

  IBuffer fileBuffer = co_await PathIO::ReadBufferAsync(filename);

  std::vector<byte> returnBuffer;
  returnBuffer.resize(fileBuffer.Length());
  DataReader::FromBuffer(fileBuffer)
      .ReadBytes(winrt::array_view<uint8_t>(returnBuffer));
  return returnBuffer;
}

// Converts a length in device-independent pixels (DIPs) to a length in physical
// pixels.
inline float ConvertDipsToPixels(float dips, float dpi) {
  constexpr float dipsPerInch = 96.0f;
  return floorf(dips * dpi / dipsPerInch + 0.5f); // Round to nearest integer.
}

inline winrt::Windows::Graphics::DirectX::Direct3D11::IDirect3DSurface
CreateDepthTextureInteropObject(
    const Microsoft::WRL::ComPtr<ID3D11Texture2D> spTexture2D) {
  // Direct3D interop APIs are used to provide the buffer to the WinRT API.
  Microsoft::WRL::ComPtr<IDXGIResource1> depthStencilResource;
  winrt::check_hresult(spTexture2D.As(&depthStencilResource));
  Microsoft::WRL::ComPtr<IDXGISurface2> depthDxgiSurface;
  winrt::check_hresult(
      depthStencilResource->CreateSubresourceSurface(0, &depthDxgiSurface));
  winrt::com_ptr<::IInspectable> inspectableSurface;
  winrt::check_hresult(CreateDirect3D11SurfaceFromDXGISurface(
      depthDxgiSurface.Get(),
      reinterpret_cast<IInspectable **>(winrt::put_abi(inspectableSurface))));

  return inspectableSurface
      .as<winrt::Windows::Graphics::DirectX::Direct3D11::IDirect3DSurface>();
}

#if defined(_DEBUG)
// Check for SDK Layer support.
inline bool SdkLayersAvailable() {
  HRESULT hr = D3D11CreateDevice(
      nullptr,
      D3D_DRIVER_TYPE_NULL, // There is no need to create a real hardware
                            // device.
      0,
      D3D11_CREATE_DEVICE_DEBUG, // Check for the SDK layers.
      nullptr,                   // Any feature level will do.
      0,
      D3D11_SDK_VERSION, // Always set this to D3D11_SDK_VERSION for Windows
                         // Runtime apps.
      nullptr,           // No need to keep the D3D device reference.
      nullptr,           // No need to know the feature level.
      nullptr            // No need to keep the D3D device context reference.
  );

  return SUCCEEDED(hr);
}
#endif
} // namespace DX
