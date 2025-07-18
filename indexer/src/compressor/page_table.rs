use std::collections::HashMap;

use crate::compressor::page::Page;

pub struct PageTable {
    pages: HashMap<u16, Page>, // 页ID到页对象的映射
    next_page_id: u16, // 下一个可用的页ID
}

impl PageTable {
    /// 创建一个新的空页表
    pub fn new() -> Self {
        PageTable {
            pages: HashMap::new(),
            next_page_id: 0,
        }
    }

    /// 获取或创建一个新的页
    pub fn get_or_create_page(&mut self) -> u16 {
        let page_id = self.next_page_id; // 使用当前的页ID
        self.pages.entry(page_id).or_insert_with(Page::new); // 如果页不存在，则创建新页
        self.next_page_id += 1; // 更新下一个可用的页ID
        page_id // 返回当前页ID
    }

    /// 根据键获取页ID
    pub fn calc_page_id(&self, key: &str) -> u16 {
        // 计算给定键的页ID
        let hs = fxhash::hash(key);
        (hs % 65536) as u16 //  64 位哈希值压缩成 16 位
    }
    
    /// 插入一个条目到页表中
    /// 返回页ID和局部ID的组合(u32,前16位为页ID，后16位为页内ID)
    pub fn insert(&mut self, key: &str) -> u32 {
        let page_id = self.calc_page_id(key);
        if !self.pages.contains_key(&page_id) {
            self.pages.insert(page_id, Page::new());
        }
        let local_id = self.pages.get_mut(&page_id).unwrap().get_or_insert(key);
        (page_id as u32) << 16 | (local_id as u32) // 返回页ID和局部ID的组合
    }

    /// 根据键查找对应的字符串
    pub fn lookup(&self, id: &u32) -> Option<&str> {
        let page_id = id >> 16; // 获取页ID
        let local_id = id & 0xFFFF; // 获取局部ID
        self.pages.get(&(page_id as u16)).and_then(|page| page.get_string(local_id as u16)) // 从页中获取对应的字符串
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn test_page_table() {
        let mut page_table = super::PageTable::new();
        let id1 = page_table.insert("test1");
        let id2 = page_table.insert("test2");
        assert_ne!(id1, id2); // 确保不同的条目有不同的ID
        assert_eq!(page_table.lookup(&id1), Some("test1"));
        assert_eq!(page_table.lookup(&id2), Some("test2"));
        assert_eq!(page_table.lookup(&0xFFFFFFFF), None); // 测试不存在的ID
    }

    /// 测试创建新页
     #[test]
    fn test_page_table_create() {
        let mut page_table = super::PageTable::new();
        let page_id = super::PageTable::get_or_create_page(&mut page_table); // 创建一个新页
        assert!(page_table.pages.contains_key(&page_id)); // 确保页已创建
        assert_eq!(page_table.next_page_id, page_id + 1); // 确保下一个页ID已更新
        let id1 = page_table.insert("test1"); // 插入一个条目 
        assert_eq!(page_table.lookup(&id1), Some("test1")); // 确保条目可以被查找
        let id2 = page_table.insert("test2"); // 插入另一个条目
        assert_ne!(id1, id2); // 确保不同的条目有不同的ID
        assert_eq!(page_table.lookup(&id2), Some("test2")); // 确保第二个条目可以被查找
        let id3 = page_table.insert("test1"); // 再次插入相同的条目
        assert_eq!(id1, id3); // 确保相同的条目返回相同的ID
        assert_eq!(page_table.lookup(&id3), Some("test1")); //
        assert_eq!(page_table.lookup(&0xFFFFFFFF), None); // 测试不存在的ID
    }
    
  }