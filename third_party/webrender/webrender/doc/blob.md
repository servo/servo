# Blob images

Blob image is fallback mechanism for webrender that Gecko uses to render primitives that aren't currently supported by webrender. The main idea is to provide webrender with a custom handler that can take arbitray drawing commands serialized as buffers of bytes (the blobs) and turn them into images that webrender internally will treat as regular images.

At the API level, blob images are treated as other images. They are resources created and associated with image keys, and are used in the display list with regular image display items. 


## Active area

In order to support scrolling very large content, blob images don't necessarily have a finite size. They can grow in any direction. At any time they do have an "active area", also called "visible area" which defines the portion that has to be rasterized. Typically this active area moves along large blob images depending on the scroll position.
The coordinate system of active area the *should* be the one of the blob's drawing commands (this is really up to the blob handler implementation to enforce that, Gecko does), and its scale should correspond to device pixels. The active area's coordinates can be negative.

As far as positioning goes, the active area maps to the image display item's bounds. In other words the content at the top-left corner of the active area will be rendered on screen at the position of the top-left corner of the display item's local rect.

In Gecko, the active area corresponds to the intersection of the fallback content's rect and the displayport.

The terms "visible area" and "visible rect" are used a lot in the blobs code, unfortunately they collide with frame building's visibility/culling terminology. They don't correspond to what is visible to the user, but rather what is in the displayport.


## Tiling

Blob images can be either tiled or non-tiled. Non-tiled blob images support invalid rects while tiled blob images track only validty at the tile level. In gecko all blobs are tiled with a tile size of 256x256.

Just like regular tiled images, blob image tiles along the border of the image are shrinked to fit the remaining size. The only difference is that the tiling pattern always starts at the top-left corner for regular images (smaller boundary tiles only along the right and bottom edges), while it can be aribtrarily positioned for blob images (smaller boundary tiles potentially on all sides).

The tiling logic is in webrender/src/image.rs.


## Async rasterization

Blobs are typically too slow to rasterize on the critical path. We try to avoid blocking frame building on blob image rasterization. In order to do that we rasterize blobs as part of scene building. Rather than rasterize tiles on demand from visibility informating, we rasterize the entire active area during scene building. This means we potentially process a lot more content than will be displayed if the user doesn't scroll through all of the visible area.

When the render backend receives a transaction, it looks for all new and update blob images, and generate blob rasterization requests for all tiles of the blob images that overlap their active area. The requests are bundled with an `AsyncBlobImageRasterizer` object in the transaction that is sent to the scene builder thread. The async rasterizer is created by the `BlobImageHandler` at each transaction. It is a snapshot of the state of the blobs as well as external information such as fonts, and does the actual rasterization.

While tiles are rasterized eagerly during scene building, their content is uploaded lazily to the texture cache depending on the result of the visibility pass during frame building.


## Late rasterization

In some case we run into a missing blob image during frame building and have to rasterize it synchronously. This happens when a rasterized tile is uploaded to the texture cache (at which point the CPU side is discarded), the texture cache entry expires and after scrolling back into view the tile is needed again.
We should really keep the rasterized blobs around just like we keep regular images in the cache. Hopefully this section will become obsolete eventually and we'll be able to remove late blob rasterization.

The information needed for async rasterization corresponds to the state of blobs before scene building while late rasterization needs the state of blobs after the last complete scene build. This means we have to be careful about which version we manipulate in the resource cache.
