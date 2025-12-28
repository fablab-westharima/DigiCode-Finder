import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { getVersion } from '@tauri-apps/api/app';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useDevices } from './hooks/useDevices';
import { useUpdater } from './hooks/useUpdater';
import DeviceCard from './components/DeviceCard';
import Header from './components/Header';
import './i18n';

// Windowsかどうかを判定
const isWindows = navigator.platform.toLowerCase().includes('win');

function App() {
  const { t, i18n: i18nInstance } = useTranslation();
  const { devices, isSearching, isVerifying, refresh } = useDevices();
  const { updateAvailable, newVersion, isDownloading, downloadAndInstall } = useUpdater();
  const [appVersion, setAppVersion] = useState('');
  const [allCopied, setAllCopied] = useState(false);
  const currentLang = i18nInstance.language?.startsWith('ja') ? 'ja' : 'en';

  const handleCopyAllDevices = async () => {
    const deviceInfos = devices.map(device => ({
      type: 'digicode-device',
      name: device.txt?.name || device.name,
      ip: device.addresses[0] || '',
      port: device.port,
      version: device.txt?.version || '',
      uuid: device.txt?.uuid || '',
    }));

    try {
      await writeText(JSON.stringify(deviceInfos));
      setAllCopied(true);
      setTimeout(() => setAllCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy all devices:', err);
    }
  };

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => setAppVersion('unknown'));
  }, []);

  return (
    <div className="min-h-screen bg-gray-50">
      {updateAvailable && (
        <div className="bg-blue-600 text-white px-4 py-2 flex items-center justify-between text-sm">
          <span>{t('update.available', { version: newVersion })}</span>
          <button
            onClick={downloadAndInstall}
            disabled={isDownloading}
            className="bg-white text-blue-600 px-3 py-1 rounded font-medium hover:bg-blue-50 disabled:opacity-50"
          >
            {isDownloading ? t('update.updating') : t('update.updateNow')}
          </button>
        </div>
      )}
      <Header isSearching={isSearching} onRefresh={refresh} />

      <main className="p-4">
        {devices.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-gray-500">
            {isSearching ? (
              <>
                <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mb-4" />
                <p>{t('search.searching')}</p>
              </>
            ) : (
              <>
                <svg
                  className="w-16 h-16 mb-4 text-gray-300"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={1.5}
                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                  />
                </svg>
                <p className="text-lg font-medium mb-2">{t('search.noDevices')}</p>
                <p className="text-sm text-gray-400 mb-4">
                  {t('search.hint')}
                </p>
                <button
                  onClick={refresh}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg text-sm font-medium transition-colors"
                >
                  {t('search.startSearch')}
                </button>
                {isWindows && (
                  <div className="mt-6 p-3 bg-amber-50 border border-amber-200 rounded-lg text-sm">
                    <p className="text-amber-800 mb-2">{t('search.windowsHint')}</p>
                    <a
                      href="https://support.apple.com/kb/DL999"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="inline-flex items-center gap-1 text-amber-600 hover:text-amber-700 font-medium"
                    >
                      {t('search.installBonjour')}
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                      </svg>
                    </a>
                  </div>
                )}
              </>
            )}
          </div>
        ) : (
          <div className="space-y-2">
            {devices.length >= 2 && (
              <button
                onClick={handleCopyAllDevices}
                className={`w-full py-2 px-4 rounded-lg font-medium text-sm transition-all ${
                  allCopied
                    ? 'bg-green-500 text-white'
                    : 'bg-blue-600 hover:bg-blue-700 text-white'
                }`}
              >
                {allCopied ? (
                  <span className="flex items-center justify-center gap-2">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    {t('batch.selected', { count: devices.length })}
                  </span>
                ) : (
                  <span className="flex items-center justify-center gap-2">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                    </svg>
                    {t('batch.selectAll', { count: devices.length })}
                  </span>
                )}
              </button>
            )}
            {devices.map((device) => (
              <DeviceCard key={device.name} device={device} />
            ))}
          </div>
        )}
      </main>

      <footer className="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 px-4 py-2">
        <div className="flex items-center justify-between text-xs text-gray-400">
          <span>
            {t('app.deviceCount', { count: devices.length })}
            {isVerifying && (
              <span className="ml-2 inline-flex items-center">
                <span className="w-1.5 h-1.5 bg-yellow-500 rounded-full animate-pulse-dot mr-1" />
                {t('app.verifying', 'Verifying...')}
              </span>
            )}
            {isSearching && !isVerifying && (
              <span className="ml-2 inline-flex items-center">
                <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse-dot mr-1" />
                {t('app.searching')}
              </span>
            )}
          </span>
          <div className="flex items-center gap-3">
            <div className="flex items-center text-[11px]">
              <button
                onClick={() => i18nInstance.changeLanguage('ja')}
                className={`px-1 ${currentLang === 'ja' ? 'text-blue-600 font-medium' : 'hover:text-gray-600'}`}
              >
                日本語
              </button>
              <span className="text-gray-300">|</span>
              <button
                onClick={() => i18nInstance.changeLanguage('en')}
                className={`px-1 ${currentLang === 'en' ? 'text-blue-600 font-medium' : 'hover:text-gray-600'}`}
              >
                EN
              </button>
            </div>
            <span>{t('app.version', { version: appVersion })}</span>
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
