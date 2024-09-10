use std::sync::Mutex;

/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub path: Mutex<String>, 
}
