use std::{collections::HashMap, sync::Arc};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct StringPool;

// 全局常驻字符串池
static STRING_POOL: Lazy<Mutex<HashMap<String, Arc<str>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl StringPool {
    pub fn get_arc_str(str: &str) -> Arc<str> {
        let mut pool = STRING_POOL.lock().unwrap();
        if let Some(a) = pool.get(str) {
            a.clone()
        } else {
            let arc = Arc::<str>::from(str);
            pool.insert(str.to_string(), arc.clone());
            arc
        }
    }
}
