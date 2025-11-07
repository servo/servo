import sys
import time
import subprocess
import common_function_for_servo_test
from selenium.webdriver.common.by import By


def operator():
    # Step 1. Open mossel
    cmd_start_servo = [
        f'hdc shell aa force-stop org.servo.servo',
        f'hdc shell aa start -a EntryAbility -b org.servo.servo -U https://m.huaweimossel.com --psn --webdriver',
    ]

    for cmd in cmd_start_servo:
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
    time.sleep(10)

    driver = common_function_for_servo_test.setup_hdc_forward()
    if driver is not False:
        # Step 2. Click to close the pop-up
        birthday_ = driver.find_element(By.CSS_SELECTOR,
                                        "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view:nth-child(5) > uni-view.m-popup.m-popup_transition.m-mask_show.m-mask_fade.m-popup_push.m-fixed_mid > uni-view > uni-view > uni-button:nth-child(1)")
        birthday_.click()
        time.sleep(1)

        # Step 3. Click 'Categories'
        cmd = 'hdc shell "uinput -T -c 380 2556"'
        subprocess.run(cmd, capture_output=True, text=True, timeout=10)
        time.sleep(5)

        # Step 4. Click on the first product
        first_good = driver.find_element(By.CSS_SELECTOR, "#goodsGroup > uni-view:nth-child(1) > img")
        first_good.click()
        time.sleep(5)

        # Step 5. Click for details
        content = driver.find_element(By.CSS_SELECTOR, "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view > uni-view.m-tab.product.m-flex__level > uni-text:nth-child(2) > span")
        content.click()
        time.sleep(5)

        # Step 6. Catch trace
        process = subprocess.Popen(
            "hdc shell hitrace -b 81920 -t 10 nweb ace app ohos zimage zmedia zcamera zaudio ability distributeddatamgr graphic sched freq irq sync mmc rpc workq idle disk pagecache memreclaim binder -o /data/local/tmp/my_trace.html",
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        time.sleep(4)

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

        frame_rate = common_function_for_servo_test.calculate_frame_rate()
        print(f'framerate is {frame_rate}')

        if frame_rate >= 115:
            cmd = 'hdc shell aa force-stop org.servo.servo'
            subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            return True
        else:
            cmd = 'hdc shell aa force-stop org.servo.servo'
            subprocess.run(cmd, capture_output=True, text=True, timeout=10)
            return False
    else:
        return False


if __name__ == '__main__':
    result = operator()
    if not result:
        sys.exit(1)

