import { createContextId } from '@builder.io/qwik';
import type { User, UserRole } from '~/types';
import apiService from '~/services/api';

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export const AUTH_CONTEXT = createContextId<AuthState>('auth');

export const initialAuthState: AuthState = {
  user: null,
  token: null,
  isAuthenticated: false,
  isLoading: false,
  error: null,
};

export const hasPermission = (user: User | null, requiredRoles: UserRole[]): boolean => {
  if (!user) return false;
  return requiredRoles.includes(user.role);
};

export const canManageTowers = (user: User | null): boolean => {
  return hasPermission(user, ['admin', 'engineer']);
};

export const canAcknowledgeAlerts = (user: User | null): boolean => {
  return hasPermission(user, ['admin', 'engineer', 'technician', 'operator']);
};

export const canTriggerShutdown = (user: User | null): boolean => {
  return hasPermission(user, ['admin', 'engineer']);
};

export const canCreateInspections = (user: User | null): boolean => {
  return hasPermission(user, ['admin', 'engineer', 'technician']);
};

export const canViewReports = (user: User | null): boolean => {
  return hasPermission(user, ['admin', 'engineer', 'technician', 'operator', 'viewer']);
};

export const initializeAuth = async (): Promise<{ user: User | null; token: string | null }> => {
  const token = apiService.loadToken();
  if (!token) {
    return { user: null, token: null };
  }

  try {
    const user = await apiService.getCurrentUser();
    return { user, token };
  } catch (error) {
    apiService.clearToken();
    return { user: null, token: null };
  }
};
