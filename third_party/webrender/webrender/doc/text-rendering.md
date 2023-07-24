# Text Rendering

This document describes the details of how WebRender renders text, particularly the blending stage of text rendering.
We will go into grayscale text blending, subpixel text blending, and "subpixel text with background color" blending.

### Prerequisites

The description below assumes you're familiar with regular rgba compositing, operator over,
and the concept of premultiplied alpha.

### Not covered in this document

We are going to treat the origin of the text mask as a black box.
We're also going to assume we can blend text in the device color space and will not go into the gamma correction and linear pre-blending that happens in some of the backends that produce the text masks.

## Grayscale Text Blending

Grayscale text blending is the simplest form of text blending. Our blending function has three inputs:

 - The text color, as a premultiplied rgba color.
 - The text mask, as a single-channel alpha texture.
 - The existing contents of the framebuffer that we're rendering to, the "destination". This is also a premultiplied rgba buffer.

Note: The word "grayscale" here does *not* mean that we can only draw gray text.
It means that the mask only has a single alpha value per pixel, so we can visualize
the mask in our minds as a grayscale image.

### Deriving the math

We want to mask our text color using the single-channel mask, and composite that to the destination.
This compositing step uses operator "over", just like regular compositing of rgba images.

I'll be using GLSL syntax to describe the blend equations, but please consider most of the code below pseudocode.

We can express the blending described above as the following blend equation:

```glsl
vec4 textblend(vec4 text_color, vec4 mask, vec4 dest) {
  return over(in(text_color, mask), dest);
}
```

with `over` being the blend function for (premultiplied) operator "over":

```glsl
vec4 over(vec4 src, vec4 dest) {
  return src + (1.0 - src.a) * dest;
}
```

and `in` being the blend function for (premultiplied) operator "in", i.e. the masking operator:

```glsl
vec4 in(vec4 src, vec4 mask) {
  return src * mask.a;
}
```

So the complete blending function is:

```glsl
result.r = text_color.r * mask.a + (1.0 - text_color.a * mask.a) * dest.r;
result.g = text_color.g * mask.a + (1.0 - text_color.a * mask.a) * dest.g;
result.b = text_color.b * mask.a + (1.0 - text_color.a * mask.a) * dest.b;
result.a = text_color.a * mask.a + (1.0 - text_color.a * mask.a) * dest.a;
```

### Rendering this with OpenGL

In general, a fragment shader does not have access to the destination.
So the full blend equation needs to be expressed in a way that the shader only computes values that are independent of the destination,
and the parts of the equation that use the destination values need to be applied by the OpenGL blend pipeline itself.
The OpenGL blend pipeline can be tweaked using the functions `glBlendEquation` and `glBlendFunc`.

In our example, the fragment shader can output just `text_color * mask.a`:

```glsl
  oFragColor = text_color * mask.a;
```

and the OpenGL blend pipeline can be configured like so:

```rust
    pub fn set_blend_mode_premultiplied_alpha(&self) {
        self.gl.blend_func(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        self.gl.blend_equation(gl::FUNC_ADD);
    }
```

This results in an overall blend equation of

```
result.r = 1 * oFragColor.r + (1 - oFragColor.a) * dest.r;
           ^                ^  ^^^^^^^^^^^^^^^^^
           |                |         |
           +--gl::ONE       |         +-- gl::ONE_MINUS_SRC_ALPHA
                            |
                            +-- gl::FUNC_ADD

         = 1 * (text_color.r * mask.a) + (1 - (text_color.a * mask.a)) * dest.r
         = text_color.r * mask.a + (1 - text_color.a * mask.a) * dest.r
```

which is exactly what we wanted.

### Differences to the actual WebRender code

There are two minor differences between the shader code above and the actual code in the text run shader in WebRender:

```glsl
oFragColor = text_color * mask.a;    // (shown above)
// vs.
oFragColor = vColor * mask * alpha;  // (actual webrender code)
```

`vColor` is set to the text color. The differences are:

 - WebRender multiplies with all components of `mask` instead of just with `mask.a`.
   However, our font rasterization code fills the rgb values of `mask` with the value of `mask.a`,
   so this is completely equivalent.
 - WebRender applies another alpha to the text. This is coming from the clip.
   You can think of this alpha to be a pre-adjustment of the text color for that pixel, or as an
   additional mask that gets applied to the mask.

