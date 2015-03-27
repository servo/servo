"""Definition of various capability-related constants."""

class Capability:
    """Standard capability names."""
    BROWSER_NAME = 'browserName'
    BROWSER_VERSION = 'browserVersion'
    PAGE_LOADING_STRATEGY = 'pageLoadingStrategy'
    PLATFORM_NAME = 'platformName'
    PLATFORM_VERSION = 'platformVersion'
    SECURE_SSL = 'secureSsl'
    TAKES_SCREENSHOT = 'takesScreenshot'
    TAKE_ELEMENT_SCREENSHOT = 'takeElementScreenshot'
    TOUCH_ENABLED = 'touchEnabled'

class Platform:
    """Standard OS names."""
    ANY = 'any'
    ANDROID = 'android'
    IOS = 'ios'
    LINUX = 'linux'
    MAC = 'mac'
    UNIX = 'unix'
    WINDOW = 'windows'

class PageLoadingStrategy:
    """Standard page loading strategies."""
    CONSERVATIVE = 'conservative'
    NORMAL = 'normal'
    EAGER = 'eager'
    NONE = 'none'
