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

  // The state seem to be required to be on the top.
  @State urlToLoad: string = ""
  @State tablist: Array<number> = [0];
  @State currentIndex: number = 0;

  @Builder tabBuilder(title: string, index: number) {
    Row() {
      Text(String(index))
        .padding('10pt')
      Button('X')
      .backgroundColor('#00000000')
      .visibility(this.tablist.length>1 && index != 0 ? Visibility.Visible: Visibility.Hidden)
      .fontColor(Color.Black)
      .fontWeight(FontWeight.Bolder)
      .onClick((event) => {
          this.currentIndex = 0;
          servoshell.deleteWebview(index, this.tablist);
          this.tablist.splice(index,1);
        })
    }
    .width(65)
    .height(50)
    .justifyContent(FlexAlign.Center)
    .backgroundColor(this.currentIndex === index ? '#1B8381': Color.White)
    .borderRadius(10)
  }

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
            const new_id = this.tablist[this.tablist.length-1]+1;
            this.currentIndex = this.tablist.push(new_id) - 1;
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
        ForEach(this.tablist, (item: number, index: number) => {
          TabContent() {
            XComponent(this.xComponentAttrs)
              .focusable(true)
          }.tabBar(this.tabBuilder(String(item), index))
        })
      }.onChange((index: number) => {
        this.currentIndex = index;
        servoshell.focusWebview(index, this.tablist);
      })
      .backgroundColor('#c7c7c7')
    }
    .width('100%')
  }
}

interface XComponentAttrs {
  id: string;
  type: number;
  libraryname: string;
}
