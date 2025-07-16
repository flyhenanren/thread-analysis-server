use common::error::AnalysisError;
use db::db_access::db_cpu;
use domain::model::cpu::CpuCount;
use sqlx::SqlitePool;


/// 获取线程详情
pub async fn count_cpu_status(pool: &SqlitePool, work_space: &str) -> Result<CpuCount, AnalysisError> {
  match db_cpu::count_info(pool, &work_space).await{
      Ok(cpu_info) => {
          let exe_time = cpu_info.iter().map(|c| c.exe_time).collect();
          let us = cpu_info.iter().map(|c| c.us).collect();
          let sy = cpu_info.iter().map(|c| c.sy).collect();
          let ids = cpu_info.iter().map(|c| c.ids).collect();
          return Ok(CpuCount{
            exe_time,
            us,
            sy,
            ids,
        });
      },
      Err(err) => Err(AnalysisError::DBError(format!("查询CPU状态错误:{}", err))),
  }
}