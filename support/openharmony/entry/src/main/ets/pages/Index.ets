import { BusinessError } from '@ohos.base';
import servoshell from 'libservoshell.so';

@Entry({ useSharedStorage: true })
@Component
struct Index {
  // The state seem to be required to be on the top.
  @State urlToLoad: string = ""
  @State tablist: Array<number> = [0];
  @State currentId: number = 0;
  private nextTabId: number = 1;

  @Builder
  tabBuilder(id: number, index: number) {
    Row() {
      // TODO: Hook-up title here.
      Text(String(index))
        .padding('10pt')
      Button('X')
        .backgroundColor('#00000000')
        .visibility(this.tablist.length > 1 && index != 0 ? Visibility.Visible : Visibility.Hidden)
        .fontColor(Color.Black)
        .fontWeight(FontWeight.Bolder)
        .onClick((event) => {
          this.currentId = this.tablist[0];
          servoshell.deleteWebview(index, this.tablist);
          this.tablist.splice(index, 1);
          // TODO: Instead of defaulting to 0, remember the previously focused Webview.
          servoshell.focusWebview(0, this.tablist)
        })
    }
    .width(65)
    .height(40)
    .justifyContent(FlexAlign.Center)
    .backgroundColor(this.currentId === id ? '#1B8381' : Color.White)
    .borderRadius(10)
    .shadow({
      radius: 10,
      color: Color.Gray,
      offsetX: 10,
      offsetY: 10
    })
    .onClick(() => {
      this.currentId = id;
      servoshell.focusWebview(index, this.tablist)
    })
  }

  aboutToAppear(): void {
    console.info("aboutToAppear - registering callbacks!")
    servoshell.registerURLcallback((new_url: string) => {
      console.info('New URL from native: ', new_url)
      this.urlToLoad = new_url
    })
    const promptAction = this.getUIContext().getPromptAction();
    servoshell.registerPromptToastCallback((msg: string) => {
      try {
        promptAction.showToast({
          message: msg,
          duration: 2000
        });
      } catch (error) {
        let err = error as BusinessError;
        console.warn(`showToast failed with error code ${err.code}, message: ${err.message}`);
      }
    })
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
    Flex({ direction: FlexDirection.Column }) {
      Row() {
        Button('+')
          .backgroundColor(Color.White)
          .fontColor(Color.Black)
          .fontWeight(FontWeight.Bolder)
          .fontSize(22)
          .width('12%')
          .onClick((event) => {
            const new_id = this.nextTabId++;
            // the order is important as adding a new tab will automatically create a new surface.
            servoshell.nextWindowId(new_id);
            this.tablist.push(new_id)
            this.currentId = new_id;
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

      List({ space: 10 }) {
        ForEach(this.tablist, (item: number, index: number) => {
          ListItem() {
            this.tabBuilder(item, index)
          }
        })
      }
      .listDirection(Axis.Horizontal)
      .height(40)
      .width('100%')
      .padding({ left: 10, right: 10 })
      .backgroundColor('#e1e1e1')

      Stack() {
        ForEach(this.tablist, (item: number) => {
          XComponent({ id: 'servo-window-' + item, type: XComponentType.SURFACE, libraryname: 'servoshell' })
            .width('100%')
            .height('100%')
            .visibility(this.currentId === item ? Visibility.Visible : Visibility.Hidden)
            .focusable(true)
            .focusOnTouch(true)
        }, (item: number) => String(item))
      }
      .flexGrow(1)
      .width('100%')
      .backgroundColor('#e1e1e1')
    }
    .width('100%')
  }
}
