import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { DigiCodeDevice } from '../types';

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
      const deviceList = await invoke<DigiCodeDevice[]>('get_devices');
      setDevices(deviceList);
    } catch (error) {
      console.error('Failed to fetch devices:', error);
    }
  }, []);

  // ステータスを取得
  const fetchStatus = useCallback(async () => {
    try {
      const [searching, count] = await invoke<[boolean, number]>('get_status');
      setIsSearching(searching);
      // deviceCount は表示用に使うかもしれないが、現在は使わない
      void count;
    } catch (error) {
      console.error('Failed to fetch status:', error);
    }
  }, []);

  // 検索をリフレッシュ
  const refresh = useCallback(async () => {
    try {
      setIsSearching(true);
      setDevices([]);
      await invoke('start_search');
    } catch (error) {
      console.error('Failed to start search:', error);
      setIsSearching(false);
    }
  }, []);

  // 初期化とイベントリスナーのセットアップ
  useEffect(() => {
    let unlistenFound: UnlistenFn | undefined;
    let unlistenRemoved: UnlistenFn | undefined;

    const setup = async () => {
      // 初期データを取得
      await fetchDevices();
      await fetchStatus();

      // デバイス発見イベント
      unlistenFound = await listen<DigiCodeDevice>('device-found', (event) => {
        const device = event.payload;
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
      unlistenRemoved = await listen<string>('device-removed', (event) => {
        const deviceName = event.payload;
        setDevices((prev) => prev.filter((d) => d.name !== deviceName));
      });
    };

    setup();

    // 定期的にステータスを更新
    const statusInterval = setInterval(fetchStatus, 2000);

    return () => {
      unlistenFound?.();
      unlistenRemoved?.();
      clearInterval(statusInterval);
    };
  }, [fetchDevices, fetchStatus]);

  return { devices, isSearching, refresh };
}
