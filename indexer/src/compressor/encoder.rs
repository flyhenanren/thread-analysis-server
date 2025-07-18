use crate::compressor::page_table::PageTable;

pub struct Encoder{
  page_table: PageTable
}

impl Encoder {
    /// 创建一个新的编码器
    pub fn new() -> Self {
        Encoder {
            page_table: PageTable::new(),
        }
    }

     /// 插入字符串，返回全局ID
    pub fn encode(&mut self, s: &str) -> u32 {
        self.page_table.insert(s)
    }

    /// 查ID对应的字符串
    pub fn decode(&self, id: u32) -> Option<&str> {
        self.page_table.lookup(&id)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_encode() {
        let mut encoder = super::Encoder::new();
        let id1 = encoder.encode("test1");
        let id2 = encoder.encode("test2");
        assert_ne!(id1, id2); // 确保不同的条目有不同的ID
        assert_eq!(encoder.decode(id1), Some("test1")); 
        assert_eq!(encoder.decode(id2), Some("test2"));
    }
  }