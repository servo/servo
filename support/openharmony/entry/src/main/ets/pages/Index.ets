import { common } from '@kit.AbilityKit';
import display from '@ohos.display';
import deviceInfo from '@ohos.deviceInfo';
import promptAction from '@ohos.promptAction';

interface ServoXComponentInterface {
  loadURL(url: string): void;

  goBack(): void;

  goForward(): void;

  registerURLcallback(callback: (url: string) => void): void;
  registerTerminateCallback(callback: () => void): void;

  registerPromptToastCallback(callback: (msg: string) => void): void

  focusWebview(index: number):void;

  initServo(options: InitOpts): void;
}

interface InitOpts {
  url: string;
  resourceDir: string,
  commandlineArgs: string,
}

function prompt_toast(msg: string) {
  promptAction.showToast({
    message: msg,
    duration: 2000
  });
}

// Use the getShared API to obtain the LocalStorage instance shared by stage.
let storage = LocalStorage.getShared()

@Entry(storage)
@Component
struct Index {
  xComponentContext: ServoXComponentInterface | undefined = undefined;
  xComponentAttrs: XComponentAttrs = {
    id: 'ServoDemo',
    type: XComponentType.SURFACE,
    libraryname: 'servoshell',
  }
  private context = getContext(this) as common.UIAbilityContext;
  @LocalStorageProp('InitialURI') InitialURI: string = "unused"
  @LocalStorageProp('CommandlineArgs') CommandlineArgs: string = ""
  @State urlToLoad: string = this.InitialURI
  @State tablist: Array<number> = [];
  @State currentIndex: number = 0;

  // Called when the user swipes from the right or left edge to the middle
  // Default behavior is bringing the app to the background.
  onBackPress(): boolean | void {
    this.xComponentContext?.goBack()
    return true;
  }

  build() {
    // We originally use `Column()` here, but for some reason the column
    // extends beyond the edge of the screen. This does not happen with
    // Flex.
    Flex({ direction: FlexDirection.Column}) {
      Row() {
        Button('+')
          .backgroundColor(Color.White)
          .fontColor(Color.Black)
          .fontWeight(FontWeight.Bolder)
          .fontSize(22)
          .width('12%')
          .onClick((event) => {
            if (this.tablist.length==0) {
              this.tablist.push(2);
            } else {
              this.tablist.push(this.tablist[this.tablist.length-1]+1);
            }
            // yes this is correct as we always have one tab extra
	    // The tab extra is seperate for the initialization and will always exist.
	    // It is not in the tablist.
            this.currentIndex = this.tablist.length;
          })
        Button('⇦')
          .backgroundColor(Color.White)
          .fontColor(Color.Black)
          .fontWeight(FontWeight.Bolder)
          .width('12%')
          .fontSize(12)
          .onClick(() => {
            this.onBackPress()
          })
        Button('⇨')
          .backgroundColor(Color.White)
          .fontColor(Color.Black)
          .fontWeight(FontWeight.Bolder)
          .fontSize(12)
          .width('12%')
          .onClick(() => {
            this.xComponentContext?.goForward()
          })
        TextInput({ placeholder: 'URL', text: $$this.urlToLoad })
          .type(InputType.Normal)
          .width('76%')
          .onChange((value) => {
            this.urlToLoad = value
          })
          .onSubmit((EnterKeyType) => {
            this.xComponentContext?.loadURL(this.urlToLoad)
            console.info('Load URL: ', this.urlToLoad)
          })
      }

      Tabs({ barPosition: BarPosition.Start, index: this.currentIndex}) {
        TabContent() {
          XComponent(this.xComponentAttrs)
            .focusable(true)
            .onLoad((xComponentContext) => {
              this.xComponentContext = xComponentContext as ServoXComponentInterface;
              let resource_dir: string = this.context.resourceDir;
              let cache_dir: string = this.context.cacheDir;
              console.debug("resourceDir: ", resource_dir);
              console.debug("cacheDir: ", cache_dir);
              let init_options: InitOpts = {
                url: this.urlToLoad,
                resourceDir: resource_dir,
                commandlineArgs: this.CommandlineArgs
              }
              this.xComponentContext.initServo(init_options)
              this.xComponentContext.registerURLcallback((new_url) => {
                console.info('New URL from native: ', new_url)
                this.urlToLoad = new_url
              })
              this.xComponentContext.registerPromptToastCallback(prompt_toast)
            })
        }.tabBar('1')
        ForEach(this.tablist, (item: number) => {
          TabContent() {
            XComponent(this.xComponentAttrs)
              .focusable(true)
          }.tabBar(String(item))
        })
      }.onChange((index: number) => {
        this.xComponentContext?.focusWebview(index);
      })
    }
    .width('100%')
  }
}

interface XComponentAttrs {
  id: string;
  type: number;
  libraryname: string;
}
