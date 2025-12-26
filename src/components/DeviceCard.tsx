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
  const version = device.txt?.version || 'unknown';
  const primaryAddress = device.addresses[0] || device.host;

  // 最終検出時刻をフォーマット
  const formatLastSeen = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - new Date(date).getTime();
    const seconds = Math.floor(diff / 1000);

    if (seconds < 60) return 'たった今';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}分前`;
    return `${Math.floor(seconds / 3600)}時間前`;
  };

  return (
    <div className="bg-white rounded-xl border border-gray-200 p-4 hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between">
        <div className="flex items-center space-x-3">
          {/* デバイスアイコン */}
          <div className="w-10 h-10 bg-gradient-to-br from-green-400 to-emerald-500 rounded-lg flex items-center justify-center flex-shrink-0">
            <svg
              className="w-6 h-6 text-white"
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
          <div>
            <h3 className="font-semibold text-gray-800">{displayName}</h3>
            <p className="text-sm text-gray-500">{device.host}</p>
          </div>
        </div>

        {/* ステータスインジケーター */}
        <div className="flex items-center space-x-1">
          <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse-dot" />
          <span className="text-xs text-green-600">オンライン</span>
        </div>
      </div>

      {/* 詳細情報 */}
      <div className="mt-4 pt-3 border-t border-gray-100 grid grid-cols-2 gap-3 text-sm">
        <div>
          <span className="text-gray-400 text-xs">IP アドレス</span>
          <p className="text-gray-700 font-mono">{primaryAddress}</p>
        </div>
        <div>
          <span className="text-gray-400 text-xs">ポート</span>
          <p className="text-gray-700 font-mono">{device.port}</p>
        </div>
        <div>
          <span className="text-gray-400 text-xs">バージョン</span>
          <p className="text-gray-700">{version}</p>
        </div>
        <div>
          <span className="text-gray-400 text-xs">最終検出</span>
          <p className="text-gray-700">{formatLastSeen(device.lastSeen)}</p>
        </div>
      </div>

      {/* UUID があれば表示 */}
      {device.txt?.uuid && (
        <div className="mt-3 pt-3 border-t border-gray-100">
          <span className="text-gray-400 text-xs">UUID</span>
          <p className="text-gray-500 font-mono text-xs truncate">{device.txt.uuid}</p>
        </div>
      )}

      {/* コピーボタン */}
      <button
        onClick={handleCopyDeviceInfo}
        className={`mt-4 w-full py-2 px-4 rounded-lg font-medium text-sm transition-all ${
          copied
            ? 'bg-green-500 text-white'
            : 'bg-purple-600 hover:bg-purple-700 text-white'
        }`}
      >
        {copied ? (
          <span className="flex items-center justify-center gap-2">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            コピーしました
          </span>
        ) : (
          <span className="flex items-center justify-center gap-2">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
            </svg>
            書込みデバイス情報を取得
          </span>
        )}
      </button>
    </div>
  );
}

export default DeviceCard;
