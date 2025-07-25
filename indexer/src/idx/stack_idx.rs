use std::path::Path;

use common::error::AnalysisError;
use regex::Regex;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{BooleanQuery, FuzzyTermQuery, Occur, Query, RegexQuery},
    schema::*,
    Index,
};

pub struct ThreadSearchIdx {
    index: Index,
    method_raw: Field,
    method_token: Field,
}

impl ThreadSearchIdx {
    pub fn create(path: &str) -> Result<Self, AnalysisError> {
        let mut schema_builder = Schema::builder();
        let method_raw = schema_builder.add_text_field("method_raw", STRING | STORED);
        let method_token = schema_builder.add_text_field("method_token", TEXT | STORED);
        
        let schema = schema_builder.build();
        let index_path = Path::new(path);
        let dir = MmapDirectory::open(index_path)?;
        let index = Index::open_or_create(dir, schema.clone())?;
        Ok(Self {
            index,
            method_raw,
            method_token,
        })
    }

    pub fn add_doc(&self, method_value: &str) -> Result<(), AnalysisError> {
        let mut index_writer = self.index.writer(50_000_000)?; // 缓冲区大小 (字节)
                                                               // 添加文档 1
        index_writer.add_document(doc!(
            self.method_raw => method_value,
            self.method_token => method_value
        ))?;

        index_writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, max_edits: u8) -> Result<Vec<String>, AnalysisError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let mut queries = vec![];

        if contains_wildcard(query_str) {
            let pattern = safe_regex(query_str)?;
            queries.push((
                Occur::Should,
                Box::new(RegexQuery::from_pattern(&pattern, self.method_token)?) as Box<dyn Query>,
            ));
        } else {
            queries.push((
                Occur::Should,
                Box::new(FuzzyTermQuery::new(
                    Term::from_field_text(self.method_raw, query_str),
                    max_edits,
                    true,
                )) as Box<dyn Query>,
            ));

            queries.push((
                Occur::Should,
                Box::new(FuzzyTermQuery::new(
                    Term::from_field_text(self.method_token, query_str),
                    max_edits,
                    true,
                )),
            ));
              queries.push((
                Occur::Should,
                Box::new(FuzzyTermQuery::new_prefix(
                    Term::from_field_text(self.method_token, query_str),
                    1,
                    true,
                )),
            ));
        }

        let query = BooleanQuery::new(queries);
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

        let mut result = Vec::new();
        for (_, doc_address) in top_docs {
            let doc = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let class_str = doc
                .get_first(self.method_raw)
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            let method_str = doc
                .get_first(self.method_token)
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            result.push(format!("{}.{}", class_str, method_str));
        }
        Ok(result)
    }

    pub fn clean(&self) -> Result<(), AnalysisError> {
        let mut writer = self.index.writer::<tantivy::TantivyDocument>(50_000_000)?;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }
}


fn safe_regex(input: &str) -> Result<String, AnalysisError> {
    let match_str = "[a-zA-z0-9]*";
    let mut result = String::from(match_str);
    for c in input.chars() {
        match c {
            '*' => result.push_str(match_str),
            ' ' => result.push_str(match_str),
            other => result.push(other),
        }
    }
    if !result.ends_with("*") {
        result.push_str(match_str);    
    }
    println!("生成的正则表达式: {}", result);
    match Regex::new(&result){
        Ok(_) => Ok(result),
        Err(err) => Err(AnalysisError::RegError(format!("正则表达式解析错误: {}", err))),
    }
}



fn contains_wildcard(query: &str) -> bool {
    !query.contains(".") && (query.contains('*') || query.contains('?') || query.contains(" "))
}

#[cfg(test)]
pub mod tests {
    use std::vec;

    use super::*;

    #[test]
    pub fn test_search_pre() {
        let search_index = ThreadSearchIdx::create("D:\\dump\\.idx").unwrap();
        search_index.clean().unwrap();
        // 添加测试数据
        let test_data = vec![
            "com.jiuqi.nr.entity.search",
            "com.jiuqi.nr.task.query",
            "org.slf4j.LoggerFactory.getLogger",
            "com.jiuqi.np.definition.facade.FieldDefine.create",
        ];

        for method in &test_data {
            search_index.add_doc(method).unwrap();
        }


        let results = search_index.search("co", 1).unwrap();
        assert!(!results.is_empty(), "无返回");
        println!("'co' 结果: {:?}", results);
        
    }

    #[test]
    pub fn test_search_reg() {
        let search_index = ThreadSearchIdx::create("D:\\dump\\.idx").unwrap();
        search_index.clean().unwrap();
        // 添加测试数据
        let test_data = vec![
            "com.jiuqi.nr.entity.search",
            "com.jiuqi.nr.task.query",
            "org.slf4j.LoggerFactory.getLogger",
            "com.jiuqi.np.definition.facade.FieldDefine.create",
        ];

        for method in &test_data {
            search_index.add_doc(method).unwrap();
        }

        let full_class = "Field*";
        let results = search_index.search(full_class, 2).unwrap();
        assert!(!results.is_empty(), "无返回");
        println!("{:?}", results);
    }

#[test]
    pub fn test_search_any() {
        let search_index = ThreadSearchIdx::create("D:\\dump\\.idx").unwrap();
        search_index.clean().unwrap();
        // 添加测试数据
        let test_data = vec![
            "com.jiuqi.nr.entity.search",
            "com.jiuqi.nr.task.query",
            "org.slf4j.LoggerFactory.getLogger",
            "com.jiuqi.np.definition.facade.FieldDefine.create",
        ];

        for method in &test_data {
            search_index.add_doc(method).unwrap();
        }

        
        let results = search_index.search("def", 2).unwrap();
        assert!(!results.is_empty(), "无返回");
        println!("'def' 结果: {:?}", results);
        
    }

}
