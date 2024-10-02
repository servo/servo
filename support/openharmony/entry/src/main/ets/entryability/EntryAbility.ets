import AbilityConstant from '@ohos.app.ability.AbilityConstant';
import hilog from '@ohos.hilog';
import UIAbility from '@ohos.app.ability.UIAbility';
import Want from '@ohos.app.ability.Want';
import window from '@ohos.window';

export default class EntryAbility extends UIAbility {
  init_params: LocalStorage = new LocalStorage();
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam) {
    let uri: string = want.uri || "https://servo.org"
    let params: string[] = []
    if (typeof want.parameters !== "undefined") {
      Object.entries(want.parameters).forEach((entry) => {
        let key = entry[0]
        // Skip some default parameters, since servo is not interested in those
        if (key.startsWith("ohos.")
          || key.startsWith("component.")
          || key === "isCallBySCB"
          || key === "moduleName"
        ) {
          return
        }
        let value = entry[1]
        params.push("--" + key + "=" + value.toString())
      })
    }
    let servoshell_params = params.join("\u{001f}")
    hilog.debug(0x0000, 'Servo EntryAbility', 'Servoshell parameters: %{public}s', servoshell_params);
    this.init_params.setOrCreate('InitialURI', uri)
    this.init_params.setOrCreate('CommandlineArgs', servoshell_params)
  }

  onDestroy() {
    hilog.info(0x0000, 'testTag', '%{public}s', 'Ability onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage) {
    // Main window is created, set main page for this ability
    hilog.info(0x0000, 'testTag', '%{public}s', 'Ability onWindowStageCreate');

    windowStage.loadContent('pages/Index', this.init_params, (err, data) => {
      if (err.code) {
        hilog.error(0x0000, 'testTag', 'Failed to load the content. Cause: %{public}s', JSON.stringify(err) ?? '');
        return;
      }
      hilog.info(0x0000, 'testTag', 'Succeeded in loading the content. Data: %{public}s', JSON.stringify(data) ?? '');
    });
  }

  onWindowStageDestroy() {
    // Main window is destroyed, release UI related resources
    hilog.info(0x0000, 'testTag', '%{public}s', 'Ability onWindowStageDestroy');
  }

  onForeground() {
    // Ability has brought to foreground
    hilog.info(0x0000, 'testTag', '%{public}s', 'Ability onForeground');
  }

  onBackground() {
    // Ability has back to background
    hilog.info(0x0000, 'testTag', '%{public}s', 'Ability onBackground');
  }
};
