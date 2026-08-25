from .. import any_string


def assert_extension_id(extension_id, extension_data):
    if extension_data["id"] is None:
      any_string(extension_id)
    else:
      assert extension_id == extension_data["id"]
