use std::env;

pub struct EnvVars {
    pub cpu_file: String,
    pub cpu_top: String,
    pub thread_dump: String,
    pub gc: String,
    pub gc_util: String,
}

impl EnvVars {
  pub fn load() -> Self {
      dotenv::dotenv().ok();
      EnvVars {
          cpu_file: env::var("CPU_FILE_KEY_WORDS").expect("找不到环境变量中的信息"),
          cpu_top: env::var("CPU_TOP_KEY_WORDS").expect("找不到环境变量中的信息"),
          thread_dump: env::var("THREAD_DUMP_KEY_WORDS").expect("找不到环境变量中的信息"),
          gc: env::var("GC_KEY_WORDS").expect("找不到环境变量中的信息"),
          gc_util: env::var("GC_UTIL_KEY_WORDS").expect("找不到环境变量中的信息"),
      }
  }
}