pub trait AudioRenderer: Send + 'static {
    fn render(&mut self, sample: Box<dyn AsRef<[f32]>>, channel: u32);
}
