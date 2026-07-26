# OsaMail

[English](README.md) | [简体中文](README.zh-CN.md)

一个小巧、可脚本化的 Apple Mail 命令行工具，由 `osascript` 驱动。

OsaMail 控制已经在 Apple Mail 中配置好的账户。它不会直接连接 Gmail、
iCloud Mail、Exchange、IMAP、SMTP 或其他邮件服务。

```text
$ osamail unread --count
12

$ osamail recent --limit 2
RECEIVED              STATUS   SENDER                        SUBJECT
2026-07-26T02:30:00Z  unread   GitHub <noreply@github.com>   New pull request
  ref: eyJ2ZXJzaW9uIjoxLC...
2026-07-25T18:04:00Z  read     Build service                 Release complete
  ref: eyJ2ZXJzaW9uIjoxLC...
```

上面的输出用于展示终端格式。账户名称、邮件内容和不透明引用都来自用户自己的
Apple Mail 数据。

## 功能

- 检查本机 Mail 与 macOS 自动化环境。
- 列出 Apple Mail 账户，且不读取凭据。
- 在不加载邮件正文的情况下列出最近邮件或未读邮件。
- 按发件人和主题元数据搜索邮件，可选择同时搜索正文。
- 在终端中以文本或结构化 JSON 显示邮件。
- 在 Apple Mail 中打开指定邮件。
- 通过 Apple Mail 账户发送纯文本邮件，并提供不发送邮件的演练模式。
- 交互使用时输出易读文本，脚本调用时输出 JSON。

OsaMail 以本地优先为原则，不包含遥测，也不会自行发起网络请求。与邮件服务商的
通信仍由 Apple Mail 负责。

## 系统要求

- macOS，且存在 `/System/Applications/Mail.app` 和
  `/usr/bin/osascript`。
- 使用账户或邮件命令前，至少需要在 Apple Mail 中配置一个账户。
- 调用 OsaMail 的终端或应用需要获得自动化权限。
- 从源码构建时需要 Rust 1.85 或更高版本。

OsaMail 0.1.2 已在 macOS 15.3 的 Mail 16.0 上开发并进行实际测试。
自动化授权与调用 OsaMail 的具体终端或应用绑定。

## 安装

### Homebrew

```bash
brew install tinylion1024/tap/osamail
```

推送版本标签后，GitHub Actions 会创建 GitHub Release；发布压缩包通过构建和安装
检查后，工作流会自动更新 Homebrew Tap 中的公式。维护者可以参考
[Homebrew 发布指南](docs/homebrew.md)了解初始化和故障恢复方式。

### GitHub Release

版本发布后，从仓库 Releases 页面下载
`osamail-v0.1.2-universal-apple-darwin.tar.gz`，校验旁边提供的 SHA-256
文件，然后安装二进制：

```bash
tar -xzf osamail-v0.1.2-universal-apple-darwin.tar.gz
install -m 0755 osamail-v0.1.2/osamail /usr/local/bin/osamail
osamail --version
```

在 Apple Silicon 设备上，`/opt/homebrew/bin` 也是常见的用户管理目录。
请使用已经包含在 `PATH` 中的目录；写入 `/usr/local/bin` 可能需要管理员权限。

### Cargo

0.1.2 发布到 crates.io 后，可运行：

```bash
cargo install osamail
```

若要直接安装当前源码，无需等待发布：

```bash
cargo install --path .
```

开发过程中，仓库自动化不会执行真实发布。

## 快速开始

先在 Apple Mail 中配置所需账户，然后运行：

```bash
osamail doctor
osamail accounts
osamail unread --count
osamail recent --limit 5
osamail search "GitHub"
osamail show <ref>
osamail open <ref>
```

`recent`、`unread` 和 `search` 会为每封邮件返回一个不透明的 `ref`。
将这个可以安全作为单个 shell 参数使用的值传给 `show` 或 `open`。

## 命令

全局选项可以放在子命令之前或之后：

```text
--json               输出结构化 JSON
--timeout <SECONDS>  覆盖命令默认超时时间（1–3600 秒）
--quiet              隐藏成功时的易读文本输出
```

`--quiet` 不会隐藏错误，也不会阻止明确请求的 JSON 输出。

### 检查环境

```bash
osamail doctor
osamail doctor --json
osamail --timeout 30 doctor
```

该检查覆盖 macOS、`/usr/bin/osascript`、Mail.app、Mail 自动化权限和已配置
账户数量。

### 列出账户

```bash
osamail accounts
osamail accounts --json
```

账户输出只包含账户名称、配置的电子邮件地址和启用状态，不会包含密码、令牌或
服务器凭据。

### 列出最近邮件

```bash
osamail recent
osamail recent --limit 10
osamail recent --account "Personal"
osamail recent --mailbox "INBOX"
osamail recent --account "Personal" --mailbox "Receipts" --json
```

