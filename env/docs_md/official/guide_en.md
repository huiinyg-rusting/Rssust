### [中文](/docs/guide_cn.html)
# What is Rssust?

Rssust is an **information aggregation and conversion software** built with Rust, aiming to enable everyone to use website-to-RSS technology on desktop computers, similar to Rsshub, ***but currently still in development***.

## We Hope for Your Help

Tired of **massive memory consumption and slow running speed**, or **dependent on someone else's** Rsshub server that suddenly shuts down one day, or are you simply a **Rust enthusiast**?

___

Plug a USB drive into someone else's computer, then ***double-click*** to open the binary program and a lightweight RSS client to use RSS on their computer without having to find someone else's server. Let everyone handle their own computing power while actually gaining more stability.

___

##### So is this project meant to run locally? Not necessarily. *It's more about hoping for local operation*, but if you have a NAS at home or have the financial means to build a server, never mind what I said. **But it's still worth trying Rssust**.

---

Although Rssust's routers are not on the same scale as Rsshub's in terms of quantity, it provides a precedent. Perhaps with AI technology, Rssust's router count can grow, but this may seem somewhat like plagiarism, so it's better to let AI generate its own. Therefore, if this **infringes on your rights**, please email me and contact me through all possible means, and I will handle it as soon as possible.

If you want to create your own router, click here: [Router Development Guide — English](/docs/new_router_en.html)
Here are the APIs currently supported by the server. All the routers in your left Router column are listed, but there are some categorized here [API DOCS](/docs/api.html)

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

Early versions may not include binaries; only versions with an "S" suffix will generate binaries.
Running the binary is very simple: extract and then start the binary file in the env folder.

### Building from Source:

Linux users:
```sh
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
cargo build
cp -r ./target/debug/rssust ./env
./env/rssust docs
```

Then when there's no version update, you can enter:
```sh
cd Rssust
./env/rssust
```

to start it.

Windows users: CMD (unverified)
```shell
git clone https://github.com/huiinyg-rusting/Rssust
cd Rssust
cargo build
robocopy .\target\debug .\env rssust /IF /S
.\env\rssust.exe docs
```