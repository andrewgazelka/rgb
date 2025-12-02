import { goto } from '$app/navigation';

export interface Keybind {
  key: string;
  description: string;
  action: () => void;
  category: 'navigation' | 'view' | 'action' | 'map';
}

export interface KeybindState {
  showHelp: boolean;
}

// Page-specific action handlers
type ActionHandler = () => void;

interface PageHandlers {
  panUp?: ActionHandler;
  panDown?: ActionHandler;
  panLeft?: ActionHandler;
  panRight?: ActionHandler;
  zoomIn?: ActionHandler;
  zoomOut?: ActionHandler;
  reset?: ActionHandler;
  refresh?: ActionHandler;
  selectNext?: ActionHandler;
  selectPrev?: ActionHandler;
  openSelected?: ActionHandler;
}

let pageHandlers: PageHandlers = {};

export function registerPageHandlers(handlers: PageHandlers): () => void {
  pageHandlers = { ...pageHandlers, ...handlers };
  return () => {
    // Cleanup: remove these handlers
    for (const key of Object.keys(handlers) as (keyof PageHandlers)[]) {
      if (pageHandlers[key] === handlers[key]) {
        delete pageHandlers[key];
      }
    }
  };
}

let state: KeybindState = $state({ showHelp: false });

export function getState(): KeybindState {
  return state;
}

export function toggleHelp(): void {
  state.showHelp = !state.showHelp;
}

export function closeHelp(): void {
  state.showHelp = false;
}

export const keybinds: Keybind[] = [
  // Navigation
  { key: 'g d', description: 'Go to Dashboard', action: () => goto('/'), category: 'navigation' },
  { key: 'g w', description: 'Go to World', action: () => goto('/world'), category: 'navigation' },
  { key: 'g m', description: 'Go to Map', action: () => goto('/map'), category: 'navigation' },

  // View
  { key: '?', description: 'Toggle help', action: toggleHelp, category: 'view' },
  { key: 'Escape', description: 'Close overlay', action: closeHelp, category: 'view' },

  // Map/World navigation (vim-style)
  { key: 'h', description: 'Pan left', action: () => pageHandlers.panLeft?.(), category: 'map' },
  {
    key: 'j',
    description: 'Pan down / Next row',
    action: () => {
      pageHandlers.panDown?.();
      pageHandlers.selectNext?.();
    },
    category: 'map',
  },
  {
    key: 'k',
    description: 'Pan up / Prev row',
    action: () => {
      pageHandlers.panUp?.();
      pageHandlers.selectPrev?.();
    },
    category: 'map',
  },
  { key: 'l', description: 'Pan right', action: () => pageHandlers.panRight?.(), category: 'map' },
  { key: '+', description: 'Zoom in', action: () => pageHandlers.zoomIn?.(), category: 'map' },
  { key: '=', description: 'Zoom in', action: () => pageHandlers.zoomIn?.(), category: 'map' },
  { key: '-', description: 'Zoom out', action: () => pageHandlers.zoomOut?.(), category: 'map' },
  { key: '0', description: 'Reset view', action: () => pageHandlers.reset?.(), category: 'map' },
  { key: 'r', description: 'Refresh data', action: () => pageHandlers.refresh?.(), category: 'action' },
  { key: 'Enter', description: 'Open selected', action: () => pageHandlers.openSelected?.(), category: 'action' },
];

// Sequence tracking for multi-key bindings
let keySequence: string[] = [];
let sequenceTimeout: number | undefined;

function resetSequence(): void {
  keySequence = [];
  if (sequenceTimeout) {
    clearTimeout(sequenceTimeout);
    sequenceTimeout = undefined;
  }
}

function matchKeybind(sequence: string[]): Keybind | undefined {
  const sequenceStr = sequence.join(' ');
  return keybinds.find((kb) => kb.key === sequenceStr);
}

function isPartialMatch(sequence: string[]): boolean {
  const sequenceStr = sequence.join(' ');
  return keybinds.some((kb) => kb.key.startsWith(sequenceStr + ' '));
}

export function handleKeydown(event: KeyboardEvent): boolean {
  // Don't handle if user is typing in an input
  const target = event.target as HTMLElement;
  if (
    target.tagName === 'INPUT' ||
    target.tagName === 'TEXTAREA' ||
    target.tagName === 'SELECT' ||
    target.isContentEditable
  ) {
    return false;
  }

  // Get the key representation
  const key = event.key;

  // Handle Escape specially - always close help
  if (key === 'Escape') {
    if (state.showHelp) {
      closeHelp();
      event.preventDefault();
      event.stopPropagation();
      return true;
    }
    resetSequence();
    return false;
  }

  // Handle ? for help
  if (key === '?') {
    toggleHelp();
    event.preventDefault();
    event.stopPropagation();
    return true;
  }

  // Don't process other keys if help is open
  if (state.showHelp) {
    return false;
  }

  // Add to sequence
  keySequence.push(key);

  // Clear previous timeout
  if (sequenceTimeout) {
    clearTimeout(sequenceTimeout);
  }

  // Check for exact match
  const match = matchKeybind(keySequence);
  if (match) {
    event.preventDefault();
    event.stopPropagation();
    match.action();
    resetSequence();
    return true;
  }

  // Check for partial match
  if (isPartialMatch(keySequence)) {
    event.preventDefault();
    event.stopPropagation();
    // Set timeout to reset sequence if no follow-up key
    sequenceTimeout = window.setTimeout(resetSequence, 1000);
    return true;
  }

  // No match, reset sequence
  resetSequence();
  return false;
}

export function getKeybindsByCategory(): Record<string, Keybind[]> {
  const result: Record<string, Keybind[]> = {};
  for (const kb of keybinds) {
    if (!result[kb.category]) {
      result[kb.category] = [];
    }
    result[kb.category].push(kb);
  }
  return result;
}