默认返回 10 封邮件，可接受的范围为 1 到 200。该命令不会加载邮件正文。

### 列出或统计未读邮件

```bash
osamail unread
osamail unread --limit 20
osamail unread --account "Personal"
osamail unread --mailbox "INBOX"
osamail unread --count
osamail unread --count --json
```

易读模式下，`--count` 只输出一个整数。

### 搜索邮件

```bash
osamail search "GitHub"
osamail search "invoice" --account "Personal"
osamail search "release" --limit 20
osamail search "security" --unread
osamail search "notice" --from "alerts@example.com"
osamail search "quarterly" --subject "report"
osamail search "exact body text" --body
```

位置参数默认搜索主题和发件人。`--from` 和 `--subject` 可增加过滤条件。
`--body` 还会搜索 Mail 中的邮件正文，因此可能明显变慢；OsaMail 不会为了搜索
而把所有正文传输到 Rust 进程。

### 显示邮件

```bash
osamail show <ref>
osamail show <ref> --body
osamail show <ref> --headers
osamail show <ref> --max-body-bytes 131072
osamail show <ref> --json
```

易读模式默认包含正文，并在 65,536 字节处截断。调用者也可以用 `--body` 明确
表达需要默认正文。使用 `--max-body-bytes` 调整易读模式的显示上限；JSON 会保留
完整正文。`--headers` 会请求 Mail 返回原始文本头。显示邮件不会有意改变其已读
状态，也不会加载附件。

### 在 Mail 中打开邮件

```bash
osamail open <ref>
```

OsaMail 会验证引用，请求 Mail 打开匹配的邮件并激活 Mail。窗口焦点和选中状态
仍取决于 Mail 当前的界面状态。

### 发送纯文本邮件

以下命令会真实发送邮件：

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body "Test message"
```

需要多个收件人或指定 Apple Mail 账户时，可以重复收件人选项并传入精确的账户名：

```bash
osamail send \
  --to first@example.com \
  --to second@example.com \
  --cc copy@example.com \
  --bcc audit@example.com \
  --account "Personal" \
  --subject "Status" \
  --body "Complete"
```

从文件或标准输入读取正文：

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body-file message.txt

printf '%s\n' 'Test message' \
  | osamail send --to user@example.com --subject "Hello" --stdin
```

`--body`、`--body-file` 和 `--stdin` 互斥。使用演练模式验证收件人和正文输入，
且不创建或发送邮件：

```bash
osamail send \
  --to user@example.com \
  --subject "Hello" \
  --body "Test message" \
  --dry-run
```

指定 `--account` 后，即使是演练模式也会检查是否存在同名且已启用的账户，因此
仍需要 Mail 自动化权限。真实发送成功表示 Apple Mail 已接受请求，并不代表远端
投递已经完成。

运行 `osamail <command> --help` 可查看权威的选项列表。

## JSON 输出

`--json` 向标准输出写入且只写入一个 JSON 值。成功响应使用
`{"ok":true,"data":...}`；失败响应使用
`{"ok":false,"error":{"code":...,"message":...}}` 并写入标准错误。

成功的“不发送”演练示例：

```json
{
  "ok": true,
  "data": {
    "sent": false,
    "dry_run": true,
    "account": null,
    "recipient_count": 1
  }
}
```

邮件列表数据结构如下：

```json
{
  "ok": true,
  "data": {
    "messages": [
      {
        "ref": "opaque-reference",
        "account": "Personal",
        "mailbox": "INBOX",
        "sender": "GitHub <noreply@github.com>",
        "subject": "New pull request",
        "received_at": "2026-07-26T02:30:00.000Z",
        "unread": true
      }
    ],
    "count": 1
  }
}
```

字段名称构成 0.1.2 的机器可读接口。根据响应模型，可选的 Mail 值可能为 `null`
或被省略。

## Shell 管道

使用 `jq` 提取主题：

```bash
osamail search "invoice" --json \
  | jq -r '.data.messages[].subject'
```

以数字形式统计未读邮件：

```bash
osamail unread --count --json \
  | jq -r '.data.count'
```

将 JSON 标准输出与诊断信息分开：

```bash
if ! result="$(osamail recent --json)"; then
  printf '%s\n' "OsaMail failed" >&2
  exit 1
fi
printf '%s\n' "$result" | jq '.data.messages'
```

## macOS 自动化权限

第一次执行真实命令时，macOS 可能会请求授权，让调用 OsaMail 的终端或应用控制
Mail。OsaMail 无法自行授予该权限。

如果访问被拒绝或命令无法继续：

1. 打开“系统设置”。
2. 打开“隐私与安全性”。
3. 打开“自动化”。
4. 允许调用 OsaMail 的终端应用控制“邮件”。
5. 再次运行 `osamail doctor`。

