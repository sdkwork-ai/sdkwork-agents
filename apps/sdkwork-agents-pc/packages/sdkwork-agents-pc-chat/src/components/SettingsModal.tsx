import React, { useState } from 'react';
import { X, Sun, Moon, Monitor } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useTheme } from '@sdkwork/agents-pc-commons';
import { useModelSettings, ModelSettings } from '../hooks/useModelSettings';

interface SettingsModalProps {
  onClose: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({ onClose }) => {
  const { t, i18n } = useTranslation('settings');
  const { theme, setTheme } = useTheme();

  const [activeTab, setActiveTab] = useState('Google');
  const { settings, setSettings } = useModelSettings(activeTab);

  const handleLanguageChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    window.localStorage.setItem('user_explicit_lang', e.target.value);
    void i18n.changeLanguage(e.target.value);
  };

  const updateSetting = <K extends keyof ModelSettings>(key: K, value: ModelSettings[K]) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  };

  const vendors = ['Google', 'OpenAI', 'Anthropic'];

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center">
      <div className="bg-white dark:bg-[#282828] border border-gray-200 dark:border-gray-800 rounded-2xl w-full max-w-3xl h-[650px] shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        <div className="px-6 py-4 border-b border-gray-200 dark:border-[#333] flex justify-between items-center bg-gray-50 dark:bg-[#202020]">
          <h2 className="font-semibold text-lg text-gray-900 dark:text-white">{t('title')}</h2>
          <button onClick={onClose} className="text-gray-400 dark:text-gray-500 hover:text-gray-900 dark:hover:text-white p-1 rounded-md hover:bg-gray-200 dark:hover:bg-white/10 transition-colors">
            <X size={20} />
          </button>
        </div>
        
        <div className="flex flex-1 overflow-hidden">
          <div className="w-48 bg-gray-50 dark:bg-[#202020] border-r border-gray-200 dark:border-[#333] p-4 flex flex-col gap-1">
            <p className="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2 px-2">{t('providers')}</p>
            {vendors.map(v => (
              <button
                key={v}
                onClick={() => setActiveTab(v)}
                className={`text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                  activeTab === v 
                    ? 'bg-[#1890ff] text-white font-medium' 
                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#333]'
                }`}
              >
                {v}
              </button>
            ))}
            
            <p className="text-xs font-semibold text-gray-500 uppercase tracking-wider mt-4 mb-2 px-2">{t('general')}</p>
            <button
                onClick={() => setActiveTab('General')}
                className={`text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                  activeTab === 'General' 
                    ? 'bg-[#1890ff] text-white font-medium' 
                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-[#333]'
                }`}
              >
                {t('appearance')}
              </button>
          </div>

          <div className="flex-1 p-6 overflow-y-auto">
            {activeTab !== 'General' ? (
              <div className="space-y-6">
                <div>
                  <h3 className="text-base font-medium text-gray-900 dark:text-white mb-1">{activeTab} {t('configuration')}</h3>
                  <p className="text-sm text-gray-500">{t('apiKeysDescription')}</p>
                </div>
                
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">{t('apiKeyLabel')}</label>
                    <input 
                      type="password" 
                      value={settings.apiKey}
                      onChange={(e) => updateSetting('apiKey', e.target.value)}
                      placeholder={`sk-...`} 
                      className="w-full bg-white dark:bg-[#333] border border-gray-200 dark:border-[#444] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white focus:ring-1 focus:ring-[#1890ff] dark:focus:ring-[#1890ff] outline-none placeholder-gray-400 dark:placeholder-gray-500 focus:border-[#1890ff] transition-colors" 
                    />
                  </div>
                  
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">{t('baseUrlLabel')}</label>
                    <input 
                      type="text" 
                      value={settings.baseUrl}
                      onChange={(e) => updateSetting('baseUrl', e.target.value)}
                      placeholder={t('baseUrlDefault')} 
                      className="w-full bg-white dark:bg-[#333] border border-gray-200 dark:border-[#444] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white focus:ring-1 focus:ring-[#1890ff] dark:focus:ring-[#1890ff] outline-none placeholder-gray-400 dark:placeholder-gray-500 focus:border-[#1890ff] transition-colors" 
                    />
                  </div>

                  <div>
                    <label className="flex justify-between text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                      <span>{t('temperatureLabel')}</span>
                      <span>{settings.temperature}</span>
                    </label>
                    <input 
                      type="range" 
                      min="0" max="2" step="0.1"
                      value={settings.temperature}
                      onChange={(e) => updateSetting('temperature', parseFloat(e.target.value))}
                      className="w-full accent-[#1890ff]" 
                    />
                  </div>
                </div>
              </div>
            ) : (
              <div className="space-y-6">
                <div>
                  <h3 className="text-base font-medium text-gray-900 dark:text-white mb-4">{t('appearance')}</h3>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">{t('theme')}</label>
                  <div className="grid grid-cols-3 gap-4">
                    {[
                      { id: 'light', label: t('lightTheme'), icon: <Sun size={24} className="mb-3" /> },
                      { id: 'system', label: t('systemDefault'), icon: <Monitor size={24} className="mb-3" /> },
                      { id: 'dark', label: t('sleekDark'), icon: <Moon size={24} className="mb-3" /> },
                    ].map((tOption) => (
                      <button
                        key={tOption.id}
                        onClick={() => setTheme(tOption.id as 'light' | 'dark' | 'system')}
                        className={`p-4 rounded-xl border-2 flex flex-col items-center justify-center transition-all ${
                          theme === tOption.id
                            ? 'border-[#1890ff] bg-[#1890ff]/5 text-[#1890ff] dark:bg-[#1890ff]/10 dark:text-[#1890ff] shadow-sm'
                            : 'border-transparent bg-gray-100 dark:bg-[#333] text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-[#444]'
                        }`}
                      >
                        {tOption.icon}
                        <span className="text-sm font-medium">{tOption.label}</span>
                      </button>
                    ))}
                  </div>
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{t('language')}</label>
                  <select 
                    value={i18n.language}
                    onChange={handleLanguageChange}
                    className="w-full bg-white dark:bg-[#333] border border-gray-200 dark:border-[#444] rounded-lg px-4 py-2.5 text-sm text-gray-900 dark:text-white focus:ring-1 focus:ring-[#1890ff] outline-none appearance-none cursor-pointer focus:border-[#1890ff] transition-colors"
                  >
                    <option value="en-US">English</option>
                    <option value="zh-CN">简体中文</option>
                  </select>
                </div>
              </div>
            )}
          </div>
        </div>

        <div className="px-6 py-4 border-t border-gray-200 dark:border-[#333] flex justify-end gap-3 bg-gray-50 dark:bg-[#202020]">
          <button onClick={onClose} className="px-5 py-2.5 rounded-xl text-sm font-medium bg-[#1890ff] text-white hover:bg-[#40a9ff] transition-colors shadow-sm shadow-[#1890ff]/20">{t('ok')}</button>
        </div>
      </div>
    </div>
  );
};
