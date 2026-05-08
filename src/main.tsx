/*
 * DigiCode Finder - mDNS Device Detector
 * Copyright (C) 2024-2026 DigiCo LLC
 *
 * Licensed under the GNU Affero General Public License version 3 or later.
 * See LICENSE file in the repository root for full terms.
 */

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles/globals.css';
import './i18n';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