## Subpixel Text Blending

Now that we have the blend equation for single-channel text blending, we can look at subpixel text blending.

The main difference between subpixel text blending and grayscale text blending is the fact that,
for subpixel text, the text mask contains a separate alpha value for each color component.

### Component alpha

Regular painting uses four values per pixel: three color values, and one alpha value. The alpha value applies to all components of the pixel equally.

Imagine for a second a world in which you have *three alpha values per pixel*, one for each color component.

 - Old world: Each pixel has four values: `color.r`, `color.g`, `color.b`, and `color.a`.
 - New world: Each pixel has *six* values: `color.r`, `color.a_r`, `color.g`, `color.a_g`, `color.b`, and `color.a_b`.

In such a world we can define a component-alpha-aware operator "over":

```glsl
vec6 over_comp(vec6 src, vec6 dest) {
  vec6 result;
  result.r = src.r + (1.0 - src.a_r) * dest.r;
  result.g = src.g + (1.0 - src.a_g) * dest.g;
  result.b = src.b + (1.0 - src.a_b) * dest.b;
  result.a_r = src.a_r + (1.0 - src.a_r) * dest.a_r;
  result.a_g = src.a_g + (1.0 - src.a_g) * dest.a_g;
  result.a_b = src.a_b + (1.0 - src.a_b) * dest.a_b;
  return result;
}
```

and a component-alpha-aware operator "in":

```glsl
vec6 in_comp(vec6 src, vec6 mask) {
  vec6 result;
  result.r = src.r * mask.a_r;
  result.g = src.g * mask.a_g;
  result.b = src.b * mask.a_b;
  result.a_r = src.a_r * mask.a_r;
  result.a_g = src.a_g * mask.a_g;
  result.a_b = src.a_b * mask.a_b;
  return result;
}
```

and even a component-alpha-aware version of `textblend`:

```glsl
vec6 textblend_comp(vec6 text_color, vec6 mask, vec6 dest) {
  return over_comp(in_comp(text_color, mask), dest);
}
```

This results in the following set of equations:

```glsl
result.r = text_color.r * mask.a_r + (1.0 - text_color.a_r * mask.a_r) * dest.r;
result.g = text_color.g * mask.a_g + (1.0 - text_color.a_g * mask.a_g) * dest.g;
result.b = text_color.b * mask.a_b + (1.0 - text_color.a_b * mask.a_b) * dest.b;
result.a_r = text_color.a_r * mask.a_r + (1.0 - text_color.a_r * mask.a_r) * dest.a_r;
result.a_g = text_color.a_g * mask.a_g + (1.0 - text_color.a_g * mask.a_g) * dest.a_g;
result.a_b = text_color.a_b * mask.a_b + (1.0 - text_color.a_b * mask.a_b) * dest.a_b;
```

### Back to the real world

If we want to transfer the component alpha blend equation into the real world, we need to make a few small changes:

 - Our text color only needs one alpha value.
   So we'll replace all instances of `text_color.a_r/g/b` with `text_color.a`.
 - We're currently not making use of the mask's `r`, `g` and `b` values, only of the `a_r`, `a_g` and `a_b` values.
   So in the real world, we can use the rgb channels of `mask` to store those component alphas and
   replace `mask.a_r/g/b` with `mask.r/g/b`.

These two changes give us:

```glsl
result.r = text_color.r * mask.r + (1.0 - text_color.a * mask.r) * dest.r;
result.g = text_color.g * mask.g + (1.0 - text_color.a * mask.g) * dest.g;
result.b = text_color.b * mask.b + (1.0 - text_color.a * mask.b) * dest.b;
result.a_r = text_color.a * mask.r + (1.0 - text_color.a * mask.r) * dest.a_r;
result.a_g = text_color.a * mask.g + (1.0 - text_color.a * mask.g) * dest.a_g;
result.a_b = text_color.a * mask.b + (1.0 - text_color.a * mask.b) * dest.a_b;
```

There's a third change we need to make:

 - We're rendering to a destination surface that only has one alpha channel instead of three.
   So `dest.a_r/g/b` and `result.a_r/g/b` will need to become `dest.a` and `result.a`.

This creates a problem: We're currently assigning different values to `result.a_r`, `result.a_g` and `result.a_b`.
Which of them should we use to compute `result.a`?

This question does not have an answer. One alpha value per pixel is simply not sufficient
to express the same information as three alpha values.

