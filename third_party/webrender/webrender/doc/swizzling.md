> It'd be great to have some (in-tree) docs describing the process you've worked through here, the overall motivation, how it works on different GPUs / platforms etc. Perhaps as a follow up?

# Swizzling in WR

## Problem statement

The goal is to avoid the CPU conversion of data done by the driver on texture data uploads. It's slow and always done synchronously, hurting our "Renderer" thread CPU utilization.

Gecko produces all of the image data in BGRA. Switching "imagelib" to RGBA is possible, but modifying Skia to follow is less trivial.
OpenGL support for BGRA8 as an internal texture format is a complex story: it's officially supported in Angle and a fair share of Android devices, but it's not available on the desktop GL (and until recently wasn't available in the Android emulator). Unofficially, when textures are initialized with `glTexImage` (as opposed to `glTexStorage`) with RGBA internal format, the desktop GL drivers often prefer to store the data in BGRA8 format, actually.

The only way to avoid the CPU conversion is to provide the data in exactly the same format that the driver is using internally for a texture. In this case, the driver does a straght `memcpy` into its CPU-visible memory, which is the best we can hope for with OpenGL API.

## Solution: swizzling

https://phabricator.services.mozilla.com/D21965 is providing the solution to this problem. The main principles are:

  1. Use `glTexStorage` whenever it's available. Doing so gives us full control of the internal format, also allows to avoid allocating memory for mipmaps that we don't use.
  2. Make the shared texture cache format to be determined at the init time, based on the GL device capabilities. For Angle and OpenGL ES this is BGRA8, for desktop this is RGBA8 (since desktop GL doesn't support BGRA internal formats). WebRender is now able to tell Gecko, which color format it prefers the texture data to use.
  3. If the data comes in a format that is different from our best case, we pretend that the data is actually in our best case format, and associate the allocated cache entry with the `Swizzle`. That swizzle configuration changes the way shaders sample from a texture, adjusting for the fact the data was provided in a different format.
  4. The lifetime of a "swizzled" texture cache data is starting at the point the data is uploaded and ending at a point where any shader samples from this data. Any other operation on that data (copying or blitting) is not configurable by `Swizzle` and thus would produce incorrect results. To address this, the change enhances `cs_copy` shader to be used in place of blitting from the texture cache, where needed.
  5. Swizzling becomes a part of the batch key per texture. Mixing up draw calls with texture data that is differently swizzled then introduces the batch breaks. This is a downside for the swizzling approach in general, but it's not clear to what extent this would affect Gecko.

## Code paths

Windows/Angle and Android:
  - we use `glTexStorage` with BGRA8 internal format, no swizzling is needed in general case.

Windows (non-Angle), Mac, Linux:
  - if `glTexStorage` is available, we use it with RGBA8 internal format, swizzling everything on texture sampling.
  - otherwise, we use RGBA unsized format with `gTexImage` and expect the data to come in BGRA format, no swizzling is involved.
