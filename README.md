# Salary Garden / PVZ 工资花园

> 一个运行在 Windows 桌面的「实时工资可视化 + 极轻量 PVZ 游戏化」应用。  
> 当前开发基线：**Phase 5C 已完成**，下一阶段为 **Phase 6 — PVZ Character Animation & Scene Polish**。  
> 技术栈：**Tauri 2 + React + TypeScript + PixiJS + Rust + SQLite**

---

## 1. 项目是什么

Salary Garden 是一个本地优先、离线可用的桌面应用。

它把「上班过程中逐渐累积的工资」转换成一个 PVZ 风格的可视化庭院：

- Rust 后端根据当前时间、工资周期、实际工作日和固定工作时段，确定性计算“此刻理论上已经赚到的工资”；
- Reward Engine 根据有效工作秒生成银币、金币、钻石 entitlement；
- PixiJS 场景把这些 Reward 以金盏花掉落货币的方式表现出来；
- 用户可以手动点击工资货币；
- 也可以由固定磁吸菇在短暂等待后自动收集；
- 真正发生收集事务后，金额才进入“游戏钱包”；
- 应用关闭、休眠或未启动期间遗漏的收益不会丢失，而会在恢复时对账并包装成离线收益钱袋；
- “真实工资”与“游戏钱包”严格分离。

本项目**不是完整的《植物大战僵尸》复刻**。游戏部分只服务于工资可视化，不包含关卡、波次、胜负、复杂植物/僵尸系统。

---

## 2. 当前开发状态

| 阶段 | 状态 | 内容 |
|---|---|---|
| Phase 1 | ✅ | Payroll Engine / 工资核心 |
| Phase 1.5 | ✅ | Tauri + React + TypeScript + Vite Desktop Shell |
| Phase 2 | ✅ | Deterministic Reward Engine |
| Phase 3 | ✅ | SQLite / Ledger / Offline Reconciliation |
| Phase 3.5 | ✅ | Tauri Application API / Frontend Bridge |
| Phase 4A | ✅ | Salary Configuration & Calendar API |
| Phase 4B.1 | ✅ | React Product UI |
| Phase 4B.2 | ✅ | PVZ Visual Skin |
| Phase 5A | ✅ | PixiJS Core Scene & 3×7 Lawn |
| Phase 5B | ✅ | Live Salary Reward Materialization & Settlement |
| Phase 5C | ✅ | Reward Presentation / Magnet Auto Collection |
| Phase 6 | ⏭️ | Character Animation & Scene Polish |
| Phase 7A | ⏳ | Sun Economy / Sunflower / Planting |
| Phase 7B | ⏳ | Peashooter / Zombie Combat |
| Phase 8 | ⏳ | Windows Desktop Polish / Release |

当前验证基线：

- Rust tests：**64/64 passed**
- Frontend tests：**34/34 passed**
- `cargo fmt --check`：通过
- `cargo clippy --all-targets -- -D warnings`：通过
- `npm run build`：通过
- `git diff --check`：通过
- `npm run tauri dev`：真实 Windows Tauri 窗口运行通过

---

## 3. 已实现功能

### 3.1 Payroll Engine

已实现工资周期、工作日历、本地时区、每日有效工作秒、工作状态状态机、今日真实工资、当前周期真实工资、高精度 Decimal、工资周期快照、首次工资初始化和下周期工资修改。

固定计薪时段：

- 08:40–11:40
- 13:30–17:30
- 午休 11:40–13:30 不计薪
- 每个完整工作日共 7 小时 / 25,200 秒有效计薪时间

### 3.2 Reward Engine

工资货币：

- Silver
- Gold
- Diamond

事件按**有效工作秒**生成：

- 每 10 秒一个 reward boundary；
- 60 秒边界生成 Gold，覆盖 Silver；
- 3600 秒边界生成 Diamond，覆盖 Gold / Silver。

完整有效工作小时：

- Silver = 300
- Gold = 59
- Diamond = 1

完整 7 小时工作日：

- Silver = 2100
- Gold = 413
- Diamond = 7

币值权重：

```text
Silver : Gold : Diamond = 1 : 6 : 360
```

### 3.3 SQLite / Ledger / Offline

已实现：

- schema migrations；
- payroll cycle snapshot；
- daily reward state；
- accounted / collected 计数；
- collection ledger；
- offline reward bag；
- live reward event；
- 钱袋领取；
- 实时货币领取；
- 幂等保护；
- 重启恢复；
- 时间回拨保护；
- 跨工资周期隔离。