However, see what happens if the destination is already opaque:

We have `dest.a_r == 1`, `dest.a_g == 1`, and `dest.a_b == 1`.

```
result.a_r = text_color.a * mask.r + (1 - text_color.a * mask.r) * dest.a_r
           = text_color.a * mask.r + (1 - text_color.a * mask.r) * 1
           = text_color.a * mask.r + 1 - text_color.a * mask.r
           = 1
same for result.a_g and result.a_b
```

In other words, for opaque destinations, it doesn't matter what which channel of the mask we use when computing `result.a`, the result will always be completely opaque anyways. In WebRender we just pick `mask.g` (or rather,
have font rasterization set `mask.a` to the value of `mask.g`) because it's as good as any.

The takeaway here is: **Subpixel text blending is only supported for opaque destinations.** Attempting to render subpixel
text into partially transparent destinations will result in bad alpha values. Or rather, it will result in alpha values which
are not anticipated by the r, g, and b values in the same pixel, so that subsequent blend operations, which will mix r and a values
from the same pixel, will produce incorrect colors.

Here's the final subpixel blend function:

```glsl
vec4 subpixeltextblend(vec4 text_color, vec4 mask, vec4 dest) {
  vec4 result;
  result.r = text_color.r * mask.r + (1.0 - text_color.a * mask.r) * dest.r;
  result.g = text_color.g * mask.g + (1.0 - text_color.a * mask.g) * dest.g;
  result.b = text_color.b * mask.b + (1.0 - text_color.a * mask.b) * dest.b;
  result.a = text_color.a * mask.a + (1.0 - text_color.a * mask.a) * dest.a;
  return result;
}
```

or for short:

```glsl
vec4 subpixeltextblend(vec4 text_color, vec4 mask, vec4 dest) {
  return text_color * mask + (1.0 - text_color.a * mask) * dest;
}
```

To recap, here's what we gained and lost by making the transition from the full-component-alpha world to the
regular rgba world: All colors and textures now only need four values to be represented, we still use a
component alpha mask, and the results are equivalent to the full-component-alpha result assuming that the
destination is opaque. We lost the ability to draw to partially transparent destinations.

### Making this work in OpenGL

We have the complete subpixel blend function.
Now we need to cut it into pieces and mix it with the OpenGL blend pipeline in such a way that
the fragment shader does not need to know about the destination.

Compare the equation for the red channel and the alpha channel between the two ways of text blending:

```
  single-channel alpha:
    result.r = text_color.r * mask.a + (1.0 - text_color.a * mask.a) * dest.r
    result.a = text_color.a * mask.a + (1.0 - text_color.a * mask.a) * dest.r

  component alpha:
    result.r = text_color.r * mask.r + (1.0 - text_color.a * mask.r) * dest.r
    result.a = text_color.a * mask.a + (1.0 - text_color.a * mask.a) * dest.r
```

Notably, in the single-channel alpha case, all three destination color channels are multiplied with the same thing:
`(1.0 - text_color.a * mask.a)`. This factor also happens to be "one minus `oFragColor.a`".
So we were able to take advantage of OpenGL's `ONE_MINUS_SRC_ALPHA` blend func.

In the component alpha case, we're not so lucky: Each destination color channel
is multiplied with a different factor. We can use `ONE_MINUS_SRC_COLOR` instead,
and output `text_color.a * mask` from our fragment shader.
But then there's still the problem that the first summand of the computation for `result.r` uses
`text_color.r * mask.r` and the second summand uses `text_color.a * mask.r`.

There are multiple ways to deal with this. They are:

 1. Making use of `glBlendColor` and the `GL_CONSTANT_COLOR` blend func.
 2. Using a two-pass method.
 3. Using "dual source blending".

Let's look at them in order.

#### 1. Subpixel text blending in OpenGL using `glBlendColor`

In this approach we return `text_color.a * mask` from the shader.
Then we set the blend color to `text_color / text_color.a` and use `GL_CONSTANT_COLOR` as the source blendfunc.
This results in the following blend equation:

```
result.r = (text_color.r / text_color.a) * oFragColor.r + (1 - oFragColor.r) * dest.r;
           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                ^  ^^^^^^^^^^^^^^^^^
                         |                              |      |
                         +--gl::CONSTANT_COLOR          |      +-- gl::ONE_MINUS_SRC_COLOR
                                                        |
                                                        +-- gl::FUNC_ADD

         = (text_color.r / text_color.a) * (text_color.a * mask.r) + (1 - (text_color.a * mask.r)) * dest.r
         = text_color.r * mask.r + (1 - text_color.a * mask.r) * dest.r
```

