import React from 'react';
import { t } from 'i18next';

function App() {
  return (
    <div>
      <h1>{t('欢迎使用我们的应用')}</h1>
      <p>{t('这是一个测试页面')}</p>
      <button>{t('点击这里')}</button>
      <span>{t('用户名')}</span>
      <div>{t('密码')}</div>
      <label>{t('登录')}</label>
      <p>{t('注册新账户')}</p>
      <span>{t('忘记密码？')}</span>
    </div>
  );
}

export default App;