use crate::compressor::encoder::Encoder;

pub struct IncrementalMerger<'a>{
  encoder: &'a mut Encoder
}

impl <'a> IncrementalMerger<'a> {
    /// 创建一个新的增量合并器
    pub fn new(encoder: &'a mut Encoder) -> Self {
       Self { encoder }
    }

    /// 增量压缩一个字符串切片的栈
    /// 返回一个包含页ID和局部ID组合的向量
    pub fn compress_stack(&mut self, stack: Vec<&str>) -> Vec<u32> {
       stack.into_iter()
            .map(|key| self.encoder.encode(key)) // 插入每个条目到编码器中
            .collect() // 收集所有的页ID和局部ID组合
    }
    
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_increment() {
        let mut encoder = super::Encoder::new();
        let mut merger = super::IncrementalMerger::new(&mut encoder);
        
        // 模拟一个调用栈
        let stack = vec!["com.example.Foo.method1", "com.example.Bar.method2"];
        
        // 压缩调用栈
        let compressed_ids = merger.compress_stack(stack);
        
        // 验证压缩结果
        assert_eq!(compressed_ids.len(), 2); // 应该有两个条目
        assert!(compressed_ids[0] != compressed_ids[1]); // 确保不同的条目有不同的ID
    }
  }