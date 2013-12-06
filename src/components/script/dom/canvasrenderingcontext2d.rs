use dom::bindings::utils::{DOMString, Reflectable, Reflector, reflect_dom_object};
use dom::bindings::codegen::CanvasRenderingContext2DBinding;
use dom::window::Window;
use azure::azure_hl::{DrawTarget, Color, B8G8R8A8, SkiaBackend, StrokeOptions, DrawOptions};
use geom::rect::Rect;
use azure::azure_hl::{ColorPattern};
use geom::point::Point2D;
use geom::size::Size2D;


pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    window: @mut Window,
    width: i32,
    height: i32,
}

impl CanvasRenderingContext2D {
    pub fn new_inherited(window: @mut Window) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            window: window,
	    width: 0,
            height: 0,
        }
    }
	
    pub fn new(window: @mut Window) -> @mut CanvasRenderingContext2D {
        reflect_dom_object(@mut CanvasRenderingContext2D::new_inherited(window), window, CanvasRenderingContext2DBinding::Wrap)
    }



    /* 
      fn FillRect - It takes the (x,y)co-ordinates and height , weight as parameters of the rectangle to be filled.
	            The rectangle will be filled with the color specified by colorPattern variable that holds a default color(r,g,b,a).
		    We have used Color(1.0, 0.0, 0.0, 0.0) i.e Red      	
    */
     pub fn FillRect(&self, x: f32, y: f32, width: f32, height: f32) {  
      let colorpattern = ColorPattern(Color(1.0, 0.0, 0.0, 0.0));
      let Azrect = Rect(Point2D(x,y), Size2D(width,height));
      let drawtarget = DrawTarget::new(SkiaBackend, Size2D(100 as i32,100 as i32), B8G8R8A8);
      drawtarget.fill_rect(&Azrect, &colorpattern); 	
    }

    
    /*
     fn clearRect - It takes (x,y) co-ordinates and widht, height as parameters and clears the specified pixels of the rectangle.
    */
     pub fn ClearRect(&self, x: f32, y: f32, width: f32, height: f32) {

      let Azrect = Rect(Point2D(x,y), Size2D(width,height));
      let drawtarget = DrawTarget::new(SkiaBackend, Size2D(100 as i32,100 as i32), B8G8R8A8);
      drawtarget.clear_rect(&Azrect);
    }

    /*
     fn strokeRect - It takes (x,y) co-ordinates and widht, height as parameters of the rectangle to be created i.e no fill
    */
    pub fn StrokeRect(&self, x: f32, y: f32, width: f32, height: f32) {
      let colorpattern = ColorPattern(Color(1.0, 0.0, 0.0, 0.0));
      let Azrect = Rect(Point2D(x,y), Size2D(width,height));
      let drawtarget = DrawTarget::new(SkiaBackend, Size2D(100 as i32,100 as i32), B8G8R8A8);
      let strokeopts = StrokeOptions(10.0, 10.0); 
      let drawopts = DrawOptions(1.0, 0);  
      drawtarget.stroke_rect(&Azrect, &colorpattern, &strokeopts, &drawopts);
    }	

    /*
     fn strokeLine - It strokes the line from the start and end points provided as parameters.
    */
    pub fn StrokeLine(&self, x: f32, y: f32, x1: f32, y1: f32) {
  
      let colorpattern = ColorPattern(Color(1.0, 0.0, 0.0, 0.0));
      let pt1 = Point2D(x,y);
      let pt2 = Point2D(x1,y1);
      let drawtarget = DrawTarget::new(SkiaBackend, Size2D(100 as i32,100 as i32), B8G8R8A8);
      let strokeopts = StrokeOptions(10.0, 10.0); 
      let drawopts = DrawOptions(1.0, 0);  
      drawtarget.stroke_line(pt1, pt2, &colorpattern, &strokeopts, &drawopts);
    }	 

    pub fn Width(&self) -> i32 {                    
        self.width
    }

    pub fn SetWidth(&mut self, width: i32) {
        self.width = width
    }

    pub fn Height(&self) -> i32 {                
        self.height
    }

    pub fn SetHeight(&mut self, height: i32) {
        self.height = height
    }

}

impl Reflectable for CanvasRenderingContext2D {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