Offline reconciliation 必须是：

```text
reward entitlement counts - accounted reward counts
```

不能使用：

```text
real salary - game wallet
```

### 3.4 React UI

已实现：

- Dashboard；
- Calendar；
- Settings；
- 实时工资 / 游戏钱包切换；
- 今日金额；
- 当前工资周期金额；
- Pending rewards；
- Offline Reward Bag；
- Salary configuration；
- Next-cycle salary；
- Work status；
- Calendar month navigation；
- Loading / Error / Retry；
- PVZ 风格 UI Skin。

### 3.5 PixiJS

已实现：

- 3×7 草坪逻辑坐标；
- 左房屋 / 中草坪 / 右街道；
- Marigold；
- Magnet-shroom；
- Wall-nut；
- Bucket Zombie；
- Pixi lifecycle；
- display layers；
- Silver / Gold / Diamond live rewards；
- manual click；
- Magnet auto collection；
- 约 2.5 秒手动点击窗口；
- deterministic placement；
- reward visual state machine；
- lightweight sprite pool。

---

## 4. 尚未实现

- 完整角色组合动画；
- Marigold blink / petals / production reaction；
- Magnet-shroom 更完整的吸取反馈；
- Wall-nut bite reaction；
- Bucket Zombie bite animation；
- Zombie HP / death / respawn；
- Sun economy；
- Sunflower 产阳光；
- 用户种植；
- Peashooter shooting / collision；
- 游戏状态持久化；
- SFX 正式接入；
- 收起模式；
- always-on-top；
- autostart；
- Windows desktop polish；
- release packaging polish。

---

## 5. 核心工资规则

工资周期：

> 从“上个月最后一个工作日之后的第一个工作日”开始，到“本月最后一个工作日”结束。

当前 MVP：

- 周六、周日不是工作日；
- 法定节假日不是工作日；
- 不实现调休工作周末。

设：

```text
M = 月薪
N = 当前周期实际工作日数量
D = 日工资
H = 有效小时工资
R = 有效秒工资
```

则：

```text
D = M / N
H = D / 7
R = D / 25200
```

### 真实工资不是 timer 累加

禁止：

```text
setInterval(() => balance += R)
```

正确方式：

```text
当前时间
+ 工资周期
+ 工作日历
+ 有效工作时段
=> 当前理论真实工资
```

所以应用关闭、休眠、UI 掉帧都不会导致少算工资。

---

## 6. 三层账本

### Payroll Truth

回答：

> 此刻理论上已经赚到多少钱？

确定性、可重算。

### Reward Entitlement

回答：

> 此刻理论上应该产生多少 Silver / Gold / Diamond？

确定性、可重算。

### Collected Wallet

回答：

> 玩家真正领取了多少？

只有真实 collection transaction 才增加。

因此：

```text
Real Salary != Game Wallet
```

完全正常。

---

## 7. 系统架构

```text
React UI
  │
  │ Tauri invoke
  ▼
Tauri Application API
  │
  ├──────────────┐
  ▼              ▼
Payroll Domain   Reward Domain
  │              │
  └──────┬───────┘
         ▼
Services / Ledger / Reconciliation
         │
         ▼
       SQLite
```

游戏场景：

```text
React Dashboard
      │
      ▼
    PixiJS
      │
      ├─ Lawn
      ├─ Plants
      ├─ Zombie
      ├─ Rewards
      └─ Effects
```

职责：

- **Rust**：所有工资、Reward、Wallet、SQLite 权威逻辑；
- **React**：产品 UI 与 command 调用；
- **PixiJS**：场景、实体和视觉动画；
- **SQLite**：本地持久化与事务安全。

---

## 8. 关键数据流

### Real Salary

```text
Current Time
  ↓
WorkCalendar
  ↓
Effective Work Seconds
  ↓
Payroll Engine
  ↓
AppSnapshot
  ↓
React
```

### Live Reward

```text
Effective Work Seconds
  ↓
Reward Entitlement
  ↓
sync_live_rewards()
  ↓
SQLite live_reward_events
  ↓
PENDING
  ↓
Pixi LiveRewardEntity
  ↓
Manual Click / Magnet
  ↓
collect_live_reward(event_id)
  ↓
SQLite Transaction
  ↓
COLLECTED + collection_ledger
  ↓
AppSnapshot Refresh
  ↓
Game Wallet
```

### Offline Reward

