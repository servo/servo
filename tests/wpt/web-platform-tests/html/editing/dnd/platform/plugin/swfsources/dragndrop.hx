import Std;
import flash.Lib;
import flash.events.MouseEvent;

class DragNDrop
{
  static var s_box_draggable:Box;
  static var s_box_target:Box;
  static var s_dragging:Bool = false;

  private static function main()
  {
    /* Blue box (target). */
    s_box_target = new Box(0x0000ff, 100, 100, 150, 150);
    /* Orange box (draggable). */
    s_box_draggable = new Box(0xffa500, 25, 25, 50, 50);

    flash.Lib.current.stage.addEventListener(MouseEvent.MOUSE_DOWN, OnMouseDown);
    flash.Lib.current.stage.addEventListener(MouseEvent.MOUSE_MOVE, OnMouseMove);
    flash.Lib.current.stage.addEventListener(MouseEvent.MOUSE_UP, OnMouseUp);
  }

  static function OnMouseDown(event:MouseEvent):Void
  {
    if (s_box_draggable.IsHit(event.stageX, event.stageY))
      s_dragging = true;
  }

  static function OnMouseUp(event:MouseEvent):Void
  {
      s_dragging = false;

      /* Check if passed. */
      if (s_box_draggable.IsWithin(s_box_target))
      {
        s_box_draggable.Hide();
        s_box_target.SetColor(0xffee00);
      }
  }

  static function OnMouseMove(event:MouseEvent):Void
  {
    if (s_dragging)
    {
      ClearCanvas();
      s_box_target.Redraw();
      s_box_draggable.Move(event.stageX, event.stageY);
    }
  }

  static function ClearCanvas():Void
  {
    var mc:flash.display.MovieClip = flash.Lib.current;
    mc.graphics.beginFill(0xffffff);
    mc.graphics.drawRect(0, 0, flash.Lib.current.stage.width, flash.Lib.current.stage.height);
    mc.graphics.endFill();
  }
}

class Box
{
  var m_mc:flash.display.MovieClip;
  var m_color:Int;
  var m_rel_x:Float;
  var m_rel_y:Float;
  var m_x:Float;
  var m_y:Float;
  var m_width:Int;
  var m_height:Int;

  public function new(color:Int, x:Int, y:Int, width:Int, height:Int)
  {
    m_mc = flash.Lib.current;
    m_color = color;
    m_x = x;
    m_y = y;
    m_width = width;
    m_height = height;

    Redraw();
  }

  public function IsHit(x:Float, y:Float):Bool
  {
    if ((x >= m_x && x <= m_x + m_width) && (y >= m_y && y <= m_y + m_height))
    {
      m_rel_x = x - m_x;
      m_rel_y = y - m_y;
      return true;
    }

    return false;
  }

  public function IsWithin(other:Box):Bool
  {
    return m_x >= other.m_x && m_x + m_width <= other.m_x + other.m_width
           && m_y >= other.m_y && m_y + m_height <= other.m_y + other.m_height;
  }

  public function Hide():Void
  {
    m_width = 0;
    m_height = 0;
  }

  public function SetColor(color:Int):Void
  {
    m_color = color;
    Redraw();
  }

  public function Move(x:Float, y:Float):Void
  {
    /* Accounting for click offset. */
    m_x = x - m_rel_x;
    m_y = y - m_rel_y;
    Draw(Std.int(m_x), Std.int(m_y), m_width, m_height);
  }

  public function Redraw():Void
  {
    Draw(Std.int(m_x), Std.int(m_y), m_width, m_height);
  }

  private function Draw(x:Int, y:Int, width:Int, height:Int)
  {
    /* Draw moved rect. */
    m_mc.graphics.beginFill(m_color);
    m_mc.graphics.drawRect(x, y, width, height);
    m_mc.graphics.endFill();
  }
}
