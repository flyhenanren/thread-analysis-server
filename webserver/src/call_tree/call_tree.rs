use std::{collections::HashMap, usize};

use crate::models::thread::{CallFrame, Frame, Thread};

#[derive(Debug, Clone, PartialEq)]
pub struct CallTree {
    method_name: String,
    line: u32,
    time: u32,
    percent: f64,
    count: u32,
    next: Option<Vec<Box<CallTree>>>,
}

impl CallTree {
    pub fn new(threads: Vec<Thread>) -> CallTree {
        let mut root: HashMap<String, CallTree> = HashMap::new();
        let frames_count: usize = threads.iter().map(|t| t.frames.len()).sum();

        let percent = 100.0 / frames_count as f64; // 每个方法帧的占比

        for thread in threads {
            Self::convert_to_call_tree(thread, &mut root, percent);
            println!("完成一轮");
            for ele in &root {
                println!("{:?}", ele.1);
            }
        }

        root.into_iter().next().map_or_else(
            || CallTree {
                method_name: String::new(),
                line: 0,
                time: 0,
                percent: 0.0,
                count: 0,
                next: None,
            },
            |(_, tree)| tree,
        )
    }

    fn convert_to_call_tree(thread: Thread, root: &mut HashMap<String, CallTree>, percent: f64) {
        let mut path: Vec<CallFrame> = thread.frames;
        if path.len() == 0 {
            ()
        }
        if let Some(root_frame) = path.pop() {
            if let Frame::MethodCall = &root_frame.frame {
                let method_name = root_frame.method_name.clone().unwrap_or_default();
                let line = root_frame.line_number.unwrap_or_default();
                let root_node = root.entry(method_name.clone()).or_insert(CallTree {
                    method_name,
                    line,
                    time: 0,
                    percent: 0.0,
                    count: 0,
                    next: None,
                });
                Self::build_tree_from_frames(path, root_node, percent);
            }
        }
    }
    fn build_tree_from_frames(mut frames: Vec<CallFrame>, parent_node: &mut CallTree, percent: f64) {
        // Process each frame in the current level
        if let Some(frame) = frames.pop() {
            let method_name = frame.method_name.as_ref().unwrap_or(&String::new()).clone();

            // Check if the current frame exists in the parent node's next nodes
            let next_nodes = parent_node.next.get_or_insert_with(Vec::new);

            // Try to find the node in the current next nodes
            let existing_node = next_nodes
                .iter_mut()
                .find(|node| node.method_name == method_name);

            if let Some(node) = existing_node {
                // Update existing node
                node.count += 1;
                node.percent += percent;
                node.time += 15;

                Self::build_tree_from_frames(frames, node, percent);
            } else {
                // Create a new node if not found
                let mut new_node = CallTree {
                    method_name: method_name.clone(),
                    line: frame.line_number.unwrap_or_default(),
                    time: 15,
                    percent,
                    count: 1,
                    next: None,
                };

                // Recursively process the remaining frames for this new node
                Self::build_tree_from_frames(frames, &mut new_node, percent);

                // Add the new node to the next nodes of the parent
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