```text
Downtime / sleep / restart
  ↓
Recalculate entitlement
  ↓
entitled - accounted
  ↓
Offline Reward Bag
  ↓
Manual Claim
  ↓
SQLite Transaction
  ↓
Game Wallet
```

Offline Bag 不允许被 Magnet 自动领取。

---

## 9. 项目目录

```text
desk_pet_salary/
├─ src/
│  ├─ app/
│  ├─ assets/
│  ├─ components/
│  ├─ features/
│  │  ├─ dashboard/
│  │  ├─ wallet/
│  │  ├─ calendar/
│  │  ├─ settings/
│  │  ├─ offline-bag/
│  │  └─ game/
│  │     ├─ PixiGameScene.tsx
│  │     └─ pixi/
│  │        ├─ assets/
│  │        ├─ entities/
│  │        ├─ layout/
│  │        └─ scene/
│  ├─ shared/
│  └─ styles/
│
├─ src-tauri/
│  ├─ migrations/
│  │  ├─ 0001_phase3_persistence.sql
│  │  └─ 0002_phase5b_live_rewards.sql
│  ├─ src/
│  │  ├─ application/
│  │  ├─ commands/
│  │  ├─ domain/
│  │  ├─ persistence/
│  │  ├─ services/
│  │  └─ main.rs
│  ├─ tests/
│  ├─ Cargo.toml
│  └─ tauri.conf.json
│
├─ resources/
├─ package.json
├─ vite.config.ts
├─ README.md
└─ Salary_Garden_PVZ_Product_Design_Spec_v1.1.md
```

---

## 10. 本地开发环境

主要目标平台：Windows。

需要：

- Node.js / npm
- Rust / rustup / Cargo
- Microsoft C++ Build Tools
- Windows SDK
- WebView2 Runtime
- Git

如果 Rust 报：

```text
link.exe not found
```

通常是 MSVC C++ Build Tools 环境不完整。

---

## 11. 如何运行

```powershell
git clone <your-repository-url>
cd desk_pet_salary
npm install
npm run tauri dev
```

正常流程：

```text
Vite
  ↓
Cargo / Rust
  ↓
Tauri
  ↓
Windows desktop window
```

`vite.config.ts` 已配置忽略 `src-tauri/**`，避免 Windows 上 Vite watch Rust `target/*.exe` 导致 `EBUSY`。

---

## 12. 如何测试

Frontend：

```powershell
npm test
npm run build
```

当前：

```text
34/34 tests passed
```

Rust：

```powershell
cd src-tauri
cargo test --all-targets
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

当前：

```text
64/64 tests passed
```

每个 Phase 完成后建议执行：

```text
npm test
npm run build
cargo test --all-targets
cargo fmt --check
cargo clippy --all-targets -- -D warnings
git diff --check
npm run tauri dev
```

---

## 13. SQLite

主要表：

```text
schema_migrations
payroll_cycles
daily_reward_state
offline_reward_bags
offline_reward_bag_items
collection_ledger
app_settings
live_reward_events
```

### live_reward_events

业务唯一键：

```text
cycle_id
+ work_date
+ event_index
```

状态：

```text
PENDING
COLLECTED
PACKAGED
```

它用于给“已经物化的可领取工资币”一个稳定身份。

它**不是 Reward Entitlement 真值来源**；Entitlement 始终由有效工作秒重算。

---

## 14. Tauri Application API

当前主要 commands：

```text
get_app_snapshot
get_app_settings
update_app_settings

list_offline_reward_bags
claim_offline_reward_bag

get_salary_configuration
initialize_salary
update_next_cycle_salary
get_calendar_month

sync_live_rewards
list_pending_live_rewards
collect_live_reward
```

前端禁止直接访问 SQLite，也禁止复制工资/Reward 算法。

---

## 15. PixiJS 与 Live Reward

Lawn：

```text
3 rows × 7 columns
```

中路固定：

```text
Column 0 → Marigold
Column 1 → Magnet-shroom
Column 3 → Wall-nut
Right road → Bucket Zombie
```

Display layers：

```text
backgroundLayer
gridLayer
plantLayer
zombieLayer
projectileLayer
rewardLayer
effectLayer
debugLayer
```

Reward visual state：

```text
SPAWNING
  ↓
IDLE
  ↓
MAGNETIZING
  ↓
SETTLING
  ↓
REMOVED
```

Manual click 和 Magnet **必须共用同一个 Rust settlement**：

```text
collect_live_reward(event_id)
```

禁止：

```text
wallet += coinValue
```

---

## 16. PVZ 资源说明

项目使用用户本地准备的 PVZ 资源，通过集中式 manifest 映射。

原则：

```text
semantic asset key
  ↓
