use crate::{view, App, Key, RouteId};

pub fn handle(key: Key, app: &mut App) {
    let routes = app.get_current_routes();
    let route = routes.last().unwrap();

    if key == Key::Esc {
        if app.pop_route().is_none() {
            app.quit();
        }
    } else {
        match route.id {
            RouteId::Home => view::home::handle_key(key, app),
            RouteId::InstanceMenu => view::instance_menu::handle_key(key, app),
            RouteId::NewInstance => view::new_instance::handle_key(key, app),
            RouteId::RemoveInstance => view::remove_instance::handle_key(key, app),
            RouteId::RenameInstance => view::rename_instance::handle_key(key, app),
        }
    }
}
