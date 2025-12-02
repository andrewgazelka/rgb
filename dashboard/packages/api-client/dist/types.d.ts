export interface WorldResponse {
    entity_count: number;
    archetype_count: number;
    component_count: number;
    globals: Record<string, unknown>;
}
export interface EntitySummary {
    id: number;
    name: string | null;
    components: string[];
}
export interface ListEntitiesResponse {
    entities: EntitySummary[];
    total: number;
}
export interface ComponentValue {
    name: string;
    full_name: string;
    value: unknown;
    is_opaque: boolean;
    schema: unknown | null;
}
export interface EntityResponse {
    found: boolean;
    id: number;
    name: string | null;
    components: ComponentValue[];
    parent: number | null;
    children: number[];
}
export interface ComponentResponse {
    found: boolean;
    value: ComponentValue | null;
}
export interface UpdateResponse {
    success: boolean;
    error: string | null;
}
export interface SpawnResponse {
    success: boolean;
    entity: number | null;
    error: string | null;
}
export interface QuerySpec {
    with?: string[];
    optional?: string[];
    filter?: string[];
    without?: string[];
    limit?: number;
    offset?: number;
}
export interface QueryResultRow {
    entity: number;
    name: string | null;
    components: Record<string, unknown>;
}
export interface QueryResponse {
    entities: QueryResultRow[];
    total: number;
    execution_time_us: number;
}
export interface ComponentTypeInfo {
    id: number;
    name: string;
    full_name: string;
    size: number;
    is_opaque: boolean;
    schema: unknown | null;
}
export interface ComponentTypesResponse {
    types: ComponentTypeInfo[];
}
export interface ChunkInfo {
    x: number;
    z: number;
    color: 'red' | 'green' | 'blue';
    loaded: boolean;
}
export interface ChunksResponse {
    chunks: ChunkInfo[];
}
export type ChangeSource = 'dashboard' | 'system' | 'spawn' | 'revert';
export interface HistoryEntry {
    id: number;
    timestamp: number;
    entity: number;
    component: string;
    old_value: unknown | null;
    new_value: unknown | null;
    source: ChangeSource;
}
export interface HistoryResponse {
    entries: HistoryEntry[];
    total: number;
}
export interface RevertResponse {
    success: boolean;
    reverted_to?: number;
    error?: string;
}
//# sourceMappingURL=types.d.ts.map