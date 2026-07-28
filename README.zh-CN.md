# OsaMail — macOS 上的 Apple Mail 命令行工具

[English](README.md) | [简体中文](README.zh-CN.md)

在终端中读取、搜索、整理、自动化处理和发送 Apple Mail 邮件。

OsaMail 是一个开源、本地优先的 macOS Apple Mail CLI。它通过系统自带的
`osascript` 控制已经配置在 Mail 中的账户，因此无需重新配置 IMAP、SMTP、
OAuth 或邮件服务商 API 凭据，就能自动化处理邮件。

```bash
brew install tinylion1024/tap/osamail
osamail doctor
osamail unread --titles
```

OsaMail 支持列出、搜索、查看、打开、标记邮件，以及发送纯文本邮件。它既提供
适合人类阅读的终端输出，也提供适合脚本的结构化 JSON。OsaMail 不是独立邮件
客户端，也不会直接连接 Gmail、iCloud Mail、Exchange 或其他邮件服务商。

## 为什么选择 OsaMail？

| 你的需求 | OsaMail 提供的能力 |
| --- | --- |
| 不离开终端就能检查邮件 | 未读计数、精简主题列表和最近邮件 |
| 自动化现有 Apple Mail | 稳定的 CLI 命令和 JSON 输出 |
| 不想再走一套凭据授权流程 | 使用已在 Apple Mail 中完成认证的账户 |
| 希望邮件访问保持在本地 | 无遥测、不访问 Mail 私有数据库、不主动联网 |
| 明确控制邮件状态 | 一个 `mark` 命令，并提供不做修改的 `--dry-run` 演练 |
| 发送前保持可控 | 支持纯文本发送和“不发送”的 `--dry-run` 演练 |

OsaMail 适合开发者、自动化工作流，以及希望用轻量命令行代替另一个邮件应用的
macOS 用户。

## 30 秒快速开始

先在 Apple Mail 中配置至少一个账户，然后运行：

```bash
# 确认 macOS、Mail 和自动化权限已经就绪
osamail doctor

# 只看未读主题，不加载邮件正文
osamail unread --titles

# 搜索主题和发件人元数据
osamail search "invoice" --titles

# 为脚本获取结构化输出
osamail recent --limit 5 --json
```

邮件列表会返回一个不透明的 `ref`。将这个可以安全作为单个 shell 参数使用的值
传给下面的命令，即可查看邮件、在 Apple Mail 中打开邮件，或预演状态修改：

```bash
osamail show <ref>
osamail open <ref>
osamail mark read <ref> --dry-run
```

## 按目标选择命令

| 目标 | 命令 |
| --- | --- |
| 检查运行环境 | `osamail doctor` |
| 列出 Mail 中的账户 | `osamail accounts` |
| 列出最近邮件 | `osamail recent` |
| 列出未读邮件 | `osamail unread` |
| 统计未读邮件 | `osamail unread --count` |
| 只输出邮件主题 | `osamail unread --titles` |
| 搜索邮件 | `osamail search "query"` |
| 在终端阅读邮件 | `osamail show <ref>` |
| 在 Apple Mail 中打开邮件 | `osamail open <ref>` |
| 预演邮件状态修改 | `osamail mark read <ref> --dry-run` |
| 修改已读或旗标状态 | `osamail mark <action> <ref>` |
| 不发送，只验证邮件参数 | `osamail send ... --dry-run` |
| 发送纯文本邮件 | `osamail send ...` |

运行 `osamail <command> --help` 可查看权威的选项列表。

## 安装

### Homebrew（推荐）

```bash
brew install tinylion1024/tap/osamail
```

带版本号的 GitHub Release 通过构建和安装检查后，公开 Tap 会自动更新。

### GitHub Release

