use std::collections::HashMap;

/// 页对象，用于存储条目和反向索引
pub struct Page {
    entries: HashMap<String, u16>, //字符串到页内ID的索引
    reverse: Vec<String>, // 字符串列表，按页内ID顺序存储
}

impl Page {
    /// 创建一个新的空页
    pub fn new() -> Self {
        Page {
            entries: HashMap::new(),
            reverse: Vec::new(),
        }
    }

    /// 添加一个条目到页中
    pub fn get_or_insert(&mut self, key: &str) -> u16 {
        if let Some(&id) = self.entries.get(key) {
            return id; // 如果条目已存在，返回其ID
        }
        let id = self.reverse.len() as u16; // 新条目的页内ID为当前列表长度
        self.entries.insert(key.to_string(), id); // 插入到索引中
        self.reverse.push(key.to_string()); // 添加到反向索引列表中
        id // 返回新条目的ID  
    }

    /// 获取条目的页内ID
    pub fn get(&self, key: &str) -> Option<u16> { 
        self.entries.get(key).cloned() // 从索引中获取条目ID
    } 
    
    /// 根据页内ID获取对应的字符串
    pub fn get_string(&self, id: u16) -> Option<&str> {
        self.reverse.get(id as usize).map(|s| s.as_str()) // 根据页内ID获取对应的字符串 
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn test_page() {
        let mut page = super::Page::new();
        let id1 = page.get_or_insert("test1");
        let id2 = page.get_or_insert("test2");
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(page.get_string(id1), Some("test1"));
        assert_eq!(page.get_string(id2), Some("test2"));
        assert_eq!(page.get("test1"), Some(0));
        assert_eq!(page.get("test2"), Some(1));
    }
}