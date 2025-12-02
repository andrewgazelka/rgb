export class RgbClient {
    baseUrl;
    timeout;
    constructor(options) {
        this.baseUrl = options.baseUrl.replace(/\/$/, '');
        this.timeout = options.timeout ?? 5000;
    }
    async fetch(path, options) {
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
        }
        finally {
            clearTimeout(timeoutId);
        }
    }
    // World endpoints
    async getWorld() {
        return this.fetch('/api/world');
    }
    // Entity endpoints
    async listEntities(options) {
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
    async getEntity(id) {
        return this.fetch(`/api/entities/${id}`);
    }
    async spawnEntity(options) {
        return this.fetch('/api/entities', {
            method: 'POST',
            body: JSON.stringify(options),
        });
    }
    async despawnEntity(id) {
        return this.fetch(`/api/entities/${id}`, {
            method: 'DELETE',
        });
    }
    // Component endpoints
    async getComponent(entityId, componentName) {
        return this.fetch(`/api/entities/${entityId}/components/${componentName}`);
    }
    async updateComponent(entityId, componentName, value) {
        return this.fetch(`/api/entities/${entityId}/components/${componentName}`, {
            method: 'PUT',
            body: JSON.stringify(value),
        });
    }
    async addComponent(entityId, componentName, value) {
        return this.fetch(`/api/entities/${entityId}/components/${componentName}`, {
            method: 'POST',
            body: JSON.stringify(value),
        });
    }
    async removeComponent(entityId, componentName) {
        return this.fetch(`/api/entities/${entityId}/components/${componentName}`, {
            method: 'DELETE',
        });
    }
    // Query endpoint
    async query(spec) {
        return this.fetch('/api/query', {
            method: 'POST',
            body: JSON.stringify(spec),
        });
    }
    // Component types
    async getComponentTypes() {
        return this.fetch('/api/component-types');
    }
    // Chunks (map view)
    async getChunks() {
        return this.fetch('/api/chunks');
    }
    // History endpoints
    async getGlobalHistory(limit) {
        const params = limit ? `?limit=${limit}` : '';
        return this.fetch(`/api/history${params}`);
    }
    async getEntityHistory(entityId, limit) {
        const params = limit ? `?limit=${limit}` : '';
        return this.fetch(`/api/history/entity/${entityId}${params}`);
    }
    async getComponentHistory(entityId, componentName, limit) {
        const params = limit ? `?limit=${limit}` : '';
        return this.fetch(`/api/history/entity/${entityId}/component/${componentName}${params}`);
    }
    async revertToEntry(entryId) {
        return this.fetch(`/api/history/revert/${entryId}`, {
            method: 'POST',
        });
    }
}
// Default client factory
export function createClient(baseUrl = 'http://localhost:8080') {
    return new RgbClient({ baseUrl });
}
//# sourceMappingURL=client.js.map