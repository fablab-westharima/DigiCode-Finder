import { useState, useEffect, useCallback } from 'react';
import { DigiCodeDevice } from '../../shared/types';

interface UseDevicesReturn {
  devices: DigiCodeDevice[];
  isSearching: boolean;
  refresh: () => Promise<void>;
}

export function useDevices(): UseDevicesReturn {
  const [devices, setDevices] = useState<DigiCodeDevice[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  // デバイス一覧を取得
  const fetchDevices = useCallback(async () => {
    try {
      const deviceList = await window.electronAPI.getDevices();
      setDevices(deviceList);
    } catch (error) {
      console.error('Failed to fetch devices:', error);
    }
  }, []);

  // ステータスを取得
  const fetchStatus = useCallback(async () => {
    try {
      const status = await window.electronAPI.getStatus();
      setIsSearching(status.isSearching);
    } catch (error) {
      console.error('Failed to fetch status:', error);
    }
  }, []);

  // 検索をリフレッシュ
  const refresh = useCallback(async () => {
    try {
      setIsSearching(true);
      setDevices([]);
      await window.electronAPI.startSearch();
    } catch (error) {
      console.error('Failed to start search:', error);
      setIsSearching(false);
    }
  }, []);

  // 初期化とイベントリスナーのセットアップ
  useEffect(() => {
    // 初期データを取得
    fetchDevices();
    fetchStatus();

    // デバイス発見イベント
    const unsubscribeFound = window.electronAPI.onDeviceFound((device) => {
      setDevices((prev) => {
        const existing = prev.findIndex((d) => d.name === device.name);
        if (existing >= 0) {
          const updated = [...prev];
          updated[existing] = device;
          return updated;
        }
        return [...prev, device];
      });
    });

    // デバイス削除イベント
    const unsubscribeRemoved = window.electronAPI.onDeviceRemoved((deviceName) => {
      setDevices((prev) => prev.filter((d) => d.name !== deviceName));
    });

    // 定期的にステータスを更新
    const statusInterval = setInterval(fetchStatus, 2000);

    return () => {
      unsubscribeFound();
      unsubscribeRemoved();
      clearInterval(statusInterval);
    };
  }, [fetchDevices, fetchStatus]);

  return { devices, isSearching, refresh };
}
