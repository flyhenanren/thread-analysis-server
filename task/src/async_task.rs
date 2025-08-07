use domain::context::Context;
use serde::Serialize;
use sqlx::SqlitePool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};

#[derive(Clone)]
pub struct ExecuteContext {
    pub pool: Option<SqlitePool>,
    pub channel: mpsc::Sender<(Option<f64>, Option<String>, Option<TaskPhase>, Option<String>)>,
    pub param: Option<String>
}

impl ExecuteContext {
    /// 安全更新进度的方法（自动处理错误和范围校验）
    pub async fn update_progress(&self, value: f64, message: Option<String>) {
        // 校验进度值有效性
        if !(0.0..=100.0).contains(&value) {
            log::error!("非法进度之:{}", value);
            let clamped = value.clamp(0.0, 100.0);
            let _ = self.channel.send((Some(clamped), message, Some(TaskPhase::Running), None)).await;
            return;
        }
        log::info!("更新进度,progress:{}, value:{:?}", value, message);
        // 发送进度更新
        let _ = self.channel.send((Some(value), message, None, None)).await;
        
    }

    // 带返回值的严格版（按需使用）
    pub async fn try_update_progress(
        &self,
        value: f64,
        message: Option<String>,
    ) -> Result<(), String> {
        if !(0.0..=100.0).contains(&value) {
            return Err(format!("非法进度值: {}", value));
        }
        self.channel
            .send((Some(value), message, Some(TaskPhase::Running), None))
            .await
            .map_err(|e| format!("进度更新失败: {}", e))
    }

    pub async fn fail(&self, message: Option<String>) {
        let _ = self.channel
            .send((None, message, Some(TaskPhase::Failed), None))
            .await
            .map_err(|e| format!("设置失败状态错误:{}", e));
    }

    pub async fn complate(&self, message: Option<String>, result: Option<String>){
      let _ = self.channel
          .send((Some(100.0), message, Some(TaskPhase::Completed), result))
          .await
          .map_err(|e| format!("设置完成状态错误:{}", e));
  }
}

#[derive(Clone, PartialEq, Default, Serialize)]
pub enum TaskPhase {
    #[default]
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Default, Serialize)]
pub struct TaskStatus {
    pub progress: f64,
    pub message: Option<String>,
    pub phase: TaskPhase,
    pub result: Option<String>
}

pub struct TaskHandle {
    pub status: Arc<Mutex<TaskStatus>>,
}

#[async_trait::async_trait]
pub trait AsyncTask {
    async fn execute(&self, context: &ExecuteContext) -> Result<String, String>;
}
#[derive(Clone)]
pub struct TaskExecutor {
    tasks: Arc<Mutex<HashMap<String, TaskHandle>>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        TaskExecutor {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn submit_task<T: AsyncTask + Send + 'static>(
        &self,
        task_id: &str,
        task: T,
        context: &Context,
        param: Option<String>
    ) {
        // 创建进度更新通道
        let (progress_tx, mut progress_rx): (
            mpsc::Sender<(Option<f64>, Option<String>, Option<TaskPhase>, Option<String>)>,
            mpsc::Receiver<(Option<f64>, Option<String>, Option<TaskPhase>, Option<String>)>,
        ) = mpsc::channel(10);

        // 创建共享状态
        let status = Arc::new(Mutex::new(TaskStatus::default()));

        let execute_id = task_id.to_string().clone();
        // Insert initial status
        {
            let mut tasks = self.tasks.lock().await;
            tasks.insert(
                execute_id.clone(),
                TaskHandle {
                    status: status.clone(),
                },
            );
        }

        // 启动状态监听任务
        let status_clone = status.clone();

        tokio::spawn(async move {
            while let Some((progress, message, phase,result)) = progress_rx.recv().await {
                let mut status = status_clone.lock().await;
                if let Some(progress) = progress {
                    status.progress = progress;
                }
                if let Some(message) = message {
                    status.message = Some(message)
                }
                if let Some(phase) = phase {
                    status.phase = phase;
                }
                if let Some(result) = result {
                    status.result = Some(result);
                }
            }
        });
        // 启动任务执行
        let execute_context = ExecuteContext {
            pool:Some(context.pool.clone()),
            channel: progress_tx,
            param
        };
        tokio::spawn(async move {
          match task.execute(&execute_context).await{
            Ok(value) => execute_context.complate( None, Some(value.clone())).await,
            Err(err) => execute_context.fail(Some(err)).await,
          }
        });
    }

    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let tasks = self.tasks.lock().await;
        if let Some(handle) = tasks.get(task_id) {
            let status = handle.status.lock().await;
            Some(TaskStatus {
                progress: status.progress,
                message: status.message.clone(),
                phase: status.phase.clone(),
                result: status.result.clone()
            })
        } else {
            None
        }
    }

    pub async fn remove_task(&self, task_id: &str) -> Option<TaskHandle> {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(task_id)
    }
}
