# GS690 功能码调试终端 — Next.js 重构版

基于 Next.js 15 + React 19 + Tailwind CSS + Zustand 重构的 GS690 功能码调试终端。

## 技术栈

| 技术 | 用途 |
|------|------|
| Next.js 15 | React 框架（App Router） |
| React 19 | UI 库 |
| Tailwind CSS 3 | 样式框架 |
| Zustand 5 | 轻量状态管理 |
| Sonner | Toast 通知 |
| Lucide React | 图标库 |
| TypeScript 5 | 类型安全 |

## 项目结构

```
src/
├── app/                    # Next.js 页面路由
│   ├── layout.tsx          # 根布局
│   ├── page.tsx            # 主页面
│   └── globals.css         # 全局样式 + Tailwind
├── components/             # React 组件
│   ├── Header.tsx          # 标题栏
│   ├── FuncCodeTable.tsx   # 功能码表格 + 分组侧边栏
│   ├── RightPanel.tsx      # 右侧抽屉（日志/批量/设置）
│   ├── BottomPanel.tsx     # 底部面板（监视/收藏/历史/常用）
│   ├── ConnectDialog.tsx   # 串口连接对话框
│   ├── ContextMenu.tsx     # 右键菜单
│   ├── OptionPopover.tsx   # 选项弹出框
│   └── StatusBar.tsx       # 底部状态栏
├── store/
│   └── index.ts            # Zustand 全局状态
├── hooks/
│   └── useTheme.ts         # 主题管理 Hook
├── lib/
│   ├── types.ts            # TypeScript 类型定义
│   ├── constants.ts        # 协议常量
│   ├── utils.ts            # 工具函数
│   └── api.ts              # 后端 API 客户端
└── data/
    └── funcodes.json       # 默认功能码数据
```

## 快速开始

```bash
# 安装依赖
npm install

# 开发模式（端口 8899）
npm run dev

# 生产构建
npm run build
npm start
```

## API 代理

Next.js 开发服务器会将 `/api/*` 请求代理到 FastAPI 后端（默认 `localhost:8081`）。

修改 `next.config.ts` 中的 `rewrites` 配置来调整后端地址。

## 后端

FastAPI 后端（`server_serial.py`）保持不变，需要单独运行：

```bash
cd /path/to/gs690FunCode
uv run uvicorn server_serial:app --host 0.0.0.0 --port 8081 --reload
```
