### [English HERE](/docs/new_router_en.html)
# 路由制作指南
本文可以给人看，也可以给AI看
在开发前，你需要：
- 一个标准的cargo环境
- git 版本管理系统
- 编辑器
- **基础**rust的知识,最好是通过官方书籍，以及有一些std库里频率很高的方法也要学会，比如给可恢复错误做处理的几种方法之类的;
- RSS的知识 可以通过：[官方RSS 2.0.11 英文](https://www.rssboard.org/rss-specification) 或者 [博客园-小y 翻译的RSS 2.0.10 中文](https://www.cnblogs.com/tuyile006/p/3691024.html)进行学习
- RSS Crates 的知识，懂得如何构建一个RSS [官方英文文档](https://docs.rs/rss/latest/rss/)
  **尤其**是[ChannelBuilder](https://docs.rs/rss/latest/rss/struct.ChannelBuilder.html)和[ImageBuilder](https://docs.rs/rss/latest/rss/struct.ImageBuilder.html)
- 因为不同上游API,你需要学会不同的方法，比如说你的上游API是json格式，你其实实际大概率只需要学会serde_json Crates的.pointer()和.get()这两种方法，而如果你需要直接处理html和DOM,你需要[scraper](https://docs.rs/scraper/0.27.0/scraper/)的文档中推荐的方法，以及避免导致panic！！！
  
你可以在/src/router下新建一个你自己的.rs文件，但有一些要求：
1. 文件名只能包含小写字母或数字或下划线还有固定的.rs后缀，严禁在任何文件名或者网址路径中出现空格！
2. 路由的规范命名是这样的： 平台小写英文名_功能小写英文名
3. 文件名的除了后缀名的部分就是你的路由名;
4. 每一个路由在/docs下都必须有一个对应为 路由名.md的文档，待会再讲;
5. 每一个路由只能对应一种功能;

开发前，最好先编译一次，增量编译在开发模式已经开启，第一次编译会耗时十分钟以内，要下很多的Crates,之后都是十秒钟编译完的事
### 导入库
```rust
use std::collections::HashMap;
use crate::easyuser::*;
use anyhow::*;
use rss::*;
```
上面这几个库**必须导入**，下面的而看需要导入
```rust
use serde_json::Value;//json用的
use scraper::*;//html用的
```
### 每个路由的主函数规范长这样：
```rust
pub fn get(para: HashMap<String,String>) -> Result<String, Error> {
}
```
函数的调入和返回结果类型不能改的！（如果你没有用到 para,变量，你可以直接把他写成 _para ,这样编译器就不会警告）其中，para的作用是传入用户访问时的通过http get协议传入的参数，以及序列化成HashMap格式了。
项目封装了一些方便用户的函数，可以在src/easyuser.rs查看，或者在[easyuser_cn.html](/docs/easyuser_cn.html)查看
具体的程序逻辑就由你自己编写吧。
### 写完怎么注册呢？
在/src/router/mod.rs中，在最后新建一行，写上 `pub mod 你的路由名;`
然后再在src/request_rules.rs中的request_rules()函数加一个else if 分支，条件为url == "/路由名"（仿照其他分支的例子抄），在{}中用绝调用你的函数，一般而言，路径是 `你的路由名::get(parameters)` 。
### 注册完怎么运行呢？
我对二进制文件设置的一些**必要环境目录十分严苛**，`cookies.json`,`index`文件夹等，都必须在二进制文件同目录下，因此我创建了一个`env`文件夹，需要的环境都在里面。他应该是长这样的
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
我还写了一个脚本叫build.sh供用户使用（目前只做了Linux系统的脚本，Windows系统用户可以自己做一个脚本或者手动操作），其内容是 编译，拷贝（把/target/debug/rssust拷贝到/env，接着运行。
### Cookies的防止
在与二进制文件同目录下的`cookies.json`中如下：
 示例格式：
 {
   "name": "session_id",
   "value": "abc123",
   "domain": ".example.com",
   "path": "/",
   "secure": true,
   "httpOnly": true,
   "sameSite": "Lax",
   "expirationDate": 1893456000
 }
 所有的cookies挤在同一个json文件，程序启动时就会导入。
Tips：可以使用扩展 Cookie-Editor导出为json到剪贴板再合并。
### 文档的编写！
在/docs下新建 你的路由名.md的文件，复制粘贴一下再慢慢填：
```markdown
# Router-name: 
**Commit time:** 
**Cookies?:** 
**Author:** 
**Introduction:** 
**Address:** 
**Example:** 
**Parameter:** 
**Environment Variables:**
```
首先除了`**Cookies?:**`,`**Environment Variables:**`和`**Parameter:** `,其余基本上都是字面意思，但我还是介绍一下：
1. **Router-name:** 填路由名
2. **Commit time:** 提交时间，以`年.月.日`的形式填写
3. **Cookies?:** 两种情况，一种为`no`，一种为yes,后面也可以写备注解释
4. **Author:** 提交人，写你的Github名
5. **Introduction:** 写一下简介（注意，如果平台以什么语言为主就用什么语言）
6. **Address:** 填 `rssust://路由名`就好
7. **Example:** 可选项，如果要选就要用超链接`[填Address相同的](填/路由名)`
8. **Parameter:** 这个意思是http get时可能用到的参数，可以填`no`, 或者空着，另起一行，像下面的示例
``` markdown
**Parameter:** 
1. **disableembed** 
   Type of parameter: bool
   Default value: true
   Meaning: 内嵌视频
2. 接下来如法炮制
```
其中的参数名称跟在序号后面，用`** **`包裹强调
**Type of parameter:** 填类型，自己定
**Default value:** 默认值，要符合`Type of parameter`
**Meaning:** 参数的含义
### 最后一步：提交！
使用git提交前，先运行`cargo fmt`，这可以使别人更好的识别你的代码
你需要提交的有：
1. /src/router/你的路由名.rs
2. env/docs_md/路由名.md (一般情况下请不要提交您的`路由名.html`文件)
3. /src/router/mod.rs以及src/request_rules.rs
   
#### 这只是路由的指南，如果你想改进核心代码，欢迎提交关于代码的改进，完善
