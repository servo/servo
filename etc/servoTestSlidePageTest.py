import base64
import os
import sys
import time
import subprocess
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.common.options import ArgOptions
from selenium.webdriver.common.action_chains import ActionChains
from selenium.common.exceptions import NoSuchElementException


WEBDRIVER_PORT = 7000
SERVO_URL = f"http://127.0.0.1:{WEBDRIVER_PORT}"
def setup_hdc_forward():
    """
    setup_hdc_forward
    :return:
    """
    try:
        cmd = ["hdc", "fport", f"tcp:{WEBDRIVER_PORT}", f"tcp:7000"]
        print(f"Setting up HDC port forwarding: {' '.join(cmd)}")
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        print(f"HDC port forwarding established on port {WEBDRIVER_PORT}")
        return True

    except FileNotFoundError:
        print("HDC command not found. Make sure OHOS SDK is installed and hdc is in PATH.")
        return False
    except subprocess.TimeoutExpired:
        print(f"HDC port forwarding timed out on port {WEBDRIVER_PORT}")
        return False
    except Exception as e:
        print(f"failed to setup HDC forwarding: {e}")
        return False

def operator():
    # 1. Open mossel
    cmd_start_servo = [
        f'hdc shell aa force-stop org.servo.servo',
        f'hdc shell aa start -a EntryAbility -b org.servo.servo -U https://m.huaweimossel.com --psn --webdriver',
    ]

    for cmd in cmd_start_servo:
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(10)

    # Click to close the pop-up
    cmd = f'hdc shell "uinput -T -c 550 2340"'
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)

    setup_hdc_forward()
    options = ArgOptions()
    options.set_capability("browserName", "servo")

    driver = webdriver.Remote(command_executor=SERVO_URL, options=options)
    driver.implicitly_wait(15)

    # 2. Click 'Categories'
    cmd = 'hdc shell "uinput -T -c 380 2556"'
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(5)

    before = driver.get_screenshot_as_base64()

    cmd = 'hdc shell "uinput -T -m 770 2000 770 930"'
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(5)

    after = driver.get_screenshot_as_base64()

    cmd = 'hdc shell aa force-stop org.servo.servo'
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)

    if before == after:
        return False
    else:
        return True

if __name__ == '__main__':
    for v in ("HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"):
        os.environ.pop(v, None)
    result = operator()
    print(result)
    if not result:
        sys.exit(1)

