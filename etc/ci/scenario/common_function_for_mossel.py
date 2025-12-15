from selenium.common import NoSuchElementException
from selenium.webdriver.common.by import By
from selenium import webdriver


# Click to close the pop-up
# Note that the pop-up may not exist, either because we did this in the past
# which sets localstorage, or the website does not have seasonal promotions/recommendations.
def close_popup(driver: webdriver.Remote):
    popup_css_selector = (
        "#app > uni-app > uni-page > uni-page-wrapper > uni-page-body > uni-view "
        "> uni-view:nth-child(5) "
        "> uni-view.m-popup.m-popup_transition.m-mask_show.m-mask_fade.m-popup_push.m-fixed_mid "
        "> uni-view > uni-view > uni-button:nth-child(1)"
    )
    print("Waiting for popup to appear ...")
    try:
        birthday_element = driver.find_element(By.CSS_SELECTOR, popup_css_selector)
        birthday_element.click()
        print("Closed the popup")
    except NoSuchElementException as e:
        print(f"Failed to find pop_up element with selector `{popup_css_selector}`. Skip it.")
