import { createClient, type RgbClient } from '@rgb/api-client';

const DEFAULT_API_URL = 'http://localhost:8080';
const STORAGE_KEY = 'rgb-dashboard-api-url';

function getStoredUrl(): string {
  if (typeof window === 'undefined') return DEFAULT_API_URL;
  return localStorage.getItem(STORAGE_KEY) || import.meta.env.VITE_API_URL || DEFAULT_API_URL;
}

let currentUrl = getStoredUrl();
let currentClient: RgbClient = createClient(currentUrl);

export function getApiUrl(): string {
  return currentUrl;
}

export function setApiUrl(url: string): void {
  currentUrl = url;
  localStorage.setItem(STORAGE_KEY, url);
  currentClient = createClient(url);
}

export function resetApiUrl(): void {
  localStorage.removeItem(STORAGE_KEY);
  currentUrl = import.meta.env.VITE_API_URL || DEFAULT_API_URL;
  currentClient = createClient(currentUrl);
}

export const client = {
  get current(): RgbClient {
    return currentClient;
  },
};

// Re-export types for convenience
export type * from '@rgb/api-client';
