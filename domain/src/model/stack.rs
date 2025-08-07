use std::{collections::{hash_map::Entry, HashMap}, sync::Arc};

use indexer::cache::pool::StringPool;

use crate::model::thread::{CallFrame, Frame, Thread};

#[derive(Debug, Clone, PartialEq)]
pub struct CallTree {
    method_name: Arc<str>,
    samples: u32,
    threads: Vec<Arc<str>>,
    times: Vec<Arc<str>>,
    next: Option<Vec<Arc<CallTree>>>,
}

impl CallTree {
    pub fn new(threads: Vec<Thread>) -> Vec<Arc<CallTree>> {
        let mut root: HashMap<Arc<str>, Arc<CallTree>> = HashMap::new();
        for thread in threads {
            if !thread.frames.is_empty() {
                Self::convert_to_call_tree(thread, &mut root);
            }
        }
        root.into_values().collect()
    }

    /// 将单个线程的调用帧插入到调用树根节点集合中。
    /// 如果根节点已存在则合并采样，否则新建根节点。
    fn convert_to_call_tree(thread: Thread, root: &mut HashMap<Arc<str>, Arc<CallTree>>) {
        let mut frames: Vec<CallFrame> = thread.frames;
        if let Some(root_frame) = frames.pop() {
            if let Frame::MethodCall = &root_frame.frame {
                let method_name = StringPool::get_arc_str(root_frame.signature.unwrap().as_str());
                let threads: Vec<Arc<str>> = vec![]; // 线程名集合（可扩展）
                let times: Vec<Arc<str>> = vec![];   // 时间点集合（可扩展）
                // 先分离借用，避免可变借用冲突
                match root.entry(method_name.clone()) { // 查找或插入根节点
                    Entry::Occupied(mut e) => {
                        Self::build_tree_from_frames(frames, Arc::make_mut(e.get_mut()));
                    }
                    Entry::Vacant(v) => {
                        let arc = Arc::new(CallTree {
                            method_name: method_name.clone(),
                            samples: 0,
                            threads: threads.clone(),
                            times: times.clone(),
                            next: None,
                        });
                        Self::build_tree_from_frames(frames, Arc::make_mut(v.insert(arc)));
                    }
                }
            }
        }
    }

    fn build_tree_from_frames(mut frames: Vec<CallFrame>, parent_node: &mut CallTree) {
        parent_node.samples += 1;
        if let Some(frame) = frames.pop() {
            if !matches!(frame.frame, Frame::MethodCall) {
               return;
            }
             let method_name = StringPool::get_arc_str(frame.signature.unwrap().as_str());
                let next_nodes = parent_node.next.get_or_insert_with(Vec::new);
                // 查找下一个节点是否已存在（同名方法）
                let mut found_idx = None;
                for (i, node) in next_nodes.iter().enumerate() {
                    // 如果已存在同名方法节点，记录下标
                    if node.method_name == method_name {
                        found_idx = Some(i);
                        break;
                    }
                }
                if let Some(idx) = found_idx {
                    // 已存在该方法节点，递归进入并累加采样
                    let mut_node = Arc::make_mut(&mut next_nodes[idx]);
                    Self::build_tree_from_frames(frames, mut_node);
                } else {
                    // 不存在则新建节点，采样数初始为0，递归会+1
                    let mut new_node = CallTree {
                        method_name: method_name.clone(), // 方法名
                        samples: 0, // 递归会+1
                        threads: vec![], // 可根据需要填充
                        times: vec![],   // 可根据需要填充
                        next: None,      // 子节点
                    };
                    // 递归构建子路径
                    Self::build_tree_from_frames(frames, &mut new_node);
                    // 新节点加入当前节点的子节点列表
                    next_nodes.push(Arc::new(new_node));
                }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::{
        fs,
        io::{self, BufRead},
    };
    use chrono::Local;
    use super::*;

    #[test]
    pub fn test() {
        let real_start = Local::now();
        let dirs = fs::read_dir("D:\\dump\\20241029\\all_threaddump").unwrap();
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
            let build_start = Local::now();
            println!("{}-开始构建：{}，共{}行", count_file, build_start.timestamp_millis(), lines.len());
            for group in lines {
                if !Thread::is_sys_thread(&group) {
                    let thread = Thread::new(&group,0,0);
                    count_threads += 1;
                    match thread {
                        Ok(t) => threads.push(t),
                        Err(e) => println!("{}", e),
                    }
                };
            }
            let build_end = Local::now();
            println!("{}-构建结束：{}", count_file, build_end.timestamp_millis() - build_start.timestamp_millis());
        }
        println!("调用树处理：共：{}",  count_threads);
        let build_start = Local::now();
        let _call_tree = CallTree::new(threads);
        let build_end = Local::now();
        println!("调用树构建完毕：{}, 构建树形耗时：{}， 共耗时：{}", 
        build_end.timestamp_millis(), 
        build_end.timestamp_millis() - build_start.timestamp_millis(),
        build_end.timestamp_millis()- real_start.timestamp_millis());
    }
}
