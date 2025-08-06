use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub struct LruCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Eq + Hash + Clone, V> LruCache<K, V> {
    /// 创建一个新的 LRU 缓存
    pub fn new(capacity: usize) -> Self {
        LruCache {
            capacity,
            cache: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// 获取一个条目，如果存在则返回其值
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.cache.contains_key(key) {
            // 如果存在，更新访问顺序
            self.order.retain(|k| k != key);
            self.order.push_back(key.clone());
            self.cache.get(key)
        } else {
            None
        }
    }

    /// 插入一个条目到缓存中
    pub fn insert(&mut self, key: K, value: V) {
        if self.cache.contains_key(&key) {
            self.order.retain(|k| k != &key);
        } else if self.cache.len() >= self.capacity {
            // 如果缓存已满，移除最旧的条目
            if let Some(oldest_key) = self.order.pop_front() {
                self.cache.remove(&oldest_key);
            }
        }
        // 更新访问顺序
        self.order.push_back(key.clone());
        // 插入新条目
        self.cache.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use crate::compressor::encoder::Encoder;
    use crate::compressor::incremental::IncrementalMerger;

    #[test]
    fn test_lru_cache() {
        // 模拟 dump1 的堆栈信息（线程调用栈）
        let dump1 = vec![
            vec![
                "com.example.Foo.method1",
                "com.example.Bar.method2",
                "com.example.Baz.method3",
            ],
            vec!["com.example.Foo.method1", "com.example.Bar.method2"],
        ];

        // 模拟 dump2 的堆栈信息
        let dump2 = vec![
            vec!["com.example.Foo.method1", "com.example.Bar.method2"],
            vec!["com.example.Qux.method4"],
        ];

        // 创建一个全局编码器实例
        let mut encoder = Encoder::new();

        // 创建增量合并器持有编码器引用
        let mut merger = IncrementalMerger::new(&mut encoder);

        println!("Processing dump1...");
        for stack in dump1 {
            // 压缩调用栈字符串为一串全局ID
            let compressed_ids = merger.compress_stack(stack);
            println!("Dump1 stack compressed IDs: {:?}", compressed_ids);
        }

        println!("Processing dump2...");
        for stack in dump2 {
            let compressed_ids = merger.compress_stack(stack);
            println!("Dump2 stack compressed IDs: {:?}", compressed_ids);
        }

        // 测试反解，打印部分ID对应的字符串
        println!("Decode sample IDs:");
        for id in [1551040512, 3607429120, 1607794688] {
            if let Some(s) = encoder.decode(id) {
                println!("ID 0x{:08x} => {}", id, s);
            } else {
                println!("ID 0x{:08x} => <not found>", id);
            }
        }
    }
}
