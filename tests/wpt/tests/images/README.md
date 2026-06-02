Largest contentful paint test images
====================================

The images in this directory prefixed with `lcp-` are specifically intended to
be used by tests where a largest-contentful-paint entry should be generated. As
Chromium has implemented a contentfulness threshold of 0.05 bits per pixel,
these images, when rendered at their natural dimensions, have a minimum content
length, as shown in the table below:

File            | Dimensions | Area (px^2) | Minimum content-length (bytes)
----------------+------------+-------------|-------------------------------
lcp-256x256.png | 256x256    | 65536       | 410
lcp-133x106.png | 133x106    | 14098       | 89
lcp-100x50.png  | 100x50     | 5000        | 32
lcp-96x96.png   | 96x96      | 9216        | 58
lcp-16x16.png   | 16x16      | 256         | 2
lcp-2x2.png     | 2x2        | 4           | 1
lcp-1x1.png     | 1x1        | 1           | 1
