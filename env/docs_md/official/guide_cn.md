### [English HERE](/docs/guide_en.html)
# What is Rssust? 
Rssust是一个使用Rust语言做的**信息聚合转换软件**，希望让每个人在桌面电脑上使用网站转RSS这项技术，类似于Rsshub，***但目前还在开发***，
## 我们希望得到你的帮助
厌倦了**庞大的内存占用和缓慢的运行速度**以及**依赖于别人**建的Rsshub服务器但是有一天居然停服了，或者你干脆就是个**Rust迷**？
___
插个U盘在别的电脑，然后***双击***打开二进制的程序和一个绿色的RSS客户端就可以在别人电脑上使用RSS，而不是费尽心思的去找别人的服务器。让每个人承担自己的算力反而拥有更多的稳定性。
___
##### 那么本项目是一个本地运行项目吗？并不是，*更多的是希望本地运行*，但是如果你家有台NAS或者是有财力建服务器的就当我没说，
**但是也不妨试一下Rssust**。
---
虽然现在Rssust的路由和Rsshub的路由在数量上不是一个量级，但是这样提供了一个先例，也许借助AI技术，可以使Rssust的路由数量得到增长，但这可能有点抄袭的嫌疑，所以还是要求AI自己生成好了。所以如果这**侵犯了你的权利**，请邮件并且找所有可能的方式联系我，我会尽快处理。

如果你想自己做一个路由可以点这里：[路由制作指南——中文](/docs/new_router_cn.html)
这里是目前服务器支持的API,你的左侧Router一栏都是，但这里有一些分类[API DOCS](/docs/api.html)

## 对于一般用户使用：
### 安装：
值得一提的是，运行时，二进制的目录结构如下所示：
```sh
├── cookies.json
├── docs_md
│   ├── .......md
│   └── official
│       └── ......md
├── index
│   ├── 404.html
│   └── index.html
└── rssust
```
比较严苛
### cookies:
有些路由依赖于cookies,所以请看[cookies的指南](/docs/cookie.html)
### 二进制：
早期版本可能不会附带二进制，只有版本号带S才会生成二进制
运行二进制十分简单：解压，然后启动env文件夹下的二进制文件即可
### 从源码构建：
Linux用户：
```sh
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
cargo build
cp -r ./target/debug/rssust ./env
./env/rssust docs
```
然后没有版本更新的时候，你可以输入：
```sh
cd Rssust
./env/rssust
```
来启动
Windows用户：CMD（未经验证）
```shell
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
cargo build
robocopy .\target\debug .\env rssust /IF /S
.\env\rssust.exe docs
```