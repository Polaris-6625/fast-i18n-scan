// git diff apps/dev-platform/i18n/source/zh.json | grep '^-' | grep -v '^---' | sed 's/^-//' > ./a
// git diff apps/dev-platform/i18n/source/zh.json | grep '^+' | grep -v '^+++' | sed 's/^+//' > ./b
const moduleId = 2954;
const routeId = 3204;
const operate = 'parkeryu';
const keys = [
  // 待填充词条key
];
const ids = [];
const unKowns = [];
for (let i = 0; i < keys.length; i++) {
  try {
    const res = await fetch('https://lingo.woa.com/polaris/api/langpkg/queryTranslationList', {
      headers: {
        accept: 'application/json, text/plain, */*',
        'accept-language': 'zh,zh-CN;q=0.9,ja;q=0.8',
        'cache-control': 'no-cache',
        'content-type': 'application/json;charset=UTF-8',
        pragma: 'no-cache',
        priority: 'u=1, i',
        'sec-ch-ua': '"Not)A;Brand";v="8", "Chromium";v="138", "Google Chrome";v="138"',
        'sec-ch-ua-mobile': '?0',
        'sec-ch-ua-platform': '"macOS"',
        'sec-fetch-dest': 'empty',
        'sec-fetch-mode': 'cors',
        'sec-fetch-site': 'same-origin',
      },
      referrer: 'https://lingo.woa.com/',
      body: `{\"pageSize\":10,\"pageIndex\":1,\"routeId\":\"${routeId}\",\"search\":\"${keys[i]}\",\"searchMode\":-1,\"moduleId\":${moduleId},\"mtEngine\":\"ai-mt\",\"target\":[{\"moduleId\":${moduleId},\"routeId\":${routeId}}],\"operate\":\"${operate}\"}`,
      method: 'POST',
      mode: 'cors',
      credentials: 'include',
    });
    const {
      data: {
        translations: [{ id }],
      },
    } = await res.json();
    ids.push(id);
  } catch (e) {
    unKowns.push(keys[i]);
    console.error(keys[i], e);
  }
}
console.log('未找到id的标识：', unKowns);
fetch('https://lingo.woa.com/polaris/api/langpkg/deleteTranslation', {
  headers: {
    accept: 'application/json, text/plain, */*',
    'accept-language': 'zh,zh-CN;q=0.9,ja;q=0.8',
    'cache-control': 'no-cache',
    'content-type': 'application/json;charset=UTF-8',
    pragma: 'no-cache',
    priority: 'u=1, i',
    'sec-ch-ua': '"Not)A;Brand";v="8", "Chromium";v="138", "Google Chrome";v="138"',
    'sec-ch-ua-mobile': '?0',
    'sec-ch-ua-platform': '"macOS"',
    'sec-fetch-dest': 'empty',
    'sec-fetch-mode': 'cors',
    'sec-fetch-site': 'same-origin',
  },
  referrer: 'https://lingo.woa.com/',
  body: `{\"routeId\":\"${routeId}\",\"id\":[${ids.join(
    ',',
  )}],\"identifier\":\"INTL_LANGPKG_${routeId}\",\"moduleId\":${moduleId},\"target\":[{\"moduleId\":${moduleId},\"routeId\":${routeId}}],\"operate\":\"${operate}\"}`,
  method: 'POST',
  mode: 'cors',
  credentials: 'include',
});
