pub enum Event {
    ResizeEvent(uint, uint, comm::Chan<()>),
    ReflowEvent        
}

