@echo off
cargo build && (
    if exist env\rssust.exe del env\rssust.exe
    if not exist env mkdir env
    copy target\debug\rssust.exe env >nul
    start "" env\rssust.exe
)