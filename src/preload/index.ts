import { contextBridge, ipcRenderer } from 'electron';
import { DigiCodeDevice, IPC_CHANNELS } from '../shared/types';

// Renderer に公開する API
const electronAPI = {
  // デバイス一覧を取得
  getDevices: (): Promise<DigiCodeDevice[]> => {
    return ipcRenderer.invoke(IPC_CHANNELS.GET_DEVICES);
  },

  // 検索を開始
  startSearch: (): Promise<{ success: boolean }> => {
    return ipcRenderer.invoke(IPC_CHANNELS.START_SEARCH);
  },

  // ステータスを取得
  getStatus: (): Promise<{ isSearching: boolean; deviceCount: number }> => {
    return ipcRenderer.invoke(IPC_CHANNELS.GET_STATUS);
  },

  // デバイス発見イベントのリスナー
  onDeviceFound: (callback: (device: DigiCodeDevice) => void) => {
    const handler = (_event: Electron.IpcRendererEvent, device: DigiCodeDevice) => {
      callback(device);
    };
    ipcRenderer.on(IPC_CHANNELS.DEVICE_FOUND, handler);
    return () => {
      ipcRenderer.removeListener(IPC_CHANNELS.DEVICE_FOUND, handler);
    };
  },

  // デバイス削除イベントのリスナー
  onDeviceRemoved: (callback: (deviceName: string) => void) => {
    const handler = (_event: Electron.IpcRendererEvent, deviceName: string) => {
      callback(deviceName);
    };
    ipcRenderer.on(IPC_CHANNELS.DEVICE_REMOVED, handler);
    return () => {
      ipcRenderer.removeListener(IPC_CHANNELS.DEVICE_REMOVED, handler);
    };
  },
};

contextBridge.exposeInMainWorld('electronAPI', electronAPI);

// TypeScript 型定義
export type ElectronAPI = typeof electronAPI;
