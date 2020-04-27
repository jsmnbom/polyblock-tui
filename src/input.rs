use crate::{App, Key};

pub fn handle(key: Key, app: &mut App) {
    let routes = app.get_current_routes();
    let route = routes.last().unwrap();

    if key == Key::Esc {
        if app.pop_route().is_none() {
            app.quit();
        }
    } else {
        route.get_impl().handle_key(key, app);
    }
}
