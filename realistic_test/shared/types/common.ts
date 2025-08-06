// Common type definitions used across the application

export interface BaseEntity {
    id: string | number;
    createdAt: Date | string;
    updatedAt: Date | string;
    createdBy?: string;
    updatedBy?: string;
}

export interface ApiResponse<T = any> {
    success: boolean;
    data?: T;
    error?: {
        code: string;
        message: string;
        details?: any;
    };
    metadata?: {
        timestamp: number;
        requestId: string;
        version: string;
    };
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
    pagination: {
        page: number;
        pageSize: number;
        totalPages: number;
        totalItems: number;
        hasNext: boolean;
        hasPrevious: boolean;
    };
}

export interface QueryParams {
    search?: string;
    sort?: string;
    order?: 'asc' | 'desc';
    page?: number;
    limit?: number;
    filters?: Record<string, any>;
}

export interface ValidationError {
    field: string;
    message: string;
    code: string;
    value?: any;
}

export interface CacheConfig {
    ttl: number;
    key: string;
    invalidateOn?: string[];
    compress?: boolean;
}

export type Status = 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';

export interface Task extends BaseEntity {
    name: string;
    description?: string;
    status: Status;
    priority: 'low' | 'medium' | 'high' | 'critical';
    assignee?: string;
    dueDate?: Date;
    tags?: string[];
    metadata?: Record<string, any>;
}

export interface Notification {
    id: string;
    type: 'info' | 'warning' | 'error' | 'success';
    title: string;
    message: string;
    timestamp: Date;
    read: boolean;
    actionUrl?: string;
    actionLabel?: string;
}

export interface Permission {
    resource: string;
    action: string;
    conditions?: Record<string, any>;
}

export interface Role {
    id: string;
    name: string;
    permissions: Permission[];
    inheritFrom?: string[];
}

export interface AuthToken {
    accessToken: string;
    refreshToken?: string;
    expiresIn: number;
    tokenType: string;
    scope?: string[];
}

export interface WebSocketMessage<T = any> {
    event: string;
    data: T;
    timestamp: number;
    correlationId?: string;
}

export interface FileUpload {
    filename: string;
    mimetype: string;
    size: number;
    url?: string;
    thumbnail?: string;
    metadata?: {
        width?: number;
        height?: number;
        duration?: number;
        [key: string]: any;
    };
}

export interface Analytics {
    event: string;
    properties?: Record<string, any>;
    timestamp: Date;
    userId?: string;
    sessionId?: string;
    context?: {
        ip?: string;
        userAgent?: string;
        referrer?: string;
        [key: string]: any;
    };
}