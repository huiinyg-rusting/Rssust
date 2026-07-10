# 给用户使用的函数——简便
函数签名自己去src/easyuser.rs看去
## 爬虫类：
**fetch_obscura()**——无头浏览器爬取
**fetch_reqwest_get()**——reqwest get模式爬取
**fetch_reqwest_post()**——reqwest post模式爬取
## 序列化类：
**params_to_hashmap()**——从key1=1&key2=2 到{"key1": "2", "key2": "2"}的Hashmap
**hashmap_to_params()**——从HashMap到key1=1&key2=2
## 时间：
**now()**——使用RSS标准输出当前时间
**chinese_date_to_parse()**——x月y日到RSS标准（`x月y日`这个很严格）
## 字符类：
**no_double_quotes()**——去除首尾双引号

#### 总的来说，项目需要很多人完善这些代码
核心代码还有很多空缺和需要改进的地方与API