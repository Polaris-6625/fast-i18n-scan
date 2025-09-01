import React from 'react';

// This file contains examples of what the scanner detects

const App: React.FC = () => {
  // Hard-coded Chinese text (will be detected)
  const message = "你好世界";
  
  // i18n function calls (keys will be extracted)
  const greeting = t('greeting', '默认问候语');
  const farewell = i18n.t('farewell');
  
  // Hard-coded domain (will be warned)
  const apiUrl = "https://api.example.com/data";
  
  // String concatenation (will be warned)
  const fullMessage = "Hello " + "World";

  return (
    <div>
      {/* Hard-coded Chinese in JSX (will be detected) */}
      <h1>欢迎使用我们的应用</h1>
      
      {/* Proper i18n usage */}
      <p>{t('welcome.message')}</p>
      
      {/* Mixed content */}
      <span>{message} - {greeting}</span>
      
      {/* Template literal with Chinese (will be detected) */}
      <div>{`当前用户: ${username}`}</div>
    </div>
  );
};

export default App;