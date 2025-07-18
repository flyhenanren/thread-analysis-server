use domain::context::Context;
use task::async_task::TaskExecutor;


/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub executor: TaskExecutor,
    pub context: Context
}