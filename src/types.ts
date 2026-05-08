/*
 * DigiCode Finder - mDNS Device Detector
 * Copyright (C) 2024-2026 DigiCo LLC
 *
 * Licensed under the GNU Affero General Public License version 3 or later.
 * See LICENSE file in the repository root for full terms.
 */

// DigiCodeデバイス情報
export interface DigiCodeDevice {
  name: string;
  host: string;
  addresses: string[];
  port: number;
  txt: {
    uuid?: string;
    name?: string;
    version?: string;
    [key: string]: string | undefined;
  };
  lastSeen: Date;
  isOnline?: boolean; // オンライン状態（到達性確認済み）
}

// API レスポンス
export interface DevicesResponse {
  success: boolean;
  devices: DigiCodeDevice[];
  searchDuration: number;
  timestamp: string;
}

export interface SearchResponse {
  success: boolean;
  message: string;
  timeout: number;
}

export interface StatusResponse {
  success: boolean;
  version: string;
  isSearching: boolean;
  deviceCount: number;
  uptime: number;
}

// IPC チャンネル
export const IPC_CHANNELS = {
  GET_DEVICES: 'get-devices',
  START_SEARCH: 'start-search',
  STOP_SEARCH: 'stop-search',
  DEVICE_FOUND: 'device-found',
  DEVICE_REMOVED: 'device-removed',
  SEARCH_COMPLETE: 'search-complete',
  GET_STATUS: 'get-status',
} as const;

export type IpcChannel = typeof IPC_CHANNELS[keyof typeof IPC_CHANNELS];
