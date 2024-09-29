-- 创建线程文件相关脚本
CREATE TABLE IF NOT EXISTS THREAD_INFO (
  ID TEXT PRIMARY KEY,
  WORKSPACE TEXT,
  FILE_ID INTEGER,
  THREAD_ID TEXT,
  THREAD_NAME TEXT,
  DAEMON INTEGER,
  THREAD_STATUS INTEGER,
  START_LINE INTEGER,
  END_LINE INTEGER
);


CREATE TABLE IF NOT EXISTS THREAD_STACK (
  ID TEXT PRIMARY KEY,
  WORKSPACE TEXT,
  THREAD_ID INTEGER,
  CLASS_NAME TEXT,
  METHOD_NAME TEXT,
  STACK_LINE INTEGER,
  STACK_STATUS INTEGER
);
