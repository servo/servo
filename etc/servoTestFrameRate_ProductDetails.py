import sys
import os
import importlib.util

module_path = os.path.join(os.path.dirname(__file__), 'aw', 'commonFunction.py')
spec = importlib.util.spec_from_file_location("aw", module_path)
commonFunction = importlib.util.module_from_spec(spec)
sys.modules["aw"] = commonFunction
spec.loader.exec_module(commonFunction)

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
    set hdc forward
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

    # Click on the first product
    first_good = driver.find_element(By.CSS_SELECTOR, "#goodsGroup > uni-view:nth-child(1) > img")
    first_good.click()
    time.sleep(5)

    # Click for details
    content = driver.find_element(By.CSS_SELECTOR, "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view.m-tab.product.m-flex__level > uni-text:nth-child(2) > span")
    content.click()
    time.sleep(5)

    # Catch trace
    process = subprocess.Popen(
        "hdc shell hitrace -b 81920 -t 10 nweb ace app ohos zimage zmedia zcamera zaudio ability distributeddatamgr graphic sched freq irq sync mmc rpc workq idle disk pagecache memreclaim binder -o /data/local/tmp/my_trace.html",
        stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    time.sleep(4)
    print('start slide...')

    cmd = 'hdc shell "uinput -T -m 770 2000 770 770 30"'
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(20)

    if process.poll() is not None:
        stdout, stderr = process.communicate()
        print("The child process has completed, and the output is as follows")
        print(stdout.decode())
        if stderr:
            print("Error message:")
            print(stderr.decode())
    else:
        print("The child process is still running ..")
    print('end slide...')

    cmd = 'hdc file recv /data/local/tmp/my_trace.html {}'.format(
        os.path.join(os.path.dirname(__file__), 'traces', 'my_trace.html'))
    subprocess.run(cmd, capture_output=True, text=True, timeout=10)

    # 4. Calculate frame rate
    frame_rate = commonFunction.calcuteFrameRate(
        os.path.join(os.path.dirname(__file__), 'traces', 'my_trace.html'))
    print(f'framerate is {frame_rate}')

    if frame_rate >= 115:
        cmd = 'hdc shell aa force-stop org.servo.servo'
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        return True
    else:
        cmd = 'hdc shell aa force-stop org.servo.servo'
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        return False


if __name__ == '__main__':
    for v in ("HTTP_PROXY", "http_proxy", "HTTPS_PROXY", "https_proxy"):
        os.environ.pop(v, None)
    result = operator()
    print(result)
    if not result:
        sys.exit(1)

