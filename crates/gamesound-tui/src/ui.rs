use crate::app::{App, DeviceKind, Page};
use gamesound_storage::MonitorMode;
use ratatui::{prelude::*, widgets::*};
pub fn draw(f: &mut Frame, app: &App) {
    match app.page {
        Page::Onboarding => onboarding(f, app),
        Page::Help => help(f, app),
        Page::Config => config(f, app),
        Page::Device(kind) => devices(f, app, kind),
        Page::ProfilePicker => profiles(f, app),
        Page::Logs => logs(f, app),
        Page::Main => main(f, app),
    }
    if let Some(sound_id) = app.capture_hotkey.or(app.pending_hotkey_capture) {
        let area = centered(65, 22, f.area());
        let sound = app
            .sounds
            .iter()
            .find(|sound| sound.id == sound_id)
            .map(|sound| sound.name.as_str())
            .unwrap_or("?");
        let message = if app.pending_hotkey_capture.is_some() {
            tr(app, "Preparing hotkey capture…", "正在准备快捷键捕获…").to_owned()
        } else if app.is_chinese() {
            format!(
                "音效：{sound}\n\n请直接按下想绑定的组合键\n例如 Ctrl+1、Shift+Alt+K\n\nEsc 取消"
            )
        } else {
            format!("Sound: {sound}\n\nPress the desired key combination now\nFor example Ctrl+1 or Shift+Alt+K\n\nEsc cancels")
        };
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(message)
                .block(Block::bordered().title(tr(app, " Bind global hotkey ", " 绑定全局快捷键 ")))
                .wrap(Wrap { trim: false }),
            area,
        );
    } else if app.input.is_some() {
        let area = centered(70, 20, f.area());
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(app.input.as_deref().unwrap_or_default())
                .block(Block::bordered().title(app.notice.as_str()))
                .wrap(Wrap { trim: false }),
            area,
        )
    }
}
fn tr<'a>(app: &App, english: &'a str, chinese: &'a str) -> &'a str {
    if app.is_chinese() {
        chinese
    } else {
        english
    }
}
fn logs(f: &mut Frame, app: &App) {
    let lines = if app.logs.is_empty() {
        tr(app, "No runtime events yet.", "尚无运行时事件。").into()
    } else {
        app.logs.iter().cloned().collect::<Vec<_>>().join("\n")
    };
    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(tr(app, " Runtime logs ", " 运行日志 "))
                    .title_bottom(tr(app, "Esc / J return", "Esc / J 返回")),
            )
            .wrap(Wrap { trim: false }),
        centered(90, 85, f.area()),
    );
}
fn onboarding(f: &mut Frame, app: &App) {
    let text = tr(app, "Welcome to GameSound TUI\n\nBefore audio can reach a voice app you need three devices:\n\n  M  choose your real microphone\n  O  choose a virtual output (BlackHole, VB-CABLE, Loopback)\n  L  optionally choose headphones for local monitoring\n\nThen press C for configuration and T to start the audio engine. In Discord, QQ, WeChat or your game, choose the matching virtual device as the input/microphone.\n\nS marks this guide complete · Q quits", "欢迎使用 GameSound TUI\n\n在将声音送到语音软件前，需要配置三类设备：\n\n  M  选择真实麦克风\n  O  选择虚拟输出（BlackHole、VB-CABLE、Loopback）\n  L  可选：选择耳机作为本地监听\n\n按 C 进入设置，再按 T 启动音频引擎。随后在 Discord、QQ、微信或游戏中，将对应虚拟设备设为输入/麦克风。\n\nS 完成向导 · Q 退出");
    f.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().title(tr(app, " First-run setup ", " 首次配置 ")))
            .wrap(Wrap { trim: false }),
        centered(85, 70, f.area()),
    );
}
fn main(f: &mut Frame, a: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(8),
        Constraint::Length(5),
        Constraint::Length(2),
    ])
    .split(f.area());
    let status = match a.runtime_status {
        gamesound_core::RuntimeStatus::Running => tr(a, "● Running", "● 运行中"),
        gamesound_core::RuntimeStatus::Warning => tr(a, "▲ Warning", "▲ 警告"),
        gamesound_core::RuntimeStatus::Stopped if a.config.audio.devices.output.is_some() => {
            tr(a, "○ Stopped", "○ 未启动")
        }
        gamesound_core::RuntimeStatus::Stopped => tr(a, "▲ Setup needed", "▲ 需要配置"),
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                " GameSound TUI ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(tr(a, " terminal soundboard   ", " 终端音效板   ")),
            Span::styled(
                status,
                Style::default().fg(
                    if matches!(a.runtime_status, gamesound_core::RuntimeStatus::Running) {
                        Color::Green
                    } else {
                        Color::Yellow
                    },
                ),
            ),
            Span::raw(tr(
                a,
                "   Hotkeys configured in Settings",
                "   快捷键可在设置中配置",
            )),
        ]))
        .block(Block::bordered()),
        chunks[0],
    );
    let body = Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(chunks[1]);
    let cats = [
        tr(a, "All sounds", "全部音效"),
        tr(a, "Favorites", "收藏"),
        tr(a, "Recently used", "最近使用"),
    ]
    .into_iter()
    .map(ListItem::new)
    .chain(a.categories.iter().map(|c| ListItem::new(c.name.clone())))
    .collect::<Vec<_>>();
    let mut category_state = ListState::default();
    category_state.select(Some(a.selected_category));
    f.render_stateful_widget(
        List::new(cats)
            .block(Block::bordered().title(tr(a, " Categories ", " 分类 ")))
            .highlight_style(Style::default().bg(Color::Blue)),
        body[0],
        &mut category_state,
    );
    let rows = a.sounds.iter().map(|s| {
        Row::new(vec![
            s.id.to_string(),
            format!("{}{}", if s.favorite { "★ " } else { "" }, s.name),
            a.library
                .hotkey(s.id)
                .ok()
                .flatten()
                .unwrap_or_else(|| "—".into()),
            format!("{:>3}%", (s.volume * 100.) as i32),
            if s.loop_enabled {
                tr(a, "Loop", "循环").into()
            } else {
                s.playback_mode.as_str().into()
            },
            if !s.is_available() {
                tr(a, "Missing", "文件缺失").into()
            } else if a.active_sounds.contains(&s.id) {
                tr(a, "Playing", "播放中").into()
            } else {
                tr(a, "Ready", "就绪").into()
            },
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Min(16),
            Constraint::Length(16),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new([
            "ID",
            tr(a, "Name", "名称"),
            tr(a, "Hotkey", "快捷键"),
            tr(a, "Vol", "音量"),
            tr(a, "Mode", "模式"),
            tr(a, "Status", "状态"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::bordered().title(tr(a, " Sounds ", " 音效 ")))
    .row_highlight_style(Style::default().bg(Color::Blue));
    let mut state = TableState::default();
    state.select(Some(a.selected));
    f.render_stateful_widget(table, body[1], &mut state);
    let none = tr(a, "not selected", "未选择");
    let off = tr(a, "off", "关闭");
    let device = format!(
        "{}: {}    {}: {}    {}: {}\nMic {}  SFX {}  Monitor {}    {}: mic {:>3}% output {:>3}%",
        tr(a, "Mic", "麦克风"),
        a.config.audio.devices.mic.as_deref().unwrap_or(none),
        tr(a, "Output", "输出"),
        a.config.audio.devices.output.as_deref().unwrap_or(none),
        tr(a, "Monitor", "监听"),
        a.config.audio.devices.monitor.as_deref().unwrap_or(off),
        bar(a.config.volume.mic),
        bar(a.config.volume.sfx),
        bar(a.config.volume.monitor),
        tr(a, "levels", "电平"),
        (a.levels.mic * 100.) as i32,
        (a.levels.output * 100.) as i32
    );
    f.render_widget(
        Paragraph::new(device).block(Block::bordered().title(tr(
            a,
            " Devices & levels ",
            " 设备与电平 ",
        ))),
        chunks[2],
    );
    let hint = if a.notice.is_empty() {
        "A Add  E Name  T Tags  V Note  F Favorite  G Loop  I Mode  X Category  B Bind  D Delete  : Command  J Logs"
    } else {
        a.notice.as_str()
    };
    f.render_widget(
        Paragraph::new(hint)
            .style(Style::default().fg(Color::Yellow))
            .block(Block::bordered()),
        chunks[3],
    );
}
fn config(f: &mut Frame, a: &App) {
    let monitor_mode = match a.config.monitor.mode {
        MonitorMode::SfxOnly => "SFX only",
        MonitorMode::FullMix => "full mix",
        MonitorMode::Off => "off",
    };
    let none = tr(a, "not selected", "未选择");
    let enabled = tr(a, "enabled", "开启");
    let disabled = tr(a, "disabled", "关闭");
    let text = if a.is_chinese() {
        format!("当前方案：{}\n\n音频设备\n  麦克风输入：{}\n  虚拟输出：{}\n  本地监听：{}\n\n音量\n  麦克风：{:>3}%   音效：{:>3}%   监听：{:>3}%\n\n播放\n  默认模式：{}\n  Ducking：{}\n  本地监听：{} ({})\n  界面语言：中文\n\nM 麦克风  O 输出  L 监听  1/2/3 清空对应设备\nP 方案  N 新建方案  T 启动/测试  Z 切换中英\nU/Y 麦克风 +/-  +/- 音效  ]/[ 监听 +/-  H 监听开关  B 监听模式  D Ducking  S 保存  Esc 返回",a.config.app.active_profile,a.config.audio.devices.mic.as_deref().unwrap_or(none),a.config.audio.devices.output.as_deref().unwrap_or(none),a.config.audio.devices.monitor.as_deref().unwrap_or(none),(a.config.volume.mic*100.)as i32,(a.config.volume.sfx*100.)as i32,(a.config.volume.monitor*100.)as i32,a.config.audio.playback_mode,if a.config.ducking.enabled{enabled}else{disabled},if a.config.monitor.enabled{enabled}else{disabled},monitor_mode)
    } else {
        format!("Profile: {}\n\nAudio devices\n  Mic input:      {}\n  Virtual output: {}\n  Monitor:        {}\n\nVolumes\n  Mic: {:>3}%   SFX: {:>3}%   Monitor: {:>3}%\n\nPlayback\n  Default mode: {}\n  Ducking: {}\n  Local monitor: {} ({})\n  Interface language: English\n\nM Mic  O Output  L Monitor  1/2/3 Clear selected device\nP Profiles  N New profile  T Start/Test  Z Switch language\nU/Y Mic +/-   +/- SFX   ]/[ Monitor +/-   H Monitor toggle   B Monitor mode   D Ducking   S Save   Esc Back",a.config.app.active_profile,a.config.audio.devices.mic.as_deref().unwrap_or(none),a.config.audio.devices.output.as_deref().unwrap_or(none),a.config.audio.devices.monitor.as_deref().unwrap_or(none),(a.config.volume.mic*100.)as i32,(a.config.volume.sfx*100.)as i32,(a.config.volume.monitor*100.)as i32,a.config.audio.playback_mode,if a.config.ducking.enabled{enabled}else{disabled},if a.config.monitor.enabled{enabled}else{disabled},monitor_mode)
    };
    f.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().title(tr(a, " GameSound configuration ", " GameSound 设置 ")))
            .wrap(Wrap { trim: false }),
        centered(75, 75, f.area()),
    );
}
fn profiles(f: &mut Frame, a: &App) {
    let items = a.profiles.iter().map(|profile| {
        ListItem::new(if profile.id == a.active_profile_id {
            format!("{}  ({})", profile.name, tr(a, "active", "当前"))
        } else {
            profile.name.clone()
        })
    });
    let mut state = ListState::default();
    state.select(Some(a.selected_profile));
    let area = centered(60, 60, f.area());
    f.render_widget(Clear, area);
    f.render_stateful_widget(
        List::new(items)
            .block(
                Block::bordered()
                    .title(tr(a, " Select sound profile ", " 选择音效方案 "))
                    .title_bottom(tr(a, "Enter switch · Esc cancel", "Enter 切换 · Esc 取消")),
            )
            .highlight_style(Style::default().bg(Color::Blue)),
        area,
        &mut state,
    );
}
fn devices(f: &mut Frame, a: &App, kind: DeviceKind) {
    let title = match kind {
        DeviceKind::Mic => tr(a, "Select microphone", "选择麦克风"),
        DeviceKind::Output => tr(a, "Select virtual output", "选择虚拟输出"),
        DeviceKind::Monitor => tr(a, "Select monitor output", "选择监听输出"),
    };
    let items = a.devices.iter().map(|d| {
        ListItem::new(if d.is_virtual {
            format!("{}  ({})", d.name, tr(a, "virtual", "虚拟设备"))
        } else {
            d.name.clone()
        })
    });
    let mut state = ListState::default();
    state.select(Some(a.selected_device));
    f.render_widget(Clear, centered(75, 70, f.area()));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::bordered().title(title).title_bottom(tr(
                a,
                "Enter Select · Esc Cancel",
                "Enter 选择 · Esc 取消",
            )))
            .highlight_style(Style::default().bg(Color::Blue)),
        centered(75, 70, f.area()),
        &mut state,
    )
}
fn help(f: &mut Frame, app: &App) {
    let t=tr(app, "GameSound TUI help\n\nMain: Q quit · ? help · : command panel · J runtime logs · R refresh devices · Tab focus · ↑/↓ select · Enter play · Space stop · P pause/resume · S stop all\nLibrary: A add path · E name · T tags · V note · F favorite · G loop · I cycle mode · X assign current category · B bind hotkey · K clear hotkey · D delete · / search · +/- volume\nCategories: Tab to focus · N create · R rename · D remove (sounds remain)\nDevices: M microphone · O virtual output · L local monitor · C settings\nProfiles: Settings → P switch · N create.\nCommand panel: :play <id>, :stop-all, :set <mic|output|monitor> <device>, :profile <name>, :help.\n\nGlobal hotkeys are separate from TUI keys. Configure only non-conflicting combinations such as Ctrl+1.\n\nVirtual microphone note: choose BlackHole/VB-CABLE/Loopback as GameSound output, then choose its matching input in Discord/QQ/your game.\n\nEsc / Q returns.", "GameSound TUI 帮助\n\n主界面：Q 退出 · ? 帮助 · : 命令面板 · J 运行日志 · R 刷新设备 · Tab 切换区域 · ↑/↓ 选择 · Enter 播放 · Space 停止 · P 暂停/恢复 · S 停止全部\n音效库：A 添加路径 · E 名称 · T 标签 · V 备注 · F 收藏 · G 循环 · I 切换模式 · X 分配当前分类 · B 绑定快捷键 · K 清除快捷键 · D 删除 · / 搜索 · +/- 音量\n分类：Tab 聚焦 · N 新建 · R 重命名 · D 删除（保留音效）\n设备：M 麦克风 · O 虚拟输出 · L 本地监听 · C 设置\n方案：设置中按 P 切换，N 新建。\n命令面板：:play <id>、:stop-all、:set <mic|output|monitor> <设备>、:profile <名称>、:help。\n\n全局快捷键与 TUI 快捷键分开管理，请使用 Ctrl+1 等不冲突组合。\n\n虚拟麦克风说明：选择 BlackHole/VB-CABLE/Loopback 为输出，再在 Discord、QQ 或游戏中选择对应设备作为输入。\n\nEsc / Q 返回。");
    f.render_widget(
        Paragraph::new(t)
            .block(Block::bordered().title(tr(app, " Help ", " 帮助 ")))
            .wrap(Wrap { trim: false }),
        centered(85, 80, f.area()),
    );
}
fn bar(v: f32) -> String {
    let n = (v.clamp(0., 1.) * 10.).round() as usize;
    format!(
        "{}{} {:>3}%",
        "█".repeat(n),
        "░".repeat(10 - n),
        (v * 100.) as i32
    )
}
fn centered(x: u16, y: u16, area: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - y) / 2),
        Constraint::Percentage(y),
        Constraint::Percentage((100 - y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - x) / 2),
        Constraint::Percentage(x),
        Constraint::Percentage((100 - x) / 2),
    ])
    .split(v[1])[1]
}
