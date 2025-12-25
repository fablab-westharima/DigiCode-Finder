import { useDevices } from './hooks/useDevices';
import DeviceCard from './components/DeviceCard';
import Header from './components/Header';

function App() {
  const { devices, isSearching, refresh } = useDevices();

  return (
    <div className="min-h-screen bg-gray-50">
      <Header isSearching={isSearching} onRefresh={refresh} />

      <main className="p-4">
        {devices.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-gray-500">
            {isSearching ? (
              <>
                <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mb-4" />
                <p>デバイスを検索中...</p>
              </>
            ) : (
              <>
                <svg
                  className="w-16 h-16 mb-4 text-gray-300"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={1.5}
                    d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                  />
                </svg>
                <p className="text-lg font-medium mb-2">デバイスが見つかりません</p>
                <p className="text-sm text-gray-400">
                  ネットワークに接続された DigiCode デバイスを探しています
                </p>
              </>
            )}
          </div>
        ) : (
          <div className="space-y-3">
            {devices.map((device) => (
              <DeviceCard key={device.name} device={device} />
            ))}
          </div>
        )}
      </main>

      <footer className="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 px-4 py-2">
        <div className="flex items-center justify-between text-xs text-gray-400">
          <span>
            {devices.length} 台のデバイス
            {isSearching && (
              <span className="ml-2 inline-flex items-center">
                <span className="w-1.5 h-1.5 bg-green-500 rounded-full animate-pulse-dot mr-1" />
                検索中
              </span>
            )}
          </span>
          <span>DigiCode Helper v1.0.0</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
