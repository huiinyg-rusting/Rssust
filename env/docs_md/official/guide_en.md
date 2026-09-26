### [中文](guide_cn.md)
## [Online Docs](https://huiinyg-rusting.github.io/Rssust/guide_en.html)
# What is Rssust?

Rssust is an **information aggregation and conversion software** built with Rust, aiming to enable everyone to use website-to-RSS technology on desktop computers, similar to Rsshub, ***but currently still in development***.

## We Hope for Your Help

Tired of **massive memory consumption, slow running speed, and the extremely high barrier to self-hosting**, or **dependent on someone else's** Rsshub server that suddenly shuts down one day, or are you simply a **Rust enthusiast**?

Although Rssust's routers are not on the same scale as Rsshub's in terms of quantity, it provides a precedent. Perhaps with AI technology, Rssust's router count can grow, but this may seem somewhat like plagiarism, so it's better to let AI generate its own. Therefore, if this **infringes on your rights**, please email me and contact me through all possible means, and I will handle it as soon as possible.

If you want to create your own router, click here: [Router Development Guide — English](new_router_en.md)
Here are the ROUTERS currently supported by the server. All the routers in your left Router column are listed, but there are some categorized here [ROUTERS](routes.md)

## For General Users:

### Installation:

It's worth noting that when running, the binary directory structure should be as follows:
```sh
├── cookies.json
├── docs_md
│   ├── .......md
│   └── official
│       └── ......md
├── index
│   ├── 404.html
│   └── index.html
└── rssust
```

It's quite strict.
### Binary:

Running the binary is very simple: download it from the release page, extract it, and then start the binary file in the env folder.

### Building from Source:

Linux users:
```sh
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
./build.sh
```

Then when there's no version update, you can run the commands below to refresh the HTML docs and grab cookies from Firefox (note that this grabs ALL cookies — mind your privacy):
```sh
cd Rssust
./env/rssust cookie firefox
./env/rssust docs
./env/rssust
```

to start it.

Windows users: CMD (unverified)
```shell
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
cargo build
robocopy .\target\debug .\env rssust /IF /S
.\env\rssust.exe cookie firefox
.\env\rssust.exe docs
.\env\rssust.exe
```