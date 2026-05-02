# ZeroResume - 上下文文档

> AI驱动的简历投递助手，专注国内主流招聘平台。
> 基于多个优秀开源项目二次开发。

## 产品定位

- **目标用户**：学生/应届生 + 社会招聘求职者
- **核心价值**：基于求职者信息 + 岗位JD，AI生成匹配简历，提升投递效率
- **收费模式**：完全开源免费
- **技术形态**：桌面端 (Tauri) + 浏览器插件 (Manifest V3)

## 二创来源

本项目基于以下开源项目二次开发：

| 项目 | 用途 | 协议 |
|------|------|------|
| [Auto-JobHunter](https://github.com/jolie-z/Auto-JobHunter) | 爬虫架构、AI评分逻辑、数据模型参考 | 未标注 |
| [LinkedIn-AI-Job-Applier-Ultimate](https://github.com/beatwad/LinkedIn-AI-Job-Applier-Ultimate) | 数据脱敏方案、LLM集成方式 | 未标注 |
| [QuickApply](https://github.com/AbbasZaidi11/QuickApply) | 浏览器扩展架构、表单字段检测 | 未标注 |
| [Reactive-Resume](https://github.com/amruthpillai/reactive-resume) | 简历编辑器UI参考、模板系统设计 | MIT |
| [ResumeLM](https://github.com/olyaiy/resume-lm) | 简历版本管理、评分算法参考 | AGPL-3.0 |

## 术语表

### 核心领域

| 术语 | 定义 |
|------|------|
| **求职者** | 使用产品的用户，提供个人信息和经历 |
| **基础简历** | 求职者录入的原始信息（基本信息、教育、工作、项目、自我评价等） |
| **岗位** | 从招聘平台爬取的职位信息，包含JD、公司、薪资等 |
| **JD** | Job Description，职位描述，包含岗位要求、职责、技能要求 |
| **匹配简历** | 针对特定岗位JD，由AI优化生成的定制化简历 |
| **评分/匹配度** | 系统对岗位与求职者匹配程度的量化评估（多维度） |
| **投递** | 将生成的简历附件提交到招聘平台的行为 |
| **投递历史** | 记录每份简历投给了哪个公司、哪个岗位、什么时间 |
| **自我评价** | 求职者对自己的综合评价，AI优化简历时的重要参考 |
| **证件照** | 求职者上传的个人照片，嵌入到生成的简历中 |

### 技术领域

| 术语 | 定义 |
|------|------|
| **桌面端** | Tauri构建的本地应用，负责数据存储、AI调用、PDF生成 |
| **浏览器插件** | Manifest V3扩展，负责读取招聘页面JD、跳转投递 |
| **解析器** | 针对特定招聘平台的页面解析模块，提取结构化岗位数据 |
| **解析规则** | 定义如何从招聘页面提取数据的配置（CSS选择器/XPath），支持热更新 |
| **热更新** | 从远程仓库拉取最新解析规则，无需发版即可适配平台改版 |
| **数据脱敏** | 将敏感信息（姓名、公司名等）替换为占位符后再发送给第三方AI |
| **简历模板** | 定义简历排版样式的文件，系统内置1-3套极简模板 |

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        ZeroResume                             │
├─────────────────────────────┬───────────────────────────────┤
│       桌面端 (Tauri)         │      浏览器插件 (Manifest V3)  │
│  ┌───────────────────────┐  │  ┌─────────────────────────┐  │
│  │  SQLite 本地数据库      │  │  │  招聘页面JD识别与提取      │  │
│  │  - 求职者信息            │  │  │  - BOSS直聘               │  │
│  │  - 岗位数据              │  │  │  - 智联招聘               │  │
│  │  - 简历文件              │  │  │  - 猎聘                   │  │
│  │  - 投递历史              │  │  │  - 前程无忧               │  │
│  └───────────────────────┘  │  │  - 拉勾网                 │  │
│  ┌───────────────────────┐  │  └─────────────────────────┘  │
│  │  AI引擎                 │  │  ┌─────────────────────────┐  │
│  │  - 本地模型 (Ollama)     │  │  │  跳转投递页面             │  │
│  │  - 第三方API (可选)      │  │  │  - 新标签页打开投递页      │  │
│  │  - 数据脱敏处理          │  │  │  - 用户手动上传附件简历    │  │
│  └───────────────────────┘  │  └─────────────────────────┘  │
│  ┌───────────────────────┐  │                               │
│  │  简历生成器              │  │                               │
│  │  - PDF生成              │  │                               │
│  │  - DOCX生成             │  │                               │
│  │  - 极简模板 (1-3套)      │  │                               │
│  └───────────────────────┘  │                               │
│  ┌───────────────────────┐  │                               │
│  │  岗位爬虫                │  │                               │
│  │  - Playwright无头浏览器  │  │                               │
│  │  - 定时自动爬取          │  │                               │
│  │  - 多城市/多关键词        │  │                               │
│  │  - 7天过期清理           │  │                               │
│  └───────────────────────┘  │                               │
│  ┌───────────────────────┐  │                               │
│  │  本地文件系统            │  │                               │
│  │  - 证件照存储            │  │                               │
│  │  - 生成的简历文件         │  │                               │
│  │  - AI模型文件 (可选)      │  │                               │
│  └───────────────────────┘  │                               │
└─────────────────────────────┴───────────────────────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │   GitHub 热更新源   │
                    │   (解析规则仓库)     │
                    └───────────────────┘
```

### 二创架构映射

| 模块 | 二创来源 | 改造方式 |
|------|----------|----------|
| 爬虫引擎 | Auto-JobHunter | Python→Rust迁移，适配Tauri命令 |
| AI评分/简历生成 | Auto-JobHunter ai_agents | 保留Prompt逻辑，改为本地LLM+第三方API混合 |
| 数据脱敏 | LinkedIn-AI-Job-Applier | 移植脱敏算法到Rust |
| 浏览器扩展 | QuickApply | MV3架构，增加国内平台JD识别 |
| 表单字段检测 | QuickApply fieldMap.js | 扩展支持中文招聘平台字段 |
| 简历编辑器 | Reactive-Resume + ResumeLM | 参考UI设计，自研实现 |
| 模板系统 | ResumeLM | 参考版本管理设计 |

## 数据模型

### 数据库表结构

#### job_seekers（求职者信息表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT | 姓名 |
| phone | TEXT | 电话 |
| email | TEXT | 邮箱 |
| job_intention | TEXT | 求职意向（JSON数组） |
| self_evaluation | TEXT | 自我评价 |
| custom_fields | TEXT | 自定义字段（JSON） |
| photo_path | TEXT | 证件照本地路径 |
| created_at | DATETIME | 创建时间 |
| updated_at | DATETIME | 更新时间 |

#### educations（教育经历表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| job_seeker_id | INTEGER FK | 关联求职者 |
| school | TEXT | 学校 |
| major | TEXT | 专业 |
| degree | TEXT | 学历 |
| start_date | DATE | 开始时间 |
| end_date | DATE | 结束时间 |

#### work_experiences（工作/实习经历表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| job_seeker_id | INTEGER FK | 关联求职者 |
| company | TEXT | 公司名 |
| position | TEXT | 职位 |
| start_date | DATE | 开始时间 |
| end_date | DATE | 结束时间 |
| description | TEXT | 职责描述 |

#### project_experiences（项目经历表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| job_seeker_id | INTEGER FK | 关联求职者 |
| project_name | TEXT | 项目名称 |
| tech_stack | TEXT | 技术栈 |
| role | TEXT | 职责 |
| achievements | TEXT | 成果 |

#### base_resumes（基础简历表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT | 简历名称（如"互联网版"） |
| job_seeker_id | INTEGER FK | 关联求职者 |
| created_at | DATETIME | 创建时间 |
| updated_at | DATETIME | 更新时间 |

#### targeted_resumes（匹配简历表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| base_resume_id | INTEGER FK | 关联基础简历 |
| job_id | INTEGER FK | 关联目标岗位 |
| optimized_content | TEXT | AI优化后的内容（Markdown） |
| version | INTEGER | 版本号 |
| file_path_pdf | TEXT | PDF文件路径 |
| file_path_docx | TEXT | DOCX文件路径 |
| generated_at | DATETIME | 生成时间 |

#### jobs（岗位表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| platform | TEXT | 平台来源 |
| job_title | TEXT | 岗位标题 |
| company_name | TEXT | 公司名称 |
| jd_text | TEXT | JD内容 |
| salary_range | TEXT | 薪资范围 |
| location | TEXT | 工作地点 |
| skills_required | TEXT | 技能要求 |
| experience_req | TEXT | 经验要求 |
| education_req | TEXT | 学历要求 |
| match_score | REAL | 匹配度总分 |
| score_details | TEXT | 各维度得分（JSON） |
| crawl_time | DATETIME | 爬取时间 |
| is_expired | BOOLEAN | 是否过期 |

#### delivery_records（投递记录表）
| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| targeted_resume_id | INTEGER FK | 关联匹配简历 |
| company_name | TEXT | 目标公司 |
| job_title | TEXT | 目标岗位 |
| delivered_at | DATETIME | 投递时间 |
| status | TEXT | 投递状态 |

### 领域模型关系

```
JobSeeker (1) ──► (N) Education
            ──► (N) WorkExperience
            ──► (N) ProjectExperience
            ──► (N) BaseResume ──► (N) TargetedResume ──► (N) DeliveryRecord
                                       │
                                       ▼
                                    Job (1)
```

## 关键决策

1. **混合模式爬取**：系统定时自动爬取岗位存入本地，用户在本地池子中筛选
2. **多维度评分**：技能、经验、学历、薪资、地域五个维度，用户可自定义权重
3. **半自动投递**：生成简历后跳转平台投递页，用户手动上传附件确认
4. **本地优先**：所有用户数据本地存储（SQLite），无需登录，无需上传服务器
5. **数据脱敏**：发送给第三方AI前，敏感信息替换为占位符
6. **多份简历**：支持多份基础简历，每份基础简历可为不同岗位生成定向版本
7. **版本控制**：保留历史版本，同时提供直接覆盖功能
8. **自我评价**：作为独立字段录入，AI优化时的重要参考
9. **证件照管理**：本地文件系统存储，数据库只保存文件路径

## 开源策略

- **公共仓库** (`zeroresume`)：核心代码，AGPL-3.0协议
- **私有仓库** (`zeroresume-pro`)：AI提示词模板、高级简历模板、品牌资源
- **发布包**：包含开源核心 + 闭源增值内容，用户免费使用

## 项目结构

```
zeroresume/
├── src/                          # Tauri 桌面端源码 (Rust)
│   ├── main.rs                   # 主入口
│   ├── commands/                 # Tauri Commands
│   │   ├── crawler.rs            # 爬虫命令
│   │   ├── resume.rs             # 简历生成命令
│   │   ├── database.rs           # 数据库操作
│   │   └── ai.rs                 # AI模型调用
│   ├── services/                 # 核心服务
│   │   ├── crawler/              # 爬虫服务
│   │   │   ├── boss.rs           # BOSS直聘 (基于Auto-JobHunter改造)
│   │   │   ├── zhaopin.rs        # 智联招聘
│   │   │   ├── liepin.rs         # 猎聘
│   │   │   └── _51job.rs         # 前程无忧
│   │   ├── ai/                   # AI服务
│   │   │   ├── scorer.rs         # 岗位评分 (移植ai_scorer.py)
│   │   │   ├── resume_generator.rs # 简历生成 (移植apply_assistant.py)
│   │   │   └── data_masking.rs   # 数据脱敏 (移植LinkedIn-Applier)
│   │   └── database.rs           # SQLite服务
│   └── utils/
│
├── src-ui/                       # Web前端 (React + TypeScript)
│   ├── components/
│   │   ├── resume-editor/        # 简历编辑器
│   │   ├── job-list/             # 岗位列表
│   │   ├── job-detail/           # 岗位详情+评分
│   │   └── settings/             # 设置面板
│   └── stores/
│
├── extension/                    # 浏览器扩展 (Manifest V3)
│   ├── manifest.json
│   ├── content/
│   │   ├── jd-extractor.js       # JD提取 (基于QuickApply改造)
│   │   ├── field-map.js          # 字段映射 (移植fieldMap.js)
│   │   └── platform-adapters/    # 平台适配器
│   │       ├── boss.js
│   │       ├── zhaopin.js
│   │       └── liepin.js
│   └── popup/
│
├── docs/
│   ├── CONTEXT.md
│   ├── PRD.md
│   └── adr/
│
└── scripts/
    └── setup-local-llm.sh        # 本地模型安装脚本
```
