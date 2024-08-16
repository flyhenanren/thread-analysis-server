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
            Self::convert_to_call_tree(thread, &mut root, percent)
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
        let root_frame = &thread.frames[0];
        // 找到根栈帧
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

            for frame in &thread.frames[1..] {
                Self::process_frame(frame, root_node, percent);
            }
        }
    }
    fn process_frame(frame: &CallFrame, current_node: &mut CallTree, percent: f64) {
        let method_name = frame.method_name.as_ref().unwrap_or(&String::new()).clone();

        // 当前节点如果没有下级，则插入一个空集合
        let next_nodes = current_node.next.get_or_insert_with(Vec::new);

        if let Some(next_node) = next_nodes
            .iter_mut()
            .find(|node| node.method_name == method_name)
        {
            // 加权计算
            next_node.count += 1;
            next_node.percent += percent;
            next_node.time += 15;
        } else {
            // 增加新节点
            let mut new_node = CallTree {
                method_name: method_name.clone(),
                line: frame.line_number.unwrap_or_default(),
                time: 15,
                percent,
                count: 1,
                next: None,
            };
            Self::process_frame(frame, &mut new_node, percent);
            next_nodes.push(Box::new(new_node));
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::{
        fs,
        io::{self, BufRead}, time::SystemTime,
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
            count_file +=1;
            let entry = dir.unwrap();
            let path = entry.path();
            println!("{}-读取文件：{:?}",count_file,path);
            let file = fs::File::open(path).unwrap();
            let reader = io::BufReader::new(file);
            let mut lines: Vec<Vec<String>> = Vec::new();
            let mut current_group: Vec<String> = Vec::new();
            let mut start = false;
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if l.is_empty() {
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
            println!("{}-开始构建：{}，共{}行",count_file, build, lines.len());
            for group in lines {
                let thread = Thread::new(group);
                count_threads +=1;
                match thread {
                    Ok(t) => threads.push(t),
                    Err(e) => println!("{}", e),
                }
            }
            let build = Local::now();
            println!("{}-构建结束：{}", count_file,build);
        }
        let build = Local::now();
        println!("调用树处理：{}，共：{}", build, count_threads);
        let call_tree = CallTree::new(threads);
        let build = Local::now();
        println!("调用树构建完毕：{}", build);
        println!("{:?}", call_tree)
    }
}
