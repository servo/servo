def client_hints_list():
  return [b"device-memory",
          b"dpr",
        # b"width", (Only available for images)
          b"viewport-width",
          b"rtt",
          b"downlink",
          b"ect",
          b"sec-ch-ua",
          b"sec-ch-ua-arch",
          b"sec-ch-ua-platform",
          b"sec-ch-ua-model",
          b"sec-ch-ua-mobile",
          b"sec-ch-ua-full-version",
          b"sec-ch-ua-platform-version",
          b"sec-ch-prefers-color-scheme",
          b"sec-ch-prefers-reduced-motion",
          b"sec-ch-ua-bitness",
          b"sec-ch-viewport-height",
          b"sec-ch-device-memory",
          b"sec-ch-dpr",
        # b"sec-ch-width", (Only available for images)
          b"sec-ch-viewport-width",
          b"sec-ch-ua-full-version-list",
          b"sec-ch-ua-wow64",
          b"sec-ch-prefers-reduced-transparency",
  ]

def client_hints_full_list():
  return client_hints_list() + [b"width", b"sec-ch-width"]

def client_hints_ua_list():
  return [b"sec-ch-ua",
          b"sec-ch-ua-arch",
          b"sec-ch-ua-platform",
          b"sec-ch-ua-platform-version",
          b"sec-ch-ua-model",
          b"sec-ch-ua-full-version",
          b"sec-ch-ua-full-version-list",
          b"sec-ch-ua-wow64",
    ]