At the very beginning of this document, we defined `text_color` as the *premultiplied* text color.
So instead of actually doing the calculation `text_color.r / text_color.a` when specifying the blend color,
we really just want to use the *unpremultiplied* text color in that place.
That's usually the representation we start with anyway.

#### 2. Two-pass subpixel blending in OpenGL

The `glBlendColor` method has the disadvantage that the text color is part of the OpenGL state.
So if we want to draw text with different colors, we have two use separate batches / draw calls
to draw the differently-colored parts of text.

Alternatively, we can use a two-pass method which avoids the need to use the `GL_CONSTANT_COLOR` blend func:

 - The first pass outputs `text_color.a * mask` from the fragment shader and
   uses `gl::ZERO, gl::ONE_MINUS_SRC_COLOR` as the glBlendFuncs. This achieves:

```
oFragColor = text_color.a * mask;

result_after_pass0.r = 0 * oFragColor.r + (1 - oFragColor.r) * dest.r
                     = (1 - text_color.a * mask.r) * dest.r

result_after_pass0.g = 0 * oFragColor.g + (1 - oFragColor.g) * dest.r
                     = (1 - text_color.a * mask.r) * dest.r

...
```

 - The second pass outputs `text_color * mask` from the fragment shader and uses
   `gl::ONE, gl::ONE` as the glBlendFuncs. This results in the correct overall blend equation.

```
oFragColor = text_color * mask;

result_after_pass1.r
 = 1 * oFragColor.r + 1 * result_after_pass0.r
 = text_color.r * mask.r + result_after_pass0.r
 = text_color.r * mask.r + (1 - text_color.a * mask.r) * dest.r
```

#### 3. Dual source subpixel blending in OpenGL

