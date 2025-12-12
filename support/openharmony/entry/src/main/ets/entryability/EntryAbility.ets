import AbilityConstant from '@ohos.app.ability.AbilityConstant';
import hilog from '@ohos.hilog';
import UIAbility from '@ohos.app.ability.UIAbility';
import Want from '@ohos.app.ability.Want';
import window from '@ohos.window';

const WRONG_COMMAND_ARRAY = ["tracing", "devtools", "force_ipc", "multiprocess", "webdriver"];

export default class EntryAbility extends UIAbility {
  init_params: LocalStorage = new LocalStorage();
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam) {
    let uri: string = want.uri || "https://servo.org"
    let params: string[] = []
    if (typeof want.parameters !== "undefined") {
      Object.entries(want.parameters).forEach((entry) => {
        let key = entry[0]
        // The commandline parameters we are interested in all start with `--`.
        // OpenHarmony also adds quite a few default parameters to the wants.parameters
        // dictionary, and servo currently fails with an error if it receives unknown
        // options.
        // On the commandline you can pass parameters via the `aa` tool.
        // ```
        // aa start -a EntryAbility -b org.servo.servoshell --ps=--screen-size 505x413
        // ```
        // Use `--ps=--arg_name value` for key-value parameters. Please note that
        // `--ps` and `--<arg_name>` must be connected by an `=` and may not be seperated.
        // the value must be space seperated, since `aa start` will expect it as a second parameter.
        // Use `--psn=--arg_name` for flags that do not have a value.
        // See the aa tool documentation for more details:
        // https://docs.openharmony.cn/pages/v5.0/en/application-dev/tools/aa-tool.md
        if (key.startsWith("--")) {
            let value = entry[1].toString()
            params.push(key)
            if (value) {
                params.push(value)
            }
        } else if (WRONG_COMMAND_ARRAY.some(value => key.startsWith(value))) {
          hilog.error(0xE0C3, "Servo EntryAbility", "-------------------------------------------------------------------------------------------------");
          hilog.error(0xE0C3, "Servo EntryAbility", "\n\nYou probably meant to add -- infront of your argument, i.e., --psn=--tracing vs --psn=tracing. You used " + entry);
          hilog.error(0xE0C3, "Servo EntryAbility", "-------------------------------------------------------------------------------------------------");
          return;
        }
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
