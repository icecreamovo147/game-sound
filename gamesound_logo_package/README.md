# GameSound Tauri Logo Package

本压缩包包含一套可直接用于 Tauri 桌面应用的 GameSound 图标资源。

## 推荐使用方式

将以下目录复制到你的 Tauri 项目中：

```text
src-tauri/icons/
```

然后确认 `src-tauri/tauri.conf.json` 中包含：

```json
{
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

## 文件说明

### src-tauri/icons

- `32x32.png`：Tauri 桌面 PNG 图标
- `128x128.png`：Tauri 桌面 PNG 图标
- `128x128@2x.png`：Tauri 桌面高分屏 PNG 图标，实际尺寸 256x256
- `icon.png`：512x512 PNG 源图标
- `icon.ico`：Windows ICO，包含 16 / 24 / 32 / 48 / 64 / 256 图层
- `icon.icns`：macOS ICNS，包含常见 Retina 图层
- `Square*Logo.png`、`StoreLogo.png`：Windows AppX / Microsoft Store 相关图标

### source

- `app-icon.png`：1024x1024 透明源图
- `app-icon.svg`：可编辑 SVG 源文件

### extras

- `tray-icons/`：适合托盘使用的 light / dark / accent 单色小图标
- `web-favicons/`：React 前端可用 favicon / touch icon
- `macos-iconset/`：macOS `.iconset` 源图层
- `windows-store-extra/`：额外 Windows Store 缩放版本

## 设计说明

图标以“麦克风 + 声波 + 音频控制面板”为核心视觉，适配 GameSound 的游戏语音音效板、麦克风混音、虚拟音频输出等产品定位。主色采用深蓝、青蓝和浅蓝，以兼容日间 / 夜间主题和桌面应用场景。
