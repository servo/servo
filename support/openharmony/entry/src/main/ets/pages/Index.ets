import promptAction from '@ohos.promptAction';
import servoshell from 'libservoshell.so';

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
  xComponentAttrs: XComponentAttrs = {
    id: 'ServoDemo',
    type: XComponentType.SURFACE,
    libraryname: 'servoshell',
  }

  @State urlToLoad: string = ""
  @State tablist: Array<number> = [1];
  @State currentIndex: number = 0;

  aboutToAppear(): void {
    console.info("aboutToAppear - registering callbacks!")
    servoshell.registerURLcallback((new_url: string) => {
      console.info('New URL from native: ', new_url)
      this.urlToLoad = new_url
    })
    servoshell.registerPromptToastCallback(prompt_toast)
  }

  // Called when the user swipes from the right or left edge to the middle
  // Default behavior is bringing the app to the background.
  onBackPress(): boolean {
    servoshell.goBack();
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
            this.tablist.push(this.tablist[this.tablist.length-1]+1);
            this.currentIndex = this.tablist.length - 1;
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
            servoshell.goForward()
          })
        TextInput({ placeholder: 'URL', text: $$this.urlToLoad })
          .type(InputType.URL)
          .width('76%')
          .onChange((value) => {
            this.urlToLoad = value
          })
          .onSubmit((EnterKeyType) => {
            servoshell.loadURL(this.urlToLoad)
            console.info('Load URL: ', this.urlToLoad)
          })
      }

      Tabs({ barPosition: BarPosition.Start, index: this.currentIndex}) {
        ForEach(this.tablist, (item: number) => {
          TabContent() {
            XComponent(this.xComponentAttrs)
              .focusable(true)
          }.tabBar(String(item))
        })
      }.onChange((index: number) => {
        console.info("Focusing Tab number %d", index)
        servoshell.focusWebview(index);
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
