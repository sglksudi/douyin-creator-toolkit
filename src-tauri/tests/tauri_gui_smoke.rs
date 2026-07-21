use douyin_creator_tools_lib::build_app_shell;
use tauri::Manager;

#[test]
fn app_shell_builds_main_window_with_mock_runtime() {
    let app = build_app_shell(tauri::test::mock_builder())
        .build(tauri::generate_context!("tauri.conf.json"))
        .expect("app shell should build with the mock runtime");

    let main_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .expect("tauri config should define a main window");

    tauri::WebviewWindowBuilder::from_config(&app, main_config)
        .expect("main window config should be valid")
        .build()
        .expect("main webview window should build with the mock runtime");

    let main_window = app
        .get_webview_window("main")
        .expect("main webview window should be registered");

    assert_eq!(main_window.label(), "main");
}
