use cached::{Cached, SizedCache, UnboundCache};
use once_cell::sync::Lazy;
use std::{fmt::format, sync::{Arc, Mutex}};

pub struct GlobalCache;

// L1缓存，热点数据
static L1_CACHE:  Lazy<Mutex<SizedCache<String, Arc<dyn std::any::Any + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(SizedCache::with_size(100)));
// L2缓存，非热点数据
static L2_CACHE:  Lazy<Mutex<UnboundCache<String, Arc<dyn std::any::Any + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(UnboundCache::new()));

impl GlobalCache {
    pub fn put<T: 'static + Send + Sync>(key: String, value: T) {
        Self::put_l2(key.to_string(), value);
    }

    pub fn get<T: 'static + Send + Sync>(key: &str) -> Option<Arc<T>> {
      if let Some(cached) = Self::get_l1::<T>(key) {
          return Some(cached);
      }
      
      if let Some(cached) = Self::get_l2::<T>(key) {
          Self::put_l1(key.to_string(), cached.clone());
          return Some(cached);
      }
      None
    }

    pub fn exists(key: &str) -> bool {
        let cache = L2_CACHE.lock().unwrap();
        cache.get_store().contains_key(key)
    }
    
    pub fn reset(){
      let mut cache1 = L1_CACHE.lock().unwrap();
      let mut cache2 = L2_CACHE.lock().unwrap();
      cache1.cache_clear();
      cache2.cache_clear();
    }
     // L1 缓存操作
    fn put_l1<T: 'static + Send + Sync>(key: String, value: T) {
        let mut cache = L1_CACHE.lock().unwrap();
        cache.cache_set(key, Arc::new(value));
    }

    fn get_l1<T: 'static + Send + Sync>(key: &str) -> Option<Arc<T>> {
        let mut cache = L1_CACHE.lock().unwrap();
        cache
            .cache_get(key)
            .and_then(|arc_any| arc_any.clone().downcast::<T>().ok())
    }

    // L2 缓存操作
    fn put_l2<T: 'static + Send + Sync>(key: String, value: T) {
        let mut cache = L2_CACHE.lock().unwrap();
        cache.cache_set(key, Arc::new(value));
    }

    fn get_l2<T: 'static + Send + Sync>(key: &str) -> Option<Arc<T>> {
        let mut cache = L2_CACHE.lock().unwrap();
        cache
            .cache_get(key)
            .and_then(|arc_any| arc_any.clone().downcast::<T>().ok())
    }
}


pub struct CacheKey;

impl CacheKey {

    pub fn call_tree(work_space_id: &str) -> String{
      format!("{}::CALL_TREE",work_space_id)
    }

}