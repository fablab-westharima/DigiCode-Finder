import { ipcMain } from 'electron';
import { IPC_CHANNELS } from '../shared/types';
import { getDevices, isCurrentlySearching, refreshSearch } from './mdns-service';

export function setupIpcHandlers(): void {
  // デバイス一覧を取得
  ipcMain.handle(IPC_CHANNELS.GET_DEVICES, () => {
    return getDevices();
  });

  // 検索を開始
  ipcMain.handle(IPC_CHANNELS.START_SEARCH, () => {
    refreshSearch();
    return { success: true };
  });

  // ステータスを取得
  ipcMain.handle(IPC_CHANNELS.GET_STATUS, () => {
    return {
      isSearching: isCurrentlySearching(),
      deviceCount: getDevices().length,
    };
  });
}