授权与调用程序绑定，因此换用其他终端、IDE 或打包后的启动器时，可能需要单独
授权。

## Apple Mail 账户

OsaMail 不维护账户配置。请在 Apple Mail 中添加、删除、启用账户并完成认证。
通过 `--account` 传入的值必须与 Mail 中的账户名称完全一致；OsaMail 不会静默
回退到其他账户。

邮箱名称来自 Mail，可能已本地化或具有嵌套层级。`INBOX` 只是示例，并不是通用
邮箱名称。

## 安全与隐私

Rust 会将每个自动化请求序列化到一个不可预测、权限为 `0600` 的临时 JSON 文件。
脚本的业务参数只有该文件路径。用户输入绝不会被插入 JXA、AppleScript、shell
文本、日志或错误详情。

OsaMail：

- 使用绝对路径 `/usr/bin/osascript`，且不经过 `sh -c`；
- 捕获子进程输出、检查退出状态并执行超时限制；
- runner 正常完成或报错时都会删除请求文件；
- 不读取账户密码，也不访问 Mail 的私有数据库；
- 不包含遥测，也不主动发起网络请求；
- 默认测试和 CI 测试不会发送真实邮件。

数据处理模型和漏洞报告方式见 [SECURITY.md](SECURITY.md)。

## 已知限制

- 真实命令需要 macOS、Apple Mail、已配置账户和自动化权限。帮助与版本输出仍可
  在其他平台使用。
- 已在 Mail 16.0 上对 `doctor`、`accounts`、最近邮件与未读邮件列表、元数据搜索、
  `show` 和 `unread --count` 进行了只读实际验证；`open` 使用一封已读邮件完成
  验证。开发过程没有真实发送邮件，`send --dry-run` 在不创建待发邮件的情况下
  覆盖了输入和 JSON 行为。
- 不透明引用编码了带版本号的 Mail 定位信息。它可以安全地作为一个 shell 参数，
  但不是持久标识符；移动邮箱、修改账户或 Mail 数据库变化都可能使其失效。
- `open` 依赖 Mail 的脚本行为。Mail 可能会打开邮件，但不一定聚焦预期的查看窗口
  或保留特定选中状态。
- Mail 以文本或富文本形式提供邮件内容。OsaMail 不渲染 HTML、不加载附件，也不
  暴露原始 MIME。
- 正文搜索和大型邮箱可能较慢。必要时可增加 `--timeout`。
- 0.1.2 不支持附件、回复、转发、删除、移动、归档、已读状态修改、旗标、规则、
  通知、模板、HTML 编写、签名或加密。
- 当前发布产物未进行代码签名或公证。

已解决的外部验证问题记录在 [BLOCKERS.md](BLOCKERS.md)。

## 开发

克隆仓库并使用稳定版 Rust 构建：

```bash
git clone https://github.com/tinylion1024/osamail.git
cd osamail
cargo build
cargo run -- --help
```

自动化边界为 Rust -> `/usr/bin/osascript` -> 内嵌 JXA -> Apple Mail。
详见[架构指南](docs/architecture.md)和
[Mail 自动化调查记录](docs/apple-mail-automation.md)。

## 测试

默认检查使用模拟实现，绝不会发送真实邮件：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc
./scripts/check.sh
./scripts/smoke-test.sh
```

在已经授予自动化权限的 macOS 主机上，可以显式运行忽略的只读集成测试：

```bash
OSAMAIL_INTEGRATION=1 cargo test --test macos_integration -- --ignored
```

只有同时设置 `OSAMAIL_INTEGRATION=1` 和 `OSAMAIL_ALLOW_SEND_TEST=1` 时，发送集成
测试才会运行。不要在 CI 中启用发送测试。完整验证策略见
[CONTRIBUTING.md](CONTRIBUTING.md)。

## 发布

维护者必须依次完成完整本地检查、更新变更日志、发布 crate，并创建 `v*` 标签。
GitHub Release 工作流会自动构建发布产物并更新 Homebrew Tap。开发命令不会执行
真实发布。

有序检查清单见[发布指南](docs/releasing.md)，Tap 维护方式见
[Homebrew 指南](docs/homebrew.md)。

## 路线图

0.1.2 之后可能开展的工作：

- 代码签名和公证；
- 加强跨 macOS 与 Mail 版本的集成测试覆盖；
- 针对大型邮箱查询进行有测量依据的性能改进；
- 仅在安全性和脚本行为得到验证后，增加经过选择的邮件操作。

目前不承诺支持附件、回复/转发、邮件修改操作或后台行为。

## 许可证

OsaMail 使用 [MIT 许可证](LICENSE)。

## 独立项目

OsaMail 是独立的开源项目，与 Apple Inc. 没有关联，也未获得其认可。Apple、
Apple Mail 和 macOS 是 Apple Inc. 的商标。
