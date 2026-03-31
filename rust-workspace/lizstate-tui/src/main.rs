fn main() {
    todo!()
}

/*
use ratatui::prelude::*;

use crate::app::App;
use crate::app::AppState;

mod camera;

mod app;

// This isn't any different from yoloing CTRL+C or whatever.
static mut FORCE_CLOSE: bool = false;

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let app = App::new();

    let result = main_loop(terminal, app);

    ratatui::restore();
    result
}

fn main_loop<T: Backend>(mut terminal: Terminal<T>, mut app: App) -> std::io::Result<()> {
    while unsafe { !FORCE_CLOSE } {
        terminal.draw(|frame| {
            let intersection = frame.area().intersection(Rect::new(0, 0, 80, 25));
            app.render(frame, intersection)
        })?;
        app.handle_events()?;
    }
    Ok(())
}
*/
