import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { DigiCodeDevice } from '../types';

interface DeviceCardProps {
  device: DigiCodeDevice;
}

function DeviceCard({ device }: DeviceCardProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const handleCopyDeviceInfo = async () => {
    const deviceInfo = {
      type: 'digicode-device',
      name: device.txt?.name || device.name,
      ip: device.addresses[0] || '',
      port: device.port,
      version: device.txt?.version || '',
      uuid: device.txt?.uuid || '',
    };

    try {
      await writeText(JSON.stringify(deviceInfo));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const displayName = device.txt?.name || device.name;
  const version = device.txt?.version || '-';
  const primaryAddress = device.addresses[0] || device.host;
  const uuid = device.txt?.uuid || '-';

  return (
    <div className="bg-white rounded-lg border border-gray-200 p-3 hover:shadow-md hover:border-purple-300 transition-all">
      <div className="flex items-center gap-3 mb-2">
        <div className="w-10 h-10 bg-gradient-to-br from-green-400 to-emerald-500 rounded-lg flex items-center justify-center flex-shrink-0">
          <svg
            className="w-5 h-5 text-white"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"
            />
          </svg>
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-bold text-gray-800 truncate">{displayName}</h3>
            <span className="flex items-center gap-1 text-xs text-green-600 flex-shrink-0">
              <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse-dot" />
              {t('device.online')}
            </span>
          </div>
        </div>

        <button
          onClick={handleCopyDeviceInfo}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all flex-shrink-0 ${
            copied
              ? 'bg-green-500 text-white'
              : 'bg-purple-600 hover:bg-purple-700 text-white'
          }`}
        >
          {copied ? (
            <span className="flex items-center gap-1.5">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              {t('device.selected')}
            </span>
          ) : (
            <span className="flex items-center gap-1.5">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              {t('device.select')}
            </span>
          )}
        </button>
      </div>

      <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm border-t border-gray-100 pt-2">
        <div className="flex items-center gap-2">
          <span className="text-gray-400 text-xs w-16">{t('device.ip')}</span>
          <span className="text-gray-700 font-mono text-xs">{primaryAddress}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-gray-400 text-xs w-16">{t('device.port')}</span>
          <span className="text-gray-700 font-mono text-xs">{device.port}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-gray-400 text-xs w-16">{t('device.firmware')}</span>
          <span className="text-gray-700 text-xs">{version}</span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-gray-400 text-xs w-16">{t('device.uuid')}</span>
          <span className="text-gray-500 font-mono text-xs truncate">{uuid}</span>
        </div>
      </div>
    </div>
  );
}

export default DeviceCard;
