import { useState } from 'react';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { DigiCodeDevice } from '../types';

interface DeviceCardProps {
  device: DigiCodeDevice;
}

function DeviceCard({ device }: DeviceCardProps) {
  const [copied, setCopied] = useState(false);

  // デバイス情報をクリップボードにコピー
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

  return (
    <div className="bg-white rounded-lg border border-gray-200 px-3 py-2 hover:shadow-sm transition-shadow">
      <div className="flex items-center gap-3">
        {/* デバイスアイコン */}
        <div className="w-8 h-8 bg-gradient-to-br from-green-400 to-emerald-500 rounded-md flex items-center justify-center flex-shrink-0">
          <svg
            className="w-4 h-4 text-white"
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

        {/* デバイス情報 */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-gray-800 truncate">{displayName}</h3>
            <span className="text-xs text-gray-400">v{version}</span>
            <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse-dot flex-shrink-0" />
          </div>
          <p className="text-xs text-gray-500 font-mono truncate">
            {primaryAddress}:{device.port}
          </p>
        </div>

        {/* コピーボタン */}
        <button
          onClick={handleCopyDeviceInfo}
          className={`px-3 py-1.5 rounded-md text-xs font-medium transition-all flex-shrink-0 ${
            copied
              ? 'bg-green-500 text-white'
              : 'bg-purple-600 hover:bg-purple-700 text-white'
          }`}
        >
          {copied ? (
            <span className="flex items-center gap-1">
              <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              コピー済
            </span>
          ) : (
            <span className="flex items-center gap-1">
              <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
              </svg>
              コピー
            </span>
          )}
        </button>
      </div>
    </div>
  );
}

export default DeviceCard;
