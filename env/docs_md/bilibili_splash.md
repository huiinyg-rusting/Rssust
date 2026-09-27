# Router-name: bilibili_splash  
**Commit time:** 2026.08.10  
**Cookies?:** no  
**Author:** AI转写 / huiinyg-rusting审核 - 原创设计（参照官方API/文档）  
**Introduction:** B站APP端开屏广告信息（自动携带设备标识 buvid，无需登录）  
**Address:** rssust://bilibili_splash  
**Example:** [rssust://bilibili_splash](/bilibili_splash)  
**Parameter:**  
1. **build**  
   Type of parameter: number  
   Default value: 7660100  
   Meaning: 客户端内部版本号（必须是真实存在的版本号，999999999 等假版本号不下发广告）  
2. **mobi_app**  
   Type of parameter: string  
   Default value: android  
   Meaning: android, iphone, ipad  
3. **platform**  
   Type of parameter: string  
   Default value: android  
   Meaning: android, ios  
4. **height**  
   Type of parameter: number  
   Default value: 1920  
   Meaning: 屏幕高度  
5. **width**  
   Type of parameter: number  
   Default value: 1080  
   Meaning: 屏幕宽度  
6. **birth**  
   Type of parameter: string  
   Default value: 0101  
   Meaning: 生日日期(四位数，例 0101)
**Environment Variables:** no
