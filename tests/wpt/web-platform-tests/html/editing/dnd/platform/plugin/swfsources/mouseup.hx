import flash.Lib;
import flash.display.Sprite;
import flash.events.MouseEvent;

class MouseUp extends Sprite
{
  var s:Sprite;

  static function main()
  {
    new MouseUp();
  }

  public function new()
  {
    super();
    flash.Lib.current.addChild(this);

    s = new Sprite();
    s.graphics.beginFill(0xffa500);
    s.graphics.drawRect(0, 0, stage.stageWidth, stage.stageHeight);
    s.graphics.endFill();
    stage.addChild(s);

    stage.addEventListener(MouseEvent.MOUSE_UP, OnMouseUp);
  }

  function OnMouseUp(event:MouseEvent):Void
  {
    s.graphics.beginFill(0xffee00);
    s.graphics.drawRect(0, 0, stage.stageWidth, stage.stageHeight);
    s.graphics.endFill();
  }
}
