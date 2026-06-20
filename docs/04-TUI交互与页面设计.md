# 04-TUI交互与页面设计

## 1. 设计目标

TUI 应在终端窗口中提供接近桌面应用的可视化体验，让用户不必反复输入完整命令，即可完成音效管理、设备配置、快捷键绑定和播放控制。

设计原则：

- 状态常驻显示；
- 操作以快捷键为主；
- 常用功能一步触达；
- 弹窗完成复杂输入；
- 音频运行状态实时反馈；
- 界面和音频线程解耦。

## 2. 主界面布局

推荐使用四区域布局：

```text
┌──────────────────────────────────────────────────────────────────────┐
│ GameSound TUI                        ● Running  Mic OK  Hotkeys ON   │
├───────────────────┬──────────────────────────────────────────────────┤
│ Categories         │ Sounds                                           │
│ > 全部             │ ID  Name           Hotkey       Vol   Status     │
│   搞笑             │ 1   哈哈哈         Ctrl+1       80%   Ready      │
│   语音包           │ 2   完了芭比Q      Ctrl+2       90%   Ready      │
│   BGM              │ 3   登场BGM        Ctrl+3       60%   Playing    │
│   收藏             │ 4   危险警告       Ctrl+4       70%   Ready      │
├───────────────────┴──────────────────────────────────────────────────┤
│ Mic: USB Microphone        Output: BlackHole 2ch                      │
│ Monitor: Headphones        Profile: 默认方案                           │
│ Mic ████████░░ 80%         SFX ██████░░░░ 60%  Monitor █████░░░░░ 50% │
├──────────────────────────────────────────────────────────────────────┤
│ A Add  Enter Play  B Bind  E Edit  D Delete  S StopAll  C Config  Q Quit │
└──────────────────────────────────────────────────────────────────────┘
```

## 3. 顶部状态栏

顶部状态栏显示：

- 程序名称；
- 运行状态；
- 麦克风状态；
- 输出设备状态；
- 快捷键状态；
- 当前音效方案；
- 错误告警。

状态示例：

| 状态 | 含义 |
|---|---|
| Running | 音频引擎正在运行 |
| Mic OK | 麦克风采集正常 |
| Output OK | 虚拟输出设备正常 |
| Hotkeys ON | 全局快捷键已启用 |
| Warning | 存在设备或配置问题 |

## 4. 分类栏

左侧分类栏展示：

- 全部；
- 收藏；
- 最近使用；
- 用户自定义分类；
- 当前方案下的分类。

操作：

- 上下键移动；
- Tab 切换焦点；
- Enter 选择分类；
- N 新建分类；
- R 重命名分类；
- D 删除分类。

## 5. 音效列表

音效列表字段：

| 字段 | 说明 |
|---|---|
| ID | 音效编号 |
| Name | 音效名称 |
| Hotkey | 全局快捷键 |
| Vol | 单音效音量 |
| Mode | 播放模式 |
| Status | Ready / Playing / Missing / Error |

操作：

- 上下键选择音效；
- Enter 播放；
- Space 停止；
- A 添加音效；
- E 编辑音效；
- D 删除音效；
- B 绑定快捷键；
- + 增加音效音量；
- - 降低音效音量；
- / 搜索。

## 6. 设备与音量区域

底部设备区域显示：

- 当前真实麦克风输入；
- 当前虚拟输出设备；
- 当前本地监听设备；
- 当前音效方案；
- 麦克风音量；
- 音效总音量；
- 监听音量；
- 电平条。

电平条建议每 50ms 至 100ms 刷新一次，避免过高刷新影响 CPU。

## 7. 快捷键提示栏

底部提示栏始终显示当前页面可用快捷键。

主界面提示：

```text
A Add  Enter Play  B Bind  E Edit  D Delete  S StopAll  / Search  C Config  ? Help  Q Quit
```

配置页面提示：

```text
M Mic  O Output  L Monitor  H Hotkeys  P Profile  Esc Back  S Save
```

## 8. 弹窗设计

### 8.1 添加音效弹窗

```text
┌──────────── Add Sound ────────────┐
│ File Path:                         │
│ Name:                              │
│ Category:                          │
│ Hotkey:                            │
│ Volume: 80                         │
│ Mode: Once                         │
│                                    │
│ Enter Save      Esc Cancel         │
└────────────────────────────────────┘
```

### 8.2 编辑音效弹窗

字段：

- 名称；
- 分类；
- 标签；
- 音量；
- 播放模式；
- 是否循环；
- 备注。

### 8.3 快捷键绑定弹窗

```text
┌──────────── Bind Hotkey ────────────┐
│ Sound: 哈哈哈                        │
│                                      │
│ Press new hotkey...                  │
│ Current: Ctrl+1                      │
│                                      │
│ Esc Cancel                           │
└──────────────────────────────────────┘
```

绑定时应直接捕获用户按键组合，而不是要求手动输入字符串。

### 8.4 设备选择弹窗

```text
┌──────────── Select Output Device ────────────┐
│ > BlackHole 2ch                               │
│   Headphones                                  │
│   MacBook Speakers                            │
│   VB-CABLE Input                              │
│                                               │
│ Enter Select       Esc Cancel                 │
└───────────────────────────────────────────────┘
```

## 9. 配置页面

配置页面分组：

1. 音频设备；
2. 音量；
3. 播放策略；
4. 快捷键；
5. 方案管理；
6. 日志与诊断。

示例：

```text
┌──────────────────── Config ─────────────────────┐
│ Audio Devices                                    │
│   Mic Input:      USB Microphone                 │
│   Virtual Output: BlackHole 2ch                  │
│   Monitor:        Headphones                     │
│                                                   │
│ Volumes                                           │
│   Mic:      90%                                   │
│   SFX:      80%                                   │
│   Monitor:  60%                                   │
│                                                   │
│ Playback                                          │
│   Mode: Interrupt                                 │
│   Ducking: Enabled                                │
│                                                   │
│ M Select Mic  O Select Output  L Select Monitor  │
│ S Save        Esc Back                            │
└───────────────────────────────────────────────────┘
```

## 10. 帮助页面

帮助页面应列出：

- 主界面快捷键；
- 配置页面快捷键；
- 音效操作；
- 设备操作；
- 全局快捷键说明；
- 常见问题。

## 11. 日志页面

日志页面显示最近运行日志：

- 启动信息；
- 设备枚举；
- 快捷键注册；
- 音频播放事件；
- 错误信息。

日志级别：

- INFO；
- WARN；
- ERROR；
- DEBUG。

## 12. 窗口尺寸适配

当终端尺寸较小时，应启用简化布局：

- 隐藏分类栏；
- 缩短字段显示；
- 隐藏部分帮助提示；
- 提示用户放大终端窗口。

## 13. 主题设计

完整版本可支持：

- 默认主题；
- 暗色主题；
- 高对比主题；
- 自定义颜色。

终端颜色应谨慎使用，避免在不同终端中显示异常。
