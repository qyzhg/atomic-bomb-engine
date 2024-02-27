# performance-engine

轻量级，高性能压测引擎

## 项目简介

    一款高性能轻量级的压测引擎
    可以通过命令行简单调用
    也可以被其他语言作为压测平台的引擎使用
    该项目使用了rust异步开发，性能强悍，无环境依赖

## 使用方法

### 命令行
#### 从release中下载适合您的平台的可执行程序
#### 进入程序所在目录
#### windows 查看参数帮助
```bash
 performance-engine.exe -h
```
#### 类Unix查看参数帮助
```bash
performance-engine -h
```

## 示例

### 命令行

__Windows系统在命令行中要加上.exe后缀名！！！__

#### 发送GET请求，10并发，持续20秒， 超时时间60秒
```bash
performance-engine -u https://example.com -m GET -c 10 -d 20 --timeout 60
```

#### 发送POST请求，并带传入json参数,100并发，持续60秒， 超时时间10秒
```bash
performance-engine -u https://example.com -m POST --json '{"key":"val"}' -c 100 -d 60 --timeout 10 
```

#### 发送POST请求，并带传入form表单,30并发，持续100秒，超时时间10秒,设置鉴权请求头
```bash
performance-engine -u https://example.com -m POST --form key1=val1&key2=val2 -c 30 -d 100 --timeout 10 -H 'Authorization:Bearer xxx'
```
__(多个请求头可以使用多个 -H参数)__

#### 发送POST请求，并带传入json参数,66并发，持续20秒， 超时时间10秒, 设置cookie
```bash
performance-engine -u https://example.com -m POST --json '{"key":"val"}' -c 10 -d 66 --timeout 10 --cookie 11111;22222;33333
```

#### 发送GET请求，1并发，持续1秒， 超时时间10秒， 打印调试日志
```bash
performance-engine -u https://example.com -m GET -c 1 -d 1 --timeout 10 -v
```

## 技术栈

rust

## 贡献

我们欢迎并鼓励社区参与项目贡献！如果您想贡献，您可以遵循以下步骤：

1. **提出问题：** 如果您发现任何问题或有改进建议，请创建一个 Issue 来描述您的问题或建议。

2. **提交代码：** 如果您愿意贡献代码，请遵循以下步骤：

    - Fork 项目到您的仓库。
    - 在您的本地环境中克隆您 Fork 的仓库：`git clone [您的仓库 URL]`。
    - 创建一个新的分支并进行修改：`git checkout -b [分支名称]`。
    - 在您的分支上进行修改、添加或删除代码。
    - 提交您的更改：`git commit -m "描述您的更改"`。
    - 将您的更改推送到您的 GitHub 仓库：`git push origin [您的分支名称]`。
    - 创建一个 Pull Request，并描述您的更改。

3. **参与讨论：** 您也可以参与开放的讨论，提出您的观点或回复他人的问题。

我们将会仔细审查您的贡献，并在合适的时候进行合并。感谢您的支持！

## 联系方式

qyzhg@qyzhg.com
