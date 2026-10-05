### [English HERE](guide_en.md)
## [在线文档](https://huiinyg-rusting.github.io/Rssust/guide_cn.html)
# What is Rssust? 
Rssust是一个使用Rust语言做的**信息聚合转换软件**，希望让每个人在桌面电脑上使用网站转RSS这项技术，类似于Rsshub，***但目前还在开发***，
## 我们希望得到你的帮助
厌倦了**庞大的内存占用，缓慢的运行速度与自建门槛极高**以及**依赖于别人**建的Rsshub服务器但是有一天居然停服了，或者你干脆就是个**Rust迷**？

虽然现在Rssust的路由和RSShub的路由在数量上不是一个量级，但是这样提供了一个先例，也许借助AI技术，可以使Rssust的路由数量得到增长，但这可能有点抄袭的嫌疑，所以还是要求AI自己生成好了。所以如果这**侵犯了你的权利**，请邮件并且找所有可能的方式联系我，我会尽快处理。

如果你想自己做一个路由可以点这里：[路由制作指南——中文](new_router_cn.md)
不想写 Rust 的话，可以用 rhai 脚本写路由：[脚本路由说明](script_route_cn.md)
这里是目前服务器支持的ROUTERS,你的左侧Router一栏都是，但这里有一些分类[ROUTERS](routes.md)
## 对于一般用户使用：
### 安装：
值得一提的是，运行时，二进制的目录结构如下所示：
```sh
├── cookies.json
├── docs_md
│   ├── .......md
│   └── official
│       └── ......md
├── index
│   ├── 404.html
│   └── index.html
├── scripts            # 可选，需要编译时开启 scripts feature
│   └── .......rhai
└── rssust
```
比较严苛
### 二进制：
运行二进制十分简单：在release下载，解压，然后启动env文件夹下的二进制文件即可
### 从源码构建：
Linux用户：
```sh
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
./build.sh
```
想用 rhai 脚本写路由（不写 Rust），编译时额外加 `--features scripts`：
```sh
cargo build --release --features scripts
```
脚本放进 exe 同目录的 `scripts/`，文件名即路由名，写法见[脚本路由说明](script_route_cn.md)。

然后没有版本更新的时候，你可以输入如下命令来更新html文档以及从firefox抓取cookies（注意这会抓取所有cookies,注意保护隐私）：
```sh
cd Rssust
./env/rssust cookie firefox
./env/rssust docs
./env/rssust
```
来启动
Windows用户：CMD（未经验证）
```shell
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
cargo build --features scripts
robocopy .\target\debug .\env rssust /IF /S
.\env\rssust.exe cookie firefox
.\env\rssust.exe docs
.\env\rssust.exe
```