asset manifest
  ↓
actual file
```

不要在业务代码中散落真实资源路径。

### 授权注意

PVZ 原始美术和音效的版权属于其权利人。

当前工程把这些资源视为：

> 用户本地提供、可替换的外部资产层。

如果仓库准备公开：

- 不要默认公开分发原始 PVZ 资源；
- 不要声明这些资源拥有开放许可证；
- 根据你的实际授权情况决定是否把相关资源加入 `.gitignore`；
- 未来公开发行建议替换为原创或明确授权素材。

---

## 17. 不可破坏的不变量

1. `CycleEarned(end) = MonthlySalary`
2. 完整小时 = `300 Silver + 59 Gold + 1 Diamond`
3. `CoinEntitlementValue(1h) = HourlyPay`
4. `CoinEntitlementValue(7h) = DailyPay`
5. `CoinEntitlementValue(cycle) = MonthlySalary`
6. 真实工资不依赖应用持续运行
7. 游戏钱包只因 collection transaction 增长
8. Offline reconciliation 基于 entitlement count - accounted count
9. 任意 Live Reward / Offline Bag 最多结算一次
10. 午休、周末、节假日不产生工资或新 Reward
11. 阳光、植物、僵尸不得改变工资
12. Wall-nut 永远不会死亡

最重要的边界：

```text
Rust = business truth
React = UI
PixiJS = visual world
SQLite = persistence
```

---

## 18. 后续路线

### Phase 6 — Character Animation & Scene Polish

下一阶段：

- Marigold component animation；
- blink / petals / production reaction；
- Magnet-shroom idle / attracting；
- Wall-nut idle / bite reaction；
- Bucket Zombie walk / bite；
- anchor / scale / shadow；
- depth polish；
- animation lifecycle；
- **不修改 Phase 5B / 5C settlement 语义**。

### Phase 7A — Sun Economy + Planting

- Sun；
- Sunflower production；
- plant cost；
- Sunflower / Peashooter planting；
- plant layout persistence。

工资与 Sun 必须完全隔离。

### Phase 7B — Peashooter + Zombie Combat

- pea shooting；
- projectile collision；
- Zombie HP；
- hit / death；
- ~2 秒 respawn；
- Wall-nut infinite HP。

Zombie kill 不奖励工资。

### Phase 8 — Desktop Polish

- collapsed mode；
- always on top；
- autostart；
- window restore；
- performance；
- audio；
- packaging / release。

---

## 19. 新开发者接手流程

建议顺序：

1. 阅读本 README；
2. 阅读 `Salary_Garden_PVZ_Product_Design_Spec_v1.1.md`；
3. `npm install`；
4. 运行全部 frontend / Rust tests；
5. `npm run tauri dev`；
6. 验证 Dashboard / Calendar / Settings / Live Reward / Magnet / Offline Bag；
7. 从 Phase 6 开始继续；
8. 每个阶段完成后先测试和人工验收，再 commit/tag。

不要：

- 用 Pixi ticker 产生工资；
- 用 JS `number` 长期累计账本金额；
- 绕过 Rust transaction 修改钱包；
- 用 Real Salary - Game Wallet 计算离线收益；
- 为了动画修改已经冻结的财务规则。

---

## 20. Git 工作流

推荐：

```text
develop one phase
  ↓
automated tests
  ↓
real Windows validation
  ↓
git commit
  ↓
git tag
  ↓
next phase
```

阶段 tag 建议：

```text
phase-2-complete
phase-3-complete
phase-3.5-complete
phase-4a-complete
phase-4b1-complete
phase-4b2-complete
phase-5a-complete
phase-5b-complete
phase-5c-complete
```

---

## 21. 相关文档

详细产品/技术 SSOT：

```text
Salary_Garden_PVZ_Product_Design_Spec_v1.1.md
```

README 用于：

- 快速理解项目；
- 搭建环境；
- 了解当前状态；
- 接手开发。

SSOT 用于：

- 冻结产品规则；
- 保存公式；
- 保存账本语义；
- 保存验收条件；
- 保存阶段边界。

**推荐同时保留 README 和 SSOT，不建议用 README 替代 SSOT。**

---

> 工资账本必须严谨，货币账本必须可重算，离线补偿必须幂等，游戏玩法必须克制，视觉可以丰富但不能反过来绑架财务逻辑。
