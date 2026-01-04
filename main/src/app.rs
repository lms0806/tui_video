pub struct App {
    pub playing: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            playing: true,
            should_quit: false,
        }
    }
}
