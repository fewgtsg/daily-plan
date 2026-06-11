# Daily Plan v0.2.0 设计文档

## 1. 概述

### 1.1 版本目标
Daily Plan v0.2.0 在 v0.1.0 的自由填写日记 + 四象限看板基础上，重点增强**搜索回顾能力**与**交互体验**，让用户能更自然地连接日记与任务、更快地找到历史记录，同时让界面使用更顺手。

### 1.2 核心需求来源
- 改进方向：搜索与回顾（B）+ 交互与体验（E）
- 选中方案：方案 2（平衡改进）
- 不做 Markdown 增强，保持纯文本记录体验

### 1.3 功能清单
| 模块 | v0.2.0 功能 |
|------|------------|
| 标签体系 | 日记与任务通用标签；正文 `#标签` 自动识别；独立标签管理面板；按标签搜索 |
| 双向链接 | 日记中 `[[任务标题]]` 链接到任务；输入 `[[` 自动补全任务；点击跳转并高亮 |
| 布局自定义 | 侧边栏可折叠/固定；窗口与布局状态记忆 |
| 动画过渡 | 页面切换淡入滑动；看板拖拽增强反馈 |

### 1.4 明确不做
- Markdown 语法增强
- 日期链接 `[[2026-06-01]]` 或自定义主题页
- 任务反向引用列表（任务页显示哪些日记提到它）
- 标签使用统计图表
- 云同步、移动端、Web 版

## 2. 数据模型

### 2.1 新增表

#### `tags`
```sql
CREATE TABLE tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,       -- 小写统一存储
  display_name TEXT,               -- 保留原始大小写（可选）
  created_at TEXT NOT NULL         -- ISO 8601
);
```

