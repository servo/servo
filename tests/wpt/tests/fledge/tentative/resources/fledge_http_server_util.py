# Takes a map of header names to list of values that are all binary strings
# and returns an otherwise identical map where keys and values have both been
# converted to ASCII strings.
def headersToAscii(headers):
  header_map = {}
  for pair in headers.items():
      values = []
      for value in pair[1]:
          values.append(value.decode("ASCII"))
      header_map[pair[0].decode("ASCII")] = values
  return header_map
