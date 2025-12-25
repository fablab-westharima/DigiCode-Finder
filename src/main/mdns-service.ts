import Bonjour, { Service } from 'bonjour-service';
import { DigiCodeDevice, IPC_CHANNELS } from '../shared/types';
import { getMainWindow } from './index';

let bonjour: Bonjour | null = null;
let browser: ReturnType<Bonjour['find']> | null = null;
let isSearching = false;

// 検出されたデバイスを保持
const devices: Map<string, DigiCodeDevice> = new Map();

export function getDevices(): DigiCodeDevice[] {
  return Array.from(devices.values());
}

export function isCurrentlySearching(): boolean {
  return isSearching;
}

function serviceToDevice(service: Service): DigiCodeDevice {
  const txt: DigiCodeDevice['txt'] = {};

  // TXT レコードをパース
  if (service.txt) {
    for (const [key, value] of Object.entries(service.txt)) {
      txt[key] = value?.toString();
    }
  }

  return {
    name: service.name,
    host: service.host || `${service.name}.local`,
    addresses: service.addresses || [],
    port: service.port,
    txt,
    lastSeen: new Date(),
  };
}

export function startMdnsSearch(): void {
  if (isSearching) {
    console.log('[mDNS] Already searching');
    return;
  }

  console.log('[mDNS] Starting search for _digicode._tcp services...');
  isSearching = true;

  try {
    bonjour = new Bonjour();

    browser = bonjour.find({ type: 'digicode' }, (service: Service) => {
      console.log('[mDNS] Service found:', service.name);

      const device = serviceToDevice(service);
      devices.set(device.name, device);

      // Renderer に通知
      const mainWindow = getMainWindow();
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send(IPC_CHANNELS.DEVICE_FOUND, device);
      }
    });

    // サービスが消えた時の処理
    browser.on('down', (service: Service) => {
      console.log('[mDNS] Service down:', service.name);
      devices.delete(service.name);

      const mainWindow = getMainWindow();
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send(IPC_CHANNELS.DEVICE_REMOVED, service.name);
      }
    });

  } catch (error) {
    console.error('[mDNS] Failed to start search:', error);
    isSearching = false;
  }
}

export function stopMdnsSearch(): void {
  console.log('[mDNS] Stopping search...');
  isSearching = false;

  if (browser) {
    browser.stop();
    browser = null;
  }

  if (bonjour) {
    bonjour.destroy();
    bonjour = null;
  }
}

export function refreshSearch(): void {
  console.log('[mDNS] Refreshing search...');
  devices.clear();
  stopMdnsSearch();

  // 少し待ってから再開
  setTimeout(() => {
    startMdnsSearch();
  }, 500);
}
