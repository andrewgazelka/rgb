import type { WorldResponse, ListEntitiesResponse, EntityResponse, ComponentResponse, UpdateResponse, SpawnResponse, QuerySpec, QueryResponse, ComponentTypesResponse, ChunksResponse, HistoryResponse, RevertResponse } from './types';
export interface RgbClientOptions {
    baseUrl: string;
    timeout?: number;
}
export declare class RgbClient {
    private baseUrl;
    private timeout;
    constructor(options: RgbClientOptions);
    private fetch;
    getWorld(): Promise<WorldResponse>;
    listEntities(options?: {
        filter?: string[];
        limit?: number;
        offset?: number;
    }): Promise<ListEntitiesResponse>;
    getEntity(id: number): Promise<EntityResponse>;
    spawnEntity(options: {
        name?: string;
        components: [string, unknown][];
    }): Promise<SpawnResponse>;
    despawnEntity(id: number): Promise<UpdateResponse>;
    getComponent(entityId: number, componentName: string): Promise<ComponentResponse>;
    updateComponent(entityId: number, componentName: string, value: unknown): Promise<UpdateResponse>;
    addComponent(entityId: number, componentName: string, value: unknown): Promise<UpdateResponse>;
    removeComponent(entityId: number, componentName: string): Promise<UpdateResponse>;
    query(spec: QuerySpec): Promise<QueryResponse>;
    getComponentTypes(): Promise<ComponentTypesResponse>;
    getChunks(): Promise<ChunksResponse>;
    getGlobalHistory(limit?: number): Promise<HistoryResponse>;
    getEntityHistory(entityId: number, limit?: number): Promise<HistoryResponse>;
    getComponentHistory(entityId: number, componentName: string, limit?: number): Promise<HistoryResponse>;
    revertToEntry(entryId: number): Promise<RevertResponse>;
}
export declare function createClient(baseUrl?: string): RgbClient;
//# sourceMappingURL=client.d.ts.map