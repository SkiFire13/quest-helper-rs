pub trait Client {
    fn get_input(&self, event: &str, day: u32, part: u32) -> Option<String>;
}
