use std::sync::Mutex;

/*
 * 存放应用程序的状态
 */
pub struct AppState {
    pub health_check_response: String, // 共享所有线程，初始化之后是个不可变的状态
    pub visit_count: Mutex<u32>, // 访问的次数，也可以给每个线程共享，是个可变的数值类型，Mutex是保证线程安全的
}