#### `entry_tags`
日记与标签多对多关系。
```sql
CREATE TABLE entry_tags (
  entry_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  PRIMARY KEY (entry_id, tag_id),
  FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

#### `task_tags`
任务与标签多对多关系。
```sql
CREATE TABLE task_tags (
  task_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  PRIMARY KEY (task_id, tag_id),
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

#### `entry_task_links`
日记中的任务链接。
```sql
CREATE TABLE entry_task_links (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  entry_id INTEGER NOT NULL,
  task_id INTEGER,                 -- 可为 NULL，表示失效链接
  raw_text TEXT NOT NULL,          -- [[...]] 中的原始文本
  position INTEGER,                -- 在正文中的字符位置，预留
  FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE SET NULL
);
```

#### `app_settings`
应用级用户偏好。
```sql
CREATE TABLE app_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

### 2.2 数据变更触发器
- 日记保存时：重新解析 `#标签` 和 `[[...]]`，先清空旧关联再写入新关联。
- 任务保存时：重新解析 `#标签`，先清空旧关联再写入新关联。
- 任务删除时：`entry_task_links.task_id` 置 NULL，链接进入失效状态。

## 3. 标签体系

### 3.1 标签提取规则
- 匹配模式：`#` 后紧跟至少一个合法字符。
- 合法字符：中文、英文、数字、下划线 `_`、连字符 `-`。
- 结束条件：遇到空格、换行、标点符号（逗号、句号、冒号等）。
- 不合法：`#123`（纯数字）、`#` 单独存在、标签名超过 50 字符。
- 大小写：存储时统一转小写；界面显示保留首次出现的原始形式。

### 3.2 标签管理面板
- 位置：日记页右侧可折叠面板。
- 内容：
  - 当前日记已识别标签
  - 全局标签列表，按使用次数降序
  - 点击标签跳转搜索页 `/search?tag=xxx`
- 操作：支持手动添加/删除当前日记的标签（不修改正文）。

### 3.3 看板标签
- 任务卡片下方显示标签胶囊。
- 看板顶部增加标签筛选输入框，输入时自动补全已有标签。
- 筛选后隐藏不匹配任务，象限标题保持显示。

### 3.4 搜索结果页
- 路由：`/search?tag=工作&tag=灵感`
- 布局：左右两栏，左侧日记列表，右侧任务列表。
- 多标签筛选：日记/任务包含任一选中标签即匹配（OR 逻辑）。

## 4. 双向链接（任务链接）

### 4.1 语法
- 日记正文中使用 `[[任务标题]]` 创建链接。
- 标题内允许空格、中文、常见符号。
- 不支持嵌套 `[[...]]`。

### 4.2 保存时解析
1. 正则提取所有 `[[...]]` 片段。
2. 对每个片段，在现有任务中按标题精确匹配（去除前后空格）。
3. 完全匹配：绑定 `task_id`；无匹配：标记为失效链接（`task_id = NULL`）。
4. 写入 `entry_task_links` 表。

### 4.3 自动补全
- 触发：编辑器中输入 `[[`。
- 浮层位置：光标上方。
- 默认列表：最近编辑的 5 个任务。
- 过滤：继续输入时按任务标题实时过滤。
- 选择：↑/↓ 移动，Enter/Tab 插入。
- 插入后格式：`[[任务标题]]`。

### 4.4 渲染与交互
- 已绑定任务：蓝色药丸样式，点击跳转看板并高亮目标任务 2 秒。
- 失效链接：灰色删除线样式，点击无反应，hover 显示"未找到任务"。
- 任务标题变更不影响链接（通过 `task_id` 维护）。

## 5. 布局自定义

### 5.1 侧边栏
- 两种状态：展开（图标 + 文字）、折叠（仅图标）。
- 切换按钮位于侧边栏底部。
- 折叠后主内容区自动扩展。

### 5.2 状态记忆
- 保存项：
  - `sidebar_collapsed`: true/false
  - `window_width`, `window_height`
- 保存时机：窗口尺寸变化 debounce 500ms；侧边栏切换时立即保存。
- 恢复时机：应用启动时读取并应用。

### 5.3 持久化
- 通过 Tauri 命令读写 SQLite `app_settings` 表。
- 窗口尺寸恢复依赖 Tauri Window API。

## 6. 动画与过渡

### 6.1 页面切换
- 库：`framer-motion`
- 效果：淡入 + 水平滑入
- 参数：
  - 初始：`opacity: 0, x: 12`
  - 进入：`opacity: 1, x: 0`
  - 退出：`opacity: 0, x: -12`
  - 时长：200ms
  - 缓动：`ease-out`

### 6.2 看板拖拽增强
- 拖拽时卡片：`scale: 1.02`，阴影加深。
- 目标象限：背景色淡蓝高亮。
- 放置后：150ms 落位动画。
- 拖动副本：使用 `@dnd-kit` 的 `DragOverlay`。

### 6.3 新增依赖
- `framer-motion`：页面与组件动画。

## 7. 前端组件规划

| 组件 | 职责 |
|------|------|
| `TagManager` | 标签面板、标签列表、手动增删 |
| `TagInput` | 标签自动补全输入框 |
| `TaskLinkAutocomplete` | 任务链接补全浮层 |
| `LinkedTaskRenderer` | 渲染日记中的任务链接 |
| `SidebarLayout` | 侧边栏折叠与布局状态 |
| `AnimatedPage` | 页面切换动画包装 |
| `SearchPage` | 标签搜索结果页 |

## 8. 后端命令规划

| 命令 | 说明 |
|------|------|
| `extract_tags` | 解析文本中的标签 |
| `get_all_tags` | 获取全部标签及使用次数 |
| `get_entry_tags` | 获取某篇日记的标签 |
| `get_task_tags` | 获取某个任务的标签 |
| `search_by_tags` | 按标签搜索日记和任务 |
| `extract_task_links` | 解析日记中的任务链接 |
| `get_entry_task_links` | 获取日记的任务链接 |
| `search_tasks` | 任务标题搜索（用于补全） |
| `get_app_setting` / `set_app_setting` | 应用设置读写 |

## 9. 错误处理

- 标签解析失败：记录日志，不影响保存。
- 任务链接补全失败：允许用户手动输入。
- 标签面板加载失败：显示重试按钮。
- 所有后端错误统一走现有 Toast 通知机制。

## 10. 测试策略

### 10.1 后端单元测试
- 标签提取：正常标签、带标点、纯数字、空内容等边界。
- 任务链接解析：完全匹配、无匹配、多个链接、标题含空格。
- 设置读写：CRUD 与默认值。

### 10.2 前端组件测试
- `TagManager`：标签渲染、点击、手动添加。
- `TaskLinkAutocomplete`：触发、过滤、选择。
- `SidebarLayout`：折叠状态切换。

### 10.3 手动测试
- 日记保存后标签是否正确同步。
- 任务链接点击是否跳转并高亮。
- 标签筛选是否同时作用于日记和任务。
- 侧边栏折叠与窗口尺寸重启后是否记忆。

## 11. 预估工期

| 阶段 | 天数 |
|------|------|
| 数据模型 + 后端命令 | 2 |
| 标签体系前端 | 2 |
| 任务链接补全 | 1.5 |
| 布局与动画 | 1.5 |
| 测试与打磨 | 1 |
| **总计** | **约 8 天** |

按每天可投入 3-4 小时估算，约 **1.5-2 周**。

## 12. 风险与依赖

- `framer-motion` 与 React 19 兼容性：需在安装后验证。
- TipTap 编辑器中插入 `[[` 补全浮层的定位准确性。
- 任务标题变更后旧链接的显示更新策略。
