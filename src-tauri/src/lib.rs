// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

mod commands;
mod db;
mod models;
mod parser;
#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod tag_tests;
#[cfg(test)]
mod task_link_tests;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let conn = db::init_app_db(&handle)?;
            app.manage(std::sync::Mutex::new(conn));
            let _ = db::backup_db(&handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::entries::get_entry,
            commands::entries::save_entry,
            commands::entries::get_entry_dates,
            commands::tasks::get_tasks,
            commands::tasks::add_task,
            commands::tasks::edit_task,
            commands::tasks::finish_task,
            commands::tasks::remove_task,
            commands::tasks::move_task_quadrant,
            commands::search::search,
            commands::system::export_all_data,
            commands::tags::sync_entry_tags,
            commands::tags::add_tag_to_entry,
            commands::tags::get_entry_tags,
            commands::tags::sync_task_tags,
            commands::tags::get_task_tags,
            commands::tags::get_all_tags,
            commands::tags::search_by_tag,
            commands::task_links::sync_entry_task_links,
            commands::task_links::get_entry_task_links,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
