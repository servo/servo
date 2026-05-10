from base64 import decodebytes


def assert_move_to_coordinates(point, target, events):
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == point["x"]
            assert e["pageY"] == point["y"]
            assert e["target"] == target


def assert_pdf(value):
    data = decodebytes(value.encode())

    assert data.startswith(b"%PDF-"), "Decoded data starts with the PDF signature"
    assert data.endswith(b"%%EOF\n"), "Decoded data ends with the EOF flag"


def assert_png(screenshot):
    """Test that screenshot is a Base64 encoded PNG file, or a bytestring representing a PNG.

    Returns the bytestring for the PNG, if the assert passes
    """
    if type(screenshot) is str:
        image = decodebytes(screenshot.encode())
    else:
        image = screenshot
    assert image.startswith(b'\211PNG\r\n\032\n'), "Expected image to be PNG"
    return image
