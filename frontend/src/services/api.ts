import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import type {
  LoginRequest,
  LoginResponse,
  User,
  Tower,
  Sensor,
  Camera,
  VibrationReading,
  WindSpeedReading,
  IceDetectionData,
  WeatherData,
  Alert,
  IceAnalysisResult,
  ShutdownStrategy,
  InspectionRecord,
  WeatherImpactAnalysis,
  WeatherAlert,
  WeatherForecast,
  VideoVerificationRequest,
  VideoVerificationResult,
  LiveStreamSession,
  PaginatedResponse,
  HealthCheckResponse,
} from '~/types';

const API_BASE_URL = '/api';

class ApiService {
  private axios: AxiosInstance;
  private token: string | null = null;

  constructor() {
    this.axios = axios.create({
      baseURL: API_BASE_URL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    this.axios.interceptors.request.use(
      (config) => {
        if (this.token) {
          config.headers.Authorization = `Bearer ${this.token}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    this.axios.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          this.clearToken();
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }

  setToken(token: string) {
    this.token = token;
    localStorage.setItem('auth_token', token);
  }

  clearToken() {
    this.token = null;
    localStorage.removeItem('auth_token');
  }

  loadToken() {
    const token = localStorage.getItem('auth_token');
    if (token) {
      this.token = token;
    }
    return this.token;
  }

  private async request<T>(config: AxiosRequestConfig): Promise<T> {
    try {
      const response: AxiosResponse<{ success: boolean; data: T }> = await this.axios.request(config);
      return response.data.data;
    } catch (error: any) {
      if (error.response?.data?.error) {
        throw new Error(error.response.data.error.message || '请求失败');
      }
      throw error;
    }
  }

  async login(credentials: LoginRequest): Promise<LoginResponse> {
    const response = await this.axios.post('/auth/login', credentials);
    const data = response.data.data as LoginResponse;
    if (data.token) {
      this.setToken(data.token);
    }
    return data;
  }

  async logout(): Promise<void> {
    try {
      await this.axios.post('/auth/logout');
    } finally {
      this.clearToken();
    }
  }

  async getCurrentUser(): Promise<User> {
    return this.request<User>({
      method: 'GET',
      url: '/auth/me',
    });
  }

  async changePassword(oldPassword: string, newPassword: string): Promise<void> {
    return this.request<void>({
      method: 'POST',
      url: '/auth/change-password',
      data: { oldPassword, newPassword },
    });
  }

  async getTowers(page = 1, pageSize = 20): Promise<PaginatedResponse<Tower>> {
    return this.request<PaginatedResponse<Tower>>({
      method: 'GET',
      url: '/towers',
      params: { page, pageSize },
    });
  }

  async getTower(id: string): Promise<Tower> {
    return this.request<Tower>({
      method: 'GET',
      url: `/towers/${id}`,
    });
  }

  async createTower(data: Partial<Tower>): Promise<Tower> {
    return this.request<Tower>({
      method: 'POST',
      url: '/towers',
      data,
    });
  }

  async updateTower(id: string, data: Partial<Tower>): Promise<Tower> {
    return this.request<Tower>({
      method: 'PUT',
      url: `/towers/${id}`,
      data,
    });
  }

  async deleteTower(id: string): Promise<void> {
    return this.request<void>({
      method: 'DELETE',
      url: `/towers/${id}`,
    });
  }

  async getTowerSensors(towerId: string): Promise<Sensor[]> {
    return this.request<Sensor[]>({
      method: 'GET',
      url: `/towers/${towerId}/sensors`,
    });
  }

  async getTowerCameras(towerId: string): Promise<Camera[]> {
    return this.request<Camera[]>({
      method: 'GET',
      url: `/towers/${towerId}/cameras`,
    });
  }

  async getTowerStatus(towerId: string): Promise<{ status: string; riskLevel: string }> {
    return this.request<{ status: string; riskLevel: string }>({
      method: 'GET',
      url: `/towers/${towerId}/status`,
    });
  }

  async getSensors(towerId?: string, page = 1, pageSize = 50): Promise<PaginatedResponse<Sensor>> {
    return this.request<PaginatedResponse<Sensor>>({
      method: 'GET',
      url: '/sensors',
      params: { towerId, page, pageSize },
    });
  }

  async getSensor(id: string): Promise<Sensor> {
    return this.request<Sensor>({
      method: 'GET',
      url: `/sensors/${id}`,
    });
  }

  async getSensorData(sensorId: string, startTime: string, endTime: string): Promise<any[]> {
    return this.request<any[]>({
      method: 'GET',
      url: `/sensors/${sensorId}/data`,
      params: { startTime, endTime },
    });
  }

  async getVibrationData(towerId: string, startTime: string, endTime: string): Promise<VibrationReading[]> {
    return this.request<VibrationReading[]>({
      method: 'GET',
      url: '/data/vibration',
      params: { towerId, startTime, endTime },
    });
  }

  async getWindSpeedData(towerId: string, startTime: string, endTime: string): Promise<WindSpeedReading[]> {
    return this.request<WindSpeedReading[]>({
      method: 'GET',
      url: '/data/wind-speed',
      params: { towerId, startTime, endTime },
    });
  }

  async getIceDetectionData(towerId: string, startTime: string, endTime: string): Promise<IceDetectionData[]> {
    return this.request<IceDetectionData[]>({
      method: 'GET',
      url: '/data/ice-detection',
      params: { towerId, startTime, endTime },
    });
  }

  async getWeatherData(towerId: string, startTime: string, endTime: string): Promise<WeatherData[]> {
    return this.request<WeatherData[]>({
      method: 'GET',
      url: '/data/weather',
      params: { towerId, startTime, endTime },
    });
  }

  async getAlerts(
    towerId?: string,
    status?: string,
    severity?: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<Alert>> {
    return this.request<PaginatedResponse<Alert>>({
      method: 'GET',
      url: '/alerts',
      params: { towerId, status, severity, page, pageSize },
    });
  }

  async getAlert(id: string): Promise<Alert> {
    return this.request<Alert>({
      method: 'GET',
      url: `/alerts/${id}`,
    });
  }

  async acknowledgeAlert(id: string, notes?: string): Promise<Alert> {
    return this.request<Alert>({
      method: 'POST',
      url: `/alerts/${id}/acknowledge`,
      data: { notes },
    });
  }

  async resolveAlert(id: string, notes: string): Promise<Alert> {
    return this.request<Alert>({
      method: 'POST',
      url: `/alerts/${id}/resolve`,
      data: { resolutionNotes: notes },
    });
  }

  async getAlertSummary(): Promise<{ active: number; acknowledged: number; resolved: number }> {
    return this.request<{ active: number; acknowledged: number; resolved: number }>({
      method: 'GET',
      url: '/alerts/summary',
    });
  }

  async getIceAnalysis(towerId: string, hours = 24): Promise<IceAnalysisResult[]> {
    return this.request<IceAnalysisResult[]>({
      method: 'GET',
      url: '/ice-risk/analysis',
      params: { towerId, hours },
    });
  }

  async getLatestIceAnalysis(towerId: string): Promise<IceAnalysisResult> {
    return this.request<IceAnalysisResult>({
      method: 'GET',
      url: `/ice-risk/analysis/${towerId}/latest`,
    });
  }

  async triggerIceAnalysis(towerId: string): Promise<IceAnalysisResult> {
    return this.request<IceAnalysisResult>({
      method: 'POST',
      url: '/ice-risk/analyze',
      data: { towerId },
    });
  }

  async getIcePredictions(towerId: string, hours = 24): Promise<any[]> {
    return this.request<any[]>({
      method: 'GET',
      url: '/ice-risk/predictions',
      params: { towerId, hours },
    });
  }

  async getShutdownStrategies(
    towerId?: string,
    status?: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<ShutdownStrategy>> {
    return this.request<PaginatedResponse<ShutdownStrategy>>({
      method: 'GET',
      url: '/shutdown-strategies',
      params: { towerId, status, page, pageSize },
    });
  }

  async getShutdownStrategy(id: string): Promise<ShutdownStrategy> {
    return this.request<ShutdownStrategy>({
      method: 'GET',
      url: `/shutdown-strategies/${id}`,
    });
  }

  async createShutdownStrategy(data: Partial<ShutdownStrategy>): Promise<ShutdownStrategy> {
    return this.request<ShutdownStrategy>({
      method: 'POST',
      url: '/shutdown-strategies',
      data,
    });
  }

  async triggerShutdown(id: string): Promise<ShutdownStrategy> {
    return this.request<ShutdownStrategy>({
      method: 'POST',
      url: `/shutdown-strategies/${id}/trigger`,
    });
  }

  async evaluateStrategies(towerId: string): Promise<ShutdownStrategy[]> {
    return this.request<ShutdownStrategy[]>({
      method: 'POST',
      url: '/shutdown-strategies/evaluate',
      data: { towerId },
    });
  }

  async getInspections(
    towerId?: string,
    type?: string,
    status?: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<InspectionRecord>> {
    return this.request<PaginatedResponse<InspectionRecord>>({
      method: 'GET',
      url: '/inspections',
      params: { towerId, type, status, page, pageSize },
    });
  }

  async getInspection(id: string): Promise<InspectionRecord> {
    return this.request<InspectionRecord>({
      method: 'GET',
      url: `/inspections/${id}`,
    });
  }

  async createInspection(data: Partial<InspectionRecord>): Promise<InspectionRecord> {
    return this.request<InspectionRecord>({
      method: 'POST',
      url: '/inspections',
      data,
    });
  }

  async updateInspection(id: string, data: Partial<InspectionRecord>): Promise<InspectionRecord> {
    return this.request<InspectionRecord>({
      method: 'PUT',
      url: `/inspections/${id}`,
      data,
    });
  }

  async generateInspectionReport(id: string): Promise<Blob> {
    const response = await this.axios.get(`/inspections/${id}/report`, {
      responseType: 'blob',
    });
    return response.data;
  }

  async getMaintenanceTasks(towerId?: string, status?: string, page = 1, pageSize = 20) {
    return this.request<any>({
      method: 'GET',
      url: '/inspections/maintenance-tasks',
      params: { towerId, status, page, pageSize },
    });
  }

  async getWeatherAnalysis(towerId: string, days = 7): Promise<WeatherImpactAnalysis> {
    return this.request<WeatherImpactAnalysis>({
      method: 'GET',
      url: '/weather/analysis',
      params: { towerId, days },
    });
  }

  async getWeatherAlerts(towerId?: string, activeOnly = true, page = 1, pageSize = 20): Promise<PaginatedResponse<WeatherAlert>> {
    return this.request<PaginatedResponse<WeatherAlert>>({
      method: 'GET',
      url: '/weather/alerts',
      params: { towerId, activeOnly, page, pageSize },
    });
  }

  async getWeatherForecast(towerId?: string, hours = 48): Promise<WeatherForecast> {
    return this.request<WeatherForecast>({
      method: 'GET',
      url: '/weather/forecast',
      params: { towerId, hours },
    });
  }

  async getVideoRequests(
    towerId?: string,
    status?: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<VideoVerificationRequest>> {
    return this.request<PaginatedResponse<VideoVerificationRequest>>({
      method: 'GET',
      url: '/video/requests',
      params: { towerId, status, page, pageSize },
    });
  }

  async getVideoRequest(id: string): Promise<VideoVerificationRequest> {
    return this.request<VideoVerificationRequest>({
      method: 'GET',
      url: `/video/requests/${id}`,
    });
  }

  async createVideoRequest(data: Partial<VideoVerificationRequest>): Promise<VideoVerificationRequest> {
    return this.request<VideoVerificationRequest>({
      method: 'POST',
      url: '/video/requests',
      data,
    });
  }

  async getVideoResults(
    requestId?: string,
    towerId?: string,
    page = 1,
    pageSize = 20
  ): Promise<PaginatedResponse<VideoVerificationResult>> {
    return this.request<PaginatedResponse<VideoVerificationResult>>({
      method: 'GET',
      url: '/video/results',
      params: { requestId, towerId, page, pageSize },
    });
  }

  async getVideoResult(id: string): Promise<VideoVerificationResult> {
    return this.request<VideoVerificationResult>({
      method: 'GET',
      url: `/video/results/${id}`,
    });
  }

  async getCameras(towerId?: string): Promise<Camera[]> {
    return this.request<Camera[]>({
      method: 'GET',
      url: '/video/cameras',
      params: { towerId },
    });
  }

  async startLiveStream(cameraId: string): Promise<LiveStreamSession> {
    return this.request<LiveStreamSession>({
      method: 'POST',
      url: '/video/stream/start',
      data: { cameraId },
    });
  }

  async stopLiveStream(sessionId: string): Promise<void> {
    return this.request<void>({
      method: 'POST',
      url: `/video/stream/stop/${sessionId}`,
    });
  }

  async requestAutoVerification(towerId: string, alertId?: string): Promise<VideoVerificationRequest> {
    return this.request<VideoVerificationRequest>({
      method: 'POST',
      url: '/video/auto-verify',
      data: { towerId, alertId },
    });
  }

  async getHealth(): Promise<HealthCheckResponse> {
    const response = await this.axios.get('/health');
    return response.data.data as HealthCheckResponse;
  }
}

export const apiService = new ApiService();
export default apiService;
