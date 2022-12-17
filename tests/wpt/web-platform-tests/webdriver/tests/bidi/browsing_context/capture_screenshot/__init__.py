from base64 import encodebytes

from webdriver.bidi.modules.script import ContextTarget

from tests.support.image import png_dimensions

async def viewport_dimensions(bidi_session, context):
    """Get the dimensions of the viewport containing context.

    :param bidi_session: BiDiSession
    :param context: Browsing context ID
    :returns: Tuple of (int, int) containing viewport width, viewport height.
    """
    result = await bidi_session.script.call_function(
        function_declaration="""() => {
        const {devicePixelRatio, innerHeight, innerWidth} = window;

        return [
          Math.floor(innerWidth * devicePixelRatio),
          Math.floor(innerHeight * devicePixelRatio)
        ];
    }""",
        target=ContextTarget(context["context"]),
        await_promise=False)
    return tuple(item["value"] for item in result["value"])


class ImageDifference:
    """Summary of the pixel-level differences between two images.

    :param total_pixels: The total number of pixel differences between the images
    :param max_difference: The maximum difference between any corresponding color channels across
                           all pixels of the image.
    """

    def __init__(self, total_pixels, max_difference):
        self.total_pixels = total_pixels
        self.max_difference = max_difference

    def equal(self):
        return self.total_pixels == 0


async def compare_png_data(bidi_session,
                           url,
                           img1: bytes,
                           img2: bytes) -> ImageDifference:
    """Calculate difference statistics between two PNG images.

    :param bidi_session: BidiSession
    :param url: fixture to construct a URL string given a path
    :param img1: Bytes of first PNG image
    :param img2: Bytes of second PNG image
    :returns: ImageDifference representing the total number of different pixels,
              and maximum per-channel difference between the images.
    """
    if img1 == img2:
        return ImageDifference(0, 0)

    width, height = png_dimensions(img1)
    assert (width, height) == png_dimensions(img2)

    context = await bidi_session.browsing_context.create(type_hint="tab")
    await bidi_session.browsing_context.navigate(
        context=context["context"],
        url=url("/webdriver/tests/support/html/render.html"),
        wait="complete")
    result = await bidi_session.script.call_function(
        function_declaration="""(img1, img2, width, height) => {
          return compare(img1, img2, width, height)
        }""",
        target=ContextTarget(context["context"]),
        arguments=[{"type": "string",
                    "value": encodebytes(img1).decode()},
                   {"type": "string",
                    "value": encodebytes(img2).decode()},
                   {"type": "number",
                    "value": width},
                   {"type": "number",
                    "value": height}],
        await_promise=True)
    await bidi_session.browsing_context.close(context=context["context"])
    assert result["type"] == "object"
    assert set(item[0] for item in result["value"]) == {"totalPixels", "maxDifference"}
    for item in result["value"]:
        assert len(item) == 2
        assert item[1]["type"] == "number"
        if item[0] == "totalPixels":
            total_pixels = item[1]["value"]
        elif item[0] == "maxDifference":
            max_difference = item[1]["value"]
        else:
            raise Exception(f"Unexpected object key ${item[0]}")
    return ImageDifference(total_pixels, max_difference)
