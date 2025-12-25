import { useState, useEffect, useCallback } from 'react';
import { check, Update } from '@tauri-apps/plugin-updater';

interface UseUpdaterReturn {
  updateAvailable: boolean;
  newVersion: string | null;
  isChecking: boolean;
  isDownloading: boolean;
  downloadProgress: number;
  checkForUpdates: () => Promise<void>;
  downloadAndInstall: () => Promise<void>;
}

export function useUpdater(): UseUpdaterReturn {
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [newVersion, setNewVersion] = useState<string | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [update, setUpdate] = useState<Update | null>(null);

  const checkForUpdates = useCallback(async () => {
    setIsChecking(true);
    try {
      const updateResult = await check();
      if (updateResult) {
        setUpdate(updateResult);
        setUpdateAvailable(true);
        setNewVersion(updateResult.version);
      } else {
        setUpdateAvailable(false);
        setNewVersion(null);
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
    } finally {
      setIsChecking(false);
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    if (!update) return;

    setIsDownloading(true);
    setDownloadProgress(0);

    let contentLength = 0;
    let downloaded = 0;

    try {
      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          contentLength = event.data.contentLength || 0;
          console.log(`Download started, total size: ${contentLength}`);
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          if (contentLength > 0) {
            setDownloadProgress((downloaded / contentLength) * 100);
          }
        } else if (event.event === 'Finished') {
          console.log('Download finished');
          setDownloadProgress(100);
        }
      });
    } catch (error) {
      console.error('Failed to download and install update:', error);
    } finally {
      setIsDownloading(false);
    }
  }, [update]);

  // 起動時に更新チェック
  useEffect(() => {
    checkForUpdates();
  }, [checkForUpdates]);

  return {
    updateAvailable,
    newVersion,
    isChecking,
    isDownloading,
    downloadProgress,
    checkForUpdates,
    downloadAndInstall,
  };
}
