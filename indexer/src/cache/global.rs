use cached::{Cached, UnboundCache};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub struct GlobalCache;

// 全局通用缓存
static BIG_CACHE: Lazy<Mutex<UnboundCache<String, Arc<dyn std::any::Any + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(UnboundCache::new()));

impl GlobalCache {
    pub fn put<T: 'static + Send + Sync>(key: String, value: T) {
        let mut cache = BIG_CACHE.lock().unwrap();
        cache.cache_set(key, Arc::new(value));
    }

    pub fn get<T: 'static + Send + Sync>(key: &str) -> Option<Arc<T>> {
        let mut cache = BIG_CACHE.lock().unwrap();
        cache
            .cache_get(key)
            .and_then(|arc_any| arc_any.clone().downcast::<T>().ok())
    }

    pub fn exists(key: &str) -> bool {
        let cache = BIG_CACHE.lock().unwrap();
        cache.get_store().contains_key(key)
    }
}
