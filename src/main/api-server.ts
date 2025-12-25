import express, { Express, Request, Response } from 'express';
import cors from 'cors';
import { Server } from 'http';
import { getDevices, isCurrentlySearching, refreshSearch } from './mdns-service';
import { DevicesResponse, SearchResponse, StatusResponse } from '../shared/types';

const PORT = 31415;
const VERSION = '1.0.0';

let app: Express | null = null;
let server: Server | null = null;
const startTime = Date.now();

export async function startApiServer(): Promise<void> {
  return new Promise((resolve, reject) => {
    try {
      app = express();

      // CORS 設定 - DigiCode からのリクエストを許可
      app.use(cors({
        origin: [
          'http://localhost:5173',
          'http://localhost:5174',
          'https://app.digital-fab.jp',
          'https://digicode.pages.dev',
        ],
        methods: ['GET', 'POST'],
        allowedHeaders: ['Content-Type'],
      }));

      app.use(express.json());

      // GET /api/devices - デバイス一覧
      app.get('/api/devices', (_req: Request, res: Response) => {
        const devices = getDevices();
        const response: DevicesResponse = {
          success: true,
          devices,
          searchDuration: Date.now() - startTime,
          timestamp: new Date().toISOString(),
        };
        res.json(response);
      });

      // POST /api/search - 検索開始
      app.post('/api/search', (req: Request, res: Response) => {
        const timeout = req.body?.timeout || 5000;
        refreshSearch();

        const response: SearchResponse = {
          success: true,
          message: 'Search started',
          timeout,
        };
        res.json(response);
      });

      // GET /api/status - ステータス
      app.get('/api/status', (_req: Request, res: Response) => {
        const devices = getDevices();
        const response: StatusResponse = {
          success: true,
          version: VERSION,
          isSearching: isCurrentlySearching(),
          deviceCount: devices.length,
          uptime: Math.floor((Date.now() - startTime) / 1000),
        };
        res.json(response);
      });

      // Health check
      app.get('/health', (_req: Request, res: Response) => {
        res.json({ status: 'ok', version: VERSION });
      });

      server = app.listen(PORT, () => {
        console.log(`[API] Server listening on http://localhost:${PORT}`);
        resolve();
      });

      server.on('error', (error: NodeJS.ErrnoException) => {
        if (error.code === 'EADDRINUSE') {
          console.error(`[API] Port ${PORT} is already in use`);
        }
        reject(error);
      });

    } catch (error) {
      reject(error);
    }
  });
}

export function stopApiServer(): void {
  if (server) {
    server.close(() => {
      console.log('[API] Server stopped');
    });
    server = null;
  }
  app = null;
}
