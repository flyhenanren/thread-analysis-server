use std::{collections::HashMap, usize};

use crate::models::thread::{CallFrame, Frame, Thread};

#[derive(Debug, Clone, PartialEq)]
pub struct CallTree {
    method_name: String,
    line: u32,
    count: u32,
    next: Option<Vec<Box<CallTree>>>,
}

impl CallTree {
    pub fn new(threads: Vec<Thread>) -> Vec<CallTree> {
        let mut root: HashMap<String, CallTree> = HashMap::new();
        for thread in threads {
            if thread.frames.len() > 0 {
                Self::convert_to_call_tree(thread, &mut root);    
            }
        }
        root.into_values().collect()
    }

    fn convert_to_call_tree(thread: Thread, root: &mut HashMap<String, CallTree>) {
        let mut path: Vec<CallFrame> = thread.frames;
        if let Some(root_frame) = path.pop() {
            if let Frame::MethodCall = &root_frame.frame {
                let method_name = root_frame.method_name.clone().unwrap_or_default();
                let line = root_frame.line_number.unwrap_or_default();
                let root_node: &mut CallTree = root.entry(method_name.clone()).or_insert(CallTree {
                    method_name,
                    line,
                    count: 0,
                    next: None,
                });
                root_node.count += 1;
                Self::build_tree_from_frames(path, root_node);
            }
        }
    }
    fn build_tree_from_frames(mut frames: Vec<CallFrame>, parent_node: &mut CallTree) {
        if let Some(frame) = frames.pop() {
            let method_name = frame.method_name.as_ref().unwrap_or(&String::new()).clone();
            let next_nodes = parent_node.next.get_or_insert_with(Vec::new);
            let existing_node = next_nodes
                .iter_mut()
                .find(|node| node.method_name == method_name);
            if let Some(node) = existing_node {
                node.count += 1;
                Self::build_tree_from_frames(frames, node);
            } else {
                let mut new_node = CallTree {
                    method_name: method_name.clone(),
                    line: frame.line_number.unwrap_or_default(),
                    count: 1,
                    next: None,
                };
                Self::build_tree_from_frames(frames, &mut new_node);
                next_nodes.push(Box::new(new_node));
            }
        };
    }
}

#[cfg(test)]
pub mod tests {
    use std::{
        fs,
        io::{self, BufRead},
    };

    use chrono::Local;

    use crate::models::thread::{self, ThreadStatus};

    use super::*;

    #[test]
    pub fn test() {
        let dirs = fs::read_dir("D:\\dump\\a").unwrap();
        let start = Local::now();
        println!("start:{}", start);
        let mut threads = Vec::new();
        let mut count_file = 0;
        let mut count_threads = 0;
        for dir in dirs {
            count_file += 1;
            let entry = dir.unwrap();
            let path = entry.path();
            println!("{}-读取文件：{:?}", count_file, path);
            let file = fs::File::open(path).unwrap();
            let reader = io::BufReader::new(file);
            let mut lines: Vec<Vec<String>> = Vec::new();
            let mut current_group: Vec<String> = Vec::new();
            let mut start = false;
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if l.is_empty() {
                            start = false;
                            continue;
                        }
                        if l.contains("nid=") {
                            start = true;
                            if !current_group.is_empty() {
                                lines.push(current_group);
                                current_group = Vec::new();
                            }
                        }
                        if start {
                            current_group.push(l);
                        }
                    }
                    Err(_) => {}
                }
            }
            if !current_group.is_empty() {
                lines.push(current_group);
            }
            let build = Local::now();
            println!("{}-开始构建：{}，共{}行", count_file, build, lines.len());
            for group in lines {
                if !Thread::is_sys_thread(&group) {
                    let thread = Thread::new(group);
                    count_threads += 1;
                    match thread {
                        Ok(t) => threads.push(t),
                        Err(e) => println!("{}", e),
                    }
                };
            }
            let build = Local::now();
            println!("{}-构建结束：{}", count_file, build);
        }
        let build = Local::now();
        println!("调用树处理：{}，共：{}", build, count_threads);
        let call_tree = CallTree::new(threads);
        let build = Local::now();
        println!("调用树构建完毕：{}", build);
        println!("{:?}", call_tree)
    }
}
