# ADR-0004: 基于开源项目二次开发

## 状态

已接受

## 背景

在开发ZeroResume的过程中，我们发现已有多个优秀的开源项目实现了部分我们想要的功能：
- **Auto-JobHunter**：完整的国内招聘平台爬虫 + AI评分系统
- **LinkedIn-AI-Job-Applier-Ultimate**：成熟的数据脱敏 + LLM集成方案
- **QuickApply**：极简的浏览器扩展自动填充架构
- **Reactive-Resume**：优秀的简历编辑器UI设计
- **ResumeLM**：完善的简历版本管理系统

完全从零开发会：
1. 重复造轮子，浪费开发时间
2. 错过这些项目已经验证过的最佳实践
3. 无法利用社区维护的解析规则（招聘网站经常改版）

## 决策

采用**基于多个开源项目二次开发**的策略，而非从零构建。

### 二创来源与用途

| 项目 | 协议 | 用途 | 改造方式 |
|------|------|------|----------|
| [Auto-JobHunter](https://github.com/jolie-z/Auto-JobHunter) | 未标注 | 爬虫架构、AI评分逻辑、数据模型 | Python→Rust迁移，适配Tauri |
| [LinkedIn-AI-Job-Applier](https://github.com/beatwad/LinkedIn-AI-Job-Applier-Ultimate) | 未标注 | 数据脱敏方案、LLM集成 | 移植脱敏算法到Rust |
| [QuickApply](https://github.com/AbbasZaidi11/QuickApply) | 未标注 | 浏览器扩展架构、表单检测 | 增加国内平台适配 |
| [Reactive-Resume](https://github.com/amruthpillai/reactive-resume) | MIT | 简历编辑器UI参考 | 参考设计，自研实现 |
| [ResumeLM](https://github.com/olyaiy/resume-lm) | AGPL-3.0 | 简历版本管理、评分算法 | 参考设计，兼容协议 |

### 改造原则

1. **核心逻辑保留**：爬虫反风控、AI Prompt工程、数据脱敏等已验证的逻辑直接参考
2. **语言迁移**：Python后端逻辑统一迁移到Rust，适配Tauri架构
3. **协议兼容**：确保二创后的AGPL-3.0与来源协议兼容
4. **标注来源**：所有引用代码文件头部标注原始项目和作者
5. **社区回馈**：改进的解析规则等贡献回原始社区

### 代码标注规范

```rust
// Based on Auto-JobHunter by jolie-z
// Original: https://github.com/jolie-z/Auto-JobHunter
// Modified for ZeroResume: Migrated from Python to Rust, adapted for Tauri
```

## 备选方案

### 方案A：完全从零开发
- **拒绝原因**：开发周期长（预估增加2-3个月）；重复解决已解决的问题（反风控、Prompt工程）；无法利用社区维护的解析规则

### 方案B：直接Fork单项目修改
- **拒绝原因**：没有单一项目覆盖全部需求；各项目技术栈不同（Python/Django/Next.js）；需要统一为Tauri+Rust架构

### 方案C：仅参考思路，不引用代码
- **拒绝原因**：Prompt模板、解析规则等无法通过"思路"复制；数据脱敏算法需要精确实现

## 后果

### 正面
- 开发周期缩短50%以上
- 继承已验证的反风控策略和解析规则
- 可以站在巨人肩膀上优化，而非重复踩坑
- 社区解析规则可持续更新

### 负面
- 需要处理协议兼容问题（部分项目未标注协议）
- 代码风格不统一，需要重构适配
- 依赖项目的更新节奏（如解析规则更新）
- 需要仔细标注来源，避免合规风险

## 相关决策

- ADR-0001: 桌面端 + 浏览器插件混合架构
- ADR-0002: 开源策略（AGPL-3.0 + 闭源增值）
- ADR-0003: AI模型策略（本地 + 第三方混合）