The third approach is similar to the second approach, but makes use of the [`ARB_blend_func_extended`](https://www.khronos.org/registry/OpenGL/extensions/ARB/ARB_blend_func_extended.txt) extension
in order to fold the two passes into one:
Instead of outputting the two different colors in two separate passes, we output them from the same pass,
as two separate fragment shader outputs.
Those outputs can then be treated as two different sources in the blend equation.

## Subpixel Text Rendering to Transparent Destinations with a Background Color Hint

### Motivation

As we've seen in the previous section, subpixel text drawing has the limitation that it only works on opaque destinations.

In other words, if you use the `subpixeltextblend` function to draw something to a transparent surface,
and then composite that surface onto on opaque background,
the result will generally be different from drawing the text directly onto the opaque background.

Let's express that inequality in code.

```
 - vec4 text_color
 - vec4 mask
 - vec4 transparency = vec4(0.0, 0.0, 0.0, 0.0)
 - vec4 background with background.a == 1.0

over(subpixeltextblend(text_color, mask, transparency), background).rgb
 is, in general, not equal to
subpixeltextblend(text_color, mask, background).rgb
```

However, one interesting observation is that if the background is black, the two *are* equal:

```
vec4 black = vec4(0.0, 0.0, 0.0, 1.0);

over(subpixeltextblend(text_color, mask, transparency), black).r
 = subpixeltextblend(text_color, mask, transparency).r +
     (1 - subpixeltextblend(text_color, mask, transparency).a) * black.r
 = subpixeltextblend(text_color, mask, transparency).r +
     (1 - subpixeltextblend(text_color, mask, transparency).a) * 0
 = subpixeltextblend(text_color, mask, transparency).r
 = text_color.r * mask.r + (1 - text_color.a * mask.r) * transparency.r
 = text_color.r * mask.r + (1 - text_color.a * mask.r) * 0
 = text_color.r * mask.r + (1 - text_color.a * mask.r) * black.r
 = subpixeltextblend(text_color, mask, black).r
```

So it works out for black backgrounds. The further your *actual* background color gets away from black,
the more incorrect your result will be.

If it works for black, is there a way to make it work for other colors?
This is the motivating question for this third way of text blending:

We want to be able to specify an *estimated background color*, and have a blending function
`vec4 subpixeltextblend_withbgcolor(vec4 text_color, vec4 mask, vec4 bg_color, vec4 dest)`,
in such a way that the error we get by using an intermediate surface is somehow in relation
to the error we made when estimating the background color. In particular, if we estimated
the background color perfectly, we want the intermediate surface to go unnoticed.

Expressed as code:

```
over(subpixeltextblend_withbgcolor(text_color, mask, bg_color, transparency), bg_color)
 should always be equal to
subpixeltextblend(text_color, mask, bg_color)
```

This is one of three constraints we'd like `subpixeltextblend_withbgcolor` to satisfy.

The next constraint is the following: If `dest` is already opaque, `subpixeltextblend_withbgcolor`
should have the same results as `subpixeltextblend`, and the background color hint should be ignored.

```
 If dest.a == 1.0,
subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest)
 should always be equal to
subpixeltextblend(text_color, mask, dest)
```

And there's a third condition we'd like it to fulfill:
In places where the mask is zero, the destination should be unaffected.

```
subpixeltextblend_withbgcolor(text_color, transparency, bg_color, dest)
 should always be equal to
dest
```

### Use cases

The primary use case for such a blend method is text on top of vibrant areas of a window on macOS.

Vibrant backgrounds with behind-window blending are computed by the window server, and they are tinted
in a color that's based on the chosen vibrancy type.

The window's rgba buffer is transparent in the vibrant areas. Window contents, even text, are drawn onto
that transparent rgba buffer. Then the window server composites the window onto an opaque backdrop.
So the results on the screen are computed as follows:

```glsl
window_buffer_pixel = subpixeltextblend_withbgcolor(text_color, mask, bg_color, transparency);
screen_pixel = over(window_buffer_pixel, window_backdrop);
```

### Prior art

Apple has implemented such a method of text blending in CoreGraphics, specifically for rendering text onto vibrant backgrounds.
It's hidden behind the private API `CGContextSetFontSmoothingBackgroundColor` and is called by AppKit internally before
calling the `-[NSView drawRect:]` method of your `NSVisualEffectView`, with the appropriate font smoothing background color
for the vibrancy type of that view.

I'm not aware of any public documentation of this way of text blending.
It seems to be considered an implementation detail by Apple, and is probably hidden by default because it can be a footgun:
If the font smoothing background color you specify is very different from the actual background that our surface is placed
on top of, the text will look glitchy.

### Deriving the blending function from first principles

Before we dive into the math, let's repeat our goal once more.

We want to create a blending function of the form
`vec4 subpixeltextblend_withbgcolor(vec4 text_color, vec4 mask, vec4 bg_color, vec4 dest)`
(with `bg_color` being an opaque color)
which satisfies the following three constraints:

```
Constraint I:
  over(subpixeltextblend_withbgcolor(text_color, mask, bg_color, transparency), bg_color)
   should always be equal to
  subpixeltextblend(text_color, mask, bg_color)

Constraint II:
   If dest.a == 1.0,
  subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest)
   should always be equal to
  subpixeltextblend(text_color, mask, dest)

Constraint II:
  subpixeltextblend_withbgcolor(text_color, transparency, bg_color, dest)
   should always be equal to
  dest
```

Constraint I and constraint II are about what happens depending on the destination's alpha.
In particular: If the destination is completely transparent, we should blend into the
estimated background color, and if it's completely opaque, we should blend into the destination color.
In fact, we really want to blend into `over(dest, bg_color)`: we want `bg_color` to be used
as a backdrop *behind* the current destination. So let's combine constraints I and II into a new
constraint IV:

```
Constraint IV:
  over(subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest), bg_color)
   should always be equal to
  subpixeltextblend(text_color, mask, over(dest, bg_color))
```

Let's look at just the left side of that equation and rejiggle it a bit:

```
over(subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest), bg_color).r
 = subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).r +
   (1 - subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).a) * bg_color.r

<=>

over(subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest), bg_color).r -
(1 - subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).a) * bg_color.r
 = subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).r
```

Now insert the right side of constraint IV:

```
subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).r
 = over(subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest), bg_color).r -
   (1 - subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).a) * bg_color.r
 = subpixeltextblend(text_color, mask, over(dest, bg_color)).r -
   (1 - subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).a) * bg_color.r
```

Our blend function is almost finished. We just need select an alpha for our result.
Constraints I, II and IV don't really care about the alpha value. But constraint III requires that:

```
  subpixeltextblend_withbgcolor(text_color, transparency, bg_color, dest).a
   should always be equal to
  dest.a
```

so the computation of the alpha value somehow needs to take into account the mask.

Let's say we have an unknown function `make_alpha(text_color.a, mask)` which returns
a number between 0 and 1 and which is 0 if the mask is entirely zero, and let's defer
the actual implementation of that function until later.

Now we can define the alpha of our overall function using the `over` function:

```
subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).a
 := make_alpha(text_color.a, mask) + (1 - make_alpha(text_color.a, mask)) * dest.a
```

We can plug this in to our previous result:

```
subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).r
 = subpixeltextblend(text_color, mask, over(dest, bg_color)).r
   - (1 - subpixeltextblend_withbgcolor(text_color, mask, bg_color, dest).a) * bg_color.r
 = subpixeltextblend(text_color, mask, over(dest, bg_color)).r
   - (1 - (make_alpha(text_color.a, mask) +
           (1 - make_alpha(text_color.a, mask)) * dest.a)) * bg_color.r
 = text_color.r * mask.r + (1 - text_color.a * mask.r) * over(dest, bg_color).r
   - (1 - (make_alpha(text_color.a, mask)
           + (1 - make_alpha(text_color.a, mask)) * dest.a)) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * (dest.r + (1 - dest.a) * bg_color.r)
   - (1 - (make_alpha(text_color.a, mask)
           + (1 - make_alpha(text_color.a, mask)) * dest.a)) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * (dest.r + (1 - dest.a) * bg_color.r)
   - (1 - (make_alpha(text_color.a, mask)
           + (1 - make_alpha(text_color.a, mask)) * dest.a)) * bg_color.r
 = text_color.r * mask.r
   + (dest.r + (1 - dest.a) * bg_color.r)
   - (text_color.a * mask.r) * (dest.r + (1 - dest.a) * bg_color.r)
   - (1 - make_alpha(text_color.a, mask)
      - (1 - make_alpha(text_color.a, mask)) * dest.a) * bg_color.r
 = text_color.r * mask.r
   + dest.r + (1 - dest.a) * bg_color.r
   - text_color.a * mask.r * dest.r
   - text_color.a * mask.r * (1 - dest.a) * bg_color.r
   - (1 - make_alpha(text_color.a, mask)
      - (1 - make_alpha(text_color.a, mask)) * dest.a) * bg_color.r
 = text_color.r * mask.r
   + dest.r + (1 - dest.a) * bg_color.r
   - text_color.a * mask.r * dest.r
   - text_color.a * mask.r * (1 - dest.a) * bg_color.r
   - ((1 - make_alpha(text_color.a, mask)) * 1
      - (1 - make_alpha(text_color.a, mask)) * dest.a) * bg_color.r
 = text_color.r * mask.r
   + dest.r + (1 - dest.a) * bg_color.r
   - text_color.a * mask.r * dest.r
   - text_color.a * mask.r * (1 - dest.a) * bg_color.r
   - ((1 - make_alpha(text_color.a, mask)) * (1 - dest.a)) * bg_color.r
 = text_color.r * mask.r
   + dest.r - text_color.a * mask.r * dest.r
   + (1 - dest.a) * bg_color.r
   - text_color.a * mask.r * (1 - dest.a) * bg_color.r
   - (1 - make_alpha(text_color.a, mask)) * (1 - dest.a) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * dest.r
   + (1 - dest.a) * bg_color.r
   - text_color.a * mask.r * (1 - dest.a) * bg_color.r
   - (1 - make_alpha(text_color.a, mask)) * (1 - dest.a) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * dest.r
   + (1 - text_color.a * mask.r) * (1 - dest.a) * bg_color.r
   - (1 - make_alpha(text_color.a, mask)) * (1 - dest.a) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * dest.r
   + ((1 - text_color.a * mask.r)
      - (1 - make_alpha(text_color.a, mask))) * (1 - dest.a) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * dest.r
   + (1 - text_color.a * mask.r
      - 1 + make_alpha(text_color.a, mask)) * (1 - dest.a) * bg_color.r
 = text_color.r * mask.r
   + (1 - text_color.a * mask.r) * dest.r
   + (make_alpha(text_color.a, mask) - text_color.a * mask.r) * (1 - dest.a) * bg_color.r
```

We now have a term of the form `A + B + C`, with `A` and `B` being guaranteed to
be between zero and one.

We also want `C` to be between zero and one.
We can use this restriction to help us decide on an implementation of `make_alpha`.

If we define `make_alpha` as

```glsl
float make_alpha(text_color_a, mask) {
  float max_rgb = max(max(mask.r, mask.g), mask.b);
  return text_color_a * max_rgb;
}
```

, then `(make_alpha(text_color.a, mask) - text_color.a * mask.r)` becomes
`(text_color.a * max(max(mask.r, mask.g), mask.b) - text_color.a * mask.r)`, which is
`text_color.a * (max(max(mask.r, mask.g), mask.b) - mask.r)`, and the subtraction will
always yield something that's greater or equal to zero for r, g, and b,
because we will subtract each channel from the maximum of the channels.

Putting this all together, we have:

```glsl
vec4 subpixeltextblend_withbgcolor(vec4 text_color, vec4 mask, vec4 bg_color, vec4 dest) {
  float max_rgb = max(max(mask.r, mask.g), mask.b);
  vec4 result;
  result.r = text_color.r * mask.r + (1 - text_color.a * mask.r) * dest.r +
             text_color.a * bg_color.r * (max_rgb - mask.r) * (1 - dest.a);
  result.g = text_color.g * mask.g + (1 - text_color.a * mask.g) * dest.g +
             text_color.a * bg_color.g * (max_rgb - mask.g) * (1 - dest.a);
  result.b = text_color.b * mask.b + (1 - text_color.a * mask.b) * dest.b +
             text_color.a * bg_color.b * (max_rgb - mask.b) * (1 - dest.a);
  result.a = text_color.a * max_rgb + (1 - text_color.a * max_rgb) * dest.a;
  return result;
}
```

This is the final form of this blend function. It satisfies all of the four constraints.

### Implementing it with OpenGL

Our color channel equations consist of three pieces:

 - `text_color.r * mask.r`, which simply gets added to the rest.
 - `(1 - text_color.a * mask.r) * dest.r`, a factor which gets multiplied with the destination color.
 - `text_color.a * bg_color.r * (max_rgb - mask.r) * (1 - dest.a)`, a factor which gets multiplied
   with "one minus destination alpha".

We will need three passes. Each pass modifies the color channels in the destination.
This means that the part that uses `dest.r` needs to be applied first.
Then we can apply the part that uses `1 - dest.a`.
(This means that the first pass needs to leave `dest.a` untouched.)
And the final pass can apply the `result.a` equation and modify `dest.a`.

```
pub fn set_blend_mode_subpixel_with_bg_color_pass0(&self) {
    self.gl.blend_func_separate(gl::ZERO, gl::ONE_MINUS_SRC_COLOR, gl::ZERO, gl::ONE);
}
pub fn set_blend_mode_subpixel_with_bg_color_pass1(&self) {
    self.gl.blend_func_separate(gl::ONE_MINUS_DST_ALPHA, gl::ONE, gl::ZERO, gl::ONE);
}
pub fn set_blend_mode_subpixel_with_bg_color_pass2(&self) {
    self.gl.blend_func_separate(gl::ONE, gl::ONE, gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
}

Pass0:
    oFragColor = vec4(text.color.a) * mask;
Pass1:
    oFragColor = vec4(text.color.a) * text.bg_color * (vec4(mask.a) - mask);
Pass2:
    oFragColor = text.color * mask;

result_after_pass0.r = 0 * (text_color.a * mask.r) + (1 - text_color.a * mask.r) * dest.r
result_after_pass0.a = 0 * (text_color.a * mask.a) + 1 * dest.a

result_after_pass1.r = (1 - result_after_pass0.a) * (text_color.a * (mask.max_rgb - mask.r) * bg_color.r) + 1 * result_after_pass0.r
result_after_pass1.a = 0 * (text_color.a * (mask.max_rgb - mask.a) * bg_color.a) + 1 * result_after_pass0.a

result_after_pass2.r = 1 * (text_color.r * mask.r) + 1 * result_after_pass1.r
result_after_pass2.a = 1 * (text_color.a * mask.max_rgb) + (1 - text_color.a * mask.max_rgb) * result_after_pass1.a
```

Instead of computing `max_rgb` in the shader, we can just require the font rasterization code to fill
`mask.a` with the `max_rgb` value.

