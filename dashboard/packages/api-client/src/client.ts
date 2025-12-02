import type {
  WorldResponse,
  ListEntitiesResponse,
  EntityResponse,
  ComponentResponse,
  UpdateResponse,
  SpawnResponse,
  QuerySpec,
  QueryResponse,
  ComponentTypesResponse,
  ChunksResponse,
  HistoryResponse,
  RevertResponse,
} from './types';

export interface RgbClientOptions {
  baseUrl: string;
  timeout?: number;
}

export class RgbClient {
  private baseUrl: string;
  private timeout: number;

  constructor(options: RgbClientOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, '');
    this.timeout = options.timeout ?? 5000;
  }

  private async fetch<T>(path: string, options?: RequestInit): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(error.error || `HTTP ${response.status}`);
      }

      return response.json();
    } finally {
      clearTimeout(timeoutId);
    }
  }

  // World endpoints
  async getWorld(): Promise<WorldResponse> {
    return this.fetch('/api/world');
  }

  // Entity endpoints
  async listEntities(options?: {
    filter?: string[];
    limit?: number;
    offset?: number;
  }): Promise<ListEntitiesResponse> {
    const params = new URLSearchParams();
    if (options?.filter?.length) {
      params.set('filter', options.filter.join(','));
    }
    if (options?.limit !== undefined) {
      params.set('limit', String(options.limit));
    }
    if (options?.offset !== undefined) {
      params.set('offset', String(options.offset));
    }
    const queryStr = params.toString();
    return this.fetch(`/api/entities${queryStr ? `?${queryStr}` : ''}`);
  }

  async getEntity(id: number): Promise<EntityResponse> {
    return this.fetch(`/api/entities/${id}`);
  }

  async spawnEntity(options: {
    name?: string;
    components: [string, unknown][];
  }): Promise<SpawnResponse> {
    return this.fetch('/api/entities', {
      method: 'POST',
      body: JSON.stringify(options),
    });
  }

  async despawnEntity(id: number): Promise<UpdateResponse> {
    return this.fetch(`/api/entities/${id}`, {
      method: 'DELETE',
    });
  }

  // Component endpoints
  async getComponent(entityId: number, componentName: string): Promise<ComponentResponse> {
    return this.fetch(`/api/entities/${entityId}/components/${componentName}`);
  }

  async updateComponent(
    entityId: number,
    componentName: string,
    value: unknown
  ): Promise<UpdateResponse> {
    return this.fetch(`/api/entities/${entityId}/components/${componentName}`, {
      method: 'PUT',
      body: JSON.stringify(value),
    });
  }

  async addComponent(
    entityId: number,
    componentName: string,
    value: unknown
  ): Promise<UpdateResponse> {
    return this.fetch(`/api/entities/${entityId}/components/${componentName}`, {
      method: 'POST',
      body: JSON.stringify(value),
    });
  }

  async removeComponent(entityId: number, componentName: string): Promise<UpdateResponse> {
    return this.fetch(`/api/entities/${entityId}/components/${componentName}`, {
      method: 'DELETE',
    });
  }

  // Query endpoint
  async query(spec: QuerySpec): Promise<QueryResponse> {
    return this.fetch('/api/query', {
      method: 'POST',
      body: JSON.stringify(spec),
    });
  }

  // Component types
  async getComponentTypes(): Promise<ComponentTypesResponse> {
    return this.fetch('/api/component-types');
  }

  // Chunks (map view)
  async getChunks(): Promise<ChunksResponse> {
    return this.fetch('/api/chunks');
  }

  // History endpoints
  async getGlobalHistory(limit?: number): Promise<HistoryResponse> {
    const params = limit ? `?limit=${limit}` : '';
    return this.fetch(`/api/history${params}`);
  }

  async getEntityHistory(entityId: number, limit?: number): Promise<HistoryResponse> {
    const params = limit ? `?limit=${limit}` : '';
    return this.fetch(`/api/history/entity/${entityId}${params}`);
  }

  async getComponentHistory(
    entityId: number,
    componentName: string,
    limit?: number
  ): Promise<HistoryResponse> {
    const params = limit ? `?limit=${limit}` : '';
    return this.fetch(`/api/history/entity/${entityId}/component/${componentName}${params}`);
  }

  async revertToEntry(entryId: number): Promise<RevertResponse> {
    return this.fetch(`/api/history/revert/${entryId}`, {
      method: 'POST',
    });
  }
}

// Default client factory
export function createClient(baseUrl: string = 'http://localhost:8080'): RgbClient {
  return new RgbClient({ baseUrl });
}