从 [GitHub Releases](https://github.com/tinylion1024/osamail/releases)
下载通用 macOS 压缩包及旁边的 SHA-256 文件：

```bash
tar -xzf osamail-v0.2.0-universal-apple-darwin.tar.gz
install -m 0755 osamail-v0.2.0/osamail /usr/local/bin/osamail
osamail --version
```

发布二进制同时支持 Apple Silicon（`arm64`）和 Intel（`x86_64`）Mac。请安装
到已经位于 `PATH` 的目录；在 Apple Silicon 上，`/opt/homebrew/bin` 也是常见
选择。

### Cargo

0.2.0 发布到 crates.io 后，可运行：

```bash
cargo install osamail
```

安装当前检出版本：

```bash
cargo install --path .
```

### 系统要求

- macOS、Apple Mail 和 `/usr/bin/osascript`。
- 使用账户或邮件命令前，至少在 Apple Mail 中配置一个账户。
- 运行 OsaMail 的终端、IDE 或应用已获得自动化权限。
- 仅从源码构建时需要 Rust 1.85 或更高版本。

OsaMail 0.2.0 已在 macOS 15.3 的 Mail 16.0 上开发并进行实际测试。

## 常用工作流

### 少读内容，更快拿到结果

只需要主题时使用 `--titles`。OsaMail 只向 Mail 请求筛选和排序所需的最少属性，
然后每行输出一个主题：

```bash
osamail recent --titles
osamail unread --titles
osamail search "release" --titles
```

列表命令不会加载邮件正文。`--titles` 不能与 `unread --count` 同时使用。

### 筛选最近邮件或未读邮件

```bash
osamail recent --limit 20
osamail recent --account "Personal"
osamail unread --mailbox "INBOX"
osamail unread --count --json
```

默认列表数量为 10，允许范围为 1 到 200。账户名称必须与 Apple Mail 完全一致。
邮箱名称可能经过本地化，也可能包含嵌套层级。

### 搜索 Apple Mail

```bash
osamail search "GitHub"
osamail search "notice" --from "alerts@example.com"
osamail search "quarterly" --subject "report"
osamail search "security" --unread
osamail search "exact body text" --body
```

位置参数默认搜索主题和发件人元数据，`--from` 和 `--subject` 用于增加筛选条件。
正文搜索需要显式添加 `--body`，因为它在大型邮箱中可能明显更慢。

### 查看或打开邮件

```bash
osamail show <ref>
osamail show <ref> --headers
osamail show <ref> --max-body-bytes 131072
osamail show <ref> --json
osamail open <ref>
```

易读输出默认包含正文，并在 65,536 字节处截断；JSON 保留完整正文。`show` 不会
主动修改已读状态，也不会加载附件。引用是定位信息，不是持久邮件 ID；移动邮件
或修改账户后，请重新获取 `ref`。

### 安全修改邮件状态

先预演修改，再移除 `--dry-run` 执行同一个命令：

```bash
osamail mark read <ref> --dry-run
osamail mark read <ref>
osamail mark unread <ref>
osamail mark flag <ref>
osamail mark unflag <ref>
```

四种操作分别是 `read`、`unread`、`flag` 和 `unflag`。结果会明确区分
`changed`、`already_set` 和 `would_change`，方便脚本保持幂等，也让演练结果
一目了然。默认测试和 CI 绝不会修改真实邮件。

### 安全发送

先验证收件人和正文输入，不创建也不发送邮件：

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body "Test message" \
  --dry-run
```

只有确认要发送时才移除 `--dry-run`。重复使用 `--to`、`--cc` 或 `--bcc` 可添加
多个收件人，`--account` 可选择精确账户；需要正文时，可通过 `--body`、
`--body-file` 或 `--stdin` 三者之一提供。

真实发送成功表示 Apple Mail 已接受请求，不代表远端已经投递。默认测试和 CI
绝不会发送真实邮件。

## JSON 与 shell 自动化

任何命令添加 `--json` 后都会输出且只输出一个 JSON 值。成功结果使用
`{"ok":true,"data":...}` 并写入 stdout；错误使用
`{"ok":false,"error":{"code":...,"message":...}}` 并写入 stderr。如果
OsaMail 能给出明确的恢复步骤，错误还会包含 `hint`。

```bash
# 提取主题
osamail search "invoice" --json | jq -r '.data.messages[].subject'

# 在其他命令中使用未读数量
unread_count="$(osamail unread --count --json | jq -r '.data.count')"
```

全局选项可以放在子命令之前或之后：

```text
--json               输出结构化 JSON
--timeout <SECONDS>  覆盖命令超时时间（1–3600 秒）
--quiet              隐藏成功时的易读文本输出
```

`--quiet` 不会隐藏错误，也不会阻止明确请求的 JSON 输出。在交互式终端中，读取
时间较长时会在 stderr 延迟显示一条状态提示；管道、JSON 和静默输出不受影响。

现有 0.2.0 字段名称保持稳定；邮件状态结果字段从 0.3.0 开始引入。根据响应模型，
可选的 Mail 值可能为 `null` 或被省略。

## macOS 自动化权限

第一次执行真实操作时，macOS 可能会请求控制 Mail 的权限。如果访问被拒绝：

1. 打开**系统设置 → 隐私与安全性 → 自动化**。
2. 允许正在运行 OsaMail 的终端、IDE 或应用控制 **Mail**。
3. 再次运行 `osamail doctor`。

权限与发起调用的应用绑定。更换终端或改为从 IDE 运行时，可能需要单独授权。

## 安全与隐私

OsaMail 使用以下边界：

```text
Rust → /usr/bin/osascript → 内嵌 JXA → Apple Mail
```

每个请求都会被序列化到权限为 `0600` 的不可预测临时 JSON 文件中。用户输入不会
被插入 JXA、AppleScript 或 shell 命令。

- 不读取密码、令牌或服务器凭据。
- 不访问 Apple Mail 私有数据库。
- OsaMail 不添加遥测，也不主动发起网络请求。
- 直接调用 `/usr/bin/osascript`，不使用 `sh -c`。
- 正常完成或失败后都会删除临时请求文件。

与邮件服务商的通信仍由 Apple Mail 负责。完整数据处理模型见
[SECURITY.md](SECURITY.md)。

## 常见问题

### OsaMail 能配合 Gmail、iCloud Mail 或 Exchange 使用吗？

可以，前提是该账户已经在 Apple Mail 中配置并正常工作。OsaMail 控制的是 Mail，
不会直接连接邮件服务商。

### OsaMail 需要我的邮箱密码吗？

不需要。OsaMail 不读取账户密码、OAuth 令牌或服务器凭据。

### OsaMail 是 IMAP 或 SMTP 客户端吗？

不是。它是基于 Apple Mail 自动化能力构建的 macOS 原生命令行接口。

### 列出或查看邮件会把邮件标记为已读吗？

列出和默认的元数据搜索是只读的，也不会加载正文。`show` 不会主动修改已读状态。
`open` 会把邮件交给 Apple Mail，因此后续界面行为由 Mail 控制。

### OsaMail 能标记邮件为已读吗？

可以。运行 `osamail mark read <ref>`；建议先添加 `--dry-run` 预演结果，不修改
邮件。同一个命令也支持 `unread`、`flag` 和 `unflag`。

### OsaMail 能在 Linux 或 Windows 上使用吗？

邮件操作需要 macOS 和 Apple Mail。`--help` 与 `--version` 仍可在其他平台使用。

## 当前限制

OsaMail 暂不支持附件、HTML 渲染或编写、回复、转发、删除、移动、归档、规则、
后台通知、签名或加密。发布产物尚未进行代码签名或公证。

`open` 和窗口聚焦效果取决于 Apple Mail 的脚本行为及当前界面状态。大型邮箱或
正文搜索可能需要更大的 `--timeout`。

## 项目

- [v0.4.0 前的版本路线图](ROADMAP.md)
- [参与贡献](CONTRIBUTING.md)
- [架构说明](docs/architecture.md)
- [发布指南](docs/releasing.md)
- [Homebrew 发布](docs/homebrew.md)
- [更新日志](CHANGELOG.md)
- [安全策略](SECURITY.md)

本地构建与测试：

```bash
cargo build
./scripts/check.sh
./scripts/smoke-test.sh
```

除非同时开启两个显式发送测试开关，否则实际集成测试保持只读。完整策略见
[CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

OsaMail 使用 [MIT 许可证](LICENSE)。

OsaMail 是独立的开源项目，与 Apple Inc. 没有关联，也未获得其认可。Apple、
Apple Mail 和 macOS 是 Apple Inc. 的商标。
