
#[derive(Debug, Clone)]
pub struct fileInfo{

  pub path: String,
  pub file_type: FileType,
  pub time: String

}

#[derive(Debug, Clone)]
pub enum FileType{
  CpuThread,
  CpuTop,
  StackTrace,
  Gc
}

